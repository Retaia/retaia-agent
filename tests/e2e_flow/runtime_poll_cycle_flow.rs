use std::collections::VecDeque;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, CoreApiGateway, CoreApiGatewayError,
    CoreJobState, CoreJobView, LogLevel, NotificationBridgeError, NotificationMessage,
    NotificationSink, PollEndpoint, RuntimePollCycleStatus, RuntimeSession, SystemNotification,
    run_runtime_poll_cycle,
};

fn settings() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local/api/v1".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

struct SequenceGateway {
    results: std::sync::Mutex<VecDeque<Result<Vec<CoreJobView>, CoreApiGatewayError>>>,
}

impl SequenceGateway {
    fn new(results: Vec<Result<Vec<CoreJobView>, CoreApiGatewayError>>) -> Self {
        Self {
            results: std::sync::Mutex::new(results.into()),
        }
    }
}

impl CoreApiGateway for SequenceGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError> {
        self.results
            .lock()
            .expect("lock")
            .pop_front()
            .unwrap_or_else(|| Ok(Vec::new()))
    }
}

#[derive(Default)]
struct MemorySink {
    sent: std::sync::Mutex<Vec<SystemNotification>>,
}

impl MemorySink {
    fn events(&self) -> Vec<SystemNotification> {
        self.sent.lock().expect("lock").clone()
    }
}

impl NotificationSink for MemorySink {
    fn send(
        &self,
        _message: &NotificationMessage,
        source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        self.sent.lock().expect("lock").push(source.clone());
        Ok(())
    }
}

#[test]
fn e2e_runtime_poll_cycle_handles_transport_then_successful_claimed_job_transition() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = SequenceGateway::new(vec![
        Err(CoreApiGatewayError::Transport("offline".to_string())),
        Ok(vec![CoreJobView {
            job_id: "job-100".to_string(),
            asset_uuid: "asset-100".to_string(),
            state: CoreJobState::Claimed,
            required_capabilities: vec!["media.facts@1".to_string()],
        }]),
        Ok(Vec::new()),
    ]);
    let sink = MemorySink::default();

    let first = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 1);
    let second =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 2);
    let third = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 3);

    assert_eq!(first.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(second.status, RuntimePollCycleStatus::Success);
    assert_eq!(third.status, RuntimePollCycleStatus::Success);

    let events = sink.events();
    assert!(events.contains(&SystemNotification::AgentDisconnectedOrReconnecting));
    assert!(events.contains(&SystemNotification::NewJobReceived {
        job_id: "job-100".to_string()
    }));
    assert!(events.contains(&SystemNotification::AllJobsDone));
}

#[test]
fn e2e_runtime_poll_cycle_long_sequence_keeps_notification_dedup_stable() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = SequenceGateway::new(vec![
        Ok(vec![]),
        Err(CoreApiGatewayError::Throttled),
        Err(CoreApiGatewayError::Unauthorized),
        Err(CoreApiGatewayError::Unauthorized),
        Err(CoreApiGatewayError::Transport("offline".to_string())),
        Err(CoreApiGatewayError::Transport("still-offline".to_string())),
        Ok(vec![]),
    ]);
    let sink = MemorySink::default();

    let first = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 1);
    let second =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 2);
    let third = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 3);
    let fourth =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 4);
    let fifth = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 5);
    let sixth = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 6);
    let seventh =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 7);

    assert_eq!(first.status, RuntimePollCycleStatus::Success);
    assert_eq!(second.status, RuntimePollCycleStatus::Throttled);
    assert_eq!(third.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(fourth.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(fifth.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(sixth.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(seventh.status, RuntimePollCycleStatus::Success);

    assert_eq!(first.report.expect("first report").dispatch.delivered, 0);
    assert!(second.report.is_none());
    assert_eq!(third.report.expect("third report").dispatch.delivered, 1);
    assert_eq!(fourth.report.expect("fourth report").dispatch.delivered, 0);
    assert_eq!(fifth.report.expect("fifth report").dispatch.delivered, 1);
    assert_eq!(sixth.report.expect("sixth report").dispatch.delivered, 0);
    assert_eq!(
        seventh.report.expect("seventh report").dispatch.delivered,
        0
    );

    assert_eq!(
        sink.events(),
        vec![
            SystemNotification::AuthExpiredReauthRequired,
            SystemNotification::AgentDisconnectedOrReconnecting
        ]
    );
}

#[test]
fn e2e_runtime_poll_cycle_5xx_and_429_flow_keeps_disconnect_dedup_and_backoff_signal() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = SequenceGateway::new(vec![
        Err(CoreApiGatewayError::UnexpectedStatus(503)),
        Err(CoreApiGatewayError::Throttled),
        Err(CoreApiGatewayError::UnexpectedStatus(500)),
        Ok(vec![]),
    ]);
    let sink = MemorySink::default();

    let first =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 21);
    let second =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 22);
    let third =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 23);
    let fourth =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 24);

    assert_eq!(first.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(second.status, RuntimePollCycleStatus::Throttled);
    assert_eq!(third.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(fourth.status, RuntimePollCycleStatus::Success);

    assert_eq!(first.report.expect("first report").dispatch.delivered, 1);
    assert!(second.report.is_none());
    assert_eq!(third.report.expect("third report").dispatch.delivered, 0);
    assert_eq!(fourth.report.expect("fourth report").dispatch.delivered, 0);

    assert!(format!("{:?}", second.plan).contains("SchedulePoll"));
    assert_eq!(
        sink.events(),
        vec![SystemNotification::AgentDisconnectedOrReconnecting]
    );
}

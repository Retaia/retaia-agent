use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, CoreApiGateway, CoreApiGatewayError,
    CoreJobState, CoreJobView, LogLevel, NotificationBridgeError, NotificationMessage,
    NotificationSink, PollEndpoint, RuntimePollCycleStatus, RuntimeSession, SystemNotification,
    run_runtime_poll_cycle,
};
use std::cell::RefCell;

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

struct StubGateway {
    result: Result<Vec<CoreJobView>, CoreApiGatewayError>,
}

impl CoreApiGateway for StubGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError> {
        self.result.clone()
    }
}

struct SequenceGateway {
    results: RefCell<Vec<Result<Vec<CoreJobView>, CoreApiGatewayError>>>,
}

impl SequenceGateway {
    fn new(results: Vec<Result<Vec<CoreJobView>, CoreApiGatewayError>>) -> Self {
        Self {
            results: RefCell::new(results),
        }
    }
}

impl CoreApiGateway for SequenceGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError> {
        self.results.borrow_mut().remove(0)
    }
}

#[derive(Default)]
struct NopSink;

impl NotificationSink for NopSink {
    fn send(
        &self,
        _message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        Ok(())
    }
}

#[derive(Default)]
struct RecordingSink {
    notifications: RefCell<Vec<SystemNotification>>,
}

impl NotificationSink for RecordingSink {
    fn send(
        &self,
        _message: &NotificationMessage,
        source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        self.notifications.borrow_mut().push(source.clone());
        Ok(())
    }
}

#[test]
fn tdd_runtime_poll_cycle_maps_unauthorized_to_auth_notification_snapshot() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = StubGateway {
        result: Err(CoreApiGatewayError::Unauthorized),
    };

    let outcome = run_runtime_poll_cycle(
        &mut session,
        &gateway,
        &NopSink,
        PollEndpoint::Jobs,
        5_000,
        1,
    );

    assert_eq!(outcome.status, RuntimePollCycleStatus::Degraded);
    let report = outcome.report.expect("notification report expected");
    assert_eq!(report.dispatch.delivered, 1);
    assert!(
        report
            .notifications
            .contains(&SystemNotification::AuthExpiredReauthRequired)
    );
}

#[test]
fn tdd_runtime_poll_cycle_maps_throttled_to_backoff_plan_without_notifications() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = StubGateway {
        result: Err(CoreApiGatewayError::Throttled),
    };

    let outcome = run_runtime_poll_cycle(
        &mut session,
        &gateway,
        &NopSink,
        PollEndpoint::Jobs,
        5_000,
        42,
    );

    assert_eq!(outcome.status, RuntimePollCycleStatus::Throttled);
    assert!(outcome.report.is_none());
    assert!(format!("{:?}", outcome.plan).contains("SchedulePoll"));
}

#[test]
fn tdd_runtime_poll_cycle_success_projects_claimed_job_and_dispatches_new_job_notification() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = StubGateway {
        result: Ok(vec![CoreJobView {
            job_id: "job-42".to_string(),
            asset_uuid: "asset-42".to_string(),
            state: CoreJobState::Claimed,
            required_capabilities: vec!["media.facts@1".to_string()],
        }]),
    };

    let outcome = run_runtime_poll_cycle(
        &mut session,
        &gateway,
        &NopSink,
        PollEndpoint::Jobs,
        5_000,
        2,
    );

    assert_eq!(outcome.status, RuntimePollCycleStatus::Success);
    let report = outcome.report.expect("report expected");
    assert_eq!(report.dispatch.delivered, 1);
    let status = session.status_view();
    assert_eq!(
        status.current_job.as_ref().map(|job| job.job_id.as_str()),
        Some("job-42")
    );
}

#[test]
fn tdd_runtime_poll_cycle_repeated_unauthorized_is_deduplicated_then_recovery_produces_no_auth_repeat()
 {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = SequenceGateway::new(vec![
        Err(CoreApiGatewayError::Unauthorized),
        Err(CoreApiGatewayError::Unauthorized),
        Ok(vec![]),
    ]);
    let sink = RecordingSink::default();

    let first = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 1);
    let second =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 2);
    let third = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 3);

    assert_eq!(first.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(second.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(third.status, RuntimePollCycleStatus::Success);
    assert_eq!(first.report.expect("first report").dispatch.delivered, 1);
    assert_eq!(second.report.expect("second report").dispatch.delivered, 0);
    assert_eq!(third.report.expect("third report").dispatch.delivered, 0);

    let recorded = sink.notifications.borrow();
    assert_eq!(
        recorded.as_slice(),
        &[SystemNotification::AuthExpiredReauthRequired]
    );
}

#[test]
fn tdd_runtime_poll_cycle_long_sequence_success_throttle_unauthorized_transport_deduplicates_notifications()
 {
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
    let sink = RecordingSink::default();

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

    let recorded = sink.notifications.borrow();
    assert_eq!(
        recorded.as_slice(),
        &[
            SystemNotification::AuthExpiredReauthRequired,
            SystemNotification::AgentDisconnectedOrReconnecting
        ]
    );
}

#[test]
fn tdd_runtime_poll_cycle_disconnect_notification_is_re_emitted_after_recovery_transition() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = SequenceGateway::new(vec![
        Err(CoreApiGatewayError::Transport("offline".to_string())),
        Ok(vec![]),
        Err(CoreApiGatewayError::Transport("offline-again".to_string())),
    ]);
    let sink = RecordingSink::default();

    let first = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 1);
    let second =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 2);
    let third = run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 3);

    assert_eq!(first.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(second.status, RuntimePollCycleStatus::Success);
    assert_eq!(third.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(first.report.expect("first report").dispatch.delivered, 1);
    assert_eq!(second.report.expect("second report").dispatch.delivered, 0);
    assert_eq!(third.report.expect("third report").dispatch.delivered, 1);

    let recorded = sink.notifications.borrow();
    assert_eq!(
        recorded.as_slice(),
        &[
            SystemNotification::AgentDisconnectedOrReconnecting,
            SystemNotification::AgentDisconnectedOrReconnecting
        ]
    );
}

#[test]
fn tdd_runtime_poll_cycle_5xx_then_429_then_5xx_keeps_backoff_and_disconnect_observable() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let gateway = SequenceGateway::new(vec![
        Err(CoreApiGatewayError::UnexpectedStatus(503)),
        Err(CoreApiGatewayError::Throttled),
        Err(CoreApiGatewayError::UnexpectedStatus(502)),
        Ok(vec![]),
    ]);
    let sink = RecordingSink::default();

    let first =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 11);
    let second =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 12);
    let third =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 13);
    let fourth =
        run_runtime_poll_cycle(&mut session, &gateway, &sink, PollEndpoint::Jobs, 5_000, 14);

    assert_eq!(first.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(second.status, RuntimePollCycleStatus::Throttled);
    assert_eq!(third.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(fourth.status, RuntimePollCycleStatus::Success);

    assert_eq!(first.report.expect("first report").dispatch.delivered, 1);
    assert!(second.report.is_none());
    assert_eq!(third.report.expect("third report").dispatch.delivered, 0);
    assert_eq!(fourth.report.expect("fourth report").dispatch.delivered, 0);

    // 429 remains the only signal producing explicit backoff scheduling.
    assert!(format!("{:?}", second.plan).contains("SchedulePoll"));
    assert!(format!("{:?}", first.plan).contains("SchedulePoll"));
    assert!(format!("{:?}", third.plan).contains("SchedulePoll"));

    let recorded = sink.notifications.borrow();
    assert_eq!(
        recorded.as_slice(),
        &[SystemNotification::AgentDisconnectedOrReconnecting]
    );
}

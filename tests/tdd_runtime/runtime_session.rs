use retaia_agent::{
    AgentRunState, AgentRuntimeConfig, AuthMode, CORE_JOBS_RUNTIME_FEATURE, ClientRuntimeTarget,
    CoreServerPolicy, LogLevel, MenuAction, NotificationBridgeError, NotificationMessage,
    NotificationSink, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    RuntimeSession, RuntimeSnapshot, RuntimeSyncPlan, SystemNotification, TechnicalAuthConfig,
};
use std::cell::RefCell;

fn settings() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts: std::collections::BTreeMap::new(),
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

#[test]
fn tdd_runtime_session_synchronizes_menu_state_with_runtime_loop() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    assert_eq!(session.run_state(), AgentRunState::Running);

    let paused = session.on_menu_action(MenuAction::Pause);
    assert_eq!(paused, AgentRunState::Paused);

    let running = session.on_menu_action(MenuAction::PlayResume);
    assert_eq!(running, AgentRunState::Running);
}

#[test]
fn tdd_runtime_session_routes_push_hint_and_polling_plans() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let push_plan = session.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::WebSocket,
        "hint-1",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 5_000,
        },
        1_500,
    );
    assert_eq!(
        push_plan,
        RuntimeSyncPlan::TriggerPollNow {
            endpoint: PollEndpoint::Jobs
        }
    );

    let throttled = session.on_poll_throttled(PollEndpoint::Jobs, PollSignal::SlowDown429, 2, 9);
    match throttled {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
            assert!(decision.wait_ms >= 2_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }
}

#[test]
fn tdd_runtime_session_mutation_gate_depends_on_poll_compatibility() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    assert!(!session.can_issue_mutation());

    let _ = session.on_poll_success(PollEndpoint::Jobs, 2_000, true);
    assert!(session.can_issue_mutation());

    let _ = session.on_menu_action(MenuAction::Pause);
    assert!(!session.can_issue_mutation());
}

#[test]
fn tdd_runtime_session_blocks_job_processing_until_policy_enables_runtime_jobs() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    assert!(!session.can_process_jobs());

    session.apply_server_policy(CoreServerPolicy {
        min_poll_interval_seconds: Some(9),
        feature_flags: std::collections::BTreeMap::from([(
            CORE_JOBS_RUNTIME_FEATURE.to_string(),
            true,
        )]),
    });

    assert!(session.can_process_jobs());
    assert_eq!(session.jobs_poll_interval_ms(), 9_000);
}

#[test]
fn tdd_runtime_session_enforces_jobs_poll_floor_of_five_seconds() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    session.apply_server_policy(CoreServerPolicy {
        min_poll_interval_seconds: Some(1),
        feature_flags: std::collections::BTreeMap::from([(
            CORE_JOBS_RUNTIME_FEATURE.to_string(),
            true,
        )]),
    });

    assert_eq!(session.jobs_poll_interval_ms(), 5_000);
}

#[test]
fn tdd_runtime_session_can_replace_settings_after_device_bootstrap() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let mut next = settings();
    next.auth_mode = AuthMode::Technical;
    next.technical_auth = Some(TechnicalAuthConfig {
        client_id: "agent-approved".to_string(),
        secret_key: "approved-secret".to_string(),
    });

    session.replace_settings(next).expect("settings update");

    assert_eq!(session.settings().auth_mode, AuthMode::Technical);
    assert_eq!(
        session
            .settings()
            .technical_auth
            .as_ref()
            .expect("technical auth")
            .client_id,
        "agent-approved"
    );
}

#[derive(Default)]
struct CaptureSink {
    delivered: RefCell<Vec<String>>,
}

impl NotificationSink for CaptureSink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        self.delivered.borrow_mut().push(message.title.clone());
        Ok(())
    }
}

#[test]
fn tdd_runtime_session_update_snapshot_and_dispatch_returns_report() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    let sink = CaptureSink::default();

    let mut snapshot = RuntimeSnapshot::default();
    snapshot.known_job_ids.insert("job-1".to_string());
    snapshot.running_job_ids.insert("job-1".to_string());

    let report = session.update_snapshot_and_dispatch(snapshot, &sink);
    assert_eq!(
        report.notifications,
        vec![SystemNotification::NewJobReceived {
            job_id: "job-1".to_string()
        }]
    );
    assert_eq!(report.dispatch.delivered, 1);
    assert!(report.dispatch.failed.is_empty());
    assert_eq!(sink.delivered.borrow().as_slice(), &["New job received"]);
}

use retaia_agent::{
    AgentRunState, AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, LogLevel, MenuAction,
    NotificationBridgeError, NotificationMessage, NotificationSink, PollEndpoint, PushChannel,
    PushHint, RuntimeSession, RuntimeSnapshot, RuntimeSyncPlan, SystemNotification,
};
use std::cell::RefCell;

fn config() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

#[test]
fn bdd_given_runtime_session_when_pause_menu_action_then_engine_and_ui_state_pause() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let state = session.on_menu_action(MenuAction::Pause);
    assert_eq!(state, AgentRunState::Paused);
}

#[test]
fn bdd_given_runtime_session_when_push_hint_arrives_then_immediate_poll_plan_is_emitted() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let plan = session.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::Webhook,
        "hint-701",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 3_000,
        },
        1_500,
    );
    assert_eq!(
        plan,
        RuntimeSyncPlan::TriggerPollNow {
            endpoint: PollEndpoint::Jobs
        }
    );
}

struct FailAllSink {
    calls: RefCell<u32>,
}

impl NotificationSink for FailAllSink {
    fn send(
        &self,
        _message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        *self.calls.borrow_mut() += 1;
        Err(NotificationBridgeError::SinkFailed(
            "unavailable".to_string(),
        ))
    }
}

#[test]
fn bdd_given_runtime_session_dispatch_when_sink_fails_then_notification_is_reported_failed() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let sink = FailAllSink {
        calls: RefCell::new(0),
    };

    let mut snapshot = RuntimeSnapshot::default();
    snapshot.known_job_ids.insert("job-bdd".to_string());
    snapshot.running_job_ids.insert("job-bdd".to_string());
    let report = session.update_snapshot_and_dispatch(snapshot, &sink);

    assert_eq!(report.dispatch.delivered, 0);
    assert_eq!(report.dispatch.failed.len(), 1);
    assert_eq!(*sink.calls.borrow(), 1);
}

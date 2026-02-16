use retaia_agent::{
    AgentRunState, AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, LogLevel, MenuAction,
    PollEndpoint, PushChannel, PushHint, RuntimeSession, RuntimeSyncPlan,
};

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

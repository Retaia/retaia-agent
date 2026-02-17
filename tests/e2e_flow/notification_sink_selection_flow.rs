use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, LogLevel, RuntimeSession,
    notification_sink_profile_for_target, select_notification_sink,
};

fn config() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local/api/v1".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

#[test]
fn e2e_runtime_notification_sink_selection_follows_runtime_target_policy() {
    let headless_session =
        RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let headless_profile = notification_sink_profile_for_target(headless_session.target());
    let headless_sink = select_notification_sink(headless_profile);
    assert!(format!("{:?}", headless_sink).contains("Stdout"));

    let desktop_session =
        RuntimeSession::new(ClientRuntimeTarget::UiWeb, config()).expect("session");
    let desktop_profile = notification_sink_profile_for_target(desktop_session.target());
    let desktop_sink = select_notification_sink(desktop_profile);
    assert!(format!("{:?}", desktop_sink).contains("System"));
}

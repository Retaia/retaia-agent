use retaia_agent::{
    AgentRunState, AgentRuntimeApp, AgentRuntimeConfig, AuthMode, LogLevel, MenuAction,
};

fn interactive_settings() -> AgentRuntimeConfig {
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
fn bdd_given_running_agent_when_pause_clicked_then_play_visible_pause_hidden_after_toggle() {
    let mut app = AgentRuntimeApp::new(interactive_settings()).expect("valid app");
    assert_eq!(app.run_state(), AgentRunState::Running);
    assert!(!app.tray_menu_model().visibility.show_play_resume);
    assert!(app.tray_menu_model().visibility.show_pause);

    app.apply_menu_action(MenuAction::Pause);
    assert_eq!(app.run_state(), AgentRunState::Paused);
    assert!(app.tray_menu_model().visibility.show_play_resume);
    assert!(!app.tray_menu_model().visibility.show_pause);
}

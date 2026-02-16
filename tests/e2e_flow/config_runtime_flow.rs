use retaia_agent::{
    AgentRunState, AgentRuntimeConfig, AgentUiRuntime, AuthMode, ConfigValidationError, LogLevel,
    RuntimeControlCommand, SystemNotification, apply_runtime_control, compact_validation_reason,
    validate_config,
};

#[test]
fn e2e_invalid_settings_emit_single_deduplicated_notification() {
    let mut runtime = AgentUiRuntime::new();
    let config = AgentRuntimeConfig {
        core_api_url: "api-no-scheme".to_string(),
        ollama_url: "127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 0,
        log_level: LogLevel::Warn,
    };

    let errors = validate_config(&config).expect_err("invalid config should fail");
    assert!(errors.contains(&ConfigValidationError::InvalidCoreApiUrl));
    assert!(errors.contains(&ConfigValidationError::InvalidOllamaUrl));
    assert!(errors.contains(&ConfigValidationError::InvalidMaxParallelJobs));

    let reason = compact_validation_reason(&errors);
    let first = runtime.notify_settings_invalid(&reason);
    assert_eq!(first, Some(SystemNotification::SettingsInvalid { reason }));

    let second = runtime.notify_settings_invalid(
        "invalid core api url, invalid ollama url, invalid max_parallel_jobs",
    );
    assert_eq!(second, None);
}

#[test]
fn e2e_valid_settings_then_runtime_menu_controls_execute_flow() {
    let mut runtime = AgentUiRuntime::new();
    let config = AgentRuntimeConfig {
        core_api_url: "https://api.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 4,
        log_level: LogLevel::Info,
    };
    assert_eq!(validate_config(&config), Ok(()));

    let saved = runtime.notify_settings_saved();
    assert_eq!(saved, SystemNotification::SettingsSaved);

    let paused = apply_runtime_control(AgentRunState::Running, RuntimeControlCommand::Pause);
    assert_eq!(paused, AgentRunState::Paused);

    let resumed = apply_runtime_control(paused, RuntimeControlCommand::PlayResume);
    assert_eq!(resumed, AgentRunState::Running);

    let stopped = apply_runtime_control(resumed, RuntimeControlCommand::Stop);
    assert_eq!(stopped, AgentRunState::Stopped);
}

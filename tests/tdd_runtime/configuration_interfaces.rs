use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigField, ConfigInterface, ConfigValidationError, LogLevel,
    RuntimeConfigUpdate, apply_config_update, supported_config_fields,
};

fn valid_config() -> AgentRuntimeConfig {
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
fn tdd_supported_configuration_fields_are_identical_for_gui_and_cli() {
    let gui = supported_config_fields(ConfigInterface::Gui);
    let cli = supported_config_fields(ConfigInterface::Cli);
    assert_eq!(gui, cli);
    assert!(gui.contains(&ConfigField::CoreApiUrl));
    assert!(gui.contains(&ConfigField::OllamaUrl));
    assert!(gui.contains(&ConfigField::AuthMode));
    assert!(gui.contains(&ConfigField::TechnicalClientId));
    assert!(gui.contains(&ConfigField::TechnicalSecretKey));
    assert!(gui.contains(&ConfigField::MaxParallelJobs));
    assert!(gui.contains(&ConfigField::LogLevel));
}

#[test]
fn tdd_cli_update_uses_same_validation_rules_as_gui() {
    let base = valid_config();
    let invalid = RuntimeConfigUpdate {
        core_api_url: Some("core-no-scheme".to_string()),
        ..RuntimeConfigUpdate::default()
    };

    let cli_error = apply_config_update(&base, &invalid, ConfigInterface::Cli)
        .expect_err("invalid cli update should fail");
    let gui_error = apply_config_update(&base, &invalid, ConfigInterface::Gui)
        .expect_err("invalid gui update should fail");

    assert_eq!(cli_error, gui_error);
    assert!(cli_error.contains(&ConfigValidationError::InvalidCoreApiUrl));
}

#[test]
fn tdd_cli_can_apply_full_technical_configuration() {
    let base = valid_config();
    let cli_update = RuntimeConfigUpdate {
        auth_mode: Some(AuthMode::Technical),
        technical_client_id: Some("svc-agent".to_string()),
        technical_secret_key: Some("secret-key".to_string()),
        max_parallel_jobs: Some(8),
        log_level: Some(LogLevel::Debug),
        ..RuntimeConfigUpdate::default()
    };

    let next = apply_config_update(&base, &cli_update, ConfigInterface::Cli)
        .expect("cli technical update should be valid");
    assert_eq!(next.auth_mode, AuthMode::Technical);
    assert_eq!(next.max_parallel_jobs, 8);
    assert_eq!(next.log_level, LogLevel::Debug);
    let technical = next.technical_auth.expect("technical auth should be set");
    assert_eq!(technical.client_id, "svc-agent");
    assert_eq!(technical.secret_key, "secret-key");
}

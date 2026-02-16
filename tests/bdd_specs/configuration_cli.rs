use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigInterface, ConfigValidationError, LogLevel,
    RuntimeConfigUpdate, apply_config_update,
};

fn base_config() -> AgentRuntimeConfig {
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
fn bdd_given_cli_only_host_when_updating_runtime_settings_then_same_contract_as_gui() {
    let base = base_config();
    let update = RuntimeConfigUpdate {
        ollama_url: Some("http://localhost:11434".to_string()),
        max_parallel_jobs: Some(4),
        ..RuntimeConfigUpdate::default()
    };

    let cli = apply_config_update(&base, &update, ConfigInterface::Cli).expect("cli should pass");
    let gui = apply_config_update(&base, &update, ConfigInterface::Gui).expect("gui should pass");

    assert_eq!(cli, gui);
    assert_eq!(cli.max_parallel_jobs, 4);
}

#[test]
fn bdd_given_cli_switch_to_technical_without_credentials_when_validating_then_rejected() {
    let base = base_config();
    let update = RuntimeConfigUpdate {
        auth_mode: Some(AuthMode::Technical),
        ..RuntimeConfigUpdate::default()
    };

    let errors = apply_config_update(&base, &update, ConfigInterface::Cli)
        .expect_err("technical mode without credentials must fail");
    assert!(errors.contains(&ConfigValidationError::MissingTechnicalAuth));
}

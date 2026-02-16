use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigField, ConfigInterface, LogLevel, RuntimeConfigUpdate,
    apply_config_update, supported_config_fields,
};

fn base() -> AgentRuntimeConfig {
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
fn e2e_cli_and_gui_config_paths_produce_same_final_runtime_config() {
    let gui_fields = supported_config_fields(ConfigInterface::Gui);
    let cli_fields = supported_config_fields(ConfigInterface::Cli);
    assert_eq!(gui_fields, cli_fields);
    assert!(cli_fields.contains(&ConfigField::TechnicalSecretKey));

    let base = base();
    let update = RuntimeConfigUpdate {
        core_api_url: Some("https://core.ops.local".to_string()),
        ollama_url: Some("http://10.0.0.42:11434".to_string()),
        auth_mode: Some(AuthMode::Technical),
        technical_client_id: Some("agent-prod".to_string()),
        technical_secret_key: Some("prod-secret".to_string()),
        max_parallel_jobs: Some(6),
        log_level: Some(LogLevel::Warn),
        ..RuntimeConfigUpdate::default()
    };

    let cli_final = apply_config_update(&base, &update, ConfigInterface::Cli)
        .expect("cli update should produce valid config");
    let gui_final = apply_config_update(&base, &update, ConfigInterface::Gui)
        .expect("gui update should produce valid config");

    assert_eq!(cli_final, gui_final);
    assert_eq!(cli_final.auth_mode, AuthMode::Technical);
    assert_eq!(cli_final.max_parallel_jobs, 6);
}

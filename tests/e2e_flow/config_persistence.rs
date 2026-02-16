use tempfile::tempdir;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigInterface, LogLevel, RuntimeConfigUpdate,
    apply_config_update, load_config_from_path, save_config_to_path,
};

fn defaults() -> AgentRuntimeConfig {
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
fn e2e_cli_only_configuration_is_persisted_and_restored_with_same_contract() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("config.toml");

    let cli_config = apply_config_update(
        &defaults(),
        &RuntimeConfigUpdate {
            core_api_url: Some("https://core.ops.local".to_string()),
            ollama_url: Some("http://10.0.0.42:11434".to_string()),
            max_parallel_jobs: Some(6),
            log_level: Some(LogLevel::Warn),
            ..RuntimeConfigUpdate::default()
        },
        ConfigInterface::Cli,
    )
    .expect("cli update should be valid");

    save_config_to_path(&path, &cli_config).expect("save should pass");
    let loaded = load_config_from_path(&path).expect("load should pass");

    assert_eq!(loaded, cli_config);
    assert_eq!(loaded.log_level, LogLevel::Warn);
    assert_eq!(loaded.max_parallel_jobs, 6);
}

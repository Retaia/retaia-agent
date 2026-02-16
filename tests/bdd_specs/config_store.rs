use tempfile::tempdir;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigStoreError, LogLevel, RuntimeConfigUpdate,
    apply_config_update, load_config_from_path, save_config_to_path,
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
fn bdd_given_headless_server_when_cli_saves_config_then_file_can_be_loaded() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("config.toml");
    let updated = apply_config_update(
        &base_config(),
        &RuntimeConfigUpdate {
            max_parallel_jobs: Some(4),
            ..RuntimeConfigUpdate::default()
        },
        retaia_agent::ConfigInterface::Cli,
    )
    .expect("cli update should be valid");

    save_config_to_path(&path, &updated).expect("save should pass");
    let loaded = load_config_from_path(&path).expect("load should pass");
    assert_eq!(loaded.max_parallel_jobs, 4);
}

#[test]
fn bdd_given_corrupted_config_file_when_loading_then_error_is_returned() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "not = { valid = toml").expect("write corrupt file");

    let error = load_config_from_path(&path).expect_err("invalid toml should fail");
    assert!(matches!(error, ConfigStoreError::TomlDecode(_)));
}

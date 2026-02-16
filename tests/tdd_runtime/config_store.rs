use tempfile::tempdir;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigStoreError, LogLevel, TechnicalAuthConfig,
    load_config_from_path, save_config_to_path, system_config_file_path,
};

fn valid_config() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Technical,
        technical_auth: Some(TechnicalAuthConfig {
            client_id: "agent-svc".to_string(),
            secret_key: "secret".to_string(),
        }),
        max_parallel_jobs: 3,
        log_level: LogLevel::Info,
    }
}

#[test]
fn tdd_config_store_roundtrip_preserves_runtime_config() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("agent-config.toml");
    let config = valid_config();

    save_config_to_path(&path, &config).expect("save should pass");
    let loaded = load_config_from_path(&path).expect("load should pass");
    assert_eq!(loaded, config);
}

#[test]
fn tdd_config_store_rejects_invalid_config_before_persist() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("invalid.toml");
    let mut invalid = valid_config();
    invalid.max_parallel_jobs = 0;

    let error = save_config_to_path(&path, &invalid).expect_err("validation must fail");
    match error {
        ConfigStoreError::Validation(errors) => {
            assert!(errors.iter().any(|e| matches!(
                e,
                retaia_agent::ConfigValidationError::InvalidMaxParallelJobs
            )));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn tdd_config_store_system_path_resolution_returns_config_file() {
    let path = system_config_file_path().expect("system path should resolve");
    assert!(path.ends_with("config.toml"));
}

use tempfile::tempdir;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigRepository, FileConfigRepository, LogLevel,
    TechnicalAuthConfig, load_config_from_path,
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
        log_level: LogLevel::Debug,
    }
}

#[test]
fn tdd_file_config_repository_implements_port_roundtrip() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("config.toml");
    let repository = FileConfigRepository::new(path.clone());
    let config = valid_config();

    repository.save(&config).expect("save should pass");
    let loaded = repository.load().expect("load should pass");
    assert_eq!(loaded, config);

    let raw_loaded = load_config_from_path(&path).expect("direct loader should match");
    assert_eq!(raw_loaded, config);
}

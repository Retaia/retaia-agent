use tempfile::tempdir;

use retaia_agent::{
    AgentRuntimeApp, AgentRuntimeConfig, AuthMode, ConfigRepository, FileConfigRepository, LogLevel,
};

fn config() -> AgentRuntimeConfig {
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
fn bdd_given_app_settings_save_when_using_repository_port_then_file_is_persisted() {
    let dir = tempdir().expect("temp dir");
    let repository = FileConfigRepository::new(dir.path().join("config.toml"));
    let mut app = AgentRuntimeApp::new(config()).expect("valid app");

    app.save_settings_with_repository(config(), &repository)
        .expect("save via repository should pass");

    let loaded = repository
        .load()
        .expect("repository should load persisted config");
    assert_eq!(loaded.ollama_url, "http://127.0.0.1:11434");
}

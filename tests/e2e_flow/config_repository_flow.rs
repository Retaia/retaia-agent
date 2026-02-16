use tempfile::tempdir;

use retaia_agent::{
    AgentRuntimeApp, AgentRuntimeConfig, AuthMode, ConfigInterface, ConfigRepository,
    FileConfigRepository, LogLevel, RuntimeConfigUpdate, apply_config_update,
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
fn e2e_repository_port_allows_reloading_persisted_settings_into_new_app_instance() {
    let dir = tempdir().expect("temp dir");
    let repository = FileConfigRepository::new(dir.path().join("config.toml"));
    let mut app = AgentRuntimeApp::new(base()).expect("valid app");

    let updated = apply_config_update(
        app.settings(),
        &RuntimeConfigUpdate {
            ollama_url: Some("http://10.1.2.3:11434".to_string()),
            log_level: Some(LogLevel::Warn),
            max_parallel_jobs: Some(5),
            ..RuntimeConfigUpdate::default()
        },
        ConfigInterface::Cli,
    )
    .expect("update should be valid");

    app.save_settings_with_repository(updated.clone(), &repository)
        .expect("save should pass");

    let reloaded = AgentRuntimeApp::load_from_repository(&repository).expect("load should pass");
    assert_eq!(reloaded.settings(), &updated);

    let persisted = repository.load().expect("repository should have file");
    assert_eq!(persisted.max_parallel_jobs, 5);
}

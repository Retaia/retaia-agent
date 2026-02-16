use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigValidationError, LogLevel, TechnicalAuthConfig,
    compact_validation_reason, validate_config,
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
fn tdd_configuration_accepts_valid_interactive_setup() {
    let config = valid_config();
    assert_eq!(validate_config(&config), Ok(()));
}

#[test]
fn tdd_configuration_rejects_invalid_urls_and_zero_parallelism() {
    let mut config = valid_config();
    config.core_api_url = "not-a-url".to_string();
    config.ollama_url = "ollama.local".to_string();
    config.max_parallel_jobs = 0;

    let errors = validate_config(&config).expect_err("expected validation errors");
    assert!(errors.contains(&ConfigValidationError::InvalidCoreApiUrl));
    assert!(errors.contains(&ConfigValidationError::InvalidOllamaUrl));
    assert!(errors.contains(&ConfigValidationError::InvalidMaxParallelJobs));
}

#[test]
fn tdd_configuration_requires_technical_credentials_in_technical_mode() {
    let mut config = valid_config();
    config.auth_mode = AuthMode::Technical;
    config.technical_auth = None;

    let errors = validate_config(&config).expect_err("missing technical auth should fail");
    assert!(errors.contains(&ConfigValidationError::MissingTechnicalAuth));
}

#[test]
fn tdd_configuration_rejects_empty_technical_fields() {
    let mut config = valid_config();
    config.auth_mode = AuthMode::Technical;
    config.technical_auth = Some(TechnicalAuthConfig {
        client_id: " ".to_string(),
        secret_key: "".to_string(),
    });

    let errors = validate_config(&config).expect_err("empty technical fields should fail");
    assert!(errors.contains(&ConfigValidationError::EmptyClientId));
    assert!(errors.contains(&ConfigValidationError::EmptySecretKey));
}

#[test]
fn tdd_compact_validation_reason_produces_human_readable_string() {
    let reason = compact_validation_reason(&[
        ConfigValidationError::InvalidCoreApiUrl,
        ConfigValidationError::EmptySecretKey,
    ]);
    assert_eq!(reason, "invalid core api url, empty secret key");
}

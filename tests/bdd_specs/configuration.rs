use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigValidationError, LogLevel, TechnicalAuthConfig,
    validate_config,
};

fn base_settings() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://api.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

#[test]
fn bdd_given_technical_mode_without_credentials_when_validating_then_rejected() {
    let mut config = base_settings();
    config.auth_mode = AuthMode::Technical;

    let errors = validate_config(&config).expect_err("technical mode should require credentials");
    assert!(errors.contains(&ConfigValidationError::MissingTechnicalAuth));
}

#[test]
fn bdd_given_interactive_mode_when_technical_payload_missing_then_still_valid() {
    let config = base_settings();
    assert_eq!(validate_config(&config), Ok(()));
}

#[test]
fn bdd_given_technical_mode_with_blank_secrets_when_validating_then_rejected() {
    let mut config = base_settings();
    config.auth_mode = AuthMode::Technical;
    config.technical_auth = Some(TechnicalAuthConfig {
        client_id: " ".to_string(),
        secret_key: "".to_string(),
    });

    let errors = validate_config(&config).expect_err("blank technical fields should fail");
    assert!(errors.contains(&ConfigValidationError::EmptyClientId));
    assert!(errors.contains(&ConfigValidationError::EmptySecretKey));
}

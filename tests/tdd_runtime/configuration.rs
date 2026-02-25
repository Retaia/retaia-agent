use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigValidationError, LogLevel, TechnicalAuthConfig,
    compact_validation_reason, normalize_core_api_url, validate_config,
};

fn valid_config() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts: std::collections::BTreeMap::new(),
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
        ConfigValidationError::CoreApiUrlDockerHostnameForbidden,
        ConfigValidationError::EmptySecretKey,
    ]);
    assert_eq!(
        reason,
        "invalid core api url, core api url uses forbidden docker-internal hostname, empty secret key"
    );
}

#[test]
fn tdd_normalize_core_api_url_accepts_host_and_api_v1_with_or_without_trailing_slash() {
    assert_eq!(
        normalize_core_api_url("https://core.retaia.local"),
        "https://core.retaia.local/api/v1"
    );
    assert_eq!(
        normalize_core_api_url("https://core.retaia.local/api/v1"),
        "https://core.retaia.local/api/v1"
    );
    assert_eq!(
        normalize_core_api_url("https://core.retaia.local/api/v1/"),
        "https://core.retaia.local/api/v1"
    );
}

#[test]
fn tdd_configuration_accepts_routable_core_api_url_for_http_and_https() {
    for scheme in ["http", "https"] {
        let mut config = valid_config();
        config.core_api_url = format!("{scheme}://192.168.0.14:8080/api/v1");
        assert_eq!(validate_config(&config), Ok(()));

        config.core_api_url = format!("{scheme}://retaia.local/api/v1");
        assert_eq!(validate_config(&config), Ok(()));
    }
}

#[test]
fn tdd_configuration_rejects_docker_internal_core_hostname_for_http_and_https() {
    for scheme in ["http", "https"] {
        for host in ["core:8000", "app-prod:9000"] {
            let mut config = valid_config();
            config.core_api_url = format!("{scheme}://{host}/api/v1");
            let errors = validate_config(&config).expect_err("docker internal hostname must fail");
            assert!(
                errors.contains(&ConfigValidationError::CoreApiUrlDockerHostnameForbidden),
                "missing forbidden-hostname error for url={}",
                config.core_api_url
            );
        }
    }
}

#[test]
fn tdd_configuration_rejects_non_absolute_storage_mount_paths() {
    let mut config = valid_config();
    config
        .storage_mounts
        .insert("nas-main".to_string(), "mnt/nas".to_string());

    let errors = validate_config(&config).expect_err("relative mount path must fail");
    assert!(
        errors.contains(&ConfigValidationError::StorageMountPathNotAbsolute(
            "nas-main".to_string()
        ))
    );
}

#[test]
fn tdd_configuration_rejects_empty_storage_mount_id() {
    let mut config = valid_config();
    config
        .storage_mounts
        .insert("   ".to_string(), "/mnt/nas".to_string());

    let errors = validate_config(&config).expect_err("empty storage id must fail");
    assert!(errors.contains(&ConfigValidationError::EmptyStorageMountId));
}

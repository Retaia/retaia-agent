use std::sync::{Mutex, OnceLock};

use tempfile::tempdir;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigStoreError, LogLevel, TechnicalAuthConfig,
    load_config_from_path, save_config_to_path, system_config_file_path,
};

fn env_guard() -> &'static Mutex<()> {
    static ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_GUARD.get_or_init(|| Mutex::new(()))
}

fn use_memory_secret_store() {
    unsafe {
        std::env::set_var("RETAIA_AGENT_SECRET_STORE_BACKEND", "memory");
        std::env::remove_var("RETAIA_AGENT_SECRET_STORE_FILE");
    }
}

fn valid_config() -> AgentRuntimeConfig {
    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert("nas-main".to_string(), "/mnt/nas/main".to_string());

    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Technical,
        technical_auth: Some(TechnicalAuthConfig {
            client_id: "agent-svc".to_string(),
            secret_key: "secret".to_string(),
        }),
        storage_mounts,
        max_parallel_jobs: 3,
        log_level: LogLevel::Info,
    }
}

#[test]
fn tdd_config_store_roundtrip_preserves_runtime_config() {
    let _guard = env_guard().lock().expect("env guard");
    use_memory_secret_store();
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("agent-config.toml");
    let config = valid_config();

    save_config_to_path(&path, &config).expect("save should pass");
    let raw = std::fs::read_to_string(&path).expect("raw config");
    assert!(!raw.contains("secret_key"));
    assert!(raw.contains("client_id = \"agent-svc\""));
    let loaded = load_config_from_path(&path).expect("load should pass");
    assert_eq!(loaded, config);
}

#[test]
fn tdd_config_store_rejects_invalid_config_before_persist() {
    let _guard = env_guard().lock().expect("env guard");
    use_memory_secret_store();
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

#[test]
fn tdd_config_store_loads_legacy_toml_without_storage_mounts() {
    let _guard = env_guard().lock().expect("env guard");
    use_memory_secret_store();
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("legacy.toml");
    std::fs::write(
        &path,
        r#"
core_api_url = "https://core.retaia.local/api/v1"
ollama_url = "http://127.0.0.1:11434"
auth_mode = "interactive"
max_parallel_jobs = 2
log_level = "info"
"#,
    )
    .expect("write legacy config");

    let loaded = load_config_from_path(&path).expect("legacy config should load");
    assert!(loaded.storage_mounts.is_empty());
}

#[test]
fn tdd_config_store_migrates_legacy_inline_technical_secret_out_of_toml() {
    let _guard = env_guard().lock().expect("env guard");
    use_memory_secret_store();
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("legacy-technical.toml");
    std::fs::write(
        &path,
        r#"
core_api_url = "https://core.retaia.local/api/v1"
ollama_url = "http://127.0.0.1:11434"
auth_mode = "technical"
max_parallel_jobs = 2
log_level = "info"

[technical_auth]
client_id = "agent-svc"
secret_key = "legacy-secret"
"#,
    )
    .expect("write legacy config");

    let loaded = load_config_from_path(&path).expect("legacy technical config should load");
    let technical = loaded.technical_auth.expect("technical auth");
    assert_eq!(technical.client_id, "agent-svc");
    assert_eq!(technical.secret_key, "legacy-secret");

    let raw = std::fs::read_to_string(&path).expect("raw config");
    assert!(raw.contains("client_id = \"agent-svc\""));
    assert!(!raw.contains("legacy-secret"));
    assert!(!raw.contains("secret_key"));
}

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, LogLevel, SourcePathResolveError, resolve_processing_input_path,
    resolve_source_path,
};

fn config_with_mount() -> AgentRuntimeConfig {
    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert("nas-main".to_string(), "/mnt/nas/main".to_string());

    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts,
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    }
}

#[test]
fn tdd_resolve_source_path_joins_storage_mount_and_relative_path() {
    let config = config_with_mount();
    let resolved = resolve_source_path(&config, "nas-main", "INBOX/2026/asset.mp4").expect("path");
    assert_eq!(
        resolved,
        std::path::PathBuf::from("/mnt/nas/main").join("INBOX/2026/asset.mp4")
    );
}

#[test]
fn tdd_resolve_source_path_rejects_unknown_storage_id() {
    let config = config_with_mount();
    let error = resolve_source_path(&config, "missing", "INBOX/asset.mp4").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::UnknownStorageId("missing".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_parent_dir_relative_path() {
    let config = config_with_mount();
    let error = resolve_source_path(&config, "nas-main", "../etc/passwd").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("../etc/passwd".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_absolute_relative_path() {
    let config = config_with_mount();
    let error = resolve_source_path(&config, "nas-main", "/etc/passwd").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("/etc/passwd".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_null_byte() {
    let config = config_with_mount();
    let error = resolve_source_path(&config, "nas-main", "INBOX/a\0b").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("INBOX/a\0b".to_string())
    );
}

#[test]
fn tdd_resolve_processing_input_path_returns_explicit_error_without_panic() {
    let config = config_with_mount();
    let error = resolve_processing_input_path(&config, "unknown", "INBOX/asset.mp4")
        .expect_err("must fail");
    assert!(error.to_string().contains("unable to resolve source path"));
}

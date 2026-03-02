use retaia_agent::{
    AgentRuntimeConfig, AuthMode, LogLevel, SourcePathResolveError, resolve_processing_input_path,
    resolve_source_path,
};

fn write_storage_marker(root: &std::path::Path, storage_id: &str) {
    let marker = format!(
        r#"{{"version":1,"storage_id":"{storage_id}","paths":{{"inbox":"INBOX","archive":"ARCHIVE","rejects":"REJECTS"}}}}"#
    );
    std::fs::write(root.join(".retaia"), marker).expect("write marker");
}

fn config_with_mount() -> (AgentRuntimeConfig, tempfile::TempDir) {
    let mount = tempfile::tempdir().expect("temp mount");
    write_storage_marker(mount.path(), "nas-main");
    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert(
        "nas-main".to_string(),
        mount.path().display().to_string(),
    );

    (
        AgentRuntimeConfig {
            core_api_url: "https://core.retaia.local".to_string(),
            ollama_url: "http://127.0.0.1:11434".to_string(),
            auth_mode: AuthMode::Interactive,
            technical_auth: None,
            storage_mounts,
            max_parallel_jobs: 1,
            log_level: LogLevel::Info,
        },
        mount,
    )
}

#[test]
fn tdd_resolve_source_path_joins_storage_mount_and_relative_path() {
    let (config, mount) = config_with_mount();
    let resolved = resolve_source_path(&config, "nas-main", "INBOX/2026/asset.mp4").expect("path");
    assert_eq!(
        resolved,
        mount.path().join("INBOX/2026/asset.mp4")
    );
}

#[test]
fn tdd_resolve_source_path_rejects_unknown_storage_id() {
    let (config, _mount) = config_with_mount();
    let error = resolve_source_path(&config, "missing", "INBOX/asset.mp4").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::UnknownStorageId("missing".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_parent_dir_relative_path() {
    let (config, _mount) = config_with_mount();
    let error = resolve_source_path(&config, "nas-main", "../etc/passwd").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("../etc/passwd".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_absolute_relative_path() {
    let (config, _mount) = config_with_mount();
    let error = resolve_source_path(&config, "nas-main", "/etc/passwd").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("/etc/passwd".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_null_byte() {
    let (config, _mount) = config_with_mount();
    let error = resolve_source_path(&config, "nas-main", "INBOX/a\0b").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("INBOX/a\0b".to_string())
    );
}

#[test]
fn tdd_resolve_processing_input_path_returns_explicit_error_without_panic() {
    let (config, _mount) = config_with_mount();
    let error = resolve_processing_input_path(&config, "unknown", "INBOX/asset.mp4")
        .expect_err("must fail");
    assert!(error.to_string().contains("unable to resolve source path"));
}

#[test]
fn tdd_resolve_source_path_rejects_missing_storage_marker() {
    let mount = tempfile::tempdir().expect("temp mount");
    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert(
        "nas-main".to_string(),
        mount.path().display().to_string(),
    );
    let config = AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts,
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    };

    let error = resolve_source_path(&config, "nas-main", "INBOX/a.mp4").expect_err("must fail");
    assert!(matches!(
        error,
        SourcePathResolveError::StorageMarkerMissing(_)
    ));
}

#[test]
fn tdd_resolve_source_path_rejects_marker_storage_id_mismatch() {
    let mount = tempfile::tempdir().expect("temp mount");
    write_storage_marker(mount.path(), "nas-other");
    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert(
        "nas-main".to_string(),
        mount.path().display().to_string(),
    );
    let config = AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts,
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    };

    let error = resolve_source_path(&config, "nas-main", "INBOX/a.mp4").expect_err("must fail");
    assert_eq!(
        error,
        SourcePathResolveError::StorageMarkerStorageIdMismatch {
            expected: "nas-main".to_string(),
            actual: "nas-other".to_string(),
        }
    );
}

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, UNIX_EPOCH};

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, LogLevel, SourcePathResolveError, StorageMarkerProvider,
    StorageMarkerRead, resolve_processing_input_path, resolve_source_path_with_marker_provider,
};

#[derive(Debug, Default)]
struct FakeStorageMarkerProvider {
    markers: HashMap<String, StorageMarkerRead>,
}

impl FakeStorageMarkerProvider {
    fn with_marker(mut self, mount: &str, marker_json: String) -> Self {
        let marker_path = Path::new(mount).join(".retaia");
        static NEXT_TICK: AtomicU64 = AtomicU64::new(1);
        let tick = NEXT_TICK.fetch_add(1, Ordering::Relaxed);
        self.markers.insert(
            marker_path.display().to_string(),
            StorageMarkerRead {
                modified_at: UNIX_EPOCH + Duration::from_secs(tick),
                contents: marker_json,
            },
        );
        self
    }
}

impl StorageMarkerProvider for FakeStorageMarkerProvider {
    fn read_marker(&self, marker_path: &Path) -> Result<StorageMarkerRead, SourcePathResolveError> {
        self.markers
            .get(&marker_path.display().to_string())
            .cloned()
            .ok_or_else(|| {
                SourcePathResolveError::StorageMarkerMissing(marker_path.display().to_string())
            })
    }
}

fn marker_json(storage_id: &str, version: u64) -> String {
    format!(
        r#"{{"version":{version},"storage_id":"{storage_id}","paths":{{"inbox":"INBOX","archive":"ARCHIVE","rejects":"REJECTS"}}}}"#
    )
}

fn config_with_mount(mount: &str) -> AgentRuntimeConfig {
    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert("nas-main".to_string(), mount.to_string());

    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local/api/v1".to_string(),
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
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider =
        FakeStorageMarkerProvider::default().with_marker(mount, marker_json("nas-main", 1));

    let resolved = resolve_source_path_with_marker_provider(
        &config,
        "nas-main",
        "INBOX/2026/asset.mp4",
        &provider,
    )
    .expect("path");

    assert_eq!(
        resolved,
        std::path::PathBuf::from(mount).join("INBOX/2026/asset.mp4")
    );
}

#[test]
fn tdd_resolve_source_path_rejects_unknown_storage_id() {
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider =
        FakeStorageMarkerProvider::default().with_marker(mount, marker_json("nas-main", 1));

    let error =
        resolve_source_path_with_marker_provider(&config, "missing", "INBOX/asset.mp4", &provider)
            .expect_err("must fail");

    assert_eq!(
        error,
        SourcePathResolveError::UnknownStorageId("missing".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_parent_dir_relative_path() {
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider =
        FakeStorageMarkerProvider::default().with_marker(mount, marker_json("nas-main", 1));

    let error =
        resolve_source_path_with_marker_provider(&config, "nas-main", "../etc/passwd", &provider)
            .expect_err("must fail");

    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("../etc/passwd".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_absolute_relative_path() {
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider =
        FakeStorageMarkerProvider::default().with_marker(mount, marker_json("nas-main", 1));

    let error =
        resolve_source_path_with_marker_provider(&config, "nas-main", "/etc/passwd", &provider)
            .expect_err("must fail");

    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("/etc/passwd".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_rejects_null_byte() {
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider =
        FakeStorageMarkerProvider::default().with_marker(mount, marker_json("nas-main", 1));

    let error =
        resolve_source_path_with_marker_provider(&config, "nas-main", "INBOX/a\0b", &provider)
            .expect_err("must fail");

    assert_eq!(
        error,
        SourcePathResolveError::UnsafeRelativePath("INBOX/a\0b".to_string())
    );
}

#[test]
fn tdd_resolve_processing_input_path_returns_explicit_error_without_panic() {
    let config = config_with_mount("/mnt/nas/main");
    let error = resolve_processing_input_path(&config, "unknown", "INBOX/asset.mp4")
        .expect_err("must fail");
    assert!(error.to_string().contains("unable to resolve source path"));
}

#[test]
fn tdd_resolve_source_path_rejects_missing_storage_marker() {
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider = FakeStorageMarkerProvider::default();

    let error =
        resolve_source_path_with_marker_provider(&config, "nas-main", "INBOX/a.mp4", &provider)
            .expect_err("must fail");
    assert!(matches!(
        error,
        SourcePathResolveError::StorageMarkerMissing(_)
    ));
}

#[test]
fn tdd_resolve_source_path_rejects_marker_storage_id_mismatch() {
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider =
        FakeStorageMarkerProvider::default().with_marker(mount, marker_json("nas-other", 1));

    let error =
        resolve_source_path_with_marker_provider(&config, "nas-main", "INBOX/a.mp4", &provider)
            .expect_err("must fail");

    assert_eq!(
        error,
        SourcePathResolveError::StorageMarkerStorageIdMismatch {
            expected: "nas-main".to_string(),
            actual: "nas-other".to_string(),
        }
    );
}

#[test]
fn tdd_resolve_source_path_rejects_non_inbox_for_marker_v1() {
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider =
        FakeStorageMarkerProvider::default().with_marker(mount, marker_json("nas-main", 1));

    let error =
        resolve_source_path_with_marker_provider(&config, "nas-main", "ARCHIVE/a.mp4", &provider)
            .expect_err("must fail");

    assert_eq!(
        error,
        SourcePathResolveError::PathOutsideMarkerRoots("ARCHIVE/a.mp4".to_string())
    );
}

#[test]
fn tdd_resolve_source_path_allows_non_inbox_for_marker_v2() {
    let mount = "/mnt/nas/main";
    let config = config_with_mount(mount);
    let provider =
        FakeStorageMarkerProvider::default().with_marker(mount, marker_json("nas-main", 2));

    let resolved =
        resolve_source_path_with_marker_provider(&config, "nas-main", "ARCHIVE/a.mp4", &provider)
            .expect("must pass");
    assert_eq!(
        resolved,
        std::path::PathBuf::from(mount).join("ARCHIVE/a.mp4")
    );
}

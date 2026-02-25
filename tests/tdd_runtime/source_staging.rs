use std::path::Path;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClaimedDerivedJob, DerivedJobType, DiskSpaceProbe,
    Fs2DiskSpaceProbe, LogLevel, SourceStagingError, stage_claimed_job_source,
    stage_claimed_job_source_with_probe,
};

fn config_with_mount(mount_path: &Path) -> AgentRuntimeConfig {
    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert("nas-main".to_string(), mount_path.display().to_string());

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

fn claimed_job(relative: &str) -> ClaimedDerivedJob {
    ClaimedDerivedJob {
        job_id: "job-1".to_string(),
        asset_uuid: "asset-1".to_string(),
        lock_token: "lock-1".to_string(),
        job_type: DerivedJobType::GenerateProxy,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: relative.to_string(),
        source_sidecars_relative: Vec::new(),
    }
}

#[test]
fn tdd_source_staging_copies_source_to_local_temp_and_cleans_up_on_drop() {
    let source_dir = tempfile::tempdir().expect("source dir");
    let source_rel = "INBOX/clip.mp4";
    let source_path = source_dir.path().join(source_rel);
    std::fs::create_dir_all(source_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&source_path, b"video-bytes").expect("write source");

    let config = config_with_mount(source_dir.path());

    let staged_path = {
        let staged = stage_claimed_job_source(&config, &claimed_job(source_rel)).expect("stage");
        let bytes = std::fs::read(staged.path()).expect("read staged");
        assert_eq!(bytes, b"video-bytes");
        staged.path().to_path_buf()
    };

    assert!(
        !staged_path.exists(),
        "temp folder must be removed when staged file is dropped"
    );
}

#[test]
fn tdd_source_staging_copies_sidecars_to_local_temp_and_cleans_up_on_drop() {
    let source_dir = tempfile::tempdir().expect("source dir");
    let source_rel = "INBOX/clip.mp4";
    let sidecar_a_rel = "INBOX/clip.xmp";
    let sidecar_b_rel = "INBOX/clip.srt";
    for (relative, bytes) in [
        (source_rel, b"video-bytes".as_slice()),
        (sidecar_a_rel, b"xmp-bytes".as_slice()),
        (sidecar_b_rel, b"srt-bytes".as_slice()),
    ] {
        let path = source_dir.path().join(relative);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, bytes).expect("write");
    }

    let mut claim = claimed_job(source_rel);
    claim.source_sidecars_relative = vec![sidecar_a_rel.to_string(), sidecar_b_rel.to_string()];
    let config = config_with_mount(source_dir.path());

    let (staged_main, staged_sidecars) = {
        let staged = stage_claimed_job_source(&config, &claim).expect("stage");
        assert_eq!(
            std::fs::read(staged.path()).expect("read staged main"),
            b"video-bytes"
        );
        let sidecars = staged.sidecar_paths().to_vec();
        assert_eq!(sidecars.len(), 2);
        let staged_payloads = sidecars
            .iter()
            .map(|path| std::fs::read(path).expect("read staged sidecar"))
            .collect::<Vec<_>>();
        assert_eq!(
            staged_payloads,
            vec![b"xmp-bytes".to_vec(), b"srt-bytes".to_vec()]
        );
        (staged.path().to_path_buf(), sidecars)
    };

    assert!(
        !staged_main.exists(),
        "staged source should be cleaned on drop"
    );
    for path in staged_sidecars {
        assert!(!path.exists(), "staged sidecar should be cleaned on drop");
    }
}

#[test]
fn tdd_source_staging_preserves_relative_paths_to_avoid_sidecar_name_collisions() {
    let source_dir = tempfile::tempdir().expect("source dir");
    let source_rel = "INBOX/clip.mp4";
    let sidecar_a_rel = "INBOX/cam-a/clip.xmp";
    let sidecar_b_rel = "INBOX/cam-b/clip.xmp";
    for (relative, bytes) in [
        (source_rel, b"video-bytes".as_slice()),
        (sidecar_a_rel, b"xmp-a".as_slice()),
        (sidecar_b_rel, b"xmp-b".as_slice()),
    ] {
        let path = source_dir.path().join(relative);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, bytes).expect("write");
    }

    let mut claim = claimed_job(source_rel);
    claim.source_sidecars_relative = vec![sidecar_a_rel.to_string(), sidecar_b_rel.to_string()];
    let config = config_with_mount(source_dir.path());

    let staged = stage_claimed_job_source(&config, &claim).expect("stage");
    let staged_source = staged.path().to_path_buf();
    assert!(
        staged_source.to_string_lossy().ends_with("INBOX/clip.mp4"),
        "source should keep relative path in staging: {}",
        staged_source.display()
    );
    let sidecars = staged.sidecar_paths().to_vec();
    assert_eq!(sidecars.len(), 2);
    assert_ne!(sidecars[0], sidecars[1], "sidecars must not overwrite each other");
    assert_eq!(
        std::fs::read(&sidecars[0]).expect("read sidecar A"),
        b"xmp-a"
    );
    assert_eq!(
        std::fs::read(&sidecars[1]).expect("read sidecar B"),
        b"xmp-b"
    );
}

#[derive(Debug, Clone, Copy)]
struct ZeroSpaceProbe;

impl DiskSpaceProbe for ZeroSpaceProbe {
    fn available_space(&self, _path: &Path) -> Result<u64, SourceStagingError> {
        Ok(0)
    }
}

#[test]
fn tdd_source_staging_rejects_when_available_disk_space_is_insufficient() {
    let source_dir = tempfile::tempdir().expect("source dir");
    let source_rel = "INBOX/clip.mp4";
    let source_path = source_dir.path().join(source_rel);
    std::fs::create_dir_all(source_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&source_path, b"video-bytes").expect("write source");

    let config = config_with_mount(source_dir.path());
    let error =
        stage_claimed_job_source_with_probe(&config, &claimed_job(source_rel), &ZeroSpaceProbe)
            .expect_err("must fail");

    assert!(matches!(
        error,
        SourceStagingError::InsufficientDiskSpace {
            required_bytes: _,
            available_bytes: 0
        }
    ));
}

#[test]
fn tdd_source_staging_rejects_when_available_disk_space_is_insufficient_with_sidecars() {
    let source_dir = tempfile::tempdir().expect("source dir");
    let source_rel = "INBOX/clip.mp4";
    let sidecar_rel = "INBOX/clip.xmp";
    let source_path = source_dir.path().join(source_rel);
    let sidecar_path = source_dir.path().join(sidecar_rel);
    std::fs::create_dir_all(source_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&source_path, b"video-bytes").expect("write source");
    std::fs::write(&sidecar_path, b"xmp-bytes").expect("write sidecar");

    let config = config_with_mount(source_dir.path());
    let mut claim = claimed_job(source_rel);
    claim.source_sidecars_relative = vec![sidecar_rel.to_string()];
    let error = stage_claimed_job_source_with_probe(&config, &claim, &ZeroSpaceProbe)
        .expect_err("must fail");

    assert!(matches!(
        error,
        SourceStagingError::InsufficientDiskSpace {
            required_bytes: _,
            available_bytes: 0
        }
    ));
}

#[test]
fn tdd_source_staging_rejects_when_storage_mapping_is_missing() {
    let mut config = config_with_mount(Path::new("/tmp"));
    config.storage_mounts.clear();
    let error =
        stage_claimed_job_source(&config, &claimed_job("INBOX/clip.mp4")).expect_err("must fail");

    assert!(matches!(error, SourceStagingError::ResolvePath(_)));
}

#[test]
fn tdd_source_staging_real_disk_probe_reads_available_space() {
    let probe = Fs2DiskSpaceProbe;
    let temp_dir = std::env::temp_dir();
    let available = probe.available_space(&temp_dir).expect("space");
    assert!(available > 0);
}

use std::sync::Mutex;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClaimedDerivedJob, DerivedExecutionPlan, DerivedExecutionPlanner,
    DerivedJobExecutorError, DerivedJobType, DerivedKind, DerivedManifestItem,
    DerivedProcessingError, DerivedProcessingGateway, DerivedUploadComplete, DerivedUploadInit,
    DerivedUploadPart, HeartbeatReceipt, LogLevel, RuntimeDerivedPlanner, SubmitDerivedPayload,
    UploadedDerivedPart, execute_derived_job_once, execute_derived_job_once_with_source_staging,
};

fn write_storage_marker(root: &std::path::Path, storage_id: &str) {
    let marker = format!(
        r#"{{"version":1,"storage_id":"{storage_id}","paths":{{"inbox":"INBOX","archive":"ARCHIVE","rejects":"REJECTS"}}}}"#
    );
    std::fs::write(root.join(".retaia"), marker).expect("write marker");
}

#[derive(Default)]
struct MemoryGateway {
    calls: Mutex<Vec<String>>,
}

impl MemoryGateway {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("calls").clone()
    }
}

impl DerivedProcessingGateway for MemoryGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("claim:{job_id}"));
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-1".to_string(),
            lock_token: "lock-1".to_string(),
            fencing_token: 1,
            job_type: DerivedJobType::GeneratePreview,
            source_storage_id: "nas-main".to_string(),
            source_original_relative: "INBOX/sample-source.bin".to_string(),
            source_sidecars_relative: Vec::new(),
        })
    }

    fn heartbeat(
        &self,
        job_id: &str,
        _lock_token: &str,
        _fencing_token: i32,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("heartbeat:{job_id}"));
        Ok(HeartbeatReceipt {
            locked_until: None,
            fencing_token: 1,
        })
    }

    fn submit_derived(
        &self,
        job_id: &str,
        _lock_token: &str,
        _fencing_token: i32,
        _idempotency_key: &str,
        _payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("submit:{job_id}"));
        Ok(())
    }

    fn upload_init(&self, request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("upload_init:{}", request.asset_uuid));
        Ok(())
    }

    fn upload_part(
        &self,
        request: &DerivedUploadPart,
    ) -> Result<UploadedDerivedPart, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("upload_part:{}", request.part_number));
        Ok(UploadedDerivedPart {
            part_number: request.part_number,
            part_etag: format!("etag-{}", request.part_number),
        })
    }

    fn upload_complete(
        &self,
        request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("upload_complete:{}", request.asset_uuid));
        Ok(())
    }
}

struct ProxyPlanner;

impl DerivedExecutionPlanner for ProxyPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![retaia_agent::DerivedUploadPlan {
                init: DerivedUploadInit {
                    asset_uuid: claimed.asset_uuid.clone(),
                    kind: DerivedKind::PreviewVideo,
                    content_type: "video/mp4".to_string(),
                    size_bytes: 1024,
                    sha256: None,
                    idempotency_key: "idem-init".to_string(),
                },
                parts: vec![DerivedUploadPart {
                    asset_uuid: claimed.asset_uuid.clone(),
                    upload_id: "up-1".to_string(),
                    part_number: 1,
                    chunk_path: std::path::PathBuf::from("/tmp/up-1.bin"),
                }],
                complete: DerivedUploadComplete {
                    asset_uuid: claimed.asset_uuid.clone(),
                    upload_id: "up-1".to_string(),
                    idempotency_key: "idem-complete".to_string(),
                    parts: None,
                },
            }],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GeneratePreview,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::PreviewVideo,
                    reference: "s3://derived/proxy.mp4".to_string(),
                    size_bytes: Some(1024),
                    sha256: None,
                }],
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-submit".to_string(),
        })
    }
}

struct ExtractFactsGateway {
    calls: Mutex<Vec<String>>,
    submitted_payload: Mutex<Option<SubmitDerivedPayload>>,
}

impl Default for ExtractFactsGateway {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            submitted_payload: Mutex::new(None),
        }
    }
}

impl ExtractFactsGateway {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("calls").clone()
    }

    fn submitted_payload(&self) -> Option<SubmitDerivedPayload> {
        self.submitted_payload.lock().expect("payload").clone()
    }
}

impl DerivedProcessingGateway for ExtractFactsGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("claim:{job_id}"));
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-facts".to_string(),
            lock_token: "lock-facts".to_string(),
            fencing_token: 1,
            job_type: DerivedJobType::ExtractFacts,
            source_storage_id: "nas-main".to_string(),
            source_original_relative: "INBOX/sample-source.bin".to_string(),
            source_sidecars_relative: vec![
                "INBOX/sidecars/sample-source.xmp".to_string(),
                "INBOX/sidecars/sample-source.srt".to_string(),
            ],
        })
    }

    fn heartbeat(
        &self,
        job_id: &str,
        _lock_token: &str,
        _fencing_token: i32,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("heartbeat:{job_id}"));
        Ok(HeartbeatReceipt {
            locked_until: None,
            fencing_token: 1,
        })
    }

    fn submit_derived(
        &self,
        job_id: &str,
        _lock_token: &str,
        _fencing_token: i32,
        _idempotency_key: &str,
        payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("submit:{job_id}"));
        *self.submitted_payload.lock().expect("payload") = Some(payload.clone());
        Ok(())
    }

    fn upload_init(&self, _request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push("upload_init".to_string());
        Ok(())
    }

    fn upload_part(
        &self,
        request: &DerivedUploadPart,
    ) -> Result<UploadedDerivedPart, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push("upload_part".to_string());
        Ok(UploadedDerivedPart {
            part_number: request.part_number,
            part_etag: format!("etag-{}", request.part_number),
        })
    }

    fn upload_complete(
        &self,
        _request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push("upload_complete".to_string());
        Ok(())
    }
}

struct MissingIdempotencyPlanner;

impl DerivedExecutionPlanner for MissingIdempotencyPlanner {
    fn plan_for_claimed_job(
        &self,
        _claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GeneratePreview,
                manifest: vec![],
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: String::new(),
        })
    }
}

struct MismatchedJobTypePlanner;

impl DerivedExecutionPlanner for MismatchedJobTypePlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GenerateThumbnails,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::Thumb,
                    reference: format!("s3://derived/{}/thumb.jpg", claimed.asset_uuid),
                    size_bytes: Some(1),
                    sha256: None,
                }],
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-submit".to_string(),
        })
    }
}

struct EmptyProxyManifestPlanner;

impl DerivedExecutionPlanner for EmptyProxyManifestPlanner {
    fn plan_for_claimed_job(
        &self,
        _claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GeneratePreview,
                manifest: vec![],
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-submit".to_string(),
        })
    }
}

struct UploadNotInManifestPlanner;

impl DerivedExecutionPlanner for UploadNotInManifestPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![retaia_agent::DerivedUploadPlan {
                init: DerivedUploadInit {
                    asset_uuid: claimed.asset_uuid.clone(),
                    kind: DerivedKind::PreviewAudio,
                    content_type: "audio/mp4".to_string(),
                    size_bytes: 512,
                    sha256: None,
                    idempotency_key: "idem-init".to_string(),
                },
                parts: vec![],
                complete: DerivedUploadComplete {
                    asset_uuid: claimed.asset_uuid.clone(),
                    upload_id: "up-1".to_string(),
                    idempotency_key: "idem-complete".to_string(),
                    parts: None,
                },
            }],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GeneratePreview,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::PreviewVideo,
                    reference: "s3://derived/proxy.mp4".to_string(),
                    size_bytes: Some(1024),
                    sha256: None,
                }],
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-submit".to_string(),
        })
    }
}

struct WaveformGateway {
    calls: Mutex<Vec<String>>,
}

impl Default for WaveformGateway {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl WaveformGateway {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("calls").clone()
    }
}

impl DerivedProcessingGateway for WaveformGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("claim:{job_id}"));
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-wave-1".to_string(),
            lock_token: "lock-wave-1".to_string(),
            fencing_token: 1,
            job_type: DerivedJobType::GenerateAudioWaveform,
            source_storage_id: "nas-main".to_string(),
            source_original_relative: "INBOX/sample-source.bin".to_string(),
            source_sidecars_relative: Vec::new(),
        })
    }

    fn heartbeat(
        &self,
        job_id: &str,
        _lock_token: &str,
        _fencing_token: i32,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("heartbeat:{job_id}"));
        Ok(HeartbeatReceipt {
            locked_until: None,
            fencing_token: 1,
        })
    }

    fn submit_derived(
        &self,
        job_id: &str,
        _lock_token: &str,
        _fencing_token: i32,
        _idempotency_key: &str,
        _payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("submit:{job_id}"));
        Ok(())
    }

    fn upload_init(&self, request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("upload_init:{}", request.asset_uuid));
        Ok(())
    }

    fn upload_part(
        &self,
        request: &DerivedUploadPart,
    ) -> Result<UploadedDerivedPart, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("upload_part:{}", request.part_number));
        Ok(UploadedDerivedPart {
            part_number: request.part_number,
            part_etag: format!("etag-{}", request.part_number),
        })
    }

    fn upload_complete(
        &self,
        request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("upload_complete:{}", request.asset_uuid));
        Ok(())
    }
}

struct ValidWaveformPlanner;

impl DerivedExecutionPlanner for ValidWaveformPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![retaia_agent::DerivedUploadPlan {
                init: DerivedUploadInit {
                    asset_uuid: claimed.asset_uuid.clone(),
                    kind: DerivedKind::Waveform,
                    content_type: "application/json".to_string(),
                    size_bytes: 128,
                    sha256: None,
                    idempotency_key: "idem-wave-init".to_string(),
                },
                parts: vec![DerivedUploadPart {
                    asset_uuid: claimed.asset_uuid.clone(),
                    upload_id: "up-wave-1".to_string(),
                    part_number: 1,
                    chunk_path: std::path::PathBuf::from("/tmp/up-wave-1.bin"),
                }],
                complete: DerivedUploadComplete {
                    asset_uuid: claimed.asset_uuid.clone(),
                    upload_id: "up-wave-1".to_string(),
                    idempotency_key: "idem-wave-complete".to_string(),
                    parts: None,
                },
            }],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GenerateAudioWaveform,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::Waveform,
                    reference: "s3://derived/waveform.json".to_string(),
                    size_bytes: Some(128),
                    sha256: None,
                }],
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-wave-submit".to_string(),
        })
    }
}

struct EmptyWaveformPlanner;

impl DerivedExecutionPlanner for EmptyWaveformPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![],
            submit: SubmitDerivedPayload {
                job_type: claimed.job_type,
                manifest: vec![],
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-wave-empty-submit".to_string(),
        })
    }
}

struct IncompatibleWaveformPlanner;

impl DerivedExecutionPlanner for IncompatibleWaveformPlanner {
    fn plan_for_claimed_job(
        &self,
        _claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GenerateAudioWaveform,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::PreviewAudio,
                    reference: "s3://derived/proxy.m4a".to_string(),
                    size_bytes: Some(42),
                    sha256: None,
                }],
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-wave-submit".to_string(),
        })
    }
}

#[test]
fn tdd_execute_derived_job_once_runs_claim_heartbeat_upload_submit_flow() {
    let gateway = MemoryGateway::default();

    let report = execute_derived_job_once(&gateway, &ProxyPlanner, "job-1").expect("flow");
    assert_eq!(report.job_id, "job-1");
    assert_eq!(report.asset_uuid, "asset-1");
    assert_eq!(report.upload_count, 1);

    assert_eq!(
        gateway.calls(),
        vec![
            "claim:job-1".to_string(),
            "heartbeat:job-1".to_string(),
            "heartbeat:job-1".to_string(),
            "heartbeat:job-1".to_string(),
            "upload_init:asset-1".to_string(),
            "heartbeat:job-1".to_string(),
            "upload_part:1".to_string(),
            "heartbeat:job-1".to_string(),
            "upload_complete:asset-1".to_string(),
            "heartbeat:job-1".to_string(),
            "submit:job-1".to_string(),
        ]
    );
}

#[test]
fn tdd_execute_derived_job_once_rejects_missing_submit_idempotency_key() {
    let gateway = MemoryGateway::default();

    let err = execute_derived_job_once(&gateway, &MissingIdempotencyPlanner, "job-1")
        .expect_err("missing key should fail");
    assert_eq!(err, DerivedJobExecutorError::MissingSubmitIdempotencyKey);
}

#[test]
fn tdd_execute_derived_job_once_rejects_submit_job_type_mismatch_vs_claimed_job() {
    let gateway = MemoryGateway::default();

    let err = execute_derived_job_once(&gateway, &MismatchedJobTypePlanner, "job-1")
        .expect_err("job type mismatch should fail");
    assert_eq!(
        err,
        DerivedJobExecutorError::SubmitJobTypeMismatch {
            claimed: DerivedJobType::GeneratePreview,
            planned: DerivedJobType::GenerateThumbnails,
        }
    );
}

#[test]
fn tdd_execute_derived_job_once_rejects_empty_manifest_for_proxy_job_type() {
    let gateway = MemoryGateway::default();

    let err = execute_derived_job_once(&gateway, &EmptyProxyManifestPlanner, "job-1")
        .expect_err("proxy manifest should be required");
    assert_eq!(
        err,
        DerivedJobExecutorError::MissingSubmitManifestForJobType(DerivedJobType::GeneratePreview)
    );
}

#[test]
fn tdd_execute_derived_job_once_rejects_upload_kind_missing_from_submit_manifest() {
    let gateway = MemoryGateway::default();

    let err = execute_derived_job_once(&gateway, &UploadNotInManifestPlanner, "job-1")
        .expect_err("upload kind must exist in manifest");
    assert_eq!(
        err,
        DerivedJobExecutorError::UploadKindNotInSubmitManifest(DerivedKind::PreviewAudio)
    );
}

#[test]
fn tdd_execute_derived_job_once_allows_waveform_job_with_waveform_manifest_and_upload() {
    let gateway = WaveformGateway::default();
    let report = execute_derived_job_once(&gateway, &ValidWaveformPlanner, "job-wave-1")
        .expect("waveform flow should succeed");
    assert_eq!(report.job_id, "job-wave-1");
    assert_eq!(report.asset_uuid, "asset-wave-1");
    assert_eq!(report.upload_count, 1);
}

#[test]
fn tdd_execute_derived_job_once_rejects_non_waveform_manifest_for_waveform_job_type() {
    let gateway = WaveformGateway::default();
    let err = execute_derived_job_once(&gateway, &IncompatibleWaveformPlanner, "job-wave-2")
        .expect_err("non-waveform manifest must fail");
    assert_eq!(
        err,
        DerivedJobExecutorError::IncompatibleDerivedKindForJobType {
            job_type: DerivedJobType::GenerateAudioWaveform,
            kind: DerivedKind::PreviewAudio,
        }
    );
}

#[test]
fn tdd_execute_derived_job_once_allows_waveform_job_without_waveform_output_and_skips_uploads() {
    let gateway = WaveformGateway::default();
    let report = execute_derived_job_once(&gateway, &EmptyWaveformPlanner, "job-wave-3")
        .expect("empty waveform submit should remain valid");

    assert_eq!(report.job_id, "job-wave-3");
    assert_eq!(report.upload_count, 0);
    assert_eq!(
        gateway.calls(),
        vec![
            "claim:job-wave-3".to_string(),
            "heartbeat:job-wave-3".to_string(),
            "heartbeat:job-wave-3".to_string(),
            "heartbeat:job-wave-3".to_string(),
            "submit:job-wave-3".to_string(),
        ]
    );
}

#[test]
fn tdd_execute_derived_job_once_with_source_staging_copies_source_before_processing() {
    let source_root = tempfile::tempdir().expect("source root");
    write_storage_marker(source_root.path(), "nas-main");
    let source_path = source_root.path().join("INBOX/sample-source.bin");
    std::fs::create_dir_all(source_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&source_path, b"source-bytes").expect("write source");

    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert(
        "nas-main".to_string(),
        source_root.path().display().to_string(),
    );
    let settings = AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts,
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    };

    let gateway = MemoryGateway::default();
    let report =
        execute_derived_job_once_with_source_staging(&gateway, &ProxyPlanner, "job-1", &settings)
            .expect("flow with staging");
    assert_eq!(report.job_id, "job-1");
}

#[test]
fn tdd_execute_derived_job_once_with_source_staging_fails_explicitly_when_mapping_missing() {
    let settings = AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts: std::collections::BTreeMap::new(),
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    };

    let gateway = MemoryGateway::default();
    let error =
        execute_derived_job_once_with_source_staging(&gateway, &ProxyPlanner, "job-1", &settings)
            .expect_err("missing mapping must fail");
    assert!(matches!(error, DerivedJobExecutorError::SourceStaging(_)));
}

#[test]
fn tdd_execute_derived_job_once_with_runtime_planner_emits_upload_calls_with_staged_source() {
    let source_root = tempfile::tempdir().expect("source root");
    write_storage_marker(source_root.path(), "nas-main");
    let source_path = source_root.path().join("INBOX/sample-source.bin");
    std::fs::create_dir_all(source_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&source_path, b"source-bytes").expect("write source");

    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert(
        "nas-main".to_string(),
        source_root.path().display().to_string(),
    );
    let settings = AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts,
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    };

    let gateway = MemoryGateway::default();
    let report = execute_derived_job_once_with_source_staging(
        &gateway,
        &RuntimeDerivedPlanner,
        "job-1",
        &settings,
    )
    .expect("flow with runtime planner");
    assert_eq!(report.upload_count, 1);
    let calls = gateway.calls();
    assert!(calls.iter().any(|call| call.starts_with("upload_init:")));
    assert!(calls.iter().any(|call| call.starts_with("upload_part:")));
    assert!(
        calls
            .iter()
            .any(|call| call.starts_with("upload_complete:"))
    );
}

#[test]
fn tdd_execute_derived_job_once_with_runtime_planner_supports_extract_facts_without_upload_calls() {
    let source_root = tempfile::tempdir().expect("source root");
    write_storage_marker(source_root.path(), "nas-main");
    let source_path = source_root.path().join("INBOX/sample-source.bin");
    let sidecar_xmp_path = source_root.path().join("INBOX/sidecars/sample-source.xmp");
    let sidecar_srt_path = source_root.path().join("INBOX/sidecars/sample-source.srt");
    std::fs::create_dir_all(source_path.parent().expect("parent")).expect("mkdir");
    std::fs::create_dir_all(sidecar_xmp_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&source_path, b"source-bytes").expect("write source");
    std::fs::write(&sidecar_xmp_path, b"xmp-bytes").expect("write xmp");
    std::fs::write(&sidecar_srt_path, b"srt-bytes").expect("write srt");

    let mut storage_mounts = std::collections::BTreeMap::new();
    storage_mounts.insert(
        "nas-main".to_string(),
        source_root.path().display().to_string(),
    );
    let settings = AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts,
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    };

    let gateway = ExtractFactsGateway::default();
    let report = execute_derived_job_once_with_source_staging(
        &gateway,
        &RuntimeDerivedPlanner,
        "job-facts-1",
        &settings,
    )
    .expect("extract_facts flow");
    assert_eq!(report.upload_count, 0);
    let calls = gateway.calls();
    assert!(calls.contains(&"submit:job-facts-1".to_string()));
    assert!(!calls.iter().any(|call| call.starts_with("upload_")));
    let payload = gateway.submitted_payload().expect("submitted payload");
    let metrics = payload.metrics.expect("metrics");
    assert_eq!(
        metrics.get("staged_sidecars_count"),
        Some(&serde_json::json!(2))
    );
}

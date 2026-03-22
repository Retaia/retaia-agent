use std::sync::Arc;
use std::sync::Mutex;

use retaia_agent::{
    AgentRuntimeConfig, AudioProxyRequest, AuthMode, ClaimedDerivedJob, DerivedExecutionPlan,
    DerivedExecutionPlanner, DerivedJobExecutorError, DerivedJobType, DerivedKind,
    DerivedManifestItem, DerivedProcessingError, DerivedProcessingGateway, DerivedUploadComplete,
    DerivedUploadInit, DerivedUploadPart, FactsPatchPayload, HeartbeatReceipt, LogLevel,
    PhotoProxyRequest, ProxyGenerationError, ProxyGenerator, RuntimeDerivedPlanner,
    SubmitDerivedPayload, UploadedDerivedPart, VideoProxyRequest, execute_derived_job_once,
    execute_derived_job_once_with_source_staging,
};

#[derive(Default)]
struct RecordingGateway {
    calls: Mutex<Vec<String>>,
}

impl RecordingGateway {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("calls").clone()
    }
}

impl DerivedProcessingGateway for RecordingGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("claim:{job_id}"));
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-22".to_string(),
            lock_token: "lock-22".to_string(),
            fencing_token: 1,
            job_type: DerivedJobType::GenerateThumbnails,
            source_storage_id: "nas-main".to_string(),
            source_original_relative: "INBOX/sample-source.bin".to_string(),
            source_sidecars_relative: Vec::new(),
        })
    }

    fn fetch_asset_revision_etag(
        &self,
        asset_uuid: &str,
    ) -> Result<String, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("fetch_revision_etag:{asset_uuid}"));
        Ok("\"asset-rev-22\"".to_string())
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
            locked_until: Some("2026-02-17T12:00:00Z".to_string()),
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
        self.calls.lock().expect("calls").push(format!(
            "upload_init:{}:{}",
            request.asset_uuid,
            request.kind.as_str()
        ));
        Ok(())
    }

    fn upload_part(
        &self,
        request: &DerivedUploadPart,
    ) -> Result<UploadedDerivedPart, DerivedProcessingError> {
        self.calls.lock().expect("calls").push(format!(
            "upload_part:{}:{}",
            request.asset_uuid, request.part_number
        ));
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

#[derive(Debug, Default)]
struct WritingPreviewGenerator;

impl ProxyGenerator for WritingPreviewGenerator {
    fn generate_video_proxy(
        &self,
        request: &VideoProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        std::fs::write(&request.output_path, b"generated-video")
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))
    }

    fn generate_audio_proxy(
        &self,
        request: &AudioProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        std::fs::write(&request.output_path, b"generated-audio")
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))
    }

    fn generate_photo_proxy(
        &self,
        request: &PhotoProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        std::fs::write(&request.output_path, b"generated-photo")
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))
    }

    fn extract_media_facts(
        &self,
        _input_path: &str,
    ) -> Result<FactsPatchPayload, ProxyGenerationError> {
        Ok(FactsPatchPayload {
            duration_ms: Some(1_000),
            media_format: Some("mp4".to_string()),
            video_codec: Some("h264".to_string()),
            audio_codec: Some("aac".to_string()),
            width: Some(1280),
            height: Some(720),
            fps: Some(25.0),
            ..FactsPatchPayload::default()
        })
    }
}

#[derive(Default)]
struct ExtractFactsRecordingGateway {
    calls: Mutex<Vec<String>>,
    submitted_payload: Mutex<Option<SubmitDerivedPayload>>,
}

impl ExtractFactsRecordingGateway {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("calls").clone()
    }

    fn submitted_payload(&self) -> Option<SubmitDerivedPayload> {
        self.submitted_payload.lock().expect("payload").clone()
    }
}

impl DerivedProcessingGateway for ExtractFactsRecordingGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("claim:{job_id}"));
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-facts-e2e".to_string(),
            lock_token: "lock-facts-e2e".to_string(),
            fencing_token: 1,
            job_type: DerivedJobType::ExtractFacts,
            source_storage_id: "nas-main".to_string(),
            source_original_relative: "INBOX/sample-source.bin".to_string(),
            source_sidecars_relative: vec!["INBOX/sidecars/sample-source.xmp".to_string()],
        })
    }

    fn fetch_asset_revision_etag(
        &self,
        _asset_uuid: &str,
    ) -> Result<String, DerivedProcessingError> {
        unreachable!("extract_facts does not upload derived files")
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
        unreachable!("extract_facts does not upload derived files")
    }

    fn upload_part(
        &self,
        _request: &DerivedUploadPart,
    ) -> Result<UploadedDerivedPart, DerivedProcessingError> {
        unreachable!("extract_facts does not upload derived files")
    }

    fn upload_complete(
        &self,
        _request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        unreachable!("extract_facts does not upload derived files")
    }
}

struct ThumbnailPlanner;

impl DerivedExecutionPlanner for ThumbnailPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![retaia_agent::DerivedUploadPlan {
                init: DerivedUploadInit {
                    asset_uuid: claimed.asset_uuid.clone(),
                    revision_etag: String::new(),
                    kind: DerivedKind::Thumb,
                    content_type: "image/jpeg".to_string(),
                    size_bytes: 4096,
                    sha256: None,
                    idempotency_key: "idem-init-thumb".to_string(),
                },
                parts: vec![DerivedUploadPart {
                    asset_uuid: claimed.asset_uuid.clone(),
                    revision_etag: String::new(),
                    upload_id: "up-thumb-1".to_string(),
                    part_number: 1,
                    chunk_path: std::path::PathBuf::from("/tmp/up-thumb-1.bin"),
                }],
                complete: DerivedUploadComplete {
                    asset_uuid: claimed.asset_uuid.clone(),
                    revision_etag: String::new(),
                    upload_id: "up-thumb-1".to_string(),
                    idempotency_key: "idem-complete-thumb".to_string(),
                    parts: None,
                },
            }],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GenerateThumbnails,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::Thumb,
                    reference: "s3://derived/thumb.jpg".to_string(),
                    size_bytes: Some(4096),
                    sha256: None,
                }],
                facts_patch: None,
                transcript_patch: None,
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-submit-thumb".to_string(),
        })
    }
}

#[test]
fn e2e_derived_job_executor_flow_claims_uploads_and_submits_for_v1_derived_job() {
    let gateway = RecordingGateway::default();

    let report = execute_derived_job_once(&gateway, &ThumbnailPlanner, "job-22")
        .expect("flow should complete");

    assert_eq!(report.job_id, "job-22");
    assert_eq!(report.asset_uuid, "asset-22");
    assert_eq!(report.upload_count, 1);
    assert_eq!(
        gateway.calls(),
        vec![
            "claim:job-22".to_string(),
            "heartbeat:job-22".to_string(),
            "heartbeat:job-22".to_string(),
            "fetch_revision_etag:asset-22".to_string(),
            "heartbeat:job-22".to_string(),
            "upload_init:asset-22:thumb".to_string(),
            "heartbeat:job-22".to_string(),
            "upload_part:asset-22:1".to_string(),
            "heartbeat:job-22".to_string(),
            "upload_complete:asset-22".to_string(),
            "heartbeat:job-22".to_string(),
            "submit:job-22".to_string(),
        ]
    );
}

fn write_storage_marker(root: &std::path::Path, storage_id: &str) {
    let marker = format!(
        r#"{{"version":1,"storage_id":"{storage_id}","paths":{{"inbox":"INBOX","archive":"ARCHIVE","rejects":"REJECTS"}}}}"#
    );
    std::fs::write(root.join(".retaia"), marker).expect("write marker");
}

#[test]
fn e2e_derived_job_executor_flow_extract_facts_submits_useful_patch_without_uploads() {
    let source_root = tempfile::tempdir().expect("source root");
    write_storage_marker(source_root.path(), "nas-main");
    let source_path = source_root.path().join("INBOX/sample-source.bin");
    let sidecar_path = source_root.path().join("INBOX/sidecars/sample-source.xmp");
    std::fs::create_dir_all(source_path.parent().expect("parent")).expect("mkdir");
    std::fs::create_dir_all(sidecar_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&source_path, b"source-bytes").expect("write source");
    std::fs::write(&sidecar_path, b"xmp-bytes").expect("write sidecar");

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

    let gateway = ExtractFactsRecordingGateway::default();
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );

    let report = execute_derived_job_once_with_source_staging(
        &gateway,
        &planner,
        "job-facts-e2e",
        &settings,
    )
    .expect("extract_facts e2e flow");

    assert_eq!(report.upload_count, 0);
    let calls = gateway.calls();
    assert!(calls.contains(&"submit:job-facts-e2e".to_string()));
    assert!(!calls.iter().any(|call| call.starts_with("upload_")));

    let payload = gateway.submitted_payload().expect("submitted payload");
    let facts = payload.facts_patch.expect("facts patch");
    assert_eq!(facts.duration_ms, Some(1_000));
    assert_eq!(facts.media_format.as_deref(), Some("mp4"));
    assert_eq!(facts.video_codec.as_deref(), Some("h264"));
    assert_eq!(facts.audio_codec.as_deref(), Some("aac"));
    assert_eq!(facts.width, Some(1280));
    assert_eq!(facts.height, Some(720));
    let metrics = payload.metrics.expect("metrics");
    assert_eq!(
        metrics.get("staged_sidecars_count"),
        Some(&serde_json::json!(1))
    );
}

struct IncompatibleThumbnailManifestPlanner;

impl DerivedExecutionPlanner for IncompatibleThumbnailManifestPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GenerateThumbnails,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::PreviewPhoto,
                    reference: format!("s3://derived/{}/proxy.webp", claimed.asset_uuid),
                    size_bytes: Some(1024),
                    sha256: None,
                }],
                facts_patch: None,
                transcript_patch: None,
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-invalid-thumb".to_string(),
        })
    }
}

#[test]
fn e2e_derived_job_executor_flow_rejects_submit_manifest_kind_not_compatible_with_job_type() {
    let gateway = RecordingGateway::default();
    let err = execute_derived_job_once(&gateway, &IncompatibleThumbnailManifestPlanner, "job-23")
        .expect_err("incompatible kind should be rejected");

    assert_eq!(
        err,
        DerivedJobExecutorError::IncompatibleDerivedKindForJobType {
            job_type: DerivedJobType::GenerateThumbnails,
            kind: DerivedKind::PreviewPhoto,
        }
    );
}

#[derive(Default)]
struct WaveformOptionalGateway {
    calls: Mutex<Vec<String>>,
}

impl DerivedProcessingGateway for WaveformOptionalGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("claim:{job_id}"));
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-wave-opt-1".to_string(),
            lock_token: "lock-wave-opt-1".to_string(),
            fencing_token: 1,
            job_type: DerivedJobType::GenerateAudioWaveform,
            source_storage_id: "nas-main".to_string(),
            source_original_relative: "INBOX/sample-source.bin".to_string(),
            source_sidecars_relative: Vec::new(),
        })
    }

    fn fetch_asset_revision_etag(
        &self,
        asset_uuid: &str,
    ) -> Result<String, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("fetch_revision_etag:{asset_uuid}"));
        Ok("\"asset-rev-wave-opt-1\"".to_string())
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

    fn upload_init(&self, _request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        panic!("waveform optional flow should not upload");
    }

    fn upload_part(
        &self,
        _request: &DerivedUploadPart,
    ) -> Result<UploadedDerivedPart, DerivedProcessingError> {
        panic!("waveform optional flow should not upload");
    }

    fn upload_complete(
        &self,
        _request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        panic!("waveform optional flow should not upload");
    }
}

struct WaveformOptionalPlanner;

impl DerivedExecutionPlanner for WaveformOptionalPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![],
            submit: SubmitDerivedPayload {
                job_type: claimed.job_type,
                manifest: vec![],
                facts_patch: None,
                transcript_patch: None,
                warnings: None,
                metrics: None,
            },
            submit_idempotency_key: "idem-wave-opt-submit".to_string(),
        })
    }
}

#[test]
fn e2e_derived_job_executor_flow_rejects_audio_waveform_submit_without_output_artifact() {
    let gateway = WaveformOptionalGateway::default();
    let err = execute_derived_job_once(&gateway, &WaveformOptionalPlanner, "job-wave-opt-1")
        .expect_err("waveform output must be required");

    assert_eq!(
        err,
        DerivedJobExecutorError::MissingSubmitManifestForJobType(
            DerivedJobType::GenerateAudioWaveform
        )
    );
}

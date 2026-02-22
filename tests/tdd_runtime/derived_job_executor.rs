use std::sync::Mutex;

use retaia_agent::{
    ClaimedDerivedJob, DerivedExecutionPlan, DerivedExecutionPlanner, DerivedJobExecutorError,
    DerivedJobType, DerivedKind, DerivedManifestItem, DerivedProcessingError,
    DerivedProcessingGateway, DerivedUploadComplete, DerivedUploadInit, DerivedUploadPart,
    HeartbeatReceipt, SubmitDerivedPayload, execute_derived_job_once,
};

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
            job_type: DerivedJobType::GenerateProxy,
        })
    }

    fn heartbeat(
        &self,
        job_id: &str,
        _lock_token: &str,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("heartbeat:{job_id}"));
        Ok(HeartbeatReceipt { locked_until: None })
    }

    fn submit_derived(
        &self,
        job_id: &str,
        _lock_token: &str,
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

    fn upload_part(&self, request: &DerivedUploadPart) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("upload_part:{}", request.part_number));
        Ok(())
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
                    kind: DerivedKind::ProxyVideo,
                    content_type: "video/mp4".to_string(),
                    size_bytes: 1024,
                    sha256: None,
                    idempotency_key: "idem-init".to_string(),
                },
                parts: vec![DerivedUploadPart {
                    asset_uuid: claimed.asset_uuid.clone(),
                    upload_id: "up-1".to_string(),
                    part_number: 1,
                }],
                complete: DerivedUploadComplete {
                    asset_uuid: claimed.asset_uuid.clone(),
                    upload_id: "up-1".to_string(),
                    idempotency_key: "idem-complete".to_string(),
                    parts: None,
                },
            }],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GenerateProxy,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::ProxyVideo,
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

struct MissingIdempotencyPlanner;

impl DerivedExecutionPlanner for MissingIdempotencyPlanner {
    fn plan_for_claimed_job(
        &self,
        _claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![],
            submit: SubmitDerivedPayload {
                job_type: DerivedJobType::GenerateProxy,
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
                job_type: DerivedJobType::GenerateProxy,
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
                    kind: DerivedKind::ProxyAudio,
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
                job_type: DerivedJobType::GenerateProxy,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::ProxyVideo,
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
            job_type: DerivedJobType::GenerateAudioWaveform,
        })
    }

    fn heartbeat(
        &self,
        job_id: &str,
        _lock_token: &str,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("heartbeat:{job_id}"));
        Ok(HeartbeatReceipt { locked_until: None })
    }

    fn submit_derived(
        &self,
        job_id: &str,
        _lock_token: &str,
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

    fn upload_part(&self, request: &DerivedUploadPart) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("upload_part:{}", request.part_number));
        Ok(())
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
                    kind: DerivedKind::ProxyAudio,
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
            "upload_init:asset-1".to_string(),
            "upload_part:1".to_string(),
            "upload_complete:asset-1".to_string(),
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
            claimed: DerivedJobType::GenerateProxy,
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
        DerivedJobExecutorError::MissingSubmitManifestForJobType(DerivedJobType::GenerateProxy)
    );
}

#[test]
fn tdd_execute_derived_job_once_rejects_upload_kind_missing_from_submit_manifest() {
    let gateway = MemoryGateway::default();

    let err = execute_derived_job_once(&gateway, &UploadNotInManifestPlanner, "job-1")
        .expect_err("upload kind must exist in manifest");
    assert_eq!(
        err,
        DerivedJobExecutorError::UploadKindNotInSubmitManifest(DerivedKind::ProxyAudio)
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
            kind: DerivedKind::ProxyAudio,
        }
    );
}

use retaia_agent::{
    ClaimedDerivedJob, DerivedExecutionPlan, DerivedExecutionPlanner, DerivedJobExecutorError,
    DerivedJobType, DerivedKind, DerivedManifestItem, DerivedProcessingError,
    DerivedProcessingGateway, DerivedUploadComplete, DerivedUploadInit, DerivedUploadPart,
    HeartbeatReceipt, SubmitDerivedPayload, execute_derived_job_once,
};

struct AssetMismatchPlanner;

impl DerivedExecutionPlanner for AssetMismatchPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        Ok(DerivedExecutionPlan {
            uploads: vec![retaia_agent::DerivedUploadPlan {
                init: DerivedUploadInit {
                    asset_uuid: format!("{}-mismatch", claimed.asset_uuid),
                    kind: DerivedKind::ProxyVideo,
                    content_type: "video/mp4".to_string(),
                    size_bytes: 1,
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

struct NoopGateway;

impl DerivedProcessingGateway for NoopGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-1".to_string(),
            lock_token: "lock-1".to_string(),
            job_type: DerivedJobType::GenerateProxy,
        })
    }

    fn heartbeat(
        &self,
        _job_id: &str,
        _lock_token: &str,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        Ok(HeartbeatReceipt { locked_until: None })
    }

    fn submit_derived(
        &self,
        _job_id: &str,
        _lock_token: &str,
        _idempotency_key: &str,
        _payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        Ok(())
    }

    fn upload_init(&self, _request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        Ok(())
    }

    fn upload_part(&self, _request: &DerivedUploadPart) -> Result<(), DerivedProcessingError> {
        Ok(())
    }

    fn upload_complete(
        &self,
        _request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        Ok(())
    }
}

#[test]
fn bdd_given_claimed_job_when_upload_targets_another_asset_then_execution_is_rejected() {
    let err = execute_derived_job_once(&NoopGateway, &AssetMismatchPlanner, "job-1")
        .expect_err("asset mismatch should fail");

    assert_eq!(
        err,
        DerivedJobExecutorError::UploadAssetMismatch {
            job_id: "job-1".to_string()
        }
    );
}

struct WaveformOptionalManifestGateway;

impl DerivedProcessingGateway for WaveformOptionalManifestGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-wf-1".to_string(),
            lock_token: "lock-wf-1".to_string(),
            job_type: DerivedJobType::GenerateAudioWaveform,
        })
    }

    fn heartbeat(
        &self,
        _job_id: &str,
        _lock_token: &str,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        Ok(HeartbeatReceipt { locked_until: None })
    }

    fn submit_derived(
        &self,
        _job_id: &str,
        _lock_token: &str,
        _idempotency_key: &str,
        _payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        Ok(())
    }

    fn upload_init(&self, _request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        Ok(())
    }

    fn upload_part(&self, _request: &DerivedUploadPart) -> Result<(), DerivedProcessingError> {
        Ok(())
    }

    fn upload_complete(
        &self,
        _request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        Ok(())
    }
}

struct WaveformOptionalManifestPlanner;

impl DerivedExecutionPlanner for WaveformOptionalManifestPlanner {
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
            submit_idempotency_key: "idem-waveform-submit".to_string(),
        })
    }
}

#[test]
fn bdd_given_generate_audio_waveform_job_when_no_waveform_derived_is_produced_then_submit_is_still_valid()
 {
    let result = execute_derived_job_once(
        &WaveformOptionalManifestGateway,
        &WaveformOptionalManifestPlanner,
        "job-waveform-1",
    );
    assert!(result.is_ok());
}

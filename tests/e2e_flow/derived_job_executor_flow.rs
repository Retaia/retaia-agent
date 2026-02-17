use std::sync::Mutex;

use retaia_agent::{
    ClaimedDerivedJob, DerivedExecutionPlan, DerivedExecutionPlanner, DerivedJobExecutorError,
    DerivedJobType, DerivedKind, DerivedManifestItem, DerivedProcessingError,
    DerivedProcessingGateway, DerivedUploadComplete, DerivedUploadInit, DerivedUploadPart,
    HeartbeatReceipt, SubmitDerivedPayload, execute_derived_job_once,
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
            job_type: DerivedJobType::GenerateThumbnails,
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
        Ok(HeartbeatReceipt {
            locked_until: Some("2026-02-17T12:00:00Z".to_string()),
        })
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
        self.calls.lock().expect("calls").push(format!(
            "upload_init:{}:{}",
            request.asset_uuid,
            request.kind.as_str()
        ));
        Ok(())
    }

    fn upload_part(&self, request: &DerivedUploadPart) -> Result<(), DerivedProcessingError> {
        self.calls.lock().expect("calls").push(format!(
            "upload_part:{}:{}",
            request.asset_uuid, request.part_number
        ));
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
                    kind: DerivedKind::Thumb,
                    content_type: "image/jpeg".to_string(),
                    size_bytes: 4096,
                    sha256: None,
                    idempotency_key: "idem-init-thumb".to_string(),
                },
                parts: vec![DerivedUploadPart {
                    asset_uuid: claimed.asset_uuid.clone(),
                    upload_id: "up-thumb-1".to_string(),
                    part_number: 1,
                }],
                complete: DerivedUploadComplete {
                    asset_uuid: claimed.asset_uuid.clone(),
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
            "upload_init:asset-22:thumb".to_string(),
            "upload_part:asset-22:1".to_string(),
            "upload_complete:asset-22".to_string(),
            "submit:job-22".to_string(),
        ]
    );
}

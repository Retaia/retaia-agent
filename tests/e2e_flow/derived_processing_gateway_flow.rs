use std::sync::Mutex;

use retaia_agent::{
    ClaimedDerivedJob, DerivedJobType, DerivedKind, DerivedManifestItem, DerivedProcessingError,
    DerivedProcessingGateway, DerivedUploadComplete, DerivedUploadInit, DerivedUploadPart,
    HeartbeatReceipt, SubmitDerivedPayload, UploadedDerivedPart,
};

#[derive(Default)]
struct MemoryDerivedGateway {
    calls: Mutex<Vec<String>>,
}

impl MemoryDerivedGateway {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("calls").clone()
    }
}

impl DerivedProcessingGateway for MemoryDerivedGateway {
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

    fn fetch_asset_revision_etag(
        &self,
        asset_uuid: &str,
    ) -> Result<String, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls")
            .push(format!("fetch_revision_etag:{asset_uuid}"));
        Ok("\"asset-rev-1\"".to_string())
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

#[test]
fn e2e_derived_processing_gateway_flow_claim_upload_submit_sequence_is_supported() {
    let gateway = MemoryDerivedGateway::default();

    let claimed = gateway.claim_job("job-1").expect("claim");
    gateway
        .heartbeat(&claimed.job_id, &claimed.lock_token, claimed.fencing_token)
        .expect("heartbeat");

    gateway
        .upload_init(&DerivedUploadInit {
            asset_uuid: claimed.asset_uuid.clone(),
            revision_etag: "\"asset-rev-1\"".to_string(),
            kind: DerivedKind::PreviewVideo,
            content_type: "video/mp4".to_string(),
            size_bytes: 2048,
            sha256: None,
            idempotency_key: "idem-up-init".to_string(),
        })
        .expect("upload init");
    gateway
        .upload_part(&DerivedUploadPart {
            asset_uuid: claimed.asset_uuid.clone(),
            revision_etag: "\"asset-rev-1\"".to_string(),
            upload_id: "up-1".to_string(),
            part_number: 1,
            chunk_path: std::path::PathBuf::from("/tmp/up-1.bin"),
        })
        .expect("upload part");
    gateway
        .upload_complete(&DerivedUploadComplete {
            asset_uuid: claimed.asset_uuid.clone(),
            revision_etag: "\"asset-rev-1\"".to_string(),
            upload_id: "up-1".to_string(),
            idempotency_key: "idem-up-complete".to_string(),
            parts: None,
        })
        .expect("upload complete");

    gateway
        .submit_derived(
            &claimed.job_id,
            &claimed.lock_token,
            claimed.fencing_token,
            "idem-submit",
            &SubmitDerivedPayload {
                job_type: DerivedJobType::GeneratePreview,
                manifest: vec![DerivedManifestItem {
                    kind: DerivedKind::PreviewVideo,
                    reference: "s3://derived/proxy.mp4".to_string(),
                    size_bytes: Some(2048),
                    sha256: None,
                }],
                facts_patch: None,
                transcript_patch: None,
                warnings: None,
                metrics: None,
            },
        )
        .expect("submit");

    let calls = gateway.calls();
    assert_eq!(
        calls,
        vec![
            "claim:job-1".to_string(),
            "heartbeat:job-1".to_string(),
            "upload_init:asset-1:preview_video".to_string(),
            "upload_part:asset-1:1".to_string(),
            "upload_complete:asset-1".to_string(),
            "submit:job-1".to_string(),
        ]
    );
}

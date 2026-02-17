use thiserror::Error;

use crate::application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedProcessingError, DerivedProcessingGateway, DerivedUploadComplete,
    DerivedUploadInit, DerivedUploadPart, SubmitDerivedPayload,
};

#[derive(Debug, Clone, PartialEq)]
pub struct DerivedUploadPlan {
    pub init: DerivedUploadInit,
    pub parts: Vec<DerivedUploadPart>,
    pub complete: DerivedUploadComplete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DerivedExecutionPlan {
    pub uploads: Vec<DerivedUploadPlan>,
    pub submit: SubmitDerivedPayload,
    pub submit_idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedExecutionReport {
    pub job_id: String,
    pub asset_uuid: String,
    pub upload_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DerivedJobExecutorError {
    #[error("derived processing gateway error: {0}")]
    Gateway(DerivedProcessingError),
    #[error("execution plan invalid: submit idempotency key is required")]
    MissingSubmitIdempotencyKey,
    #[error("execution plan invalid: upload asset mismatch for job {job_id}")]
    UploadAssetMismatch { job_id: String },
    #[error("execution plan invalid: upload init and complete asset mismatch")]
    UploadInitCompleteAssetMismatch,
}

pub trait DerivedExecutionPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError>;
}

pub fn execute_derived_job_once<G: DerivedProcessingGateway, P: DerivedExecutionPlanner>(
    gateway: &G,
    planner: &P,
    job_id: &str,
) -> Result<DerivedExecutionReport, DerivedJobExecutorError> {
    let claimed = gateway
        .claim_job(job_id)
        .map_err(DerivedJobExecutorError::Gateway)?;
    gateway
        .heartbeat(&claimed.job_id, &claimed.lock_token)
        .map_err(DerivedJobExecutorError::Gateway)?;

    let plan = planner.plan_for_claimed_job(&claimed)?;
    if plan.submit_idempotency_key.trim().is_empty() {
        return Err(DerivedJobExecutorError::MissingSubmitIdempotencyKey);
    }

    for upload in &plan.uploads {
        if upload.init.asset_uuid != claimed.asset_uuid
            || upload.complete.asset_uuid != claimed.asset_uuid
        {
            return Err(DerivedJobExecutorError::UploadAssetMismatch {
                job_id: claimed.job_id.clone(),
            });
        }
        if upload.init.asset_uuid != upload.complete.asset_uuid {
            return Err(DerivedJobExecutorError::UploadInitCompleteAssetMismatch);
        }

        gateway
            .upload_init(&upload.init)
            .map_err(DerivedJobExecutorError::Gateway)?;
        for part in &upload.parts {
            gateway
                .upload_part(part)
                .map_err(DerivedJobExecutorError::Gateway)?;
        }
        gateway
            .upload_complete(&upload.complete)
            .map_err(DerivedJobExecutorError::Gateway)?;
    }

    gateway
        .submit_derived(
            &claimed.job_id,
            &claimed.lock_token,
            &plan.submit_idempotency_key,
            &plan.submit,
        )
        .map_err(DerivedJobExecutorError::Gateway)?;

    Ok(DerivedExecutionReport {
        job_id: claimed.job_id,
        asset_uuid: claimed.asset_uuid,
        upload_count: plan.uploads.len(),
    })
}

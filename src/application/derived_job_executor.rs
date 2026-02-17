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
    #[error(
        "execution plan invalid: submit job type {planned:?} does not match claimed job type {claimed:?}"
    )]
    SubmitJobTypeMismatch {
        claimed: crate::application::derived_processing_gateway::DerivedJobType,
        planned: crate::application::derived_processing_gateway::DerivedJobType,
    },
    #[error("execution plan invalid: submit manifest is required for job type {0:?}")]
    MissingSubmitManifestForJobType(crate::application::derived_processing_gateway::DerivedJobType),
    #[error(
        "execution plan invalid: derived kind {kind:?} is incompatible with job type {job_type:?}"
    )]
    IncompatibleDerivedKindForJobType {
        job_type: crate::application::derived_processing_gateway::DerivedJobType,
        kind: crate::application::derived_processing_gateway::DerivedKind,
    },
    #[error("execution plan invalid: upload kind {0:?} is not present in submit manifest")]
    UploadKindNotInSubmitManifest(crate::application::derived_processing_gateway::DerivedKind),
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
    validate_submit_payload_for_claimed_job(&claimed, &plan.submit)?;
    validate_uploads_against_submit_manifest(&plan)?;

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

fn validate_submit_payload_for_claimed_job(
    claimed: &ClaimedDerivedJob,
    submit: &SubmitDerivedPayload,
) -> Result<(), DerivedJobExecutorError> {
    if submit.job_type != claimed.job_type {
        return Err(DerivedJobExecutorError::SubmitJobTypeMismatch {
            claimed: claimed.job_type,
            planned: submit.job_type,
        });
    }

    use crate::application::derived_processing_gateway::{DerivedJobType, DerivedKind};
    match submit.job_type {
        DerivedJobType::GenerateProxy => {
            if submit.manifest.is_empty() {
                return Err(DerivedJobExecutorError::MissingSubmitManifestForJobType(
                    DerivedJobType::GenerateProxy,
                ));
            }
            for item in &submit.manifest {
                match item.kind {
                    DerivedKind::ProxyVideo | DerivedKind::ProxyAudio | DerivedKind::ProxyPhoto => {
                    }
                    kind => {
                        return Err(DerivedJobExecutorError::IncompatibleDerivedKindForJobType {
                            job_type: DerivedJobType::GenerateProxy,
                            kind,
                        });
                    }
                }
            }
        }
        DerivedJobType::GenerateThumbnails => {
            if submit.manifest.is_empty() {
                return Err(DerivedJobExecutorError::MissingSubmitManifestForJobType(
                    DerivedJobType::GenerateThumbnails,
                ));
            }
            for item in &submit.manifest {
                if item.kind != DerivedKind::Thumb {
                    return Err(DerivedJobExecutorError::IncompatibleDerivedKindForJobType {
                        job_type: DerivedJobType::GenerateThumbnails,
                        kind: item.kind,
                    });
                }
            }
        }
        DerivedJobType::GenerateAudioWaveform => {
            for item in &submit.manifest {
                if item.kind != DerivedKind::Waveform {
                    return Err(DerivedJobExecutorError::IncompatibleDerivedKindForJobType {
                        job_type: DerivedJobType::GenerateAudioWaveform,
                        kind: item.kind,
                    });
                }
            }
        }
    }

    Ok(())
}

fn validate_uploads_against_submit_manifest(
    plan: &DerivedExecutionPlan,
) -> Result<(), DerivedJobExecutorError> {
    let manifest_kinds = plan
        .submit
        .manifest
        .iter()
        .map(|item| item.kind)
        .collect::<Vec<_>>();

    for upload in &plan.uploads {
        if !manifest_kinds.contains(&upload.init.kind) {
            return Err(DerivedJobExecutorError::UploadKindNotInSubmitManifest(
                upload.init.kind,
            ));
        }
    }

    Ok(())
}

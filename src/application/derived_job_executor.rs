use std::path::Path;
use thiserror::Error;

use crate::AgentRuntimeConfig;
use crate::application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedProcessingError, DerivedProcessingGateway, DerivedUploadComplete,
    DerivedUploadInit, DerivedUploadPart, SubmitDerivedPayload,
};
use crate::application::source_staging::{SourceStagingError, stage_claimed_job_source};

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
    #[error("source staging failed: {0}")]
    SourceStaging(SourceStagingError),
    #[error("planner error: {0}")]
    Planner(String),
}

pub trait DerivedExecutionPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError>;

    fn plan_for_claimed_job_with_source(
        &self,
        claimed: &ClaimedDerivedJob,
        _staged_source_path: Option<&Path>,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        self.plan_for_claimed_job(claimed)
    }
}

pub fn execute_derived_job_once<
    G: DerivedProcessingGateway + ?Sized,
    P: DerivedExecutionPlanner + ?Sized,
>(
    gateway: &G,
    planner: &P,
    job_id: &str,
) -> Result<DerivedExecutionReport, DerivedJobExecutorError> {
    execute_derived_job_once_internal(gateway, planner, job_id, None)
}

pub fn execute_derived_job_once_with_source_staging<
    G: DerivedProcessingGateway + ?Sized,
    P: DerivedExecutionPlanner + ?Sized,
>(
    gateway: &G,
    planner: &P,
    job_id: &str,
    settings: &AgentRuntimeConfig,
) -> Result<DerivedExecutionReport, DerivedJobExecutorError> {
    execute_derived_job_once_internal(gateway, planner, job_id, Some(settings))
}

fn execute_derived_job_once_internal<
    G: DerivedProcessingGateway + ?Sized,
    P: DerivedExecutionPlanner + ?Sized,
>(
    gateway: &G,
    planner: &P,
    job_id: &str,
    settings: Option<&AgentRuntimeConfig>,
) -> Result<DerivedExecutionReport, DerivedJobExecutorError> {
    let claimed = gateway
        .claim_job(job_id)
        .map_err(DerivedJobExecutorError::Gateway)?;
    send_heartbeat(gateway, &claimed)?;
    let staged_source = if let Some(settings) = settings {
        Some(
            stage_claimed_job_source(settings, &claimed)
                .map_err(DerivedJobExecutorError::SourceStaging)?,
        )
    } else {
        None
    };
    send_heartbeat(gateway, &claimed)?;

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, staged_source.as_ref().map(|s| s.path()))?;
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

        send_heartbeat(gateway, &claimed)?;
        gateway
            .upload_init(&upload.init)
            .map_err(DerivedJobExecutorError::Gateway)?;
        for part in &upload.parts {
            send_heartbeat(gateway, &claimed)?;
            gateway
                .upload_part(part)
                .map_err(DerivedJobExecutorError::Gateway)?;
        }
        send_heartbeat(gateway, &claimed)?;
        gateway
            .upload_complete(&upload.complete)
            .map_err(DerivedJobExecutorError::Gateway)?;
    }

    send_heartbeat(gateway, &claimed)?;
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
        DerivedJobType::ExtractFacts => {}
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

fn send_heartbeat<G: DerivedProcessingGateway + ?Sized>(
    gateway: &G,
    claimed: &ClaimedDerivedJob,
) -> Result<(), DerivedJobExecutorError> {
    gateway
        .heartbeat(&claimed.job_id, &claimed.lock_token)
        .map_err(DerivedJobExecutorError::Gateway)?;
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

use std::collections::BTreeSet;

use thiserror::Error;

use crate::domain::capabilities::{declared_agent_capabilities, has_required_capabilities};
use crate::domain::runtime_ui::{
    ConnectivityState, JobFailure, JobStage, JobStatus, RuntimeSnapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreJobState {
    Pending,
    Claimed,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreJobView {
    pub job_id: String,
    pub asset_uuid: String,
    pub state: CoreJobState,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoreApiGatewayError {
    #[error("core API unauthorized")]
    Unauthorized,
    #[error("core API throttled")]
    Throttled,
    #[error("core API returned unexpected status {0}")]
    UnexpectedStatus(u16),
    #[error("core API transport error: {0}")]
    Transport(String),
}

pub trait CoreApiGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError>;
}

pub fn runtime_snapshot_from_polled_jobs(jobs: &[CoreJobView]) -> RuntimeSnapshot {
    let mut known_job_ids = BTreeSet::new();
    let mut running_job_ids = BTreeSet::new();
    let mut failed_jobs = Vec::new();
    let mut current_job = None;

    for job in jobs {
        known_job_ids.insert(job.job_id.clone());
        match job.state {
            CoreJobState::Claimed => {
                running_job_ids.insert(job.job_id.clone());
                if current_job.is_none() {
                    current_job = Some(JobStatus {
                        job_id: job.job_id.clone(),
                        asset_uuid: job.asset_uuid.clone(),
                        progress_percent: 0,
                        stage: JobStage::Claim,
                        short_status: "claimed".to_string(),
                    });
                }
            }
            CoreJobState::Failed => failed_jobs.push(JobFailure {
                job_id: job.job_id.clone(),
                error_code: "JOB_FAILED_REMOTE".to_string(),
            }),
            CoreJobState::Pending | CoreJobState::Completed => {}
        }
    }

    RuntimeSnapshot {
        known_job_ids,
        running_job_ids,
        failed_jobs,
        connectivity: ConnectivityState::Connected,
        auth_reauth_required: false,
        available_update: None,
        current_job,
    }
}

pub fn filter_jobs_for_declared_capabilities(jobs: Vec<CoreJobView>) -> Vec<CoreJobView> {
    let declared = declared_agent_capabilities();
    jobs.into_iter()
        .filter(|job| has_required_capabilities(&job.required_capabilities, &declared))
        .collect()
}

pub fn poll_runtime_snapshot<G: CoreApiGateway>(
    gateway: &G,
) -> Result<RuntimeSnapshot, CoreApiGatewayError> {
    let jobs = filter_jobs_for_declared_capabilities(gateway.poll_jobs()?);
    Ok(runtime_snapshot_from_polled_jobs(&jobs))
}

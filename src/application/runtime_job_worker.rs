use thiserror::Error;

use crate::application::core_api_gateway::{
    CoreApiGateway, CoreApiGatewayError, CoreJobState, filter_jobs_for_declared_capabilities,
};
use crate::application::derived_job_executor::{
    DerivedExecutionPlanner, DerivedExecutionReport, DerivedJobExecutorError,
    execute_derived_job_once_with_source_staging,
};
use crate::application::derived_processing_gateway::DerivedProcessingGateway;
use crate::application::runtime_session::RuntimeSession;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuntimeJobWorkerError {
    #[error("unable to poll pending jobs: {0}")]
    Poll(CoreApiGatewayError),
    #[error("unable to execute claimed job: {0}")]
    Execute(DerivedJobExecutorError),
}

pub fn process_next_pending_job<
    C: CoreApiGateway + ?Sized,
    D: DerivedProcessingGateway + ?Sized,
    P: DerivedExecutionPlanner + ?Sized,
>(
    session: &RuntimeSession,
    core_gateway: &C,
    derived_gateway: &D,
    planner: &P,
) -> Result<Option<DerivedExecutionReport>, RuntimeJobWorkerError> {
    if !session.can_process_jobs() || !session.can_issue_mutation() {
        return Ok(None);
    }
    let jobs = core_gateway
        .poll_jobs()
        .map_err(RuntimeJobWorkerError::Poll)?;
    let filtered = filter_jobs_for_declared_capabilities(jobs);
    let next_pending = filtered
        .iter()
        .find(|job| matches!(job.state, CoreJobState::Pending))
        .map(|job| job.job_id.clone());
    let Some(job_id) = next_pending else {
        return Ok(None);
    };

    let report = execute_derived_job_once_with_source_staging(
        derived_gateway,
        planner,
        &job_id,
        session.settings(),
    )
    .map_err(RuntimeJobWorkerError::Execute)?;
    Ok(Some(report))
}

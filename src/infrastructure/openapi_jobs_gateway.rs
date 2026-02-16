#[cfg(feature = "core-api-client")]
use std::sync::Arc;

#[cfg(feature = "core-api-client")]
use crate::application::core_api_gateway::{
    CoreApiGateway, CoreApiGatewayError, CoreJobState, CoreJobView,
};

#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::Error as OpenApiError;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::configuration::Configuration;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::jobs_api::{JobsApi, JobsApiClient, JobsGetError};
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::job::Status;

#[cfg(feature = "core-api-client")]
#[derive(Debug, Clone)]
pub struct OpenApiJobsGateway {
    configuration: Configuration,
}

#[cfg(feature = "core-api-client")]
impl OpenApiJobsGateway {
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }
}

#[cfg(feature = "core-api-client")]
impl CoreApiGateway for OpenApiJobsGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError> {
        let api = JobsApiClient::new(Arc::new(self.configuration.clone()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| CoreApiGatewayError::Transport(error.to_string()))?;

        let jobs = runtime
            .block_on(api.jobs_get())
            .map_err(map_openapi_jobs_error)?;

        Ok(jobs
            .into_iter()
            .map(|job| CoreJobView {
                job_id: job.job_id,
                asset_uuid: job.asset_uuid,
                state: map_job_state(job.status),
                required_capabilities: job.required_capabilities,
            })
            .collect())
    }
}

#[cfg(feature = "core-api-client")]
fn map_job_state(status: Status) -> CoreJobState {
    match status {
        Status::Pending => CoreJobState::Pending,
        Status::Claimed => CoreJobState::Claimed,
        Status::Completed => CoreJobState::Completed,
        Status::Failed => CoreJobState::Failed,
    }
}

#[cfg(feature = "core-api-client")]
fn map_openapi_jobs_error(error: OpenApiError<JobsGetError>) -> CoreApiGatewayError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => CoreApiGatewayError::Unauthorized,
            429 => CoreApiGatewayError::Throttled,
            code => CoreApiGatewayError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => CoreApiGatewayError::Transport(err.to_string()),
        OpenApiError::Serde(err) => CoreApiGatewayError::Transport(err.to_string()),
        OpenApiError::Io(err) => CoreApiGatewayError::Transport(err.to_string()),
    }
}

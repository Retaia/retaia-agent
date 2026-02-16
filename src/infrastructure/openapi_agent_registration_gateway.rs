#[cfg(feature = "core-api-client")]
use std::sync::Arc;

#[cfg(feature = "core-api-client")]
use crate::application::agent_registration::{
    AgentRegistrationCommand, AgentRegistrationError, AgentRegistrationGateway,
    AgentRegistrationOutcome,
};

#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::Error as OpenApiError;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::agents_api::{AgentsApi, AgentsApiClient, AgentsRegisterPostError};
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::configuration::Configuration;
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::AgentsRegisterPostRequest;

#[cfg(feature = "core-api-client")]
#[derive(Debug, Clone)]
pub struct OpenApiAgentRegistrationGateway {
    configuration: Configuration,
}

#[cfg(feature = "core-api-client")]
impl OpenApiAgentRegistrationGateway {
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }
}

#[cfg(feature = "core-api-client")]
impl AgentRegistrationGateway for OpenApiAgentRegistrationGateway {
    fn register_agent(
        &self,
        command: &AgentRegistrationCommand,
    ) -> Result<AgentRegistrationOutcome, AgentRegistrationError> {
        let api = AgentsApiClient::new(Arc::new(self.configuration.clone()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| AgentRegistrationError::Transport(error.to_string()))?;

        let mut request = AgentsRegisterPostRequest::new(
            command.agent_name.clone(),
            command.agent_version.clone(),
            command.capabilities.clone(),
        );
        request.platform = command.platform.clone();
        request.client_feature_flags_contract_version =
            command.client_feature_flags_contract_version.clone();
        request.max_parallel_jobs = command.max_parallel_jobs.map(i32::from);

        let response = runtime
            .block_on(api.agents_register_post(request))
            .map_err(map_openapi_register_error)?;

        Ok(AgentRegistrationOutcome {
            agent_id: response.agent_id,
            effective_capabilities: response.effective_capabilities.unwrap_or_default(),
            capability_warnings: response.capability_warnings.unwrap_or_default(),
        })
    }
}

#[cfg(feature = "core-api-client")]
fn map_openapi_register_error(
    error: OpenApiError<AgentsRegisterPostError>,
) -> AgentRegistrationError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => AgentRegistrationError::Unauthorized,
            426 => AgentRegistrationError::UpgradeRequired,
            code => AgentRegistrationError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => AgentRegistrationError::Transport(err.to_string()),
        OpenApiError::Serde(err) => AgentRegistrationError::Transport(err.to_string()),
        OpenApiError::Io(err) => AgentRegistrationError::Transport(err.to_string()),
    }
}

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
use retaia_core_client::models::_agents_register_post_request::{Arch, OsName};
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::AgentsRegisterPostRequest;

#[cfg(feature = "core-api-client")]
const PLACEHOLDER_OPENPGP_PUBLIC_KEY: &str =
    "-----BEGIN PGP PUBLIC KEY BLOCK-----\nplaceholder\n-----END PGP PUBLIC KEY BLOCK-----";
#[cfg(feature = "core-api-client")]
const PLACEHOLDER_OPENPGP_FINGERPRINT: &str = "0000000000000000000000000000000000000000";
#[cfg(feature = "core-api-client")]
const PLACEHOLDER_SIGNATURE: &str = "placeholder-signature";

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

        let agent_id = uuid::Uuid::parse_str(&command.agent_id)
            .map_err(|error| AgentRegistrationError::Transport(error.to_string()))?;
        let os_name = map_os_name(&command.os_name)?;
        let arch = map_arch(&command.arch)?;

        let mut request = AgentsRegisterPostRequest::new(
            agent_id,
            command.agent_name.clone(),
            command.agent_version.clone(),
            PLACEHOLDER_OPENPGP_PUBLIC_KEY.to_string(),
            PLACEHOLDER_OPENPGP_FINGERPRINT.to_string(),
            os_name,
            command.os_version.clone(),
            arch,
            command.capabilities.clone(),
        );
        request.client_feature_flags_contract_version =
            command.client_feature_flags_contract_version.clone();
        request.max_parallel_jobs = command.max_parallel_jobs.map(i32::from);

        let signature_timestamp = signature_timestamp_rfc3339_utc();
        let signature_nonce = uuid::Uuid::new_v4().to_string();
        let response = runtime
            .block_on(api.agents_register_post(
                &command.agent_id,
                PLACEHOLDER_OPENPGP_FINGERPRINT,
                PLACEHOLDER_SIGNATURE,
                signature_timestamp,
                &signature_nonce,
                request,
            ))
            .map_err(map_openapi_register_error)?;

        Ok(AgentRegistrationOutcome {
            agent_id: response.agent_id.map(|value| value.to_string()),
            effective_capabilities: response.effective_capabilities.unwrap_or_default(),
            capability_warnings: response.capability_warnings.unwrap_or_default(),
        })
    }
}

#[cfg(feature = "core-api-client")]
fn signature_timestamp_rfc3339_utc() -> String {
    "1970-01-01T00:00:00Z".to_string()
}

#[cfg(feature = "core-api-client")]
fn map_os_name(value: &str) -> Result<OsName, AgentRegistrationError> {
    match value {
        "linux" => Ok(OsName::Linux),
        "macos" => Ok(OsName::Macos),
        "windows" => Ok(OsName::Windows),
        other => Err(AgentRegistrationError::Transport(format!(
            "unsupported os_name: {other}"
        ))),
    }
}

#[cfg(feature = "core-api-client")]
fn map_arch(value: &str) -> Result<Arch, AgentRegistrationError> {
    match value {
        "x86_64" => Ok(Arch::X8664),
        "arm64" => Ok(Arch::Arm64),
        "armv7" => Ok(Arch::Armv7),
        "other" => Ok(Arch::Other),
        other => Err(AgentRegistrationError::Transport(format!(
            "unsupported arch: {other}"
        ))),
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

#[cfg(all(test, feature = "core-api-client"))]
mod tests {
    use super::map_openapi_register_error;
    use crate::AgentRegistrationError;
    use reqwest::StatusCode;
    use retaia_core_client::apis::agents_api::AgentsRegisterPostError;
    use retaia_core_client::apis::{Error as OpenApiError, ResponseContent};

    fn response_error(status: u16) -> OpenApiError<AgentsRegisterPostError> {
        OpenApiError::ResponseError(ResponseContent {
            status: StatusCode::from_u16(status).expect("valid status"),
            content: String::new(),
            entity: None,
        })
    }

    #[test]
    fn tdd_openapi_agent_registration_gateway_maps_expected_http_statuses() {
        assert_eq!(
            map_openapi_register_error(response_error(401)),
            AgentRegistrationError::Unauthorized
        );
        assert_eq!(
            map_openapi_register_error(response_error(426)),
            AgentRegistrationError::UpgradeRequired
        );
        assert_eq!(
            map_openapi_register_error(response_error(422)),
            AgentRegistrationError::UnexpectedStatus(422)
        );
        assert_eq!(
            map_openapi_register_error(response_error(500)),
            AgentRegistrationError::UnexpectedStatus(500)
        );
    }

    #[test]
    fn tdd_openapi_agent_registration_gateway_maps_transport_errors() {
        let error =
            map_openapi_register_error(OpenApiError::Io(std::io::Error::other("connection reset")));
        match error {
            AgentRegistrationError::Transport(message) => {
                assert!(message.contains("connection reset"))
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}

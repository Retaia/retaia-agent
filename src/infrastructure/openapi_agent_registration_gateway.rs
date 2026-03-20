#[cfg(feature = "core-api-client")]
use crate::application::agent_registration::{
    AgentRegistrationCommand, AgentRegistrationError, AgentRegistrationGateway,
    AgentRegistrationOutcome,
};
#[cfg(feature = "core-api-client")]
use crate::infrastructure::agent_identity::AgentIdentity;
#[cfg(feature = "core-api-client")]
use crate::infrastructure::signed_core_http::{json_bytes, signed_json_request};

#[cfg(feature = "core-api-client")]
use reqwest::StatusCode;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::configuration::Configuration;
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::_agents_register_post_request::{Arch, OsName};
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::{AgentsRegisterPost200Response, AgentsRegisterPostRequest};

#[cfg(feature = "core-api-client")]
#[derive(Debug, Clone)]
pub struct OpenApiAgentRegistrationGateway {
    configuration: Configuration,
    identity: Option<AgentIdentity>,
}

#[cfg(feature = "core-api-client")]
impl OpenApiAgentRegistrationGateway {
    pub fn new(configuration: Configuration) -> Self {
        Self {
            configuration,
            identity: None,
        }
    }

    pub fn new_with_identity(configuration: Configuration, identity: AgentIdentity) -> Self {
        Self {
            configuration,
            identity: Some(identity),
        }
    }
}

#[cfg(feature = "core-api-client")]
impl AgentRegistrationGateway for OpenApiAgentRegistrationGateway {
    fn register_agent(
        &self,
        command: &AgentRegistrationCommand,
    ) -> Result<AgentRegistrationOutcome, AgentRegistrationError> {
        let identity = match &self.identity {
            Some(identity) => identity.clone(),
            None => AgentIdentity::generate_ephemeral(Some(&command.agent_id))
                .map_err(|error| AgentRegistrationError::Transport(error.to_string()))?,
        };
        if identity.agent_id != command.agent_id {
            return Err(AgentRegistrationError::Transport(format!(
                "agent identity mismatch (expected={} actual={})",
                identity.agent_id, command.agent_id
            )));
        }

        let agent_id = uuid::Uuid::parse_str(&command.agent_id)
            .map_err(|error| AgentRegistrationError::Transport(error.to_string()))?;
        let os_name = map_os_name(&command.os_name)?;
        let arch = map_arch(&command.arch)?;

        let mut request = AgentsRegisterPostRequest::new(
            agent_id,
            command.agent_name.clone(),
            command.agent_version.clone(),
            identity.openpgp_public_key.clone(),
            identity.openpgp_fingerprint.clone(),
            os_name,
            command.os_version.clone(),
            arch,
            command.capabilities.clone(),
        );
        request.client_feature_flags_contract_version =
            command.client_feature_flags_contract_version.clone();
        request.max_parallel_jobs = command.max_parallel_jobs.map(i32::from);
        let payload = json_bytes(&request)
            .map_err(|error| AgentRegistrationError::Transport(error.to_string()))?;

        let response = signed_json_request(
            &reqwest::blocking::Client::new(),
            &identity,
            self.configuration.bearer_access_token.as_deref(),
            &self.configuration.base_path,
            reqwest::Method::POST,
            "/agents/register",
            &payload,
            None,
        )
        .map_err(|error| AgentRegistrationError::Transport(error.to_string()))?
        .send()
        .map_err(|error| AgentRegistrationError::Transport(error.to_string()))?;

        map_registration_response(response)
    }
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
fn map_registration_response(
    response: reqwest::blocking::Response,
) -> Result<AgentRegistrationOutcome, AgentRegistrationError> {
    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(AgentRegistrationError::Unauthorized);
    }
    if status.as_u16() == 426 {
        return Err(AgentRegistrationError::UpgradeRequired);
    }
    if !status.is_success() {
        return Err(AgentRegistrationError::UnexpectedStatus(status.as_u16()));
    }

    let response: AgentsRegisterPost200Response = response
        .json()
        .map_err(|error| AgentRegistrationError::Transport(error.to_string()))?;
    Ok(AgentRegistrationOutcome {
        agent_id: response.agent_id.map(|value| value.to_string()),
        effective_capabilities: response.effective_capabilities.unwrap_or_default(),
        capability_warnings: response.capability_warnings.unwrap_or_default(),
    })
}

#[cfg(all(test, feature = "core-api-client"))]
mod tests {
    use super::{map_arch, map_os_name};
    use crate::AgentRegistrationError;

    #[test]
    fn tdd_openapi_agent_registration_gateway_rejects_unknown_os() {
        assert_eq!(
            map_os_name("amiga").expect_err("must fail"),
            AgentRegistrationError::Transport("unsupported os_name: amiga".to_string())
        );
    }

    #[test]
    fn tdd_openapi_agent_registration_gateway_rejects_unknown_arch() {
        assert_eq!(
            map_arch("sparc").expect_err("must fail"),
            AgentRegistrationError::Transport("unsupported arch: sparc".to_string())
        );
    }
}

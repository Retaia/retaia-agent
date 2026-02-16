use thiserror::Error;

use crate::domain::capabilities::declared_agent_capabilities;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRegistrationIntent {
    pub agent_name: String,
    pub agent_version: String,
    pub platform: Option<String>,
    pub client_feature_flags_contract_version: Option<String>,
    pub max_parallel_jobs: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRegistrationCommand {
    pub agent_name: String,
    pub agent_version: String,
    pub platform: Option<String>,
    pub capabilities: Vec<String>,
    pub client_feature_flags_contract_version: Option<String>,
    pub max_parallel_jobs: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRegistrationOutcome {
    pub agent_id: Option<String>,
    pub effective_capabilities: Vec<String>,
    pub capability_warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AgentRegistrationError {
    #[error("core API unauthorized")]
    Unauthorized,
    #[error("core API requires client upgrade (426)")]
    UpgradeRequired,
    #[error("core API returned unexpected status {0}")]
    UnexpectedStatus(u16),
    #[error("core API transport error: {0}")]
    Transport(String),
}

pub trait AgentRegistrationGateway {
    fn register_agent(
        &self,
        command: &AgentRegistrationCommand,
    ) -> Result<AgentRegistrationOutcome, AgentRegistrationError>;
}

pub fn build_agent_registration_command(
    intent: AgentRegistrationIntent,
) -> AgentRegistrationCommand {
    let mut capabilities = declared_agent_capabilities()
        .into_iter()
        .collect::<Vec<_>>();
    capabilities.sort();

    AgentRegistrationCommand {
        agent_name: intent.agent_name,
        agent_version: intent.agent_version,
        platform: intent.platform,
        capabilities,
        client_feature_flags_contract_version: intent.client_feature_flags_contract_version,
        max_parallel_jobs: intent.max_parallel_jobs,
    }
}

pub fn register_agent<G: AgentRegistrationGateway>(
    gateway: &G,
    intent: AgentRegistrationIntent,
) -> Result<AgentRegistrationOutcome, AgentRegistrationError> {
    let command = build_agent_registration_command(intent);
    gateway.register_agent(&command)
}

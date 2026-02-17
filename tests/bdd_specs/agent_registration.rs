use retaia_agent::{
    AgentRegistrationCommand, AgentRegistrationError, AgentRegistrationGateway,
    AgentRegistrationIntent, AgentRegistrationOutcome, register_agent,
};

struct CaptureGateway {
    captured: std::sync::Mutex<Option<AgentRegistrationCommand>>,
}

impl CaptureGateway {
    fn new() -> Self {
        Self {
            captured: std::sync::Mutex::new(None),
        }
    }
}

impl AgentRegistrationGateway for CaptureGateway {
    fn register_agent(
        &self,
        command: &AgentRegistrationCommand,
    ) -> Result<AgentRegistrationOutcome, AgentRegistrationError> {
        *self.captured.lock().expect("captured lock") = Some(command.clone());
        Ok(AgentRegistrationOutcome {
            agent_id: Some("agent-bdd".to_string()),
            effective_capabilities: command.capabilities.clone(),
            capability_warnings: Vec::new(),
        })
    }
}

#[test]
fn bdd_given_agent_registration_when_building_command_then_first_capability_is_declared() {
    let gateway = CaptureGateway::new();
    let intent = AgentRegistrationIntent {
        agent_name: "retaia-agent".to_string(),
        agent_version: "0.1.0".to_string(),
        platform: Some("linux-x86_64".to_string()),
        client_feature_flags_contract_version: None,
        max_parallel_jobs: Some(1),
    };

    let outcome = register_agent(&gateway, intent).expect("registration should succeed");
    assert!(
        outcome
            .effective_capabilities
            .contains(&"media.facts@1".to_string())
    );

    let captured = gateway
        .captured
        .lock()
        .expect("captured lock")
        .clone()
        .expect("captured command");
    assert!(captured.capabilities.contains(&"media.facts@1".to_string()));
    assert!(
        captured
            .capabilities
            .contains(&"media.thumbnails@1".to_string())
    );
}

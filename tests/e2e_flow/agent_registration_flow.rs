use retaia_agent::{
    AgentRegistrationCommand, AgentRegistrationError, AgentRegistrationGateway,
    AgentRegistrationIntent, AgentRegistrationOutcome, register_agent,
};

#[derive(Default)]
struct MemoryRegistrationGateway {
    captured: std::sync::Mutex<Vec<AgentRegistrationCommand>>,
}

impl AgentRegistrationGateway for MemoryRegistrationGateway {
    fn register_agent(
        &self,
        command: &AgentRegistrationCommand,
    ) -> Result<AgentRegistrationOutcome, AgentRegistrationError> {
        self.captured
            .lock()
            .expect("captured lock")
            .push(command.clone());
        Ok(AgentRegistrationOutcome {
            agent_id: Some("agent-e2e".to_string()),
            effective_capabilities: command.capabilities.clone(),
            capability_warnings: Vec::new(),
        })
    }
}

#[test]
fn e2e_agent_registration_flow_builds_declared_capabilities_and_returns_effective_set() {
    let gateway = MemoryRegistrationGateway::default();
    let intent = AgentRegistrationIntent {
        agent_name: "retaia-agent".to_string(),
        agent_version: "0.1.0".to_string(),
        platform: Some("windows-x86_64".to_string()),
        client_feature_flags_contract_version: Some("1.0.0".to_string()),
        max_parallel_jobs: Some(3),
    };

    let outcome = register_agent(&gateway, intent).expect("registration should succeed");
    assert_eq!(outcome.agent_id.as_deref(), Some("agent-e2e"));
    assert!(
        outcome
            .effective_capabilities
            .contains(&"media.facts@1".to_string())
    );
    assert!(
        outcome
            .effective_capabilities
            .contains(&"media.thumbnails@1".to_string())
    );

    let captured = gateway.captured.lock().expect("captured lock");
    assert_eq!(captured.len(), 1);
    assert!(
        captured[0]
            .capabilities
            .contains(&"media.facts@1".to_string())
    );
    assert!(
        captured[0]
            .capabilities
            .contains(&"media.proxies.video@1".to_string())
    );
}

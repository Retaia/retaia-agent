use retaia_agent::{
    AgentRegistrationCommand, AgentRegistrationError, AgentRegistrationGateway,
    AgentRegistrationIntent, AgentRegistrationOutcome, build_agent_registration_command,
    ffmpeg_available, register_agent,
};

#[derive(Default)]
struct StubGateway {
    last_command: std::sync::Mutex<Option<AgentRegistrationCommand>>,
    result: std::sync::Mutex<Option<Result<AgentRegistrationOutcome, AgentRegistrationError>>>,
}

impl StubGateway {
    fn with_result(result: Result<AgentRegistrationOutcome, AgentRegistrationError>) -> Self {
        Self {
            last_command: std::sync::Mutex::new(None),
            result: std::sync::Mutex::new(Some(result)),
        }
    }
}

impl AgentRegistrationGateway for StubGateway {
    fn register_agent(
        &self,
        command: &AgentRegistrationCommand,
    ) -> Result<AgentRegistrationOutcome, AgentRegistrationError> {
        *self.last_command.lock().expect("last command lock") = Some(command.clone());
        self.result
            .lock()
            .expect("result lock")
            .clone()
            .expect("stub result")
    }
}

fn intent() -> AgentRegistrationIntent {
    AgentRegistrationIntent {
        agent_name: "retaia-agent".to_string(),
        agent_version: "0.1.0".to_string(),
        platform: Some("macos-arm64".to_string()),
        client_feature_flags_contract_version: Some("1.0.0".to_string()),
        max_parallel_jobs: Some(2),
    }
}

#[test]
fn tdd_build_agent_registration_command_includes_declared_capabilities() {
    let command = build_agent_registration_command(intent());
    assert_eq!(command.agent_name, "retaia-agent");
    assert_eq!(command.agent_version, "0.1.0");
    assert!(command.capabilities.contains(&"media.facts@1".to_string()));
    assert!(
        command
            .capabilities
            .contains(&"media.thumbnails@1".to_string())
    );
    if ffmpeg_available() {
        assert!(
            command
                .capabilities
                .contains(&"media.proxies.video@1".to_string())
        );
    } else {
        assert!(
            !command
                .capabilities
                .contains(&"media.proxies.video@1".to_string())
        );
    }
}

#[test]
fn tdd_register_agent_passes_built_command_to_gateway() {
    let gateway = StubGateway::with_result(Ok(AgentRegistrationOutcome {
        agent_id: Some("agent-1".to_string()),
        effective_capabilities: vec![
            "media.facts@1".to_string(),
            "media.thumbnails@1".to_string(),
        ],
        capability_warnings: Vec::new(),
    }));

    let result = register_agent(&gateway, intent()).expect("register should succeed");
    assert_eq!(result.agent_id.as_deref(), Some("agent-1"));

    let sent = gateway
        .last_command
        .lock()
        .expect("last command lock")
        .clone()
        .expect("sent command");
    assert!(sent.capabilities.contains(&"media.facts@1".to_string()));
    assert!(
        sent.capabilities
            .contains(&"media.thumbnails@1".to_string())
    );
}

#[test]
fn tdd_register_agent_propagates_gateway_errors() {
    let gateway = StubGateway::with_result(Err(AgentRegistrationError::Unauthorized));
    let error = register_agent(&gateway, intent()).expect_err("should fail");
    assert_eq!(error, AgentRegistrationError::Unauthorized);
}

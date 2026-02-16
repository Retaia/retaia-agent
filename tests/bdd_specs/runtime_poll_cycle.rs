use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, CoreApiGateway, CoreApiGatewayError,
    LogLevel, NotificationBridgeError, NotificationMessage, NotificationSink, PollEndpoint,
    RuntimePollCycleStatus, RuntimeSession, SystemNotification, run_runtime_poll_cycle,
};

fn config() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local/api/v1".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    }
}

struct ScenarioGateway {
    result: Result<Vec<retaia_agent::CoreJobView>, CoreApiGatewayError>,
}

impl CoreApiGateway for ScenarioGateway {
    fn poll_jobs(&self) -> Result<Vec<retaia_agent::CoreJobView>, CoreApiGatewayError> {
        self.result.clone()
    }
}

#[derive(Default)]
struct NopSink;

impl NotificationSink for NopSink {
    fn send(
        &self,
        _message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        Ok(())
    }
}

#[test]
fn bdd_given_runtime_poll_unauthorized_when_cycle_runs_then_auth_reauth_notification_is_emitted_once()
 {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let gateway = ScenarioGateway {
        result: Err(CoreApiGatewayError::Unauthorized),
    };

    let first = run_runtime_poll_cycle(
        &mut session,
        &gateway,
        &NopSink,
        PollEndpoint::Jobs,
        5_000,
        7,
    );
    let second = run_runtime_poll_cycle(
        &mut session,
        &gateway,
        &NopSink,
        PollEndpoint::Jobs,
        5_000,
        8,
    );

    assert_eq!(first.status, RuntimePollCycleStatus::Degraded);
    assert_eq!(
        first
            .report
            .expect("first report")
            .notifications
            .iter()
            .filter(|n| matches!(n, SystemNotification::AuthExpiredReauthRequired))
            .count(),
        1
    );
    assert_eq!(
        second
            .report
            .expect("second report")
            .notifications
            .iter()
            .filter(|n| matches!(n, SystemNotification::AuthExpiredReauthRequired))
            .count(),
        0
    );
}

#[test]
fn bdd_given_runtime_poll_transport_error_when_cycle_runs_then_connectivity_notification_is_emitted()
 {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let gateway = ScenarioGateway {
        result: Err(CoreApiGatewayError::Transport("offline".to_string())),
    };

    let outcome = run_runtime_poll_cycle(
        &mut session,
        &gateway,
        &NopSink,
        PollEndpoint::Jobs,
        5_000,
        9,
    );

    assert_eq!(outcome.status, RuntimePollCycleStatus::Degraded);
    assert!(
        outcome
            .report
            .expect("report")
            .notifications
            .contains(&SystemNotification::AgentDisconnectedOrReconnecting)
    );
}

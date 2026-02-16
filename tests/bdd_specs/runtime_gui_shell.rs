use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, DaemonLabelRequest, DaemonManager,
    DaemonManagerError, DaemonStatus, LogLevel, RuntimeSession, settings_panel_content,
    status_window_content,
};
use retaia_agent::{GuiDaemonContext, GuiMenuAction, apply_gui_menu_action};

fn config() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local/api/v1".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

struct ScenarioDaemonManager {
    status: DaemonStatus,
}

impl DaemonManager for ScenarioDaemonManager {
    fn install(
        &self,
        _request: retaia_agent::DaemonInstallRequest,
    ) -> Result<(), DaemonManagerError> {
        Ok(())
    }

    fn uninstall(&self, _request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
        Ok(())
    }

    fn start(&self, _request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
        Ok(())
    }

    fn stop(&self, _request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
        Ok(())
    }

    fn status(&self, _request: DaemonLabelRequest) -> Result<DaemonStatus, DaemonManagerError> {
        Ok(self.status.clone())
    }
}

#[test]
fn bdd_given_gui_status_window_when_opened_then_content_matches_runtime_status_projection() {
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");

    let content = status_window_content(&session);
    assert!(content.contains("run_state=running"));
    assert!(content.contains("status=idle"));
}

#[test]
fn bdd_given_gui_settings_panel_when_opened_then_content_matches_shared_config_contract() {
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");

    let content = settings_panel_content(&session);
    assert!(content.contains("core_api_url=https://core.retaia.local/api/v1"));
    assert!(content.contains("ollama_url=http://127.0.0.1:11434"));
}

#[test]
fn bdd_given_gui_daemon_status_refresh_when_requested_then_status_is_returned_from_daemon_manager()
{
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let manager = ScenarioDaemonManager {
        status: DaemonStatus::Stopped(Some("manual stop".to_string())),
    };

    let outcome = apply_gui_menu_action(
        &mut session,
        &manager,
        &GuiDaemonContext::default(),
        GuiMenuAction::RefreshDaemonStatus,
    )
    .expect("refresh should succeed");

    assert_eq!(
        outcome.daemon_status,
        Some(DaemonStatus::Stopped(Some("manual stop".to_string())))
    );
}

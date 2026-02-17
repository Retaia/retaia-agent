use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, DaemonLabelRequest, DaemonLevel,
    DaemonManager, DaemonManagerError, DaemonStatus, LogLevel, RuntimeSession,
    apply_gui_menu_action, menu_view,
};
use retaia_agent::{GuiDaemonContext, GuiMenuAction};

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

struct StubDaemonManager {
    status: DaemonStatus,
    fail: bool,
}

impl Default for StubDaemonManager {
    fn default() -> Self {
        Self {
            status: DaemonStatus::NotInstalled,
            fail: false,
        }
    }
}

impl DaemonManager for StubDaemonManager {
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
        if self.fail {
            return Err(DaemonManagerError::OperationFailed("boom".to_string()));
        }
        Ok(())
    }

    fn stop(&self, _request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
        if self.fail {
            return Err(DaemonManagerError::OperationFailed("boom".to_string()));
        }
        Ok(())
    }

    fn status(&self, _request: DaemonLabelRequest) -> Result<DaemonStatus, DaemonManagerError> {
        Ok(self.status.clone())
    }
}

#[test]
fn tdd_gui_menu_view_reflects_play_pause_toggle_rule() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");

    let running = menu_view(&session, Some(DaemonStatus::Running));
    assert!(!running.show_play_resume);
    assert!(running.show_pause);

    session.on_menu_action(retaia_agent::MenuAction::Pause);
    let paused = menu_view(&session, Some(DaemonStatus::Running));
    assert!(paused.show_play_resume);
    assert!(!paused.show_pause);
}

#[test]
fn tdd_gui_menu_action_start_daemon_returns_status() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let manager = StubDaemonManager {
        status: DaemonStatus::Running,
        fail: false,
    };

    let outcome = apply_gui_menu_action(
        &mut session,
        &manager,
        &GuiDaemonContext::default(),
        GuiMenuAction::StartDaemon,
    )
    .expect("start daemon should succeed");

    assert_eq!(outcome.daemon_status, Some(DaemonStatus::Running));
}

#[test]
fn tdd_gui_menu_action_stop_daemon_propagates_operation_error() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let manager = StubDaemonManager {
        status: DaemonStatus::Stopped(None),
        fail: true,
    };

    let err = apply_gui_menu_action(
        &mut session,
        &manager,
        &GuiDaemonContext {
            label: "io.retaia.agent".to_string(),
            level: DaemonLevel::User,
        },
        GuiMenuAction::StopDaemon,
    )
    .expect_err("daemon stop should fail");

    assert!(err.to_string().contains("daemon operation failed"));
}

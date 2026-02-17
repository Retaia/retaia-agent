use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, DaemonLabelRequest, DaemonManager,
    DaemonManagerError, DaemonStatus, DesktopShellBridge, DesktopShellController, GuiMenuAction,
    LogLevel, RuntimeSession,
};

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

struct ScenarioDaemonManager;

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
        Ok(DaemonStatus::Running)
    }
}

#[derive(Default)]
struct ScenarioBridge {
    status_opens: usize,
    settings_opens: usize,
    quit_calls: usize,
    menu_renders: usize,
}

impl DesktopShellBridge for ScenarioBridge {
    fn render_menu(&mut self, _view: &retaia_agent::GuiMenuView) {
        self.menu_renders += 1;
    }

    fn open_status_window(&mut self, _content: &str) {
        self.status_opens += 1;
    }

    fn open_settings_panel(&mut self, _content: &str) {
        self.settings_opens += 1;
    }

    fn request_quit(&mut self) {
        self.quit_calls += 1;
    }
}

#[test]
fn bdd_given_desktop_shell_controller_when_menu_actions_trigger_windows_then_bridge_is_notified() {
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let mut controller =
        DesktopShellController::with_default_user_daemon(session, ScenarioDaemonManager);
    let mut bridge = ScenarioBridge::default();

    controller
        .handle_action(GuiMenuAction::OpenStatusWindow, &mut bridge)
        .expect("status open");
    controller
        .handle_action(GuiMenuAction::OpenSettings, &mut bridge)
        .expect("settings open");

    assert_eq!(bridge.status_opens, 1);
    assert_eq!(bridge.settings_opens, 1);
    assert!(bridge.menu_renders >= 2);
}

#[test]
fn bdd_given_desktop_shell_controller_when_quit_action_received_then_bridge_quit_is_requested_once()
{
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let mut controller =
        DesktopShellController::with_default_user_daemon(session, ScenarioDaemonManager);
    let mut bridge = ScenarioBridge::default();

    controller
        .handle_action(GuiMenuAction::Quit, &mut bridge)
        .expect("quit");

    assert_eq!(bridge.quit_calls, 1);
}

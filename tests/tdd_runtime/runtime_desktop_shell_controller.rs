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

#[derive(Default)]
struct StubDaemonManager;

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
struct MemoryBridge {
    menu_renders: usize,
    opened_status: Vec<String>,
    opened_settings: Vec<String>,
    quit_requested: bool,
}

impl DesktopShellBridge for MemoryBridge {
    fn render_menu(&mut self, _view: &retaia_agent::GuiMenuView) {
        self.menu_renders += 1;
    }

    fn open_status_window(&mut self, content: &str) {
        self.opened_status.push(content.to_string());
    }

    fn open_settings_panel(&mut self, content: &str) {
        self.opened_settings.push(content.to_string());
    }

    fn request_quit(&mut self) {
        self.quit_requested = true;
    }
}

#[test]
fn tdd_desktop_shell_controller_opens_status_and_settings_panels_from_shared_runtime_content() {
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let manager = StubDaemonManager;
    let mut controller = DesktopShellController::with_default_user_daemon(session, manager);
    let mut bridge = MemoryBridge::default();

    controller.render_initial_menu(&mut bridge);
    controller
        .handle_action(GuiMenuAction::OpenStatusWindow, &mut bridge)
        .expect("status should open");
    controller
        .handle_action(GuiMenuAction::OpenSettings, &mut bridge)
        .expect("settings should open");

    assert!(bridge.menu_renders >= 3);
    assert!(bridge.opened_status[0].contains("run_state=running"));
    assert!(bridge.opened_settings[0].contains("core_api_url="));
}

#[test]
fn tdd_desktop_shell_controller_quit_action_requests_bridge_quit() {
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let manager = StubDaemonManager;
    let mut controller = DesktopShellController::with_default_user_daemon(session, manager);
    let mut bridge = MemoryBridge::default();

    controller
        .handle_action(GuiMenuAction::Quit, &mut bridge)
        .expect("quit should succeed");

    assert!(bridge.quit_requested);
}

#[test]
fn tdd_desktop_shell_controller_start_daemon_updates_daemon_status_projection() {
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let manager = StubDaemonManager;
    let mut controller = DesktopShellController::with_default_user_daemon(session, manager);
    let mut bridge = MemoryBridge::default();

    controller
        .handle_action(GuiMenuAction::StartDaemon, &mut bridge)
        .expect("daemon start should succeed");

    assert_eq!(controller.daemon_status(), Some(&DaemonStatus::Running));
}

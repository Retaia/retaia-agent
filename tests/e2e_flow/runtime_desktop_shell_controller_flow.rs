use std::sync::Mutex;

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

struct MemoryDaemonManager {
    status: Mutex<DaemonStatus>,
}

impl Default for MemoryDaemonManager {
    fn default() -> Self {
        Self {
            status: Mutex::new(DaemonStatus::NotInstalled),
        }
    }
}

impl DaemonManager for MemoryDaemonManager {
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
        *self.status.lock().expect("status") = DaemonStatus::Running;
        Ok(())
    }

    fn stop(&self, _request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
        *self.status.lock().expect("status") = DaemonStatus::Stopped(None);
        Ok(())
    }

    fn status(&self, _request: DaemonLabelRequest) -> Result<DaemonStatus, DaemonManagerError> {
        Ok(self.status.lock().expect("status").clone())
    }
}

#[derive(Default)]
struct MemoryBridge {
    renders: usize,
    last_status_content: Option<String>,
    last_settings_content: Option<String>,
    quit_requested: bool,
}

impl DesktopShellBridge for MemoryBridge {
    fn render_menu(&mut self, _view: &retaia_agent::GuiMenuView) {
        self.renders += 1;
    }

    fn open_status_window(&mut self, content: &str) {
        self.last_status_content = Some(content.to_string());
    }

    fn open_settings_panel(&mut self, content: &str) {
        self.last_settings_content = Some(content.to_string());
    }

    fn request_quit(&mut self) {
        self.quit_requested = true;
    }
}

#[test]
fn e2e_desktop_shell_controller_flow_uses_shared_runtime_engine_and_daemon_port() {
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let manager = MemoryDaemonManager {
        status: Mutex::new(DaemonStatus::NotInstalled),
    };
    let mut controller = DesktopShellController::with_default_user_daemon(session, manager);
    let mut bridge = MemoryBridge::default();

    controller.render_initial_menu(&mut bridge);
    controller
        .handle_action(GuiMenuAction::StartDaemon, &mut bridge)
        .expect("start");
    controller
        .handle_action(GuiMenuAction::OpenStatusWindow, &mut bridge)
        .expect("status");
    controller
        .handle_action(GuiMenuAction::OpenSettings, &mut bridge)
        .expect("settings");
    controller
        .handle_action(GuiMenuAction::Quit, &mut bridge)
        .expect("quit");

    assert_eq!(controller.daemon_status(), Some(&DaemonStatus::Running));
    assert!(
        bridge
            .last_status_content
            .unwrap_or_default()
            .contains("run_state=")
    );
    assert!(
        bridge
            .last_settings_content
            .unwrap_or_default()
            .contains("core_api_url=")
    );
    assert!(bridge.quit_requested);
    assert!(bridge.renders >= 5);
}

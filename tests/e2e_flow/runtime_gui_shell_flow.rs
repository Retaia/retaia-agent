use std::sync::Mutex;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, DaemonLabelRequest, DaemonManager,
    DaemonManagerError, DaemonStatus, LogLevel, RuntimeSession, apply_gui_menu_action,
    settings_panel_content, status_window_content,
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

struct MemoryDaemonManager {
    events: Mutex<Vec<String>>,
    status: Mutex<DaemonStatus>,
}

impl Default for MemoryDaemonManager {
    fn default() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            status: Mutex::new(DaemonStatus::NotInstalled),
        }
    }
}

impl MemoryDaemonManager {
    fn events(&self) -> Vec<String> {
        self.events.lock().expect("events lock").clone()
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
        self.events
            .lock()
            .expect("events lock")
            .push("start".to_string());
        *self.status.lock().expect("status lock") = DaemonStatus::Running;
        Ok(())
    }

    fn stop(&self, _request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
        self.events
            .lock()
            .expect("events lock")
            .push("stop".to_string());
        *self.status.lock().expect("status lock") = DaemonStatus::Stopped(None);
        Ok(())
    }

    fn status(&self, _request: DaemonLabelRequest) -> Result<DaemonStatus, DaemonManagerError> {
        Ok(self.status.lock().expect("status lock").clone())
    }
}

#[test]
fn e2e_gui_menu_actions_and_daemon_controls_use_shared_runtime_session_and_daemon_port() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");
    let manager = MemoryDaemonManager::default();
    let daemon = GuiDaemonContext::default();

    let pause = apply_gui_menu_action(&mut session, &manager, &daemon, GuiMenuAction::Pause)
        .expect("pause should succeed");
    assert_eq!(format!("{:?}", pause.run_state), "Paused");

    let play = apply_gui_menu_action(&mut session, &manager, &daemon, GuiMenuAction::PlayResume)
        .expect("play should succeed");
    assert_eq!(format!("{:?}", play.run_state), "Running");

    let daemon_start =
        apply_gui_menu_action(&mut session, &manager, &daemon, GuiMenuAction::StartDaemon)
            .expect("daemon start should succeed");
    assert_eq!(daemon_start.daemon_status, Some(DaemonStatus::Running));

    let daemon_stop =
        apply_gui_menu_action(&mut session, &manager, &daemon, GuiMenuAction::StopDaemon)
            .expect("daemon stop should succeed");
    assert_eq!(daemon_stop.daemon_status, Some(DaemonStatus::Stopped(None)));

    let events = manager.events();
    assert_eq!(events, vec!["start".to_string(), "stop".to_string()]);

    let status_content = status_window_content(&session);
    let settings_content = settings_panel_content(&session);
    assert!(status_content.contains("run_state=running"));
    assert!(settings_content.contains("core_api_url=https://core.retaia.local/api/v1"));
}

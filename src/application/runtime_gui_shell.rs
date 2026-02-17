use crate::application::daemon_manager::{
    DaemonLabelRequest, DaemonLevel, DaemonManager, DaemonManagerError, DaemonStatus,
};
use crate::application::runtime_cli_shell::{format_settings, format_status};
use crate::application::runtime_session::RuntimeSession;
use crate::domain::runtime_ui::{AgentRunState, MenuAction};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiMenuAction {
    OpenStatusWindow,
    OpenSettings,
    PlayResume,
    Pause,
    Stop,
    StartDaemon,
    StopDaemon,
    RefreshDaemonStatus,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiDaemonContext {
    pub label: String,
    pub level: DaemonLevel,
}

impl Default for GuiDaemonContext {
    fn default() -> Self {
        Self {
            label: "io.retaia.agent".to_string(),
            level: DaemonLevel::User,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiMenuView {
    pub run_state: AgentRunState,
    pub show_play_resume: bool,
    pub show_pause: bool,
    pub can_play_resume: bool,
    pub can_pause: bool,
    pub can_stop: bool,
    pub daemon_status: Option<DaemonStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiActionOutcome {
    pub run_state: AgentRunState,
    pub daemon_status: Option<DaemonStatus>,
    pub open_status_window: bool,
    pub open_settings_panel: bool,
    pub should_quit: bool,
}

#[derive(Debug, Error)]
pub enum RuntimeGuiShellError {
    #[error("daemon operation failed: {0}")]
    Daemon(DaemonManagerError),
}

pub fn menu_view(session: &RuntimeSession, daemon_status: Option<DaemonStatus>) -> GuiMenuView {
    let model = session.tray_menu_model();
    GuiMenuView {
        run_state: session.run_state(),
        show_play_resume: model.visibility.show_play_resume,
        show_pause: model.visibility.show_pause,
        can_play_resume: model.availability.can_play_resume,
        can_pause: model.availability.can_pause,
        can_stop: model.availability.can_stop,
        daemon_status,
    }
}

pub fn status_window_content(session: &RuntimeSession) -> String {
    format_status(session)
}

pub fn settings_panel_content(session: &RuntimeSession) -> String {
    format_settings(session.settings())
}

pub fn apply_gui_menu_action<M: DaemonManager>(
    session: &mut RuntimeSession,
    manager: &M,
    daemon: &GuiDaemonContext,
    action: GuiMenuAction,
) -> Result<GuiActionOutcome, RuntimeGuiShellError> {
    let base_run_state = session.run_state();
    let mut outcome = GuiActionOutcome {
        run_state: base_run_state,
        daemon_status: None,
        open_status_window: false,
        open_settings_panel: false,
        should_quit: false,
    };

    match action {
        GuiMenuAction::OpenStatusWindow => {
            outcome.open_status_window = true;
        }
        GuiMenuAction::OpenSettings => {
            outcome.open_settings_panel = true;
        }
        GuiMenuAction::PlayResume => {
            outcome.run_state = session.on_menu_action(MenuAction::PlayResume);
        }
        GuiMenuAction::Pause => {
            outcome.run_state = session.on_menu_action(MenuAction::Pause);
        }
        GuiMenuAction::Stop => {
            outcome.run_state = session.on_menu_action(MenuAction::Stop);
        }
        GuiMenuAction::StartDaemon => {
            manager
                .start(daemon_label_request(daemon))
                .map_err(RuntimeGuiShellError::Daemon)?;
            outcome.daemon_status = Some(
                manager
                    .status(daemon_label_request(daemon))
                    .map_err(RuntimeGuiShellError::Daemon)?,
            );
        }
        GuiMenuAction::StopDaemon => {
            manager
                .stop(daemon_label_request(daemon))
                .map_err(RuntimeGuiShellError::Daemon)?;
            outcome.daemon_status = Some(
                manager
                    .status(daemon_label_request(daemon))
                    .map_err(RuntimeGuiShellError::Daemon)?,
            );
        }
        GuiMenuAction::RefreshDaemonStatus => {
            outcome.daemon_status = Some(
                manager
                    .status(daemon_label_request(daemon))
                    .map_err(RuntimeGuiShellError::Daemon)?,
            );
        }
        GuiMenuAction::Quit => {
            outcome.should_quit = true;
        }
    }

    Ok(outcome)
}

fn daemon_label_request(daemon: &GuiDaemonContext) -> DaemonLabelRequest {
    DaemonLabelRequest {
        label: daemon.label.clone(),
        level: daemon.level,
    }
}

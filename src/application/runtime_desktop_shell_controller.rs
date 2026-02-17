use crate::application::daemon_manager::{DaemonLevel, DaemonManager, DaemonStatus};
use crate::application::runtime_gui_shell::{
    GuiDaemonContext, GuiMenuAction, GuiMenuView, RuntimeGuiShellError, apply_gui_menu_action,
    menu_view, settings_panel_content, status_window_content,
};
use crate::application::runtime_session::RuntimeSession;

pub trait DesktopShellBridge {
    fn render_menu(&mut self, view: &GuiMenuView);
    fn open_status_window(&mut self, content: &str);
    fn open_settings_panel(&mut self, content: &str);
    fn request_quit(&mut self);
}

pub struct DesktopShellController<M: DaemonManager> {
    session: RuntimeSession,
    daemon_manager: M,
    daemon_context: GuiDaemonContext,
    daemon_status: Option<DaemonStatus>,
}

impl<M: DaemonManager> DesktopShellController<M> {
    pub fn new(
        session: RuntimeSession,
        daemon_manager: M,
        daemon_context: GuiDaemonContext,
    ) -> Self {
        Self {
            session,
            daemon_manager,
            daemon_context,
            daemon_status: None,
        }
    }

    pub fn with_default_user_daemon(session: RuntimeSession, daemon_manager: M) -> Self {
        Self::new(
            session,
            daemon_manager,
            GuiDaemonContext {
                label: "io.retaia.agent".to_string(),
                level: DaemonLevel::User,
            },
        )
    }

    pub fn session(&self) -> &RuntimeSession {
        &self.session
    }

    pub fn daemon_status(&self) -> Option<&DaemonStatus> {
        self.daemon_status.as_ref()
    }

    pub fn render_initial_menu<B: DesktopShellBridge>(&self, bridge: &mut B) {
        bridge.render_menu(&menu_view(&self.session, self.daemon_status.clone()));
    }

    pub fn handle_action<B: DesktopShellBridge>(
        &mut self,
        action: GuiMenuAction,
        bridge: &mut B,
    ) -> Result<(), RuntimeGuiShellError> {
        let outcome = apply_gui_menu_action(
            &mut self.session,
            &self.daemon_manager,
            &self.daemon_context,
            action,
        )?;

        if let Some(status) = outcome.daemon_status {
            self.daemon_status = Some(status);
        }

        if outcome.open_status_window {
            bridge.open_status_window(&status_window_content(&self.session));
        }

        if outcome.open_settings_panel {
            bridge.open_settings_panel(&settings_panel_content(&self.session));
        }

        if outcome.should_quit {
            bridge.request_quit();
        }

        bridge.render_menu(&menu_view(&self.session, self.daemon_status.clone()));
        Ok(())
    }
}

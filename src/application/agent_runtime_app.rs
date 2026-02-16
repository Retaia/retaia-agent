use crate::application::config_repository::{ConfigRepository, ConfigRepositoryError};
use crate::domain::configuration::{
    AgentRuntimeConfig, ConfigValidationError, compact_validation_reason, validate_config,
};
use crate::domain::runtime_control::{
    RuntimeControlAvailability, RuntimeControlCommand, apply_runtime_control,
    runtime_control_availability,
};
use crate::domain::runtime_ui::{
    AgentRunState, AgentUiRuntime, JobStatus, MenuAction, MenuVisibility, RuntimeSnapshot,
    SystemNotification, base_menu_actions, menu_visibility,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayMenuModel {
    pub visibility: MenuVisibility,
    pub availability: RuntimeControlAvailability,
    pub actions: Vec<MenuAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStatusView {
    pub run_state: AgentRunState,
    pub current_job: Option<JobStatus>,
}

#[derive(Debug, Clone)]
pub struct AgentRuntimeApp {
    run_state: AgentRunState,
    ui_runtime: AgentUiRuntime,
    settings: AgentRuntimeConfig,
    latest_snapshot: RuntimeSnapshot,
}

#[derive(Debug, Error)]
pub enum SettingsSaveError {
    #[error("settings validation failed")]
    Validation(SystemNotification),
    #[error("settings repository error: {0}")]
    Repository(ConfigRepositoryError),
}

impl AgentRuntimeApp {
    pub fn new(settings: AgentRuntimeConfig) -> Result<Self, Vec<ConfigValidationError>> {
        validate_config(&settings)?;
        Ok(Self {
            run_state: AgentRunState::Running,
            ui_runtime: AgentUiRuntime::new(),
            settings,
            latest_snapshot: RuntimeSnapshot::default(),
        })
    }

    pub fn run_state(&self) -> AgentRunState {
        self.run_state
    }

    pub fn load_from_repository<R: ConfigRepository>(
        repository: &R,
    ) -> Result<Self, ConfigRepositoryError> {
        let settings = repository.load()?;
        Self::new(settings).map_err(ConfigRepositoryError::Validation)
    }

    pub fn settings(&self) -> &AgentRuntimeConfig {
        &self.settings
    }

    pub fn tray_menu_model(&self) -> TrayMenuModel {
        TrayMenuModel {
            visibility: menu_visibility(self.run_state),
            availability: runtime_control_availability(self.run_state),
            actions: base_menu_actions(),
        }
    }

    pub fn status_view(&self) -> RuntimeStatusView {
        RuntimeStatusView {
            run_state: self.run_state,
            current_job: AgentUiRuntime::status_window_job(&self.latest_snapshot).cloned(),
        }
    }

    pub fn apply_menu_action(&mut self, action: MenuAction) {
        let maybe_command = match action {
            MenuAction::PlayResume => Some(RuntimeControlCommand::PlayResume),
            MenuAction::Pause => Some(RuntimeControlCommand::Pause),
            MenuAction::Stop => Some(RuntimeControlCommand::Stop),
            MenuAction::Quit | MenuAction::OpenStatusWindow | MenuAction::OpenSettings => None,
        };

        if let Some(command) = maybe_command {
            self.run_state = apply_runtime_control(self.run_state, command);
        }
    }

    pub fn update_snapshot(&mut self, snapshot: RuntimeSnapshot) -> Vec<SystemNotification> {
        self.latest_snapshot = snapshot.clone();
        self.ui_runtime.update_snapshot(snapshot)
    }

    pub fn save_settings(
        &mut self,
        new_settings: AgentRuntimeConfig,
    ) -> Result<SystemNotification, SystemNotification> {
        match validate_config(&new_settings) {
            Ok(()) => {
                self.settings = new_settings;
                Ok(self.ui_runtime.notify_settings_saved())
            }
            Err(errors) => {
                let reason = compact_validation_reason(&errors);
                Err(self
                    .ui_runtime
                    .notify_settings_invalid(&reason)
                    .unwrap_or(SystemNotification::SettingsInvalid { reason }))
            }
        }
    }

    pub fn save_settings_with_repository<R: ConfigRepository>(
        &mut self,
        new_settings: AgentRuntimeConfig,
        repository: &R,
    ) -> Result<SystemNotification, SettingsSaveError> {
        match validate_config(&new_settings) {
            Ok(()) => {
                repository
                    .save(&new_settings)
                    .map_err(SettingsSaveError::Repository)?;
                self.settings = new_settings;
                Ok(self.ui_runtime.notify_settings_saved())
            }
            Err(errors) => {
                let reason = compact_validation_reason(&errors);
                Err(SettingsSaveError::Validation(
                    self.ui_runtime
                        .notify_settings_invalid(&reason)
                        .unwrap_or(SystemNotification::SettingsInvalid { reason }),
                ))
            }
        }
    }
}

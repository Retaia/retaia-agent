pub mod application;
pub mod domain;

pub use application::agent_runtime_app::{AgentRuntimeApp, RuntimeStatusView, TrayMenuModel};
pub use domain::configuration::{
    AgentRuntimeConfig, AuthMode, ConfigField, ConfigInterface, ConfigValidationError, LogLevel,
    RuntimeConfigUpdate, TechnicalAuthConfig, apply_config_update, compact_validation_reason,
    supported_config_fields, validate_config,
};
pub use domain::feature_flags::{
    ClientKind, can_issue_client_token, can_process_jobs, resolve_effective_features,
};
pub use domain::runtime_control::{
    RuntimeControlAvailability, RuntimeControlCommand, apply_runtime_control,
    runtime_control_availability,
};
pub use domain::runtime_ui::{
    AgentRunState, AgentUiRuntime, ConnectivityState, JobFailure, JobStage, JobStatus, MenuAction,
    MenuVisibility, RuntimeSnapshot, SystemNotification, base_menu_actions, menu_visibility,
};

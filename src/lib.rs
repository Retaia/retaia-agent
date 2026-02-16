pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::agent_registration::{
    AgentRegistrationCommand, AgentRegistrationError, AgentRegistrationGateway,
    AgentRegistrationIntent, AgentRegistrationOutcome, build_agent_registration_command,
    register_agent,
};
pub use application::agent_runtime_app::{
    AgentRuntimeApp, RuntimeStatusView, SettingsSaveError, TrayMenuModel,
};
pub use application::config_repository::{ConfigRepository, ConfigRepositoryError};
pub use application::core_api_gateway::{
    CoreApiGateway, CoreApiGatewayError, CoreJobState, CoreJobView,
    filter_jobs_for_declared_capabilities, poll_runtime_snapshot,
    runtime_snapshot_from_polled_jobs,
};
pub use application::daemon_manager::{
    DaemonInstallRequest, DaemonLabelRequest, DaemonLevel, DaemonManager, DaemonManagerError,
    DaemonStatus,
};
pub use application::notification_bridge::{
    NotificationBridgeError, NotificationDispatchReport, NotificationMessage, NotificationSink,
    dispatch_notifications, notification_message,
};
pub use application::runtime_cli_shell::{
    ShellCommand, ShellCommandResult, execute_shell_command, format_menu, format_settings,
    format_status, help_text, parse_shell_command,
};
pub use application::runtime_loop_engine::RuntimeLoopEngine;
pub use application::runtime_session::{RuntimeNotificationReport, RuntimeSession};
pub use application::runtime_sync_coordinator::{RuntimeSyncCoordinator, RuntimeSyncPlan};
pub use domain::capabilities::{
    AgentCapability, declared_agent_capabilities, has_required_capabilities,
};
pub use domain::configuration::{
    AgentRuntimeConfig, AuthMode, ConfigField, ConfigInterface, ConfigValidationError, LogLevel,
    RuntimeConfigUpdate, TechnicalAuthConfig, apply_config_update, compact_validation_reason,
    normalize_core_api_url, supported_config_fields, validate_config,
};
pub use domain::feature_flags::{
    ClientKind, can_issue_client_token, can_process_jobs, resolve_effective_features,
};
pub use domain::runtime_control::{
    RuntimeControlAvailability, RuntimeControlCommand, apply_runtime_control,
    runtime_control_availability,
};
pub use domain::runtime_orchestration::{
    ClientRuntimeTarget, PollDecision, PollDecisionReason, PollEndpoint, PollSignal, PushChannel,
    PushHint, PushHintDecision, RuntimeOrchestrationMode, can_issue_mutation_after_poll,
    is_push_channel_supported_for_target, is_push_hint_fresh, mobile_push_allowed_for_target,
    next_poll_decision, push_channels_allowed, push_is_authoritative, runtime_orchestration_mode,
    should_trigger_poll_from_push, throttled_backoff_with_jitter,
};
pub use domain::runtime_status_tracker::{RuntimeStatusEvent, RuntimeStatusTracker};
pub use domain::runtime_sync::{PushProcessResult, RuntimeSyncState};
pub use domain::runtime_ui::{
    AgentRunState, AgentUiRuntime, ConnectivityState, JobFailure, JobStage, JobStatus, MenuAction,
    MenuVisibility, RuntimeSnapshot, SystemNotification, base_menu_actions, menu_visibility,
};
pub use infrastructure::config_repository::{FileConfigRepository, SystemConfigRepository};
pub use infrastructure::config_store::{
    CONFIG_FILE_ENV, CONFIG_FILE_NAME, ConfigStoreError, load_config_from_path, load_system_config,
    save_config_to_path, save_system_config, system_config_file_path,
};
pub use infrastructure::notification_sink::{
    StdoutNotificationSink, SystemNotificationSink, dispatch_system_notification,
};
#[cfg(feature = "core-api-client")]
pub use infrastructure::openapi_agent_registration_gateway::OpenApiAgentRegistrationGateway;
#[cfg(feature = "core-api-client")]
pub use infrastructure::openapi_client::{build_core_api_client, with_bearer_token};
#[cfg(feature = "core-api-client")]
pub use infrastructure::openapi_jobs_gateway::OpenApiJobsGateway;
#[cfg(feature = "tauri-notifications")]
pub use infrastructure::tauri_notification_sink::TauriNotificationSink;

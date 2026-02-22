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
pub use application::derived_job_executor::{
    DerivedExecutionPlan, DerivedExecutionPlanner, DerivedExecutionReport, DerivedJobExecutorError,
    DerivedUploadPlan, execute_derived_job_once,
};
pub use application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedJobType, DerivedKind, DerivedManifestItem, DerivedProcessingError,
    DerivedProcessingGateway, DerivedUploadComplete, DerivedUploadInit, DerivedUploadPart,
    HeartbeatReceipt, SubmitDerivedPayload, validate_derived_upload_init,
};
pub use application::notification_bridge::{
    NotificationBridgeError, NotificationDispatchReport, NotificationMessage, NotificationSink,
    dispatch_notifications, notification_message,
};
pub use application::proxy_generator::{
    AudioProxyFormat, AudioProxyRequest, PhotoProxyFormat, PhotoProxyRequest, ProxyGenerationError,
    ProxyGenerator, VideoProxyRequest,
};
pub use application::runtime_cli_shell::{
    ShellCommand, ShellCommandResult, execute_shell_command, format_menu, format_settings,
    format_status, help_text, parse_shell_command,
};
pub use application::runtime_desktop_shell_controller::{
    DesktopShellBridge, DesktopShellController,
};
pub use application::runtime_gui_shell::{
    GuiActionOutcome, GuiDaemonContext, GuiMenuAction, GuiMenuView, RuntimeGuiShellError,
    apply_gui_menu_action, menu_view, settings_panel_content, status_window_content,
};
pub use application::runtime_loop_engine::RuntimeLoopEngine;
pub use application::runtime_poll_cycle::{
    RuntimePollCycleOutcome, RuntimePollCycleStatus, run_runtime_poll_cycle,
};
pub use application::runtime_session::{RuntimeNotificationReport, RuntimeSession};
pub use application::runtime_sync_coordinator::{RuntimeSyncCoordinator, RuntimeSyncPlan};
pub use domain::capabilities::{
    AgentCapability, declared_agent_capabilities, declared_agent_capabilities_with_ffmpeg,
    declared_agent_capabilities_with_runtime, ffmpeg_available, has_required_capabilities,
    photo_proxy_available, photo_source_extension_supported,
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
pub use infrastructure::ffmpeg_proxy_generator::{
    CommandOutput, CommandRunner, FfmpegProxyGenerator, StdCommandRunner, build_audio_proxy_args,
    build_video_proxy_args,
};
pub use infrastructure::notification_sink::{
    NotificationSinkProfile, RuntimeNotificationSink, StdoutNotificationSink,
    SystemNotificationSink, dispatch_system_notification, notification_sink_profile_for_target,
    select_notification_sink,
};
#[cfg(feature = "core-api-client")]
pub use infrastructure::openapi_agent_registration_gateway::OpenApiAgentRegistrationGateway;
#[cfg(feature = "core-api-client")]
pub use infrastructure::openapi_client::{build_core_api_client, with_bearer_token};
#[cfg(feature = "core-api-client")]
pub use infrastructure::openapi_derived_processing_gateway::OpenApiDerivedProcessingGateway;
#[cfg(feature = "core-api-client")]
pub use infrastructure::openapi_jobs_gateway::OpenApiJobsGateway;
pub use infrastructure::rust_photo_proxy_generator::{
    RawPhotoDecoder, RawloaderPhotoDecoder, RustPhotoProxyGenerator,
};
#[cfg(feature = "tauri-notifications")]
pub use infrastructure::tauri_notification_sink::TauriNotificationSink;

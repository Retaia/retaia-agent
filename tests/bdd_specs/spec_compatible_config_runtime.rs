use std::ffi::OsString;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use tempfile::tempdir;

use retaia_agent::{
    AgentRunState, AgentRuntimeApp, AgentRuntimeConfig, AuthMode, ClientRuntimeTarget,
    ConfigRepository, ConfigRepositoryError, CONFIG_FILE_ENV, ConnectivityState, JobStage,
    JobStatus, LogLevel, MenuAction, PollEndpoint, PollSignal, RuntimeSession, RuntimeSnapshot,
    RuntimeSyncPlan, SystemConfigRepository, SystemNotification, SystemNotificationSink,
};

fn settings() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

static CONFIG_ENV_GUARD: Mutex<()> = Mutex::new(());

struct ConfigEnvOverride {
    _guard: MutexGuard<'static, ()>,
    previous: Option<OsString>,
}

impl ConfigEnvOverride {
    fn set(path: &Path) -> Self {
        let guard = CONFIG_ENV_GUARD
            .lock()
            .expect("config env mutex should not be poisoned");
        let previous = std::env::var_os(CONFIG_FILE_ENV);
        unsafe {
            std::env::set_var(CONFIG_FILE_ENV, path.as_os_str());
        }
        Self {
            _guard: guard,
            previous,
        }
    }
}

impl Drop for ConfigEnvOverride {
    fn drop(&mut self) {
        match self.previous.as_ref() {
            Some(value) => unsafe {
                std::env::set_var(CONFIG_FILE_ENV, value);
            },
            None => unsafe {
                std::env::remove_var(CONFIG_FILE_ENV);
            },
        }
    }
}

#[test]
fn bdd_given_repository_backed_app_when_loading_then_runtime_menu_and_status_are_available() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    let _override = ConfigEnvOverride::set(&config_path);

    let repository = SystemConfigRepository;
    repository
        .save(&settings())
        .expect("system config should be persisted");

    let app = AgentRuntimeApp::load_from_repository(&repository).expect("app should load");
    assert_eq!(app.run_state(), AgentRunState::Running);
    assert_eq!(app.settings().core_api_url, "https://core.retaia.local");

    let tray = app.tray_menu_model();
    assert!(!tray.visibility.show_play_resume);
    assert!(tray.visibility.show_pause);

    let status = app.status_view();
    assert_eq!(status.run_state, AgentRunState::Running);
    assert!(status.current_job.is_none());
}

#[test]
fn bdd_given_runtime_app_save_settings_when_valid_then_saved_when_invalid_then_rejected() {
    let mut app = AgentRuntimeApp::new(settings()).expect("valid app");

    let mut valid = settings();
    valid.max_parallel_jobs = 4;
    let saved = app
        .save_settings(valid)
        .expect("valid settings should be accepted");
    assert_eq!(saved, SystemNotification::SettingsSaved);

    let mut invalid = settings();
    invalid.core_api_url = "".to_string();
    let invalid_result = app
        .save_settings(invalid)
        .expect_err("invalid settings should be rejected");
    assert!(matches!(
        invalid_result,
        SystemNotification::SettingsInvalid { .. }
    ));
}

#[test]
fn bdd_given_runtime_session_poll_flow_when_throttled_and_success_then_sync_plans_match_specs() {
    let mut session =
        RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session should build");

    assert_eq!(session.run_state(), AgentRunState::Running);
    assert_eq!(session.target(), ClientRuntimeTarget::Agent);
    assert_eq!(session.settings().ollama_url, "http://127.0.0.1:11434");

    let tray = session.tray_menu_model();
    assert!(tray.visibility.show_pause);
    let status = session.status_view();
    assert_eq!(status.run_state, AgentRunState::Running);

    let throttled = session.on_poll_throttled(PollEndpoint::Jobs, PollSignal::SlowDown429, 2, 7);
    assert!(matches!(throttled, RuntimeSyncPlan::SchedulePoll(_)));

    let success = session.on_poll_success(PollEndpoint::Jobs, 2_000, true);
    assert!(matches!(success, RuntimeSyncPlan::SchedulePoll(_)));
    assert!(session.can_issue_mutation());

    let paused = session.on_menu_action(MenuAction::Pause);
    assert_eq!(paused, AgentRunState::Paused);
}

#[test]
fn bdd_given_runtime_session_snapshot_when_updated_then_status_view_exposes_current_job() {
    let mut session =
        RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session should build");

    let snapshot = RuntimeSnapshot {
        known_job_ids: ["job-100".to_string()].into_iter().collect(),
        running_job_ids: ["job-100".to_string()].into_iter().collect(),
        failed_jobs: Vec::new(),
        connectivity: ConnectivityState::Connected,
        auth_reauth_required: false,
        available_update: None,
        current_job: Some(JobStatus {
            job_id: "job-100".to_string(),
            asset_uuid: "asset-1".to_string(),
            progress_percent: 55,
            stage: JobStage::Processing,
            short_status: "processing".to_string(),
        }),
    };

    let _ = session.update_snapshot(snapshot);
    let status = session.status_view();
    let current = status.current_job.expect("current job should be shown");
    assert_eq!(current.job_id, "job-100");
    assert_eq!(current.progress_percent, 55);
}

#[test]
fn bdd_given_system_config_repository_when_path_overridden_then_save_load_and_path_are_consistent() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("state").join("agent-config.toml");
    let _override = ConfigEnvOverride::set(&config_path);

    let repository = SystemConfigRepository;
    let reported_path = repository
        .config_path()
        .expect("config path should resolve from env override");
    assert_eq!(reported_path, config_path);

    repository
        .save(&settings())
        .expect("save should use overridden path");
    let loaded = repository.load().expect("load should succeed");

    assert_eq!(loaded.core_api_url, "https://core.retaia.local");
    assert!(config_path.exists());
}

#[test]
fn bdd_given_system_notification_sink_constructors_when_initialized_then_runtime_sink_is_available() {
    let _sink = SystemNotificationSink::new();
    let _default_sink = SystemNotificationSink::default();
}

#[test]
fn bdd_given_system_repository_with_invalid_config_when_loading_then_validation_error_is_exposed() {
    let dir = tempdir().expect("tempdir");
    let config_path = dir.path().join("invalid-config.toml");
    let _override = ConfigEnvOverride::set(&config_path);

    std::fs::write(
        &config_path,
        r#"core_api_url = "https://core.retaia.local"
ollama_url = "http://127.0.0.1:11434"
auth_mode = "technical"
max_parallel_jobs = 2
log_level = "info"
"#,
    )
    .expect("invalid technical config should be written");

    let repository = SystemConfigRepository;
    let error = repository
        .load()
        .expect_err("missing technical credentials should fail validation");
    assert!(matches!(error, ConfigRepositoryError::Validation(_)));
}

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use tempfile::tempdir;

use retaia_agent::{
    AgentRunState, AgentRuntimeApp, AgentRuntimeConfig, AuthMode, ClientRuntimeTarget,
    ConfigRepository, ConfigRepositoryError, CONFIG_FILE_ENV, FileConfigRepository, LogLevel,
    MenuAction, NotificationMessage, NotificationSink, PollEndpoint, PollSignal, RuntimeSession,
    RuntimeSyncPlan, SettingsSaveError, StdoutNotificationSink, SystemConfigRepository,
    SystemNotification, dispatch_notifications, notification_message,
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

#[derive(Default)]
struct FailingRepo;

impl ConfigRepository for FailingRepo {
    fn load(&self) -> Result<AgentRuntimeConfig, ConfigRepositoryError> {
        Ok(settings())
    }

    fn save(&self, _config: &AgentRuntimeConfig) -> Result<(), ConfigRepositoryError> {
        Err(ConfigRepositoryError::Persistence("io failure".to_string()))
    }

    fn config_path(&self) -> Result<PathBuf, ConfigRepositoryError> {
        Ok(PathBuf::from("/tmp/retaia-agent/config.toml"))
    }
}

#[derive(Default)]
struct MemorySink {
    delivered: std::sync::Mutex<Vec<String>>,
}

impl NotificationSink for MemorySink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), retaia_agent::NotificationBridgeError> {
        self.delivered
            .lock()
            .expect("delivery lock")
            .push(message.title.clone());
        Ok(())
    }
}

#[test]
fn e2e_file_repository_roundtrip_and_missing_file_error_follow_runtime_contract() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("config.toml");
    let repository = FileConfigRepository::new(path.clone());
    repository.save(&settings()).expect("save should succeed");
    let loaded = repository.load().expect("load should succeed");
    assert_eq!(loaded.core_api_url, "https://core.retaia.local");

    let missing = FileConfigRepository::new(dir.path().join("missing.toml"));
    let error = missing.load().expect_err("missing file should fail");
    assert!(matches!(error, ConfigRepositoryError::NotFound));
}

#[test]
fn e2e_application_save_with_repository_failure_returns_repository_error() {
    let mut app = AgentRuntimeApp::new(settings()).expect("app");
    let error = app
        .save_settings_with_repository(settings(), &FailingRepo)
        .expect_err("save should fail");

    match error {
        SettingsSaveError::Repository(ConfigRepositoryError::Persistence(_)) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn e2e_notification_mapping_and_dispatch_covers_runtime_notification_contract() {
    let notifications = vec![
        SystemNotification::NewJobReceived {
            job_id: "job-1".to_string(),
        },
        SystemNotification::AllJobsDone,
        SystemNotification::JobFailed {
            job_id: "job-1".to_string(),
            error_code: "E_TIMEOUT".to_string(),
        },
        SystemNotification::AgentDisconnectedOrReconnecting,
        SystemNotification::AuthExpiredReauthRequired,
        SystemNotification::SettingsSaved,
        SystemNotification::SettingsInvalid {
            reason: "invalid ollama url".to_string(),
        },
        SystemNotification::UpdatesAvailable {
            version: "9.9.9".to_string(),
        },
    ];

    let sink = MemorySink::default();
    let report = dispatch_notifications(&sink, &notifications);
    assert_eq!(report.delivered, notifications.len());
    assert!(report.failed.is_empty());

    let mapped = notifications
        .iter()
        .map(notification_message)
        .map(|m| m.title)
        .collect::<Vec<_>>();
    assert!(mapped.contains(&"New job received".to_string()));
    assert!(mapped.contains(&"Auth expired / re-auth required".to_string()));
}

#[test]
fn e2e_runtime_session_status_and_settings_access_are_stable_for_spec_runtime_flow() {
    let session =
        RuntimeSession::new(retaia_agent::ClientRuntimeTarget::Agent, settings()).expect("session");
    assert_eq!(session.settings().max_parallel_jobs, 2);
    assert_eq!(
        session.status_view().run_state,
        retaia_agent::AgentRunState::Running
    );

    let stdout_sink = StdoutNotificationSink;
    let result = stdout_sink.send(
        &NotificationMessage {
            title: "All jobs done".to_string(),
            body: "No running jobs remain.".to_string(),
        },
        &SystemNotification::AllJobsDone,
    );
    assert!(result.is_ok());
}

#[test]
fn e2e_runtime_session_proxy_and_poll_paths_match_spec_runtime_contract() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    assert_eq!(session.run_state(), AgentRunState::Running);
    assert_eq!(session.target(), ClientRuntimeTarget::Agent);
    assert_eq!(session.settings().core_api_url, "https://core.retaia.local");
    assert!(session.tray_menu_model().visibility.show_pause);

    let throttled = session.on_poll_throttled(PollEndpoint::Jobs, PollSignal::SlowDown429, 1, 99);
    assert!(matches!(throttled, RuntimeSyncPlan::SchedulePoll(_)));

    let success = session.on_poll_success(PollEndpoint::Jobs, 2_000, true);
    assert!(matches!(success, RuntimeSyncPlan::SchedulePoll(_)));
    assert!(session.can_issue_mutation());

    let paused = session.on_menu_action(MenuAction::Pause);
    assert_eq!(paused, AgentRunState::Paused);
}

#[test]
fn e2e_agent_runtime_app_save_settings_covers_validation_and_success_paths() {
    let mut app = AgentRuntimeApp::new(settings()).expect("app");

    let mut valid = settings();
    valid.max_parallel_jobs = 3;
    let saved = app.save_settings(valid).expect("save should succeed");
    assert_eq!(saved, SystemNotification::SettingsSaved);

    let mut invalid = settings();
    invalid.ollama_url = "".to_string();
    let failed = app
        .save_settings(invalid)
        .expect_err("invalid settings should fail");
    assert!(matches!(failed, SystemNotification::SettingsInvalid { .. }));
}

#[test]
fn e2e_system_config_repository_overridden_path_roundtrip_matches_cli_gui_contract() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("config").join("agent.toml");
    let _override = ConfigEnvOverride::set(&path);

    let repository = SystemConfigRepository;
    assert_eq!(
        repository.config_path().expect("path should resolve"),
        path
    );

    repository.save(&settings()).expect("save should succeed");
    let loaded = repository.load().expect("load should succeed");
    assert_eq!(loaded.max_parallel_jobs, 2);
}

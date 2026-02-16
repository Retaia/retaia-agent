use std::cell::RefCell;
use std::path::PathBuf;

use tempfile::tempdir;

use retaia_agent::{
    AgentRunState, AgentRuntimeApp, AgentRuntimeConfig, AuthMode, ClientRuntimeTarget,
    ConfigRepository, ConfigRepositoryError, FileConfigRepository, LogLevel, MenuAction,
    NotificationMessage, NotificationSink, PollEndpoint, PollSignal, RuntimeControlCommand,
    RuntimeLoopEngine, RuntimeSession, RuntimeSyncPlan, SettingsSaveError, StdoutNotificationSink,
    SystemNotification, notification_message,
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

#[derive(Default)]
struct SaveErrorRepository;

impl ConfigRepository for SaveErrorRepository {
    fn load(&self) -> Result<AgentRuntimeConfig, ConfigRepositoryError> {
        Ok(settings())
    }

    fn save(&self, _config: &AgentRuntimeConfig) -> Result<(), ConfigRepositoryError> {
        Err(ConfigRepositoryError::Persistence(
            "disk write failed".to_string(),
        ))
    }

    fn config_path(&self) -> Result<PathBuf, ConfigRepositoryError> {
        Ok(PathBuf::from("/tmp/retaia-agent/config.toml"))
    }
}

#[derive(Default)]
struct CaptureSink {
    delivered_titles: RefCell<Vec<String>>,
}

impl NotificationSink for CaptureSink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), retaia_agent::NotificationBridgeError> {
        self.delivered_titles
            .borrow_mut()
            .push(message.title.clone());
        Ok(())
    }
}

#[test]
fn bdd_given_runtime_menu_non_control_actions_when_applied_then_run_state_is_unchanged() {
    let mut app = AgentRuntimeApp::new(settings()).expect("valid app");
    let initial = app.run_state();
    app.apply_menu_action(MenuAction::OpenSettings);
    app.apply_menu_action(MenuAction::OpenStatusWindow);
    app.apply_menu_action(MenuAction::Quit);
    assert_eq!(app.run_state(), initial);
}

#[test]
fn bdd_given_runtime_loop_stopped_when_poll_events_arrive_then_sync_plan_is_none() {
    let mut engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    let _ = engine.apply_control(RuntimeControlCommand::Stop);
    assert_eq!(engine.run_state(), AgentRunState::Stopped);

    let on_success = engine.on_poll_success(PollEndpoint::Jobs, 2_000, true);
    assert_eq!(on_success, RuntimeSyncPlan::None);

    let on_429 = engine.on_poll_throttled(PollEndpoint::Jobs, PollSignal::SlowDown429, 1, 7);
    assert_eq!(on_429, RuntimeSyncPlan::None);
}

#[test]
fn bdd_given_runtime_session_when_querying_target_and_settings_then_bootstrap_values_are_kept() {
    let session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    assert_eq!(session.target(), ClientRuntimeTarget::Agent);
    assert_eq!(session.settings().core_api_url, "https://core.retaia.local");
}

#[test]
fn bdd_given_repository_save_failure_when_saving_settings_then_repository_error_is_returned() {
    let mut app = AgentRuntimeApp::new(settings()).expect("valid app");
    let repository = SaveErrorRepository;

    let error = app
        .save_settings_with_repository(settings(), &repository)
        .expect_err("save must fail");

    match error {
        SettingsSaveError::Repository(ConfigRepositoryError::Persistence(_)) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn bdd_given_file_repository_config_path_when_requested_then_path_is_returned() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("config.toml");
    let repository = FileConfigRepository::new(path.clone());
    let reported = repository.config_path().expect("path should resolve");
    assert_eq!(reported, path);
}

#[test]
fn bdd_given_missing_file_repository_when_loading_then_not_found_error_is_returned() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("missing.toml");
    let repository = FileConfigRepository::new(path);
    let error = repository.load().expect_err("missing file should fail");
    assert!(matches!(error, ConfigRepositoryError::NotFound));
}

#[test]
fn bdd_given_invalid_file_repository_when_loading_then_invalid_data_error_is_returned() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("invalid.toml");
    std::fs::write(&path, "not = { valid = toml").expect("write file");
    let repository = FileConfigRepository::new(path);
    let error = repository.load().expect_err("invalid file should fail");
    assert!(matches!(error, ConfigRepositoryError::InvalidData(_)));
}

#[test]
fn bdd_given_stdout_sink_when_sending_notification_then_delivery_succeeds() {
    let sink = StdoutNotificationSink;
    let result = sink.send(
        &NotificationMessage {
            title: "Settings saved".to_string(),
            body: "Configuration has been persisted.".to_string(),
        },
        &SystemNotification::SettingsSaved,
    );
    assert!(result.is_ok());
}

#[test]
fn bdd_given_all_runtime_notification_types_when_mapping_then_messages_follow_contract() {
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
            reason: "invalid core api url".to_string(),
        },
        SystemNotification::UpdatesAvailable {
            version: "1.2.3".to_string(),
        },
    ];

    let sink = CaptureSink::default();
    for notification in &notifications {
        let message = notification_message(notification);
        sink.send(&message, notification)
            .expect("sink should accept");
    }

    assert_eq!(sink.delivered_titles.borrow().len(), notifications.len());
    assert!(
        sink.delivered_titles
            .borrow()
            .contains(&"Auth expired / re-auth required".to_string())
    );
    assert!(
        sink.delivered_titles
            .borrow()
            .contains(&"Updates available".to_string())
    );
}

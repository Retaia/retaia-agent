use retaia_agent::{
    AgentRuntimeApp, AgentRuntimeConfig, AuthMode, ConfigValidationError, JobStage, JobStatus,
    LogLevel, MenuAction, RuntimeSnapshot, SystemNotification,
};

fn valid_settings() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

#[test]
fn tdd_application_rejects_invalid_bootstrap_settings() {
    let mut invalid = valid_settings();
    invalid.core_api_url = "no-scheme".to_string();

    let errors = AgentRuntimeApp::new(invalid).expect_err("app bootstrap should validate settings");
    assert!(errors.contains(&ConfigValidationError::InvalidCoreApiUrl));
}

#[test]
fn tdd_application_menu_actions_change_run_state_via_domain_control() {
    let mut app = AgentRuntimeApp::new(valid_settings()).expect("valid app");
    assert_eq!(app.run_state(), retaia_agent::AgentRunState::Running);

    app.apply_menu_action(MenuAction::Pause);
    assert_eq!(app.run_state(), retaia_agent::AgentRunState::Paused);

    app.apply_menu_action(MenuAction::PlayResume);
    assert_eq!(app.run_state(), retaia_agent::AgentRunState::Running);

    app.apply_menu_action(MenuAction::Stop);
    assert_eq!(app.run_state(), retaia_agent::AgentRunState::Stopped);
}

#[test]
fn tdd_application_status_view_tracks_current_job_from_latest_snapshot() {
    let mut app = AgentRuntimeApp::new(valid_settings()).expect("valid app");
    let snapshot = RuntimeSnapshot {
        current_job: Some(JobStatus {
            job_id: "job-11".to_string(),
            asset_uuid: "asset-1".to_string(),
            progress_percent: 52,
            stage: JobStage::Upload,
            short_status: "uploading".to_string(),
        }),
        ..RuntimeSnapshot::default()
    };
    app.update_snapshot(snapshot);

    let status = app.status_view();
    let job = status.current_job.expect("job should be exposed");
    assert_eq!(job.job_id, "job-11");
    assert_eq!(job.progress_percent, 52);
}

#[test]
fn tdd_application_save_settings_returns_saved_or_invalid_notification() {
    let mut app = AgentRuntimeApp::new(valid_settings()).expect("valid app");

    let saved = app
        .save_settings(valid_settings())
        .expect("valid settings should be saved");
    assert_eq!(saved, SystemNotification::SettingsSaved);

    let mut invalid = valid_settings();
    invalid.ollama_url = "invalid-url".to_string();
    let err = app
        .save_settings(invalid)
        .expect_err("invalid settings should fail");
    assert_eq!(
        err,
        SystemNotification::SettingsInvalid {
            reason: "invalid ollama url".to_string()
        }
    );
}

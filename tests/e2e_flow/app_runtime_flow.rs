use retaia_agent::{
    AgentRunState, AgentRuntimeApp, AgentRuntimeConfig, AuthMode, JobStage, JobStatus, LogLevel,
    MenuAction, RuntimeSnapshot, SystemNotification,
};

fn config() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 4,
        log_level: LogLevel::Info,
    }
}

#[test]
fn e2e_app_flow_menu_snapshot_and_notifications_stay_consistent() {
    let mut app = AgentRuntimeApp::new(config()).expect("valid app");
    assert_eq!(app.run_state(), AgentRunState::Running);

    app.apply_menu_action(MenuAction::Pause);
    assert_eq!(app.run_state(), AgentRunState::Paused);
    assert!(app.tray_menu_model().visibility.show_play_resume);
    assert!(!app.tray_menu_model().visibility.show_pause);

    let mut snapshot = RuntimeSnapshot::default();
    snapshot.known_job_ids.insert("job-007".to_string());
    snapshot.running_job_ids.insert("job-007".to_string());
    snapshot.current_job = Some(JobStatus {
        job_id: "job-007".to_string(),
        asset_uuid: "asset-007".to_string(),
        progress_percent: 18,
        stage: JobStage::Processing,
        short_status: "starting".to_string(),
    });

    let notifications = app.update_snapshot(snapshot);
    assert_eq!(
        notifications,
        vec![SystemNotification::NewJobReceived {
            job_id: "job-007".to_string()
        }]
    );

    let status = app.status_view();
    let current = status.current_job.expect("status window should expose job");
    assert_eq!(current.job_id, "job-007");
    assert_eq!(current.progress_percent, 18);

    app.apply_menu_action(MenuAction::PlayResume);
    assert_eq!(app.run_state(), AgentRunState::Running);
}

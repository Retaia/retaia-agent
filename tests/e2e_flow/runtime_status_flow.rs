use retaia_agent::{
    AgentRuntimeApp, AgentRuntimeConfig, AgentUiRuntime, AuthMode, ConnectivityState, JobStage,
    LogLevel, RuntimeStatusEvent, RuntimeStatusTracker, SystemNotification,
};

fn config() -> AgentRuntimeConfig {
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
fn e2e_runtime_status_tracker_projects_flow_for_status_window_and_notifications() {
    let mut tracker = RuntimeStatusTracker::new();
    let mut app = AgentRuntimeApp::new(config()).expect("app should be valid");
    let mut ui_runtime = AgentUiRuntime::new();

    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-101".to_string(),
        asset_uuid: "asset-101".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobProgress {
        job_id: "job-101".to_string(),
        asset_uuid: "asset-101".to_string(),
        progress_percent: 65,
        stage: JobStage::Submit,
        short_status: "submitting".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::ConnectivityChanged {
        connectivity: ConnectivityState::Disconnected,
    });

    let first_notifs = ui_runtime.update_snapshot(tracker.snapshot().clone());
    assert!(first_notifs.contains(&SystemNotification::NewJobReceived {
        job_id: "job-101".to_string()
    }));
    assert!(first_notifs.contains(&SystemNotification::AgentDisconnectedOrReconnecting));

    app.update_snapshot(tracker.snapshot().clone());
    let status = app.status_view();
    let current = status
        .current_job
        .expect("status window should have current job");
    assert_eq!(current.progress_percent, 65);
    assert_eq!(current.stage, JobStage::Submit);
    assert_eq!(current.short_status, "submitting");

    tracker.apply(RuntimeStatusEvent::JobCompleted {
        job_id: "job-101".to_string(),
    });
    let done_notifs = ui_runtime.update_snapshot(tracker.snapshot().clone());
    assert_eq!(done_notifs, vec![SystemNotification::AllJobsDone]);
}

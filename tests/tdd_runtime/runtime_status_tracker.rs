use retaia_agent::{
    ConnectivityState, JobStage, RuntimeStatusEvent, RuntimeStatusTracker, SystemNotification,
};

#[test]
fn tdd_status_tracker_claim_and_progress_expose_current_job_status() {
    let mut tracker = RuntimeStatusTracker::new();

    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-1".to_string(),
        asset_uuid: "asset-a".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobProgress {
        job_id: "job-1".to_string(),
        asset_uuid: "asset-a".to_string(),
        progress_percent: 42,
        stage: JobStage::Processing,
        short_status: "processing".to_string(),
    });

    let snapshot = tracker.snapshot();
    let current = snapshot
        .current_job
        .as_ref()
        .expect("current job should exist");
    assert_eq!(current.job_id, "job-1");
    assert_eq!(current.progress_percent, 42);
    assert_eq!(current.stage, JobStage::Processing);
}

#[test]
fn tdd_status_tracker_completion_switches_current_job_and_clears_when_empty() {
    let mut tracker = RuntimeStatusTracker::new();
    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-1".to_string(),
        asset_uuid: "asset-a".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-2".to_string(),
        asset_uuid: "asset-b".to_string(),
    });

    tracker.apply(RuntimeStatusEvent::JobCompleted {
        job_id: "job-1".to_string(),
    });
    let snapshot = tracker.snapshot();
    assert_eq!(
        snapshot.current_job.as_ref().map(|job| job.job_id.as_str()),
        Some("job-2")
    );

    tracker.apply(RuntimeStatusEvent::JobCompleted {
        job_id: "job-2".to_string(),
    });
    assert!(tracker.snapshot().current_job.is_none());
}

#[test]
fn tdd_status_tracker_failure_is_deduplicated_by_job_id() {
    let mut tracker = RuntimeStatusTracker::new();
    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-9".to_string(),
        asset_uuid: "asset-x".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobFailed {
        job_id: "job-9".to_string(),
        error_code: "E_TIMEOUT".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobFailed {
        job_id: "job-9".to_string(),
        error_code: "E_NETWORK".to_string(),
    });

    let failed = &tracker.snapshot().failed_jobs;
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].job_id, "job-9");
    assert_eq!(failed[0].error_code, "E_NETWORK");
}

#[test]
fn tdd_status_tracker_updates_connectivity_auth_and_version_fields() {
    let mut tracker = RuntimeStatusTracker::new();
    tracker.apply(RuntimeStatusEvent::ConnectivityChanged {
        connectivity: ConnectivityState::Reconnecting,
    });
    tracker.apply(RuntimeStatusEvent::AuthReauthRequired { required: true });
    tracker.apply(RuntimeStatusEvent::UpdateAvailable {
        version: Some("1.0.1".to_string()),
    });

    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.connectivity, ConnectivityState::Reconnecting);
    assert!(snapshot.auth_reauth_required);
    assert_eq!(snapshot.available_update.as_deref(), Some("1.0.1"));
}

#[test]
fn tdd_status_tracker_snapshot_can_feed_notifications() {
    let mut tracker = RuntimeStatusTracker::new();
    let mut ui = retaia_agent::AgentUiRuntime::new();

    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-12".to_string(),
        asset_uuid: "asset-12".to_string(),
    });
    let first = ui.update_snapshot(tracker.snapshot().clone());
    assert_eq!(
        first,
        vec![SystemNotification::NewJobReceived {
            job_id: "job-12".to_string()
        }]
    );

    tracker.apply(RuntimeStatusEvent::JobCompleted {
        job_id: "job-12".to_string(),
    });
    let second = ui.update_snapshot(tracker.snapshot().clone());
    assert_eq!(second, vec![SystemNotification::AllJobsDone]);
}

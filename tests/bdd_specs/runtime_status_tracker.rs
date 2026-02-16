use retaia_agent::{JobStage, RuntimeStatusEvent, RuntimeStatusTracker};

#[test]
fn bdd_given_claimed_job_when_progress_event_arrives_then_status_window_fields_match_specs() {
    let mut tracker = RuntimeStatusTracker::new();
    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-42".to_string(),
        asset_uuid: "asset-42".to_string(),
    });

    tracker.apply(RuntimeStatusEvent::JobProgress {
        job_id: "job-42".to_string(),
        asset_uuid: "asset-42".to_string(),
        progress_percent: 78,
        stage: JobStage::Upload,
        short_status: "uploading artifact".to_string(),
    });

    let snapshot = tracker.snapshot();
    let current = snapshot
        .current_job
        .as_ref()
        .expect("current job should exist");
    assert_eq!(current.progress_percent, 78);
    assert_eq!(current.stage, JobStage::Upload);
    assert_eq!(current.job_id, "job-42");
    assert_eq!(current.asset_uuid, "asset-42");
    assert_eq!(current.short_status, "uploading artifact");
}

#[test]
fn bdd_given_running_jobs_when_first_job_finishes_then_next_running_job_becomes_current() {
    let mut tracker = RuntimeStatusTracker::new();
    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-a".to_string(),
        asset_uuid: "asset-a".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-b".to_string(),
        asset_uuid: "asset-b".to_string(),
    });

    tracker.apply(RuntimeStatusEvent::JobCompleted {
        job_id: "job-a".to_string(),
    });

    let snapshot = tracker.snapshot();
    assert_eq!(
        snapshot.current_job.as_ref().map(|job| job.job_id.as_str()),
        Some("job-b")
    );
}

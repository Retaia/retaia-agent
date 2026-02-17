use retaia_agent::{
    CoreJobState, CoreJobView, filter_jobs_for_declared_capabilities,
    runtime_snapshot_from_polled_jobs,
};

#[test]
fn tdd_runtime_snapshot_from_polled_jobs_maps_known_running_and_current_job() {
    let jobs = vec![
        CoreJobView {
            job_id: "job-1".to_string(),
            asset_uuid: "asset-1".to_string(),
            state: CoreJobState::Pending,
            required_capabilities: vec!["media.facts@1".to_string()],
        },
        CoreJobView {
            job_id: "job-2".to_string(),
            asset_uuid: "asset-2".to_string(),
            state: CoreJobState::Claimed,
            required_capabilities: vec!["media.facts@1".to_string()],
        },
    ];

    let snapshot = runtime_snapshot_from_polled_jobs(&jobs);
    assert_eq!(snapshot.known_job_ids.len(), 2);
    assert!(snapshot.known_job_ids.contains("job-1"));
    assert!(snapshot.known_job_ids.contains("job-2"));
    assert_eq!(snapshot.running_job_ids.len(), 1);
    assert!(snapshot.running_job_ids.contains("job-2"));
    assert_eq!(
        snapshot.current_job.as_ref().map(|job| job.job_id.as_str()),
        Some("job-2")
    );
}

#[test]
fn tdd_runtime_snapshot_from_polled_jobs_maps_failed_jobs_to_notification_input() {
    let jobs = vec![CoreJobView {
        job_id: "job-fail".to_string(),
        asset_uuid: "asset-fail".to_string(),
        state: CoreJobState::Failed,
        required_capabilities: vec!["media.facts@1".to_string()],
    }];

    let snapshot = runtime_snapshot_from_polled_jobs(&jobs);
    assert_eq!(snapshot.failed_jobs.len(), 1);
    assert_eq!(snapshot.failed_jobs[0].job_id, "job-fail");
    assert_eq!(snapshot.failed_jobs[0].error_code, "JOB_FAILED_REMOTE");
}

#[test]
fn tdd_filter_jobs_for_declared_capabilities_keeps_only_subset_matches() {
    let jobs = vec![
        CoreJobView {
            job_id: "job-supported".to_string(),
            asset_uuid: "asset-1".to_string(),
            state: CoreJobState::Pending,
            required_capabilities: vec!["media.facts@1".to_string()],
        },
        CoreJobView {
            job_id: "job-unsupported".to_string(),
            asset_uuid: "asset-2".to_string(),
            state: CoreJobState::Pending,
            required_capabilities: vec!["media.unknown@1".to_string()],
        },
    ];

    let filtered = filter_jobs_for_declared_capabilities(jobs);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].job_id, "job-supported");
}

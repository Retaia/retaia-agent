use retaia_agent::{AgentUiRuntime, JobStage, JobStatus, RuntimeSnapshot, SystemNotification};

#[test]
fn e2e_status_window_and_notifications_work_across_poll_transitions() {
    let mut runtime = AgentUiRuntime::new();
    let mut first = RuntimeSnapshot::default();
    first.known_job_ids.insert("job-100".to_string());
    first.running_job_ids.insert("job-100".to_string());
    first.current_job = Some(JobStatus {
        job_id: "job-100".to_string(),
        asset_uuid: "asset-9".to_string(),
        progress_percent: 37,
        stage: JobStage::Processing,
        short_status: "transcoding".to_string(),
    });

    let first_notifs = runtime.update_snapshot(first.clone());
    assert_eq!(
        first_notifs,
        vec![SystemNotification::NewJobReceived {
            job_id: "job-100".to_string()
        }]
    );
    let current = AgentUiRuntime::status_window_job(&first).expect("current job missing");
    assert_eq!(current.progress_percent, 37);
    assert_eq!(current.short_status, "transcoding");

    let second = RuntimeSnapshot::default();
    let second_notifs = runtime.update_snapshot(second);
    assert_eq!(second_notifs, vec![SystemNotification::AllJobsDone]);
}

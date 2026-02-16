use retaia_agent::{AgentUiRuntime, JobFailure, RuntimeSnapshot, SystemNotification};

#[test]
fn bdd_given_new_job_when_polling_then_notify_new_job_once() {
    let mut runtime = AgentUiRuntime::new();
    let mut snapshot = RuntimeSnapshot::default();
    snapshot.known_job_ids.insert("job-42".to_string());

    let first = runtime.update_snapshot(snapshot.clone());
    assert_eq!(
        first,
        vec![SystemNotification::NewJobReceived {
            job_id: "job-42".to_string()
        }]
    );

    let second = runtime.update_snapshot(snapshot);
    assert!(second.is_empty());
}

#[test]
fn bdd_given_same_failed_job_on_poll_when_already_notified_then_no_repeat() {
    let mut runtime = AgentUiRuntime::new();
    let mut snapshot = RuntimeSnapshot::default();
    snapshot.failed_jobs.push(JobFailure {
        job_id: "job-failed".to_string(),
        error_code: "E_CODEC".to_string(),
    });

    let first = runtime.update_snapshot(snapshot.clone());
    assert_eq!(
        first,
        vec![SystemNotification::JobFailed {
            job_id: "job-failed".to_string(),
            error_code: "E_CODEC".to_string()
        }]
    );

    let second = runtime.update_snapshot(snapshot);
    assert!(second.is_empty());
}

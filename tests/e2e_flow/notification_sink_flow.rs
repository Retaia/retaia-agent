use retaia_agent::{
    AgentUiRuntime, BestEffortNotificationSink, NotificationBridgeError, NotificationMessage,
    RuntimeSnapshot, dispatch_notifications,
};

fn dispatcher_err(_message: &NotificationMessage) -> Result<(), NotificationBridgeError> {
    Err(NotificationBridgeError::SinkFailed(
        "system backend not available".to_string(),
    ))
}

#[test]
fn e2e_runtime_notifications_are_dispatched_once_with_system_fallback_sink() {
    let mut runtime = AgentUiRuntime::new();
    let sink = BestEffortNotificationSink::with_dispatcher(dispatcher_err);

    let mut started = RuntimeSnapshot::default();
    started.known_job_ids.insert("job-42".to_string());
    started.running_job_ids.insert("job-42".to_string());
    let start_notifications = runtime.update_snapshot(started);
    let start_report = dispatch_notifications(&sink, &start_notifications);
    assert_eq!(start_report.delivered, 1);
    assert!(start_report.failed.is_empty());

    let stable = RuntimeSnapshot {
        known_job_ids: ["job-42".to_string()].into_iter().collect(),
        running_job_ids: ["job-42".to_string()].into_iter().collect(),
        ..RuntimeSnapshot::default()
    };
    let stable_notifications = runtime.update_snapshot(stable);
    let stable_report = dispatch_notifications(&sink, &stable_notifications);
    assert_eq!(stable_report.delivered, 0);
    assert!(stable_report.failed.is_empty());

    let finished = RuntimeSnapshot::default();
    let done_notifications = runtime.update_snapshot(finished);
    let done_report = dispatch_notifications(&sink, &done_notifications);
    assert_eq!(done_report.delivered, 1);
    assert!(done_report.failed.is_empty());
}

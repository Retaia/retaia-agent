use retaia_agent::{
    AgentUiRuntime, RuntimeSnapshot, SystemNotificationSink, dispatch_notifications,
};

use crate::system_dispatcher_mock::{MockDispatcherScope, dispatch};

#[test]
fn e2e_runtime_notifications_are_dispatched_once_and_fail_when_system_sink_unavailable() {
    let mock = MockDispatcherScope::new();
    mock.set_error("system backend not available");
    let mut runtime = AgentUiRuntime::new();
    let sink = SystemNotificationSink::with_dispatcher(dispatch);

    let mut started = RuntimeSnapshot::default();
    started.known_job_ids.insert("job-42".to_string());
    started.running_job_ids.insert("job-42".to_string());
    let start_notifications = runtime.update_snapshot(started);
    let start_report = dispatch_notifications(&sink, &start_notifications);
    assert_eq!(start_report.delivered, 0);
    assert_eq!(start_report.failed.len(), 1);

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
    assert_eq!(done_report.delivered, 0);
    assert_eq!(done_report.failed.len(), 1);
    assert_eq!(mock.call_count(), 2);
}

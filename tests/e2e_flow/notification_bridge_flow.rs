use std::cell::RefCell;

use retaia_agent::{
    AgentUiRuntime, NotificationBridgeError, NotificationMessage, NotificationSink,
    RuntimeSnapshot, SystemNotification, dispatch_notifications,
};

#[derive(Default)]
struct CollectSink {
    titles: RefCell<Vec<String>>,
}

impl NotificationSink for CollectSink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        self.titles.borrow_mut().push(message.title.clone());
        Ok(())
    }
}

#[test]
fn e2e_notifications_from_runtime_snapshot_can_be_bridged_without_repetition() {
    let mut runtime = AgentUiRuntime::new();
    let sink = CollectSink::default();

    let mut first = RuntimeSnapshot::default();
    first.known_job_ids.insert("job-200".to_string());
    first.running_job_ids.insert("job-200".to_string());
    let first_notifications = runtime.update_snapshot(first);
    let first_report = dispatch_notifications(&sink, &first_notifications);
    assert_eq!(first_report.delivered, 1);
    assert_eq!(sink.titles.borrow().as_slice(), &["New job received"]);

    let same_state = RuntimeSnapshot {
        known_job_ids: ["job-200".to_string()].into_iter().collect(),
        running_job_ids: ["job-200".to_string()].into_iter().collect(),
        ..RuntimeSnapshot::default()
    };
    let repeated_notifications = runtime.update_snapshot(same_state);
    let repeated_report = dispatch_notifications(&sink, &repeated_notifications);
    assert_eq!(repeated_report.delivered, 0);

    let done_state = RuntimeSnapshot::default();
    let done_notifications = runtime.update_snapshot(done_state);
    let done_report = dispatch_notifications(&sink, &done_notifications);
    assert_eq!(done_report.delivered, 1);
    assert_eq!(
        sink.titles.borrow().as_slice(),
        &["New job received", "All jobs done"]
    );
}

use std::cell::RefCell;

use retaia_agent::{
    NotificationBridgeError, NotificationMessage, NotificationSink, SystemNotification,
    dispatch_notifications, notification_message,
};

#[derive(Default)]
struct MemorySink {
    delivered: RefCell<Vec<NotificationMessage>>,
}

impl NotificationSink for MemorySink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        self.delivered.borrow_mut().push(message.clone());
        Ok(())
    }
}

#[test]
fn tdd_notification_message_maps_required_notification_types() {
    let notif = SystemNotification::JobFailed {
        job_id: "job-1".to_string(),
        error_code: "E_IO".to_string(),
    };
    let message = notification_message(&notif);
    assert_eq!(message.title, "Job failed");
    assert!(message.body.contains("job-1"));
    assert!(message.body.contains("E_IO"));
}

#[test]
fn tdd_dispatch_notifications_delivers_every_item_to_sink() {
    let sink = MemorySink::default();
    let notifications = vec![
        SystemNotification::NewJobReceived {
            job_id: "job-10".to_string(),
        },
        SystemNotification::AllJobsDone,
    ];

    let report = dispatch_notifications(&sink, &notifications);
    assert_eq!(report.delivered, 2);
    assert!(report.failed.is_empty());
    assert_eq!(sink.delivered.borrow().len(), 2);
}

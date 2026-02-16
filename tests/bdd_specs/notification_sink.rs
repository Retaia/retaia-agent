use retaia_agent::{
    NotificationBridgeError, NotificationMessage, SystemNotification, SystemNotificationSink,
    dispatch_notifications,
};

fn dispatcher_err(_message: &NotificationMessage) -> Result<(), NotificationBridgeError> {
    Err(NotificationBridgeError::SinkFailed(
        "system notifications unsupported".to_string(),
    ))
}

#[test]
fn bdd_given_system_notification_dispatch_not_available_when_runtime_dispatches_then_delivery_is_reported_failed()
 {
    let sink = SystemNotificationSink::with_dispatcher(dispatcher_err);
    let notifications = vec![
        SystemNotification::NewJobReceived {
            job_id: "job-1".to_string(),
        },
        SystemNotification::AllJobsDone,
    ];

    let report = dispatch_notifications(&sink, &notifications);

    assert_eq!(report.delivered, 0);
    assert_eq!(report.failed.len(), 2);
}

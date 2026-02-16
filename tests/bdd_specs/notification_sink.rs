use retaia_agent::{
    BestEffortNotificationSink, NotificationBridgeError, NotificationMessage, SystemNotification,
    dispatch_notifications,
};

fn dispatcher_err(_message: &NotificationMessage) -> Result<(), NotificationBridgeError> {
    Err(NotificationBridgeError::SinkFailed(
        "system notifications unsupported".to_string(),
    ))
}

#[test]
fn bdd_given_system_notification_dispatch_not_available_when_runtime_dispatches_then_delivery_continues_with_no_bridge_failure()
 {
    let sink = BestEffortNotificationSink::with_dispatcher(dispatcher_err);
    let notifications = vec![
        SystemNotification::NewJobReceived {
            job_id: "job-1".to_string(),
        },
        SystemNotification::AllJobsDone,
    ];

    let report = dispatch_notifications(&sink, &notifications);

    assert_eq!(report.delivered, 2);
    assert!(report.failed.is_empty());
}

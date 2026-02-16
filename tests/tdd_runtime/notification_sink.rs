use retaia_agent::{
    BestEffortNotificationSink, NotificationBridgeError, NotificationMessage, NotificationSink,
    SystemNotification,
};

fn message() -> NotificationMessage {
    NotificationMessage {
        title: "New job received".to_string(),
        body: "Job job-1 is now available.".to_string(),
    }
}

fn source() -> SystemNotification {
    SystemNotification::NewJobReceived {
        job_id: "job-1".to_string(),
    }
}

fn dispatcher_ok(_message: &NotificationMessage) -> Result<(), NotificationBridgeError> {
    Ok(())
}

fn dispatcher_err(_message: &NotificationMessage) -> Result<(), NotificationBridgeError> {
    Err(NotificationBridgeError::SinkFailed(
        "notification backend unavailable".to_string(),
    ))
}

#[test]
fn tdd_given_system_dispatcher_ok_when_sending_then_notification_is_delivered() {
    let sink = BestEffortNotificationSink::with_dispatcher(dispatcher_ok);

    let result = sink.send(&message(), &source());

    assert!(result.is_ok());
}

#[test]
fn tdd_given_system_dispatcher_failure_when_sending_then_stdout_fallback_preserves_delivery() {
    let sink = BestEffortNotificationSink::with_dispatcher(dispatcher_err);

    let result = sink.send(&message(), &source());

    assert!(result.is_ok());
}

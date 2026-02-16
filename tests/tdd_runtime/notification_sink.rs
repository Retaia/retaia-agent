use retaia_agent::{
    NotificationMessage, NotificationSink, SystemNotification, SystemNotificationSink,
};

use crate::system_dispatcher_mock::{MockDispatcherScope, dispatch};

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

#[test]
fn tdd_given_system_dispatcher_ok_when_sending_then_notification_is_delivered() {
    let mock = MockDispatcherScope::new();
    mock.set_ok();
    let sink = SystemNotificationSink::with_dispatcher(dispatch);

    let result = sink.send(&message(), &source());

    assert!(result.is_ok());
    assert_eq!(mock.call_count(), 1);
    assert_eq!(mock.received_titles(), vec!["New job received".to_string()]);
}

#[test]
fn tdd_given_system_dispatcher_failure_when_sending_then_notification_fails() {
    let mock = MockDispatcherScope::new();
    mock.set_error("notification backend unavailable");
    let sink = SystemNotificationSink::with_dispatcher(dispatch);

    let result = sink.send(&message(), &source());

    assert!(result.is_err());
    assert_eq!(mock.call_count(), 1);
}

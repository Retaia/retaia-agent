use retaia_agent::{SystemNotification, SystemNotificationSink, dispatch_notifications};

use crate::system_dispatcher_mock::{MockDispatcherScope, dispatch};

#[test]
fn bdd_given_system_notification_dispatch_not_available_when_runtime_dispatches_then_delivery_is_reported_failed()
 {
    let mock = MockDispatcherScope::new();
    mock.set_error("system notifications unsupported");
    let sink = SystemNotificationSink::with_dispatcher(dispatch);
    let notifications = vec![
        SystemNotification::NewJobReceived {
            job_id: "job-1".to_string(),
        },
        SystemNotification::AllJobsDone,
    ];

    let report = dispatch_notifications(&sink, &notifications);

    assert_eq!(report.delivered, 0);
    assert_eq!(report.failed.len(), 2);
    assert_eq!(mock.call_count(), 2);
}

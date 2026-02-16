use std::cell::RefCell;

use retaia_agent::{
    NotificationBridgeError, NotificationMessage, NotificationSink, SystemNotification,
    dispatch_notifications,
};

struct FlakySink {
    fail_on_title: String,
    delivered_titles: RefCell<Vec<String>>,
}

impl NotificationSink for FlakySink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        if message.title == self.fail_on_title {
            return Err(NotificationBridgeError::SinkFailed(
                "sink failure".to_string(),
            ));
        }
        self.delivered_titles
            .borrow_mut()
            .push(message.title.clone());
        Ok(())
    }
}

#[test]
fn bdd_given_sink_failure_when_dispatching_notifications_then_failures_are_reported_and_flow_continues()
 {
    let sink = FlakySink {
        fail_on_title: "Job failed".to_string(),
        delivered_titles: RefCell::new(Vec::new()),
    };
    let notifications = vec![
        SystemNotification::NewJobReceived {
            job_id: "job-1".to_string(),
        },
        SystemNotification::JobFailed {
            job_id: "job-1".to_string(),
            error_code: "E_TIMEOUT".to_string(),
        },
        SystemNotification::AllJobsDone,
    ];

    let report = dispatch_notifications(&sink, &notifications);
    assert_eq!(report.delivered, 2);
    assert_eq!(report.failed.len(), 1);
    assert_eq!(
        sink.delivered_titles.borrow().as_slice(),
        &["New job received".to_string(), "All jobs done".to_string()]
    );
}

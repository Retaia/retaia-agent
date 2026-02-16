use crate::domain::runtime_ui::SystemNotification;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationMessage {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationBridgeError {
    SinkFailed(String),
}

pub trait NotificationSink {
    fn send(
        &self,
        message: &NotificationMessage,
        source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NotificationDispatchReport {
    pub delivered: usize,
    pub failed: Vec<SystemNotification>,
}

pub fn notification_message(notification: &SystemNotification) -> NotificationMessage {
    match notification {
        SystemNotification::NewJobReceived { job_id } => NotificationMessage {
            title: "New job received".to_string(),
            body: format!("Job {job_id} is now available."),
        },
        SystemNotification::AllJobsDone => NotificationMessage {
            title: "All jobs done".to_string(),
            body: "No running jobs remain.".to_string(),
        },
        SystemNotification::JobFailed { job_id, error_code } => NotificationMessage {
            title: "Job failed".to_string(),
            body: format!("Job {job_id} failed ({error_code})."),
        },
        SystemNotification::AgentDisconnectedOrReconnecting => NotificationMessage {
            title: "Agent disconnected / reconnecting".to_string(),
            body: "Backend connectivity changed, reconnect sequence started.".to_string(),
        },
        SystemNotification::AuthExpiredReauthRequired => NotificationMessage {
            title: "Auth expired / re-auth required".to_string(),
            body: "Runtime authentication is no longer valid.".to_string(),
        },
        SystemNotification::SettingsSaved => NotificationMessage {
            title: "Settings saved".to_string(),
            body: "Configuration has been persisted.".to_string(),
        },
        SystemNotification::SettingsInvalid { reason } => NotificationMessage {
            title: "Settings invalid".to_string(),
            body: reason.clone(),
        },
        SystemNotification::UpdatesAvailable { version } => NotificationMessage {
            title: "Updates available".to_string(),
            body: format!("Version {version} is available."),
        },
    }
}

pub fn dispatch_notifications<S: NotificationSink>(
    sink: &S,
    notifications: &[SystemNotification],
) -> NotificationDispatchReport {
    let mut report = NotificationDispatchReport::default();

    for notification in notifications {
        let message = notification_message(notification);
        match sink.send(&message, notification) {
            Ok(()) => report.delivered += 1,
            Err(_) => report.failed.push(notification.clone()),
        }
    }

    report
}

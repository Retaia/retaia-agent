use crate::application::notification_bridge::{
    NotificationBridgeError, NotificationMessage, NotificationSink,
};
use crate::domain::runtime_ui::SystemNotification;

pub type SystemNotificationDispatcher =
    fn(&NotificationMessage) -> Result<(), NotificationBridgeError>;

#[derive(Debug, Default, Clone, Copy)]
pub struct StdoutNotificationSink;

impl NotificationSink for StdoutNotificationSink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        println!("[notification] {}: {}", message.title, message.body);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BestEffortNotificationSink {
    dispatcher: SystemNotificationDispatcher,
    fallback: StdoutNotificationSink,
}

impl Default for BestEffortNotificationSink {
    fn default() -> Self {
        Self::new()
    }
}

impl BestEffortNotificationSink {
    pub fn new() -> Self {
        Self {
            dispatcher: dispatch_system_notification,
            fallback: StdoutNotificationSink,
        }
    }

    pub fn with_dispatcher(dispatcher: SystemNotificationDispatcher) -> Self {
        Self {
            dispatcher,
            fallback: StdoutNotificationSink,
        }
    }
}

impl NotificationSink for BestEffortNotificationSink {
    fn send(
        &self,
        message: &NotificationMessage,
        source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        match (self.dispatcher)(message) {
            Ok(()) => Ok(()),
            Err(_) => self.fallback.send(message, source),
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub fn dispatch_system_notification(
    message: &NotificationMessage,
) -> Result<(), NotificationBridgeError> {
    notify_rust::Notification::new()
        .summary(&message.title)
        .body(&message.body)
        .show()
        .map(|_| ())
        .map_err(|error| NotificationBridgeError::SinkFailed(error.to_string()))
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn dispatch_system_notification(
    _message: &NotificationMessage,
) -> Result<(), NotificationBridgeError> {
    Err(NotificationBridgeError::SinkFailed(
        "system notifications are unsupported on this OS".to_string(),
    ))
}

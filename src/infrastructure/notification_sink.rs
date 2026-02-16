use crate::application::notification_bridge::{
    NotificationBridgeError, NotificationMessage, NotificationSink,
};
use crate::domain::runtime_ui::SystemNotification;

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

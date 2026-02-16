use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::application::notification_bridge::{
    NotificationBridgeError, NotificationMessage, NotificationSink,
};
use crate::domain::runtime_ui::SystemNotification;

#[derive(Debug, Clone)]
pub struct TauriNotificationSink<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriNotificationSink<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> NotificationSink for TauriNotificationSink<R> {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        self.app
            .notification()
            .builder()
            .title(&message.title)
            .body(&message.body)
            .show()
            .map_err(|error| NotificationBridgeError::SinkFailed(error.to_string()))
    }
}

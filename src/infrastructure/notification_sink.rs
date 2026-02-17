use crate::application::notification_bridge::{
    NotificationBridgeError, NotificationMessage, NotificationSink,
};
use crate::domain::runtime_orchestration::ClientRuntimeTarget;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSinkProfile {
    HeadlessCli,
    DesktopSystem,
}

#[derive(Debug, Clone, Copy)]
pub enum RuntimeNotificationSink {
    Stdout(StdoutNotificationSink),
    System(SystemNotificationSink),
}

impl NotificationSink for RuntimeNotificationSink {
    fn send(
        &self,
        message: &NotificationMessage,
        source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        match self {
            Self::Stdout(sink) => sink.send(message, source),
            Self::System(sink) => sink.send(message, source),
        }
    }
}

pub fn select_notification_sink(profile: NotificationSinkProfile) -> RuntimeNotificationSink {
    match profile {
        NotificationSinkProfile::HeadlessCli => {
            RuntimeNotificationSink::Stdout(StdoutNotificationSink)
        }
        NotificationSinkProfile::DesktopSystem => {
            RuntimeNotificationSink::System(SystemNotificationSink::new())
        }
    }
}

pub fn notification_sink_profile_for_target(
    target: ClientRuntimeTarget,
) -> NotificationSinkProfile {
    match target {
        ClientRuntimeTarget::Agent | ClientRuntimeTarget::Mcp => {
            NotificationSinkProfile::HeadlessCli
        }
        ClientRuntimeTarget::UiWeb | ClientRuntimeTarget::UiMobile => {
            NotificationSinkProfile::DesktopSystem
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemNotificationSink {
    dispatcher: SystemNotificationDispatcher,
}

impl Default for SystemNotificationSink {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemNotificationSink {
    pub fn new() -> Self {
        Self {
            dispatcher: dispatch_system_notification,
        }
    }

    pub fn with_dispatcher(dispatcher: SystemNotificationDispatcher) -> Self {
        Self { dispatcher }
    }
}

impl NotificationSink for SystemNotificationSink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        (self.dispatcher)(message)
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

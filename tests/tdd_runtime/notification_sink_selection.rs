use retaia_agent::{
    ClientRuntimeTarget, NotificationSinkProfile, notification_sink_profile_for_target,
    select_notification_sink,
};

#[test]
fn tdd_notification_sink_profile_for_agent_and_mcp_targets_is_headless() {
    assert_eq!(
        notification_sink_profile_for_target(ClientRuntimeTarget::Agent),
        NotificationSinkProfile::HeadlessCli
    );
    assert_eq!(
        notification_sink_profile_for_target(ClientRuntimeTarget::Mcp),
        NotificationSinkProfile::HeadlessCli
    );
}

#[test]
fn tdd_notification_sink_profile_for_ui_targets_is_desktop_system() {
    assert_eq!(
        notification_sink_profile_for_target(ClientRuntimeTarget::UiWeb),
        NotificationSinkProfile::DesktopSystem
    );
    assert_eq!(
        notification_sink_profile_for_target(ClientRuntimeTarget::UiMobile),
        NotificationSinkProfile::DesktopSystem
    );
}

#[test]
fn tdd_select_notification_sink_returns_expected_runtime_variant() {
    let headless = select_notification_sink(NotificationSinkProfile::HeadlessCli);
    assert!(format!("{:?}", headless).contains("Stdout"));

    let desktop = select_notification_sink(NotificationSinkProfile::DesktopSystem);
    assert!(format!("{:?}", desktop).contains("System"));
}

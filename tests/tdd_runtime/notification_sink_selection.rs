use retaia_agent::{
    ClientRuntimeTarget, NotificationSinkProfile, notification_sink_profile_for_target,
    select_notification_sink,
};

#[test]
fn tdd_notification_sink_profile_for_agent_target_is_headless() {
    assert_eq!(
        notification_sink_profile_for_target(ClientRuntimeTarget::Agent),
        NotificationSinkProfile::HeadlessCli
    );
}

#[test]
fn tdd_select_notification_sink_returns_expected_runtime_variant() {
    let headless = select_notification_sink(NotificationSinkProfile::HeadlessCli);
    assert!(format!("{:?}", headless).contains("Stdout"));

    let desktop = select_notification_sink(NotificationSinkProfile::DesktopSystem);
    assert!(format!("{:?}", desktop).contains("System"));
}

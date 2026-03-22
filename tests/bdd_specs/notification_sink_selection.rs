use retaia_agent::{
    ClientRuntimeTarget, NotificationSinkProfile, notification_sink_profile_for_target,
    select_notification_sink,
};

#[test]
fn bdd_given_headless_runtime_targets_when_selecting_notification_profile_then_stdout_sink_policy_is_selected()
 {
    let profile = notification_sink_profile_for_target(ClientRuntimeTarget::Agent);
    assert_eq!(profile, NotificationSinkProfile::HeadlessCli);
    let sink = select_notification_sink(profile);
    assert!(format!("{:?}", sink).contains("Stdout"));
}

#[test]
fn bdd_given_desktop_shell_when_selecting_notification_profile_then_system_sink_policy_is_selected()
{
    let sink = select_notification_sink(NotificationSinkProfile::DesktopSystem);
    assert!(format!("{:?}", sink).contains("System"));
}

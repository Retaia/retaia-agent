use retaia_agent::{
    ClientRuntimeTarget, NotificationSinkProfile, notification_sink_profile_for_target,
    select_notification_sink,
};

#[test]
fn bdd_given_headless_runtime_targets_when_selecting_notification_profile_then_stdout_sink_policy_is_selected()
 {
    let targets = [ClientRuntimeTarget::Agent, ClientRuntimeTarget::Mcp];

    for target in targets {
        let profile = notification_sink_profile_for_target(target);
        assert_eq!(profile, NotificationSinkProfile::HeadlessCli);
        let sink = select_notification_sink(profile);
        assert!(format!("{:?}", sink).contains("Stdout"));
    }
}

#[test]
fn bdd_given_desktop_runtime_targets_when_selecting_notification_profile_then_system_sink_policy_is_selected()
 {
    let targets = [ClientRuntimeTarget::UiWeb, ClientRuntimeTarget::UiMobile];

    for target in targets {
        let profile = notification_sink_profile_for_target(target);
        assert_eq!(profile, NotificationSinkProfile::DesktopSystem);
        let sink = select_notification_sink(profile);
        assert!(format!("{:?}", sink).contains("System"));
    }
}

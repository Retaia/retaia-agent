use retaia_agent::{AgentUiRuntime, ConnectivityState, RuntimeSnapshot, SystemNotification};

#[test]
fn tdd_all_jobs_done_notified_once_on_transition_to_zero_running_jobs() {
    let mut runtime = AgentUiRuntime::new();

    let mut first = RuntimeSnapshot::default();
    first.running_job_ids.insert("job-1".to_string());
    runtime.update_snapshot(first);

    let second = RuntimeSnapshot::default();
    let notifications = runtime.update_snapshot(second.clone());
    assert_eq!(notifications, vec![SystemNotification::AllJobsDone]);

    let notifications_again = runtime.update_snapshot(second);
    assert!(notifications_again.is_empty());
}

#[test]
fn tdd_auth_expired_notified_only_on_state_change() {
    let mut runtime = AgentUiRuntime::new();
    let mut snapshot = RuntimeSnapshot::default();
    snapshot.auth_reauth_required = true;
    let notifications = runtime.update_snapshot(snapshot.clone());
    assert_eq!(
        notifications,
        vec![SystemNotification::AuthExpiredReauthRequired]
    );

    let notifications_again = runtime.update_snapshot(snapshot);
    assert!(notifications_again.is_empty());
}

#[test]
fn tdd_disconnect_notification_is_transition_based() {
    let mut runtime = AgentUiRuntime::new();

    let mut disconnected = RuntimeSnapshot::default();
    disconnected.connectivity = ConnectivityState::Disconnected;
    let first = runtime.update_snapshot(disconnected.clone());
    assert_eq!(
        first,
        vec![SystemNotification::AgentDisconnectedOrReconnecting]
    );

    let second = runtime.update_snapshot(disconnected);
    assert!(second.is_empty());
}

#[test]
fn tdd_settings_invalid_is_deduplicated_and_reset_on_success() {
    let mut runtime = AgentUiRuntime::new();

    let first = runtime.notify_settings_invalid("ollama unreachable");
    assert_eq!(
        first,
        Some(SystemNotification::SettingsInvalid {
            reason: "ollama unreachable".to_string(),
        })
    );

    let second = runtime.notify_settings_invalid("ollama unreachable");
    assert_eq!(second, None);

    let saved = runtime.notify_settings_saved();
    assert_eq!(saved, SystemNotification::SettingsSaved);

    let third = runtime.notify_settings_invalid("ollama unreachable");
    assert_eq!(
        third,
        Some(SystemNotification::SettingsInvalid {
            reason: "ollama unreachable".to_string(),
        })
    );
}

#[test]
fn tdd_updates_available_notified_once_per_version() {
    let mut runtime = AgentUiRuntime::new();

    let first = RuntimeSnapshot {
        available_update: Some("1.2.3".to_string()),
        ..RuntimeSnapshot::default()
    };
    let notifs_v123 = runtime.update_snapshot(first);
    assert_eq!(
        notifs_v123,
        vec![SystemNotification::UpdatesAvailable {
            version: "1.2.3".to_string()
        }]
    );

    let repeated = RuntimeSnapshot {
        available_update: Some("1.2.3".to_string()),
        ..RuntimeSnapshot::default()
    };
    assert!(runtime.update_snapshot(repeated).is_empty());

    let next = RuntimeSnapshot {
        available_update: Some("1.2.4".to_string()),
        ..RuntimeSnapshot::default()
    };
    assert_eq!(
        runtime.update_snapshot(next),
        vec![SystemNotification::UpdatesAvailable {
            version: "1.2.4".to_string()
        }]
    );
}

#[test]
fn tdd_status_window_job_returns_none_without_current_job() {
    let snapshot = RuntimeSnapshot::default();
    assert!(AgentUiRuntime::status_window_job(&snapshot).is_none());
}

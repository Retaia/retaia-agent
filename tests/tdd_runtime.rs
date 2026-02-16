use std::collections::BTreeMap;

use retaia_agent::{
    AgentRunState, AgentUiRuntime, ClientKind, ConnectivityState, MenuAction, RuntimeSnapshot,
    SystemNotification, base_menu_actions, can_issue_client_token, can_process_jobs,
    menu_visibility, resolve_effective_features,
};

#[test]
fn tdd_resolve_features_respects_user_and_app_values() {
    let app = BTreeMap::from([(String::from("search"), true)]);
    let user = BTreeMap::from([(String::from("search"), false)]);
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);

    assert_eq!(effective.get("search"), Some(&false));
}

#[test]
fn tdd_missing_user_key_defaults_to_true() {
    let app = BTreeMap::from([(String::from("ai"), true)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);

    assert_eq!(effective.get("ai"), Some(&true));
}

#[test]
fn tdd_dependency_off_forces_dependent_off() {
    let app = BTreeMap::from([
        (String::from("ai"), true),
        (String::from("suggestions"), true),
    ]);
    let user = BTreeMap::from([(String::from("ai"), false)]);
    let deps = BTreeMap::from([(String::from("suggestions"), vec![String::from("ai")])]);
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);

    assert_eq!(effective.get("ai"), Some(&false));
    assert_eq!(effective.get("suggestions"), Some(&false));
}

#[test]
fn tdd_disable_escalation_turns_children_off() {
    let app = BTreeMap::from([
        (String::from("parent"), false),
        (String::from("child"), true),
    ]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::from([(String::from("parent"), vec![String::from("child")])]);

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);

    assert_eq!(effective.get("child"), Some(&false));
}

#[test]
fn tdd_client_token_policy_matches_actor_rules() {
    assert!(can_issue_client_token(ClientKind::Agent, false));
    assert!(!can_issue_client_token(ClientKind::UiRust, true));
    assert!(!can_issue_client_token(ClientKind::Mcp, false));
    assert!(can_issue_client_token(ClientKind::Mcp, true));
}

#[test]
fn tdd_processing_is_agent_only() {
    assert!(can_process_jobs(ClientKind::Agent));
    assert!(!can_process_jobs(ClientKind::Mcp));
    assert!(!can_process_jobs(ClientKind::UiRust));
}

#[test]
fn tdd_menu_toggle_visibility_respects_running_and_paused_state() {
    let running = menu_visibility(AgentRunState::Running);
    assert!(!running.show_play_resume);
    assert!(running.show_pause);

    let paused = menu_visibility(AgentRunState::Paused);
    assert!(paused.show_play_resume);
    assert!(!paused.show_pause);
}

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
fn tdd_base_menu_actions_are_stable() {
    let actions = base_menu_actions();
    assert_eq!(
        actions,
        vec![
            MenuAction::OpenStatusWindow,
            MenuAction::OpenSettings,
            MenuAction::Stop,
            MenuAction::Quit,
        ]
    );
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

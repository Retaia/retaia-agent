use std::collections::BTreeMap;

use retaia_agent::{
    AgentUiRuntime, ClientKind, JobFailure, RuntimeSnapshot, SystemNotification,
    can_issue_client_token, resolve_effective_features,
};

#[test]
fn bdd_given_mcp_when_ai_disabled_globally_then_token_is_forbidden() {
    let ai_enabled = false;
    let allowed = can_issue_client_token(ClientKind::Mcp, ai_enabled);
    assert!(!allowed);
}

#[test]
fn bdd_given_ui_rust_when_client_token_requested_then_forbidden_actor() {
    let allowed = can_issue_client_token(ClientKind::UiRust, true);
    assert!(!allowed);
}

#[test]
fn bdd_given_missing_user_feature_key_when_resolving_then_treated_as_true() {
    let app = BTreeMap::from([(String::from("ai"), true)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);

    assert_eq!(effective.get("ai"), Some(&true));
}

#[test]
fn bdd_given_parent_disabled_when_disable_escalation_exists_then_child_is_disabled() {
    let app = BTreeMap::from([
        (String::from("features.ai"), false),
        (String::from("features.suggestions"), true),
    ]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::from([(
        String::from("features.ai"),
        vec![String::from("features.suggestions")],
    )]);

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);

    assert_eq!(effective.get("features.suggestions"), Some(&false));
}

#[test]
fn bdd_given_new_job_when_polling_then_notify_new_job_once() {
    let mut runtime = AgentUiRuntime::new();
    let mut snapshot = RuntimeSnapshot::default();
    snapshot.known_job_ids.insert("job-42".to_string());

    let first = runtime.update_snapshot(snapshot.clone());
    assert_eq!(
        first,
        vec![SystemNotification::NewJobReceived {
            job_id: "job-42".to_string()
        }]
    );

    let second = runtime.update_snapshot(snapshot);
    assert!(second.is_empty());
}

#[test]
fn bdd_given_same_failed_job_on_poll_when_already_notified_then_no_repeat() {
    let mut runtime = AgentUiRuntime::new();
    let mut snapshot = RuntimeSnapshot::default();
    snapshot.failed_jobs.push(JobFailure {
        job_id: "job-failed".to_string(),
        error_code: "E_CODEC".to_string(),
    });

    let first = runtime.update_snapshot(snapshot.clone());
    assert_eq!(
        first,
        vec![SystemNotification::JobFailed {
            job_id: "job-failed".to_string(),
            error_code: "E_CODEC".to_string()
        }]
    );

    let second = runtime.update_snapshot(snapshot);
    assert!(second.is_empty());
}

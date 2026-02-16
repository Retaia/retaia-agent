use std::collections::BTreeMap;

use retaia_agent::{
    ClientKind,
    can_issue_client_token,
    can_process_jobs,
    resolve_effective_features,
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

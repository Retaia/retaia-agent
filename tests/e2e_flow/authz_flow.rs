use std::collections::BTreeMap;

use retaia_agent::{
    CORE_CLIENTS_BOOTSTRAP_FEATURE, CORE_JOBS_RUNTIME_FEATURE, ClientKind,
    can_issue_client_token, can_process_jobs, resolve_effective_features,
};

#[test]
fn e2e_agent_service_mode_keeps_processing_authorized() {
    let flags = BTreeMap::from([
        (String::from("features.ai"), true),
        (CORE_JOBS_RUNTIME_FEATURE.to_string(), true),
    ]);
    let app = BTreeMap::from([(String::from("features.ai"), true)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&flags, &app, &user, &deps, &escalation);
    let ai_enabled = *effective.get("features.ai").unwrap_or(&false);

    assert!(can_issue_client_token(ClientKind::Agent, ai_enabled));
    assert!(can_process_jobs(ClientKind::Agent));
}

#[test]
fn e2e_interactive_clients_cannot_issue_technical_client_tokens_or_process_jobs() {
    assert!(!can_issue_client_token(ClientKind::UiWeb, true));
    assert!(!can_issue_client_token(ClientKind::UiMobile, true));
    assert!(!can_process_jobs(ClientKind::UiWeb));
    assert!(!can_process_jobs(ClientKind::UiMobile));
}

#[test]
fn e2e_core_v1_global_features_stay_enabled_even_if_runtime_payload_sets_them_false() {
    let flags = BTreeMap::from([
        (CORE_JOBS_RUNTIME_FEATURE.to_string(), false),
        (CORE_CLIENTS_BOOTSTRAP_FEATURE.to_string(), false),
    ]);

    let effective =
        resolve_effective_features(&flags, &BTreeMap::new(), &BTreeMap::new(), &BTreeMap::new(), &BTreeMap::new());

    assert_eq!(effective.get(CORE_JOBS_RUNTIME_FEATURE), Some(&true));
    assert_eq!(effective.get(CORE_CLIENTS_BOOTSTRAP_FEATURE), Some(&true));
}

#[test]
fn e2e_absent_runtime_flag_stays_false_for_non_global_feature_even_if_app_and_user_are_true() {
    let feature = "features.ai".to_string();
    let flags = BTreeMap::new();
    let app = BTreeMap::from([(feature.clone(), true)]);
    let user = BTreeMap::from([(feature.clone(), true)]);

    let effective = resolve_effective_features(
        &flags,
        &app,
        &user,
        &BTreeMap::new(),
        &BTreeMap::new(),
    );

    assert_eq!(effective.get(&feature), Some(&false));
}

use std::collections::BTreeMap;

use retaia_agent::{
    ClientKind, can_issue_client_token, can_process_jobs, resolve_effective_features,
};

#[test]
fn e2e_agent_service_mode_keeps_processing_authorized() {
    let app = BTreeMap::from([(String::from("features.ai"), true)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);
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

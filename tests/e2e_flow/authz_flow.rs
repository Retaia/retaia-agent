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
fn e2e_mcp_can_orchestrate_but_never_process_jobs() {
    let app = BTreeMap::from([(String::from("features.ai"), true)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);
    let ai_enabled = *effective.get("features.ai").unwrap_or(&false);

    assert!(can_issue_client_token(ClientKind::Mcp, ai_enabled));
    assert!(!can_process_jobs(ClientKind::Mcp));
}

#[test]
fn e2e_mcp_global_ai_off_blocks_client_token_flow() {
    let app = BTreeMap::from([(String::from("features.ai"), false)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);
    let ai_enabled = *effective.get("features.ai").unwrap_or(&true);

    assert!(!can_issue_client_token(ClientKind::Mcp, ai_enabled));
    assert!(!can_process_jobs(ClientKind::Mcp));
}

#[test]
fn e2e_ui_web_and_ui_mobile_cannot_issue_technical_client_tokens_or_process_jobs() {
    assert!(!can_issue_client_token(ClientKind::UiWeb, true));
    assert!(!can_issue_client_token(ClientKind::UiMobile, true));
    assert!(!can_process_jobs(ClientKind::UiWeb));
    assert!(!can_process_jobs(ClientKind::UiMobile));
}

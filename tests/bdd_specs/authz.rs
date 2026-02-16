use retaia_agent::{ClientKind, can_issue_client_token};

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

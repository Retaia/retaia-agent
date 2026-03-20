use retaia_agent::{ClientKind, can_issue_client_token};

#[test]
fn bdd_given_ui_web_when_client_token_requested_then_forbidden_actor() {
    let allowed = can_issue_client_token(ClientKind::UiWeb, true);
    assert!(!allowed);
}

#[test]
fn bdd_given_agent_ui_when_client_token_requested_then_forbidden_actor() {
    let allowed = can_issue_client_token(ClientKind::UiMobile, true);
    assert!(!allowed);
}

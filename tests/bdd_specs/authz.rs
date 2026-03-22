use retaia_agent::{ClientKind, can_issue_client_token, can_process_jobs};

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

#[test]
fn bdd_given_agent_when_client_token_requested_then_allowed_even_without_ai() {
    assert!(can_issue_client_token(ClientKind::Agent, false));
    assert!(can_issue_client_token(ClientKind::Agent, true));
}

#[test]
fn bdd_given_only_agent_when_jobs_processing_is_checked_then_only_agent_is_allowed() {
    assert!(can_process_jobs(ClientKind::Agent));
    assert!(!can_process_jobs(ClientKind::UiWeb));
    assert!(!can_process_jobs(ClientKind::UiMobile));
}

use retaia_agent::{
    CORE_AUTH_FEATURE, CORE_CLIENTS_BOOTSTRAP_FEATURE, CORE_JOBS_RUNTIME_FEATURE, ClientKind,
    can_issue_client_token, can_process_jobs, core_v1_global_features,
};

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

#[test]
fn bdd_given_core_v1_globals_when_listed_then_runtime_auth_and_bootstrap_flags_are_present() {
    let globals = core_v1_global_features();
    assert!(globals.contains(CORE_AUTH_FEATURE));
    assert!(globals.contains(CORE_JOBS_RUNTIME_FEATURE));
    assert!(globals.contains(CORE_CLIENTS_BOOTSTRAP_FEATURE));
}

use retaia_agent::{
    PollDecisionReason, PollEndpoint, PollSignal, can_issue_mutation_after_poll, next_poll_decision,
};

#[test]
fn bdd_given_contract_interval_when_polling_jobs_then_client_waits_exact_interval() {
    let decision = next_poll_decision(
        PollEndpoint::Jobs,
        PollSignal::ContractInterval { interval_ms: 1_500 },
        0,
        3,
    );
    assert_eq!(decision.wait_ms, 1_500);
    assert_eq!(decision.reason, PollDecisionReason::ContractInterval);
}

#[test]
fn bdd_given_429_too_many_attempts_when_polling_policy_then_backoff_jitter_is_applied() {
    let decision = next_poll_decision(PollEndpoint::Policy, PollSignal::TooManyAttempts429, 3, 11);
    assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
    assert!(decision.wait_ms >= 4_000);
}

#[test]
fn bdd_given_no_compatible_state_from_poll_when_mutation_requested_then_forbidden() {
    assert!(!can_issue_mutation_after_poll(false));
}

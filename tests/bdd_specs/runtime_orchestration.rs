use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    PushHintDecision, can_issue_mutation_after_poll, next_poll_decision,
    should_trigger_poll_from_push,
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
fn bdd_given_mobile_push_on_mobile_ui_when_hint_is_fresh_then_poll_is_triggered() {
    let decision = should_trigger_poll_from_push(
        ClientRuntimeTarget::UiMobile,
        PushChannel::MobileApns,
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 5_000,
        },
        2_000,
        false,
    );
    assert_eq!(decision, PushHintDecision::TriggerPoll);
}

#[test]
fn bdd_given_mobile_push_on_agent_when_received_then_hint_is_ignored() {
    let decision = should_trigger_poll_from_push(
        ClientRuntimeTarget::Agent,
        PushChannel::MobileFcm,
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 5_000,
        },
        2_000,
        false,
    );
    assert_eq!(decision, PushHintDecision::Ignore);
}

#[test]
fn bdd_given_no_compatible_state_from_poll_when_mutation_requested_then_forbidden() {
    assert!(!can_issue_mutation_after_poll(false));
}

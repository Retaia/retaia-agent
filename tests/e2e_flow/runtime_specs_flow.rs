use retaia_agent::{
    PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint, PushHintDecision,
    PushProcessResult, RuntimeSyncState, can_issue_mutation_after_poll, next_poll_decision,
    should_trigger_poll_from_push,
};

#[test]
fn e2e_specs_v1_push_hint_triggers_poll_but_is_not_authoritative() {
    let decision = should_trigger_poll_from_push(
        retaia_agent::ClientRuntimeTarget::Agent,
        PushChannel::WebSocket,
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 5_000,
        },
        2_000,
        false,
    );
    assert_eq!(decision, PushHintDecision::TriggerPoll);

    assert!(!can_issue_mutation_after_poll(false));
    assert!(can_issue_mutation_after_poll(true));
}

#[test]
fn e2e_specs_v1_expired_or_duplicate_hint_is_ignored() {
    let mut sync = RuntimeSyncState::new(retaia_agent::ClientRuntimeTarget::Agent);

    let expired = sync.process_push_hint(
        PushChannel::Sse,
        "hint-expired",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 500,
        },
        2_000,
    );
    assert_eq!(expired, PushProcessResult::Ignored);

    let fresh = sync.process_push_hint(
        PushChannel::Sse,
        "hint-dup",
        PushHint {
            issued_at_ms: 2_000,
            ttl_ms: 5_000,
        },
        2_100,
    );
    assert_eq!(fresh, PushProcessResult::PollTriggered);

    let duplicate = sync.process_push_hint(
        PushChannel::Sse,
        "hint-dup",
        PushHint {
            issued_at_ms: 2_000,
            ttl_ms: 5_000,
        },
        2_200,
    );
    assert_eq!(duplicate, PushProcessResult::Ignored);
}

#[test]
fn e2e_specs_v1_too_many_attempts_429_backoff_and_contract_polling() {
    let throttled = next_poll_decision(
        PollEndpoint::DeviceFlow,
        PollSignal::TooManyAttempts429,
        3,
        13,
    );
    assert_eq!(throttled.reason, PollDecisionReason::BackoffFrom429);
    assert!(throttled.wait_ms >= 4_000);

    let periodic = next_poll_decision(
        PollEndpoint::Policy,
        PollSignal::ContractInterval { interval_ms: 2_000 },
        0,
        0,
    );
    assert_eq!(periodic.reason, PollDecisionReason::ContractInterval);
    assert_eq!(periodic.wait_ms, 2_000);
}

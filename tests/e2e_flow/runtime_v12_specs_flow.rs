use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    PushHintDecision, PushProcessResult, RuntimeSyncState, next_poll_decision,
    should_trigger_poll_from_push,
};

#[test]
fn e2e_specs_v12_mobile_push_is_ui_mobile_only_and_agent_ignores_it() {
    let hint = PushHint {
        issued_at_ms: 1_000,
        ttl_ms: 5_000,
    };

    let ui_mobile = should_trigger_poll_from_push(
        ClientRuntimeTarget::UiMobile,
        PushChannel::MobileApns,
        hint,
        2_000,
        false,
    );
    assert_eq!(ui_mobile, PushHintDecision::TriggerPoll);

    let agent = should_trigger_poll_from_push(
        ClientRuntimeTarget::Agent,
        PushChannel::MobileApns,
        hint,
        2_000,
        false,
    );
    assert_eq!(agent, PushHintDecision::Ignore);
}

#[test]
fn e2e_specs_v12_expired_or_duplicate_push_hint_does_not_trigger_poll() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::UiRustDesktop);

    let expired = sync.process_push_hint(
        PushChannel::WebSocket,
        "hint-expired",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 500,
        },
        2_000,
    );
    assert_eq!(expired, PushProcessResult::Ignored);
    assert_eq!(sync.seen_hint_count(), 0);

    let fresh = sync.process_push_hint(
        PushChannel::WebSocket,
        "hint-dup",
        PushHint {
            issued_at_ms: 2_000,
            ttl_ms: 5_000,
        },
        2_500,
    );
    assert_eq!(fresh, PushProcessResult::PollTriggered);
    assert_eq!(sync.seen_hint_count(), 1);

    let duplicate = sync.process_push_hint(
        PushChannel::WebSocket,
        "hint-dup",
        PushHint {
            issued_at_ms: 2_000,
            ttl_ms: 5_000,
        },
        2_700,
    );
    assert_eq!(duplicate, PushProcessResult::Ignored);
    assert_eq!(sync.seen_hint_count(), 1);
}

#[test]
fn e2e_specs_v12_too_many_attempts_429_uses_backoff_and_regular_polling_remains_available() {
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

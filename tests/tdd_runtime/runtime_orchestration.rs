use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    PushHintDecision, RuntimeOrchestrationMode, can_issue_mutation_after_poll, next_poll_decision,
    push_channels_allowed, push_is_authoritative, runtime_orchestration_mode,
    should_trigger_poll_from_push, throttled_backoff_with_jitter,
};

#[test]
fn tdd_runtime_orchestration_is_status_driven_polling_and_push_is_hint_only() {
    assert_eq!(
        runtime_orchestration_mode(),
        RuntimeOrchestrationMode::StatusDrivenPolling
    );
    assert!(push_channels_allowed());
    assert!(!push_is_authoritative());
}

#[test]
fn tdd_contract_interval_is_used_for_polling_decision() {
    let decision = next_poll_decision(
        PollEndpoint::Jobs,
        PollSignal::ContractInterval { interval_ms: 2_000 },
        0,
        7,
    );
    assert_eq!(decision.wait_ms, 2_000);
    assert_eq!(decision.reason, PollDecisionReason::ContractInterval);
}

#[test]
fn tdd_429_signal_uses_backoff_with_jitter() {
    let throttled = next_poll_decision(PollEndpoint::Policy, PollSignal::SlowDown429, 2, 19);
    assert_eq!(throttled.reason, PollDecisionReason::BackoffFrom429);
    assert!(throttled.wait_ms >= 2_000);
}

#[test]
fn tdd_backoff_with_jitter_is_bounded_and_monotonic_by_attempt() {
    let first = throttled_backoff_with_jitter(0, 42);
    let second = throttled_backoff_with_jitter(1, 42);
    let third = throttled_backoff_with_jitter(2, 42);
    assert!(first <= second);
    assert!(second <= third);
    assert!(third <= 60_000);
}

#[test]
fn tdd_mutations_require_compatible_state_read_from_polling() {
    assert!(can_issue_mutation_after_poll(true));
    assert!(!can_issue_mutation_after_poll(false));
}

#[test]
fn tdd_mobile_push_only_targets_mobile_ui() {
    let hint = PushHint {
        issued_at_ms: 1_000,
        ttl_ms: 2_000,
    };
    let now_ms = 2_000;

    let mobile = should_trigger_poll_from_push(
        ClientRuntimeTarget::UiMobile,
        PushChannel::MobileFcm,
        hint,
        now_ms,
        false,
    );
    assert_eq!(mobile, PushHintDecision::TriggerPoll);

    let agent = should_trigger_poll_from_push(
        ClientRuntimeTarget::Agent,
        PushChannel::MobileFcm,
        hint,
        now_ms,
        false,
    );
    assert_eq!(agent, PushHintDecision::Ignore);
}

#[test]
fn tdd_push_hint_dedup_or_expired_is_ignored() {
    let expired = should_trigger_poll_from_push(
        ClientRuntimeTarget::UiRustDesktop,
        PushChannel::WebSocket,
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 300,
        },
        2_000,
        false,
    );
    assert_eq!(expired, PushHintDecision::Ignore);

    let deduped = should_trigger_poll_from_push(
        ClientRuntimeTarget::UiRustDesktop,
        PushChannel::Sse,
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 5_000,
        },
        2_000,
        true,
    );
    assert_eq!(deduped, PushHintDecision::Ignore);
}

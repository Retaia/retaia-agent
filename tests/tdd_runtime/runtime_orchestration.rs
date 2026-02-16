use retaia_agent::{
    PollDecisionReason, PollEndpoint, PollSignal, RuntimeOrchestrationMode,
    can_issue_mutation_after_poll, next_poll_decision, push_channels_allowed,
    runtime_orchestration_mode, throttled_backoff_with_jitter,
};

#[test]
fn tdd_runtime_orchestration_mode_is_pull_only_and_disables_push_channels() {
    assert_eq!(
        runtime_orchestration_mode(),
        RuntimeOrchestrationMode::PullOnly
    );
    assert!(!push_channels_allowed());
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

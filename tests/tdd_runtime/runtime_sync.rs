use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    PushProcessResult, RuntimeSyncState,
};

#[test]
fn tdd_runtime_sync_triggers_poll_once_for_fresh_unique_hint() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::UiMobile);
    let hint = PushHint {
        issued_at_ms: 1_000,
        ttl_ms: 5_000,
    };

    let first = sync.process_push_hint(PushChannel::MobileFcm, "hint-1", hint, 2_000);
    assert_eq!(first, PushProcessResult::PollTriggered);
    assert_eq!(sync.seen_hint_count(), 1);

    let duplicate = sync.process_push_hint(PushChannel::MobileFcm, "hint-1", hint, 2_500);
    assert_eq!(duplicate, PushProcessResult::Ignored);
    assert_eq!(sync.seen_hint_count(), 1);
}

#[test]
fn tdd_runtime_sync_ignores_mobile_push_for_non_mobile_target() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::Agent);
    let hint = PushHint {
        issued_at_ms: 1_000,
        ttl_ms: 5_000,
    };

    let result = sync.process_push_hint(PushChannel::MobileApns, "hint-a", hint, 2_000);
    assert_eq!(result, PushProcessResult::Ignored);
    assert_eq!(sync.seen_hint_count(), 0);
}

#[test]
fn tdd_runtime_sync_exposes_contract_and_429_poll_decisions() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::UiWeb);

    let contract = sync.poll_by_contract(PollEndpoint::Policy, 1_500);
    assert_eq!(contract.reason, PollDecisionReason::ContractInterval);
    assert_eq!(contract.wait_ms, 1_500);

    let throttled =
        sync.poll_after_429_tracked(PollEndpoint::Policy, PollSignal::TooManyAttempts429, 17);
    assert_eq!(throttled.reason, PollDecisionReason::BackoffFrom429);
    assert!(throttled.wait_ms >= 2_000);
}

#[test]
fn tdd_runtime_sync_tracked_429_attempts_increase_and_reset_after_success() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::Agent);

    let first = sync.poll_after_429_tracked(PollEndpoint::Jobs, PollSignal::SlowDown429, 7);
    let second = sync.poll_after_429_tracked(PollEndpoint::Jobs, PollSignal::SlowDown429, 7);
    let reset = sync.poll_by_contract_and_reset(PollEndpoint::Jobs, 5_000);
    let after_reset = sync.poll_after_429_tracked(PollEndpoint::Jobs, PollSignal::SlowDown429, 7);

    assert!(first.wait_ms >= 2_000);
    assert!(second.wait_ms >= first.wait_ms);
    assert_eq!(reset.wait_ms, 5_000);
    assert!(after_reset.wait_ms >= 2_000);
    assert!(after_reset.wait_ms <= second.wait_ms);
}

#[test]
fn tdd_runtime_sync_opens_mutation_gate_only_after_compatible_poll_state() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::Agent);
    assert!(!sync.can_issue_mutation());

    sync.observe_polled_state(false);
    assert!(!sync.can_issue_mutation());

    sync.observe_polled_state(true);
    assert!(sync.can_issue_mutation());
}

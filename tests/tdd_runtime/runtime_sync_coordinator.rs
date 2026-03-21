use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    RuntimeSyncCoordinator, RuntimeSyncPlan,
};

#[test]
fn tdd_runtime_sync_coordinator_triggers_immediate_poll_for_fresh_hint() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    let plan = coordinator.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::WebSocket,
        "hint-1",
        PushHint {
            issued_at_ms: 100,
            ttl_ms: 5_000,
        },
        200,
    );

    assert_eq!(
        plan,
        RuntimeSyncPlan::TriggerPollNow {
            endpoint: PollEndpoint::Jobs
        }
    );
    assert_eq!(coordinator.seen_hint_count(), 1);
}

#[test]
fn tdd_runtime_sync_coordinator_ignores_duplicate_hint() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    let hint = PushHint {
        issued_at_ms: 100,
        ttl_ms: 5_000,
    };
    let _ = coordinator.on_push_hint(PollEndpoint::Jobs, PushChannel::Sse, "dup", hint, 200);

    let second = coordinator.on_push_hint(PollEndpoint::Jobs, PushChannel::Sse, "dup", hint, 300);
    assert_eq!(second, RuntimeSyncPlan::None);
}

#[test]
fn tdd_runtime_sync_coordinator_schedules_contract_poll_and_updates_mutation_gate() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    assert!(!coordinator.can_issue_mutation());

    let plan = coordinator.on_poll_success(PollEndpoint::Policy, 2_000, true);
    match plan {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::ContractInterval);
            assert_eq!(decision.wait_ms, 2_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }
    assert!(coordinator.can_issue_mutation());
}

#[test]
fn tdd_runtime_sync_coordinator_schedules_backoff_on_throttling() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    let plan =
        coordinator.on_poll_throttled_tracked(PollEndpoint::DeviceFlow, PollSignal::SlowDown429, 9);
    match plan {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
            assert!(decision.wait_ms >= 2_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }
}

#[test]
fn tdd_runtime_sync_coordinator_tracked_429_attempts_reset_after_success() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);

    let first =
        coordinator.on_poll_throttled_tracked(PollEndpoint::Jobs, PollSignal::SlowDown429, 9);
    let second =
        coordinator.on_poll_throttled_tracked(PollEndpoint::Jobs, PollSignal::SlowDown429, 9);
    let _ = coordinator.on_poll_success(PollEndpoint::Jobs, 5_000, true);
    let third =
        coordinator.on_poll_throttled_tracked(PollEndpoint::Jobs, PollSignal::SlowDown429, 9);

    let extract_wait = |plan: RuntimeSyncPlan| match plan {
        RuntimeSyncPlan::SchedulePoll(decision) => decision.wait_ms,
        other => panic!("unexpected plan: {other:?}"),
    };

    let first_wait = extract_wait(first);
    let second_wait = extract_wait(second);
    let third_wait = extract_wait(third);

    assert!(second_wait >= first_wait);
    assert!(third_wait <= second_wait);
}

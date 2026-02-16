use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    RuntimeSyncCoordinator, RuntimeSyncPlan,
};

#[test]
fn bdd_given_push_hint_when_received_then_sync_coordinator_requests_immediate_poll() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    let plan = coordinator.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::Webhook,
        "hint-77",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 3_000,
        },
        1_500,
    );
    assert_eq!(
        plan,
        RuntimeSyncPlan::TriggerPollNow {
            endpoint: PollEndpoint::Jobs
        }
    );
}

#[test]
fn bdd_given_429_when_polling_then_sync_coordinator_schedules_backoff_retry() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    let plan =
        coordinator.on_poll_throttled(PollEndpoint::Policy, PollSignal::TooManyAttempts429, 3, 17);

    match plan {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
            assert!(decision.wait_ms >= 4_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }
}

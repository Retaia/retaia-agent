use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    RuntimeSyncCoordinator, RuntimeSyncPlan,
};

#[test]
fn e2e_sync_coordinator_flow_push_poll_throttle_and_mutation_gate() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    assert!(!coordinator.can_issue_mutation());

    let push_plan = coordinator.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::WebSocket,
        "hint-100",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 5_000,
        },
        1_200,
    );
    assert_eq!(
        push_plan,
        RuntimeSyncPlan::TriggerPollNow {
            endpoint: PollEndpoint::Jobs
        }
    );

    let throttled_plan =
        coordinator.on_poll_throttled(PollEndpoint::Jobs, PollSignal::TooManyAttempts429, 3, 13);
    match throttled_plan {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
            assert!(decision.wait_ms >= 4_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }

    let steady_plan = coordinator.on_poll_success(PollEndpoint::Jobs, 2_000, true);
    match steady_plan {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::ContractInterval);
            assert_eq!(decision.wait_ms, 2_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }
    assert!(coordinator.can_issue_mutation());
}

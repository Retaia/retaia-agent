use retaia_agent::{
    AgentRunState, ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel,
    PushHint, RuntimeControlCommand, RuntimeLoopEngine, RuntimeSyncPlan,
};

#[test]
fn e2e_runtime_loop_engine_applies_run_state_and_sync_plans_consistently() {
    let mut engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    assert_eq!(engine.run_state(), AgentRunState::Running);
    assert!(engine.can_sync());

    let first_push = engine.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::WebSocket,
        "hint-201",
        PushHint {
            issued_at_ms: 10_000,
            ttl_ms: 5_000,
        },
        10_100,
    );
    assert_eq!(
        first_push,
        RuntimeSyncPlan::TriggerPollNow {
            endpoint: PollEndpoint::Jobs
        }
    );

    let throttled = engine.on_poll_throttled(PollEndpoint::Jobs, PollSignal::SlowDown429, 2, 9);
    match throttled {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
            assert!(decision.wait_ms >= 2_000);
        }
        other => panic!("unexpected throttled plan: {other:?}"),
    }

    let steady = engine.on_poll_success(PollEndpoint::Jobs, 2_000, true);
    match steady {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::ContractInterval);
            assert_eq!(decision.wait_ms, 2_000);
        }
        other => panic!("unexpected steady plan: {other:?}"),
    }
    assert!(engine.can_issue_mutation());

    engine.apply_control(RuntimeControlCommand::Stop);
    assert_eq!(engine.run_state(), AgentRunState::Stopped);
    assert!(!engine.can_sync());
    assert!(!engine.can_issue_mutation());

    let stopped_push = engine.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::WebSocket,
        "hint-202",
        PushHint {
            issued_at_ms: 20_000,
            ttl_ms: 5_000,
        },
        20_100,
    );
    assert_eq!(stopped_push, RuntimeSyncPlan::None);
}

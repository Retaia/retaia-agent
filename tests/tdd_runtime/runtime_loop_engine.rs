use retaia_agent::{
    AgentRunState, ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel,
    PushHint, RuntimeControlCommand, RuntimeLoopEngine, RuntimeSyncPlan,
};

#[test]
fn tdd_runtime_loop_engine_starts_running_and_allows_sync() {
    let engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    assert_eq!(engine.run_state(), AgentRunState::Running);
    assert!(engine.can_sync());
}

#[test]
fn tdd_runtime_loop_engine_stopped_state_blocks_sync_plans() {
    let mut engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    engine.apply_control(RuntimeControlCommand::Stop);
    assert_eq!(engine.run_state(), AgentRunState::Stopped);
    assert!(!engine.can_sync());

    let push = engine.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::WebSocket,
        "hint-1",
        PushHint {
            issued_at_ms: 100,
            ttl_ms: 5_000,
        },
        200,
    );
    assert_eq!(push, RuntimeSyncPlan::None);
}

#[test]
fn tdd_runtime_loop_engine_throttling_returns_backoff_schedule() {
    let mut engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    let plan = engine.on_poll_throttled(
        PollEndpoint::DeviceFlow,
        PollSignal::TooManyAttempts429,
        3,
        21,
    );
    match plan {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
            assert!(decision.wait_ms >= 4_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }
}

#[test]
fn tdd_runtime_loop_engine_mutation_gate_needs_running_state_and_compatible_poll() {
    let mut engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    assert!(!engine.can_issue_mutation());

    let _ = engine.on_poll_success(PollEndpoint::Jobs, 1_500, true);
    assert!(engine.can_issue_mutation());

    engine.apply_control(RuntimeControlCommand::Pause);
    assert_eq!(engine.run_state(), AgentRunState::Paused);
    assert!(!engine.can_issue_mutation());
}

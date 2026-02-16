use retaia_agent::{
    AgentRunState, ClientRuntimeTarget, PollEndpoint, PushChannel, PushHint, RuntimeControlCommand,
    RuntimeLoopEngine, RuntimeSyncPlan,
};

#[test]
fn bdd_given_engine_running_when_push_hint_arrives_then_poll_is_triggered() {
    let mut engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    let plan = engine.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::Webhook,
        "hint-10",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 3_000,
        },
        1_200,
    );
    assert_eq!(
        plan,
        RuntimeSyncPlan::TriggerPollNow {
            endpoint: PollEndpoint::Jobs
        }
    );
}

#[test]
fn bdd_given_engine_stopped_when_push_hint_arrives_then_no_poll_plan_is_emitted() {
    let mut engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    engine.apply_control(RuntimeControlCommand::Stop);
    assert_eq!(engine.run_state(), AgentRunState::Stopped);

    let plan = engine.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::Webhook,
        "hint-11",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 3_000,
        },
        1_300,
    );
    assert_eq!(plan, RuntimeSyncPlan::None);
}

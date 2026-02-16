use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    PushProcessResult, RuntimeSyncState,
};

#[test]
fn e2e_runtime_sync_hint_then_poll_then_mutation_gate_flow() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::Agent);
    assert!(!sync.can_issue_mutation());

    let hint_result = sync.process_push_hint(
        PushChannel::WebSocket,
        "job-update-1",
        PushHint {
            issued_at_ms: 100,
            ttl_ms: 5_000,
        },
        200,
    );
    assert_eq!(hint_result, PushProcessResult::PollTriggered);

    let throttled = sync.poll_after_429(PollEndpoint::Jobs, PollSignal::SlowDown429, 2, 9);
    assert_eq!(throttled.reason, PollDecisionReason::BackoffFrom429);
    assert!(throttled.wait_ms >= 2_000);

    sync.observe_polled_state(true);
    assert!(sync.can_issue_mutation());
}

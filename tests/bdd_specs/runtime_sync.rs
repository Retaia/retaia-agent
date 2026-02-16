use retaia_agent::{
    ClientRuntimeTarget, PollEndpoint, PushChannel, PushHint, PushProcessResult, RuntimeSyncState,
};

#[test]
fn bdd_given_realtime_push_hint_when_fresh_then_runtime_sync_triggers_poll() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::UiRustDesktop);
    let hint = PushHint {
        issued_at_ms: 10_000,
        ttl_ms: 2_000,
    };

    let result = sync.process_push_hint(PushChannel::WebSocket, "hint-42", hint, 11_000);
    assert_eq!(result, PushProcessResult::PollTriggered);
}

#[test]
fn bdd_given_same_hint_twice_when_processed_then_second_event_is_ignored() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::UiMobile);
    let hint = PushHint {
        issued_at_ms: 10_000,
        ttl_ms: 2_000,
    };

    let first = sync.process_push_hint(PushChannel::MobileFcm, "dup", hint, 10_500);
    assert_eq!(first, PushProcessResult::PollTriggered);
    let second = sync.process_push_hint(PushChannel::MobileFcm, "dup", hint, 10_700);
    assert_eq!(second, PushProcessResult::Ignored);
}

#[test]
fn bdd_given_polled_state_compatible_when_querying_mutation_gate_then_allowed() {
    let mut sync = RuntimeSyncState::new(ClientRuntimeTarget::Agent);
    sync.observe_polled_state(true);
    assert!(sync.can_issue_mutation());
    let _ = sync.poll_by_contract(PollEndpoint::Jobs, 2_000);
}

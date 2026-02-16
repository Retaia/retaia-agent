use retaia_agent::{
    ClientRuntimeTarget, PollDecisionReason, PollEndpoint, PollSignal, PushChannel, PushHint,
    PushHintDecision, can_issue_mutation_after_poll, next_poll_decision, push_channels_allowed,
    push_is_authoritative, should_trigger_poll_from_push,
};

#[test]
fn e2e_runtime_status_driven_polling_with_push_hint_then_poll_confirmation_flow() {
    assert!(push_channels_allowed());
    assert!(!push_is_authoritative());

    let push_decision = should_trigger_poll_from_push(
        ClientRuntimeTarget::UiWeb,
        PushChannel::WebSocket,
        PushHint {
            issued_at_ms: 100,
            ttl_ms: 2_000,
        },
        500,
        false,
    );
    assert_eq!(push_decision, PushHintDecision::TriggerPoll);

    let jobs_poll = next_poll_decision(
        PollEndpoint::Jobs,
        PollSignal::ContractInterval { interval_ms: 1_000 },
        0,
        1,
    );
    assert_eq!(jobs_poll.reason, PollDecisionReason::ContractInterval);
    assert_eq!(jobs_poll.wait_ms, 1_000);

    let throttled = next_poll_decision(PollEndpoint::Jobs, PollSignal::SlowDown429, 2, 9);
    assert_eq!(throttled.reason, PollDecisionReason::BackoffFrom429);
    assert!(throttled.wait_ms >= jobs_poll.wait_ms);

    assert!(!can_issue_mutation_after_poll(false));
    assert!(can_issue_mutation_after_poll(true));
}

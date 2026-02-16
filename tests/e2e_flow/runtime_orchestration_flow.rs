use retaia_agent::{
    PollDecisionReason, PollEndpoint, PollSignal, can_issue_mutation_after_poll,
    next_poll_decision, push_channels_allowed,
};

#[test]
fn e2e_runtime_is_pull_only_with_contract_interval_then_429_backoff_flow() {
    assert!(!push_channels_allowed());

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

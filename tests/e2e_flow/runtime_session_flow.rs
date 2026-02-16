use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, LogLevel, MenuAction, PollDecisionReason,
    PollEndpoint, PollSignal, PushChannel, PushHint, RuntimeSession, RuntimeSyncPlan,
};

fn config() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 3,
        log_level: LogLevel::Info,
    }
}

#[test]
fn e2e_runtime_session_unifies_menu_sync_and_polling_flow() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, config()).expect("session");

    let push_plan = session.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::WebSocket,
        "hint-500",
        PushHint {
            issued_at_ms: 1_000,
            ttl_ms: 5_000,
        },
        1_500,
    );
    assert_eq!(
        push_plan,
        RuntimeSyncPlan::TriggerPollNow {
            endpoint: PollEndpoint::Jobs
        }
    );

    let throttled = session.on_poll_throttled(PollEndpoint::Jobs, PollSignal::SlowDown429, 2, 9);
    match throttled {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
            assert!(decision.wait_ms >= 2_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }

    let _ = session.on_poll_success(PollEndpoint::Jobs, 2_000, true);
    assert!(session.can_issue_mutation());

    let _ = session.on_menu_action(MenuAction::Stop);
    assert!(!session.can_issue_mutation());

    let stopped_push = session.on_push_hint(
        PollEndpoint::Jobs,
        PushChannel::WebSocket,
        "hint-501",
        PushHint {
            issued_at_ms: 2_000,
            ttl_ms: 5_000,
        },
        2_100,
    );
    assert_eq!(stopped_push, RuntimeSyncPlan::None);
}

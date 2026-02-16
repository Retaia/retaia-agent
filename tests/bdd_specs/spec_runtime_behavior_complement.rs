use std::cell::RefCell;

use retaia_agent::{
    AgentRunState, AgentUiRuntime, ClientRuntimeTarget, ConfigValidationError, ConnectivityState,
    NotificationBridgeError, NotificationMessage, NotificationSink, PollDecisionReason,
    PollEndpoint, PollSignal, PushChannel, PushHint, RuntimeControlAvailability,
    RuntimeControlCommand, RuntimeSession, RuntimeSnapshot, RuntimeStatusEvent,
    RuntimeStatusTracker, RuntimeSyncCoordinator, RuntimeSyncPlan, SystemNotification,
    apply_runtime_control, base_menu_actions, compact_validation_reason, dispatch_notifications,
    menu_visibility, runtime_control_availability, validate_config,
};

fn valid_config() -> retaia_agent::AgentRuntimeConfig {
    retaia_agent::AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: retaia_agent::AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: retaia_agent::LogLevel::Info,
    }
}

#[derive(Default)]
struct CaptureSink {
    titles: RefCell<Vec<String>>,
}

impl NotificationSink for CaptureSink {
    fn send(
        &self,
        message: &NotificationMessage,
        _source: &SystemNotification,
    ) -> Result<(), NotificationBridgeError> {
        self.titles.borrow_mut().push(message.title.clone());
        Ok(())
    }
}

#[test]
fn bdd_given_invalid_configuration_when_validating_then_multiple_errors_are_reported() {
    let mut config = valid_config();
    config.core_api_url = "invalid".to_string();
    config.ollama_url = "invalid".to_string();
    config.max_parallel_jobs = 0;
    let errors = validate_config(&config).expect_err("expected validation failure");
    assert!(errors.contains(&ConfigValidationError::InvalidCoreApiUrl));
    assert!(errors.contains(&ConfigValidationError::InvalidOllamaUrl));
    assert!(errors.contains(&ConfigValidationError::InvalidMaxParallelJobs));
}

#[test]
fn bdd_given_validation_errors_when_compacting_reason_then_message_is_human_readable() {
    let reason = compact_validation_reason(&[
        ConfigValidationError::InvalidCoreApiUrl,
        ConfigValidationError::EmptySecretKey,
    ]);
    assert_eq!(reason, "invalid core api url, empty secret key");
}

#[test]
fn bdd_given_runtime_status_tracker_when_failure_updates_repeat_then_last_error_is_kept() {
    let mut tracker = RuntimeStatusTracker::new();
    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-9".to_string(),
        asset_uuid: "asset-x".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobFailed {
        job_id: "job-9".to_string(),
        error_code: "E_TIMEOUT".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobFailed {
        job_id: "job-9".to_string(),
        error_code: "E_NETWORK".to_string(),
    });
    let failed = &tracker.snapshot().failed_jobs;
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].error_code, "E_NETWORK");
}

#[test]
fn bdd_given_runtime_status_tracker_when_connectivity_auth_update_change_then_snapshot_reflects_it()
{
    let mut tracker = RuntimeStatusTracker::new();
    tracker.apply(RuntimeStatusEvent::ConnectivityChanged {
        connectivity: ConnectivityState::Reconnecting,
    });
    tracker.apply(RuntimeStatusEvent::AuthReauthRequired { required: true });
    tracker.apply(RuntimeStatusEvent::UpdateAvailable {
        version: Some("1.0.1".to_string()),
    });
    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.connectivity, ConnectivityState::Reconnecting);
    assert!(snapshot.auth_reauth_required);
    assert_eq!(snapshot.available_update.as_deref(), Some("1.0.1"));
}

#[test]
fn bdd_given_runtime_session_when_dispatching_notifications_then_report_matches_sink_deliveries() {
    let mut session =
        RuntimeSession::new(ClientRuntimeTarget::Agent, valid_config()).expect("session");
    let sink = CaptureSink::default();
    let mut snapshot = RuntimeSnapshot::default();
    snapshot.known_job_ids.insert("job-1".to_string());
    snapshot.running_job_ids.insert("job-1".to_string());
    let report = session.update_snapshot_and_dispatch(snapshot, &sink);
    assert_eq!(report.dispatch.delivered, 1);
    assert_eq!(sink.titles.borrow().as_slice(), &["New job received"]);
}

#[test]
fn bdd_given_runtime_loop_mutation_gate_when_compatible_poll_received_then_mutation_is_allowed() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    assert!(!coordinator.can_issue_mutation());
    let plan = coordinator.on_poll_success(PollEndpoint::Jobs, 1_500, true);
    match plan {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::ContractInterval);
            assert_eq!(decision.wait_ms, 1_500);
        }
        other => panic!("unexpected plan: {other:?}"),
    }
    assert!(coordinator.can_issue_mutation());
}

#[test]
fn bdd_given_runtime_sync_coordinator_when_throttled_then_backoff_plan_is_emitted() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    let plan =
        coordinator.on_poll_throttled(PollEndpoint::DeviceFlow, PollSignal::SlowDown429, 2, 9);
    match plan {
        RuntimeSyncPlan::SchedulePoll(decision) => {
            assert_eq!(decision.reason, PollDecisionReason::BackoffFrom429);
            assert!(decision.wait_ms >= 2_000);
        }
        other => panic!("unexpected plan: {other:?}"),
    }
}

#[test]
fn bdd_given_runtime_sync_coordinator_when_duplicate_push_hint_then_second_is_ignored() {
    let mut coordinator = RuntimeSyncCoordinator::new(ClientRuntimeTarget::Agent);
    let hint = PushHint {
        issued_at_ms: 100,
        ttl_ms: 5_000,
    };
    let _ = coordinator.on_push_hint(PollEndpoint::Jobs, PushChannel::Sse, "dup", hint, 200);
    let second = coordinator.on_push_hint(PollEndpoint::Jobs, PushChannel::Sse, "dup", hint, 300);
    assert_eq!(second, RuntimeSyncPlan::None);
}

#[test]
fn bdd_given_runtime_ui_when_transitioning_states_then_notifications_are_deduplicated() {
    let mut ui = AgentUiRuntime::new();

    let mut first = RuntimeSnapshot::default();
    first.running_job_ids.insert("job-1".to_string());
    let _ = ui.update_snapshot(first);

    let done = RuntimeSnapshot::default();
    assert_eq!(
        ui.update_snapshot(done.clone()),
        vec![SystemNotification::AllJobsDone]
    );
    assert!(ui.update_snapshot(done).is_empty());

    let mut disconnected = RuntimeSnapshot::default();
    disconnected.connectivity = ConnectivityState::Disconnected;
    assert_eq!(
        ui.update_snapshot(disconnected.clone()),
        vec![SystemNotification::AgentDisconnectedOrReconnecting]
    );
    assert!(ui.update_snapshot(disconnected).is_empty());
}

#[test]
fn bdd_given_runtime_control_rules_when_applying_commands_then_transitions_match_contract() {
    assert_eq!(
        runtime_control_availability(AgentRunState::Running),
        RuntimeControlAvailability {
            can_play_resume: false,
            can_pause: true,
            can_stop: true
        }
    );
    assert_eq!(
        apply_runtime_control(AgentRunState::Paused, RuntimeControlCommand::PlayResume),
        AgentRunState::Running
    );
    assert_eq!(
        apply_runtime_control(AgentRunState::Stopped, RuntimeControlCommand::Pause),
        AgentRunState::Stopped
    );
}

#[test]
fn bdd_given_menu_contract_when_rendering_visibility_and_base_actions_then_layout_is_stable() {
    let running = menu_visibility(AgentRunState::Running);
    assert!(running.show_pause);
    assert!(!running.show_play_resume);
    assert_eq!(
        base_menu_actions(),
        vec![
            retaia_agent::MenuAction::OpenStatusWindow,
            retaia_agent::MenuAction::OpenSettings,
            retaia_agent::MenuAction::Stop,
            retaia_agent::MenuAction::Quit
        ]
    );
}

#[test]
fn bdd_given_notification_bridge_when_dispatching_batch_then_all_items_are_delivered() {
    let sink = CaptureSink::default();
    let notifications = vec![
        SystemNotification::NewJobReceived {
            job_id: "job-10".to_string(),
        },
        SystemNotification::AllJobsDone,
    ];
    let report = dispatch_notifications(&sink, &notifications);
    assert_eq!(report.delivered, 2);
    assert!(report.failed.is_empty());
}

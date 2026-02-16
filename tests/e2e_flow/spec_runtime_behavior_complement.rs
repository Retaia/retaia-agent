use retaia_agent::{
    AgentRunState, AgentUiRuntime, ClientRuntimeTarget, ConnectivityState, JobStage,
    RuntimeControlCommand, RuntimeLoopEngine, RuntimeSnapshot, RuntimeStatusEvent,
    RuntimeStatusTracker, SystemNotification, apply_runtime_control, base_menu_actions,
    menu_visibility,
};

#[test]
fn e2e_runtime_status_tracker_to_ui_flow_emits_expected_notifications_sequence() {
    let mut tracker = RuntimeStatusTracker::new();
    let mut ui = AgentUiRuntime::new();

    tracker.apply(RuntimeStatusEvent::JobClaimed {
        job_id: "job-12".to_string(),
        asset_uuid: "asset-12".to_string(),
    });
    tracker.apply(RuntimeStatusEvent::JobProgress {
        job_id: "job-12".to_string(),
        asset_uuid: "asset-12".to_string(),
        progress_percent: 42,
        stage: JobStage::Processing,
        short_status: "processing".to_string(),
    });
    let first = ui.update_snapshot(tracker.snapshot().clone());
    assert_eq!(
        first,
        vec![SystemNotification::NewJobReceived {
            job_id: "job-12".to_string()
        }]
    );

    tracker.apply(RuntimeStatusEvent::JobCompleted {
        job_id: "job-12".to_string(),
    });
    let second = ui.update_snapshot(tracker.snapshot().clone());
    assert_eq!(second, vec![SystemNotification::AllJobsDone]);
}

#[test]
fn e2e_runtime_loop_control_and_menu_projection_follow_spec_rules() {
    let mut engine = RuntimeLoopEngine::new(ClientRuntimeTarget::Agent);
    assert_eq!(engine.run_state(), AgentRunState::Running);
    assert_eq!(
        apply_runtime_control(AgentRunState::Running, RuntimeControlCommand::Pause),
        AgentRunState::Paused
    );

    let _ = engine.apply_control(RuntimeControlCommand::Stop);
    assert_eq!(engine.run_state(), AgentRunState::Stopped);
    assert!(!engine.can_sync());

    let visibility = menu_visibility(AgentRunState::Paused);
    assert!(visibility.show_play_resume);
    assert!(!visibility.show_pause);
    assert_eq!(base_menu_actions().len(), 4);
}

#[test]
fn e2e_runtime_ui_transition_dedup_flow_is_stable_across_connectivity_changes() {
    let mut ui = AgentUiRuntime::new();
    let mut disconnected = RuntimeSnapshot::default();
    disconnected.connectivity = ConnectivityState::Disconnected;

    let first = ui.update_snapshot(disconnected.clone());
    assert_eq!(
        first,
        vec![SystemNotification::AgentDisconnectedOrReconnecting]
    );
    let second = ui.update_snapshot(disconnected.clone());
    assert!(second.is_empty());

    let mut reconnecting = RuntimeSnapshot::default();
    reconnecting.connectivity = ConnectivityState::Reconnecting;
    let third = ui.update_snapshot(reconnecting);
    assert_eq!(
        third,
        vec![SystemNotification::AgentDisconnectedOrReconnecting]
    );
}

use retaia_agent::{
    AgentRunState, RuntimeControlAvailability, RuntimeControlCommand, apply_runtime_control,
    runtime_control_availability,
};

#[test]
fn tdd_runtime_control_availability_is_state_driven() {
    let running = runtime_control_availability(AgentRunState::Running);
    assert_eq!(
        running,
        RuntimeControlAvailability {
            can_play_resume: false,
            can_pause: true,
            can_stop: true,
        }
    );

    let paused = runtime_control_availability(AgentRunState::Paused);
    assert_eq!(
        paused,
        RuntimeControlAvailability {
            can_play_resume: true,
            can_pause: false,
            can_stop: true,
        }
    );

    let stopped = runtime_control_availability(AgentRunState::Stopped);
    assert_eq!(
        stopped,
        RuntimeControlAvailability {
            can_play_resume: true,
            can_pause: false,
            can_stop: false,
        }
    );
}

#[test]
fn tdd_runtime_control_apply_transitions_follow_menu_intent() {
    assert_eq!(
        apply_runtime_control(AgentRunState::Running, RuntimeControlCommand::Pause),
        AgentRunState::Paused
    );
    assert_eq!(
        apply_runtime_control(AgentRunState::Running, RuntimeControlCommand::Stop),
        AgentRunState::Stopped
    );
    assert_eq!(
        apply_runtime_control(AgentRunState::Paused, RuntimeControlCommand::PlayResume),
        AgentRunState::Running
    );
    assert_eq!(
        apply_runtime_control(AgentRunState::Paused, RuntimeControlCommand::Stop),
        AgentRunState::Stopped
    );
    assert_eq!(
        apply_runtime_control(AgentRunState::Stopped, RuntimeControlCommand::PlayResume),
        AgentRunState::Running
    );
}

#[test]
fn tdd_runtime_control_ignores_invalid_or_redundant_commands() {
    assert_eq!(
        apply_runtime_control(AgentRunState::Stopped, RuntimeControlCommand::Stop),
        AgentRunState::Stopped
    );
    assert_eq!(
        apply_runtime_control(AgentRunState::Stopped, RuntimeControlCommand::Pause),
        AgentRunState::Stopped
    );
    assert_eq!(
        apply_runtime_control(AgentRunState::Running, RuntimeControlCommand::PlayResume),
        AgentRunState::Running
    );
}

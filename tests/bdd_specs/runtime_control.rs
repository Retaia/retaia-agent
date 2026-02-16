use retaia_agent::{AgentRunState, RuntimeControlCommand, apply_runtime_control};

#[test]
fn bdd_given_paused_agent_when_play_resume_clicked_then_agent_runs() {
    let next = apply_runtime_control(AgentRunState::Paused, RuntimeControlCommand::PlayResume);
    assert_eq!(next, AgentRunState::Running);
}

#[test]
fn bdd_given_running_agent_when_pause_clicked_then_agent_pauses() {
    let next = apply_runtime_control(AgentRunState::Running, RuntimeControlCommand::Pause);
    assert_eq!(next, AgentRunState::Paused);
}

#[test]
fn bdd_given_stopped_agent_when_stop_clicked_then_state_is_unchanged() {
    let next = apply_runtime_control(AgentRunState::Stopped, RuntimeControlCommand::Stop);
    assert_eq!(next, AgentRunState::Stopped);
}

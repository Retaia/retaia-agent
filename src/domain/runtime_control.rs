use crate::domain::runtime_ui::AgentRunState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeControlCommand {
    PlayResume,
    Pause,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeControlAvailability {
    pub can_play_resume: bool,
    pub can_pause: bool,
    pub can_stop: bool,
}

pub fn runtime_control_availability(state: AgentRunState) -> RuntimeControlAvailability {
    match state {
        AgentRunState::Running => RuntimeControlAvailability {
            can_play_resume: false,
            can_pause: true,
            can_stop: true,
        },
        AgentRunState::Paused => RuntimeControlAvailability {
            can_play_resume: true,
            can_pause: false,
            can_stop: true,
        },
        AgentRunState::Stopped => RuntimeControlAvailability {
            can_play_resume: true,
            can_pause: false,
            can_stop: false,
        },
    }
}

pub fn apply_runtime_control(
    current: AgentRunState,
    command: RuntimeControlCommand,
) -> AgentRunState {
    match (current, command) {
        (AgentRunState::Running, RuntimeControlCommand::Pause) => AgentRunState::Paused,
        (AgentRunState::Running, RuntimeControlCommand::Stop) => AgentRunState::Stopped,
        (AgentRunState::Paused, RuntimeControlCommand::PlayResume) => AgentRunState::Running,
        (AgentRunState::Paused, RuntimeControlCommand::Stop) => AgentRunState::Stopped,
        (AgentRunState::Stopped, RuntimeControlCommand::PlayResume) => AgentRunState::Running,
        (state, _) => state,
    }
}

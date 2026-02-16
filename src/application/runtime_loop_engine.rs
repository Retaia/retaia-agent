use crate::application::runtime_sync_coordinator::{RuntimeSyncCoordinator, RuntimeSyncPlan};
use crate::domain::runtime_control::{RuntimeControlCommand, apply_runtime_control};
use crate::domain::runtime_orchestration::{
    ClientRuntimeTarget, PollEndpoint, PollSignal, PushChannel, PushHint,
};
use crate::domain::runtime_ui::AgentRunState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLoopEngine {
    run_state: AgentRunState,
    sync_coordinator: RuntimeSyncCoordinator,
}

impl RuntimeLoopEngine {
    pub fn new(target: ClientRuntimeTarget) -> Self {
        Self {
            run_state: AgentRunState::Running,
            sync_coordinator: RuntimeSyncCoordinator::new(target),
        }
    }

    pub fn run_state(&self) -> AgentRunState {
        self.run_state
    }

    pub fn target(&self) -> ClientRuntimeTarget {
        self.sync_coordinator.target()
    }

    pub fn apply_control(&mut self, command: RuntimeControlCommand) -> AgentRunState {
        self.run_state = apply_runtime_control(self.run_state, command);
        self.run_state
    }

    pub fn can_sync(&self) -> bool {
        self.run_state != AgentRunState::Stopped
    }

    pub fn can_issue_mutation(&self) -> bool {
        self.run_state == AgentRunState::Running && self.sync_coordinator.can_issue_mutation()
    }

    pub fn on_push_hint(
        &mut self,
        endpoint: PollEndpoint,
        channel: PushChannel,
        hint_id: &str,
        hint: PushHint,
        now_ms: u64,
    ) -> RuntimeSyncPlan {
        if !self.can_sync() {
            return RuntimeSyncPlan::None;
        }
        self.sync_coordinator
            .on_push_hint(endpoint, channel, hint_id, hint, now_ms)
    }

    pub fn on_poll_success(
        &mut self,
        endpoint: PollEndpoint,
        contract_interval_ms: u64,
        compatible_for_mutation: bool,
    ) -> RuntimeSyncPlan {
        if !self.can_sync() {
            return RuntimeSyncPlan::None;
        }
        self.sync_coordinator.on_poll_success(
            endpoint,
            contract_interval_ms,
            compatible_for_mutation,
        )
    }

    pub fn on_poll_throttled(
        &mut self,
        endpoint: PollEndpoint,
        signal: PollSignal,
        attempt: u32,
        jitter_seed: u64,
    ) -> RuntimeSyncPlan {
        if !self.can_sync() {
            return RuntimeSyncPlan::None;
        }
        self.sync_coordinator
            .on_poll_throttled(endpoint, signal, attempt, jitter_seed)
    }
}

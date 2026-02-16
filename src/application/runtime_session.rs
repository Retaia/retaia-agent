use crate::application::agent_runtime_app::{AgentRuntimeApp, RuntimeStatusView, TrayMenuModel};
use crate::application::runtime_loop_engine::RuntimeLoopEngine;
use crate::application::runtime_sync_coordinator::RuntimeSyncPlan;
use crate::domain::configuration::{AgentRuntimeConfig, ConfigValidationError};
use crate::domain::runtime_control::RuntimeControlCommand;
use crate::domain::runtime_orchestration::{
    ClientRuntimeTarget, PollEndpoint, PollSignal, PushChannel, PushHint,
};
use crate::domain::runtime_ui::{AgentRunState, MenuAction, RuntimeSnapshot, SystemNotification};

#[derive(Debug, Clone)]
pub struct RuntimeSession {
    app: AgentRuntimeApp,
    loop_engine: RuntimeLoopEngine,
}

impl RuntimeSession {
    pub fn new(
        target: ClientRuntimeTarget,
        settings: AgentRuntimeConfig,
    ) -> Result<Self, Vec<ConfigValidationError>> {
        Ok(Self {
            app: AgentRuntimeApp::new(settings)?,
            loop_engine: RuntimeLoopEngine::new(target),
        })
    }

    pub fn run_state(&self) -> AgentRunState {
        self.app.run_state()
    }

    pub fn target(&self) -> ClientRuntimeTarget {
        self.loop_engine.target()
    }

    pub fn settings(&self) -> &AgentRuntimeConfig {
        self.app.settings()
    }

    pub fn tray_menu_model(&self) -> TrayMenuModel {
        self.app.tray_menu_model()
    }

    pub fn status_view(&self) -> RuntimeStatusView {
        self.app.status_view()
    }

    pub fn on_menu_action(&mut self, action: MenuAction) -> AgentRunState {
        self.app.apply_menu_action(action);
        if let Some(command) = command_from_menu_action(action) {
            self.loop_engine.apply_control(command);
        }
        debug_assert_eq!(self.app.run_state(), self.loop_engine.run_state());
        self.app.run_state()
    }

    pub fn on_push_hint(
        &mut self,
        endpoint: PollEndpoint,
        channel: PushChannel,
        hint_id: &str,
        hint: PushHint,
        now_ms: u64,
    ) -> RuntimeSyncPlan {
        self.loop_engine
            .on_push_hint(endpoint, channel, hint_id, hint, now_ms)
    }

    pub fn on_poll_success(
        &mut self,
        endpoint: PollEndpoint,
        contract_interval_ms: u64,
        compatible_for_mutation: bool,
    ) -> RuntimeSyncPlan {
        self.loop_engine
            .on_poll_success(endpoint, contract_interval_ms, compatible_for_mutation)
    }

    pub fn on_poll_throttled(
        &mut self,
        endpoint: PollEndpoint,
        signal: PollSignal,
        attempt: u32,
        jitter_seed: u64,
    ) -> RuntimeSyncPlan {
        self.loop_engine
            .on_poll_throttled(endpoint, signal, attempt, jitter_seed)
    }

    pub fn update_snapshot(&mut self, snapshot: RuntimeSnapshot) -> Vec<SystemNotification> {
        self.app.update_snapshot(snapshot)
    }

    pub fn can_issue_mutation(&self) -> bool {
        self.loop_engine.can_issue_mutation()
    }
}

fn command_from_menu_action(action: MenuAction) -> Option<RuntimeControlCommand> {
    match action {
        MenuAction::PlayResume => Some(RuntimeControlCommand::PlayResume),
        MenuAction::Pause => Some(RuntimeControlCommand::Pause),
        MenuAction::Stop => Some(RuntimeControlCommand::Stop),
        MenuAction::Quit | MenuAction::OpenStatusWindow | MenuAction::OpenSettings => None,
    }
}

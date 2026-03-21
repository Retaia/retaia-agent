use crate::application::agent_runtime_app::{AgentRuntimeApp, RuntimeStatusView, TrayMenuModel};
use crate::application::core_api_gateway::CoreServerPolicy;
use crate::application::notification_bridge::{
    NotificationDispatchReport, NotificationSink, dispatch_notifications,
};
use crate::application::runtime_loop_engine::RuntimeLoopEngine;
use crate::application::runtime_sync_coordinator::RuntimeSyncPlan;
use crate::domain::configuration::{AgentRuntimeConfig, ConfigValidationError};
use crate::domain::feature_flags::CORE_JOBS_RUNTIME_FEATURE;
use crate::domain::runtime_control::RuntimeControlCommand;
use crate::domain::runtime_orchestration::{
    ClientRuntimeTarget, PollEndpoint, PollSignal, PushChannel, PushHint,
};
use crate::domain::runtime_ui::{AgentRunState, MenuAction, RuntimeSnapshot, SystemNotification};

#[derive(Debug, Clone)]
pub struct RuntimeSession {
    app: AgentRuntimeApp,
    loop_engine: RuntimeLoopEngine,
    server_policy: CoreServerPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeNotificationReport {
    pub notifications: Vec<SystemNotification>,
    pub dispatch: NotificationDispatchReport,
}

impl RuntimeSession {
    pub fn new(
        target: ClientRuntimeTarget,
        settings: AgentRuntimeConfig,
    ) -> Result<Self, Vec<ConfigValidationError>> {
        Ok(Self {
            app: AgentRuntimeApp::new(settings)?,
            loop_engine: RuntimeLoopEngine::new(target),
            server_policy: CoreServerPolicy::default(),
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

    pub fn on_poll_throttled_tracked(
        &mut self,
        endpoint: PollEndpoint,
        signal: PollSignal,
        jitter_seed: u64,
    ) -> RuntimeSyncPlan {
        self.loop_engine
            .on_poll_throttled_tracked(endpoint, signal, jitter_seed)
    }

    pub fn update_snapshot(&mut self, snapshot: RuntimeSnapshot) -> Vec<SystemNotification> {
        self.app.update_snapshot(snapshot)
    }

    pub fn update_snapshot_and_dispatch<S: NotificationSink>(
        &mut self,
        snapshot: RuntimeSnapshot,
        sink: &S,
    ) -> RuntimeNotificationReport {
        let notifications = self.app.update_snapshot(snapshot);
        let dispatch = dispatch_notifications(sink, &notifications);
        RuntimeNotificationReport {
            notifications,
            dispatch,
        }
    }

    pub fn can_issue_mutation(&self) -> bool {
        self.loop_engine.can_issue_mutation()
    }

    pub fn apply_server_policy(&mut self, policy: CoreServerPolicy) {
        self.server_policy = policy;
    }

    pub fn server_policy(&self) -> &CoreServerPolicy {
        &self.server_policy
    }

    pub fn effective_feature_enabled(&self, feature_key: &str) -> bool {
        self.server_policy
            .feature_flags
            .get(feature_key)
            .copied()
            .unwrap_or(false)
    }

    pub fn jobs_poll_interval_ms(&self) -> u64 {
        self.server_policy
            .min_poll_interval_seconds
            .unwrap_or(5)
            .max(5)
            .saturating_mul(1_000)
    }

    pub fn can_process_jobs(&self) -> bool {
        matches!(self.target(), ClientRuntimeTarget::Agent)
            && self.effective_feature_enabled(CORE_JOBS_RUNTIME_FEATURE)
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

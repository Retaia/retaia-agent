use crate::application::core_api_gateway::{
    CoreApiGateway, CoreApiGatewayError, poll_runtime_snapshot,
};
use crate::application::notification_bridge::NotificationSink;
use crate::application::runtime_session::{RuntimeNotificationReport, RuntimeSession};
use crate::application::runtime_sync_coordinator::RuntimeSyncPlan;
use crate::domain::runtime_orchestration::{PollEndpoint, PollSignal};
use crate::domain::runtime_ui::{ConnectivityState, RuntimeSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePollCycleStatus {
    Success,
    Throttled,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePollCycleOutcome {
    pub status: RuntimePollCycleStatus,
    pub plan: RuntimeSyncPlan,
    pub report: Option<RuntimeNotificationReport>,
}

pub fn run_runtime_poll_cycle<G: CoreApiGateway + ?Sized, S: NotificationSink>(
    session: &mut RuntimeSession,
    gateway: &G,
    sink: &S,
    endpoint: PollEndpoint,
    contract_interval_ms: u64,
    jitter_seed: u64,
) -> RuntimePollCycleOutcome {
    match poll_runtime_snapshot(gateway) {
        Ok(snapshot) => {
            let plan = session.on_poll_success(endpoint, contract_interval_ms, true);
            let report = session.update_snapshot_and_dispatch(snapshot, sink);
            RuntimePollCycleOutcome {
                status: RuntimePollCycleStatus::Success,
                plan,
                report: Some(report),
            }
        }
        Err(CoreApiGatewayError::Throttled) => {
            let plan = session.on_poll_throttled(endpoint, PollSignal::SlowDown429, 1, jitter_seed);
            RuntimePollCycleOutcome {
                status: RuntimePollCycleStatus::Throttled,
                plan,
                report: None,
            }
        }
        Err(error) => {
            let degraded = degraded_snapshot_from_error(error);
            let plan = session.on_poll_success(endpoint, contract_interval_ms, false);
            let report = session.update_snapshot_and_dispatch(degraded, sink);
            RuntimePollCycleOutcome {
                status: RuntimePollCycleStatus::Degraded,
                plan,
                report: Some(report),
            }
        }
    }
}

fn degraded_snapshot_from_error(error: CoreApiGatewayError) -> RuntimeSnapshot {
    match error {
        CoreApiGatewayError::Unauthorized => RuntimeSnapshot {
            auth_reauth_required: true,
            ..RuntimeSnapshot::default()
        },
        CoreApiGatewayError::UnexpectedStatus(_) | CoreApiGatewayError::Transport(_) => {
            RuntimeSnapshot {
                connectivity: ConnectivityState::Reconnecting,
                ..RuntimeSnapshot::default()
            }
        }
        CoreApiGatewayError::Throttled => RuntimeSnapshot::default(),
    }
}

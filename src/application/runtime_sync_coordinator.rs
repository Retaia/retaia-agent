use crate::domain::runtime_orchestration::{
    ClientRuntimeTarget, PollDecision, PollEndpoint, PollSignal, PushChannel, PushHint,
};
use crate::domain::runtime_sync::{PushProcessResult, RuntimeSyncState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeSyncPlan {
    None,
    TriggerPollNow { endpoint: PollEndpoint },
    SchedulePoll(PollDecision),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSyncCoordinator {
    sync: RuntimeSyncState,
}

impl RuntimeSyncCoordinator {
    pub fn new(target: ClientRuntimeTarget) -> Self {
        Self {
            sync: RuntimeSyncState::new(target),
        }
    }

    pub fn target(&self) -> ClientRuntimeTarget {
        self.sync.target()
    }

    pub fn seen_hint_count(&self) -> usize {
        self.sync.seen_hint_count()
    }

    pub fn on_push_hint(
        &mut self,
        endpoint: PollEndpoint,
        channel: PushChannel,
        hint_id: &str,
        hint: PushHint,
        now_ms: u64,
    ) -> RuntimeSyncPlan {
        match self.sync.process_push_hint(channel, hint_id, hint, now_ms) {
            PushProcessResult::PollTriggered => RuntimeSyncPlan::TriggerPollNow { endpoint },
            PushProcessResult::Ignored => RuntimeSyncPlan::None,
        }
    }

    pub fn on_poll_success(
        &mut self,
        endpoint: PollEndpoint,
        contract_interval_ms: u64,
        compatible_for_mutation: bool,
    ) -> RuntimeSyncPlan {
        self.sync.observe_polled_state(compatible_for_mutation);
        RuntimeSyncPlan::SchedulePoll(self.sync.poll_by_contract(endpoint, contract_interval_ms))
    }

    pub fn on_poll_throttled(
        &mut self,
        endpoint: PollEndpoint,
        signal: PollSignal,
        attempt: u32,
        jitter_seed: u64,
    ) -> RuntimeSyncPlan {
        RuntimeSyncPlan::SchedulePoll(self.sync.poll_after_429(
            endpoint,
            signal,
            attempt,
            jitter_seed,
        ))
    }

    pub fn can_issue_mutation(&self) -> bool {
        self.sync.can_issue_mutation()
    }
}

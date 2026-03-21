use std::collections::{BTreeMap, BTreeSet};

use crate::domain::runtime_orchestration::{
    ClientRuntimeTarget, PollDecision, PollEndpoint, PollSignal, PushChannel, PushHint,
    PushHintDecision, can_issue_mutation_after_poll, next_poll_decision,
    should_trigger_poll_from_push,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushProcessResult {
    PollTriggered,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSyncState {
    target: ClientRuntimeTarget,
    seen_hint_ids: BTreeSet<String>,
    last_compatible_state_read: bool,
    throttle_attempts_by_endpoint: BTreeMap<PollEndpoint, u32>,
}

impl RuntimeSyncState {
    pub fn new(target: ClientRuntimeTarget) -> Self {
        Self {
            target,
            seen_hint_ids: BTreeSet::new(),
            last_compatible_state_read: false,
            throttle_attempts_by_endpoint: BTreeMap::new(),
        }
    }

    pub fn target(&self) -> ClientRuntimeTarget {
        self.target
    }

    pub fn seen_hint_count(&self) -> usize {
        self.seen_hint_ids.len()
    }

    pub fn process_push_hint(
        &mut self,
        channel: PushChannel,
        hint_id: &str,
        hint: PushHint,
        now_ms: u64,
    ) -> PushProcessResult {
        let already_seen = self.seen_hint_ids.contains(hint_id);
        let decision =
            should_trigger_poll_from_push(self.target, channel, hint, now_ms, already_seen);

        match decision {
            PushHintDecision::TriggerPoll => {
                self.seen_hint_ids.insert(hint_id.to_string());
                PushProcessResult::PollTriggered
            }
            PushHintDecision::Ignore => PushProcessResult::Ignored,
        }
    }

    pub fn poll_by_contract(&self, endpoint: PollEndpoint, interval_ms: u64) -> PollDecision {
        next_poll_decision(endpoint, PollSignal::ContractInterval { interval_ms }, 0, 0)
    }

    pub fn poll_by_contract_and_reset(
        &mut self,
        endpoint: PollEndpoint,
        interval_ms: u64,
    ) -> PollDecision {
        self.throttle_attempts_by_endpoint.remove(&endpoint);
        self.poll_by_contract(endpoint, interval_ms)
    }

    pub fn poll_after_429(
        &self,
        endpoint: PollEndpoint,
        signal: PollSignal,
        attempt: u32,
        jitter_seed: u64,
    ) -> PollDecision {
        next_poll_decision(endpoint, signal, attempt, jitter_seed)
    }

    pub fn poll_after_429_tracked(
        &mut self,
        endpoint: PollEndpoint,
        signal: PollSignal,
        jitter_seed: u64,
    ) -> PollDecision {
        let attempt = self
            .throttle_attempts_by_endpoint
            .entry(endpoint)
            .and_modify(|value| *value = value.saturating_add(1))
            .or_insert(1);
        next_poll_decision(endpoint, signal, *attempt, jitter_seed)
    }

    pub fn observe_polled_state(&mut self, compatible_for_mutation: bool) {
        self.last_compatible_state_read = compatible_for_mutation;
    }

    pub fn can_issue_mutation(&self) -> bool {
        can_issue_mutation_after_poll(self.last_compatible_state_read)
    }
}

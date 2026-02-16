#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeOrchestrationMode {
    StatusDrivenPolling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientRuntimeTarget {
    Agent,
    Mcp,
    UiRustDesktop,
    UiMobile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushChannel {
    WebSocket,
    Sse,
    Webhook,
    MobileFcm,
    MobileApns,
    MobileEpns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PushHint {
    pub issued_at_ms: u64,
    pub ttl_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushHintDecision {
    Ignore,
    TriggerPoll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollEndpoint {
    Jobs,
    Policy,
    DeviceFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollSignal {
    ContractInterval { interval_ms: u64 },
    TooManyAttempts429,
    SlowDown429,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollDecisionReason {
    ContractInterval,
    BackoffFrom429,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PollDecision {
    pub endpoint: PollEndpoint,
    pub wait_ms: u64,
    pub reason: PollDecisionReason,
}

const MIN_INTERVAL_MS: u64 = 100;
const MAX_BACKOFF_MS: u64 = 60_000;

pub fn runtime_orchestration_mode() -> RuntimeOrchestrationMode {
    RuntimeOrchestrationMode::StatusDrivenPolling
}

pub fn push_channels_allowed() -> bool {
    true
}

pub fn push_is_authoritative() -> bool {
    false
}

pub fn mobile_push_allowed_for_target(target: ClientRuntimeTarget) -> bool {
    matches!(target, ClientRuntimeTarget::UiMobile)
}

pub fn is_push_channel_supported_for_target(
    target: ClientRuntimeTarget,
    channel: PushChannel,
) -> bool {
    match channel {
        PushChannel::WebSocket | PushChannel::Sse | PushChannel::Webhook => true,
        PushChannel::MobileFcm | PushChannel::MobileApns | PushChannel::MobileEpns => {
            mobile_push_allowed_for_target(target)
        }
    }
}

pub fn should_trigger_poll_from_push(
    target: ClientRuntimeTarget,
    channel: PushChannel,
    hint: PushHint,
    now_ms: u64,
    already_seen_hint: bool,
) -> PushHintDecision {
    if !is_push_channel_supported_for_target(target, channel) {
        return PushHintDecision::Ignore;
    }
    if !is_push_hint_fresh(hint, now_ms) {
        return PushHintDecision::Ignore;
    }
    if already_seen_hint {
        return PushHintDecision::Ignore;
    }
    PushHintDecision::TriggerPoll
}

pub fn is_push_hint_fresh(hint: PushHint, now_ms: u64) -> bool {
    if hint.ttl_ms == 0 {
        return false;
    }
    let expires_at = hint.issued_at_ms.saturating_add(hint.ttl_ms);
    now_ms <= expires_at
}

pub fn can_issue_mutation_after_poll(compatible_state_read: bool) -> bool {
    compatible_state_read
}

pub fn next_poll_decision(
    endpoint: PollEndpoint,
    signal: PollSignal,
    attempt: u32,
    jitter_seed: u64,
) -> PollDecision {
    match signal {
        PollSignal::ContractInterval { interval_ms } => PollDecision {
            endpoint,
            wait_ms: interval_ms.max(MIN_INTERVAL_MS),
            reason: PollDecisionReason::ContractInterval,
        },
        PollSignal::TooManyAttempts429 | PollSignal::SlowDown429 => PollDecision {
            endpoint,
            wait_ms: throttled_backoff_with_jitter(attempt, jitter_seed),
            reason: PollDecisionReason::BackoffFrom429,
        },
    }
}

pub fn throttled_backoff_with_jitter(attempt: u32, jitter_seed: u64) -> u64 {
    let exp = attempt.min(10);
    let base = 500_u64.saturating_mul(1_u64 << exp).min(MAX_BACKOFF_MS);
    let jitter_cap = (base / 5).max(1);
    let jitter = jitter_seed % (jitter_cap + 1);
    (base + jitter).min(MAX_BACKOFF_MS)
}

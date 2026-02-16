#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeOrchestrationMode {
    PullOnly,
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
    RuntimeOrchestrationMode::PullOnly
}

pub fn push_channels_allowed() -> bool {
    false
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

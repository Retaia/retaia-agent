use std::sync::{Mutex, MutexGuard, OnceLock};

use retaia_agent::{NotificationBridgeError, NotificationMessage};

#[derive(Debug, Clone)]
enum MockOutcome {
    Ok,
    Err(String),
}

#[derive(Debug, Clone)]
struct MockState {
    outcome: MockOutcome,
    call_count: usize,
    received_titles: Vec<String>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            outcome: MockOutcome::Ok,
            call_count: 0,
            received_titles: Vec::new(),
        }
    }
}

fn state() -> &'static Mutex<MockState> {
    static STATE: OnceLock<Mutex<MockState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(MockState::default()))
}

fn test_lock() -> &'static Mutex<()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

pub struct MockDispatcherScope {
    _guard: MutexGuard<'static, ()>,
}

impl MockDispatcherScope {
    pub fn new() -> Self {
        let guard = test_lock()
            .lock()
            .expect("mock dispatcher lock should not be poisoned");
        reset_locked();
        Self { _guard: guard }
    }

    #[allow(dead_code)]
    pub fn set_ok(&self) {
        let mut state = state()
            .lock()
            .expect("mock dispatcher state should not be poisoned");
        state.outcome = MockOutcome::Ok;
    }

    pub fn set_error(&self, message: &str) {
        let mut state = state()
            .lock()
            .expect("mock dispatcher state should not be poisoned");
        state.outcome = MockOutcome::Err(message.to_string());
    }

    pub fn call_count(&self) -> usize {
        let state = state()
            .lock()
            .expect("mock dispatcher state should not be poisoned");
        state.call_count
    }

    #[allow(dead_code)]
    pub fn received_titles(&self) -> Vec<String> {
        let state = state()
            .lock()
            .expect("mock dispatcher state should not be poisoned");
        state.received_titles.clone()
    }
}

impl Drop for MockDispatcherScope {
    fn drop(&mut self) {
        reset_locked();
    }
}

fn reset_locked() {
    let mut state = state()
        .lock()
        .expect("mock dispatcher state should not be poisoned");
    *state = MockState::default();
}

pub fn dispatch(message: &NotificationMessage) -> Result<(), NotificationBridgeError> {
    let mut state = state()
        .lock()
        .expect("mock dispatcher state should not be poisoned");
    state.call_count += 1;
    state.received_titles.push(message.title.clone());

    match &state.outcome {
        MockOutcome::Ok => Ok(()),
        MockOutcome::Err(error) => Err(NotificationBridgeError::SinkFailed(error.clone())),
    }
}

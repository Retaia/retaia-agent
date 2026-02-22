use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::runtime_ui::AgentRunState;
use crate::infrastructure::config_store::{ConfigStoreError, system_config_file_path};

pub const DAEMON_STATS_FILE_NAME: &str = "daemon-stats.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonCurrentJobStats {
    pub job_id: String,
    pub asset_uuid: String,
    pub progress_percent: u8,
    pub stage: String,
    pub status: String,
    pub started_at_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonLastJobStats {
    pub job_id: String,
    pub duration_ms: u64,
    pub completed_at_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonRuntimeStats {
    pub updated_at_unix_ms: u64,
    pub run_state: String,
    pub tick: u64,
    pub current_job: Option<DaemonCurrentJobStats>,
    pub last_job: Option<DaemonLastJobStats>,
}

impl DaemonRuntimeStats {
    pub fn new_idle(tick: u64) -> Self {
        Self {
            updated_at_unix_ms: now_unix_ms(),
            run_state: run_state_label(AgentRunState::Running).to_string(),
            tick,
            current_job: None,
            last_job: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum RuntimeStatsStoreError {
    #[error("stats file path unavailable: {0}")]
    Path(ConfigStoreError),
    #[error("stats file not found")]
    NotFound,
    #[error("io error: {0}")]
    Io(io::Error),
    #[error("json decode error: {0}")]
    JsonDecode(serde_json::Error),
    #[error("json encode error: {0}")]
    JsonEncode(serde_json::Error),
}

pub fn runtime_stats_file_path() -> Result<PathBuf, RuntimeStatsStoreError> {
    let config_path = system_config_file_path().map_err(RuntimeStatsStoreError::Path)?;
    let parent = config_path.parent().ok_or(RuntimeStatsStoreError::Path(
        ConfigStoreError::SystemConfigDirectoryUnavailable,
    ))?;
    Ok(parent.join(DAEMON_STATS_FILE_NAME))
}

pub fn save_runtime_stats(stats: &DaemonRuntimeStats) -> Result<PathBuf, RuntimeStatsStoreError> {
    let path = runtime_stats_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(RuntimeStatsStoreError::Io)?;
    }
    let payload = serde_json::to_vec_pretty(stats).map_err(RuntimeStatsStoreError::JsonEncode)?;
    fs::write(&path, payload).map_err(RuntimeStatsStoreError::Io)?;
    Ok(path)
}

pub fn load_runtime_stats() -> Result<DaemonRuntimeStats, RuntimeStatsStoreError> {
    let path = runtime_stats_file_path()?;
    let content = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(RuntimeStatsStoreError::NotFound);
        }
        Err(error) => return Err(RuntimeStatsStoreError::Io(error)),
    };
    serde_json::from_str(&content).map_err(RuntimeStatsStoreError::JsonDecode)
}

pub fn run_state_label(state: AgentRunState) -> &'static str {
    match state {
        AgentRunState::Running => "running",
        AgentRunState::Paused => "paused",
        AgentRunState::Stopped => "stopped",
    }
}

pub fn now_unix_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonLevel {
    User,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonStatus {
    NotInstalled,
    Running,
    Stopped(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonInstallRequest {
    pub label: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub level: DaemonLevel,
    pub autostart: bool,
    pub username: Option<String>,
    pub working_directory: Option<PathBuf>,
    pub environment: Option<Vec<(String, String)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonLabelRequest {
    pub label: String,
    pub level: DaemonLevel,
}

#[derive(Debug, Error)]
pub enum DaemonManagerError {
    #[error("service manager unavailable: {0}")]
    Unavailable(String),
    #[error("invalid daemon label: {0}")]
    InvalidLabel(String),
    #[error("daemon operation failed: {0}")]
    OperationFailed(String),
}

pub trait DaemonManager {
    fn install(&self, request: DaemonInstallRequest) -> Result<(), DaemonManagerError>;
    fn uninstall(&self, request: DaemonLabelRequest) -> Result<(), DaemonManagerError>;
    fn start(&self, request: DaemonLabelRequest) -> Result<(), DaemonManagerError>;
    fn stop(&self, request: DaemonLabelRequest) -> Result<(), DaemonManagerError>;
    fn status(&self, request: DaemonLabelRequest) -> Result<DaemonStatus, DaemonManagerError>;
}

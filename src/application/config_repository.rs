use std::path::PathBuf;

use crate::domain::configuration::{AgentRuntimeConfig, ConfigValidationError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigRepositoryError {
    #[error("storage unavailable")]
    StorageUnavailable,
    #[error("configuration not found")]
    NotFound,
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error("persistence error: {0}")]
    Persistence(String),
    #[error("validation error")]
    Validation(Vec<ConfigValidationError>),
}

pub trait ConfigRepository {
    fn load(&self) -> Result<AgentRuntimeConfig, ConfigRepositoryError>;
    fn save(&self, config: &AgentRuntimeConfig) -> Result<(), ConfigRepositoryError>;
    fn config_path(&self) -> Result<PathBuf, ConfigRepositoryError>;
}

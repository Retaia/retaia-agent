use std::path::PathBuf;

use crate::domain::configuration::{AgentRuntimeConfig, ConfigValidationError};

#[derive(Debug)]
pub enum ConfigRepositoryError {
    StorageUnavailable,
    NotFound,
    InvalidData(String),
    Persistence(String),
    Validation(Vec<ConfigValidationError>),
}

pub trait ConfigRepository {
    fn load(&self) -> Result<AgentRuntimeConfig, ConfigRepositoryError>;
    fn save(&self, config: &AgentRuntimeConfig) -> Result<(), ConfigRepositoryError>;
    fn config_path(&self) -> Result<PathBuf, ConfigRepositoryError>;
}

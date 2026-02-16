use std::path::PathBuf;

use crate::application::config_repository::{ConfigRepository, ConfigRepositoryError};
use crate::domain::configuration::AgentRuntimeConfig;
use crate::infrastructure::config_store::{
    ConfigStoreError, load_config_from_path, load_system_config, save_config_to_path,
    save_system_config, system_config_file_path,
};

fn map_store_error(error: ConfigStoreError) -> ConfigRepositoryError {
    match error {
        ConfigStoreError::SystemConfigDirectoryUnavailable => {
            ConfigRepositoryError::StorageUnavailable
        }
        ConfigStoreError::Io(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => {
            ConfigRepositoryError::NotFound
        }
        ConfigStoreError::Io(io_error) => ConfigRepositoryError::Persistence(io_error.to_string()),
        ConfigStoreError::TomlDecode(error) => {
            ConfigRepositoryError::InvalidData(error.to_string())
        }
        ConfigStoreError::TomlEncode(error) => {
            ConfigRepositoryError::Persistence(error.to_string())
        }
        ConfigStoreError::Validation(errors) => ConfigRepositoryError::Validation(errors),
    }
}

#[derive(Debug, Clone)]
pub struct FileConfigRepository {
    path: PathBuf,
}

impl FileConfigRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ConfigRepository for FileConfigRepository {
    fn load(&self) -> Result<AgentRuntimeConfig, ConfigRepositoryError> {
        load_config_from_path(&self.path).map_err(map_store_error)
    }

    fn save(&self, config: &AgentRuntimeConfig) -> Result<(), ConfigRepositoryError> {
        save_config_to_path(&self.path, config).map_err(map_store_error)
    }

    fn config_path(&self) -> Result<PathBuf, ConfigRepositoryError> {
        Ok(self.path.clone())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemConfigRepository;

impl ConfigRepository for SystemConfigRepository {
    fn load(&self) -> Result<AgentRuntimeConfig, ConfigRepositoryError> {
        load_system_config().map_err(map_store_error)
    }

    fn save(&self, config: &AgentRuntimeConfig) -> Result<(), ConfigRepositoryError> {
        save_system_config(config)
            .map(|_| ())
            .map_err(map_store_error)
    }

    fn config_path(&self) -> Result<PathBuf, ConfigRepositoryError> {
        system_config_file_path().map_err(map_store_error)
    }
}

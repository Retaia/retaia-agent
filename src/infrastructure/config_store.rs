use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::configuration::{
    AgentRuntimeConfig, AuthMode, ConfigValidationError, LogLevel, TechnicalAuthConfig,
    validate_config,
};

pub const CONFIG_FILE_ENV: &str = "RETAIA_AGENT_CONFIG_PATH";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Error)]
pub enum ConfigStoreError {
    #[error("system config directory unavailable")]
    SystemConfigDirectoryUnavailable,
    #[error("io error: {0}")]
    Io(io::Error),
    #[error("toml decode error: {0}")]
    TomlDecode(toml::de::Error),
    #[error("toml encode error: {0}")]
    TomlEncode(toml::ser::Error),
    #[error("config validation failed")]
    Validation(Vec<ConfigValidationError>),
}

impl From<io::Error> for ConfigStoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for ConfigStoreError {
    fn from(error: toml::de::Error) -> Self {
        Self::TomlDecode(error)
    }
}

impl From<toml::ser::Error> for ConfigStoreError {
    fn from(error: toml::ser::Error) -> Self {
        Self::TomlEncode(error)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StoredAuthMode {
    Interactive,
    Technical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StoredLogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredTechnicalAuthConfig {
    client_id: String,
    secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAgentRuntimeConfig {
    core_api_url: String,
    ollama_url: String,
    auth_mode: StoredAuthMode,
    technical_auth: Option<StoredTechnicalAuthConfig>,
    max_parallel_jobs: u16,
    log_level: StoredLogLevel,
}

impl From<StoredAuthMode> for AuthMode {
    fn from(value: StoredAuthMode) -> Self {
        match value {
            StoredAuthMode::Interactive => AuthMode::Interactive,
            StoredAuthMode::Technical => AuthMode::Technical,
        }
    }
}

impl From<AuthMode> for StoredAuthMode {
    fn from(value: AuthMode) -> Self {
        match value {
            AuthMode::Interactive => StoredAuthMode::Interactive,
            AuthMode::Technical => StoredAuthMode::Technical,
        }
    }
}

impl From<StoredLogLevel> for LogLevel {
    fn from(value: StoredLogLevel) -> Self {
        match value {
            StoredLogLevel::Error => LogLevel::Error,
            StoredLogLevel::Warn => LogLevel::Warn,
            StoredLogLevel::Info => LogLevel::Info,
            StoredLogLevel::Debug => LogLevel::Debug,
            StoredLogLevel::Trace => LogLevel::Trace,
        }
    }
}

impl From<LogLevel> for StoredLogLevel {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => StoredLogLevel::Error,
            LogLevel::Warn => StoredLogLevel::Warn,
            LogLevel::Info => StoredLogLevel::Info,
            LogLevel::Debug => StoredLogLevel::Debug,
            LogLevel::Trace => StoredLogLevel::Trace,
        }
    }
}

impl From<StoredTechnicalAuthConfig> for TechnicalAuthConfig {
    fn from(value: StoredTechnicalAuthConfig) -> Self {
        Self {
            client_id: value.client_id,
            secret_key: value.secret_key,
        }
    }
}

impl From<TechnicalAuthConfig> for StoredTechnicalAuthConfig {
    fn from(value: TechnicalAuthConfig) -> Self {
        Self {
            client_id: value.client_id,
            secret_key: value.secret_key,
        }
    }
}

impl From<StoredAgentRuntimeConfig> for AgentRuntimeConfig {
    fn from(value: StoredAgentRuntimeConfig) -> Self {
        Self {
            core_api_url: value.core_api_url,
            ollama_url: value.ollama_url,
            auth_mode: value.auth_mode.into(),
            technical_auth: value.technical_auth.map(Into::into),
            max_parallel_jobs: value.max_parallel_jobs,
            log_level: value.log_level.into(),
        }
    }
}

impl From<AgentRuntimeConfig> for StoredAgentRuntimeConfig {
    fn from(value: AgentRuntimeConfig) -> Self {
        Self {
            core_api_url: value.core_api_url,
            ollama_url: value.ollama_url,
            auth_mode: value.auth_mode.into(),
            technical_auth: value.technical_auth.map(Into::into),
            max_parallel_jobs: value.max_parallel_jobs,
            log_level: value.log_level.into(),
        }
    }
}

pub fn system_config_file_path() -> Result<PathBuf, ConfigStoreError> {
    if let Ok(override_path) = env::var(CONFIG_FILE_ENV) {
        let override_path = override_path.trim();
        if !override_path.is_empty() {
            return Ok(PathBuf::from(override_path));
        }
    }

    let dirs = ProjectDirs::from("io", "Retaia", "retaia-agent")
        .ok_or(ConfigStoreError::SystemConfigDirectoryUnavailable)?;
    Ok(dirs.config_dir().join(CONFIG_FILE_NAME))
}

pub fn save_config_to_path(
    path: &Path,
    config: &AgentRuntimeConfig,
) -> Result<(), ConfigStoreError> {
    validate_config(config).map_err(ConfigStoreError::Validation)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let raw: StoredAgentRuntimeConfig = config.clone().into();
    let toml = toml::to_string_pretty(&raw)?;
    fs::write(path, toml)?;
    Ok(())
}

pub fn load_config_from_path(path: &Path) -> Result<AgentRuntimeConfig, ConfigStoreError> {
    let content = fs::read_to_string(path)?;
    let stored: StoredAgentRuntimeConfig = toml::from_str(&content)?;
    let config: AgentRuntimeConfig = stored.into();
    validate_config(&config).map_err(ConfigStoreError::Validation)?;
    Ok(config)
}

pub fn save_system_config(config: &AgentRuntimeConfig) -> Result<PathBuf, ConfigStoreError> {
    let path = system_config_file_path()?;
    save_config_to_path(&path, config)?;
    Ok(path)
}

pub fn load_system_config() -> Result<AgentRuntimeConfig, ConfigStoreError> {
    let path = system_config_file_path()?;
    load_config_from_path(&path)
}

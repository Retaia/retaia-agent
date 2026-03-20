use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::configuration::{
    AgentRuntimeConfig, AuthMode, ConfigValidationError, LogLevel, TechnicalAuthConfig,
    normalize_storage_mounts, validate_config,
};
use crate::infrastructure::technical_secret_store::{
    delete_technical_secret, load_technical_secret, persist_technical_secret,
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
    #[error("secret store error: {0}")]
    SecretStore(String),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAgentRuntimeConfig {
    core_api_url: String,
    ollama_url: String,
    auth_mode: StoredAuthMode,
    technical_auth: Option<StoredTechnicalAuthConfig>,
    #[serde(default)]
    storage_mounts: BTreeMap<String, String>,
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
            secret_key: value.secret_key.unwrap_or_default(),
        }
    }
}

impl From<TechnicalAuthConfig> for StoredTechnicalAuthConfig {
    fn from(value: TechnicalAuthConfig) -> Self {
        Self {
            client_id: value.client_id,
            secret_key: None,
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
            storage_mounts: normalize_storage_mounts(&value.storage_mounts),
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
            storage_mounts: normalize_storage_mounts(&value.storage_mounts),
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

    let previous = if path.exists() {
        Some(toml::from_str::<StoredAgentRuntimeConfig>(
            &fs::read_to_string(path)?,
        )?)
    } else {
        None
    };

    let raw: StoredAgentRuntimeConfig = config.clone().into();
    sync_technical_secret(path, previous.as_ref(), config)?;
    let toml = toml::to_string_pretty(&raw)?;
    fs::write(path, toml)?;
    Ok(())
}

pub fn load_config_from_path(path: &Path) -> Result<AgentRuntimeConfig, ConfigStoreError> {
    let content = fs::read_to_string(path)?;
    let stored: StoredAgentRuntimeConfig = toml::from_str(&content)?;
    let (config, migrated_legacy_secret) = hydrate_runtime_config(path, stored)?;
    if migrated_legacy_secret {
        save_config_to_path(path, &config)?;
    }
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

fn hydrate_runtime_config(
    path: &Path,
    stored: StoredAgentRuntimeConfig,
) -> Result<(AgentRuntimeConfig, bool), ConfigStoreError> {
    let mut migrated_legacy_secret = false;
    let technical_auth = match stored.technical_auth {
        Some(technical) => {
            if let Some(secret_key) = technical.secret_key.as_deref() {
                persist_technical_secret(path, &technical.client_id, secret_key)
                    .map_err(ConfigStoreError::SecretStore)?;
                migrated_legacy_secret = true;
            }
            let secret_key = load_technical_secret(path, &technical.client_id)
                .map_err(ConfigStoreError::SecretStore)?;
            Some(TechnicalAuthConfig {
                client_id: technical.client_id,
                secret_key,
            })
        }
        None => None,
    };

    Ok((
        AgentRuntimeConfig {
            core_api_url: stored.core_api_url,
            ollama_url: stored.ollama_url,
            auth_mode: stored.auth_mode.into(),
            technical_auth,
            storage_mounts: normalize_storage_mounts(&stored.storage_mounts),
            max_parallel_jobs: stored.max_parallel_jobs,
            log_level: stored.log_level.into(),
        },
        migrated_legacy_secret,
    ))
}

fn sync_technical_secret(
    path: &Path,
    previous: Option<&StoredAgentRuntimeConfig>,
    config: &AgentRuntimeConfig,
) -> Result<(), ConfigStoreError> {
    let previous_client_id = previous
        .and_then(|value| value.technical_auth.as_ref())
        .map(|technical| technical.client_id.as_str());
    let next_client_id = config
        .technical_auth
        .as_ref()
        .map(|technical| technical.client_id.as_str());

    if let Some(client_id) = previous_client_id {
        if next_client_id != Some(client_id) {
            delete_technical_secret(path, client_id).map_err(ConfigStoreError::SecretStore)?;
        }
    }

    if let Some(technical) = &config.technical_auth {
        persist_technical_secret(path, &technical.client_id, &technical.secret_key)
            .map_err(ConfigStoreError::SecretStore)?;
    }

    Ok(())
}

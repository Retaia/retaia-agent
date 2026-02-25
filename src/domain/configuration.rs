use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    Interactive,
    Technical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechnicalAuthConfig {
    pub client_id: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRuntimeConfig {
    pub core_api_url: String,
    pub ollama_url: String,
    pub auth_mode: AuthMode,
    pub technical_auth: Option<TechnicalAuthConfig>,
    pub storage_mounts: BTreeMap<String, String>,
    pub max_parallel_jobs: u16,
    pub log_level: LogLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValidationError {
    InvalidCoreApiUrl,
    CoreApiUrlDockerHostnameForbidden,
    InvalidOllamaUrl,
    MissingTechnicalAuth,
    EmptyClientId,
    EmptySecretKey,
    EmptyStorageMountId,
    StorageMountPathNotAbsolute(String),
    InvalidMaxParallelJobs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigField {
    CoreApiUrl,
    OllamaUrl,
    AuthMode,
    TechnicalClientId,
    TechnicalSecretKey,
    StorageMounts,
    MaxParallelJobs,
    LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigInterface {
    Gui,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeConfigUpdate {
    pub core_api_url: Option<String>,
    pub ollama_url: Option<String>,
    pub auth_mode: Option<AuthMode>,
    pub technical_client_id: Option<String>,
    pub technical_secret_key: Option<String>,
    pub clear_technical_auth: bool,
    pub storage_mounts: Option<BTreeMap<String, String>>,
    pub clear_storage_mounts: bool,
    pub max_parallel_jobs: Option<u16>,
    pub log_level: Option<LogLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SourcePathResolveError {
    #[error("unknown storage_id: {0}")]
    UnknownStorageId(String),
    #[error("unsafe relative path: {0}")]
    UnsafeRelativePath(String),
}

fn is_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn is_docker_internal_like_hostname(value: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(value) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let lowercase = host.to_ascii_lowercase();
    if lowercase == "localhost" {
        return false;
    }
    if host.parse::<IpAddr>().is_ok() {
        return false;
    }
    !host.contains('.')
}

pub fn normalize_core_api_url(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/').to_string();
    if !is_http_url(&trimmed) {
        return trimmed;
    }
    if trimmed.ends_with("/api/v1") {
        return trimmed;
    }
    format!("{trimmed}/api/v1")
}

pub fn validate_config(config: &AgentRuntimeConfig) -> Result<(), Vec<ConfigValidationError>> {
    let mut errors = Vec::new();

    if !is_http_url(&config.core_api_url) {
        errors.push(ConfigValidationError::InvalidCoreApiUrl);
    } else if is_docker_internal_like_hostname(&config.core_api_url) {
        errors.push(ConfigValidationError::CoreApiUrlDockerHostnameForbidden);
    }
    if !is_http_url(&config.ollama_url) {
        errors.push(ConfigValidationError::InvalidOllamaUrl);
    }

    if config.max_parallel_jobs == 0 {
        errors.push(ConfigValidationError::InvalidMaxParallelJobs);
    }

    for (storage_id, mount_path) in &config.storage_mounts {
        if storage_id.trim().is_empty() {
            errors.push(ConfigValidationError::EmptyStorageMountId);
        }
        if !Path::new(mount_path).is_absolute() {
            errors.push(ConfigValidationError::StorageMountPathNotAbsolute(
                storage_id.clone(),
            ));
        }
    }

    if config.auth_mode == AuthMode::Technical {
        match &config.technical_auth {
            None => errors.push(ConfigValidationError::MissingTechnicalAuth),
            Some(technical) => {
                if technical.client_id.trim().is_empty() {
                    errors.push(ConfigValidationError::EmptyClientId);
                }
                if technical.secret_key.trim().is_empty() {
                    errors.push(ConfigValidationError::EmptySecretKey);
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn compact_validation_reason(errors: &[ConfigValidationError]) -> String {
    errors
        .iter()
        .map(|err| match err {
            ConfigValidationError::InvalidCoreApiUrl => "invalid core api url",
            ConfigValidationError::CoreApiUrlDockerHostnameForbidden => {
                "core api url uses forbidden docker-internal hostname"
            }
            ConfigValidationError::InvalidOllamaUrl => "invalid ollama url",
            ConfigValidationError::MissingTechnicalAuth => "missing technical auth",
            ConfigValidationError::EmptyClientId => "empty client id",
            ConfigValidationError::EmptySecretKey => "empty secret key",
            ConfigValidationError::EmptyStorageMountId => "empty storage mount id",
            ConfigValidationError::StorageMountPathNotAbsolute(_) => {
                "storage mount path is not absolute"
            }
            ConfigValidationError::InvalidMaxParallelJobs => "invalid max_parallel_jobs",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn supported_config_fields(interface: ConfigInterface) -> BTreeSet<ConfigField> {
    let _ = interface;
    BTreeSet::from([
        ConfigField::CoreApiUrl,
        ConfigField::OllamaUrl,
        ConfigField::AuthMode,
        ConfigField::TechnicalClientId,
        ConfigField::TechnicalSecretKey,
        ConfigField::StorageMounts,
        ConfigField::MaxParallelJobs,
        ConfigField::LogLevel,
    ])
}

pub fn apply_config_update(
    current: &AgentRuntimeConfig,
    update: &RuntimeConfigUpdate,
    interface: ConfigInterface,
) -> Result<AgentRuntimeConfig, Vec<ConfigValidationError>> {
    let mut next = current.clone();

    if let Some(core_api_url) = &update.core_api_url {
        next.core_api_url = normalize_core_api_url(core_api_url);
    }
    if let Some(ollama_url) = &update.ollama_url {
        next.ollama_url = ollama_url.clone();
    }
    if let Some(auth_mode) = update.auth_mode {
        next.auth_mode = auth_mode;
    }

    if update.clear_technical_auth {
        next.technical_auth = None;
    }
    if update.clear_storage_mounts {
        next.storage_mounts.clear();
    }

    if update.technical_client_id.is_some() || update.technical_secret_key.is_some() {
        let mut technical = next.technical_auth.unwrap_or(TechnicalAuthConfig {
            client_id: String::new(),
            secret_key: String::new(),
        });

        if let Some(client_id) = &update.technical_client_id {
            technical.client_id = client_id.clone();
        }
        if let Some(secret_key) = &update.technical_secret_key {
            technical.secret_key = secret_key.clone();
        }
        next.technical_auth = Some(technical);
    }

    if let Some(max_parallel_jobs) = update.max_parallel_jobs {
        next.max_parallel_jobs = max_parallel_jobs;
    }
    if let Some(storage_mounts) = &update.storage_mounts {
        next.storage_mounts = normalize_storage_mounts(storage_mounts);
    }
    if let Some(log_level) = update.log_level {
        next.log_level = log_level;
    }

    let _ = interface;
    validate_config(&next)?;
    Ok(next)
}

pub fn normalize_storage_mount_path(value: &str) -> String {
    let mut normalized = value.trim().to_string();
    while normalized.len() > 1 && (normalized.ends_with('/') || normalized.ends_with('\\')) {
        normalized.pop();
    }
    normalized
}

pub fn normalize_storage_mounts(value: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    value
        .iter()
        .map(|(storage_id, path)| {
            (
                storage_id.trim().to_string(),
                normalize_storage_mount_path(path),
            )
        })
        .collect()
}

pub fn resolve_source_path(
    config: &AgentRuntimeConfig,
    storage_id: &str,
    relative_path: &str,
) -> Result<PathBuf, SourcePathResolveError> {
    let base = config
        .storage_mounts
        .get(storage_id)
        .ok_or_else(|| SourcePathResolveError::UnknownStorageId(storage_id.to_string()))?;
    let sanitized_relative = sanitize_relative_path(relative_path)?;
    Ok(Path::new(base).join(sanitized_relative))
}

fn sanitize_relative_path(value: &str) -> Result<PathBuf, SourcePathResolveError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err(SourcePathResolveError::UnsafeRelativePath(
            value.to_string(),
        ));
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(SourcePathResolveError::UnsafeRelativePath(
            value.to_string(),
        ));
    }

    let mut sanitized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => sanitized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(SourcePathResolveError::UnsafeRelativePath(
                    value.to_string(),
                ));
            }
        }
    }

    if sanitized.as_os_str().is_empty() {
        return Err(SourcePathResolveError::UnsafeRelativePath(
            value.to_string(),
        ));
    }
    Ok(sanitized)
}

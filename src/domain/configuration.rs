use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use serde::Deserialize;
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
    #[error("storage marker missing: {0}")]
    StorageMarkerMissing(String),
    #[error("storage marker invalid: {0}")]
    StorageMarkerInvalid(String),
    #[error("storage marker storage_id mismatch (expected={expected} actual={actual})")]
    StorageMarkerStorageIdMismatch { expected: String, actual: String },
    #[error("unsafe relative path: {0}")]
    UnsafeRelativePath(String),
    #[error("path outside marker-declared roots: {0}")]
    PathOutsideMarkerRoots(String),
}

const STORAGE_MARKER_FILENAME: &str = ".retaia";

#[derive(Debug, Deserialize)]
struct StorageMarker {
    version: u64,
    storage_id: String,
    paths: StorageMarkerPaths,
}

#[derive(Debug, Deserialize)]
struct StorageMarkerPaths {
    inbox: String,
    archive: String,
    rejects: String,
}

#[derive(Debug, Clone)]
struct ValidatedStorageMarkerPaths {
    version: u64,
    inbox: PathBuf,
    archive: PathBuf,
    rejects: PathBuf,
}

#[derive(Debug, Clone)]
struct CachedStorageMarker {
    modified_at: SystemTime,
    paths: ValidatedStorageMarkerPaths,
}

static STORAGE_MARKER_CACHE: OnceLock<Mutex<BTreeMap<String, CachedStorageMarker>>> =
    OnceLock::new();

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
    let marker_paths = load_and_validate_storage_marker(Path::new(base), storage_id)?;
    let sanitized_relative = sanitize_relative_path(relative_path)?;
    ensure_path_within_marker_roots(&sanitized_relative, &marker_paths, relative_path)?;
    Ok(Path::new(base).join(sanitized_relative))
}

fn load_and_validate_storage_marker(
    mount_root: &Path,
    expected_storage_id: &str,
) -> Result<ValidatedStorageMarkerPaths, SourcePathResolveError> {
    let marker_path = mount_root.join(STORAGE_MARKER_FILENAME);
    let metadata = std::fs::metadata(&marker_path).map_err(|_| {
        SourcePathResolveError::StorageMarkerMissing(marker_path.display().to_string())
    })?;
    let modified_at = metadata.modified().map_err(|error| {
        SourcePathResolveError::StorageMarkerInvalid(format!(
            "{} (unable to read mtime: {error})",
            marker_path.display()
        ))
    })?;
    let cache_key = format!("{}::{expected_storage_id}", mount_root.display());
    if let Some(cached) = cached_storage_marker(&cache_key, modified_at) {
        return Ok(cached);
    }

    let raw = std::fs::read_to_string(&marker_path).map_err(|error| {
        SourcePathResolveError::StorageMarkerInvalid(format!(
            "{} (unable to read marker: {error})",
            marker_path.display()
        ))
    })?;
    let marker: StorageMarker = serde_json::from_str(&raw).map_err(|error| {
        SourcePathResolveError::StorageMarkerInvalid(format!(
            "{} ({error})",
            marker_path.display()
        ))
    })?;

    if marker.version == 0 {
        return Err(SourcePathResolveError::StorageMarkerInvalid(format!(
            "{} (version must be > 0)",
            marker_path.display()
        )));
    }
    if marker.storage_id.trim().is_empty() {
        return Err(SourcePathResolveError::StorageMarkerInvalid(format!(
            "{} (storage_id must not be empty)",
            marker_path.display()
        )));
    }
    if marker.storage_id != expected_storage_id {
        return Err(SourcePathResolveError::StorageMarkerStorageIdMismatch {
            expected: expected_storage_id.to_string(),
            actual: marker.storage_id,
        });
    }

    let inbox = sanitize_relative_path(&marker.paths.inbox).map_err(|_| {
        SourcePathResolveError::StorageMarkerInvalid(format!(
            "{} (paths.inbox is invalid)",
            marker_path.display()
        ))
    })?;
    let archive = sanitize_relative_path(&marker.paths.archive).map_err(|_| {
        SourcePathResolveError::StorageMarkerInvalid(format!(
            "{} (paths.archive is invalid)",
            marker_path.display()
        ))
    })?;
    let rejects = sanitize_relative_path(&marker.paths.rejects).map_err(|_| {
        SourcePathResolveError::StorageMarkerInvalid(format!(
            "{} (paths.rejects is invalid)",
            marker_path.display()
        ))
    })?;

    let validated = ValidatedStorageMarkerPaths {
        version: marker.version,
        inbox,
        archive,
        rejects,
    };
    cache_storage_marker(&cache_key, modified_at, &validated);
    Ok(validated)
}

fn ensure_path_within_marker_roots(
    sanitized_relative: &Path,
    marker_paths: &ValidatedStorageMarkerPaths,
    original_relative: &str,
) -> Result<(), SourcePathResolveError> {
    if marker_paths.version == 1 {
        if sanitized_relative.starts_with(&marker_paths.inbox) {
            return Ok(());
        }
    } else if sanitized_relative.starts_with(&marker_paths.inbox)
        || sanitized_relative.starts_with(&marker_paths.archive)
        || sanitized_relative.starts_with(&marker_paths.rejects)
    {
        return Ok(());
    }
    Err(SourcePathResolveError::PathOutsideMarkerRoots(
        original_relative.to_string(),
    ))
}

fn cached_storage_marker(cache_key: &str, modified_at: SystemTime) -> Option<ValidatedStorageMarkerPaths> {
    let cache = STORAGE_MARKER_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let guard = cache.lock().ok()?;
    let cached = guard.get(cache_key)?;
    if cached.modified_at != modified_at {
        return None;
    }
    Some(cached.paths.clone())
}

fn cache_storage_marker(cache_key: &str, modified_at: SystemTime, paths: &ValidatedStorageMarkerPaths) {
    let cache = STORAGE_MARKER_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Ok(mut guard) = cache.lock() {
        guard.insert(
            cache_key.to_string(),
            CachedStorageMarker {
                modified_at,
                paths: paths.clone(),
            },
        );
    }
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

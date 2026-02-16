use std::collections::BTreeSet;

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
    pub max_parallel_jobs: u16,
    pub log_level: LogLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValidationError {
    InvalidCoreApiUrl,
    InvalidOllamaUrl,
    MissingTechnicalAuth,
    EmptyClientId,
    EmptySecretKey,
    InvalidMaxParallelJobs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigField {
    CoreApiUrl,
    OllamaUrl,
    AuthMode,
    TechnicalClientId,
    TechnicalSecretKey,
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
    pub max_parallel_jobs: Option<u16>,
    pub log_level: Option<LogLevel>,
}

fn is_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
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
    }
    if !is_http_url(&config.ollama_url) {
        errors.push(ConfigValidationError::InvalidOllamaUrl);
    }

    if config.max_parallel_jobs == 0 {
        errors.push(ConfigValidationError::InvalidMaxParallelJobs);
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
            ConfigValidationError::InvalidOllamaUrl => "invalid ollama url",
            ConfigValidationError::MissingTechnicalAuth => "missing technical auth",
            ConfigValidationError::EmptyClientId => "empty client id",
            ConfigValidationError::EmptySecretKey => "empty secret key",
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
    if let Some(log_level) = update.log_level {
        next.log_level = log_level;
    }

    let _ = interface;
    validate_config(&next)?;
    Ok(next)
}

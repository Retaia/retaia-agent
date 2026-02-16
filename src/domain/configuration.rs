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

fn is_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
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

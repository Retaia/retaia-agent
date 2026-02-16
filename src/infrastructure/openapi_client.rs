use retaia_core_client::apis::configuration::Configuration;

use crate::domain::configuration::AgentRuntimeConfig;

pub fn build_core_api_client(config: &AgentRuntimeConfig) -> Configuration {
    let mut client = Configuration::new();
    client.base_path = config.core_api_url.clone();
    client
}

pub fn with_bearer_token(mut client: Configuration, token: impl Into<String>) -> Configuration {
    let token = token.into();
    client.bearer_access_token = Some(token.clone());
    // Generated jobs/auth APIs use oauth_access_token for bearer auth.
    client.oauth_access_token = Some(token);
    client
}

#[cfg(test)]
mod tests {
    use super::{build_core_api_client, with_bearer_token};
    use crate::domain::configuration::{AgentRuntimeConfig, AuthMode, LogLevel};

    fn runtime_config() -> AgentRuntimeConfig {
        AgentRuntimeConfig {
            core_api_url: "https://core.retaia.local/api/v1".to_string(),
            ollama_url: "http://127.0.0.1:11434".to_string(),
            max_parallel_jobs: 2,
            log_level: LogLevel::Info,
            auth_mode: AuthMode::Interactive,
            technical_auth: None,
        }
    }

    #[test]
    fn tdd_openapi_client_uses_runtime_core_api_url_as_base_path() {
        let client = build_core_api_client(&runtime_config());
        assert_eq!(client.base_path, "https://core.retaia.local/api/v1");
    }

    #[test]
    fn tdd_openapi_client_can_attach_bearer_token() {
        let client = build_core_api_client(&runtime_config());
        let client = with_bearer_token(client, "token-abc");
        assert_eq!(client.bearer_access_token.as_deref(), Some("token-abc"));
    }
}

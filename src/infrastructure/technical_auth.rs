use std::sync::Arc;

use thiserror::Error;

use crate::TechnicalAuthConfig;

#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::Error as OpenApiError;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::auth_api::{AuthApi, AuthApiClient, AuthClientsTokenPostError};
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::configuration::Configuration;
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::{AuthClientTokenRequest, NonUiClientKind};

#[derive(Debug, Error)]
pub enum TechnicalAuthError {
    #[error("technical auth unavailable")]
    MissingTechnicalAuth,
    #[error("technical auth unauthorized")]
    Unauthorized,
    #[error("technical auth unexpected status {0}")]
    UnexpectedStatus(u16),
    #[error("technical auth transport error: {0}")]
    Transport(String),
}

#[cfg(feature = "core-api-client")]
pub fn mint_technical_bearer(
    configuration: &Configuration,
    technical_auth: &TechnicalAuthConfig,
) -> Result<String, TechnicalAuthError> {
    let api = AuthApiClient::new(Arc::new(configuration.clone()));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| TechnicalAuthError::Transport(error.to_string()))?;
    let request = AuthClientTokenRequest::new(
        technical_auth.client_id.clone(),
        NonUiClientKind::Agent,
        technical_auth.secret_key.clone(),
    );
    let response = runtime
        .block_on(api.auth_clients_token_post(request, None))
        .map_err(map_openapi_auth_error)?;
    Ok(response.access_token)
}

#[cfg(feature = "core-api-client")]
fn map_openapi_auth_error(error: OpenApiError<AuthClientsTokenPostError>) -> TechnicalAuthError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => TechnicalAuthError::Unauthorized,
            code => TechnicalAuthError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => TechnicalAuthError::Transport(err.to_string()),
        OpenApiError::Serde(err) => TechnicalAuthError::Transport(err.to_string()),
        OpenApiError::Io(err) => TechnicalAuthError::Transport(err.to_string()),
    }
}

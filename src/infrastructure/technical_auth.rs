use std::sync::Arc;

use thiserror::Error;

use crate::TechnicalAuthConfig;

#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::Error as OpenApiError;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::auth_api::{
    AuthApi, AuthApiClient, AuthClientsClientIdRotateSecretPostError,
    AuthClientsDeviceCancelPostError, AuthClientsDevicePollPostError,
    AuthClientsDeviceStartPostError, AuthClientsTokenPostError,
};
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::configuration::Configuration;
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::{
    AuthClientTokenRequest, AuthDeviceCancelRequest, AuthDevicePollRequest, AuthDevicePollResponse,
    AuthDeviceStartRequest, NonUiClientKind,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceBootstrapStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in_seconds: u64,
    pub interval_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceBootstrapPollStatus {
    Pending {
        interval_seconds: Option<u64>,
    },
    Approved {
        client_id: String,
        secret_key: String,
    },
    Denied,
    Expired,
}

#[derive(Debug, Error)]
pub enum DeviceBootstrapError {
    #[error("device bootstrap unauthorized")]
    Unauthorized,
    #[error("device bootstrap unexpected status {0}")]
    UnexpectedStatus(u16),
    #[error("device bootstrap transport error: {0}")]
    Transport(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotateSecretResult {
    pub client_id: String,
    pub secret_key: String,
    pub rotated_at: Option<String>,
}

#[derive(Debug, Error)]
pub enum RotateSecretError {
    #[error("rotate secret unauthorized")]
    Unauthorized,
    #[error("rotate secret unexpected status {0}")]
    UnexpectedStatus(u16),
    #[error("rotate secret transport error: {0}")]
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
pub fn start_device_bootstrap(
    configuration: &Configuration,
    client_label: Option<String>,
) -> Result<DeviceBootstrapStart, DeviceBootstrapError> {
    let api = AuthApiClient::new(Arc::new(configuration.clone()));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| DeviceBootstrapError::Transport(error.to_string()))?;
    let mut request = AuthDeviceStartRequest::new(NonUiClientKind::Agent);
    request.client_label = client_label;
    let response = runtime
        .block_on(api.auth_clients_device_start_post(request, None))
        .map_err(map_openapi_device_start_error)?;

    Ok(DeviceBootstrapStart {
        device_code: response.device_code,
        user_code: response.user_code,
        verification_uri: response.verification_uri,
        verification_uri_complete: response.verification_uri_complete,
        expires_in_seconds: u64::try_from(response.expires_in).unwrap_or_default(),
        interval_seconds: u64::try_from(response.interval).unwrap_or(5).max(1),
    })
}

#[cfg(feature = "core-api-client")]
pub fn poll_device_bootstrap(
    configuration: &Configuration,
    device_code: &str,
) -> Result<DeviceBootstrapPollStatus, DeviceBootstrapError> {
    let api = AuthApiClient::new(Arc::new(configuration.clone()));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| DeviceBootstrapError::Transport(error.to_string()))?;
    let response = runtime
        .block_on(api.auth_clients_device_poll_post(
            AuthDevicePollRequest::new(device_code.to_string()),
            None,
        ))
        .map_err(map_openapi_device_poll_error)?;

    match response {
        AuthDevicePollResponse::AuthDevicePollPending(pending) => {
            Ok(DeviceBootstrapPollStatus::Pending {
                interval_seconds: pending.interval.and_then(|value| u64::try_from(value).ok()),
            })
        }
        AuthDevicePollResponse::AuthDevicePollApproved(approved) => {
            Ok(DeviceBootstrapPollStatus::Approved {
                client_id: approved.client_id.clone(),
                secret_key: approved.secret_key.clone(),
            })
        }
        AuthDevicePollResponse::AuthDevicePollDenied(_) => Ok(DeviceBootstrapPollStatus::Denied),
        AuthDevicePollResponse::AuthDevicePollExpired(_) => Ok(DeviceBootstrapPollStatus::Expired),
    }
}

#[cfg(feature = "core-api-client")]
pub fn cancel_device_bootstrap(
    configuration: &Configuration,
    device_code: &str,
) -> Result<bool, DeviceBootstrapError> {
    let api = AuthApiClient::new(Arc::new(configuration.clone()));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| DeviceBootstrapError::Transport(error.to_string()))?;
    let response = runtime
        .block_on(api.auth_clients_device_cancel_post(
            AuthDeviceCancelRequest::new(device_code.to_string()),
            None,
        ))
        .map_err(map_openapi_device_cancel_error)?;
    Ok(response.canceled)
}

#[cfg(feature = "core-api-client")]
pub fn rotate_client_secret(
    configuration: &Configuration,
    client_id: &str,
) -> Result<RotateSecretResult, RotateSecretError> {
    let api = AuthApiClient::new(Arc::new(configuration.clone()));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| RotateSecretError::Transport(error.to_string()))?;
    let response = runtime
        .block_on(api.auth_clients_client_id_rotate_secret_post(client_id, None))
        .map_err(map_openapi_rotate_secret_error)?;
    Ok(RotateSecretResult {
        client_id: response.client_id,
        secret_key: response.secret_key,
        rotated_at: response.rotated_at,
    })
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

#[cfg(feature = "core-api-client")]
fn map_openapi_device_start_error(
    error: OpenApiError<AuthClientsDeviceStartPostError>,
) -> DeviceBootstrapError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => DeviceBootstrapError::Unauthorized,
            code => DeviceBootstrapError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => DeviceBootstrapError::Transport(err.to_string()),
        OpenApiError::Serde(err) => DeviceBootstrapError::Transport(err.to_string()),
        OpenApiError::Io(err) => DeviceBootstrapError::Transport(err.to_string()),
    }
}

#[cfg(feature = "core-api-client")]
fn map_openapi_device_poll_error(
    error: OpenApiError<AuthClientsDevicePollPostError>,
) -> DeviceBootstrapError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => DeviceBootstrapError::Unauthorized,
            code => DeviceBootstrapError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => DeviceBootstrapError::Transport(err.to_string()),
        OpenApiError::Serde(err) => DeviceBootstrapError::Transport(err.to_string()),
        OpenApiError::Io(err) => DeviceBootstrapError::Transport(err.to_string()),
    }
}

#[cfg(feature = "core-api-client")]
fn map_openapi_device_cancel_error(
    error: OpenApiError<AuthClientsDeviceCancelPostError>,
) -> DeviceBootstrapError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => DeviceBootstrapError::Unauthorized,
            code => DeviceBootstrapError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => DeviceBootstrapError::Transport(err.to_string()),
        OpenApiError::Serde(err) => DeviceBootstrapError::Transport(err.to_string()),
        OpenApiError::Io(err) => DeviceBootstrapError::Transport(err.to_string()),
    }
}

#[cfg(feature = "core-api-client")]
fn map_openapi_rotate_secret_error(
    error: OpenApiError<AuthClientsClientIdRotateSecretPostError>,
) -> RotateSecretError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => RotateSecretError::Unauthorized,
            code => RotateSecretError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => RotateSecretError::Transport(err.to_string()),
        OpenApiError::Serde(err) => RotateSecretError::Transport(err.to_string()),
        OpenApiError::Io(err) => RotateSecretError::Transport(err.to_string()),
    }
}

#[cfg(feature = "core-api-client")]
use chrono::{DateTime, Utc};
#[cfg(feature = "core-api-client")]
use reqwest::header::{
    ACCEPT_LANGUAGE, AUTHORIZATION, CONTENT_TYPE, HeaderMap, RETRY_AFTER, USER_AGENT,
};
#[cfg(feature = "core-api-client")]
use serde::de::DeserializeOwned;

#[cfg(feature = "core-api-client")]
use crate::Language;
#[cfg(feature = "core-api-client")]
use crate::application::core_api_gateway::{
    CoreApiGateway, CoreApiGatewayError, CoreJobState, CoreJobView, CoreServerPolicy,
};
#[cfg(feature = "core-api-client")]
use crate::detect_language;

#[cfg(all(test, feature = "core-api-client"))]
use retaia_core_client::apis::Error as OpenApiError;
#[cfg(all(test, feature = "core-api-client"))]
use retaia_core_client::apis::auth_api::AppPolicyGetError;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::configuration::Configuration;
#[cfg(all(test, feature = "core-api-client"))]
use retaia_core_client::apis::jobs_api::JobsGetError;
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::job::Status;
#[cfg(feature = "core-api-client")]
use retaia_core_client::models::{AppPolicyResponse, Job};

#[cfg(feature = "core-api-client")]
#[derive(Debug, Clone)]
pub struct OpenApiJobsGateway {
    configuration: Configuration,
}

#[cfg(feature = "core-api-client")]
impl OpenApiJobsGateway {
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }
}

#[cfg(feature = "core-api-client")]
impl CoreApiGateway for OpenApiJobsGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError> {
        let jobs: Vec<Job> = self.get_json("/jobs")?;

        Ok(jobs
            .into_iter()
            .map(|job| CoreJobView {
                job_id: job.job_id,
                asset_uuid: job.asset_uuid,
                state: map_job_state(job.status),
                required_capabilities: job.required_capabilities,
            })
            .collect())
    }

    fn fetch_server_policy(&self) -> Result<CoreServerPolicy, CoreApiGatewayError> {
        let response: AppPolicyResponse = self.get_json("/app/policy")?;

        Ok(CoreServerPolicy {
            min_poll_interval_seconds: response
                .server_policy
                .min_poll_interval_seconds
                .and_then(|value| u64::try_from(value).ok()),
            feature_flags: response.server_policy.feature_flags.into_iter().collect(),
        })
    }
}

#[cfg(feature = "core-api-client")]
impl OpenApiJobsGateway {
    fn get_json<T: DeserializeOwned>(&self, relative_path: &str) -> Result<T, CoreApiGatewayError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| CoreApiGatewayError::Transport(error.to_string()))?;

        runtime.block_on(async {
            let url = format!("{}{}", self.configuration.base_path, relative_path);
            let mut request = self.configuration.client.get(url);

            if let Some(user_agent) = self.configuration.user_agent.as_deref() {
                request = request.header(USER_AGENT, user_agent);
            }
            request = request.header(ACCEPT_LANGUAGE, accept_language_header_value());
            if let Some(token) = self.configuration.oauth_access_token.as_deref() {
                request = request.header(AUTHORIZATION, format!("Bearer {token}"));
            } else if let Some(token) = self.configuration.bearer_access_token.as_deref() {
                request = request.header(AUTHORIZATION, format!("Bearer {token}"));
            }

            let response = request
                .send()
                .await
                .map_err(|error| CoreApiGatewayError::Transport(error.to_string()))?;

            let status = response.status();
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(CoreApiGatewayError::Unauthorized);
            }
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(CoreApiGatewayError::Throttled {
                    retry_after_ms: parse_retry_after_ms(response.headers()),
                });
            }
            if !status.is_success() {
                return Err(CoreApiGatewayError::UnexpectedStatus(status.as_u16()));
            }

            let content_type = response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_string();
            if !content_type.starts_with("application/json") {
                return Err(CoreApiGatewayError::Transport(format!(
                    "unexpected content type for {relative_path}: {content_type}"
                )));
            }

            response
                .json::<T>()
                .await
                .map_err(|error| CoreApiGatewayError::Transport(error.to_string()))
        })
    }
}

#[cfg(feature = "core-api-client")]
fn parse_retry_after_ms(headers: &HeaderMap) -> Option<u64> {
    let raw = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();
    if let Ok(seconds) = raw.parse::<u64>() {
        return Some(seconds.saturating_mul(1_000));
    }

    let retry_at = DateTime::parse_from_rfc2822(raw).ok()?.with_timezone(&Utc);
    let now = Utc::now();
    let millis = retry_at
        .signed_duration_since(now)
        .num_milliseconds()
        .max(0) as u64;
    Some(millis)
}

#[cfg(feature = "core-api-client")]
fn accept_language_header_value() -> &'static str {
    match detect_language() {
        Language::Fr => "fr",
        Language::En => "en",
    }
}

#[cfg(feature = "core-api-client")]
fn map_job_state(status: Status) -> CoreJobState {
    match status {
        Status::Pending => CoreJobState::Pending,
        Status::Claimed => CoreJobState::Claimed,
        Status::Completed => CoreJobState::Completed,
        Status::Failed => CoreJobState::Failed,
    }
}

#[cfg(all(test, feature = "core-api-client"))]
fn map_openapi_jobs_error(error: OpenApiError<JobsGetError>) -> CoreApiGatewayError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => CoreApiGatewayError::Unauthorized,
            429 => CoreApiGatewayError::Throttled {
                retry_after_ms: None,
            },
            code => CoreApiGatewayError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => CoreApiGatewayError::Transport(err.to_string()),
        OpenApiError::Serde(err) => CoreApiGatewayError::Transport(err.to_string()),
        OpenApiError::Io(err) => CoreApiGatewayError::Transport(err.to_string()),
    }
}

#[cfg(all(test, feature = "core-api-client"))]
fn map_openapi_policy_error(error: OpenApiError<AppPolicyGetError>) -> CoreApiGatewayError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => CoreApiGatewayError::Unauthorized,
            429 => CoreApiGatewayError::Throttled {
                retry_after_ms: None,
            },
            code => CoreApiGatewayError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => CoreApiGatewayError::Transport(err.to_string()),
        OpenApiError::Serde(err) => CoreApiGatewayError::Transport(err.to_string()),
        OpenApiError::Io(err) => CoreApiGatewayError::Transport(err.to_string()),
    }
}

#[cfg(all(test, feature = "core-api-client"))]
mod tests {
    use super::{map_openapi_jobs_error, map_openapi_policy_error, parse_retry_after_ms};
    use crate::application::core_api_gateway::CoreApiGatewayError;
    use chrono::{Duration, Utc};
    use reqwest::StatusCode;
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
    use retaia_core_client::apis::Error as OpenApiError;
    use retaia_core_client::apis::ResponseContent;
    use retaia_core_client::apis::auth_api::AppPolicyGetError;
    use retaia_core_client::apis::jobs_api::JobsGetError;

    fn response_error(status: u16) -> OpenApiError<JobsGetError> {
        OpenApiError::ResponseError(ResponseContent {
            status: StatusCode::from_u16(status).expect("valid status"),
            content: String::new(),
            entity: None,
        })
    }

    #[test]
    fn tdd_openapi_jobs_gateway_maps_expected_http_statuses() {
        assert_eq!(
            map_openapi_jobs_error(response_error(401)),
            CoreApiGatewayError::Unauthorized
        );
        assert_eq!(
            map_openapi_jobs_error(response_error(429)),
            CoreApiGatewayError::Throttled {
                retry_after_ms: None,
            }
        );
        assert_eq!(
            map_openapi_jobs_error(response_error(422)),
            CoreApiGatewayError::UnexpectedStatus(422)
        );
        assert_eq!(
            map_openapi_jobs_error(response_error(500)),
            CoreApiGatewayError::UnexpectedStatus(500)
        );
    }

    #[test]
    fn tdd_openapi_jobs_gateway_maps_transport_errors() {
        let error = map_openapi_jobs_error(OpenApiError::Io(std::io::Error::other("network down")));
        match error {
            CoreApiGatewayError::Transport(message) => assert!(message.contains("network down")),
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    fn policy_response_error(status: u16) -> OpenApiError<AppPolicyGetError> {
        OpenApiError::ResponseError(ResponseContent {
            status: StatusCode::from_u16(status).expect("valid status"),
            content: String::new(),
            entity: None,
        })
    }

    #[test]
    fn tdd_openapi_policy_gateway_maps_expected_http_statuses() {
        assert_eq!(
            map_openapi_policy_error(policy_response_error(401)),
            CoreApiGatewayError::Unauthorized
        );
        assert_eq!(
            map_openapi_policy_error(policy_response_error(429)),
            CoreApiGatewayError::Throttled {
                retry_after_ms: None,
            }
        );
        assert_eq!(
            map_openapi_policy_error(policy_response_error(503)),
            CoreApiGatewayError::UnexpectedStatus(503)
        );
    }

    #[test]
    fn tdd_openapi_jobs_gateway_parses_retry_after_seconds_header() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("12"));

        assert_eq!(parse_retry_after_ms(&headers), Some(12_000));
    }

    #[test]
    fn tdd_openapi_jobs_gateway_parses_retry_after_http_date_header() {
        let mut headers = HeaderMap::new();
        let retry_at = (Utc::now() + Duration::seconds(5)).to_rfc2822();
        headers.insert(
            RETRY_AFTER,
            HeaderValue::from_str(&retry_at).expect("header"),
        );

        let parsed = parse_retry_after_ms(&headers).expect("retry-after parsed");
        assert!(parsed <= 5_000);
    }
}

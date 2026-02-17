#[cfg(feature = "core-api-client")]
use std::sync::Arc;

#[cfg(feature = "core-api-client")]
use crate::application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedJobType, DerivedManifestItem, DerivedProcessingError,
    DerivedProcessingGateway, DerivedUploadComplete, DerivedUploadInit, DerivedUploadPart,
    HeartbeatReceipt, SubmitDerivedPayload, validate_derived_upload_init,
};

#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::Error as OpenApiError;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::configuration::Configuration;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::derived_api::{
    AssetsUuidDerivedUploadCompletePostError, AssetsUuidDerivedUploadInitPostError,
    AssetsUuidDerivedUploadPartPostError, DerivedApi, DerivedApiClient,
};
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::jobs_api::{
    JobsApi, JobsApiClient, JobsJobIdClaimPostError, JobsJobIdHeartbeatPostError,
    JobsJobIdSubmitPostError,
};
#[cfg(feature = "core-api-client")]
use retaia_core_client::models;

#[cfg(feature = "core-api-client")]
#[derive(Debug, Clone)]
pub struct OpenApiDerivedProcessingGateway {
    configuration: Configuration,
}

#[cfg(feature = "core-api-client")]
impl OpenApiDerivedProcessingGateway {
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }
}

#[cfg(feature = "core-api-client")]
impl DerivedProcessingGateway for OpenApiDerivedProcessingGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        let api = JobsApiClient::new(Arc::new(self.configuration.clone()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let job = runtime
            .block_on(api.jobs_job_id_claim_post(job_id))
            .map_err(map_claim_error)?;

        let lock_token = job
            .lock_token
            .ok_or(DerivedProcessingError::MissingLockToken)?;
        let job_type = map_job_type(job.job_type)?;

        Ok(ClaimedDerivedJob {
            job_id: job.job_id,
            asset_uuid: job.asset_uuid,
            lock_token,
            job_type,
        })
    }

    fn heartbeat(
        &self,
        job_id: &str,
        lock_token: &str,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        let api = JobsApiClient::new(Arc::new(self.configuration.clone()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let request = models::JobsJobIdHeartbeatPostRequest::new(lock_token.to_string());
        let response = runtime
            .block_on(api.jobs_job_id_heartbeat_post(job_id, request))
            .map_err(map_heartbeat_error)?;

        Ok(HeartbeatReceipt {
            locked_until: response.locked_until,
        })
    }

    fn submit_derived(
        &self,
        job_id: &str,
        lock_token: &str,
        idempotency_key: &str,
        payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        let api = JobsApiClient::new(Arc::new(self.configuration.clone()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let derived_patch = build_derived_patch(&payload.manifest)?;
        let mut result = models::SubmitDerivedResult::new(derived_patch);
        result.warnings = payload.warnings.clone();
        result.metrics = payload.metrics.clone();

        let submit = models::SubmitDerived::new(
            lock_token.to_string(),
            map_submit_job_type(payload.job_type),
            result,
        );

        runtime
            .block_on(api.jobs_job_id_submit_post(
                job_id,
                idempotency_key,
                models::JobSubmitRequest::SubmitDerived(Box::new(submit)),
            ))
            .map_err(map_submit_error)
    }

    fn upload_init(&self, request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        validate_derived_upload_init(request)?;

        let api = DerivedApiClient::new(Arc::new(self.configuration.clone()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let size_bytes = i32::try_from(request.size_bytes).map_err(|_| {
            DerivedProcessingError::NumericOverflow("size_bytes > i32::MAX".to_string())
        })?;

        let mut payload = models::AssetsUuidDerivedUploadInitPostRequest::new(
            map_upload_kind(request.kind),
            request.content_type.clone(),
            size_bytes,
        );
        payload.sha256 = request.sha256.clone();

        runtime
            .block_on(api.assets_uuid_derived_upload_init_post(
                &request.asset_uuid,
                &request.idempotency_key,
                payload,
            ))
            .map_err(map_upload_init_error)
    }

    fn upload_part(&self, request: &DerivedUploadPart) -> Result<(), DerivedProcessingError> {
        let api = DerivedApiClient::new(Arc::new(self.configuration.clone()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let part_number = i32::try_from(request.part_number).map_err(|_| {
            DerivedProcessingError::NumericOverflow("part_number > i32::MAX".to_string())
        })?;

        let payload = models::AssetsUuidDerivedUploadPartPostRequest::new(
            request.upload_id.clone(),
            part_number,
        );

        runtime
            .block_on(api.assets_uuid_derived_upload_part_post(&request.asset_uuid, payload))
            .map_err(map_upload_part_error)
    }

    fn upload_complete(
        &self,
        request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        let api = DerivedApiClient::new(Arc::new(self.configuration.clone()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let mut payload =
            models::AssetsUuidDerivedUploadCompletePostRequest::new(request.upload_id.clone());
        payload.parts = request.parts.clone();

        runtime
            .block_on(api.assets_uuid_derived_upload_complete_post(
                &request.asset_uuid,
                &request.idempotency_key,
                payload,
            ))
            .map_err(map_upload_complete_error)
    }
}

#[cfg(feature = "core-api-client")]
fn map_job_type(job_type: models::job::JobType) -> Result<DerivedJobType, DerivedProcessingError> {
    match job_type {
        models::job::JobType::GenerateProxy => Ok(DerivedJobType::GenerateProxy),
        models::job::JobType::GenerateThumbnails => Ok(DerivedJobType::GenerateThumbnails),
        models::job::JobType::GenerateAudioWaveform => Ok(DerivedJobType::GenerateAudioWaveform),
        models::job::JobType::ExtractFacts => Err(DerivedProcessingError::NotDerivedJobType(
            "extract_facts".to_string(),
        )),
    }
}

#[cfg(feature = "core-api-client")]
fn map_submit_job_type(job_type: DerivedJobType) -> models::submit_derived::JobType {
    match job_type {
        DerivedJobType::GenerateProxy => models::submit_derived::JobType::GenerateProxy,
        DerivedJobType::GenerateThumbnails => models::submit_derived::JobType::GenerateThumbnails,
        DerivedJobType::GenerateAudioWaveform => {
            models::submit_derived::JobType::GenerateAudioWaveform
        }
    }
}

#[cfg(feature = "core-api-client")]
fn map_upload_kind(
    kind: crate::application::derived_processing_gateway::DerivedKind,
) -> models::_assets__uuid__derived_upload_init_post_request::Kind {
    match kind {
        crate::application::derived_processing_gateway::DerivedKind::ProxyVideo => {
            models::_assets__uuid__derived_upload_init_post_request::Kind::ProxyVideo
        }
        crate::application::derived_processing_gateway::DerivedKind::ProxyAudio => {
            models::_assets__uuid__derived_upload_init_post_request::Kind::ProxyAudio
        }
        crate::application::derived_processing_gateway::DerivedKind::ProxyPhoto => {
            models::_assets__uuid__derived_upload_init_post_request::Kind::ProxyPhoto
        }
        crate::application::derived_processing_gateway::DerivedKind::Thumb => {
            models::_assets__uuid__derived_upload_init_post_request::Kind::Thumb
        }
        crate::application::derived_processing_gateway::DerivedKind::Waveform => {
            models::_assets__uuid__derived_upload_init_post_request::Kind::Waveform
        }
    }
}

#[cfg(feature = "core-api-client")]
fn build_derived_patch(
    manifest: &[DerivedManifestItem],
) -> Result<models::DerivedPatch, DerivedProcessingError> {
    let mut patch = models::DerivedPatch::new();
    let mut items = Vec::with_capacity(manifest.len());

    for item in manifest {
        let mut mapped = models::DerivedPatchDerivedManifestInner::new(
            match item.kind {
                crate::application::derived_processing_gateway::DerivedKind::ProxyVideo => {
                    models::derived_patch_derived_manifest_inner::Kind::ProxyVideo
                }
                crate::application::derived_processing_gateway::DerivedKind::ProxyAudio => {
                    models::derived_patch_derived_manifest_inner::Kind::ProxyAudio
                }
                crate::application::derived_processing_gateway::DerivedKind::ProxyPhoto => {
                    models::derived_patch_derived_manifest_inner::Kind::ProxyPhoto
                }
                crate::application::derived_processing_gateway::DerivedKind::Thumb => {
                    models::derived_patch_derived_manifest_inner::Kind::Thumb
                }
                crate::application::derived_processing_gateway::DerivedKind::Waveform => {
                    models::derived_patch_derived_manifest_inner::Kind::Waveform
                }
            },
            item.reference.clone(),
        );

        mapped.size_bytes = match item.size_bytes {
            Some(value) => Some(i32::try_from(value).map_err(|_| {
                DerivedProcessingError::NumericOverflow(
                    "manifest.size_bytes > i32::MAX".to_string(),
                )
            })?),
            None => None,
        };
        mapped.sha256 = item.sha256.clone();
        items.push(mapped);
    }

    patch.derived_manifest = Some(items);
    Ok(patch)
}

#[cfg(feature = "core-api-client")]
fn map_claim_error(error: OpenApiError<JobsJobIdClaimPostError>) -> DerivedProcessingError {
    map_status_error(error)
}

#[cfg(feature = "core-api-client")]
fn map_heartbeat_error(error: OpenApiError<JobsJobIdHeartbeatPostError>) -> DerivedProcessingError {
    map_status_error(error)
}

#[cfg(feature = "core-api-client")]
fn map_submit_error(error: OpenApiError<JobsJobIdSubmitPostError>) -> DerivedProcessingError {
    map_status_error(error)
}

#[cfg(feature = "core-api-client")]
fn map_upload_init_error(
    error: OpenApiError<AssetsUuidDerivedUploadInitPostError>,
) -> DerivedProcessingError {
    map_status_error(error)
}

#[cfg(feature = "core-api-client")]
fn map_upload_part_error(
    error: OpenApiError<AssetsUuidDerivedUploadPartPostError>,
) -> DerivedProcessingError {
    map_status_error(error)
}

#[cfg(feature = "core-api-client")]
fn map_upload_complete_error(
    error: OpenApiError<AssetsUuidDerivedUploadCompletePostError>,
) -> DerivedProcessingError {
    map_status_error(error)
}

#[cfg(feature = "core-api-client")]
fn map_status_error<T>(error: OpenApiError<T>) -> DerivedProcessingError {
    match error {
        OpenApiError::ResponseError(response) => match response.status.as_u16() {
            401 => DerivedProcessingError::Unauthorized,
            429 => DerivedProcessingError::Throttled,
            code => DerivedProcessingError::UnexpectedStatus(code),
        },
        OpenApiError::Reqwest(err) => DerivedProcessingError::Transport(err.to_string()),
        OpenApiError::Serde(err) => DerivedProcessingError::Transport(err.to_string()),
        OpenApiError::Io(err) => DerivedProcessingError::Transport(err.to_string()),
    }
}

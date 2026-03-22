#[cfg(feature = "core-api-client")]
use std::fs;

#[cfg(feature = "core-api-client")]
use crate::application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedJobType, DerivedManifestItem, DerivedProcessingError,
    DerivedProcessingGateway, DerivedUploadComplete, DerivedUploadInit, DerivedUploadPart,
    HeartbeatReceipt, SubmitDerivedPayload, UploadedDerivedPart, validate_derived_upload_init,
};
#[cfg(feature = "core-api-client")]
use crate::infrastructure::agent_identity::AgentIdentity;
#[cfg(feature = "core-api-client")]
use crate::infrastructure::signed_core_http::{
    json_bytes, multipart_part_request, signed_empty_request, signed_json_request,
};

#[cfg(feature = "core-api-client")]
use reqwest::StatusCode;
#[cfg(feature = "core-api-client")]
use retaia_core_client::apis::configuration::Configuration;
#[cfg(feature = "core-api-client")]
use retaia_core_client::models;

#[cfg(feature = "core-api-client")]
#[derive(Debug, Clone)]
pub struct OpenApiDerivedProcessingGateway {
    configuration: Configuration,
    identity: AgentIdentity,
}

#[cfg(feature = "core-api-client")]
impl OpenApiDerivedProcessingGateway {
    pub fn new(configuration: Configuration) -> Self {
        let identity = AgentIdentity::load_or_create(None)
            .expect("agent identity must load for derived gateway");
        Self {
            configuration,
            identity,
        }
    }

    pub fn new_with_identity(configuration: Configuration, identity: AgentIdentity) -> Self {
        Self {
            configuration,
            identity,
        }
    }
}

#[cfg(feature = "core-api-client")]
impl DerivedProcessingGateway for OpenApiDerivedProcessingGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        let path = format!("/jobs/{job_id}/claim");
        let response = signed_empty_request(
            &reqwest::blocking::Client::new(),
            &self.identity,
            self.configuration.bearer_access_token.as_deref(),
            &self.configuration.base_path,
            reqwest::Method::POST,
            &path,
            None,
        )
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?
        .send()
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let response = require_success(response, map_claim_status)?;
        let job: models::Job = response
            .json()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let lock_token = job
            .lock_token
            .ok_or(DerivedProcessingError::MissingLockToken)?;
        let fencing_token = job
            .fencing_token
            .ok_or(DerivedProcessingError::MissingFencingToken)?;
        let job_type = map_job_type(job.job_type)?;

        Ok(ClaimedDerivedJob {
            job_id: job.job_id,
            asset_uuid: job.asset_uuid,
            lock_token,
            fencing_token,
            job_type,
            source_storage_id: job.source.storage_id,
            source_original_relative: job.source.original_relative,
            source_sidecars_relative: job.source.sidecars_relative.unwrap_or_default(),
        })
    }

    fn fetch_asset_revision_etag(
        &self,
        asset_uuid: &str,
    ) -> Result<String, DerivedProcessingError> {
        let path = format!("/assets/{asset_uuid}");
        let response = signed_empty_request(
            &reqwest::blocking::Client::new(),
            &self.identity,
            self.configuration.bearer_access_token.as_deref(),
            &self.configuration.base_path,
            reqwest::Method::GET,
            &path,
            None,
        )
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?
        .send()
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let response = require_success(response, map_asset_get_status)?;
        let etag = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                DerivedProcessingError::Transport(
                    "core API asset detail response missing ETag header".to_string(),
                )
            })?;
        Ok(etag.to_string())
    }

    fn heartbeat(
        &self,
        job_id: &str,
        lock_token: &str,
        fencing_token: i32,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        let path = format!("/jobs/{job_id}/heartbeat");
        let request =
            models::JobsJobIdHeartbeatPostRequest::new(lock_token.to_string(), fencing_token);
        let payload = json_bytes(&request)
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;
        let response = signed_json_request(
            &reqwest::blocking::Client::new(),
            &self.identity,
            self.configuration.bearer_access_token.as_deref(),
            &self.configuration.base_path,
            reqwest::Method::POST,
            &path,
            &payload,
            None,
        )
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?
        .send()
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let response = require_success(response, map_heartbeat_status)?;
        let response: models::JobsJobIdHeartbeatPost200Response = response
            .json()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;
        Ok(HeartbeatReceipt {
            locked_until: Some(response.locked_until),
            fencing_token: response.fencing_token,
        })
    }

    fn submit_derived(
        &self,
        job_id: &str,
        lock_token: &str,
        fencing_token: i32,
        idempotency_key: &str,
        payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        let path = format!("/jobs/{job_id}/submit");
        let request = if payload.job_type == DerivedJobType::ExtractFacts {
            let facts_patch = payload
                .facts_patch
                .as_ref()
                .map(map_facts_patch)
                .unwrap_or_else(models::FactsPatch::new);
            let mut result = models::SubmitExtractFactsResult::new(facts_patch);
            result.warnings = payload.warnings.clone();
            result.metrics = payload.metrics.clone();
            models::JobSubmitRequest::SubmitExtractFacts(Box::new(models::SubmitExtractFacts::new(
                lock_token.to_string(),
                fencing_token,
                models::submit_extract_facts::JobType::ExtractFacts,
                result,
            )))
        } else if payload.job_type == DerivedJobType::TranscribeAudio {
            let transcript_patch = payload
                .transcript_patch
                .as_ref()
                .map(map_transcript_patch)
                .unwrap_or_else(models::TranscriptPatch::new);
            let mut result = models::SubmitTranscriptResult::new(transcript_patch);
            result.warnings = payload.warnings.clone();
            result.metrics = payload.metrics.clone();
            models::JobSubmitRequest::SubmitTranscript(Box::new(models::SubmitTranscript::new(
                lock_token.to_string(),
                fencing_token,
                models::submit_transcript::JobType::TranscribeAudio,
                result,
            )))
        } else {
            let derived_patch = build_derived_patch(&payload.manifest)?;
            let mut result = models::SubmitDerivedResult::new(derived_patch);
            result.warnings = payload.warnings.clone();
            result.metrics = payload.metrics.clone();
            models::JobSubmitRequest::SubmitDerived(Box::new(models::SubmitDerived::new(
                lock_token.to_string(),
                fencing_token,
                map_submit_job_type(payload.job_type),
                result,
            )))
        };

        let body = json_bytes(&request)
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;
        let response = signed_json_request(
            &reqwest::blocking::Client::new(),
            &self.identity,
            self.configuration.bearer_access_token.as_deref(),
            &self.configuration.base_path,
            reqwest::Method::POST,
            &path,
            &body,
            None,
        )
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?
        .header("Idempotency-Key", idempotency_key)
        .send()
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        require_success(response, map_submit_status)?;
        Ok(())
    }

    fn upload_init(&self, request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        validate_derived_upload_init(request)?;

        let size_bytes = i32::try_from(request.size_bytes).map_err(|_| {
            DerivedProcessingError::NumericOverflow("size_bytes > i32::MAX".to_string())
        })?;
        let mut payload = models::AssetsUuidDerivedUploadInitPostRequest::new(
            map_upload_kind(request.kind),
            request.content_type.clone(),
            size_bytes,
        );
        payload.sha256 = request.sha256.clone();

        let body = json_bytes(&payload)
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;
        let path = format!("/assets/{}/derived/upload/init", request.asset_uuid);
        let response = signed_json_request(
            &reqwest::blocking::Client::new(),
            &self.identity,
            self.configuration.bearer_access_token.as_deref(),
            &self.configuration.base_path,
            reqwest::Method::POST,
            &path,
            &body,
            None,
        )
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?
        .header("If-Match", request.revision_etag.clone())
        .header("Idempotency-Key", request.idempotency_key.clone())
        .send()
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        require_success(response, map_upload_init_status)?;
        Ok(())
    }

    fn upload_part(
        &self,
        request: &DerivedUploadPart,
    ) -> Result<UploadedDerivedPart, DerivedProcessingError> {
        let path = format!("/assets/{}/derived/upload/part", request.asset_uuid);
        let chunk = fs::read(&request.chunk_path)
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;
        let response = multipart_part_request(
            &reqwest::blocking::Client::new(),
            &self.identity,
            self.configuration.bearer_access_token.as_deref(),
            &self.configuration.base_path,
            &path,
            &request.revision_etag,
            &request.upload_id,
            request.part_number,
            chunk,
            None,
        )
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?
        .send()
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        let response = require_success(response, map_upload_part_status)?;
        let response: models::AssetsUuidDerivedUploadPartPost200Response = response
            .json()
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        Ok(UploadedDerivedPart {
            part_number: request.part_number,
            part_etag: response.part_etag,
        })
    }

    fn upload_complete(
        &self,
        request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        let mut payload =
            models::AssetsUuidDerivedUploadCompletePostRequest::new(request.upload_id.clone());
        payload.parts = request.parts.as_ref().map(|parts| {
            parts
                .iter()
                .map(|part| {
                    models::AssetsUuidDerivedUploadCompletePostRequestPartsInner::new(
                        i32::try_from(part.part_number).unwrap_or(i32::MAX),
                        part.part_etag.clone(),
                    )
                })
                .collect()
        });

        let body = json_bytes(&payload)
            .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;
        let path = format!("/assets/{}/derived/upload/complete", request.asset_uuid);
        let response = signed_json_request(
            &reqwest::blocking::Client::new(),
            &self.identity,
            self.configuration.bearer_access_token.as_deref(),
            &self.configuration.base_path,
            reqwest::Method::POST,
            &path,
            &body,
            None,
        )
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?
        .header("If-Match", request.revision_etag.clone())
        .header("Idempotency-Key", request.idempotency_key.clone())
        .send()
        .map_err(|error| DerivedProcessingError::Transport(error.to_string()))?;

        require_success(response, map_upload_complete_status)?;
        Ok(())
    }
}

#[cfg(feature = "core-api-client")]
fn map_job_type(job_type: models::job::JobType) -> Result<DerivedJobType, DerivedProcessingError> {
    match job_type {
        models::job::JobType::ExtractFacts => Ok(DerivedJobType::ExtractFacts),
        models::job::JobType::GeneratePreview => Ok(DerivedJobType::GeneratePreview),
        models::job::JobType::GenerateThumbnails => Ok(DerivedJobType::GenerateThumbnails),
        models::job::JobType::GenerateAudioWaveform => Ok(DerivedJobType::GenerateAudioWaveform),
        models::job::JobType::TranscribeAudio => Ok(DerivedJobType::TranscribeAudio),
    }
}

#[cfg(feature = "core-api-client")]
fn map_submit_job_type(job_type: DerivedJobType) -> models::submit_derived::JobType {
    match job_type {
        DerivedJobType::ExtractFacts => unreachable!("extract_facts must use SubmitExtractFacts"),
        DerivedJobType::GeneratePreview => models::submit_derived::JobType::GeneratePreview,
        DerivedJobType::GenerateThumbnails => models::submit_derived::JobType::GenerateThumbnails,
        DerivedJobType::GenerateAudioWaveform => {
            models::submit_derived::JobType::GenerateAudioWaveform
        }
        DerivedJobType::TranscribeAudio => {
            unreachable!("transcribe_audio must use SubmitTranscript")
        }
    }
}

#[cfg(feature = "core-api-client")]
fn map_upload_kind(
    kind: crate::application::derived_processing_gateway::DerivedKind,
) -> models::_assets__uuid__derived_upload_init_post_request::Kind {
    match kind {
        crate::application::derived_processing_gateway::DerivedKind::PreviewVideo => {
            models::_assets__uuid__derived_upload_init_post_request::Kind::PreviewVideo
        }
        crate::application::derived_processing_gateway::DerivedKind::PreviewAudio => {
            models::_assets__uuid__derived_upload_init_post_request::Kind::PreviewAudio
        }
        crate::application::derived_processing_gateway::DerivedKind::PreviewPhoto => {
            models::_assets__uuid__derived_upload_init_post_request::Kind::PreviewPhoto
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
                crate::application::derived_processing_gateway::DerivedKind::PreviewVideo => {
                    models::derived_patch_derived_manifest_inner::Kind::PreviewVideo
                }
                crate::application::derived_processing_gateway::DerivedKind::PreviewAudio => {
                    models::derived_patch_derived_manifest_inner::Kind::PreviewAudio
                }
                crate::application::derived_processing_gateway::DerivedKind::PreviewPhoto => {
                    models::derived_patch_derived_manifest_inner::Kind::PreviewPhoto
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
        mapped.size_bytes = item.size_bytes.and_then(|value| i32::try_from(value).ok());
        mapped.sha256 = item.sha256.clone();
        items.push(mapped);
    }

    patch.derived_manifest = Some(items);
    Ok(patch)
}

#[cfg(feature = "core-api-client")]
fn map_facts_patch(
    facts: &crate::application::derived_processing_gateway::FactsPatchPayload,
) -> models::FactsPatch {
    models::FactsPatch {
        duration_ms: facts.duration_ms,
        media_format: facts.media_format.clone(),
        video_codec: facts.video_codec.clone(),
        audio_codec: facts.audio_codec.clone(),
        width: facts.width,
        height: facts.height,
        fps: facts.fps,
        captured_at: facts.captured_at.clone(),
        exposure_time_s: facts.exposure_time_s,
        aperture_f_number: facts.aperture_f_number,
        iso: facts.iso,
        focal_length_mm: facts.focal_length_mm,
        camera_make: facts.camera_make.clone(),
        camera_model: facts.camera_model.clone(),
        lens_model: facts.lens_model.clone(),
        orientation: facts.orientation,
        bitrate_kbps: facts.bitrate_kbps,
        sample_rate_hz: facts.sample_rate_hz,
        channel_count: facts.channel_count,
        bits_per_sample: facts.bits_per_sample,
        rotation_deg: facts.rotation_deg,
        timecode_start: facts.timecode_start.clone(),
        pixel_format: facts.pixel_format.clone(),
        color_range: facts.color_range.clone(),
        color_space: facts.color_space.clone(),
        color_transfer: facts.color_transfer.clone(),
        color_primaries: facts.color_primaries.clone(),
        recorder_model: facts.recorder_model.clone(),
        gps_latitude: facts.gps_latitude,
        gps_longitude: facts.gps_longitude,
        gps_altitude_m: facts.gps_altitude_m,
        gps_altitude_relative_m: facts.gps_altitude_relative_m,
        gps_altitude_absolute_m: facts.gps_altitude_absolute_m,
        exposure_compensation_ev: facts.exposure_compensation_ev,
        color_mode: facts.color_mode.clone(),
        color_temperature_k: facts.color_temperature_k,
        has_dji_metadata_track: facts.has_dji_metadata_track,
        dji_metadata_track_types: facts.dji_metadata_track_types.clone(),
    }
}

#[cfg(feature = "core-api-client")]
fn map_transcript_patch(
    transcript: &crate::application::derived_processing_gateway::TranscriptPatchPayload,
) -> models::TranscriptPatch {
    let status = transcript.status.as_deref().map(|value| match value {
        "RUNNING" => models::transcript_patch::Status::Running,
        "DONE" => models::transcript_patch::Status::Done,
        "FAILED" => models::transcript_patch::Status::Failed,
        _ => models::transcript_patch::Status::None,
    });

    models::TranscriptPatch {
        status,
        text: transcript.text.clone(),
        text_preview: transcript.text_preview.clone(),
        language: transcript.language.clone(),
        updated_at: transcript.updated_at.clone(),
    }
}

#[cfg(feature = "core-api-client")]
fn require_success(
    response: reqwest::blocking::Response,
    map_status: fn(StatusCode, &str) -> DerivedProcessingError,
) -> Result<reqwest::blocking::Response, DerivedProcessingError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response.text().unwrap_or_default();
    Err(map_status(status, &body))
}

#[cfg(feature = "core-api-client")]
fn map_claim_status(status: StatusCode, body: &str) -> DerivedProcessingError {
    match status.as_u16() {
        401 => DerivedProcessingError::Unauthorized,
        429 => DerivedProcessingError::Throttled,
        409 | 412 => map_lock_error(status, body),
        code => DerivedProcessingError::UnexpectedStatus(code),
    }
}

#[cfg(feature = "core-api-client")]
fn map_asset_get_status(status: StatusCode, _body: &str) -> DerivedProcessingError {
    match status.as_u16() {
        401 => DerivedProcessingError::Unauthorized,
        429 => DerivedProcessingError::Throttled,
        code => DerivedProcessingError::UnexpectedStatus(code),
    }
}

#[cfg(feature = "core-api-client")]
fn map_heartbeat_status(status: StatusCode, body: &str) -> DerivedProcessingError {
    match status.as_u16() {
        401 => DerivedProcessingError::Unauthorized,
        409 | 412 => map_lock_error(status, body),
        code => DerivedProcessingError::UnexpectedStatus(code),
    }
}

#[cfg(feature = "core-api-client")]
fn map_submit_status(status: StatusCode, body: &str) -> DerivedProcessingError {
    match status.as_u16() {
        401 => DerivedProcessingError::Unauthorized,
        429 => DerivedProcessingError::Throttled,
        409 | 412 => map_lock_error(status, body),
        code => DerivedProcessingError::UnexpectedStatus(code),
    }
}

#[cfg(feature = "core-api-client")]
fn map_upload_init_status(status: StatusCode, body: &str) -> DerivedProcessingError {
    match status.as_u16() {
        401 => DerivedProcessingError::Unauthorized,
        409 | 412 => map_lock_error(status, body),
        code => DerivedProcessingError::UnexpectedStatus(code),
    }
}

#[cfg(feature = "core-api-client")]
fn map_upload_part_status(status: StatusCode, body: &str) -> DerivedProcessingError {
    match status.as_u16() {
        401 => DerivedProcessingError::Unauthorized,
        429 => DerivedProcessingError::Throttled,
        409 | 412 => map_lock_error(status, body),
        code => DerivedProcessingError::UnexpectedStatus(code),
    }
}

#[cfg(feature = "core-api-client")]
fn map_upload_complete_status(status: StatusCode, body: &str) -> DerivedProcessingError {
    match status.as_u16() {
        401 => DerivedProcessingError::Unauthorized,
        409 | 412 => map_lock_error(status, body),
        code => DerivedProcessingError::UnexpectedStatus(code),
    }
}

#[cfg(feature = "core-api-client")]
fn map_lock_error(status: StatusCode, body: &str) -> DerivedProcessingError {
    match parse_error_code(body).as_deref() {
        Some("LOCK_REQUIRED") => DerivedProcessingError::LockRequired,
        Some("LOCK_INVALID") => DerivedProcessingError::LockInvalid,
        Some("STALE_LOCK_TOKEN") => DerivedProcessingError::StaleLockToken,
        _ => DerivedProcessingError::UnexpectedStatus(status.as_u16()),
    }
}

#[cfg(feature = "core-api-client")]
fn parse_error_code(body: &str) -> Option<String> {
    let parsed = serde_json::from_str::<serde_json::Value>(body).ok()?;
    parsed.get("code")?.as_str().map(ToString::to_string)
}

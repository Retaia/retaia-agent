use std::collections::HashMap;

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedJobType {
    GenerateProxy,
    GenerateThumbnails,
    GenerateAudioWaveform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedKind {
    ProxyVideo,
    ProxyAudio,
    ProxyPhoto,
    Thumb,
    Waveform,
}

impl DerivedKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProxyVideo => "proxy_video",
            Self::ProxyAudio => "proxy_audio",
            Self::ProxyPhoto => "proxy_photo",
            Self::Thumb => "thumb",
            Self::Waveform => "waveform",
        }
    }

    pub fn allows_content_type(self, content_type: &str) -> bool {
        let value = content_type.trim().to_ascii_lowercase();
        match self {
            Self::ProxyVideo => value == "video/mp4",
            Self::ProxyAudio => value == "audio/mp4" || value == "audio/mpeg",
            Self::ProxyPhoto | Self::Thumb => value == "image/jpeg" || value == "image/webp",
            Self::Waveform => value == "application/json" || value == "application/octet-stream",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedDerivedJob {
    pub job_id: String,
    pub asset_uuid: String,
    pub lock_token: String,
    pub job_type: DerivedJobType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartbeatReceipt {
    pub locked_until: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedManifestItem {
    pub kind: DerivedKind,
    pub reference: String,
    pub size_bytes: Option<u64>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubmitDerivedPayload {
    pub job_type: DerivedJobType,
    pub manifest: Vec<DerivedManifestItem>,
    pub warnings: Option<Vec<String>>,
    pub metrics: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedUploadInit {
    pub asset_uuid: String,
    pub kind: DerivedKind,
    pub content_type: String,
    pub size_bytes: u64,
    pub sha256: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedUploadPart {
    pub asset_uuid: String,
    pub upload_id: String,
    pub part_number: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DerivedUploadComplete {
    pub asset_uuid: String,
    pub upload_id: String,
    pub idempotency_key: String,
    pub parts: Option<Vec<Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DerivedProcessingError {
    #[error("core API unauthorized")]
    Unauthorized,
    #[error("core API throttled")]
    Throttled,
    #[error("core API returned unexpected status {0}")]
    UnexpectedStatus(u16),
    #[error("core API transport error: {0}")]
    Transport(String),
    #[error("invalid derived content type for kind {kind}: {content_type}")]
    InvalidDerivedContentType { kind: String, content_type: String },
    #[error("invalid derived upload size: {0}")]
    InvalidDerivedSize(String),
    #[error("job is not a derived processing job: {0}")]
    NotDerivedJobType(String),
    #[error("claimed job missing lock token")]
    MissingLockToken,
    #[error("numeric conversion overflow: {0}")]
    NumericOverflow(String),
}

pub trait DerivedProcessingGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError>;
    fn heartbeat(
        &self,
        job_id: &str,
        lock_token: &str,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError>;
    fn submit_derived(
        &self,
        job_id: &str,
        lock_token: &str,
        idempotency_key: &str,
        payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError>;
    fn upload_init(&self, request: &DerivedUploadInit) -> Result<(), DerivedProcessingError>;
    fn upload_part(&self, request: &DerivedUploadPart) -> Result<(), DerivedProcessingError>;
    fn upload_complete(
        &self,
        request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError>;
}

pub fn validate_derived_upload_init(
    request: &DerivedUploadInit,
) -> Result<(), DerivedProcessingError> {
    if request.size_bytes == 0 {
        return Err(DerivedProcessingError::InvalidDerivedSize(
            "size_bytes must be greater than zero".to_string(),
        ));
    }
    if !request.kind.allows_content_type(&request.content_type) {
        return Err(DerivedProcessingError::InvalidDerivedContentType {
            kind: request.kind.as_str().to_string(),
            content_type: request.content_type.clone(),
        });
    }
    Ok(())
}

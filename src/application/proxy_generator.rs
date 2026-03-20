use crate::application::derived_processing_gateway::FactsPatchPayload;
use crate::{AgentRuntimeConfig, resolve_source_path};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioProxyFormat {
    Mp4Aac,
    Mpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoProxyFormat {
    Jpeg,
    Webp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailFormat {
    Jpeg,
    Webp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoProxyRequest {
    pub input_path: String,
    pub output_path: String,
    pub max_width: u16,
    pub max_height: u16,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioProxyRequest {
    pub input_path: String,
    pub output_path: String,
    pub format: AudioProxyFormat,
    pub audio_bitrate_kbps: u32,
    pub sample_rate_hz: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoProxyRequest {
    pub input_path: String,
    pub output_path: String,
    pub format: PhotoProxyFormat,
    pub max_width: u16,
    pub max_height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoThumbnailRequest {
    pub input_path: String,
    pub output_path: String,
    pub format: ThumbnailFormat,
    pub max_width: u16,
    pub seek_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioWaveformRequest {
    pub input_path: String,
    pub output_path: String,
    pub bucket_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProxyGenerationError {
    #[error("invalid proxy request: {0}")]
    InvalidRequest(String),
    #[error("ffmpeg command failed with status {status_code:?}: {stderr}")]
    CommandFailed {
        status_code: Option<i32>,
        stderr: String,
    },
    #[error("proxy generation process failed: {0}")]
    Process(String),
}

pub trait ProxyGenerator {
    fn generate_video_proxy(&self, request: &VideoProxyRequest)
    -> Result<(), ProxyGenerationError>;
    fn generate_audio_proxy(&self, request: &AudioProxyRequest)
    -> Result<(), ProxyGenerationError>;
    fn generate_photo_proxy(&self, request: &PhotoProxyRequest)
    -> Result<(), ProxyGenerationError>;
    fn generate_video_thumbnail(
        &self,
        _request: &VideoThumbnailRequest,
    ) -> Result<(), ProxyGenerationError> {
        Err(ProxyGenerationError::InvalidRequest(
            "video thumbnail generation is not supported by this generator".to_string(),
        ))
    }
    fn generate_audio_waveform(
        &self,
        _request: &AudioWaveformRequest,
    ) -> Result<(), ProxyGenerationError> {
        Err(ProxyGenerationError::InvalidRequest(
            "audio waveform generation is not supported by this generator".to_string(),
        ))
    }
    fn extract_media_facts(
        &self,
        _input_path: &str,
    ) -> Result<FactsPatchPayload, ProxyGenerationError> {
        Err(ProxyGenerationError::InvalidRequest(
            "fact extraction is not supported by this generator".to_string(),
        ))
    }
}

pub fn resolve_processing_input_path(
    settings: &AgentRuntimeConfig,
    storage_id: &str,
    relative_path: &str,
) -> Result<String, ProxyGenerationError> {
    resolve_source_path(settings, storage_id, relative_path)
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|error| {
            ProxyGenerationError::InvalidRequest(format!(
                "unable to resolve source path: {error:?}"
            ))
        })
}

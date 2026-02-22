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
}

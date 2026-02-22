use std::process::Command;

use crate::application::proxy_generator::{
    AudioProxyFormat, AudioProxyRequest, PhotoProxyRequest, ProxyGenerationError, ProxyGenerator,
    VideoProxyRequest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub status_code: Option<i32>,
    pub stderr: String,
}

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ProxyGenerationError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StdCommandRunner;

impl CommandRunner for StdCommandRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ProxyGenerationError> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
        Ok(CommandOutput {
            status_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FfmpegProxyGenerator<R: CommandRunner = StdCommandRunner> {
    ffmpeg_binary: String,
    runner: R,
}

impl Default for FfmpegProxyGenerator<StdCommandRunner> {
    fn default() -> Self {
        Self::new("ffmpeg".to_string(), StdCommandRunner)
    }
}

impl<R: CommandRunner> FfmpegProxyGenerator<R> {
    pub fn new(ffmpeg_binary: String, runner: R) -> Self {
        Self {
            ffmpeg_binary,
            runner,
        }
    }

    pub fn runner(&self) -> &R {
        &self.runner
    }
}

impl<R: CommandRunner> ProxyGenerator for FfmpegProxyGenerator<R> {
    fn generate_video_proxy(
        &self,
        request: &VideoProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        validate_video_request(request)?;
        run_ffmpeg(
            &self.runner,
            &self.ffmpeg_binary,
            &build_video_proxy_args(request),
        )
    }

    fn generate_audio_proxy(
        &self,
        request: &AudioProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        validate_audio_request(request)?;
        run_ffmpeg(
            &self.runner,
            &self.ffmpeg_binary,
            &build_audio_proxy_args(request),
        )
    }

    fn generate_photo_proxy(
        &self,
        _request: &PhotoProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        Err(ProxyGenerationError::InvalidRequest(
            "photo proxy generation is handled by RustPhotoProxyGenerator".to_string(),
        ))
    }
}

fn run_ffmpeg<R: CommandRunner>(
    runner: &R,
    ffmpeg_binary: &str,
    args: &[String],
) -> Result<(), ProxyGenerationError> {
    let output = runner.run(ffmpeg_binary, args)?;
    if output.status_code == Some(0) {
        return Ok(());
    }
    Err(ProxyGenerationError::CommandFailed {
        status_code: output.status_code,
        stderr: output.stderr,
    })
}

fn validate_video_request(request: &VideoProxyRequest) -> Result<(), ProxyGenerationError> {
    if request.input_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "video input path is required".to_string(),
        ));
    }
    if request.output_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "video output path is required".to_string(),
        ));
    }
    if request.max_width == 0 || request.max_height == 0 {
        return Err(ProxyGenerationError::InvalidRequest(
            "video max dimensions must be > 0".to_string(),
        ));
    }
    if request.video_bitrate_kbps == 0 || request.audio_bitrate_kbps == 0 {
        return Err(ProxyGenerationError::InvalidRequest(
            "video/audio bitrate must be > 0".to_string(),
        ));
    }
    Ok(())
}

fn validate_audio_request(request: &AudioProxyRequest) -> Result<(), ProxyGenerationError> {
    if request.input_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "audio input path is required".to_string(),
        ));
    }
    if request.output_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "audio output path is required".to_string(),
        ));
    }
    if request.audio_bitrate_kbps == 0 {
        return Err(ProxyGenerationError::InvalidRequest(
            "audio bitrate must be > 0".to_string(),
        ));
    }
    if request.sample_rate_hz == 0 {
        return Err(ProxyGenerationError::InvalidRequest(
            "audio sample rate must be > 0".to_string(),
        ));
    }
    Ok(())
}

pub fn build_video_proxy_args(request: &VideoProxyRequest) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-i".to_string(),
        request.input_path.clone(),
        "-vf".to_string(),
        format!(
            "scale=w={}:h={}:force_original_aspect_ratio=decrease",
            request.max_width, request.max_height
        ),
        "-vsync".to_string(),
        "cfr".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-b:v".to_string(),
        format!("{}k", request.video_bitrate_kbps),
        "-g".to_string(),
        "48".to_string(),
        "-keyint_min".to_string(),
        "48".to_string(),
        "-sc_threshold".to_string(),
        "0".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-profile:a".to_string(),
        "aac_low".to_string(),
        "-b:a".to_string(),
        format!("{}k", request.audio_bitrate_kbps),
        "-movflags".to_string(),
        "+faststart".to_string(),
        request.output_path.clone(),
    ]
}

pub fn build_audio_proxy_args(request: &AudioProxyRequest) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        request.input_path.clone(),
    ];

    match request.format {
        AudioProxyFormat::Mp4Aac => {
            args.extend_from_slice(&[
                "-c:a".to_string(),
                "aac".to_string(),
                "-profile:a".to_string(),
                "aac_low".to_string(),
                "-movflags".to_string(),
                "+faststart".to_string(),
            ]);
        }
        AudioProxyFormat::Mpeg => {
            args.extend_from_slice(&["-c:a".to_string(), "libmp3lame".to_string()]);
        }
    }

    args.extend_from_slice(&[
        "-b:a".to_string(),
        format!("{}k", request.audio_bitrate_kbps),
        "-ar".to_string(),
        request.sample_rate_hz.to_string(),
        request.output_path.clone(),
    ]);
    args
}

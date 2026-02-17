use std::sync::Mutex;

use retaia_agent::{
    AudioProxyFormat, AudioProxyRequest, CommandOutput, CommandRunner, FfmpegProxyGenerator,
    ProxyGenerationError, ProxyGenerator, VideoProxyRequest,
};

struct ScenarioRunner {
    output: CommandOutput,
    calls: Mutex<Vec<Vec<String>>>,
}

impl ScenarioRunner {
    fn with_output(output: CommandOutput) -> Self {
        Self {
            output,
            calls: Mutex::new(Vec::new()),
        }
    }

    fn first_args(&self) -> Vec<String> {
        self.calls
            .lock()
            .expect("calls")
            .first()
            .expect("first call")
            .clone()
    }

    fn call_count(&self) -> usize {
        self.calls.lock().expect("calls").len()
    }
}

impl CommandRunner for ScenarioRunner {
    fn run(&self, _program: &str, args: &[String]) -> Result<CommandOutput, ProxyGenerationError> {
        self.calls.lock().expect("calls").push(args.to_vec());
        Ok(self.output.clone())
    }
}

#[test]
fn bdd_given_audio_proxy_format_mpeg_when_generating_then_ffmpeg_uses_libmp3lame_encoder() {
    let runner = ScenarioRunner::with_output(CommandOutput {
        status_code: Some(0),
        stderr: String::new(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);

    generator
        .generate_audio_proxy(&AudioProxyRequest {
            input_path: "/tmp/in.wav".to_string(),
            output_path: "/tmp/out.mp3".to_string(),
            format: AudioProxyFormat::Mpeg,
            audio_bitrate_kbps: 192,
            sample_rate_hz: 44100,
        })
        .expect("mpeg proxy should succeed");

    let args = generator.runner().first_args().join(" ");
    assert!(args.contains("-c:a libmp3lame"));
    assert!(!args.contains("-movflags +faststart"));
}

#[test]
fn bdd_given_ffmpeg_non_zero_exit_when_generating_proxy_then_command_failed_error_is_returned() {
    let runner = ScenarioRunner::with_output(CommandOutput {
        status_code: Some(1),
        stderr: "encoding failed".to_string(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);

    let err = generator
        .generate_audio_proxy(&AudioProxyRequest {
            input_path: "/tmp/in.wav".to_string(),
            output_path: "/tmp/out.m4a".to_string(),
            format: AudioProxyFormat::Mp4Aac,
            audio_bitrate_kbps: 160,
            sample_rate_hz: 48000,
        })
        .expect_err("ffmpeg failure should be propagated");

    assert!(matches!(err, ProxyGenerationError::CommandFailed { .. }));
}

#[test]
fn bdd_given_video_proxy_request_when_generating_then_ffmpeg_is_called_with_h264_cfr_and_faststart()
{
    let runner = ScenarioRunner::with_output(CommandOutput {
        status_code: Some(0),
        stderr: String::new(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);

    generator
        .generate_video_proxy(&VideoProxyRequest {
            input_path: "/tmp/in.mov".to_string(),
            output_path: "/tmp/out.mp4".to_string(),
            max_width: 1280,
            max_height: 720,
            video_bitrate_kbps: 3000,
            audio_bitrate_kbps: 128,
        })
        .expect("video proxy should succeed");

    let args = generator.runner().first_args().join(" ");
    assert!(args.contains("-c:v libx264"));
    assert!(args.contains("-vsync cfr"));
    assert!(args.contains("-movflags +faststart"));
    assert!(args.contains("force_original_aspect_ratio=decrease"));
}

#[test]
fn bdd_given_invalid_video_proxy_request_when_generating_then_validation_fails_before_runner_call()
{
    let runner = ScenarioRunner::with_output(CommandOutput {
        status_code: Some(0),
        stderr: String::new(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);

    let err = generator
        .generate_video_proxy(&VideoProxyRequest {
            input_path: String::new(),
            output_path: "/tmp/out.mp4".to_string(),
            max_width: 1280,
            max_height: 720,
            video_bitrate_kbps: 3000,
            audio_bitrate_kbps: 128,
        })
        .expect_err("invalid video request should fail");

    assert!(matches!(err, ProxyGenerationError::InvalidRequest(_)));
    assert_eq!(generator.runner().call_count(), 0);
}

#[test]
fn bdd_given_invalid_audio_proxy_request_when_zero_sample_rate_then_validation_fails() {
    let runner = ScenarioRunner::with_output(CommandOutput {
        status_code: Some(0),
        stderr: String::new(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);

    let err = generator
        .generate_audio_proxy(&AudioProxyRequest {
            input_path: "/tmp/in.wav".to_string(),
            output_path: "/tmp/out.m4a".to_string(),
            format: AudioProxyFormat::Mp4Aac,
            audio_bitrate_kbps: 160,
            sample_rate_hz: 0,
        })
        .expect_err("invalid audio request should fail");

    assert!(matches!(err, ProxyGenerationError::InvalidRequest(_)));
    assert_eq!(generator.runner().call_count(), 0);
}

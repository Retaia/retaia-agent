use std::sync::Mutex;

use retaia_agent::{
    AudioProxyFormat, AudioProxyRequest, CommandOutput, CommandRunner, FfmpegProxyGenerator,
    ProxyGenerationError, ProxyGenerator,
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

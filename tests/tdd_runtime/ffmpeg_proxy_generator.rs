use std::sync::Mutex;

use retaia_agent::{
    AudioProxyFormat, AudioProxyRequest, CommandOutput, CommandRunner, FfmpegProxyGenerator,
    ProxyGenerationError, ProxyGenerator, VideoProxyRequest,
};

#[derive(Debug)]
struct RecordedCall {
    program: String,
    args: Vec<String>,
}

struct FakeRunner {
    output: CommandOutput,
    calls: Mutex<Vec<RecordedCall>>,
}

impl FakeRunner {
    fn success() -> Self {
        Self {
            output: CommandOutput {
                status_code: Some(0),
                stderr: String::new(),
            },
            calls: Mutex::new(Vec::new()),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.lock().expect("calls").len()
    }

    fn first_call(&self) -> RecordedCall {
        let calls = self.calls.lock().expect("calls");
        let call = calls.first().expect("at least one call");
        RecordedCall {
            program: call.program.clone(),
            args: call.args.clone(),
        }
    }
}

impl CommandRunner for FakeRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ProxyGenerationError> {
        self.calls.lock().expect("calls").push(RecordedCall {
            program: program.to_string(),
            args: args.to_vec(),
        });
        Ok(self.output.clone())
    }
}

#[test]
fn tdd_ffmpeg_video_proxy_uses_h264_aac_cfr_and_faststart() {
    let runner = FakeRunner::success();
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);
    let request = VideoProxyRequest {
        input_path: "/tmp/in.mov".to_string(),
        output_path: "/tmp/out.mp4".to_string(),
        max_width: 1280,
        max_height: 720,
        video_bitrate_kbps: 3500,
        audio_bitrate_kbps: 128,
    };

    generator
        .generate_video_proxy(&request)
        .expect("video proxy should succeed");

    let call = generator_runner_call(&generator);
    let joined = call.args.join(" ");
    assert_eq!(call.program, "ffmpeg");
    assert!(joined.contains("-c:v libx264"));
    assert!(joined.contains("-c:a aac"));
    assert!(joined.contains("-vsync cfr"));
    assert!(joined.contains("-movflags +faststart"));
    assert!(joined.contains("force_original_aspect_ratio=decrease"));
}

#[test]
fn tdd_ffmpeg_audio_proxy_mp4_uses_aac_low_profile_and_faststart() {
    let runner = FakeRunner::success();
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);
    let request = AudioProxyRequest {
        input_path: "/tmp/in.wav".to_string(),
        output_path: "/tmp/out.m4a".to_string(),
        format: AudioProxyFormat::Mp4Aac,
        audio_bitrate_kbps: 160,
        sample_rate_hz: 48000,
    };

    generator
        .generate_audio_proxy(&request)
        .expect("audio proxy should succeed");

    let call = generator_runner_call(&generator);
    let joined = call.args.join(" ");
    assert!(joined.contains("-c:a aac"));
    assert!(joined.contains("-profile:a aac_low"));
    assert!(joined.contains("-movflags +faststart"));
    assert!(joined.contains("-ar 48000"));
}

#[test]
fn tdd_ffmpeg_proxy_rejects_invalid_request_before_running_process() {
    let runner = FakeRunner::success();
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);
    let invalid = VideoProxyRequest {
        input_path: String::new(),
        output_path: "/tmp/out.mp4".to_string(),
        max_width: 1280,
        max_height: 720,
        video_bitrate_kbps: 3500,
        audio_bitrate_kbps: 128,
    };

    let err = generator
        .generate_video_proxy(&invalid)
        .expect_err("invalid request must fail");
    assert!(matches!(err, ProxyGenerationError::InvalidRequest(_)));
    assert_eq!(generator_runner_call_count(&generator), 0);
}

fn generator_runner_call(generator: &FfmpegProxyGenerator<FakeRunner>) -> RecordedCall {
    generator.runner().first_call()
}

fn generator_runner_call_count(generator: &FfmpegProxyGenerator<FakeRunner>) -> usize {
    generator.runner().call_count()
}

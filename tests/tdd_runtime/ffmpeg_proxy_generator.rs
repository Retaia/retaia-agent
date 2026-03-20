use std::sync::Mutex;

use retaia_agent::{
    AudioProxyFormat, AudioProxyRequest, AudioWaveformRequest, CommandOutput, CommandRunner,
    FfmpegProxyGenerator, ProxyGenerationError, ProxyGenerator, ThumbnailFormat, VideoProxyRequest,
    VideoThumbnailRequest,
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
                stdout: String::new(),
                stderr: String::new(),
            },
            calls: Mutex::new(Vec::new()),
        }
    }

    fn with_output(output: CommandOutput) -> Self {
        Self {
            output,
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

struct WaveformRunner {
    calls: Mutex<Vec<RecordedCall>>,
}

impl WaveformRunner {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl CommandRunner for WaveformRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ProxyGenerationError> {
        self.calls.lock().expect("calls").push(RecordedCall {
            program: program.to_string(),
            args: args.to_vec(),
        });
        let wav_path = args.last().expect("wav output path");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(wav_path, spec)
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
        for sample in [0_i16, 8_000, -16_000, 4_000, 0] {
            writer
                .write_sample(sample)
                .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
        }
        writer
            .finalize()
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
        Ok(CommandOutput {
            status_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        })
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
    assert!(joined.contains("-profile:v high"));
    assert!(joined.contains("-preset medium"));
    assert!(joined.contains("-crf 23"));
    assert!(joined.contains("-c:a aac"));
    assert!(joined.contains("-vsync cfr"));
    assert!(joined.contains("-ac 2"));
    assert!(joined.contains("-ar 48000"));
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
    assert!(joined.contains("-ac 2"));
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

#[test]
fn tdd_ffmpeg_thumbnail_uses_webp_encoder_and_representative_seek() {
    let runner = FakeRunner::success();
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);
    let request = VideoThumbnailRequest {
        input_path: "/tmp/in.mov".to_string(),
        output_path: "/tmp/out.webp".to_string(),
        format: ThumbnailFormat::Webp,
        max_width: 480,
        seek_ms: 1_000,
    };

    generator
        .generate_video_thumbnail(&request)
        .expect("thumbnail generation should succeed");

    let call = generator_runner_call(&generator);
    let joined = call.args.join(" ");
    assert!(joined.contains("-ss 1.000"));
    assert!(joined.contains("-frames:v 1"));
    assert!(joined.contains("-c:v libwebp"));
    assert!(joined.contains("-quality 75"));
    assert!(joined.contains("scale=w=480:h=-2:force_original_aspect_ratio=decrease"));
}

#[test]
fn tdd_ffmpeg_waveform_generates_json_with_requested_bucket_count() {
    let runner = WaveformRunner::new();
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("waveform.json");

    generator
        .generate_audio_waveform(&AudioWaveformRequest {
            input_path: "/tmp/in.wav".to_string(),
            output_path: output.display().to_string(),
            bucket_count: 100,
        })
        .expect("waveform generation should succeed");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).expect("read waveform json"))
            .expect("json payload");
    assert_eq!(payload.get("bucket_count"), Some(&serde_json::json!(100)));
    assert_eq!(
        payload
            .get("samples")
            .and_then(|samples| samples.as_array())
            .map(|samples| samples.len()),
        Some(100)
    );
}

#[test]
fn tdd_ffmpeg_extract_media_facts_maps_ffprobe_json_to_patch() {
    let runner = FakeRunner::with_output(CommandOutput {
        status_code: Some(0),
        stdout: r#"{
            "format":{"duration":"12.345","format_name":"mov,mp4,m4a,3gp,3g2,mj2"},
            "streams":[
                {"codec_type":"video","codec_name":"h264","width":1920,"height":1080,"avg_frame_rate":"30000/1001"},
                {"codec_type":"audio","codec_name":"aac"}
            ]
        }"#
        .to_string(),
        stderr: String::new(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);

    let facts = generator
        .extract_media_facts("/tmp/in.mov")
        .expect("facts extraction should succeed");

    assert_eq!(facts.duration_ms, Some(12_345));
    assert_eq!(facts.media_format.as_deref(), Some("mov"));
    assert_eq!(facts.video_codec.as_deref(), Some("h264"));
    assert_eq!(facts.audio_codec.as_deref(), Some("aac"));
    assert_eq!(facts.width, Some(1920));
    assert_eq!(facts.height, Some(1080));
    assert_eq!(facts.fps, Some(30000.0 / 1001.0));
}

fn generator_runner_call(generator: &FfmpegProxyGenerator<FakeRunner>) -> RecordedCall {
    generator.runner().first_call()
}

fn generator_runner_call_count(generator: &FfmpegProxyGenerator<FakeRunner>) -> usize {
    generator.runner().call_count()
}

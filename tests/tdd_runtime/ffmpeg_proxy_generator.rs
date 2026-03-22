use std::sync::Mutex;
use std::time::{Duration, SystemTime};

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

#[test]
fn tdd_ffmpeg_extract_media_facts_repairs_rode_bext_timestamp_from_file_year() {
    let runner = FakeRunner::with_output(CommandOutput {
        status_code: Some(0),
        stdout: r#"{
            "format":{"duration":"240.326","format_name":"wav","tags":{"encoded_by":"RODE Wireless PRO"}},
            "streams":[
                {"codec_type":"audio","codec_name":"pcm_f32le","sample_rate":"48000","channels":1,"bits_per_sample":32}
            ]
        }"#
        .to_string(),
        stderr: String::new(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("rode.wav");
    write_test_wav_with_chunks(
        &path,
        Some(TestBextChunk {
            originator: "RODE Wireless PRO",
            origination_date: "0026-03-22",
            origination_time: "10:03:39",
        }),
        Some(
            r#"<BWFXML><TIMESTAMP_SAMPLES_SINCE_MIDNIGHT>1738563538</TIMESTAMP_SAMPLES_SINCE_MIDNIGHT><TIMESTAMP_SAMPLE_RATE>48000</TIMESTAMP_SAMPLE_RATE></BWFXML>"#,
        ),
    );
    let modified_at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_774_176_219);
    filetime::set_file_mtime(&path, filetime::FileTime::from_system_time(modified_at))
        .expect("set mtime");

    let facts = generator
        .extract_media_facts(&path.display().to_string())
        .expect("facts extraction should succeed");

    assert_eq!(facts.media_format.as_deref(), Some("wav"));
    assert_eq!(facts.audio_codec.as_deref(), Some("pcm_f32le"));
    assert_eq!(facts.recorder_model.as_deref(), Some("RODE Wireless PRO"));
    assert_eq!(facts.captured_at.as_deref(), Some("2026-03-22T10:03:39Z"));
}

#[test]
fn tdd_ffmpeg_extract_media_facts_falls_back_to_file_timestamp_when_bext_date_cannot_be_repaired() {
    let runner = FakeRunner::with_output(CommandOutput {
        status_code: Some(0),
        stdout: r#"{
            "format":{"duration":"240.326","format_name":"wav","tags":{"encoded_by":"RODE Wireless PRO"}},
            "streams":[
                {"codec_type":"audio","codec_name":"pcm_f32le","sample_rate":"48000","channels":1,"bits_per_sample":32}
            ]
        }"#
        .to_string(),
        stderr: String::new(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("rode.wav");
    write_test_wav_with_chunks(
        &path,
        Some(TestBextChunk {
            originator: "RODE Wireless PRO",
            origination_date: "0026-03-22",
            origination_time: "10:03:39",
        }),
        None,
    );
    let modified_at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_774_089_600);
    filetime::set_file_mtime(&path, filetime::FileTime::from_system_time(modified_at))
        .expect("set mtime");

    let facts = generator
        .extract_media_facts(&path.display().to_string())
        .expect("facts extraction should succeed");

    assert_eq!(facts.captured_at.as_deref(), Some("2026-03-21T10:40:00Z"));
    assert_eq!(facts.recorder_model.as_deref(), Some("RODE Wireless PRO"));
}

#[test]
fn tdd_ffmpeg_extract_media_facts_falls_back_to_file_creation_time_for_plain_wav() {
    let runner = FakeRunner::with_output(CommandOutput {
        status_code: Some(0),
        stdout: r#"{
            "format":{"duration":"1.000","format_name":"wav"},
            "streams":[
                {"codec_type":"audio","codec_name":"pcm_s16le","sample_rate":"48000","channels":1,"bits_per_sample":16}
            ]
        }"#
        .to_string(),
        stderr: String::new(),
    });
    let generator = FfmpegProxyGenerator::new("ffmpeg".to_string(), runner);
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("plain.wav");
    write_test_wav_with_chunks(&path, None, None);
    let modified_at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_774_175_848);
    filetime::set_file_mtime(&path, filetime::FileTime::from_system_time(modified_at))
        .expect("set mtime");

    let facts = generator
        .extract_media_facts(&path.display().to_string())
        .expect("facts extraction should succeed");

    assert_eq!(facts.media_format.as_deref(), Some("wav"));
    assert_eq!(facts.audio_codec.as_deref(), Some("pcm_s16le"));
    assert_eq!(facts.captured_at.as_deref(), Some("2026-03-22T10:37:28Z"));
}

struct TestBextChunk<'a> {
    originator: &'a str,
    origination_date: &'a str,
    origination_time: &'a str,
}

fn write_test_wav_with_chunks(
    path: &std::path::Path,
    bext: Option<TestBextChunk<'_>>,
    ixml: Option<&str>,
) {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"WAVE");
    push_chunk(&mut payload, b"fmt ", &build_pcm_fmt_chunk());
    if let Some(bext) = bext {
        push_chunk(&mut payload, b"bext", &build_bext_chunk(&bext));
    }
    if let Some(ixml) = ixml {
        push_chunk(&mut payload, b"iXML", ixml.as_bytes());
    }
    push_chunk(&mut payload, b"data", &[0, 0, 0, 0]);

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&payload);
    std::fs::write(path, bytes).expect("write wav bytes");
}

fn build_pcm_fmt_chunk() -> Vec<u8> {
    let mut chunk = Vec::new();
    chunk.extend_from_slice(&1u16.to_le_bytes());
    chunk.extend_from_slice(&1u16.to_le_bytes());
    chunk.extend_from_slice(&48_000u32.to_le_bytes());
    chunk.extend_from_slice(&192_000u32.to_le_bytes());
    chunk.extend_from_slice(&4u16.to_le_bytes());
    chunk.extend_from_slice(&32u16.to_le_bytes());
    chunk
}

fn build_bext_chunk(data: &TestBextChunk<'_>) -> Vec<u8> {
    let mut chunk = vec![0_u8; 602];
    write_ascii_field(&mut chunk[256..288], data.originator);
    write_ascii_field(&mut chunk[320..330], data.origination_date);
    write_ascii_field(&mut chunk[330..338], data.origination_time);
    chunk
}

fn write_ascii_field(slice: &mut [u8], value: &str) {
    let bytes = value.as_bytes();
    let len = bytes.len().min(slice.len());
    slice[..len].copy_from_slice(&bytes[..len]);
}

fn push_chunk(target: &mut Vec<u8>, id: &[u8; 4], data: &[u8]) {
    target.extend_from_slice(id);
    target.extend_from_slice(&(data.len() as u32).to_le_bytes());
    target.extend_from_slice(data);
    if data.len() % 2 == 1 {
        target.push(0);
    }
}

fn generator_runner_call(generator: &FfmpegProxyGenerator<FakeRunner>) -> RecordedCall {
    generator.runner().first_call()
}

fn generator_runner_call_count(generator: &FfmpegProxyGenerator<FakeRunner>) -> usize {
    generator.runner().call_count()
}

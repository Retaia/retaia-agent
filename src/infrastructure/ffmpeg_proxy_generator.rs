use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::process::Command;

use crate::application::derived_processing_gateway::FactsPatchPayload;
use crate::application::proxy_generator::{
    AudioProxyFormat, AudioProxyRequest, AudioWaveformRequest, PhotoProxyRequest,
    ProxyGenerationError, ProxyGenerator, ThumbnailFormat, VideoProxyRequest,
    VideoThumbnailRequest,
};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
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
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FfmpegProxyGenerator<
    R: CommandRunner = StdCommandRunner,
    T: FileTimestampProvider = StdFileTimestampProvider,
> {
    ffmpeg_binary: String,
    runner: R,
    timestamp_provider: T,
}

impl Default for FfmpegProxyGenerator<StdCommandRunner, StdFileTimestampProvider> {
    fn default() -> Self {
        Self::new("ffmpeg".to_string(), StdCommandRunner)
    }
}

pub trait FileTimestampProvider {
    fn created_at_utc(&self, path: &Path) -> Option<chrono::DateTime<Utc>>;
    fn modified_at_utc(&self, path: &Path) -> Option<chrono::DateTime<Utc>>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StdFileTimestampProvider;

impl FileTimestampProvider for StdFileTimestampProvider {
    fn created_at_utc(&self, path: &Path) -> Option<chrono::DateTime<Utc>> {
        fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.created().ok())
            .map(chrono::DateTime::<Utc>::from)
    }

    fn modified_at_utc(&self, path: &Path) -> Option<chrono::DateTime<Utc>> {
        fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(chrono::DateTime::<Utc>::from)
    }
}

impl<R: CommandRunner> FfmpegProxyGenerator<R, StdFileTimestampProvider> {
    pub fn new(ffmpeg_binary: String, runner: R) -> Self {
        Self::new_with_timestamp_provider(ffmpeg_binary, runner, StdFileTimestampProvider)
    }
}

impl<R: CommandRunner, T: FileTimestampProvider> FfmpegProxyGenerator<R, T> {
    pub fn new_with_timestamp_provider(
        ffmpeg_binary: String,
        runner: R,
        timestamp_provider: T,
    ) -> Self {
        Self {
            ffmpeg_binary,
            runner,
            timestamp_provider,
        }
    }

    pub fn runner(&self) -> &R {
        &self.runner
    }
}

impl<R: CommandRunner, T: FileTimestampProvider> ProxyGenerator for FfmpegProxyGenerator<R, T> {
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

    fn generate_video_thumbnail(
        &self,
        request: &VideoThumbnailRequest,
    ) -> Result<(), ProxyGenerationError> {
        validate_thumbnail_request(request)?;
        run_ffmpeg(
            &self.runner,
            &self.ffmpeg_binary,
            &build_video_thumbnail_args(request),
        )
    }

    fn generate_audio_waveform(
        &self,
        request: &AudioWaveformRequest,
    ) -> Result<(), ProxyGenerationError> {
        validate_waveform_request(request)?;
        let output = Path::new(&request.output_path);
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
            }
        }

        let temp_dir = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let temp_wav = tempfile::Builder::new()
            .prefix("retaia-waveform-")
            .suffix(".wav")
            .tempfile_in(temp_dir)
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
        let wav_path = temp_wav.path().to_path_buf();
        drop(temp_wav);

        let generation_result = run_ffmpeg(
            &self.runner,
            &self.ffmpeg_binary,
            &build_audio_waveform_decode_args(request, &wav_path),
        )
        .and_then(|()| write_waveform_json_from_wav(&wav_path, output, request.bucket_count));

        let _ = fs::remove_file(&wav_path);
        generation_result
    }

    fn extract_media_facts(
        &self,
        input_path: &str,
    ) -> Result<FactsPatchPayload, ProxyGenerationError> {
        if input_path.trim().is_empty() {
            return Err(ProxyGenerationError::InvalidRequest(
                "facts input path is required".to_string(),
            ));
        }
        let output = self.runner.run(
            &ffprobe_binary(&self.ffmpeg_binary),
            &build_ffprobe_args(input_path),
        )?;
        if output.status_code != Some(0) {
            return Err(ProxyGenerationError::CommandFailed {
                status_code: output.status_code,
                stderr: output.stderr,
            });
        }
        let mut facts = parse_ffprobe_facts(&output.stdout)?;
        merge_wav_container_facts(input_path, &mut facts, &self.timestamp_provider)?;
        Ok(facts)
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

fn validate_thumbnail_request(request: &VideoThumbnailRequest) -> Result<(), ProxyGenerationError> {
    if request.input_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "thumbnail input path is required".to_string(),
        ));
    }
    if request.output_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "thumbnail output path is required".to_string(),
        ));
    }
    if request.max_width == 0 {
        return Err(ProxyGenerationError::InvalidRequest(
            "thumbnail max width must be > 0".to_string(),
        ));
    }
    Ok(())
}

fn validate_waveform_request(request: &AudioWaveformRequest) -> Result<(), ProxyGenerationError> {
    if request.input_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "waveform input path is required".to_string(),
        ));
    }
    if request.output_path.trim().is_empty() {
        return Err(ProxyGenerationError::InvalidRequest(
            "waveform output path is required".to_string(),
        ));
    }
    if request.bucket_count < 100 {
        return Err(ProxyGenerationError::InvalidRequest(
            "waveform bucket_count must be >= 100".to_string(),
        ));
    }
    Ok(())
}

pub fn build_video_proxy_args(request: &VideoProxyRequest) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-i".to_string(),
        request.input_path.clone(),
        "-map".to_string(),
        "0:v:0".to_string(),
        "-map".to_string(),
        "0:a:0?".to_string(),
        "-vf".to_string(),
        format!(
            "scale=w={}:h={}:force_original_aspect_ratio=decrease",
            request.max_width, request.max_height
        ),
        "-vsync".to_string(),
        "cfr".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-profile:v".to_string(),
        "high".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-preset".to_string(),
        "medium".to_string(),
        "-crf".to_string(),
        "23".to_string(),
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
        "-ac".to_string(),
        "2".to_string(),
        "-ar".to_string(),
        "48000".to_string(),
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
        "-ac".to_string(),
        "2".to_string(),
        "-ar".to_string(),
        request.sample_rate_hz.to_string(),
        request.output_path.clone(),
    ]);
    args
}

pub fn build_video_thumbnail_args(request: &VideoThumbnailRequest) -> Vec<String> {
    let codec = match request.format {
        ThumbnailFormat::Jpeg => "mjpeg",
        ThumbnailFormat::Webp => "libwebp",
    };
    let quality_args = match request.format {
        ThumbnailFormat::Jpeg => vec!["-q:v".to_string(), "2".to_string()],
        ThumbnailFormat::Webp => vec!["-quality".to_string(), "75".to_string()],
    };

    let mut args = vec![
        "-y".to_string(),
        "-ss".to_string(),
        format!("{:.3}", request.seek_ms as f64 / 1000.0),
        "-i".to_string(),
        request.input_path.clone(),
        "-frames:v".to_string(),
        "1".to_string(),
        "-vf".to_string(),
        format!(
            "scale=w={}:h=-2:force_original_aspect_ratio=decrease",
            request.max_width
        ),
        "-c:v".to_string(),
        codec.to_string(),
    ];
    args.extend(quality_args);
    args.push(request.output_path.clone());
    args
}

pub fn build_audio_waveform_decode_args(
    request: &AudioWaveformRequest,
    wav_path: &Path,
) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-i".to_string(),
        request.input_path.clone(),
        "-vn".to_string(),
        "-ac".to_string(),
        "1".to_string(),
        "-ar".to_string(),
        "16000".to_string(),
        "-c:a".to_string(),
        "pcm_s16le".to_string(),
        wav_path.to_string_lossy().to_string(),
    ]
}

pub fn build_ffprobe_args(input_path: &str) -> Vec<String> {
    vec![
        "-v".to_string(),
        "quiet".to_string(),
        "-print_format".to_string(),
        "json".to_string(),
        "-show_format".to_string(),
        "-show_streams".to_string(),
        input_path.to_string(),
    ]
}

#[derive(Serialize)]
struct WaveformJson {
    duration_ms: u64,
    bucket_count: usize,
    samples: Vec<f32>,
}

fn write_waveform_json_from_wav(
    wav_path: &Path,
    output_path: &Path,
    bucket_count: usize,
) -> Result<(), ProxyGenerationError> {
    let mut reader = hound::WavReader::open(wav_path)
        .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
    let spec = reader.spec();
    let sample_rate = u64::from(spec.sample_rate);
    if sample_rate == 0 {
        return Err(ProxyGenerationError::Process(
            "waveform sample_rate must be > 0".to_string(),
        ));
    }

    let samples: Vec<i16> = reader
        .samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
    if samples.is_empty() {
        return Err(ProxyGenerationError::Process(
            "waveform source produced no samples".to_string(),
        ));
    }

    let duration_ms = ((samples.len() as f64 / sample_rate as f64) * 1000.0).round() as u64;
    let mut buckets = Vec::with_capacity(bucket_count);
    for bucket in 0..bucket_count {
        let start = bucket * samples.len() / bucket_count;
        let end = ((bucket + 1) * samples.len() / bucket_count).max(start + 1);
        let end = end.min(samples.len());
        let peak = samples[start..end]
            .iter()
            .map(|sample| i32::from(*sample).unsigned_abs())
            .max()
            .unwrap_or(0) as f32
            / i16::MAX as f32;
        buckets.push(peak.clamp(0.0, 1.0));
    }

    let file = fs::File::create(output_path)
        .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(
        writer,
        &WaveformJson {
            duration_ms,
            bucket_count,
            samples: buckets,
        },
    )
    .map_err(|error| ProxyGenerationError::Process(error.to_string()))
}

fn ffprobe_binary(ffmpeg_binary: &str) -> String {
    ffmpeg_binary
        .rsplit_once("ffmpeg")
        .map(|(prefix, _)| format!("{prefix}ffprobe"))
        .unwrap_or_else(|| "ffprobe".to_string())
}

fn parse_ffprobe_facts(stdout: &str) -> Result<FactsPatchPayload, ProxyGenerationError> {
    let value: serde_json::Value = serde_json::from_str(stdout)
        .map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
    let format = value.get("format");
    let streams = value
        .get("streams")
        .and_then(|streams| streams.as_array())
        .cloned()
        .unwrap_or_default();

    let video_stream = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(|v| v.as_str()) == Some("video"));
    let audio_stream = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(|v| v.as_str()) == Some("audio"));

    let duration_ms = format
        .and_then(|format| format.get("duration").and_then(|v| v.as_str()))
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| (value * 1000.0).round() as i32);
    let media_format = format
        .and_then(|format| format.get("format_name").and_then(|v| v.as_str()))
        .map(|value| value.split(',').next().unwrap_or(value).to_string());
    let video_codec = video_stream
        .and_then(|stream| stream.get("codec_name").and_then(|v| v.as_str()))
        .map(ToString::to_string);
    let audio_codec = audio_stream
        .and_then(|stream| stream.get("codec_name").and_then(|v| v.as_str()))
        .map(ToString::to_string);
    let width = video_stream
        .and_then(|stream| stream.get("width").and_then(|v| v.as_i64()))
        .and_then(|value| i32::try_from(value).ok());
    let height = video_stream
        .and_then(|stream| stream.get("height").and_then(|v| v.as_i64()))
        .and_then(|value| i32::try_from(value).ok());
    let fps = video_stream.and_then(parse_stream_fps);
    let captured_at = format
        .and_then(|value| value.get("tags"))
        .and_then(parse_format_tags_captured_at);
    let camera_make = format
        .and_then(|value| value.get("tags"))
        .and_then(|value| value.get("com.apple.quicktime.make"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let camera_model = format
        .and_then(|value| value.get("tags"))
        .and_then(|value| value.get("com.apple.quicktime.model"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let recorder_model = format
        .and_then(|value| value.get("tags"))
        .and_then(|value| {
            value
                .get("encoded_by")
                .or_else(|| value.get("encoder"))
                .and_then(|value| value.as_str())
        })
        .map(ToString::to_string);
    let bitrate_kbps = video_stream
        .and_then(|stream| stream.get("bit_rate").and_then(|value| value.as_str()))
        .or_else(|| {
            audio_stream.and_then(|stream| stream.get("bit_rate").and_then(|value| value.as_str()))
        })
        .and_then(|value| value.parse::<u64>().ok())
        .and_then(|value| i32::try_from(value / 1000).ok());
    let sample_rate_hz = audio_stream
        .and_then(|stream| stream.get("sample_rate").and_then(|value| value.as_str()))
        .and_then(|value| value.parse::<i32>().ok());
    let channel_count = audio_stream
        .and_then(|stream| stream.get("channels").and_then(|value| value.as_i64()))
        .and_then(|value| i32::try_from(value).ok());
    let bits_per_sample = audio_stream
        .and_then(|stream| {
            stream
                .get("bits_per_sample")
                .and_then(|value| value.as_i64())
        })
        .and_then(|value| i32::try_from(value).ok());
    let rotation_deg = video_stream
        .and_then(|stream| stream.get("tags"))
        .and_then(|value| value.get("rotate"))
        .and_then(|value| value.as_str())
        .and_then(|value| value.parse::<i32>().ok());
    let timecode_start = video_stream
        .and_then(|stream| stream.get("tags"))
        .and_then(|value| value.get("timecode"))
        .and_then(|value| value.as_str())
        .or_else(|| {
            audio_stream
                .and_then(|stream| stream.get("tags"))
                .and_then(|value| value.get("timecode"))
                .and_then(|value| value.as_str())
        })
        .map(ToString::to_string);
    let pixel_format = video_stream
        .and_then(|stream| stream.get("pix_fmt").and_then(|value| value.as_str()))
        .map(ToString::to_string);
    let color_range = video_stream
        .and_then(|stream| stream.get("color_range").and_then(|value| value.as_str()))
        .map(ToString::to_string);
    let color_space = video_stream
        .and_then(|stream| stream.get("color_space").and_then(|value| value.as_str()))
        .map(ToString::to_string);
    let color_transfer = video_stream
        .and_then(|stream| {
            stream
                .get("color_transfer")
                .and_then(|value| value.as_str())
        })
        .map(ToString::to_string);
    let color_primaries = video_stream
        .and_then(|stream| {
            stream
                .get("color_primaries")
                .and_then(|value| value.as_str())
        })
        .map(ToString::to_string);
    let dji_metadata_track_types: Vec<String> = streams
        .iter()
        .filter(|stream| stream.get("codec_type").and_then(|value| value.as_str()) == Some("data"))
        .filter_map(|stream| {
            stream
                .get("codec_tag_string")
                .and_then(|value| value.as_str())
        })
        .filter(|value| !value.is_empty() && *value != "[0][0][0][0]")
        .map(ToString::to_string)
        .collect();

    Ok(FactsPatchPayload {
        duration_ms,
        media_format,
        video_codec,
        audio_codec,
        width,
        height,
        fps,
        captured_at,
        camera_make,
        camera_model,
        bitrate_kbps,
        sample_rate_hz,
        channel_count,
        bits_per_sample,
        rotation_deg,
        timecode_start,
        pixel_format,
        color_range,
        color_space,
        color_transfer,
        color_primaries,
        recorder_model,
        has_dji_metadata_track: (!dji_metadata_track_types.is_empty()).then_some(true),
        dji_metadata_track_types: (!dji_metadata_track_types.is_empty())
            .then_some(dji_metadata_track_types),
        ..FactsPatchPayload::default()
    })
}

fn parse_stream_fps(stream: &serde_json::Value) -> Option<f64> {
    let fps = stream
        .get("avg_frame_rate")
        .or_else(|| stream.get("r_frame_rate"))
        .and_then(|value| value.as_str())?;
    let (num, den) = fps.split_once('/')?;
    let numerator = num.parse::<f64>().ok()?;
    let denominator = den.parse::<f64>().ok()?;
    if denominator == 0.0 {
        return None;
    }
    Some(numerator / denominator)
}

fn merge_wav_container_facts(
    input_path: &str,
    facts: &mut FactsPatchPayload,
    timestamp_provider: &dyn FileTimestampProvider,
) -> Result<(), ProxyGenerationError> {
    let is_wav = facts
        .media_format
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("wav"))
        .unwrap_or(false);
    if !is_wav {
        return Ok(());
    }

    let Some(parsed) = parse_wav_container_facts(Path::new(input_path), timestamp_provider)? else {
        return Ok(());
    };
    facts.recorder_model = facts.recorder_model.take().or(parsed.recorder_model);
    facts.captured_at = parsed.captured_at.or_else(|| facts.captured_at.take());
    Ok(())
}

#[derive(Debug, Default)]
struct ParsedWavFacts {
    recorder_model: Option<String>,
    captured_at: Option<String>,
}

#[derive(Debug, Default)]
struct BextChunkData {
    originator: Option<String>,
    origination_date: Option<String>,
    origination_time: Option<String>,
}

#[derive(Debug, Default)]
struct IxmlChunkData {
    timestamp_samples_since_midnight: Option<u64>,
    timestamp_sample_rate: Option<u32>,
}

fn parse_wav_container_facts(
    path: &Path,
    timestamp_provider: &dyn FileTimestampProvider,
) -> Result<Option<ParsedWavFacts>, ProxyGenerationError> {
    let bytes = fs::read(path).map_err(|error| ProxyGenerationError::Process(error.to_string()))?;
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Ok(None);
    }

    let mut offset = 12usize;
    let mut bext = None;
    let mut ixml = None;
    while offset + 8 <= bytes.len() {
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_size = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .expect("slice size"),
        ) as usize;
        let data_start = offset + 8;
        let data_end = data_start.saturating_add(chunk_size).min(bytes.len());
        if data_end > bytes.len() || data_start > bytes.len() {
            break;
        }
        let chunk = &bytes[data_start..data_end];
        match chunk_id {
            b"bext" => bext = Some(parse_bext_chunk(chunk)),
            b"iXML" => ixml = Some(parse_ixml_chunk(chunk)),
            _ => {}
        }
        offset = data_end + (chunk_size % 2);
    }

    let created_at = timestamp_provider.created_at_utc(path);
    let modified_at = timestamp_provider.modified_at_utc(path);

    let recorder_model = bext
        .as_ref()
        .and_then(|bext| bext.originator.clone())
        .filter(|value| !value.is_empty());
    let captured_at = repaired_bext_captured_at(bext.as_ref(), ixml.as_ref(), modified_at.as_ref())
        .or_else(|| created_at.as_ref().map(system_time_to_rfc3339))
        .or_else(|| modified_at.as_ref().map(system_time_to_rfc3339));

    if recorder_model.is_none() && captured_at.is_none() {
        return Ok(None);
    }

    Ok(Some(ParsedWavFacts {
        recorder_model,
        captured_at,
    }))
}

fn parse_bext_chunk(chunk: &[u8]) -> BextChunkData {
    if chunk.len() < 338 {
        return BextChunkData::default();
    }
    BextChunkData {
        originator: read_bext_string(&chunk[256..288]),
        origination_date: read_bext_string(&chunk[320..330]),
        origination_time: read_bext_string(&chunk[330..338]),
    }
}

fn parse_ixml_chunk(chunk: &[u8]) -> IxmlChunkData {
    let xml = String::from_utf8_lossy(chunk);
    let timestamp_hi = xml_tag_value(&xml, "TIMESTAMP_SAMPLES_SINCE_MIDNIGHT_HI")
        .and_then(|value| value.parse::<u64>().ok());
    let timestamp_lo = xml_tag_value(&xml, "TIMESTAMP_SAMPLES_SINCE_MIDNIGHT_LO")
        .and_then(|value| value.parse::<u64>().ok());
    IxmlChunkData {
        timestamp_samples_since_midnight: xml_tag_value(&xml, "TIMESTAMP_SAMPLES_SINCE_MIDNIGHT")
            .and_then(|value| value.parse::<u64>().ok())
            .or_else(|| match (timestamp_hi, timestamp_lo) {
                (Some(hi), Some(lo)) => Some((hi << 32) | lo),
                _ => None,
            }),
        timestamp_sample_rate: xml_tag_value(&xml, "TIMESTAMP_SAMPLE_RATE")
            .and_then(|value| value.parse::<u32>().ok()),
    }
}

fn parse_format_tags_captured_at(tags: &serde_json::Value) -> Option<String> {
    let creation_time = tags.get("creation_time").and_then(|value| value.as_str());
    let date = tags.get("date").and_then(|value| value.as_str());
    if let Some(value) = creation_time {
        if looks_like_iso_datetime(value) {
            return Some(value.to_string());
        }
    }
    match (date, creation_time) {
        (Some(date), Some(time)) => repaired_ffprobe_date_time(date, time),
        _ => None,
    }
}

fn looks_like_iso_datetime(value: &str) -> bool {
    value.contains('T') && (value.ends_with('Z') || value.contains('+'))
}

fn repaired_ffprobe_date_time(date: &str, time: &str) -> Option<String> {
    let (year, month, day) = parse_bext_date(date)?;
    if year < 1000 {
        return None;
    }
    let time = parse_bext_time(time)?;
    let datetime = NaiveDateTime::new(NaiveDate::from_ymd_opt(year, month, day)?, time);
    Some(
        Utc.from_utc_datetime(&datetime)
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    )
}

fn system_time_to_rfc3339(value: &chrono::DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn read_bext_string(bytes: &[u8]) -> Option<String> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let value = String::from_utf8_lossy(&bytes[..end]).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn xml_tag_value<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let start = xml.find(&start_tag)? + start_tag.len();
    let end = xml[start..].find(&end_tag)? + start;
    Some(xml[start..end].trim())
}

fn repaired_bext_captured_at(
    bext: Option<&BextChunkData>,
    ixml: Option<&IxmlChunkData>,
    modified_at: Option<&chrono::DateTime<Utc>>,
) -> Option<String> {
    let bext = bext?;
    let date = bext.origination_date.as_deref()?;
    let time = bext.origination_time.as_deref()?;
    let (mut year, month, day) = parse_bext_date(date)?;

    if year < 1000 {
        let modified_at = modified_at?;
        if modified_at.month() != month || modified_at.day() != day {
            return None;
        }
        year = modified_at.year();
    }

    let base_time = parse_bext_time(time)?;
    if let (Some(samples), Some(rate)) = (
        ixml.and_then(|value| value.timestamp_samples_since_midnight),
        ixml.and_then(|value| value.timestamp_sample_rate),
    ) {
        if rate != 0 {
            let total_seconds = samples as f64 / rate as f64;
            let ixml_seconds = total_seconds % 86_400.0;
            let bext_seconds = base_time.num_seconds_from_midnight() as f64;
            if (ixml_seconds - bext_seconds).abs() > 2.0 {
                return None;
            }
        }
    }

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let datetime = NaiveDateTime::new(date, base_time);
    Some(
        Utc.from_utc_datetime(&datetime)
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    )
}

fn parse_bext_date(value: &str) -> Option<(i32, u32, u32)> {
    let mut parts = value.split('-');
    let year = parts.next()?.trim().parse::<i32>().ok()?;
    let month = parts.next()?.trim().parse::<u32>().ok()?;
    let day = parts.next()?.trim().parse::<u32>().ok()?;
    Some((year, month, day))
}

fn parse_bext_time(value: &str) -> Option<NaiveTime> {
    let mut parts = value.split(':');
    let hour = parts.next()?.trim().parse::<u32>().ok()?;
    let minute = parts.next()?.trim().parse::<u32>().ok()?;
    let second = parts.next()?.trim().parse::<u32>().ok()?;
    NaiveTime::from_hms_opt(hour, minute, second)
}

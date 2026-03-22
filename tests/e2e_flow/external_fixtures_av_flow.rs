use retaia_agent::{
    AudioProxyFormat, AudioProxyRequest, AudioWaveformRequest, FfmpegProxyGenerator,
    ProxyGenerator, ThumbnailFormat, VideoProxyRequest, VideoThumbnailRequest, ffmpeg_available,
};

use crate::external_fixtures::load_manifest_entries;

#[test]
fn e2e_external_fixture_flow_manifest_contains_supported_audio_and_video_entries() {
    let entries = load_manifest_entries();
    let audio = entries
        .iter()
        .filter(|entry| entry.kind == "preview_audio" && entry.expected == "supported")
        .count();
    let video = entries
        .iter()
        .filter(|entry| entry.kind == "preview_video" && entry.expected == "supported")
        .count();
    assert!(audio > 0, "expected supported audio fixtures in manifest");
    assert!(video > 0, "expected supported video fixtures in manifest");
}

#[test]
fn e2e_external_fixture_flow_generates_audio_video_proxies_with_ffmpeg_when_available() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping external AV fixture transcoding test");
        return;
    }

    let entries = load_manifest_entries();
    let audio_entries: Vec<_> = entries
        .iter()
        .filter(|entry| entry.kind == "preview_audio" && entry.expected == "supported")
        .collect();
    let video_entries: Vec<_> = entries
        .iter()
        .filter(|entry| entry.kind == "preview_video" && entry.expected == "supported")
        .collect();
    assert!(!audio_entries.is_empty(), "missing supported audio entries");
    assert!(!video_entries.is_empty(), "missing supported video entries");

    let temp = tempfile::tempdir().expect("tempdir");
    let generator = FfmpegProxyGenerator::default();

    for (index, entry) in audio_entries.iter().enumerate() {
        let output = temp.path().join(format!("audio-proxy-{index}.m4a"));
        generator
            .generate_audio_proxy(&AudioProxyRequest {
                input_path: entry.absolute_path().display().to_string(),
                output_path: output.display().to_string(),
                format: AudioProxyFormat::Mp4Aac,
                audio_bitrate_kbps: 128,
                sample_rate_hz: 44100,
            })
            .unwrap_or_else(|error| {
                panic!(
                    "audio fixture should generate proxy: {} ({error:?})",
                    entry.relative_path
                )
            });
        let metadata = std::fs::metadata(&output).expect("audio output metadata");
        assert!(
            metadata.len() > 0,
            "audio output should be non-empty for {}",
            entry.relative_path
        );
    }

    for (index, entry) in video_entries.iter().enumerate() {
        let output = temp.path().join(format!("video-proxy-{index}.mp4"));
        generator
            .generate_video_proxy(&VideoProxyRequest {
                input_path: entry.absolute_path().display().to_string(),
                output_path: output.display().to_string(),
                max_width: 640,
                max_height: 360,
                video_bitrate_kbps: 1200,
                audio_bitrate_kbps: 96,
            })
            .unwrap_or_else(|error| {
                panic!(
                    "video fixture should generate proxy: {} ({error:?})",
                    entry.relative_path
                )
            });
        let metadata = std::fs::metadata(&output).expect("video output metadata");
        assert!(
            metadata.len() > 0,
            "video output should be non-empty for {}",
            entry.relative_path
        );
    }
}

#[test]
fn e2e_external_fixture_flow_generates_video_thumbnail_with_ffmpeg_when_available() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping external AV fixture thumbnail test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.kind == "preview_video" && entry.expected == "supported")
        .expect("missing supported video fixture");

    let temp = tempfile::tempdir().expect("tempdir");
    let output = temp.path().join("video-thumb.jpg");
    let generator = FfmpegProxyGenerator::default();

    generator
        .generate_video_thumbnail(&VideoThumbnailRequest {
            input_path: entry.absolute_path().display().to_string(),
            output_path: output.display().to_string(),
            format: ThumbnailFormat::Jpeg,
            max_width: 480,
            seek_ms: 1_000,
        })
        .unwrap_or_else(|error| {
            panic!(
                "video fixture should generate thumbnail: {} ({error:?})",
                entry.relative_path
            )
        });

    let metadata = std::fs::metadata(&output).expect("thumbnail output metadata");
    assert!(metadata.len() > 0, "thumbnail output should be non-empty");
}

#[test]
fn e2e_external_fixture_flow_generates_audio_waveform_with_ffmpeg_when_available() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping external AV fixture waveform test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.kind == "preview_audio" && entry.expected == "supported")
        .expect("missing supported audio fixture");

    let temp = tempfile::tempdir().expect("tempdir");
    let output = temp.path().join("audio-waveform.json");
    let generator = FfmpegProxyGenerator::default();

    generator
        .generate_audio_waveform(&AudioWaveformRequest {
            input_path: entry.absolute_path().display().to_string(),
            output_path: output.display().to_string(),
            bucket_count: 1000,
        })
        .unwrap_or_else(|error| {
            panic!(
                "audio fixture should generate waveform: {} ({error:?})",
                entry.relative_path
            )
        });

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).expect("read waveform json"))
            .expect("waveform json payload");
    assert_eq!(payload.get("bucket_count"), Some(&serde_json::json!(1000)));
    assert!(
        payload
            .get("duration_ms")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_default()
            > 0,
        "waveform duration should be positive"
    );
    assert_eq!(
        payload
            .get("samples")
            .and_then(serde_json::Value::as_array)
            .map(|samples| samples.len()),
        Some(1000)
    );
}

#[test]
fn e2e_external_fixture_flow_extracts_audio_facts_with_ffprobe_when_available() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping external AV fixture audio facts test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.kind == "preview_audio" && entry.expected == "supported")
        .expect("missing supported audio fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "audio fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert!(facts.duration_ms.unwrap_or_default() > 0);
    assert!(facts.media_format.as_deref().is_some());
    assert!(facts.audio_codec.as_deref().is_some());
}

#[test]
fn e2e_external_fixture_flow_extracts_expected_facts_from_wireless_pro_wav() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping Wireless PRO audio facts test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "audio/wav/sample_Wireless_PRO.WAV")
        .expect("missing Wireless PRO wav fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "Wireless PRO wav fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert_eq!(facts.media_format.as_deref(), Some("wav"));
    assert_eq!(facts.audio_codec.as_deref(), Some("pcm_f32le"));
    assert_eq!(facts.sample_rate_hz, Some(48_000));
    assert_eq!(facts.channel_count, Some(1));
    assert_eq!(facts.bits_per_sample, Some(32));
    assert_eq!(facts.bitrate_kbps, Some(1536));
    assert_eq!(facts.duration_ms, Some(10_401));
    assert_eq!(facts.recorder_model, None);
    assert_eq!(facts.captured_at, None);
}

#[test]
fn e2e_external_fixture_flow_wireless_pro_app_export_matches_standard_audio_facts() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping Wireless PRO app export audio facts test");
        return;
    }

    let entries = load_manifest_entries();
    let raw_entry = entries
        .iter()
        .find(|entry| entry.relative_path == "audio/wav/sample_Wireless_PRO.WAV")
        .expect("missing Wireless PRO wav fixture");
    let app_entry = entries
        .iter()
        .find(|entry| entry.relative_path == "audio/wav/sample_Wireless_PRO_app.wav")
        .expect("missing Wireless PRO app export fixture");

    let generator = FfmpegProxyGenerator::default();
    let raw_facts = generator
        .extract_media_facts(&raw_entry.absolute_path().display().to_string())
        .expect("raw Wireless PRO facts");
    let app_facts = generator
        .extract_media_facts(&app_entry.absolute_path().display().to_string())
        .expect("app export Wireless PRO facts");

    assert_eq!(app_facts.media_format, raw_facts.media_format);
    assert_eq!(app_facts.audio_codec, raw_facts.audio_codec);
    assert_eq!(app_facts.sample_rate_hz, raw_facts.sample_rate_hz);
    assert_eq!(app_facts.channel_count, raw_facts.channel_count);
    assert_eq!(app_facts.bits_per_sample, raw_facts.bits_per_sample);
    assert_eq!(app_facts.duration_ms, raw_facts.duration_ms);
    assert_eq!(app_facts.recorder_model, None);
    assert_eq!(app_facts.captured_at, None);
}

#[test]
fn e2e_external_fixture_flow_extracts_video_facts_with_ffprobe_when_available() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping external AV fixture video facts test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.kind == "preview_video" && entry.expected == "supported")
        .expect("missing supported video fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "video fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert!(facts.duration_ms.unwrap_or_default() > 0);
    assert!(facts.media_format.as_deref().is_some());
    assert!(facts.video_codec.as_deref().is_some());
    assert!(facts.width.unwrap_or_default() > 0);
    assert!(facts.height.unwrap_or_default() > 0);
    assert!(facts.fps.unwrap_or_default() > 0.0);
}

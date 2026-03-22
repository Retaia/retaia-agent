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
fn e2e_external_fixture_flow_extracts_expected_aac_audio_metadata() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping aac audio metadata fixture test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "audio/aac/sample1.aac")
        .expect("missing aac fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "aac fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert_eq!(facts.media_format.as_deref(), Some("aac"));
    assert_eq!(facts.audio_codec.as_deref(), Some("aac"));
    assert_eq!(facts.sample_rate_hz, Some(44_100));
    assert_eq!(facts.channel_count, Some(2));
    assert_eq!(facts.bits_per_sample, Some(0));
    assert_eq!(facts.bitrate_kbps, Some(127));
    assert_eq!(facts.duration_ms, Some(128_554));
    assert_eq!(facts.captured_at, None);
    assert_eq!(facts.recorder_model, None);
}

#[test]
fn e2e_external_fixture_flow_extracts_expected_flac_audio_metadata() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping flac audio metadata fixture test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "audio/flac/sample1.flac")
        .expect("missing flac fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "flac fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert_eq!(facts.media_format.as_deref(), Some("flac"));
    assert_eq!(facts.audio_codec.as_deref(), Some("flac"));
    assert_eq!(facts.sample_rate_hz, Some(44_100));
    assert_eq!(facts.channel_count, Some(2));
    assert_eq!(facts.bits_per_sample, Some(0));
    assert_eq!(facts.bitrate_kbps, None);
    assert_eq!(facts.duration_ms, Some(122_094));
    assert_eq!(facts.captured_at, None);
    assert_eq!(facts.recorder_model.as_deref(), Some("Lavf57.83.100"));
}

#[test]
fn e2e_external_fixture_flow_extracts_expected_mp3_audio_metadata() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping mp3 audio metadata fixture test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "audio/mp3/sample1.mp3")
        .expect("missing mp3 fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "mp3 fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert_eq!(facts.media_format.as_deref(), Some("mp3"));
    assert_eq!(facts.audio_codec.as_deref(), Some("mp3"));
    assert_eq!(facts.sample_rate_hz, Some(44_100));
    assert_eq!(facts.channel_count, Some(2));
    assert_eq!(facts.bits_per_sample, Some(0));
    assert_eq!(facts.bitrate_kbps, Some(128));
    assert_eq!(facts.duration_ms, Some(122_094));
    assert_eq!(facts.captured_at, None);
    assert_eq!(facts.recorder_model.as_deref(), Some("Lavf57.83.100"));
}

#[test]
fn e2e_external_fixture_flow_extracts_expected_pcm_wav_audio_metadata() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping wav audio metadata fixture test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "audio/wav/sample1.wav")
        .expect("missing wav fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "wav fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert_eq!(facts.media_format.as_deref(), Some("wav"));
    assert_eq!(facts.audio_codec.as_deref(), Some("pcm_s16le"));
    assert_eq!(facts.sample_rate_hz, Some(44_100));
    assert_eq!(facts.channel_count, Some(2));
    assert_eq!(facts.bits_per_sample, Some(16));
    assert_eq!(facts.bitrate_kbps, Some(1411));
    assert_eq!(facts.duration_ms, Some(122_094));
    assert!(facts.captured_at.is_some());
    assert_eq!(facts.recorder_model.as_deref(), Some("Lavf57.83.100"));
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
    assert_eq!(facts.captured_at.as_deref(), Some("2026-03-22T18:17:28Z"));
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
    assert_eq!(
        app_facts.captured_at.as_deref(),
        Some("2026-03-22T18:22:23Z")
    );
}

#[test]
fn e2e_external_fixture_flow_extracts_repaired_facts_from_wireless_pro_receiver_wav() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping Wireless PRO receiver audio facts test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "audio/wav/sample_Wireless_PRO_receiver.WAV")
        .expect("missing Wireless PRO receiver wav fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "Wireless PRO receiver wav fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert_eq!(facts.media_format.as_deref(), Some("wav"));
    assert_eq!(facts.audio_codec.as_deref(), Some("pcm_f32le"));
    assert_eq!(facts.sample_rate_hz, Some(48_000));
    assert_eq!(facts.channel_count, Some(1));
    assert_eq!(facts.bits_per_sample, Some(32));
    assert_eq!(facts.duration_ms, Some(5_388));
    assert_eq!(facts.recorder_model.as_deref(), Some("RODE Wireless PRO"));
    assert_eq!(facts.captured_at.as_deref(), Some("2026-03-22T19:25:31Z"));
}

#[test]
fn e2e_external_fixture_flow_wireless_pro_receiver_app_export_matches_repaired_facts() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping Wireless PRO receiver app audio facts test");
        return;
    }

    let entries = load_manifest_entries();
    let raw_entry = entries
        .iter()
        .find(|entry| entry.relative_path == "audio/wav/sample_Wireless_PRO_receiver.WAV")
        .expect("missing Wireless PRO receiver wav fixture");
    let app_entry = entries
        .iter()
        .find(|entry| entry.relative_path == "audio/wav/sample_Wireless_PRO_receiver_app.wav")
        .expect("missing Wireless PRO receiver app export fixture");

    let generator = FfmpegProxyGenerator::default();
    let raw_facts = generator
        .extract_media_facts(&raw_entry.absolute_path().display().to_string())
        .expect("raw Wireless PRO receiver facts");
    let app_facts = generator
        .extract_media_facts(&app_entry.absolute_path().display().to_string())
        .expect("app export Wireless PRO receiver facts");

    assert_eq!(app_facts.audio_codec, raw_facts.audio_codec);
    assert_eq!(app_facts.sample_rate_hz, raw_facts.sample_rate_hz);
    assert_eq!(app_facts.channel_count, raw_facts.channel_count);
    assert_eq!(app_facts.bits_per_sample, raw_facts.bits_per_sample);
    assert_eq!(app_facts.duration_ms, raw_facts.duration_ms);
    assert_eq!(
        app_facts.recorder_model.as_deref(),
        Some("RODE Wireless PRO")
    );
    assert_eq!(
        app_facts.captured_at.as_deref(),
        Some("2026-03-22T19:25:31Z")
    );
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

#[test]
fn e2e_external_fixture_flow_extracts_expected_h264_video_metadata() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping h264 video metadata fixture test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "video/h264/sample-h264.mp4")
        .expect("missing h264 fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "h264 fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert_eq!(facts.media_format.as_deref(), Some("mov"));
    assert_eq!(facts.video_codec.as_deref(), Some("h264"));
    assert_eq!(facts.audio_codec.as_deref(), Some("aac"));
    assert_eq!(facts.width, Some(1920));
    assert_eq!(facts.height, Some(1080));
    assert_eq!(facts.fps, Some(25.0));
    assert_eq!(
        facts.captured_at.as_deref(),
        Some("2026-02-24T15:25:36.000000Z")
    );
    assert_eq!(facts.timecode_start.as_deref(), Some("01:00:00:00"));
    assert_eq!(facts.sample_rate_hz, Some(48_000));
    assert_eq!(facts.channel_count, Some(2));
    assert_eq!(facts.pixel_format.as_deref(), Some("yuv420p"));
    assert_eq!(facts.color_range.as_deref(), Some("tv"));
    assert_eq!(facts.color_space.as_deref(), Some("bt709"));
    assert_eq!(facts.color_transfer.as_deref(), Some("bt709"));
    assert_eq!(facts.color_primaries.as_deref(), Some("bt709"));
    assert_eq!(facts.has_dji_metadata_track, Some(true));
    assert_eq!(
        facts.dji_metadata_track_types,
        Some(vec!["tmcd".to_string()])
    );
}

#[test]
fn e2e_external_fixture_flow_extracts_expected_h265_video_metadata() {
    if !ffmpeg_available() {
        eprintln!("ffmpeg not available, skipping h265 video metadata fixture test");
        return;
    }

    let entry = load_manifest_entries()
        .into_iter()
        .find(|entry| entry.relative_path == "video/h265/sample-h265.mp4")
        .expect("missing h265 fixture");

    let facts = FfmpegProxyGenerator::default()
        .extract_media_facts(&entry.absolute_path().display().to_string())
        .unwrap_or_else(|error| {
            panic!(
                "h265 fixture should expose facts: {} ({error:?})",
                entry.relative_path
            )
        });

    assert_eq!(facts.media_format.as_deref(), Some("mov"));
    assert_eq!(facts.video_codec.as_deref(), Some("hevc"));
    assert_eq!(facts.audio_codec.as_deref(), Some("aac"));
    assert_eq!(facts.width, Some(1920));
    assert_eq!(facts.height, Some(1080));
    assert_eq!(facts.fps, Some(25.0));
    assert_eq!(
        facts.captured_at.as_deref(),
        Some("2026-02-24T15:25:38.000000Z")
    );
    assert_eq!(facts.timecode_start.as_deref(), Some("01:00:00:00"));
    assert_eq!(facts.sample_rate_hz, Some(48_000));
    assert_eq!(facts.channel_count, Some(2));
    assert_eq!(facts.pixel_format.as_deref(), Some("yuv420p10le"));
    assert_eq!(facts.color_range.as_deref(), Some("tv"));
    assert_eq!(facts.color_space.as_deref(), Some("bt709"));
    assert_eq!(facts.color_transfer.as_deref(), Some("bt709"));
    assert_eq!(facts.color_primaries.as_deref(), Some("bt709"));
    assert_eq!(facts.has_dji_metadata_track, Some(true));
    assert_eq!(
        facts.dji_metadata_track_types,
        Some(vec!["tmcd".to_string()])
    );
}

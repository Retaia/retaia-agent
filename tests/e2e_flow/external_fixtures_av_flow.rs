use retaia_agent::{
    AudioProxyFormat, AudioProxyRequest, FfmpegProxyGenerator, ProxyGenerator, VideoProxyRequest,
    ffmpeg_available,
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

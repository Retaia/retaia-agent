use std::collections::BTreeSet;

use retaia_agent::{
    AgentCapability, declared_agent_capabilities, declared_agent_capabilities_with_ffmpeg,
    ffmpeg_available, has_required_capabilities, photo_source_extension_supported,
};

#[test]
fn tdd_first_agent_capability_is_media_facts_v1() {
    assert_eq!(AgentCapability::MediaFactsV1.as_str(), "media.facts@1");
}

#[test]
fn tdd_declared_agent_capabilities_contains_v1_processing_capability_set() {
    let declared = declared_agent_capabilities();
    let expected_base = BTreeSet::from([
        "audio.waveform@1".to_string(),
        "media.facts@1".to_string(),
        "media.thumbnails@1".to_string(),
    ]);
    assert!(expected_base.is_subset(&declared));

    let proxy_caps = BTreeSet::from([
        "media.proxies.audio@1".to_string(),
        "media.proxies.photo@1".to_string(),
        "media.proxies.video@1".to_string(),
    ]);
    if ffmpeg_available() {
        assert!(proxy_caps.is_subset(&declared));
    } else {
        assert!(!declared.contains("media.proxies.video@1"));
        assert!(!declared.contains("media.proxies.audio@1"));
        assert!(declared.contains("media.proxies.photo@1"));
    }
}

#[test]
fn tdd_declared_agent_capabilities_without_ffmpeg_excludes_only_audio_video_proxies() {
    let declared = declared_agent_capabilities_with_ffmpeg(false);
    assert!(declared.contains("media.facts@1"));
    assert!(declared.contains("media.thumbnails@1"));
    assert!(!declared.contains("media.proxies.video@1"));
    assert!(!declared.contains("media.proxies.audio@1"));
    assert!(declared.contains("media.proxies.photo@1"));
}

#[test]
fn tdd_photo_source_extension_support_covers_standard_and_camera_raw_formats() {
    for extension in [
        "jpeg", "jpg", "png", "dng", "tiff", "cr2", "cr3", "arw", "nef",
    ] {
        assert!(
            photo_source_extension_supported(extension),
            "{extension} should be supported"
        );
    }

    assert!(!photo_source_extension_supported("gif"));
    assert!(!photo_source_extension_supported("bmp"));
    assert!(!photo_source_extension_supported("wav"));
}

#[test]
fn tdd_has_required_capabilities_checks_subset_relation() {
    let declared = BTreeSet::from([
        "media.facts@1".to_string(),
        "media.thumbnails@1".to_string(),
    ]);
    assert!(has_required_capabilities(
        &["media.facts@1".to_string()],
        &declared
    ));
    assert!(!has_required_capabilities(
        &["media.proxies.video@1".to_string()],
        &declared
    ));
}

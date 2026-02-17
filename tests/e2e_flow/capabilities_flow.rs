use retaia_agent::{
    declared_agent_capabilities_with_ffmpeg, declared_agent_capabilities_with_runtime,
    photo_source_extension_supported,
};

#[test]
fn e2e_capabilities_flow_ffmpeg_presence_toggles_only_audio_video_proxy_capabilities() {
    let without_ffmpeg = declared_agent_capabilities_with_ffmpeg(false);
    assert!(without_ffmpeg.contains("media.facts@1"));
    assert!(!without_ffmpeg.contains("media.proxies.video@1"));
    assert!(without_ffmpeg.contains("media.proxies.photo@1"));

    let with_ffmpeg = declared_agent_capabilities_with_ffmpeg(true);
    assert!(with_ffmpeg.contains("media.proxies.video@1"));
    assert!(with_ffmpeg.contains("media.proxies.audio@1"));
    assert!(with_ffmpeg.contains("media.proxies.photo@1"));
}

#[test]
fn e2e_capabilities_flow_runtime_flags_can_disable_photo_proxy_independently() {
    let declared = declared_agent_capabilities_with_runtime(true, false);
    assert!(declared.contains("media.facts@1"));
    assert!(declared.contains("media.proxies.video@1"));
    assert!(declared.contains("media.proxies.audio@1"));
    assert!(!declared.contains("media.proxies.photo@1"));
}

#[test]
fn e2e_capabilities_flow_photo_extension_matrix_covers_supported_raw_inputs() {
    for extension in [
        "jpg", "png", "tiff", "dng", "crw", "cr2", "cr3", "arw", "srf", "sr2", "nef", "nrw", "orf",
        "rw2", "raf", "pef", "dcr", "kdc", "erf", "3fr", "iiq", "mos", "raw", "rwl", "mrw", "x3f",
    ] {
        assert!(
            photo_source_extension_supported(extension),
            "{extension} should be supported"
        );
    }

    assert!(!photo_source_extension_supported(""));
    assert!(!photo_source_extension_supported("."));
    assert!(!photo_source_extension_supported("gif"));
}

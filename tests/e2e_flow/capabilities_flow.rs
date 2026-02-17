use retaia_agent::declared_agent_capabilities_with_ffmpeg;

#[test]
fn e2e_capabilities_flow_ffmpeg_presence_toggles_proxy_capability_declaration() {
    let without_ffmpeg = declared_agent_capabilities_with_ffmpeg(false);
    assert!(without_ffmpeg.contains("media.facts@1"));
    assert!(!without_ffmpeg.contains("media.proxies.video@1"));

    let with_ffmpeg = declared_agent_capabilities_with_ffmpeg(true);
    assert!(with_ffmpeg.contains("media.proxies.video@1"));
    assert!(with_ffmpeg.contains("media.proxies.audio@1"));
    assert!(with_ffmpeg.contains("media.proxies.photo@1"));
}

use retaia_agent::declared_agent_capabilities_with_ffmpeg;

#[test]
fn bdd_given_ffmpeg_missing_when_declaring_capabilities_then_proxy_capabilities_are_not_declared() {
    let declared = declared_agent_capabilities_with_ffmpeg(false);
    assert!(!declared.contains("media.proxies.video@1"));
    assert!(!declared.contains("media.proxies.audio@1"));
    assert!(!declared.contains("media.proxies.photo@1"));
}

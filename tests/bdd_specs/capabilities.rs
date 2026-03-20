use retaia_agent::declared_agent_capabilities_with_ffmpeg;

#[test]
fn bdd_given_ffmpeg_missing_when_declaring_capabilities_then_photo_proxy_stays_available() {
    let declared = declared_agent_capabilities_with_ffmpeg(false);
    assert!(!declared.contains("media.previews.video@1"));
    assert!(!declared.contains("media.previews.audio@1"));
    assert!(declared.contains("media.previews.photo@1"));
}

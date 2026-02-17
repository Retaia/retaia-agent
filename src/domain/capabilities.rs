use std::collections::BTreeSet;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCapability {
    MediaFactsV1,
    MediaProxiesVideoV1,
    MediaProxiesAudioV1,
    MediaProxiesPhotoV1,
    MediaThumbnailsV1,
    AudioWaveformV1,
}

impl AgentCapability {
    pub const fn as_str(self) -> &'static str {
        match self {
            AgentCapability::MediaFactsV1 => "media.facts@1",
            AgentCapability::MediaProxiesVideoV1 => "media.proxies.video@1",
            AgentCapability::MediaProxiesAudioV1 => "media.proxies.audio@1",
            AgentCapability::MediaProxiesPhotoV1 => "media.proxies.photo@1",
            AgentCapability::MediaThumbnailsV1 => "media.thumbnails@1",
            AgentCapability::AudioWaveformV1 => "audio.waveform@1",
        }
    }
}

pub fn declared_agent_capabilities() -> BTreeSet<String> {
    declared_agent_capabilities_with_ffmpeg(ffmpeg_available())
}

pub fn declared_agent_capabilities_with_ffmpeg(ffmpeg_is_available: bool) -> BTreeSet<String> {
    let mut capabilities = vec![
        AgentCapability::MediaFactsV1,
        AgentCapability::MediaThumbnailsV1,
        AgentCapability::AudioWaveformV1,
    ];

    if ffmpeg_is_available {
        capabilities.push(AgentCapability::MediaProxiesVideoV1);
        capabilities.push(AgentCapability::MediaProxiesAudioV1);
        capabilities.push(AgentCapability::MediaProxiesPhotoV1);
    }

    capabilities
        .into_iter()
        .map(|capability| capability.as_str().to_string())
        .collect()
}

pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn has_required_capabilities(
    required_capabilities: &[String],
    declared_capabilities: &BTreeSet<String>,
) -> bool {
    required_capabilities
        .iter()
        .all(|required| declared_capabilities.contains(required))
}

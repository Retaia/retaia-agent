use std::collections::BTreeSet;

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
    [
        AgentCapability::MediaFactsV1,
        AgentCapability::MediaProxiesVideoV1,
        AgentCapability::MediaProxiesAudioV1,
        AgentCapability::MediaProxiesPhotoV1,
        AgentCapability::MediaThumbnailsV1,
        AgentCapability::AudioWaveformV1,
    ]
    .into_iter()
    .map(|capability| capability.as_str().to_string())
    .collect()
}

pub fn has_required_capabilities(
    required_capabilities: &[String],
    declared_capabilities: &BTreeSet<String>,
) -> bool {
    required_capabilities
        .iter()
        .all(|required| declared_capabilities.contains(required))
}

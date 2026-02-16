use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCapability {
    MediaFactsV1,
}

impl AgentCapability {
    pub const fn as_str(self) -> &'static str {
        match self {
            AgentCapability::MediaFactsV1 => "media.facts@1",
        }
    }
}

pub fn declared_agent_capabilities() -> BTreeSet<String> {
    [AgentCapability::MediaFactsV1.as_str().to_string()]
        .into_iter()
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

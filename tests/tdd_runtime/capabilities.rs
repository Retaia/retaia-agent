use std::collections::BTreeSet;

use retaia_agent::{AgentCapability, declared_agent_capabilities, has_required_capabilities};

#[test]
fn tdd_first_agent_capability_is_media_facts_v1() {
    assert_eq!(AgentCapability::MediaFactsV1.as_str(), "media.facts@1");
}

#[test]
fn tdd_declared_agent_capabilities_contains_v1_processing_capability_set() {
    let declared = declared_agent_capabilities();
    let expected = BTreeSet::from([
        "audio.waveform@1".to_string(),
        "media.facts@1".to_string(),
        "media.proxies.audio@1".to_string(),
        "media.proxies.photo@1".to_string(),
        "media.proxies.video@1".to_string(),
        "media.thumbnails@1".to_string(),
    ]);
    assert_eq!(declared, expected);
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

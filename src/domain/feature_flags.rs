use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientKind {
    Agent,
    UiWeb,
    UiMobile,
}

pub const CORE_POLICY_RUNTIME_FEATURE: &str = "features.core.policy.runtime";
pub const CORE_JOBS_RUNTIME_FEATURE: &str = "features.core.jobs.runtime";
pub const CORE_DERIVED_ACCESS_FEATURE: &str = "features.core.derived.access";
pub const CORE_CLIENTS_BOOTSTRAP_FEATURE: &str = "features.core.clients.bootstrap";
pub const CORE_AUTH_FEATURE: &str = "features.core.auth";
pub const CORE_ASSETS_LIFECYCLE_FEATURE: &str = "features.core.assets.lifecycle";
pub const CORE_SEARCH_QUERY_FEATURE: &str = "features.core.search.query";

pub fn core_v1_global_features() -> BTreeSet<String> {
    [
        CORE_AUTH_FEATURE,
        CORE_ASSETS_LIFECYCLE_FEATURE,
        CORE_JOBS_RUNTIME_FEATURE,
        CORE_SEARCH_QUERY_FEATURE,
        CORE_POLICY_RUNTIME_FEATURE,
        CORE_DERIVED_ACCESS_FEATURE,
        CORE_CLIENTS_BOOTSTRAP_FEATURE,
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

pub fn resolve_effective_features(
    feature_flags: &BTreeMap<String, bool>,
    app_feature_enabled: &BTreeMap<String, bool>,
    user_feature_enabled: &BTreeMap<String, bool>,
    dependencies: &BTreeMap<String, Vec<String>>,
    disable_escalation: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, bool> {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    keys.extend(feature_flags.keys().cloned());
    keys.extend(app_feature_enabled.keys().cloned());
    keys.extend(user_feature_enabled.keys().cloned());
    keys.extend(dependencies.keys().cloned());
    keys.extend(dependencies.values().flat_map(|deps| deps.iter().cloned()));
    keys.extend(disable_escalation.keys().cloned());
    keys.extend(
        disable_escalation
            .values()
            .flat_map(|children| children.iter().cloned()),
    );
    keys.extend(core_v1_global_features());

    let mut effective: BTreeMap<String, bool> = BTreeMap::new();
    for key in keys {
        let runtime_flag = feature_flags.get(&key).copied().unwrap_or(false);
        let app_value = app_feature_enabled.get(&key).copied().unwrap_or(true);
        // Spec invariant: if a key is absent in user preferences, it is treated as true.
        let user_value = user_feature_enabled.get(&key).copied().unwrap_or(true);
        effective.insert(key, runtime_flag && app_value && user_value);
    }

    loop {
        let mut changed = false;

        for (feature, deps) in dependencies {
            let all_dependencies_enabled = deps
                .iter()
                .all(|dep| effective.get(dep).copied().unwrap_or(true));
            if !all_dependencies_enabled && effective.get(feature).copied().unwrap_or(true) {
                effective.insert(feature.clone(), false);
                changed = true;
            }
        }

        for (parent, children) in disable_escalation {
            if !effective.get(parent).copied().unwrap_or(true) {
                for child in children {
                    if effective.get(child).copied().unwrap_or(true) {
                        effective.insert(child.clone(), false);
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    for key in core_v1_global_features() {
        effective.insert(key, true);
    }

    effective
}

pub fn can_issue_client_token(client_kind: ClientKind, _ai_enabled: bool) -> bool {
    match client_kind {
        ClientKind::UiWeb | ClientKind::UiMobile => false,
        ClientKind::Agent => true,
    }
}

pub fn can_process_jobs(client_kind: ClientKind) -> bool {
    matches!(client_kind, ClientKind::Agent)
}

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientKind {
    Agent,
    Mcp,
    UiWeb,
    UiMobile,
}

pub fn resolve_effective_features(
    app_feature_enabled: &BTreeMap<String, bool>,
    user_feature_enabled: &BTreeMap<String, bool>,
    dependencies: &BTreeMap<String, Vec<String>>,
    disable_escalation: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, bool> {
    let mut keys: BTreeSet<String> = BTreeSet::new();
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

    let mut effective: BTreeMap<String, bool> = BTreeMap::new();
    for key in keys {
        let app_value = app_feature_enabled.get(&key).copied().unwrap_or(true);
        // Spec invariant: if a key is absent in user preferences, it is treated as true.
        let user_value = user_feature_enabled.get(&key).copied().unwrap_or(true);
        effective.insert(key, app_value && user_value);
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

    effective
}

pub fn can_issue_client_token(client_kind: ClientKind, ai_enabled: bool) -> bool {
    match client_kind {
        ClientKind::UiWeb | ClientKind::UiMobile => false,
        ClientKind::Agent => true,
        ClientKind::Mcp => ai_enabled,
    }
}

pub fn can_process_jobs(client_kind: ClientKind) -> bool {
    matches!(client_kind, ClientKind::Agent)
}

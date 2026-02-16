use std::collections::BTreeMap;

use retaia_agent::resolve_effective_features;

#[test]
fn bdd_given_missing_user_feature_key_when_resolving_then_treated_as_true() {
    let app = BTreeMap::from([(String::from("ai"), true)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);
    assert_eq!(effective.get("ai"), Some(&true));
}

#[test]
fn bdd_given_parent_disabled_when_disable_escalation_exists_then_child_is_disabled() {
    let app = BTreeMap::from([
        (String::from("features.ai"), false),
        (String::from("features.suggestions"), true),
    ]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::from([(
        String::from("features.ai"),
        vec![String::from("features.suggestions")],
    )]);

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);
    assert_eq!(effective.get("features.suggestions"), Some(&false));
}

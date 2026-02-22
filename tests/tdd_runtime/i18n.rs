use retaia_agent::{Language, parse_language, t};
use std::collections::BTreeSet;

fn parse_locale(raw: &str) -> serde_json::Map<String, serde_json::Value> {
    serde_json::from_str(raw).expect("locale json must be valid")
}

#[test]
fn tdd_i18n_parse_language_supports_en_and_fr_prefixes() {
    assert_eq!(parse_language("fr_FR.UTF-8"), Some(Language::Fr));
    assert_eq!(parse_language("en_US.UTF-8"), Some(Language::En));
    assert_eq!(parse_language("de_DE.UTF-8"), None);
}

#[test]
fn tdd_i18n_translation_switches_by_language() {
    assert_eq!(t(Language::En, "daemon.started"), "Daemon started.");
    assert_eq!(t(Language::Fr, "daemon.started"), "Daemon démarré.");
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "missing i18n key")]
fn tdd_i18n_missing_key_panics_in_debug_to_guard_ci() {
    let _ = t(Language::En, "missing.key");
}

#[test]
fn tdd_i18n_locales_have_same_keys_and_non_empty_values() {
    let en = parse_locale(include_str!("../../locales/en.json"));
    let fr = parse_locale(include_str!("../../locales/fr.json"));

    let en_keys: BTreeSet<_> = en.keys().cloned().collect();
    let fr_keys: BTreeSet<_> = fr.keys().cloned().collect();
    assert_eq!(en_keys, fr_keys, "locale keys must match between en/fr");

    for key in en_keys {
        let en_value = en
            .get(&key)
            .and_then(serde_json::Value::as_str)
            .expect("en locale value must be a string")
            .trim()
            .to_string();
        let fr_value = fr
            .get(&key)
            .and_then(serde_json::Value::as_str)
            .expect("fr locale value must be a string")
            .trim()
            .to_string();

        assert!(
            !en_value.is_empty(),
            "en locale value is empty for key {key}"
        );
        assert!(
            !fr_value.is_empty(),
            "fr locale value is empty for key {key}"
        );
    }
}

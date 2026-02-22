use retaia_agent::{Language, parse_language, t};

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

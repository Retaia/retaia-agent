use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Fr,
}

type LocaleMap = HashMap<String, String>;

static EN_LOCALE: OnceLock<LocaleMap> = OnceLock::new();
static FR_LOCALE: OnceLock<LocaleMap> = OnceLock::new();

pub fn detect_language() -> Language {
    if let Ok(explicit) = env::var("RETAIA_AGENT_LANG") {
        return parse_language(&explicit).unwrap_or(Language::En);
    }
    if let Ok(lang) = env::var("LANG") {
        return parse_language(&lang).unwrap_or(Language::En);
    }
    Language::En
}

pub fn parse_language(raw: &str) -> Option<Language> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.starts_with("fr") {
        return Some(Language::Fr);
    }
    if normalized.starts_with("en") {
        return Some(Language::En);
    }
    None
}

pub fn t(lang: Language, key: &str) -> &'static str {
    locale(lang).get(key).map(String::as_str).unwrap_or("")
}

fn locale(lang: Language) -> &'static LocaleMap {
    match lang {
        Language::En => {
            EN_LOCALE.get_or_init(|| parse_locale(include_str!("../../locales/en.json")))
        }
        Language::Fr => {
            FR_LOCALE.get_or_init(|| parse_locale(include_str!("../../locales/fr.json")))
        }
    }
}

fn parse_locale(raw: &str) -> LocaleMap {
    serde_json::from_str(raw).unwrap_or_else(|error| panic!("invalid locale json: {error}"))
}

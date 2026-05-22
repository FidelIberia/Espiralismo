//! Locale tables embedded at compile time (`locales/*.toml`).

use serde::Deserialize;

use super::grammar::{AgentEntry, InflectedWord, QualifierEntry, StemEntry, VerbalState};
use super::Language;

#[derive(Debug, Clone, Deserialize)]
pub struct EpithetTables {
    pub stems: Vec<StemEntry>,
    pub qualifiers: Vec<QualifierEntry>,
    pub adjectives: Vec<InflectedWord>,
    #[serde(default)]
    pub titles: Vec<InflectedWord>,
    #[serde(default)]
    pub curses: Vec<InflectedWord>,
    #[serde(default)]
    pub verbal_states: Vec<VerbalState>,
    #[serde(default)]
    pub verbal_agents: Vec<AgentEntry>,
    #[serde(default)]
    pub proper_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocaleTables {
    pub wisdom: WisdomTables,
    pub epithet: EpithetTables,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WisdomTables {
    pub fragments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct EpithetVocabOverlay {
    epithet: EpithetTables,
}

fn extend_epithet(base: &mut EpithetTables, extra: EpithetTables) {
    base.stems.extend(extra.stems);
    base.qualifiers.extend(extra.qualifiers);
    base.adjectives.extend(extra.adjectives);
    base.titles.extend(extra.titles);
    base.curses.extend(extra.curses);
    base.verbal_states.extend(extra.verbal_states);
    base.verbal_agents.extend(extra.verbal_agents);
    base.proper_names.extend(extra.proper_names);
}

fn parse_locale(base_toml: &str, vocab_toml: &str, label: &'static str) -> LocaleTables {
    let mut locale: LocaleTables = toml::from_str(base_toml)
        .unwrap_or_else(|e| panic!("invalid whisper locale {label}: {e}"));
    let overlay: EpithetVocabOverlay = toml::from_str(vocab_toml)
        .unwrap_or_else(|e| panic!("invalid whisper vocab {label}: {e}"));
    extend_epithet(&mut locale.epithet, overlay.epithet);
    locale
}

static EN: std::sync::OnceLock<LocaleTables> = std::sync::OnceLock::new();
static ES: std::sync::OnceLock<LocaleTables> = std::sync::OnceLock::new();
static RU: std::sync::OnceLock<LocaleTables> = std::sync::OnceLock::new();

/// Loaded tables for the active language.
#[must_use]
pub fn tables(language: Language) -> &'static LocaleTables {
    match language {
        Language::English => EN.get_or_init(|| {
            parse_locale(
                include_str!("locales/en.toml"),
                include_str!("locales/en_vocab.toml"),
                "en",
            )
        }),
        Language::Spanish => ES.get_or_init(|| {
            parse_locale(
                include_str!("locales/es.toml"),
                include_str!("locales/es_vocab.toml"),
                "es",
            )
        }),
        Language::Russian => RU.get_or_init(|| {
            parse_locale(
                include_str!("locales/ru.toml"),
                include_str!("locales/ru_vocab.toml"),
                "ru",
            )
        }),
    }
}

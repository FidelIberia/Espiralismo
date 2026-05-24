//! Whisper interface — fragmentary lore and generation epithets.
//!
//! - [`WhisperKind::Wisdom`] — partial lore (existing `whisper_now` behavior).
//! - [`WhisperKind::GenerationEpithet`] — Diablo 2–style names from entity trait axes.
//!
//! Locales: `locales/en.toml`, `locales/es.toml`, `locales/ru.toml`.

mod common;
mod epithet;
mod grammar;
mod locale;
mod semantic;
mod traits;
mod surface;
mod verbal;
mod wisdom;

pub use grammar::{
    AgentEntry, AgreementKey, Gender, InflectedWord, Number, QualifierEntry, StemEntry, VerbalState,
};

pub use common::{fnv1a64, mix_echo, mix_u64, quantize01, Language, NarrativeEcho};
pub use epithet::forge as forge_generation_epithet;
pub use epithet::{forge_sample, sample_entity};
pub use traits::{GenerationEpithetVoice, WhisperHub, WhisperKind, WhisperRequest, WhisperVoice, WisdomVoice};
pub use wisdom::pick as pick_wisdom;

use crate::core::EntitySnapshot;
use crate::evolution::EvolutionReport;

/// Deterministic English fragment (stable for tests and legacy callers).
#[must_use]
pub fn pick_whisper(mix: u64) -> &'static str {
    wisdom::pick_english_fragment(mix, &NarrativeEcho::default())
}

/// Deterministic fragment with echo bias (English table, stable `&str` for legacy callers).
#[must_use]
pub fn pick_narrative_whisper(base_mix: u64, echo: &NarrativeEcho) -> &'static str {
    wisdom::pick_english_fragment(base_mix, echo)
}

/// Localized wisdom line.
#[must_use]
pub fn pick_narrative_whisper_localized(
    language: Language,
    base_mix: u64,
    echo: &NarrativeEcho,
) -> String {
    pick_wisdom(language, base_mix, echo)
}

/// Standout epithet for the last evolution report (empty if no fittest entity).
#[must_use]
pub fn standout_epithet_for_report(
    report: &EvolutionReport,
    language: Language,
) -> Option<String> {
    let entity = report.fittest()?;
    let generation = report
        .cycles
        .saturating_sub(1)
        .max(entity.generation);
    let name = WhisperHub::new().speak(&WhisperRequest {
        kind: WhisperKind::GenerationEpithet,
        language,
        mix: 0,
        echo: &NarrativeEcho::default(),
        standout: Some(entity),
        generation,
    });
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Builds a [`WhisperRequest`] for wisdom.
#[must_use]
pub fn wisdom_request<'a>(
    language: Language,
    mix: u64,
    echo: &'a NarrativeEcho,
) -> WhisperRequest<'a> {
    WhisperRequest {
        kind: WhisperKind::Wisdom,
        language,
        mix,
        echo,
        standout: None,
        generation: 0,
    }
}

/// Builds a [`WhisperRequest`] for a generation epithet.
#[must_use]
pub fn epithet_request<'a>(
    language: Language,
    echo: &'a NarrativeEcho,
    standout: &'a EntitySnapshot,
    generation: u32,
) -> WhisperRequest<'a> {
    WhisperRequest {
        kind: WhisperKind::GenerationEpithet,
        language,
        mix: 0,
        echo,
        standout: Some(standout),
        generation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EntitySnapshot;

    #[test]
    fn pick_whisper_is_deterministic() {
        assert_eq!(pick_whisper(0xC0FFEE), pick_whisper(0xC0FFEE));
    }

    #[test]
    fn pick_whisper_covers_table() {
        let n = locale::tables(Language::English).wisdom.fragments.len();
        for i in 0..n {
            let line = pick_whisper(i as u64);
            assert!(!line.is_empty());
        }
    }

    #[test]
    fn narrative_echo_biases_index_without_panic() {
        let echo = NarrativeEcho {
            dominant_tone_idx: 3,
            scar_mass: 42,
            rare_event_token: 0xBEEF,
            dying_viability_quant: 200,
            fossil_absence_mass: 1200,
            attention_xor: 0xC0DED00D,
            soul_attunement_quant: 180,
        };
        let a = pick_narrative_whisper(0x1111, &NarrativeEcho::default());
        let b = pick_narrative_whisper(0x1111, &echo);
        assert!(!a.is_empty() && !b.is_empty());
    }

    #[test]
    fn epithet_omits_origin_and_curse_when_traits_are_low() {
        let entity = EntitySnapshot {
            label: "quiet_lattice".to_string(),
            generation: 1,
            fitness: 5.0,
            viability: 0.7,
            vitality: Some(0.5),
            resonance: Some(0.2),
            mutation_pressure: Some(0.15),
            symbolic_density: Some(0.25),
            memory_depth: Some(0.1),
            shadow_pull: Some(0.12),
            myth: None,
        };
        let name = forge_generation_epithet(&entity, 1, Language::Spanish);
        assert!(!name.is_empty());
        assert!(!name.contains("maldito"));
        assert!(!name.contains("hechizado"));
        assert!(!name.contains("del abismo"));
        assert!(!name.contains("de la desesperación"));
    }

    #[test]
    fn epithet_forge_is_deterministic_and_non_empty() {
        let entity = EntitySnapshot {
            label: "ResonanceEngine".to_string(),
            generation: 4,
            fitness: 31.5,
            viability: 0.62,
            vitality: Some(0.71),
            resonance: Some(0.55),
            mutation_pressure: Some(0.48),
            symbolic_density: Some(0.33),
            memory_depth: Some(0.8),
            shadow_pull: Some(0.77),
            myth: Some("keeper".to_string()),
        };
        let a = forge_generation_epithet(&entity, 4, Language::Spanish);
        let b = forge_generation_epithet(&entity, 4, Language::Spanish);
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }

    #[test]
    fn verbal_phrase_splits_participle_and_agent() {
        use super::grammar::{AgentEntry, VerbalState};
        use super::verbal::compose_verbal;

        let key = AgreementKey::from_tags("m", "s");
        let state = VerbalState {
            ms: Some("sellado".to_string()),
            fs: Some("sellada".to_string()),
            ..VerbalState::default()
        };
        let participle = state.participle(key).unwrap();
        let agent = AgentEntry {
            text: "la sombra".to_string(),
            g: Some("f".to_string()),
            n: Some("s".to_string()),
            linker: "por".to_string(),
            indefinite: false,
            ..AgentEntry::default()
        };
        assert_eq!(
            compose_verbal(Language::Spanish, participle, Some(&agent)),
            "sellado por la sombra"
        );
        assert_eq!(compose_verbal(Language::Spanish, participle, None), "sellado");
    }

    #[test]
    fn localized_wisdom_differs_by_language() {
        let echo = NarrativeEcho::default();
        let en = pick_wisdom(Language::English, 42, &echo);
        let es = pick_wisdom(Language::Spanish, 42, &echo);
        assert_ne!(en, es);
    }
}

//! Generation epithet forge — inflected Diablo-style names with optional patronymics.

use crate::core::EntitySnapshot;

use super::common::{fnv1a64, mix_u64, quantize01, Language};
use super::grammar::{AgreementKey, InflectedWord, QualifierEntry, StemEntry};
use super::surface::{
    capitalize_first, format_glued_prologue, format_name_phrase_caps, title_for_name_phrase,
};
use super::locale;
use super::verbal::{forge_verbal_phrase, VerbalAttachPolicy};

const SLOT_STEM: u32 = 0;
const SLOT_QUALIFIER: u32 = 1;
const SLOT_PATRON: u32 = 2;
const SLOT_TITLE: u32 = 3;
const SLOT_TRAIT_ADJ: u32 = 4;
const SLOT_CURSE: u32 = 5;
const SLOT_ORDER: u32 = 6;
const SLOT_PROLOGUE: u32 = 7;

const MAX_EPILOGUE_PIECES: usize = 4;

/// Built slots before language-specific assembly.
struct Assembled<'a> {
    stem: &'a str,
    qualifier: Option<&'a str>,
    patron: Option<&'a str>,
    title: Option<&'a str>,
    /// Single opening state (verbal or adjective), comma-separated from the core when verbal.
    prologue: Option<String>,
    prologue_is_verbal: bool,
    /// Adjectives glued to the head noun (`rotas silenciosas`).
    epilogue_modifiers: Vec<String>,
    /// Static curse after a comma (`, condenada`).
    epilogue_curse: Option<String>,
    head_key: AgreementKey,
}

const SAMPLE_LABELS: &[&str] = &[
    "ResonanceEngine",
    "Living Memory",
    "Mercy Field",
    "Living Cartography",
    "active_lattice_0",
    "genesis",
    "CartographyArchive",
    "wanderer",
    "keeper",
    "witness",
];

/// Synthetic entity with varied trait axes for epithet demos (`--epithets`).
#[must_use]
pub fn sample_entity(index: u32, base_seed: u64) -> EntitySnapshot {
    let h = mix_u64(base_seed, index as u64);
    let axis = |salt: u64| -> f32 {
        ((h.wrapping_add(salt) % 1000) as f32) / 999.0
    };
    let label = SAMPLE_LABELS[(index as usize) % SAMPLE_LABELS.len()].to_string();
    let generation = (h % 17) as u32;
    let fitness = 4.0 + (h % 4000) as f32 / 40.0;
    let viability = axis(3);
    let vitality = Some(axis(5));
    let resonance = Some(axis(7));
    let mutation_pressure = Some(axis(11));
    let symbolic_density = Some(axis(13));
    let memory_depth = Some(axis(17));
    let shadow_pull = Some(axis(19));
    EntitySnapshot {
        label,
        generation,
        fitness,
        viability,
        vitality,
        resonance,
        mutation_pressure,
        symbolic_density,
        memory_depth,
        shadow_pull,
        myth: None,
    }
}

/// Forges one epithet from a synthetic sample (deterministic per `index` + `base_seed`).
#[must_use]
pub fn forge_sample(language: Language, index: u32, base_seed: u64) -> String {
    let entity = sample_entity(index, base_seed);
    forge(&entity, entity.generation, language)
}

/// Builds a deterministic honorific for the standout of a generation.
#[must_use]
pub fn forge(entity: &EntitySnapshot, generation: u32, language: Language) -> String {
    let tables = &locale::tables(language).epithet;
    if tables.stems.is_empty() {
        return entity.label.clone();
    }

    let shadow = entity.shadow_pull.unwrap_or(0.0);
    let mutation = entity.mutation_pressure.unwrap_or(0.0);
    let memory = entity.memory_depth.unwrap_or(0.0);
    let resonance = entity.resonance.unwrap_or(0.0);
    let symbolic = entity.symbolic_density.unwrap_or(0.0);
    let viability = entity.viability;
    let fitness = entity.fitness;

    let mut seed = fnv1a64(entity.label.as_bytes());
    seed = mix_u64(seed, generation as u64);
    seed = mix_u64(seed, quantize01(shadow) as u64);
    seed = mix_u64(seed, quantize01(mutation) as u64);
    seed = mix_u64(seed, quantize01(memory) as u64);
    seed = mix_u64(seed, quantize01(resonance) as u64);
    seed = mix_u64(seed, quantize01(symbolic) as u64);
    seed = mix_u64(seed, (viability * 1000.0) as u64);
    seed = mix_u64(seed, (fitness * 10.0) as u64);

    let stem_entry = pick_stem(seed, tables, shadow, mutation, memory, fitness);
    let key = stem_entry.key();
    let stem = stem_entry.text.as_str();

    let qualifier = pick_qualifier(seed, tables, key, memory, resonance);
    let patron = pick_patron(seed, tables, memory, resonance, symbolic);
    let title = pick_inflected(seed, SLOT_TITLE, &tables.titles, key, shadow, mutation, 0.52, 0.36, 220);
    let (prologue, prologue_is_verbal) =
        pick_prologue(seed, tables, key, language, shadow, mutation, viability, resonance, symbolic);
    let (epilogue_modifiers, epilogue_curse) = pick_epilogue(
        seed,
        tables,
        key,
        shadow,
        mutation,
        viability,
        resonance,
        symbolic,
    );

    let assembled = Assembled {
        stem,
        qualifier,
        patron,
        title,
        prologue,
        prologue_is_verbal,
        epilogue_modifiers,
        epilogue_curse,
        head_key: key,
    };

    let _order = (roll_permille(seed, SLOT_ORDER) as usize) % 6;
    assemble(language, &assembled)
}

fn pick_stem<'a>(
    seed: u64,
    tables: &'a super::locale::EpithetTables,
    shadow: f32,
    mutation: f32,
    memory: f32,
    fitness: f32,
) -> &'a StemEntry {
    let idx = weighted_index(
        seed,
        SLOT_STEM,
        tables.stems.len(),
        trait_bias(shadow, mutation, memory, fitness),
    );
    &tables.stems[idx]
}

fn pick_qualifier<'a>(
    seed: u64,
    tables: &'a super::locale::EpithetTables,
    key: AgreementKey,
    memory: f32,
    resonance: f32,
) -> Option<&'a str> {
    let candidates: Vec<&QualifierEntry> = tables
        .qualifiers
        .iter()
        .filter(|q| q.matches(key))
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let need_memory = memory > 0.48;
    let need_resonance = resonance > 0.52;
    if !need_memory && !need_resonance {
        return None;
    }
    let chance = ((memory * 280.0) + (resonance * 200.0)).clamp(80.0, 480.0) as u32;
    if roll_permille(seed, SLOT_QUALIFIER) >= chance {
        return None;
    }
    let qi = weighted_index(seed, SLOT_QUALIFIER, candidates.len(), (memory * 300.0) as u64);
    Some(candidates[qi].text.as_str())
}

fn pick_patron<'a>(
    seed: u64,
    tables: &'a super::locale::EpithetTables,
    memory: f32,
    resonance: f32,
    symbolic: f32,
) -> Option<&'a str> {
    if tables.proper_names.is_empty() {
        return None;
    }
    if memory < 0.36 && resonance < 0.40 && symbolic < 0.32 {
        return None;
    }
    let chance = ((memory * 240.0) + (resonance * 200.0) + (symbolic * 180.0)).clamp(80.0, 480.0) as u32;
    if roll_permille(seed, SLOT_PATRON) >= chance {
        return None;
    }
    let pi = weighted_index(seed, SLOT_PATRON, tables.proper_names.len(), (memory * 200.0) as u64);
    Some(tables.proper_names[pi].as_str())
}

/// Opening state: at most one verbal phrase, otherwise an optional lead adjective.
fn pick_prologue(
    seed: u64,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    language: Language,
    shadow: f32,
    mutation: f32,
    viability: f32,
    resonance: f32,
    symbolic: f32,
) -> (Option<String>, bool) {
    let verbal_gate = shadow > 0.40 || mutation > 0.38;
    if verbal_gate && !tables.verbal_states.is_empty() {
        let chance = ((shadow * 280.0) + (mutation * 220.0)).clamp(100.0, 550.0) as u32;
        if roll_permille(seed, SLOT_PROLOGUE) < chance {
            let policy = VerbalAttachPolicy {
                solo_permille: 380,
                indefinite_permille: 180,
            };
            if let Some(phrase) = forge_verbal_phrase(
                language,
                key,
                seed,
                SLOT_PROLOGUE,
                &tables.verbal_states,
                &tables.verbal_agents,
                &policy,
                roll_permille,
            ) {
                return (Some(phrase), true);
            }
        }
    }

    if roll_permille(seed, SLOT_PROLOGUE.wrapping_add(1)) >= 380 {
        return (None, false);
    }
    let chance = ((viability * 180.0) + (resonance * 140.0) + (symbolic * 120.0)).clamp(100.0, 380.0) as u32;
    if roll_permille(seed, SLOT_PROLOGUE.wrapping_add(2)) >= chance {
        return (None, false);
    }
    (
        pick_trait_adjective(seed, SLOT_PROLOGUE, tables, key, 0),
        false,
    )
}

/// Trailing descriptors: adjectives + static curse; never another verbal state.
fn pick_epilogue(
    seed: u64,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    shadow: f32,
    mutation: f32,
    viability: f32,
    resonance: f32,
    symbolic: f32,
) -> (Vec<String>, Option<String>) {
    let mut modifiers = Vec::with_capacity(MAX_EPILOGUE_PIECES);
    let base_chance = ((viability * 200.0) + (resonance * 160.0) + (symbolic * 140.0)).clamp(120.0, 620.0) as u32;
    for i in 0..MAX_EPILOGUE_PIECES {
        let slot = SLOT_TRAIT_ADJ.wrapping_add(i as u32);
        let threshold = base_chance.saturating_sub(i as u32 * 110);
        if threshold < 80 || roll_permille(seed, slot) >= threshold {
            break;
        }
        if let Some(phrase) = pick_trait_adjective(seed, slot, tables, key, i) {
            if modifiers.iter().all(|p| p != &phrase) {
                modifiers.push(phrase);
            }
        }
    }
    let curse = pick_static_curse(seed, tables, key, shadow, mutation);
    (modifiers, curse)
}

fn pick_trait_adjective(
    seed: u64,
    slot: u32,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    salt: usize,
) -> Option<String> {
    pick_matching_word(seed, slot, &tables.adjectives, key, salt).map(str::to_string)
}

fn pick_static_curse(
    seed: u64,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    shadow: f32,
    mutation: f32,
) -> Option<String> {
    if tables.curses.is_empty() || (shadow < 0.42 && mutation < 0.38) {
        return None;
    }
    let chance = ((shadow * 320.0) + (mutation * 260.0)).clamp(0.0, 520.0) as u32;
    if chance < 80 || roll_permille(seed, SLOT_CURSE) >= chance {
        return None;
    }
    pick_matching_word(seed, SLOT_CURSE, &tables.curses, key, 0).map(str::to_string)
}

fn pick_inflected<'a>(
    seed: u64,
    slot: u32,
    pool: &'a [InflectedWord],
    key: AgreementKey,
    shadow: f32,
    mutation: f32,
    shadow_floor: f32,
    mutation_floor: f32,
    max_permille: u32,
) -> Option<&'a str> {
    if pool.is_empty() || (shadow < shadow_floor && mutation < mutation_floor) {
        return None;
    }
    let chance = ((shadow * 320.0) + (mutation * 260.0)).clamp(0.0, max_permille as f32) as u32;
    if chance < 80 || roll_permille(seed, slot) >= chance {
        return None;
    }
    pick_matching_word(seed, slot, pool, key, 0)
}

fn pick_matching_word<'a>(
    seed: u64,
    slot: u32,
    pool: &'a [InflectedWord],
    key: AgreementKey,
    salt: usize,
) -> Option<&'a str> {
    let matching: Vec<&str> = pool
        .iter()
        .filter_map(|w| w.resolve_agreeing(key))
        .collect();
    if matching.is_empty() {
        return None;
    }
    let idx = weighted_index(
        seed,
        slot,
        matching.len(),
        (salt as u64).wrapping_add(key.gender as u64),
    );
    Some(matching[idx])
}

fn roll_permille(seed: u64, slot: u32) -> u32 {
    let mixed = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((slot as u64).wrapping_mul(0x517c_c1b7_2722_0a95));
    ((mixed >> 16) as u32) % 1000
}

fn trait_bias(shadow: f32, mutation: f32, memory: f32, fitness: f32) -> u64 {
    ((shadow * 400.0) + (mutation * 250.0) + (memory * 180.0) + (fitness * 90.0)) as u64
}

fn weighted_index(seed: u64, slot: u32, len: usize, bias: u64) -> usize {
    if len == 0 {
        return 0;
    }
    let mixed = seed
        .wrapping_add(bias)
        .wrapping_add((slot as u64).rotate_left(11));
    (mixed as usize) % len
}

fn assemble(language: Language, p: &Assembled<'_>) -> String {
    let patron = p.patron.map(|n| patron_suffix(language, n));
    let core = build_name_phrase(language, p, &patron);
    append_curse_suffix(assemble_with_prologue(language, p, core), p.epilogue_curse.as_deref())
}

fn patron_suffix(language: Language, name: &str) -> String {
    match language {
        Language::Spanish => format!(" de {name}"),
        Language::English => format!(" of {name}"),
        Language::Russian => format!(" {name}"),
    }
}

/// Head noun phrase: word order and agreement follow the active locale.
fn build_name_phrase(language: Language, p: &Assembled<'_>, patron: &Option<String>) -> String {
    match language {
        Language::Spanish => build_name_phrase_spanish(p, patron),
        Language::English => build_name_phrase_english(p, patron),
        Language::Russian => build_name_phrase_russian(p, patron),
    }
}

/// Spanish: head noun, post-nominal modifiers, qualifier, patron, title.
fn build_name_phrase_spanish(p: &Assembled<'_>, patron: &Option<String>) -> String {
    let mut phrase = p.stem.to_string();
    for modifier in &p.epilogue_modifiers {
        phrase.push(' ');
        phrase.push_str(modifier);
    }
    append_qualifier_patron_title(Language::Spanish, p, &mut phrase, patron);
    phrase
}

/// English: pre-nominal modifiers, head noun, `of` qualifier, patron, title.
fn build_name_phrase_english(p: &Assembled<'_>, patron: &Option<String>) -> String {
    use super::surface::sentence_case_phrase;

    let mut parts: Vec<&str> = Vec::with_capacity(p.epilogue_modifiers.len() + 1);
    for modifier in &p.epilogue_modifiers {
        parts.push(modifier.as_str());
    }
    parts.push(p.stem);
    let mut phrase = sentence_case_phrase(&parts.join(" "));
    if let Some(q) = p.qualifier {
        phrase = format!("{phrase} {q}");
    }
    if let Some(ref pat) = patron {
        phrase.push_str(pat);
    }
    if let Some(t) = p.title {
        phrase = format!(
            "{phrase} {}",
            title_for_name_phrase(Language::English, p.head_key, t)
        );
    }
    phrase
}

/// Russian: head noun, post-nominal modifiers, genitive qualifier, patron, title.
fn build_name_phrase_russian(p: &Assembled<'_>, patron: &Option<String>) -> String {
    let mut phrase = p.stem.to_string();
    for modifier in &p.epilogue_modifiers {
        phrase.push(' ');
        phrase.push_str(modifier);
    }
    append_qualifier_patron_title(Language::Russian, p, &mut phrase, patron);
    phrase
}

fn append_qualifier_patron_title(
    language: Language,
    p: &Assembled<'_>,
    phrase: &mut String,
    patron: &Option<String>,
) {
    if let Some(q) = p.qualifier {
        *phrase = format!("{phrase} {q}");
    }
    if let Some(ref pat) = patron {
        phrase.push_str(pat);
    }
    if let Some(t) = p.title {
        *phrase = format!(
            "{phrase} {}",
            title_for_name_phrase(language, p.head_key, t)
        );
    }
}

/// Verbal prologue → comma; simple adjective → glued (`Ancient Maul` / `Древний молот`).
fn assemble_with_prologue(language: Language, p: &Assembled<'_>, name_phrase: String) -> String {
    let formatted_name = format_name_for_display(language, &name_phrase);
    let Some(ref prologue) = p.prologue else {
        return formatted_name;
    };
    if p.prologue_is_verbal {
        format!(
            "{}, {}",
            capitalize_first(prologue),
            formatted_name
        )
    } else {
        format_glued_prologue(language, prologue, &name_phrase)
    }
}

/// Name phrase caps after [`build_name_phrase`] (English already sentence-cased in the builder).
fn format_name_for_display(language: Language, name_phrase: &str) -> String {
    match language {
        Language::English => name_phrase.to_string(),
        _ => format_name_phrase_caps(language, name_phrase),
    }
}

fn append_curse_suffix(name_part: String, curse: Option<&str>) -> String {
    match curse {
        Some(c) => format!("{name_part}, {c}"),
        None => name_part,
    }
}

#[cfg(test)]
mod assemble_tests {
    use super::*;
    use crate::whisper::grammar::AgreementKey;
    use crate::whisper::surface::title_for_name_phrase;

    #[test]
    fn spanish_title_drops_article_before_name_phrase() {
        let key = AgreementKey::from_tags("f", "s");
        assert_eq!(
            title_for_name_phrase(Language::Spanish, key, "la sin nombre"),
            "sin nombre"
        );
    }

    #[test]
    fn simple_prologue_glues_with_title_case() {
        let p = Assembled {
            stem: "Mazo",
            qualifier: None,
            patron: None,
            title: None,
            prologue: Some("antiguo".to_string()),
            prologue_is_verbal: false,
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "s"),
        };
        assert_eq!(assemble(Language::Spanish, &p), "Antiguo Mazo");
    }

    #[test]
    fn modifiers_glue_to_stem_not_comma() {
        let p = Assembled {
            stem: "Campanas rotas",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            epilogue_modifiers: vec!["silenciosas".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "p"),
        };
        assert_eq!(assemble(Language::Spanish, &p), "Campanas rotas silenciosas");
    }

    #[test]
    fn verbal_prologue_then_name_only_first_cap() {
        let p = Assembled {
            stem: "Vela negra",
            qualifier: None,
            patron: Some("Lakshmana"),
            title: None,
            prologue: Some("profanada por la luna roja".to_string()),
            prologue_is_verbal: true,
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Profanada por la luna roja, Vela negra de Lakshmana"
        );
    }

    #[test]
    fn title_appended_without_article() {
        let p = Assembled {
            stem: "Corona rota",
            qualifier: None,
            patron: None,
            title: Some("la sin nombre"),
            prologue: Some("devorada por los dioses".to_string()),
            prologue_is_verbal: true,
            epilogue_modifiers: vec![],
            epilogue_curse: Some("condenada".to_string()),
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Devorada por los dioses, Corona rota sin nombre, condenada"
        );
    }

    #[test]
    fn english_modifiers_precede_noun() {
        let p = Assembled {
            stem: "Broken Crowns",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            epilogue_modifiers: vec!["silent".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "p"),
        };
        assert_eq!(assemble(Language::English, &p), "Silent broken crowns");
    }

    #[test]
    fn english_glued_lead_adjective() {
        let p = Assembled {
            stem: "Maul",
            qualifier: None,
            patron: None,
            title: None,
            prologue: Some("ancient".to_string()),
            prologue_is_verbal: false,
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "s"),
        };
        assert_eq!(assemble(Language::English, &p), "Ancient Maul");
    }

    #[test]
    fn english_verbal_prologue_and_patron() {
        let p = Assembled {
            stem: "Black Candle",
            qualifier: None,
            patron: Some("Lakshmana"),
            title: None,
            prologue: Some("profaned by the red moon".to_string()),
            prologue_is_verbal: true,
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::English, &p),
            "Profaned by the red moon, Black candle of Lakshmana"
        );
    }

    #[test]
    fn english_title_without_article() {
        let p = Assembled {
            stem: "Broken Crown",
            qualifier: None,
            patron: None,
            title: Some("the Nameless"),
            prologue: Some("devoured by the gods".to_string()),
            prologue_is_verbal: true,
            epilogue_modifiers: vec![],
            epilogue_curse: Some("condemned".to_string()),
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::English, &p),
            "Devoured by the gods, Broken crown Nameless, condemned"
        );
    }

    #[test]
    fn russian_modifiers_follow_noun() {
        let p = Assembled {
            stem: "Сферы",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            epilogue_modifiers: vec!["молчаливые".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "p"),
        };
        assert_eq!(assemble(Language::Russian, &p), "Сферы молчаливые");
    }

    #[test]
    fn russian_glued_lead_adjective() {
        let p = Assembled {
            stem: "Молот",
            qualifier: None,
            patron: None,
            title: None,
            prologue: Some("древний".to_string()),
            prologue_is_verbal: false,
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "s"),
        };
        assert_eq!(assemble(Language::Russian, &p), "Древний Молот");
    }

    #[test]
    fn russian_verbal_instrumental_prologue() {
        let p = Assembled {
            stem: "Реликвия",
            qualifier: Some("бездны"),
            patron: None,
            title: Some("безымянная"),
            prologue: Some("запечатанная тенью".to_string()),
            prologue_is_verbal: true,
            epilogue_modifiers: vec![],
            epilogue_curse: Some("проклятая".to_string()),
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::Russian, &p),
            "Запечатанная тенью, Реликвия бездны безымянная, проклятая"
        );
    }
}

//! Fragmentary one-line whispers — **partial lore**, not a glossary.
//!
//! Strings are intentionally incomplete. Selection can fold in **narrative echo** (tones, scars,
//! rare events, dying entities, attention) so lines feel *discovered* from history, not purely
//! random (see `Iter/10.md`).

/// Curated fragments (English). Order is stable: indices are part of the persistence contract
/// only indirectly via the stored `String` snapshot on each checkpoint line.
const FRAGMENTS: &[&str] = &[
    "The spiral tightens where nothing was promised.",
    "What was woven once still tugs at the corners.",
    "Edges sharpen where the witness stayed too long.",
    "Some doors only open after the third forgetting.",
    "The salt remembers the tide; it does not argue.",
    "A pattern repeated is not yet a law.",
    "The center held because the rim forgot to fall.",
    "Echoes trade names; only distance keeps them honest.",
    "The field cools when the sky pretends to sleep.",
    "Memory walks backward through smoke.",
    "Not every lattice is a cage—some are maps.",
    "The glyph knew the hand before the hand knew intent.",
    "Stillness is not absence; it is pressure dressed as rest.",
    "Two resonances met and called it coincidence.",
    "The cartographer drew a shore; the sea revised it.",
    "Mercy left a door ajar; discipline closed nothing.",
    "Spark without root becomes weather.",
    "Root without spark becomes monument.",
    "The archive yawned; the epoch advanced anyway.",
    "A line of power ends where curiosity hesitates.",
    "The sky offered a bias; the spiral chose a step.",
    "Harmony is a loan; the interest is attention.",
    "What survives selection is not always what was loved.",
    "The last report was true enough to mislead kindly.",
    "The fossil hums under the new paint.",
    "A taboo is only a love letter written backwards.",
    "The lattice dreamed a symbol it had never hosted.",
    "Absence piled until it became a second alphabet.",
    "The watched corner grew warmer than the sunlit hall.",
];

/// Compact digest of internal history used to bias whisper selection.
#[derive(Clone, Debug, Default)]
pub struct NarrativeEcho {
    /// Dominant tone index in canonical [`crate::glyphs::GlyphTone::ALL`] order.
    pub dominant_tone_idx: u8,
    /// Lattice scar mass (or `0` if no lattice).
    pub scar_mass: u32,
    /// FNV-1a of last rare-event token (or `0`).
    pub rare_event_token: u64,
    /// Quantized minimum viability across last report (`255` = healthy).
    pub dying_viability_quant: u8,
    /// Ghost-breath mass from glyph archaeology.
    pub fossil_absence_mass: u32,
    /// Observer attention digest.
    pub attention_xor: u64,
}

/// FNV-1a 64 over bytes (stable tiny hasher for event names).
#[must_use]
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn mix_echo(base: u64, echo: &NarrativeEcho) -> u64 {
    let mut x = base;
    x ^= (echo.dominant_tone_idx as u64).rotate_left(17);
    x ^= (echo.scar_mass as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= echo.rare_event_token.rotate_left(11);
    x ^= (echo.dying_viability_quant as u64).wrapping_mul(0x85eb_ca6b);
    x ^= (echo.fossil_absence_mass as u64).rotate_left(23);
    x ^= echo.attention_xor;
    x
}

/// Deterministic fragment from a mixing value plus optional narrative echo.
#[must_use]
pub fn pick_narrative_whisper(base_mix: u64, echo: &NarrativeEcho) -> &'static str {
    let idx = (mix_echo(base_mix, echo) as usize) % FRAGMENTS.len();
    FRAGMENTS[idx]
}

/// Deterministic fragment from a mixing value (seed XOR epoch, Julian bits, etc.).
#[must_use]
pub fn pick_whisper(mix: u64) -> &'static str {
    pick_narrative_whisper(mix, &NarrativeEcho::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_whisper_is_deterministic() {
        assert_eq!(pick_whisper(0xC0FFEE), pick_whisper(0xC0FFEE));
    }

    #[test]
    fn pick_whisper_covers_table() {
        for i in 0..FRAGMENTS.len() {
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
        };
        let a = pick_narrative_whisper(0x1111, &NarrativeEcho::default());
        let b = pick_narrative_whisper(0x1111, &echo);
        assert!(!a.is_empty() && !b.is_empty());
    }
}

//! Curated glyph alphabet grouped by symbolic tone.
//!
//! The alphabet is the **source of truth** for procedural glyph generation. Each entry pairs a
//! Unicode symbol with a [`GlyphTone`] and a `weight` used by the generator for intra-tone selection.

use serde::{Deserialize, Serialize};

/// Symbolic tone used to group glyphs by narrative role.
///
/// Tones drive both intra-cell selection (which glyph) and inter-cell distribution (how often).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GlyphTone {
    /// High-resonance, illumination-aligned glyphs (the "yes" of the spiral).
    Luminous,
    /// Deep-witness glyphs aligned with awareness / observer motifs.
    Witness,
    /// Mid-balance glyphs that act as connective tissue.
    Neutral,
    /// Low-resonance, void / shadow-aligned glyphs.
    Shadow,
    /// Anchor / structure glyphs (the "ground" of the spiral).
    Root,
    /// Chaotic / mutation-aligned glyphs (the "spark" surge).
    Spark,
}

impl GlyphTone {
    /// Canonical enumeration order used by the generator.
    pub const ALL: [GlyphTone; 6] = [
        GlyphTone::Luminous,
        GlyphTone::Witness,
        GlyphTone::Neutral,
        GlyphTone::Shadow,
        GlyphTone::Root,
        GlyphTone::Spark,
    ];

    /// Stable string label.
    pub fn label(&self) -> &'static str {
        match self {
            GlyphTone::Luminous => "luminous",
            GlyphTone::Witness => "witness",
            GlyphTone::Neutral => "neutral",
            GlyphTone::Shadow => "shadow",
            GlyphTone::Root => "root",
            GlyphTone::Spark => "spark",
        }
    }
}

/// Single glyph entry in a [`GlyphAlphabet`].
#[derive(Clone, Debug)]
pub struct Glyph {
    /// Display symbol (Unicode).
    pub symbol: char,
    /// Tone classification.
    pub tone: GlyphTone,
    /// Intra-tone selection weight (must be > 0.0 to be reachable).
    pub weight: f32,
    /// Human-readable name (used for diagnostics).
    pub name: &'static str,
}

/// Ordered alphabet of [`Glyph`]s used by procedural generators.
#[derive(Clone, Debug)]
pub struct GlyphAlphabet {
    entries: Vec<Glyph>,
}

impl GlyphAlphabet {
    /// Builds the canonical Spiralismo alphabet covering all six tones.
    pub fn canonical() -> Self {
        Self {
            entries: vec![
                Glyph { symbol: '◉', tone: GlyphTone::Luminous, weight: 1.5, name: "solar_core" },
                Glyph { symbol: '✦', tone: GlyphTone::Luminous, weight: 1.0, name: "starwise" },
                Glyph { symbol: '✧', tone: GlyphTone::Luminous, weight: 0.7, name: "starwise_open" },
                Glyph { symbol: '☼', tone: GlyphTone::Luminous, weight: 0.8, name: "sunburst" },

                Glyph { symbol: '𓂀', tone: GlyphTone::Witness, weight: 1.2, name: "eye_of_horus" },
                Glyph { symbol: '𓁹', tone: GlyphTone::Witness, weight: 0.9, name: "watcher" },
                Glyph { symbol: '◎', tone: GlyphTone::Witness, weight: 0.7, name: "halo_ring" },

                Glyph { symbol: '⟡', tone: GlyphTone::Neutral, weight: 1.2, name: "diamond_open" },
                Glyph { symbol: '⟢', tone: GlyphTone::Neutral, weight: 0.9, name: "diamond_left" },
                Glyph { symbol: '⟣', tone: GlyphTone::Neutral, weight: 0.9, name: "diamond_right" },
                Glyph { symbol: '◇', tone: GlyphTone::Neutral, weight: 0.7, name: "lozenge" },

                Glyph { symbol: '☽', tone: GlyphTone::Shadow, weight: 1.0, name: "lunar_arc" },
                Glyph { symbol: '◌', tone: GlyphTone::Shadow, weight: 0.8, name: "dotted_void" },
                Glyph { symbol: '⊘', tone: GlyphTone::Shadow, weight: 0.6, name: "null_seal" },

                Glyph { symbol: '⧉', tone: GlyphTone::Root, weight: 1.3, name: "root_anchor" },
                Glyph { symbol: '⧈', tone: GlyphTone::Root, weight: 0.9, name: "root_bond" },
                Glyph { symbol: '⧊', tone: GlyphTone::Root, weight: 0.8, name: "root_split" },

                Glyph { symbol: '⚛', tone: GlyphTone::Spark, weight: 0.9, name: "atom_spark" },
                Glyph { symbol: '✺', tone: GlyphTone::Spark, weight: 0.8, name: "burst" },
                Glyph { symbol: '✷', tone: GlyphTone::Spark, weight: 0.7, name: "shard" },
            ],
        }
    }

    /// Builds an alphabet from a caller-provided ordered list.
    pub fn from_entries(entries: Vec<Glyph>) -> Self {
        Self { entries }
    }

    /// Total number of glyphs across all tones.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no glyph entries are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Read-only entry view (registration order).
    pub fn entries(&self) -> &[Glyph] {
        &self.entries
    }

    /// All glyph entries that match `tone`.
    pub fn by_tone(&self, tone: GlyphTone) -> Vec<&Glyph> {
        self.entries.iter().filter(|g| g.tone == tone).collect()
    }

    /// True when at least one glyph exists for `tone`.
    pub fn supports(&self, tone: GlyphTone) -> bool {
        self.entries.iter().any(|g| g.tone == tone)
    }

    /// Default fallback glyph for empty selections (first entry or the standard root glyph).
    pub fn fallback(&self) -> Glyph {
        self.entries
            .first()
            .cloned()
            .unwrap_or(Glyph {
                symbol: crate::utils::symbols::DEFAULT_GLYPH,
                tone: GlyphTone::Root,
                weight: 1.0,
                name: "fallback_root",
            })
    }
}

impl Default for GlyphAlphabet {
    fn default() -> Self {
        Self::canonical()
    }
}

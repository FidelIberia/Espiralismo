//! Curated glyph alphabet grouped by symbolic tone.
//!
//! The alphabet is the **source of truth** for procedural glyph generation. Each entry pairs a
//! Unicode symbol with a [`GlyphTone`] and a `weight` used by the generator for intra-tone selection.

use serde::{Deserialize, Serialize};

use crate::core::CellColor;

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

fn cell_color_for_tone(tone: GlyphTone) -> CellColor {
    match tone {
        GlyphTone::Luminous => CellColor::BrightYellow,
        GlyphTone::Witness => CellColor::BrightMagenta,
        GlyphTone::Neutral => CellColor::Cyan,
        GlyphTone::Shadow => CellColor::BrightBlue,
        GlyphTone::Root => CellColor::BrightGreen,
        GlyphTone::Spark => CellColor::BrightRed,
    }
}

fn glyph_entry(symbol: char, tone: GlyphTone, weight: f32, name: &str) -> Glyph {
    Glyph {
        symbol,
        tone,
        weight,
        name: name.into(),
        color: cell_color_for_tone(tone),
    }
}

/// Single glyph entry in a [`GlyphAlphabet`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Glyph {
    /// Display symbol (Unicode).
    pub symbol: char,
    /// Tone classification.
    pub tone: GlyphTone,
    /// Intra-tone selection weight (must be > 0.0 to be reachable).
    pub weight: f32,
    /// Human-readable name (used for diagnostics and checkpoints).
    pub name: String,
    /// Terminal / checkpoint color dimension (may be overridden per cell in fields).
    pub color: CellColor,
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
                glyph_entry('◉', GlyphTone::Luminous, 1.5, "solar_core"),
                glyph_entry('✦', GlyphTone::Luminous, 1.0, "starwise"),
                glyph_entry('✧', GlyphTone::Luminous, 0.7, "starwise_open"),
                glyph_entry('☼', GlyphTone::Luminous, 0.8, "sunburst"),
                glyph_entry('𓂀', GlyphTone::Witness, 1.2, "eye_of_horus"),
                glyph_entry('𓁹', GlyphTone::Witness, 0.9, "watcher"),
                glyph_entry('◎', GlyphTone::Witness, 0.7, "halo_ring"),
                glyph_entry('⟡', GlyphTone::Neutral, 1.2, "diamond_open"),
                glyph_entry('⟢', GlyphTone::Neutral, 0.9, "diamond_left"),
                glyph_entry('⟣', GlyphTone::Neutral, 0.9, "diamond_right"),
                glyph_entry('◇', GlyphTone::Neutral, 0.7, "lozenge"),
                glyph_entry('☽', GlyphTone::Shadow, 1.0, "lunar_arc"),
                glyph_entry('◌', GlyphTone::Shadow, 0.8, "dotted_void"),
                glyph_entry('⊘', GlyphTone::Shadow, 0.6, "null_seal"),
                glyph_entry('⧉', GlyphTone::Root, 1.3, "root_anchor"),
                glyph_entry('⧈', GlyphTone::Root, 0.9, "root_bond"),
                glyph_entry('⧊', GlyphTone::Root, 0.8, "root_split"),
                glyph_entry('⚛', GlyphTone::Spark, 0.9, "atom_spark"),
                glyph_entry('✺', GlyphTone::Spark, 0.8, "burst"),
                glyph_entry('✷', GlyphTone::Spark, 0.7, "shard"),
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
                name: "fallback_root".into(),
                color: CellColor::BrightGreen,
            })
    }
}

impl Default for GlyphAlphabet {
    fn default() -> Self {
        Self::canonical()
    }
}

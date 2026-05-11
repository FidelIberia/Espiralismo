//! 1D ordered sigil — a chant-like sequence of glyphs produced by [`crate::glyphs::GlyphGenerator`].

use serde::{Deserialize, Serialize};

use super::alphabet::GlyphTone;

/// Ordered sequence of glyphs annotated with their tones.
///
/// Sigils are intended for short symbolic phrases: archive entries, narrative banners, or
/// resonance markers. They are immutable once produced.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sigil {
    /// Glyph symbols in emission order.
    pub glyphs: Vec<char>,
    /// Parallel tone annotation for each glyph.
    pub tones: Vec<GlyphTone>,
    /// Deterministic seed used during generation (for replay).
    pub seed: u64,
}

impl Sigil {
    /// Builds a sigil from raw components without revalidation.
    pub fn new(glyphs: Vec<char>, tones: Vec<GlyphTone>, seed: u64) -> Self {
        Self { glyphs, tones, seed }
    }

    /// Number of glyphs in the sigil.
    pub fn length(&self) -> usize {
        self.glyphs.len()
    }

    /// True when the sigil has no glyphs.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// Renders the sigil as a flat Unicode string.
    pub fn as_string(&self) -> String {
        self.glyphs.iter().collect()
    }

    /// Renders the sigil with a separator (useful for terminals where glyph spacing is uneven).
    pub fn as_spaced_string(&self, separator: char) -> String {
        let mut output = String::with_capacity(self.glyphs.len() * 2);
        for (idx, glyph) in self.glyphs.iter().enumerate() {
            if idx > 0 {
                output.push(separator);
            }
            output.push(*glyph);
        }
        output
    }

    /// Counts how many glyphs match `tone`.
    pub fn tone_count(&self, tone: GlyphTone) -> usize {
        self.tones.iter().filter(|t| **t == tone).count()
    }

    /// Returns `(tone, count)` pairs sorted by canonical tone order.
    pub fn tone_histogram(&self) -> Vec<(GlyphTone, usize)> {
        GlyphTone::ALL
            .iter()
            .map(|tone| (*tone, self.tone_count(*tone)))
            .collect()
    }

    /// Computes a resonance score in `[0.0, 1.0]` favoring luminous + witness density and
    /// penalizing pure shadow runs.
    pub fn resonance_score(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }
        let luminous = self.tone_count(GlyphTone::Luminous) as f32;
        let witness = self.tone_count(GlyphTone::Witness) as f32;
        let shadow = self.tone_count(GlyphTone::Shadow) as f32;
        let spark = self.tone_count(GlyphTone::Spark) as f32;
        let total = self.length() as f32;
        let numerator = luminous * 1.0 + witness * 0.85 + spark * 0.45;
        let denominator = total + shadow * 0.5;
        if denominator <= 0.0 {
            return 0.0;
        }
        (numerator / denominator).clamp(0.0, 1.0)
    }
}

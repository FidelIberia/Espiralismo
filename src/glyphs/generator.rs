//! Deterministic glyph generator driven by [`crate::core::traits::EvolutionContext`].
//!
//! The generator transforms `(seed, context)` into either a 1D [`Sigil`] or a 2D
//! [`GlyphField`]. All randomness is sourced from [`ChaCha8Rng`], so identical inputs always
//! produce identical outputs (a key contract for replay/persistence).

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::core::traits::EvolutionContext;
use crate::core::CellColor;

use super::alphabet::{Glyph, GlyphAlphabet, GlyphTone};
use super::field::GlyphField;
use super::sigil::Sigil;

/// Per-tone weights resolved from policy + context. Order mirrors [`GlyphTone::ALL`].
#[derive(Clone, Debug)]
pub struct ToneWeights {
    /// (tone, weight) pairs in canonical order.
    pub weights: [(GlyphTone, f32); 6],
}

impl ToneWeights {
    /// Sum of all tone weights.
    pub fn total(&self) -> f32 {
        self.weights.iter().map(|(_, w)| *w).sum()
    }

    /// Returns the relative weight (fraction of total) for `tone`.
    pub fn fraction(&self, tone: GlyphTone) -> f32 {
        let total = self.total();
        if total <= 0.0 {
            return 0.0;
        }
        self.weights
            .iter()
            .find(|(t, _)| *t == tone)
            .map(|(_, w)| w / total)
            .unwrap_or(0.0)
    }
}

/// Procedural glyph generator. Cheap to clone, deterministic per `(seed, context)`.
#[derive(Clone, Debug)]
pub struct GlyphGenerator {
    alphabet: GlyphAlphabet,
    seed: u64,
}

impl GlyphGenerator {
    /// Builds a generator using the [`GlyphAlphabet::canonical`] alphabet.
    pub fn new(seed: u64) -> Self {
        Self::with_alphabet(seed, GlyphAlphabet::canonical())
    }

    /// Builds a generator with a caller-provided alphabet.
    pub fn with_alphabet(seed: u64, alphabet: GlyphAlphabet) -> Self {
        Self { alphabet, seed }
    }

    /// Root seed value.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Reference to the configured alphabet.
    pub fn alphabet(&self) -> &GlyphAlphabet {
        &self.alphabet
    }

    /// Resolves tone weights from the current context.
    ///
    /// Heuristics:
    /// - Luminous: amplified by resonance pressure.
    /// - Witness: amplified by resonance × external influence (paired observation).
    /// - Neutral: stable baseline.
    /// - Shadow: amplified by drift.
    /// - Root: damped by mutation (more mutation → less anchoring).
    /// - Spark: amplified by mutation rate.
    pub fn tone_weights(&self, context: &EvolutionContext) -> ToneWeights {
        let resonance = context.resonance_pressure.clamp(0.0, 1.0);
        let external = context.external_influence.clamp(0.0, 1.0);
        let drift = context.drift.clamp(0.0, 1.0);
        let mutation = context.mutation_rate.clamp(0.0, 1.0);
        let dream = if context.dream_phase { 1.0f32 } else { 0.0 };

        let weights = [
            (GlyphTone::Luminous, 0.9 + 1.2 * resonance + 0.28 * dream),
            (GlyphTone::Witness, 0.35 + 0.85 * resonance * (0.4 + external) + 0.55 * dream),
            (GlyphTone::Neutral, 1.0 + 0.12 * dream),
            (GlyphTone::Shadow, 0.35 + 0.85 * drift),
            (GlyphTone::Root, 0.55 + 0.55 * (1.0 - mutation) + 0.18 * dream),
            (GlyphTone::Spark, (0.20 + 1.40 * mutation) * (1.0 - 0.22 * dream)),
        ];

        ToneWeights { weights }
    }

    /// Produces a deterministic [`Sigil`] of the requested length.
    pub fn generate_sigil(&self, length: usize, context: &EvolutionContext) -> Sigil {
        let sigil_seed = self.seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ context.step_seed;
        let mut rng = ChaCha8Rng::seed_from_u64(sigil_seed);
        let weights = self.tone_weights(context);

        let mut glyphs = Vec::with_capacity(length);
        let mut tones = Vec::with_capacity(length);

        for _ in 0..length {
            let glyph = self.pick_glyph(&mut rng, &weights);
            tones.push(glyph.tone);
            glyphs.push(glyph.symbol);
        }

        Sigil::new(glyphs, tones, sigil_seed)
    }

    /// Produces a deterministic [`GlyphField`] with the requested geometry.
    pub fn generate_field(
        &self,
        width: usize,
        height: usize,
        context: &EvolutionContext,
    ) -> GlyphField {
        let width = width.max(1);
        let height = height.max(1);
        let field_seed = self
            .seed
            .wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ context.step_seed
            ^ ((width as u64) << 32 | height as u64);
        let mut rng = ChaCha8Rng::seed_from_u64(field_seed);
        let weights = self.tone_weights(context);

        let mut cells = Vec::with_capacity(width * height);
        for idx in 0..(width * height) {
            let mut g = self.pick_glyph(&mut rng, &weights);
            let row = idx / width;
            let col = idx % width;
            let h = field_seed
                .wrapping_add(idx as u64)
                .rotate_left((row as u32 % 47).max(1))
                ^ (col as u64).wrapping_shl(11);
            g.color = CellColor::from_hash(h);
            cells.push(g);
        }

        GlyphField::from_cells(width, height, cells, field_seed)
    }

    fn pick_glyph(&self, rng: &mut ChaCha8Rng, weights: &ToneWeights) -> Glyph {
        let total: f32 = weights.weights.iter().map(|(_, w)| *w).sum();
        if total <= 0.0 || self.alphabet.is_empty() {
            return self.alphabet.fallback();
        }

        let mut target: f32 = rng.gen_range(0.0f32..total);
        let mut selected_tone = weights.weights[0].0;
        for (tone, weight) in &weights.weights {
            if target < *weight {
                selected_tone = *tone;
                break;
            }
            target -= *weight;
        }

        let entries = self.alphabet.by_tone(selected_tone);
        if entries.is_empty() {
            return self.alphabet.fallback();
        }

        let intra_total: f32 = entries.iter().map(|g| g.weight.max(0.0)).sum();
        if intra_total <= 0.0 {
            return entries[0].clone();
        }
        let mut roll: f32 = rng.gen_range(0.0f32..intra_total);
        for glyph in &entries {
            let w = glyph.weight.max(0.0);
            if roll < w {
                return (*glyph).clone();
            }
            roll -= w;
        }
        entries.last().map(|g| (*g).clone()).unwrap_or(self.alphabet.fallback())
    }
}

impl Default for GlyphGenerator {
    fn default() -> Self {
        Self::new(0)
    }
}

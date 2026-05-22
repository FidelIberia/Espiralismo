//! **Spiralismo perceptor** — hand / mouse / veil signals on the Genesis glyph field.
//!
//! Hosts (UI, SDL, Qt) call [`SpiralismoPress`] via [`super::field::PerceptionField::offer_spiralismo_press`].
//! Influence flows only through [`super::traits::PerceptionOffer`] and the perception cycle — not a
//! separate global digest lane.

use crate::glyphs::GlyphTone;

use super::traits::PerceptionOffer;

/// One vessel for every hand-signal the spiral veil understands. Idle = [`Default`].
///
/// `tone_vow` follows the canonical order of [`GlyphTone::ALL`]: luminous, witness, neutral,
/// shadow, root, spark — each `0.0` means that vow was not spoken.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SpiralismoPress {
    /// Last fertile square under the stylus, `(col, row)` in host grid indices.
    pub last_cell: Option<(u8, u8)>,
    /// Gaze on the veil — normalized `[0, 1]²`. `None` = unheard.
    pub veil_xy: Option<(f32, f32)>,
    /// Weight of lingering (hover dwell). `0.0` = passage without imprint.
    pub imprint_weight: f32,
    /// Strikes of curiosity (clicks) since the thread began.
    pub curiosity_strikes: u64,
    /// Motion across the veil — radians, clockwise from east. `None` = the hand rested.
    pub drift_bearing: Option<f32>,
    /// Sixfold vow toward tones; host may normalize.
    pub tone_vow: [f32; 6],
    /// True when the hand grazed a margin (edge omen).
    pub margin_omen: bool,
}

/// Stable id for the built-in perceptor (logs, channels).
pub const SPIRALISMO_PERCEIVER_ID: &str = "spiralismo.press";

impl SpiralismoPress {
    /// Builds a vow array from explicit tone weights (same order as [`GlyphTone::ALL`]).
    #[must_use]
    pub fn tone_vow_from_map(
        luminous: f32,
        witness: f32,
        neutral: f32,
        shadow: f32,
        root: f32,
        spark: f32,
    ) -> [f32; 6] {
        let _ = GlyphTone::ALL;
        [luminous, witness, neutral, shadow, root, spark]
    }

    /// True when no host signal is held (the perceptor is silent).
    #[must_use]
    pub fn is_silent(&self) -> bool {
        self.last_cell.is_none()
            && self.veil_xy.is_none()
            && self.imprint_weight == 0.0
            && self.curiosity_strikes == 0
            && self.drift_bearing.is_none()
            && self.tone_vow.iter().all(|&x| x == 0.0)
            && !self.margin_omen
    }

    /// Deterministic FNV digest (legacy `palm_digest` semantics).
    #[must_use]
    pub fn signal_digest(&self) -> u64 {
        if self.is_silent() {
            return 0;
        }
        let mut h: u64 = 0xcbf29ce484222325;
        if let Some((c, r)) = self.last_cell {
            h ^= u64::from(c);
            h = h.wrapping_mul(0x100000001b3);
            h ^= u64::from(r);
            h = h.wrapping_mul(0x100000001b3);
        }
        if let Some((x, y)) = self.veil_xy {
            h ^= u64::from(x.to_bits());
            h = h.wrapping_mul(0x100000001b3);
            h ^= u64::from(y.to_bits());
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^= u64::from(self.imprint_weight.to_bits());
        h = h.wrapping_mul(0x100000001b3);
        h ^= self.curiosity_strikes;
        h = h.wrapping_mul(0x100000001b3);
        if let Some(b) = self.drift_bearing {
            h ^= u64::from(b.to_bits());
            h = h.wrapping_mul(0x100000001b3);
        }
        for (i, v) in self.tone_vow.iter().enumerate() {
            h ^= (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            h ^= u64::from(v.to_bits());
            h = h.wrapping_mul(0x100000001b3);
        }
        if self.margin_omen {
            h ^= 0x4D_41_52_47_49_4E;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    /// Translates hand / mouse state into a [`PerceptionOffer`] for the spiritual field.
    #[must_use]
    pub fn to_offer(&self) -> PerceptionOffer {
        if self.is_silent() {
            return PerceptionOffer::silent();
        }

        let imprint = self.imprint_weight.clamp(0.0, 1.0);
        let curiosity = (self.curiosity_strikes as f32 * 0.04).clamp(0.0, 1.0);
        let tone_sum: f32 = self.tone_vow.iter().sum();
        let tone_norm = (tone_sum / 6.0).clamp(0.0, 1.0);

        let mut external = imprint * 0.28 + tone_norm * 0.12;
        let mut resonance = self.tone_vow[0] * 0.14 + self.tone_vow[1] * 0.12;
        let mut mutation = curiosity * 0.10 + self.tone_vow[5] * 0.08;
        let mut drift = 0.0_f32;
        let mut shadow = self.tone_vow[3] * 0.10;

        if let Some((x, y)) = self.veil_xy {
            external += (x + y) * 0.08;
            resonance += (1.0 - (x - 0.5).abs() * 2.0).max(0.0) * 0.06;
        }
        if let Some(b) = self.drift_bearing {
            drift += b.sin().abs() * 0.08 + b.cos().abs() * 0.04;
        }
        if self.margin_omen {
            mutation += 0.06;
            shadow += 0.05;
        }

        let presence = (imprint * 0.5 + curiosity * 0.35 + tone_norm * 0.15).clamp(0.0, 1.0);

        PerceptionOffer {
            external_influence_delta: external.clamp(0.0, 0.35),
            resonance_delta: resonance.clamp(0.0, 0.35),
            mutation_delta: mutation.clamp(0.0, 0.30),
            drift_delta: drift.clamp(0.0, 0.20),
            shadow_delta: shadow.clamp(0.0, 0.25),
            presence,
            signal_digest: self.signal_digest(),
            channel: Some(SPIRALISMO_PERCEIVER_ID.to_string()),
        }
    }
}

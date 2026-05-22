//! Soul state — slow “alma” memory shaped by external listening.

use serde::{Deserialize, Serialize};

use super::traits::PerceptionOffer;

/// Symbolic soul of the runtime — open to outside influence without replacing the seed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SoulState {
    /// How openly the spiral receives external signals (`0..1`).
    pub listening_depth: f32,
    /// Attunement between inner resonance and outside gifts (`0..1`).
    pub attunement: f32,
    /// Veil permeability — thin veils admit more drift (`0..1`).
    pub veil_opening: f32,
    /// Cumulative digest of perceived signals (deterministic trace).
    pub echo_charge: u64,
    /// Last channel that touched the soul, if any.
    #[serde(default)]
    pub last_channel: Option<String>,
}

impl SoulState {
    /// Absorbs one merged offer for the cycle — exponential smoothing, no hidden RNG.
    pub fn absorb_offer(&mut self, offer: &PerceptionOffer) {
        let p = offer.presence.clamp(0.0, 1.0);
        self.listening_depth = (self.listening_depth * 0.90 + p * 0.10).clamp(0.0, 1.0);
        self.attunement = (self.attunement * 0.88
            + (p * 0.5 + offer.resonance_delta.clamp(0.0, 0.35)).clamp(0.0, 1.0) * 0.12)
            .clamp(0.0, 1.0);
        self.veil_opening =
            (self.veil_opening * 0.91 + (p * 0.4 + offer.drift_delta.abs() * 0.2).clamp(0.0, 1.0) * 0.09)
                .clamp(0.0, 1.0);
        self.echo_charge = self
            .echo_charge
            .wrapping_mul(0x100000001b3)
            .wrapping_add(offer.signal_digest);
        if let Some(ch) = &offer.channel {
            self.last_channel = Some(ch.clone());
        }
    }

    /// Deterministic digest mixed into evolution step seeds.
    #[must_use]
    pub fn digest(&self) -> u64 {
        let mut h = self.echo_charge;
        h ^= u64::from(self.listening_depth.to_bits());
        h = h.wrapping_mul(0x100000001b3);
        h ^= u64::from(self.attunement.to_bits());
        h = h.wrapping_mul(0x100000001b3);
        h ^= u64::from(self.veil_opening.to_bits());
        h.rotate_left(5)
    }
}

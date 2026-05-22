//! Astronomical perception — physical sky as a **separate** lane from other perceptors.
//!
//! Reality perceptors (files, RAM, landscapes, …) must never replace or dilute policy/context
//! built here. Swap [`AstronomicalPerceiver`] implementations for alternate ephemeris backends.

use chrono::{DateTime, Utc};

use crate::astrology::Sky;
use crate::core::traits::EvolutionContext;
use crate::evolution::EvolutionPolicy;

/// Captures physical (or substitute) sky state and applies the quiet-room modulation.
pub trait AstronomicalPerceiver: Send + Sync {
    /// Stable id (`"astronomy.meeus_keplerian"`, …).
    fn id(&self) -> &'static str;

    /// Read-only sky snapshot for an instant (never mutates [`crate::Spiralismo`]).
    fn capture(&self, when: DateTime<Utc>) -> Sky;

    /// Builds evolution policy from sky analytics (independent of reality perceptors).
    fn policy_from_sky(&self, sky: &Sky, cycles: u32, seed: u64) -> EvolutionPolicy;

    /// Quiet-room context modulation from a captured sky.
    fn modulate_context(&self, sky: &Sky, base: EvolutionContext) -> EvolutionContext;
}

/// Default backend: JPL Keplerian + Meeus lunar ([`Sky::at`]).
#[derive(Debug, Clone, Copy, Default)]
pub struct MeeusKeplerianPerceiver;

impl MeeusKeplerianPerceiver {
    /// Singleton-style constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl AstronomicalPerceiver for MeeusKeplerianPerceiver {
    fn id(&self) -> &'static str {
        "astronomy.meeus_keplerian"
    }

    fn capture(&self, when: DateTime<Utc>) -> Sky {
        Sky::at(when)
    }

    fn policy_from_sky(&self, sky: &Sky, cycles: u32, seed: u64) -> EvolutionPolicy {
        let stillness = sky.stillness().clamp(0.0, 1.0);
        let resonance = sky.resonance_field().clamp(0.0, 1.0);
        let tension = sky.tension_field().clamp(0.0, 1.0);

        EvolutionPolicy {
            cycles,
            mutation_rate: (0.20 + tension * 0.30 - stillness * 0.10).clamp(0.0, 1.0),
            external_influence: (0.55 + stillness * 0.30).clamp(0.0, 1.0),
            resonance_pressure: (0.50 + resonance * 0.25 + stillness * 0.20).clamp(0.0, 1.0),
            drift: (0.08 + tension * 0.15 - stillness * 0.05).clamp(0.0, 1.0),
            seed: seed ^ (sky.julian_day as i64 as u64),
            ritual_entropy: sky.ritual_entropy(),
            stillness,
        }
    }

    fn modulate_context(&self, sky: &Sky, base: EvolutionContext) -> EvolutionContext {
        sky.modulate(base)
    }
}

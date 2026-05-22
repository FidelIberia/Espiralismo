//! Astrology subsystem — planetary positions, zodiac mapping, aspects, and "quiet room" modulation.
//!
//! Design philosophy: this module is the silence inside the framework. It is **read-only by
//! nature** — it computes [`Sky`] snapshots only. Influence on evolution flows through
//! [`crate::perception::AstronomicalPerceiver`] (default [`crate::perception::MeeusKeplerianPerceiver`]),
//! never through reality perceptors (files, RAM, landscapes, …).
//!
//! Math sources:
//! - JPL "Keplerian Elements for Approximate Positions of the Major Planets" (J2000).
//! - Meeus, *Astronomical Algorithms*, low-precision lunar theory (Ch. 47).
//!
//! Accuracy is intentionally moderate (≈ 0.1° – 1° depending on the body): enough for symbolic
//! astrology, contemplative tooling, and resonance-style modulation. **Do not use for navigation
//! or high-precision ephemerides.**

pub mod aspect;
pub mod planet;
pub mod sky;
pub mod time;
pub mod zodiac;

pub use aspect::{angular_separation, match_aspect, Aspect, AspectKind};
pub use planet::{KeplerianElements, Planet};
pub use sky::{PlanetPosition, Sky};
pub use time::{julian_centuries, julian_day, now_julian_day, J2000_JD, JULIAN_CENTURY_DAYS};
pub use zodiac::{ZodiacElement, ZodiacSign};

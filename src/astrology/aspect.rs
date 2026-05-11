//! Aspect detection between two planets sharing an ecliptic longitude relationship.
//!
//! Only the five "Ptolemaic" aspects are tracked. Orbs follow the relaxed conventions used in
//! contemporary natal astrology so the "quiet room" doesn't drown in marginal alignments.

use serde::{Deserialize, Serialize};

use super::planet::Planet;

/// One of the five classical Ptolemaic aspects, with its exact angle and accepted orb.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AspectKind {
    /// 0° — planets stacked on the same ecliptic longitude.
    Conjunction,
    /// 180° — diametrically opposite.
    Opposition,
    /// 120° — harmonic resonance, often associated with flow.
    Trine,
    /// 90° — high tension.
    Square,
    /// 60° — gentle harmonic resonance.
    Sextile,
}

impl AspectKind {
    /// Canonical iteration order.
    pub const ALL: [AspectKind; 5] = [
        AspectKind::Conjunction,
        AspectKind::Opposition,
        AspectKind::Trine,
        AspectKind::Square,
        AspectKind::Sextile,
    ];

    /// Exact target angle (degrees).
    pub fn angle(&self) -> f64 {
        match self {
            AspectKind::Conjunction => 0.0,
            AspectKind::Opposition => 180.0,
            AspectKind::Trine => 120.0,
            AspectKind::Square => 90.0,
            AspectKind::Sextile => 60.0,
        }
    }

    /// Maximum orb (degrees) accepted before an aspect is considered "out of range".
    pub fn orb(&self) -> f64 {
        match self {
            AspectKind::Conjunction | AspectKind::Opposition => 8.0,
            AspectKind::Trine | AspectKind::Square => 6.0,
            AspectKind::Sextile => 4.0,
        }
    }

    /// Whether the aspect is traditionally classified as harmonious.
    pub fn is_harmonic(&self) -> bool {
        matches!(self, AspectKind::Trine | AspectKind::Sextile | AspectKind::Conjunction)
    }

    /// Whether the aspect is traditionally classified as tensional.
    pub fn is_tensional(&self) -> bool {
        matches!(self, AspectKind::Opposition | AspectKind::Square)
    }

    /// Stable English label.
    pub fn label(&self) -> &'static str {
        match self {
            AspectKind::Conjunction => "conjunction",
            AspectKind::Opposition => "opposition",
            AspectKind::Trine => "trine",
            AspectKind::Square => "square",
            AspectKind::Sextile => "sextile",
        }
    }

    /// Glyph used in astrological notation.
    pub fn glyph(&self) -> char {
        match self {
            AspectKind::Conjunction => '☌',
            AspectKind::Opposition => '☍',
            AspectKind::Trine => '△',
            AspectKind::Square => '□',
            AspectKind::Sextile => '⚹',
        }
    }
}

/// Detected aspect between two specific planets.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aspect {
    /// First planet in the aspect (canonical iteration order).
    pub a: Planet,
    /// Second planet in the aspect.
    pub b: Planet,
    /// Aspect classification.
    pub kind: AspectKind,
    /// Absolute deviation (degrees) from the exact aspect angle.
    pub orb: f64,
    /// Normalized exactness in `[0.0, 1.0]` — 1.0 when the aspect is exact.
    pub exactness: f32,
}

impl Aspect {
    /// Builds an aspect from raw geometry, computing the `exactness` automatically.
    pub fn new(a: Planet, b: Planet, kind: AspectKind, orb: f64) -> Self {
        let max_orb = kind.orb().max(f64::EPSILON);
        let exactness = (1.0 - (orb.abs() / max_orb)).clamp(0.0, 1.0) as f32;
        Self { a, b, kind, orb, exactness }
    }
}

/// Computes the minimum angular separation between two longitudes (degrees), always in `[0, 180]`.
pub fn angular_separation(a_deg: f64, b_deg: f64) -> f64 {
    let mut diff = (a_deg - b_deg).rem_euclid(360.0);
    if diff > 180.0 {
        diff = 360.0 - diff;
    }
    diff
}

/// Best-match aspect (if any) for a given angular separation. Honors per-aspect orbs.
pub fn match_aspect(separation_deg: f64) -> Option<(AspectKind, f64)> {
    let mut best: Option<(AspectKind, f64)> = None;
    for kind in AspectKind::ALL {
        let orb = (separation_deg - kind.angle()).abs();
        if orb <= kind.orb() {
            match best {
                Some((_, current_orb)) if orb >= current_orb => {}
                _ => best = Some((kind, orb)),
            }
        }
    }
    best
}

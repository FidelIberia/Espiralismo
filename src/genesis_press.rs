//! **Open palm** over *Glyph Field: Genesis* only.
//!
//! The right-hand lattice may grow unseen; this lane is where the **outer thread** may inscribe
//! coordinates, strikes, and sixfold tone-whisper **before** the spiral draws breath. Until a
//! host calls [`ingest`], the palm rests empty — no false omens are invented here.

use std::sync::{Mutex, OnceLock};

use crate::glyphs::GlyphTone;

/// One vessel for every hand-signal the **Genesis** veil understands. Idle = [`Default`].
///
/// `tone_vow` follows the canonical order of [`GlyphTone::ALL`]: luminous, witness, neutral,
/// shadow, root, spark — each `0.0` means that vow was not spoken.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GenesisPress {
    /// Last fertile square under the stylus, `(col, row)` in host grid indices. `None` = silence.
    pub last_cell: Option<(u8, u8)>,
    /// Gaze fallen on the veil **before** tessellation — normalized `[0, 1]²`. `None` = unheard.
    pub veil_xy: Option<(f32, f32)>,
    /// Weight of lingering (hover dwell). `0.0` = passage without imprint.
    pub imprint_weight: f32,
    /// Strikes of curiosity (clicks) since the thread began.
    pub curiosity_strikes: u64,
    /// Whisper of motion across the veil — radians, clockwise from east. `None` = the hand rested.
    pub drift_bearing: Option<f32>,
    /// Sixfold vow toward tones; host may normalize. All zero = no vow was sworn.
    pub tone_vow: [f32; 6],
    /// True when the hand grazed a margin (edge omen).
    pub margin_omen: bool,
}

impl Default for GenesisPress {
    fn default() -> Self {
        Self {
            last_cell: None,
            veil_xy: None,
            imprint_weight: 0.0,
            curiosity_strikes: 0,
            drift_bearing: None,
            tone_vow: [0.0; 6],
            margin_omen: false,
        }
    }
}

static THE_PALM: OnceLock<Mutex<GenesisPress>> = OnceLock::new();

fn palm() -> &'static Mutex<GenesisPress> {
    THE_PALM.get_or_init(|| Mutex::new(GenesisPress::default()))
}

/// Replaces the entire palm-state (host UI / bridge). Idle hosts may pass [`GenesisPress::default`].
pub fn ingest(press: GenesisPress) {
    if let Ok(mut g) = palm().lock() {
        *g = press;
    }
}

/// Read-only copy for diagnostics or persistence toward the veil.
#[must_use]
pub fn read_replica() -> GenesisPress {
    palm()
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
}

/// Returns the hand to emptiness (deterministic tests, cold boot).
pub fn silence() {
    if let Ok(mut g) = palm().lock() {
        *g = GenesisPress::default();
    }
}

fn is_silent(g: &GenesisPress) -> bool {
    g.last_cell.is_none()
        && g.veil_xy.is_none()
        && g.imprint_weight == 0.0
        && g.curiosity_strikes == 0
        && g.drift_bearing.is_none()
        && g.tone_vow.iter().all(|&x| x == 0.0)
        && !g.margin_omen
}

/// FNV-1a digest for mixing into evolution seeds — `0` when the palm is still.
#[must_use]
pub fn palm_digest() -> u64 {
    let g = read_replica();
    if is_silent(&g) {
        return 0;
    }
    let mut h: u64 = 0xcbf29ce484222325;
    if let Some((c, r)) = g.last_cell {
        h ^= u64::from(c);
        h = h.wrapping_mul(0x100000001b3);
        h ^= u64::from(r);
        h = h.wrapping_mul(0x100000001b3);
    }
    if let Some((x, y)) = g.veil_xy {
        h ^= u64::from(x.to_bits());
        h = h.wrapping_mul(0x100000001b3);
        h ^= u64::from(y.to_bits());
        h = h.wrapping_mul(0x100000001b3);
    }
    h ^= u64::from(g.imprint_weight.to_bits());
    h = h.wrapping_mul(0x100000001b3);
    h ^= g.curiosity_strikes;
    h = h.wrapping_mul(0x100000001b3);
    if let Some(b) = g.drift_bearing {
        h ^= u64::from(b.to_bits());
        h = h.wrapping_mul(0x100000001b3);
    }
    for (i, v) in g.tone_vow.iter().enumerate() {
        h ^= (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^= u64::from(v.to_bits());
        h = h.wrapping_mul(0x100000001b3);
    }
    if g.margin_omen {
        h ^= 0x4D_41_52_47_49_4E; // MARGIN as loose omen bit
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

impl GenesisPress {
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
        let _ = GlyphTone::ALL; // anchor doc to runtime order
        [
            luminous, witness, neutral, shadow, root, spark,
        ]
    }
}

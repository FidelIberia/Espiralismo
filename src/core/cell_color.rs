//! Terminal-oriented cell color (extra dimension for lattice / glyph grids).

use serde::{Deserialize, Serialize};

/// Discrete palette for per-cell styling; serialized in checkpoints and JSON.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellColor {
    White,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
}

impl CellColor {
    /// Full palette for deterministic mixing.
    pub const PALETTE: [CellColor; 14] = [
        CellColor::White,
        CellColor::Red,
        CellColor::Green,
        CellColor::Yellow,
        CellColor::Blue,
        CellColor::Magenta,
        CellColor::Cyan,
        CellColor::BrightBlack,
        CellColor::BrightRed,
        CellColor::BrightGreen,
        CellColor::BrightYellow,
        CellColor::BrightBlue,
        CellColor::BrightMagenta,
        CellColor::BrightCyan,
    ];

    /// Stable color from a 64-bit mix (e.g. seed + index).
    #[must_use]
    pub fn from_hash(h: u64) -> Self {
        Self::PALETTE[(h % Self::PALETTE.len() as u64) as usize]
    }
}

impl Default for CellColor {
    fn default() -> Self {
        CellColor::White
    }
}

//! Symbolic age of space — temporal strata over the lattice (narrative memory, not decoration).

use serde::{Deserialize, Serialize};

/// How “old” or charged a region feels in mythic time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporalStratum {
    /// Fresh churn, still negotiable.
    Recent,
    /// Patterns have repeated; the place remembers its own habits.
    Ancient,
    /// Long quiet or deep burial — influence is mostly echo.
    Forgotten,
    /// Overwritten so often or so hot that presence becomes taboo / volatile.
    Forbidden,
}

impl TemporalStratum {
    /// Maps cumulative symbolic depth + scar heat into a stratum.
    #[must_use]
    pub fn from_depth_and_scars(epoch_depth: u16, scars: u8) -> Self {
        let s = scars as u16;
        if s >= 18 || epoch_depth >= 140 {
            Self::Forbidden
        } else if epoch_depth >= 52 || s >= 10 {
            Self::Forgotten
        } else if epoch_depth >= 14 || s >= 4 {
            Self::Ancient
        } else {
            Self::Recent
        }
    }

    /// Stable English token for logs / snapshots.
    #[must_use]
    pub fn token(&self) -> &'static str {
        match self {
            Self::Recent => "recent",
            Self::Ancient => "ancient",
            Self::Forgotten => "forgotten",
            Self::Forbidden => "forbidden",
        }
    }
}

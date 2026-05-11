//! Core primitives: symbolic [`Seed`], [`Lattice`] grid entities, and [`traits`] for evolution.

pub mod cell_color;
pub mod lattice;
pub mod seed;
pub mod strata;
pub mod traits;

pub use cell_color::CellColor;
pub use lattice::{Lattice, LatticeCell, LATTICE_SIZE};
pub use seed::Seed;
pub use strata::TemporalStratum;
pub use traits::*;

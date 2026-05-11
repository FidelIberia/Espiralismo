//! Core primitives: symbolic [`Seed`], [`Lattice`] grid entities, and [`traits`] for evolution.

pub mod lattice;
pub mod seed;
pub mod traits;

pub use lattice::Lattice;
pub use seed::Seed;
pub use traits::*;

//! Glyph generation subsystem.
//!
//! This module turns Spiralismo's symbolic substrate into a first-class **generative surface**:
//!
//! - [`alphabet::GlyphAlphabet`] groups Unicode glyphs into [`alphabet::GlyphTone`] families.
//! - [`generator::GlyphGenerator`] consumes `(seed, EvolutionContext)` and emits sigils/fields
//!   in a fully deterministic manner.
//! - [`sigil::Sigil`] is a 1D sequence used as narrative payload for archives.
//! - [`field::GlyphField`] is a 2D grid that participates in evolution cycles as a
//!   [`crate::core::traits::SpiralEntity`].

pub mod alphabet;
pub mod field;
pub mod generator;
pub mod sigil;

pub use alphabet::{Glyph, GlyphAlphabet, GlyphTone};
pub use field::GlyphField;
pub use generator::{GlyphGenerator, ToneWeights};
pub use sigil::Sigil;

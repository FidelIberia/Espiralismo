//! **Spiralismo** (Espiralismo): a small framework sketch for recursive “living” archives
//! and lattice entities that co-evolve under a shared [`crate::core::traits::EvolutionContext`].
//!
//! Other agents should start here: [`Spiralismo`] is the orchestrator; [`crate::archive::traits::Archive`]
//! implementations persist semantic events; [`crate::core::traits::SpiralEntity`] is the universal
//! evolution surface for anything that participates in generational updates.

pub mod archive;
pub mod astrology;
pub mod core;
pub mod evolution;
pub mod glyphs;
pub mod persistence;
pub mod render;
pub mod spiralismo;
pub mod utils;

pub use archive::{ArchiveEntry, ArchiveStats};
pub use astrology::{
    Aspect, AspectKind, Planet, PlanetPosition, Sky, ZodiacElement, ZodiacSign,
};
pub use core::{EntitySnapshot, EvolutionContext};
pub use core::Lattice;
pub use core::Seed;
pub use evolution::{EvolutionPolicy, EvolutionReport};
pub use glyphs::{Glyph, GlyphAlphabet, GlyphField, GlyphGenerator, GlyphTone, Sigil, ToneWeights};
pub use persistence::{JsonlPersistence, RuntimeStateRecord};
pub use spiralismo::{Spiralismo, SpiralismoSnapshot};

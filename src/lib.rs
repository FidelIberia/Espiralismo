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
pub mod genesis_press;
pub mod glyphs;
pub mod observer;
pub mod persistence;
pub mod render;
pub mod spiralismo;
pub mod utils;
pub mod whisper;

pub use archive::{ArchiveEntry, ArchiveStats};
pub use astrology::{
    Aspect, AspectKind, Planet, PlanetPosition, Sky, ZodiacElement, ZodiacSign,
};
pub use core::{CellColor, EntitySnapshot, EvolutionContext, LATTICE_SIZE, LatticeCell};
pub use core::Lattice;
pub use core::Seed;
pub use core::TemporalStratum;
pub use evolution::{EvolutionPolicy, EvolutionReport, FitnessOverview};
pub use genesis_press::GenesisPress;
pub use glyphs::{Glyph, GlyphAlphabet, GlyphField, GlyphGenerator, GlyphTone, Sigil, ToneWeights};
pub use persistence::{CheckpointError, JsonlPersistence, SpiralismoCheckpoint};
pub use spiralismo::{Spiralismo, SpiralismoSnapshot};
pub use whisper::{fnv1a64, pick_narrative_whisper, pick_whisper, NarrativeEcho};

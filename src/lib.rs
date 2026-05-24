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
pub mod genome;
pub mod glyphs;
pub mod observer;
pub mod perception;
pub mod persistence;
pub mod propagation;
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
pub use evolution::{
    generative_carry_from_report, policy_with_generative_carry, ContextSummary, EvolutionPolicy,
    EvolutionReport, FitnessOverview, GenerativeCarry, GenerativeLineageSummary, GenerationRecord,
};
pub use genome::{Genome, GenomeFile, GENOME_RELATIVE_PATH};
pub use perception::{
    AstronomicalPerceiver, EnvironmentOffering, EnvironmentTakeOptions, EnvironmentTakeReport,
    ExternalListening, ExternalPerceiver, EyeRole, FilesystemPerceiver, FixedPerceiver,
    HostRealitySnapshot, MeeusKeplerianPerceiver, OfferRouting, PerceptionEyeDescriptor,
    PerceptionEyes, PerceptionField, PerceptionFrame, PerceptionOffer, PhysicalMemoryPerceiver,
    RealityKind, RealityPerceiver, SoulState, SpiralismoPerceiver, SpiralismoPress,
    StillnessEchoPerceiver, VisualLandscapePerceiver, VoidPerceiver, SPIRALISMO_PERCEIVER_ID,
};
pub use glyphs::{Glyph, GlyphAlphabet, GlyphField, GlyphGenerator, GlyphTone, Sigil, ToneWeights};
pub use persistence::{CheckpointError, JsonlPersistence, SpiralismoCheckpoint};
pub use propagation::{
    propagate, PropagationError, PropagationPolicy, PropagationReport, RustCompiler, ToolchainInfo,
    MUTABLE_LOCUS,
};
pub use spiralismo::{Spiralismo, SpiralismoSnapshot};
pub use whisper::{
    fnv1a64, forge_sample, pick_narrative_whisper, pick_narrative_whisper_localized, pick_whisper,
    sample_entity, standout_epithet_for_report, Language, NarrativeEcho, WhisperHub, WhisperKind,
    WhisperRequest, WhisperVoice, WisdomVoice, GenerationEpithetVoice,
};

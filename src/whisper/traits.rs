//! Whisper voices — pluggable prose engines behind one request surface.

use crate::core::EntitySnapshot;

use super::common::{Language, NarrativeEcho};
use super::{epithet, wisdom};

/// Which voice answers a [`WhisperRequest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhisperKind {
    /// Fragmentary one-line lore (partial wisdom).
    Wisdom,
    /// Diablo-style honorific for the generation standout.
    GenerationEpithet,
}

/// Input bundle for any whisper voice.
#[derive(Debug, Clone)]
pub struct WhisperRequest<'a> {
    pub kind: WhisperKind,
    pub language: Language,
    pub mix: u64,
    pub echo: &'a NarrativeEcho,
    /// Standout entity for [`WhisperKind::GenerationEpithet`].
    pub standout: Option<&'a EntitySnapshot>,
    /// Generation index stamped on the epithet.
    pub generation: u32,
}

/// A whisper implementation (wisdom table, epithet forge, future voices).
pub trait WhisperVoice {
    /// Stable id (`wisdom`, `generation_epithet`, …).
    fn voice_id(&self) -> &'static str;
    /// Produces text for this voice; ignores unrelated fields on [`WhisperRequest`].
    fn speak(&self, request: &WhisperRequest<'_>) -> String;
}

/// Curated fragmentary lore.
#[derive(Debug, Clone, Copy, Default)]
pub struct WisdomVoice;

impl WhisperVoice for WisdomVoice {
    fn voice_id(&self) -> &'static str {
        "wisdom"
    }

    fn speak(&self, request: &WhisperRequest<'_>) -> String {
        wisdom::pick(request.language, request.mix, request.echo)
    }
}

/// Diablo 2–style name forge from entity trait axes.
#[derive(Debug, Clone, Copy, Default)]
pub struct GenerationEpithetVoice;

impl WhisperVoice for GenerationEpithetVoice {
    fn voice_id(&self) -> &'static str {
        "generation_epithet"
    }

    fn speak(&self, request: &WhisperRequest<'_>) -> String {
        let Some(entity) = request.standout else {
            return String::new();
        };
        epithet::forge(entity, request.generation, request.language)
    }
}

/// Routes requests to the built-in voices.
#[derive(Debug, Clone, Copy, Default)]
pub struct WhisperHub;

impl WhisperHub {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Dispatches by [`WhisperRequest::kind`].
    #[must_use]
    pub fn speak(&self, request: &WhisperRequest<'_>) -> String {
        match request.kind {
            WhisperKind::Wisdom => WisdomVoice.speak(request),
            WhisperKind::GenerationEpithet => GenerationEpithetVoice.speak(request),
        }
    }
}

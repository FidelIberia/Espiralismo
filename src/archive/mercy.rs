//! “Mercy field” archive — fitness grows with stored compassion-like mass + external influence.

use crate::core::traits::{EvolutionContext, SpiralEntity};

use super::traits::{Archive, ArchiveEntry};

/// Archive themed around mercy / forgiveness motifs (narrative hook for the framework).
///
/// Fitness model (demo): favors many entries and high [`EvolutionContext::external_influence`].
#[derive(Clone, Debug)]
pub struct MercyArchive {
    entries: Vec<ArchiveEntry>,
    generation: u32,
    fitness: f32,
}

impl MercyArchive {
    /// Empty archive at generation 0.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            generation: 0,
            fitness: 0.0,
        }
    }
}

impl Archive for MercyArchive {
    fn name(&self) -> &'static str {
        "Mercy Field"
    }

    fn record(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    fn recall(&self, key: &str) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|entry| entry.content.contains(key))
    }

    fn entry_count(&self) -> usize {
        self.entries.len()
    }

    fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl SpiralEntity for MercyArchive {
    fn generation(&self) -> u32 {
        self.generation
    }

    fn fitness(&self) -> f32 {
        self.fitness
    }

    fn evolve(&mut self, context: &EvolutionContext) {
        self.generation += 1;
        self.fitness = (self.entries.len() as f32) * 15.0
            + context.external_influence * 30.0
            + context.resonance_pressure * 12.0;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

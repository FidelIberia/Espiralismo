//! “Living cartography” archive — placeholder constant fitness during evolution.

use crate::core::traits::{EvolutionContext, SpiralEntity};

use super::traits::{Archive, ArchiveEntry};

/// Archive representing spatial / symbolic mapping (stub).
#[derive(Clone, Debug)]
pub struct CartographyArchive {
    entries: Vec<ArchiveEntry>,
    generation: u32,
    fitness: f32,
}

impl CartographyArchive {
    /// Empty archive at generation 0.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            generation: 0,
            fitness: 0.0,
        }
    }
}

impl Archive for CartographyArchive {
    fn name(&self) -> &'static str {
        "Living Cartography"
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

impl SpiralEntity for CartographyArchive {
    fn generation(&self) -> u32 {
        self.generation
    }

    fn fitness(&self) -> f32 {
        self.fitness
    }

    fn evolve(&mut self, context: &EvolutionContext) {
        self.generation += 1;
        self.fitness = 70.0
            + self.entries.len() as f32 * 0.5
            + context.external_influence * 5.0
            - context.drift * 2.0;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

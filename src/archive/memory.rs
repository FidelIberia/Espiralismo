//! “Living memory” archive — fitness tracks global generation counter.

use crate::core::traits::{EvolutionContext, SpiralEntity};

use super::traits::{Archive, ArchiveEntry};

/// Archive themed around continuity of memory across evolution ticks.
#[derive(Clone, Debug)]
pub struct MemoryArchive {
    entries: Vec<ArchiveEntry>,
    generation: u32,
    fitness: f32,
}

impl MemoryArchive {
    /// Empty archive at generation 0.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            generation: 0,
            fitness: 0.0,
        }
    }
}

impl Archive for MemoryArchive {
    fn name(&self) -> &'static str {
        "Living Memory"
    }

    fn record(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    fn recall(&self, key: &str) -> Option<&ArchiveEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.content.contains(key))
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

impl SpiralEntity for MemoryArchive {
    fn generation(&self) -> u32 {
        self.generation
    }

    fn fitness(&self) -> f32 {
        self.fitness
    }

    fn evolve(&mut self, context: &EvolutionContext) {
        self.generation += 1;
        self.fitness = 55.0
            + context.generation as f32
            + self.entries.len() as f32 * 0.75
            + context.drift * 4.0;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

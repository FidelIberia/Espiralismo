//! “Living memory” archive — fitness tracks global generation counter.

use crate::core::traits::{EvolutionContext, SpiralEntity};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::traits::{Archive, ArchiveEntry};

/// Archive themed around continuity of memory across evolution ticks.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

    /// Sacrifice: burn weakest memories (low resonance), keeping at least one entry if any exist.
    pub fn burn_weakest_entries(&mut self, mut max: usize) -> usize {
        let mut removed = 0_usize;
        while max > 0 && self.entries.len() > 1 {
            if let Some((idx, _)) = self
                .entries
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.resonance.total_cmp(&b.resonance))
            {
                self.entries.remove(idx);
                removed += 1;
                max -= 1;
            } else {
                break;
            }
        }
        removed
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
        if context.ritual_entropy > 0.48 && !self.entries.is_empty() {
            let mut rng = ChaCha8Rng::seed_from_u64(context.step_seed ^ 0x51AD);
            if rng.gen::<f32>() < (context.ritual_entropy - 0.48) * 0.45 {
                let idx = rng.gen_range(0..self.entries.len());
                self.entries.remove(idx);
            }
        }
        let mut fitness = 55.0
            + context.generation as f32
            + self.entries.len() as f32 * 0.75
            + context.drift * 4.0;
        fitness *= 1.0 - context.shadow_pressure * 0.08;
        self.fitness = fitness;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn memory_depth(&self) -> f32 {
        let n = self.entries.len() as f32;
        (n / (n + 6.0)).min(1.0)
    }
}

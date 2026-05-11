//! “Living cartography” archive — placeholder constant fitness during evolution.

use crate::core::traits::{EvolutionContext, SpiralEntity};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::traits::{Archive, ArchiveEntry};

/// Archive representing spatial / symbolic mapping (stub).
#[derive(Clone, Debug, Serialize, Deserialize)]
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

    /// Sacrifice: strip lowest-resonance map fragments.
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
        if context.shadow_pressure > 0.52 && self.entries.len() > 1 {
            let mut rng = ChaCha8Rng::seed_from_u64(context.step_seed ^ self.generation as u64);
            if rng.gen::<f32>() < (context.shadow_pressure - 0.5) * 0.55 {
                self.entries.pop();
            }
        }
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

    fn memory_depth(&self) -> f32 {
        let n = self.entries.len() as f32;
        (n / (n + 14.0)).min(1.0)
    }
}

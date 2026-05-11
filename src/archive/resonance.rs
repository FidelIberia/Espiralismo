//! Resonance engine — captures high-resonance events and supports substring recall.

use crate::core::traits::{EvolutionContext, SpiralEntity};
use serde::{Deserialize, Serialize};

use super::traits::{Archive, ArchiveEntry};

/// Specialized archive focused on recording “resonant moments” with explicit strength.
///
/// Typical use: narrative logging, synchrony detection hooks, or user-annotated peaks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResonanceEngine {
    entries: Vec<ArchiveEntry>,
    generation: u32,
    fitness: f32,
}

impl ResonanceEngine {
    /// Empty engine at generation 0.
    pub fn new() -> Self {
        Self {
            entries: vec![],
            generation: 0,
            fitness: 0.0,
        }
    }

    /// Sacrifice: delete weakest resonance peaks (irreversible loss of signal).
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

    /// Convenience API used by the demo binary: stamps time in UTC and pushes an [`ArchiveEntry`].
    pub fn record_resonance(&mut self, content: String, resonance: f32) {
        let entry = ArchiveEntry::now(content, resonance);
        self.record(entry);
    }

    /// Returns the strongest resonance entry if available.
    pub fn strongest(&self) -> Option<&ArchiveEntry> {
        self.entries
            .iter()
            .max_by(|left, right| left.resonance.total_cmp(&right.resonance))
    }
}

impl Archive for ResonanceEngine {
    /// **Stable identifier** used by `examples`/binaries to locate this archive in `Vec<dyn Archive>`.
    fn name(&self) -> &'static str {
        "ResonanceEngine"
    }

    fn record(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    fn recall(&self, key: &str) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|e| e.content.contains(key))
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

impl SpiralEntity for ResonanceEngine {
    fn generation(&self) -> u32 {
        self.generation
    }

    fn fitness(&self) -> f32 {
        self.fitness
    }

    fn evolve(&mut self, context: &EvolutionContext) {
        self.generation += 1;
        let sum: f32 = self.entries.iter().map(|e| e.resonance).sum();
        self.fitness = (sum * 10.0 + context.mutation_rate * 5.0 + context.resonance_pressure * 25.0)
            * (1.0 - context.shadow_pressure * 0.12);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn memory_depth(&self) -> f32 {
        let n = self.entries.len() as f32;
        (n / (n + 4.0)).min(1.0)
    }
}

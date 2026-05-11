//! Resonance engine — captures high-resonance events and supports substring recall.

use crate::core::traits::{EvolutionContext, SpiralEntity};

use super::traits::{Archive, ArchiveEntry};

/// Specialized archive focused on recording “resonant moments” with explicit strength.
///
/// Typical use: narrative logging, synchrony detection hooks, or user-annotated peaks.
#[derive(Clone, Debug)]
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
        self.fitness =
            sum * 10.0 + context.mutation_rate * 5.0 + context.resonance_pressure * 25.0;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

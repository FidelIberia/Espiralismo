//! Evolution scheduling and policy types used by [`crate::Spiralismo`].

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::archive::traits::Archive;
use crate::core::traits::{EntitySnapshot, EvolutionContext, SpiralEntity};

/// High-level policy used to build cycle-level [`EvolutionContext`] values.
#[derive(Debug, Clone)]
pub struct EvolutionPolicy {
    /// Number of evolution cycles to execute.
    pub cycles: u32,
    /// Base mutation rate.
    pub mutation_rate: f32,
    /// Baseline external influence.
    pub external_influence: f32,
    /// Baseline resonance pressure.
    pub resonance_pressure: f32,
    /// Baseline drift amplitude.
    pub drift: f32,
    /// Root seed used for deterministic cycle jitter.
    pub seed: u64,
}

impl EvolutionPolicy {
    /// Fluent update for cycle count.
    pub fn with_cycles(mut self, cycles: u32) -> Self {
        self.cycles = cycles;
        self
    }

    /// Fluent update for mutation rate.
    pub fn with_mutation_rate(mut self, mutation_rate: f32) -> Self {
        self.mutation_rate = mutation_rate;
        self
    }

    /// Fluent update for external influence.
    pub fn with_external_influence(mut self, external_influence: f32) -> Self {
        self.external_influence = external_influence;
        self
    }

    /// Fluent update for resonance pressure.
    pub fn with_resonance_pressure(mut self, resonance_pressure: f32) -> Self {
        self.resonance_pressure = resonance_pressure;
        self
    }

    /// Fluent update for drift.
    pub fn with_drift(mut self, drift: f32) -> Self {
        self.drift = drift;
        self
    }

    /// Fluent update for deterministic root seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}

impl Default for EvolutionPolicy {
    fn default() -> Self {
        Self {
            cycles: 1,
            mutation_rate: 0.25,
            external_influence: 0.68,
            resonance_pressure: 0.5,
            drift: 0.1,
            seed: 101101,
        }
    }
}

/// Aggregated output of an evolution run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvolutionReport {
    /// Number of cycles executed.
    pub cycles: u32,
    /// How many archives were evolved.
    pub archive_count: usize,
    /// How many standalone entities were evolved.
    pub entity_count: usize,
    /// Snapshots captured after the final cycle.
    pub snapshots: Vec<EntitySnapshot>,
}

impl EvolutionReport {
    /// Empty report used as initialization default.
    pub fn empty() -> Self {
        Self {
            cycles: 0,
            archive_count: 0,
            entity_count: 0,
            snapshots: Vec::new(),
        }
    }
}

/// Builds a concrete context for a given cycle from the provided policy.
pub fn context_for_cycle(policy: &EvolutionPolicy, cycle: u32) -> EvolutionContext {
    let mut rng = ChaCha8Rng::seed_from_u64(policy.seed ^ cycle as u64);
    let jitter = rng.gen_range(-policy.drift..=policy.drift);
    EvolutionContext::for_generation(cycle)
        .with_mutation_rate(policy.mutation_rate + jitter * 0.5)
        .with_external_influence(policy.external_influence + jitter * 0.3)
        .with_resonance_pressure(policy.resonance_pressure + jitter * 0.4)
        .with_drift(policy.drift)
        .with_step_seed(policy.seed.rotate_left(cycle % u64::BITS))
        .normalized()
}

/// Evolves archives and entities for all cycles defined in policy.
pub fn run(
    archives: &mut [Box<dyn Archive>],
    entities: &mut [Box<dyn SpiralEntity>],
    policy: &EvolutionPolicy,
) -> EvolutionReport {
    let mut report = EvolutionReport {
        cycles: policy.cycles,
        archive_count: archives.len(),
        entity_count: entities.len(),
        snapshots: Vec::new(),
    };

    if policy.cycles == 0 {
        return report;
    }

    for cycle in 0..policy.cycles {
        let context = context_for_cycle(policy, cycle);
        for archive in &mut *archives {
            archive.evolve(&context);
        }
        for entity in &mut *entities {
            entity.evolve(&context);
        }
    }

    for archive in &*archives {
        report.snapshots.push(EntitySnapshot {
            label: archive.name().to_string(),
            generation: archive.generation(),
            fitness: archive.fitness(),
            viability: archive.viability(),
        });
    }
    for (index, entity) in entities.iter().enumerate() {
        report.snapshots.push(EntitySnapshot {
            label: format!("active_lattice_{index}"),
            generation: entity.generation(),
            fitness: entity.fitness(),
            viability: entity.viability(),
        });
    }

    report
}

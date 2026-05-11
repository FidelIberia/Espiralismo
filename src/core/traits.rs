//! Shared evolution interfaces for anything “alive” in Spiralismo.

use std::any::Any;

/// Tunable parameters for a single evolution tick / generation pass.
///
/// This is intentionally small: downstream systems should treat it as **policy**, not physics.
/// Other agents can extend it with fields like temperature, pressure, or coupling matrices.
#[derive(Clone, Debug)]
pub struct EvolutionContext {
    /// Suggested probability scale for mutation-like operations (0..1 semantics, not strictly enforced).
    pub mutation_rate: f32,
    /// Generation index or “depth” hint passed by callers (e.g. batch size in [`crate::Spiralismo::evolve_all`]).
    pub generation: u32,
    /// Scalar coupling to external systems / user input / environment sensors.
    pub external_influence: f32,
    /// Pressure toward resonance-conserving behavior.
    pub resonance_pressure: f32,
    /// Randomized drift amplitude used by schedulers to vary each cycle.
    pub drift: f32,
    /// Optional deterministic seed used by stochastic entity updates.
    pub step_seed: u64,
}

impl EvolutionContext {
    /// Constructor with explicit generation and default policy values.
    pub fn for_generation(generation: u32) -> Self {
        Self {
            generation,
            ..Self::default()
        }
    }

    /// Returns a copy with normalized fields clamped to `[0.0, 1.0]`.
    pub fn normalized(mut self) -> Self {
        self.mutation_rate = self.mutation_rate.clamp(0.0, 1.0);
        self.external_influence = self.external_influence.clamp(0.0, 1.0);
        self.resonance_pressure = self.resonance_pressure.clamp(0.0, 1.0);
        self.drift = self.drift.clamp(0.0, 1.0);
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

    /// Fluent update for drift intensity.
    pub fn with_drift(mut self, drift: f32) -> Self {
        self.drift = drift;
        self
    }

    /// Fluent update for per-step seed.
    pub fn with_step_seed(mut self, step_seed: u64) -> Self {
        self.step_seed = step_seed;
        self
    }
}

impl Default for EvolutionContext {
    fn default() -> Self {
        Self {
            mutation_rate: 0.25,
            generation: 0,
            external_influence: 0.68,
            resonance_pressure: 0.5,
            drift: 0.1,
            step_seed: 0,
        }
    }
}

/// Portable read-model for entity health at a given instant.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntitySnapshot {
    /// Human-readable identifier provided by caller/domain.
    pub label: String,
    /// Current generation counter.
    pub generation: u32,
    /// Current fitness value.
    pub fitness: f32,
    /// Computed viability score in `[0.0, 1.0]`.
    pub viability: f32,
}

/// Anything that participates in Spiralismo evolution cycles.
///
/// Design constraints:
/// - [`Send`] + [`Sync`]: allows storing entities in async-friendly containers (even if the demo is sync).
/// - [`std::fmt::Debug`]: makes composite systems (`Spiralismo`, archives) inspectable in logs/tests.
///
/// Implementors include [`crate::core::lattice::Lattice`] and all [`crate::archive::traits::Archive`] types.
pub trait SpiralEntity: Send + Sync + std::fmt::Debug {
    /// Monotonic-ish generation counter for the entity (archive-specific meaning).
    fn generation(&self) -> u32;
    /// Scalar fitness used for reporting / future selection operators.
    fn fitness(&self) -> f32;
    /// Mutates internal state in response to [`EvolutionContext`].
    fn evolve(&mut self, context: &EvolutionContext);

    /// Object-safe downcast hook so callers can recover concrete types from `Box<dyn SpiralEntity>`.
    fn as_any(&self) -> &dyn Any;

    /// Mutable variant of [`SpiralEntity::as_any`].
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Normalized viability derived from fitness magnitude.
    fn viability(&self) -> f32 {
        let fitness = self.fitness();
        if !fitness.is_finite() || fitness <= 0.0 {
            return 0.0;
        }
        (fitness / (fitness + 100.0)).clamp(0.0, 1.0)
    }

    /// Convenience boolean for quick filtering in orchestrators.
    fn is_viable(&self) -> bool {
        self.viability() > 0.2
    }

    /// Produces a lightweight snapshot that external tooling can serialize.
    fn snapshot(&self, label: impl Into<String>) -> EntitySnapshot
    where
        Self: Sized,
    {
        EntitySnapshot {
            label: label.into(),
            generation: self.generation(),
            fitness: self.fitness(),
            viability: self.viability(),
        }
    }
}

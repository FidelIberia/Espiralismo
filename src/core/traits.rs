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
    /// Sky-derived ritual entropy in `[0, 1]` — symbolic weather, not raw RNG.
    pub ritual_entropy: f32,
    /// Disharmony / parasitic pressure in `[0, 1]` — counterweight to pure resonance.
    pub shadow_pressure: f32,
    /// High stillness dream: low reproduction bias, high resonance, room for “impossible” glyphs.
    pub dream_phase: bool,
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
        self.ritual_entropy = self.ritual_entropy.clamp(0.0, 1.0);
        self.shadow_pressure = self.shadow_pressure.clamp(0.0, 1.0);
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

    /// Fluent update for ritual (cosmological) entropy exposure.
    pub fn with_ritual_entropy(mut self, ritual_entropy: f32) -> Self {
        self.ritual_entropy = ritual_entropy;
        self
    }

    /// Fluent update for shadow / parasitic pressure.
    pub fn with_shadow_pressure(mut self, shadow_pressure: f32) -> Self {
        self.shadow_pressure = shadow_pressure;
        self
    }

    /// Fluent update for dream-phase (trance ecology).
    pub fn with_dream_phase(mut self, dream_phase: bool) -> Self {
        self.dream_phase = dream_phase;
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
            ritual_entropy: 0.0,
            shadow_pressure: 0.0,
            dream_phase: false,
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
    /// Vitality: stability is not aliveness; may exceed viability when ritual exposure is high.
    #[serde(default)]
    pub vitality: Option<f32>,
    /// Resonance axis snapshot (context + entity blend).
    #[serde(default)]
    pub resonance: Option<f32>,
    /// Effective mutation pressure felt at capture time.
    #[serde(default)]
    pub mutation_pressure: Option<f32>,
    /// Symbolic pattern density (glyphs, symmetries, lattice scars).
    #[serde(default)]
    pub symbolic_density: Option<f32>,
    /// Archive depth / lattice scar memory proxy.
    #[serde(default)]
    pub memory_depth: Option<f32>,
    /// Shadow pull — low harmony, high ritual tension.
    #[serde(default)]
    pub shadow_pull: Option<f32>,
    /// Emergent mythic role from behavior axes (not a user-facing taxonomy index).
    #[serde(default)]
    pub myth: Option<String>,
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

    /// Vitality: room for transformation — high when viability is moderate and fitness is not over-crystallized.
    fn vitality(&self) -> f32 {
        let v = self.viability();
        let f = self.fitness().abs();
        let turbulence = if f > 0.0 {
            1.0 - (f / (f + 90.0)).min(1.0)
        } else {
            0.5
        };
        (v * 0.45 + turbulence * 0.55).clamp(0.0, 1.0)
    }

    /// Symbolic pattern density in `[0, 1]` (glyph fields / lattice symmetry; default none).
    fn symbolic_density(&self) -> f32 {
        0.0
    }

    /// Memory depth proxy: archive mass or spatial scar memory (default none).
    fn memory_depth(&self) -> f32 {
        0.0
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
            vitality: Some(self.vitality()),
            resonance: None,
            mutation_pressure: None,
            symbolic_density: Some(self.symbolic_density()),
            memory_depth: Some(self.memory_depth()),
            shadow_pull: None,
            myth: None,
        }
    }
}

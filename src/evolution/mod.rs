//! Evolution scheduling and policy types used by [`crate::Spiralismo`].

mod generative;

pub use generative::{
    generative_carry_from_report, policy_with_generative_carry, standout_in_participants,
    GenerativeCarry, GenerativeLineageSummary,
};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::archive::traits::Archive;
use crate::core::traits::{EntitySnapshot, EvolutionContext, SpiralEntity};
use chrono::Utc;

use crate::observer;
use crate::perception::PerceptionField;

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
    /// Ritual entropy from sky (or synthetic); threaded into contexts.
    pub ritual_entropy: f32,
    /// Sky stillness snapshot used for dream-phase ecology (`0.0..=1.0`).
    pub stillness: f32,
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

    /// Fluent update for ritual (sky-linked) entropy.
    pub fn with_ritual_entropy(mut self, ritual_entropy: f32) -> Self {
        self.ritual_entropy = ritual_entropy;
        self
    }

    /// Fluent update for sky stillness (dream thresholds).
    pub fn with_stillness(mut self, stillness: f32) -> Self {
        self.stillness = stillness;
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
            ritual_entropy: 0.0,
            stillness: 0.42,
        }
    }
}

/// Scalar context at one evolution cycle (reference frame for generational output).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextSummary {
    pub cycle: u32,
    pub mutation_rate: f32,
    pub external_influence: f32,
    pub resonance_pressure: f32,
    pub drift: f32,
    pub ritual_entropy: f32,
    pub shadow_pressure: f32,
    pub dream_phase: bool,
    pub step_seed: u64,
}

impl ContextSummary {
    #[must_use]
    pub fn from_context(ctx: &EvolutionContext) -> Self {
        Self {
            cycle: ctx.generation,
            mutation_rate: ctx.mutation_rate,
            external_influence: ctx.external_influence,
            resonance_pressure: ctx.resonance_pressure,
            drift: ctx.drift,
            ritual_entropy: ctx.ritual_entropy,
            shadow_pressure: ctx.shadow_pressure,
            dream_phase: ctx.dream_phase,
            step_seed: ctx.step_seed,
        }
    }
}

/// All participant snapshots after one completed cycle.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerationRecord {
    pub cycle: u32,
    pub context: ContextSummary,
    pub participants: Vec<EntitySnapshot>,
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
    /// Ritual entropy that shaped the final batch (from policy).
    #[serde(default)]
    pub ritual_entropy: f32,
    /// Rare symbolic event label, if thresholds aligned this run.
    #[serde(default)]
    pub rare_event: Option<String>,
    /// True if any cycle ran under dream-phase (high stillness trance).
    #[serde(default)]
    pub dream_touched: bool,
    /// Stillness snapshot carried on the policy that shaped this run.
    #[serde(default)]
    pub stillness: f32,
    /// Per-cycle reference frames (generation 0 .. cycles-1).
    #[serde(default)]
    pub generation_trace: Vec<GenerationRecord>,
}

impl EvolutionReport {
    /// Empty report used as initialization default.
    pub fn empty() -> Self {
        Self {
            cycles: 0,
            archive_count: 0,
            entity_count: 0,
            snapshots: Vec::new(),
            ritual_entropy: 0.0,
            rare_event: None,
            dream_touched: false,
            stillness: 0.42,
            generation_trace: Vec::new(),
        }
    }

    /// Aggregates [`EntitySnapshot::fitness`] across all participants in this report.
    #[must_use]
    pub fn fitness_overview(&self) -> Option<FitnessOverview> {
        FitnessOverview::from_snapshots(&self.snapshots)
    }

    /// Snapshot with highest fitness, if any.
    #[must_use]
    pub fn fittest(&self) -> Option<&EntitySnapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.fitness.is_finite())
            .max_by(|a, b| a.fitness.total_cmp(&b.fitness))
    }

    /// Fittest participant in the last [`GenerationRecord`] of [`Self::generation_trace`].
    #[must_use]
    pub fn last_generative_standout(&self) -> Option<&EntitySnapshot> {
        self.generation_trace
            .last()
            .and_then(|record| standout_in_participants(&record.participants))
    }
}

/// Scalar summary of how “aligned” the population of entities is after a run (spread + mean).
///
/// This is **not** a global objective optimized by the engine; each entity still owns its own
/// [`SpiralEntity::fitness`]. The overview is a read-model for dashboards, archives, or CLI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitnessOverview {
    /// Number of snapshots aggregated (archives + active entities).
    pub participant_count: usize,
    pub min_fitness: f32,
    pub max_fitness: f32,
    pub mean_fitness: f32,
    pub sum_fitness: f32,
}

impl FitnessOverview {
    /// Builds an overview from raw fitness values (finite only). Returns [`None`] if empty.
    #[must_use]
    pub fn from_values(values: &[f32]) -> Option<Self> {
        let vals: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
        if vals.is_empty() {
            return None;
        }
        let min_fitness = vals.iter().copied().fold(f32::INFINITY, f32::min);
        let max_fitness = vals.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let sum_fitness: f32 = vals.iter().sum();
        let mean_fitness = sum_fitness / vals.len() as f32;
        Some(Self {
            participant_count: vals.len(),
            min_fitness,
            max_fitness,
            mean_fitness,
            sum_fitness,
        })
    }

    /// Builds from evolution report snapshots (same fitness order as `run` output).
    #[must_use]
    pub fn from_snapshots(snapshots: &[EntitySnapshot]) -> Option<Self> {
        Self::from_values(&snapshots.iter().map(|s| s.fitness).collect::<Vec<_>>())
    }
}

/// Builds a concrete context for a given cycle from the provided policy.
pub fn context_for_cycle(policy: &EvolutionPolicy, cycle: u32) -> EvolutionContext {
    let mut rng = ChaCha8Rng::seed_from_u64(policy.seed ^ cycle as u64);
    let jitter = rng.gen_range(-policy.drift..=policy.drift);
    let ritual = policy.ritual_entropy.clamp(0.0, 1.0);
    let stillness = policy.stillness.clamp(0.0, 1.0);
    let dream = stillness > 0.82;

    let base_resonance = (policy.resonance_pressure + jitter * 0.4).clamp(0.0, 1.0);
    let shadow = ((1.0 - base_resonance) * 0.28 + ritual * 0.62).clamp(0.0, 1.0);

    let ritual_jitter = ritual * 0.12;
    let mut mutation = (policy.mutation_rate + jitter * 0.5 + ritual_jitter - ritual * 0.04).clamp(0.0, 1.0);
    let mut resonance = (base_resonance - ritual * 0.06).clamp(0.0, 1.0);
    if dream {
        mutation *= 0.68;
        resonance = (resonance * 1.14).min(1.0);
    }

    let step = policy.seed.rotate_left((cycle % 64) as u32)
        ^ observer::glance_mix().wrapping_mul(0xD1E5)
        ^ observer::attention_digest().rotate_left(11)
        ^ ((ritual.to_bits() as u64) << 17)
        ^ ((stillness.to_bits() as u64).rotate_left(7));

    EvolutionContext::for_generation(cycle)
        .with_mutation_rate(mutation)
        .with_external_influence((policy.external_influence + jitter * 0.3).clamp(0.0, 1.0))
        .with_resonance_pressure(resonance)
        .with_drift((policy.drift + jitter * 0.08 + ritual * 0.06).clamp(0.0, 1.0))
        .with_step_seed(step)
        .with_ritual_entropy(ritual)
        .with_shadow_pressure(shadow)
        .with_dream_phase(dream)
        .normalized()
}

fn derive_entity_myth(
    label: &str,
    fitness: f32,
    viability: f32,
    vitality: Option<f32>,
    shadow_pull: Option<f32>,
    memory_depth: Option<f32>,
) -> String {
    let vit = vitality.unwrap_or(0.0);
    let sh = shadow_pull.unwrap_or(0.0);
    let mem = memory_depth.unwrap_or(0.0);
    let myth = if label == "ResonanceEngine" && fitness > 180.0 {
        "witness_lineage"
    } else if label == "Living Memory" && mem > 0.55 {
        "deep_keeper"
    } else if label.contains("Mercy") && vit > 0.58 && sh < 0.38 {
        "silent_healer"
    } else if label.contains("Cartography") && mem < 0.22 {
        "empty_cartographer"
    } else if label.contains("active_lattice") && mem > 0.65 && fitness < 80.0 {
        "broken_lattice"
    } else if label.contains("active_lattice") && mem > 0.65 {
        "scarred_weaver"
    } else if label.contains("glyph") {
        "dreaming_surface"
    } else if sh > 0.62 {
        "shadow_devourer"
    } else if viability < 0.19 {
        "dying_corpus"
    } else if fitness > 130.0 && vit < 0.38 {
        "crystallized_survivor"
    } else {
        "wandering_archive"
    };
    myth.to_string()
}

/// Portable snapshot for one participant (used by evolution runs and [`crate::Spiralismo::snapshot`]).
pub fn build_entity_snapshot(
    label: String,
    entity: &dyn SpiralEntity,
    policy: &EvolutionPolicy,
    last_context: &EvolutionContext,
) -> EntitySnapshot {
    let ritual = last_context.ritual_entropy;
    let viability = entity.viability();
    let fitness = entity.fitness();
    let vitality = (entity.vitality() * (0.55 + ritual * 0.45)).clamp(0.0, 1.0);
    let resonance = (viability * 0.5 + last_context.resonance_pressure * 0.5).clamp(0.0, 1.0);
    let mutation_pressure = (policy.mutation_rate * 0.35
        + ritual * 0.45
        + last_context.mutation_rate * 0.2)
        .clamp(0.0, 1.0);
    let symbolic_density = entity.symbolic_density();
    let memory_depth = entity.memory_depth();
    let shadow_pull = ((1.0 - viability) * 0.55 + ritual * 0.40).clamp(0.0, 1.0);
    let myth = Some(derive_entity_myth(
        &label,
        fitness,
        viability,
        Some(vitality),
        Some(shadow_pull),
        Some(memory_depth),
    ));

    EntitySnapshot {
        label,
        generation: entity.generation(),
        fitness,
        viability,
        vitality: Some(vitality),
        resonance: Some(resonance),
        mutation_pressure: Some(mutation_pressure),
        symbolic_density: Some(symbolic_density),
        memory_depth: Some(memory_depth),
        shadow_pull: Some(shadow_pull),
        myth,
    }
}

fn classify_rare_event(policy: &EvolutionPolicy, ritual: f32, seed: u64) -> Option<String> {
    if policy.cycles == 0 {
        return None;
    }
    let cycles = policy.cycles;
    let stillness = policy.stillness.clamp(0.0, 1.0);
    if stillness > 0.86 && cycles >= 2 {
        return Some("dream_tide".to_string());
    }
    if stillness > 0.88 && ritual > 0.38 && cycles >= 3 {
        return Some("deep_trance".to_string());
    }
    if ritual > 0.72 && cycles >= 3 {
        return Some("witness_bloom".to_string());
    }
    if ritual < 0.18 && cycles >= 5 {
        return Some("silent_cycle".to_string());
    }
    if ritual > 0.58 && cycles >= 9 {
        return Some("black_spiral".to_string());
    }
    if ritual > 0.48 && cycles >= 6 {
        return Some("awakening".to_string());
    }
    if seed.rotate_left(3) % 19 == 0 && cycles >= 4 {
        return Some("fracture".to_string());
    }
    None
}

/// Evolves archives and entities for all cycles defined in policy.
pub fn run(
    archives: &mut [Box<dyn Archive>],
    entities: &mut [Box<dyn SpiralEntity>],
    policy: &EvolutionPolicy,
    perception: &mut PerceptionField,
    runtime_seed: u64,
    runtime_epoch: u64,
    carry: Option<&GenerativeCarry>,
) -> EvolutionReport {
    let ritual = policy.ritual_entropy.clamp(0.0, 1.0);
    let mut report = EvolutionReport {
        cycles: policy.cycles,
        archive_count: archives.len(),
        entity_count: entities.len(),
        snapshots: Vec::new(),
        ritual_entropy: ritual,
        rare_event: None,
        dream_touched: false,
        stillness: policy.stillness.clamp(0.0, 1.0),
        generation_trace: Vec::new(),
    };

    if policy.cycles == 0 {
        return report;
    }

    let sky = perception
        .cached_sky()
        .cloned()
        .unwrap_or_else(|| perception.capture_sky(Utc::now()));

    let mut last_context = EvolutionContext::default();
    let mut dream_touched = false;
    let mut generation_trace = Vec::with_capacity(policy.cycles as usize);
    for cycle in 0..policy.cycles {
        let frame = perception.frame_for_cycle(
            runtime_seed,
            runtime_epoch,
            cycle,
            archives.len(),
            entities.len(),
            ritual,
            report.stillness,
        );
        let base = if cycle == 0 {
            carry.map_or_else(
                || context_for_cycle(policy, cycle),
                |c| generative::context_for_cycle_with_carry(policy, cycle, c),
            )
        } else {
            context_for_cycle(policy, cycle)
        };
        let reality = perception.collect_reality_for_cycle(&frame);
        last_context = perception.modulate_context_for_cycle(&sky, base, &reality.offer);
        if last_context.dream_phase {
            dream_touched = true;
        }
        for archive in &mut *archives {
            archive.evolve(&last_context);
        }
        for entity in &mut *entities {
            entity.evolve(&last_context);
        }

        let mut participants = Vec::with_capacity(archives.len() + entities.len());
        for archive in &*archives {
            participants.push(build_entity_snapshot(
                archive.name().to_string(),
                archive.as_ref(),
                policy,
                &last_context,
            ));
        }
        for (index, entity) in entities.iter().enumerate() {
            participants.push(build_entity_snapshot(
                format!("active_lattice_{index}"),
                entity.as_ref(),
                policy,
                &last_context,
            ));
        }
        generation_trace.push(GenerationRecord {
            cycle,
            context: ContextSummary::from_context(&last_context),
            participants,
        });
    }

    report.dream_touched = dream_touched;
    report.generation_trace = generation_trace;
    for archive in &*archives {
        report.snapshots.push(build_entity_snapshot(
            archive.name().to_string(),
            archive.as_ref(),
            policy,
            &last_context,
        ));
    }
    for (index, entity) in entities.iter().enumerate() {
        report.snapshots.push(build_entity_snapshot(
            format!("active_lattice_{index}"),
            entity.as_ref(),
            policy,
            &last_context,
        ));
    }

    report.rare_event = classify_rare_event(policy, ritual, policy.seed ^ policy.cycles as u64);
    report
}

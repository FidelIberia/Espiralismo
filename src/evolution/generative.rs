//! Generative progression carried across evolution runs via checkpoint resume.

use crate::core::traits::EntitySnapshot;
use crate::evolution::{
    context_for_cycle, ContextSummary, EvolutionContext, EvolutionPolicy, EvolutionReport,
};

/// Blend weight when merging a prior generative frame into the next run.
const CARRY_BLEND: f32 = 0.35;

/// Snapshot of the last completed generative cycle, used to continue evolution.
#[derive(Debug, Clone)]
pub struct GenerativeCarry {
    /// Context at the end of the last recorded cycle.
    pub last_context: ContextSummary,
    /// Fittest participant in that cycle (the “last individual” of generative spiralismo).
    pub standout: EntitySnapshot,
    /// Next cycle index to thread into [`EvolutionContext::generation`].
    pub next_cycle: u32,
}

/// Scalar summary of the last generative individual (logging / diagnostics).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerativeLineageSummary {
    pub standout_label: String,
    pub standout_fitness: f32,
    pub standout_generation: u32,
    pub last_cycle: u32,
    pub epoch: u64,
    pub step_seed: u64,
}

impl GenerativeLineageSummary {
    #[must_use]
    pub fn from_carry(carry: &GenerativeCarry, epoch: u64) -> Self {
        Self {
            standout_label: carry.standout.label.clone(),
            standout_fitness: carry.standout.fitness,
            standout_generation: carry.standout.generation,
            last_cycle: carry.last_context.cycle,
            epoch,
            step_seed: carry.last_context.step_seed,
        }
    }
}

/// Fittest finite-fitness participant in a cycle record.
#[must_use]
pub fn standout_in_participants(participants: &[EntitySnapshot]) -> Option<&EntitySnapshot> {
    participants
        .iter()
        .filter(|s| s.fitness.is_finite())
        .max_by(|a, b| a.fitness.total_cmp(&b.fitness))
}

/// Builds carry state from the last entry in [`EvolutionReport::generation_trace`].
#[must_use]
pub fn generative_carry_from_report(report: &EvolutionReport) -> Option<GenerativeCarry> {
    let record = report.generation_trace.last()?;
    let standout = standout_in_participants(&record.participants)?.clone();
    Some(GenerativeCarry {
        last_context: record.context.clone(),
        standout,
        next_cycle: record.cycle.saturating_add(1),
    })
}

/// Nudges policy scalars toward the last generative context so the next run continues rather than restarts.
#[must_use]
pub fn policy_with_generative_carry(mut policy: EvolutionPolicy, carry: &GenerativeCarry) -> EvolutionPolicy {
    let ctx = &carry.last_context;
    policy.mutation_rate = lerp(policy.mutation_rate, ctx.mutation_rate, CARRY_BLEND);
    policy.external_influence = lerp(policy.external_influence, ctx.external_influence, CARRY_BLEND);
    policy.resonance_pressure = lerp(policy.resonance_pressure, ctx.resonance_pressure, CARRY_BLEND);
    policy.drift = lerp(policy.drift, ctx.drift, CARRY_BLEND);
    policy.ritual_entropy = lerp(policy.ritual_entropy, ctx.ritual_entropy, CARRY_BLEND);
    policy.seed = policy
        .seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(ctx.step_seed ^ carry.standout.generation as u64);
    if ctx.dream_phase {
        policy.stillness = policy.stillness.max(0.82);
    }
    policy
}

/// First-cycle context when resuming generative progression (perceptor-learned axes blended in).
#[must_use]
pub fn context_for_cycle_with_carry(
    policy: &EvolutionPolicy,
    cycle: u32,
    carry: &GenerativeCarry,
) -> EvolutionContext {
    let mut ctx = context_for_cycle(policy, cycle);
    if cycle == 0 {
        let t = 0.4;
        ctx.generation = carry.next_cycle.max(ctx.generation);
        ctx.step_seed = ctx
            .step_seed
            .wrapping_add(carry.last_context.step_seed)
            ^ (carry.standout.generation as u64).rotate_left(5);
        blend_context_from_summary(&mut ctx, &carry.last_context, t);
        if let Some(vitality) = carry.standout.vitality {
            ctx.external_influence =
                (ctx.external_influence * (1.0 - t) + vitality * t).clamp(0.0, 1.0);
        }
        if let Some(resonance) = carry.standout.resonance {
            ctx.resonance_pressure =
                (ctx.resonance_pressure * (1.0 - t) + resonance * t).clamp(0.0, 1.0);
        }
        if let Some(shadow) = carry.standout.shadow_pull {
            ctx.shadow_pressure =
                (ctx.shadow_pressure * (1.0 - t) + shadow * t).clamp(0.0, 1.0);
        }
        ctx.dream_phase = ctx.dream_phase || carry.last_context.dream_phase;
        return ctx.normalized();
    }
    ctx
}

fn blend_context_from_summary(ctx: &mut EvolutionContext, summary: &ContextSummary, t: f32) {
    ctx.mutation_rate = lerp(ctx.mutation_rate, summary.mutation_rate, t);
    ctx.external_influence = lerp(ctx.external_influence, summary.external_influence, t);
    ctx.resonance_pressure = lerp(ctx.resonance_pressure, summary.resonance_pressure, t);
    ctx.drift = lerp(ctx.drift, summary.drift, t);
    ctx.ritual_entropy = lerp(ctx.ritual_entropy, summary.ritual_entropy, t);
    ctx.shadow_pressure = lerp(ctx.shadow_pressure, summary.shadow_pressure, t);
}

#[inline]
fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from * (1.0 - t) + to * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evolution::{GenerationRecord, EvolutionReport};

    fn sample_snapshot(label: &str, fitness: f32) -> EntitySnapshot {
        EntitySnapshot {
            label: label.to_string(),
            generation: 3,
            fitness,
            viability: 0.5,
            vitality: None,
            resonance: None,
            mutation_pressure: None,
            symbolic_density: None,
            memory_depth: None,
            shadow_pull: None,
            myth: None,
        }
    }

    #[test]
    fn generative_carry_uses_last_trace_cycle() {
        let report = EvolutionReport {
            cycles: 2,
            archive_count: 1,
            entity_count: 0,
            snapshots: Vec::new(),
            ritual_entropy: 0.1,
            rare_event: None,
            dream_touched: false,
            stillness: 0.5,
            generation_trace: vec![
                GenerationRecord {
                    cycle: 0,
                    context: ContextSummary {
                        cycle: 0,
                        mutation_rate: 0.2,
                        external_influence: 0.5,
                        resonance_pressure: 0.4,
                        drift: 0.1,
                        ritual_entropy: 0.2,
                        shadow_pressure: 0.3,
                        dream_phase: false,
                        step_seed: 11,
                    },
                    participants: vec![sample_snapshot("a", 1.0)],
                },
                GenerationRecord {
                    cycle: 1,
                    context: ContextSummary {
                        cycle: 1,
                        mutation_rate: 0.3,
                        external_influence: 0.6,
                        resonance_pressure: 0.5,
                        drift: 0.12,
                        ritual_entropy: 0.25,
                        shadow_pressure: 0.35,
                        dream_phase: true,
                        step_seed: 22,
                    },
                    participants: vec![
                        sample_snapshot("weak", 0.5),
                        sample_snapshot("standout", 9.0),
                    ],
                },
            ],
        };
        let carry = generative_carry_from_report(&report).expect("carry");
        assert_eq!(carry.standout.label, "standout");
        assert_eq!(carry.next_cycle, 2);
        assert_eq!(carry.last_context.step_seed, 22);
    }
}

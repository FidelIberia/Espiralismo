//! Composes participles with optional agents (`sellado` + `por` + `la sombra`).

use super::common::Language;
use super::grammar::{AgentEntry, AgreementKey, VerbalState};

/// Permille thresholds for agent attachment (tuned per call site).
pub struct VerbalAttachPolicy {
    /// Skip agent entirely (participle only).
    pub solo_permille: u32,
    /// Pick from [`AgentEntry::indefinite`] pool.
    pub indefinite_permille: u32,
}

impl Default for VerbalAttachPolicy {
    fn default() -> Self {
        Self {
            solo_permille: 320,
            indefinite_permille: 140,
        }
    }
}

/// Builds `participle` or `participle linker agent` for the active locale.
#[must_use]
pub fn compose_verbal(
    language: Language,
    participle: &str,
    agent: Option<&AgentEntry>,
) -> String {
    let Some(agent) = agent else {
        return participle.to_string();
    };
    let linker = agent.linker.trim();
    let _ = language;
    if linker.is_empty() {
        format!("{participle} {}", agent.text)
    } else {
        format!("{participle} {linker} {}", agent.text)
    }
}

/// Resolves a participle and optionally attaches an agent.
#[must_use]
pub fn forge_verbal_phrase(
    language: Language,
    key: AgreementKey,
    seed: u64,
    slot: u32,
    states: &[VerbalState],
    agents: &[AgentEntry],
    policy: &VerbalAttachPolicy,
    roll: fn(u64, u32) -> u32,
) -> Option<String> {
    if states.is_empty() {
        return None;
    }
    let matching: Vec<&str> = states
        .iter()
        .filter_map(|s| s.participle(key))
        .collect();
    if matching.is_empty() {
        return None;
    }
    let idx = weighted_verbal_index(seed, slot, matching.len());
    let participle = matching[idx];

    let r = roll(seed, slot.wrapping_add(20));
    if r < policy.solo_permille {
        return Some(participle.to_string());
    }

    let agent = if r < policy.solo_permille + policy.indefinite_permille {
        pick_agent(seed, slot.wrapping_add(21), agents, true)
    } else {
        pick_agent(seed, slot.wrapping_add(22), agents, false)
    };

    Some(compose_verbal(language, participle, agent))
}

fn pick_agent<'a>(
    seed: u64,
    slot: u32,
    agents: &'a [AgentEntry],
    indefinite: bool,
) -> Option<&'a AgentEntry> {
    let pool: Vec<&AgentEntry> = agents
        .iter()
        .filter(|a| a.indefinite == indefinite)
        .collect();
    if pool.is_empty() {
        return None;
    }
    let idx = weighted_verbal_index(seed, slot, pool.len());
    Some(pool[idx])
}

fn weighted_verbal_index(seed: u64, slot: u32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let mixed = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((slot as u64).wrapping_mul(0x517c_c1b7_2722_0a95));
    (mixed as usize) % len
}

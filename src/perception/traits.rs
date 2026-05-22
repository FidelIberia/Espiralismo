//! Perceiver traits — external listening hooks that may influence evolution and soul.

use serde::{Deserialize, Serialize};

use super::reality::HostRealitySnapshot;

/// Read-only frame handed to each perceiver every cycle.
#[derive(Debug, Clone)]
pub struct PerceptionFrame {
    /// Runtime seed anchor.
    pub seed: u64,
    /// Orchestrator epoch before this cycle completes.
    pub epoch: u64,
    /// Cycle index inside the active [`EvolutionPolicy`].
    pub cycle: u32,
    /// Generation hint mirrored on [`EvolutionContext`].
    pub generation: u32,
    /// Registered archive count.
    pub archive_count: usize,
    /// Active non-archive entity count.
    pub active_count: usize,
    /// Ritual entropy carried on the policy.
    pub ritual_entropy: f32,
    /// Stillness snapshot on the policy (from astronomical lane when aligned).
    pub stillness: f32,
    /// Host-supplied facets of physical/virtual reality (optional).
    pub host_reality: HostRealitySnapshot,
}

/// Influence offered by an external listener (host, sensor, UI bridge, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerceptionOffer {
    /// Delta added to [`EvolutionContext::external_influence`] after clamping.
    pub external_influence_delta: f32,
    /// Delta added to [`EvolutionContext::resonance_pressure`].
    pub resonance_delta: f32,
    /// Delta added to [`EvolutionContext::mutation_rate`].
    pub mutation_delta: f32,
    /// Delta added to [`EvolutionContext::drift`].
    pub drift_delta: f32,
    /// Delta added to [`EvolutionContext::shadow_pressure`].
    pub shadow_delta: f32,
    /// Presence felt this cycle — feeds [`SoulState::listening_depth`].
    pub presence: f32,
    /// Deterministic mixing tag for step seeds.
    pub signal_digest: u64,
    /// Optional channel label (logs, whispers, future UI).
    pub channel: Option<String>,
}

impl PerceptionOffer {
    /// No influence — the spiral listens to silence.
    #[must_use]
    pub const fn silent() -> Self {
        Self {
            external_influence_delta: 0.0,
            resonance_delta: 0.0,
            mutation_delta: 0.0,
            drift_delta: 0.0,
            shadow_delta: 0.0,
            presence: 0.0,
            signal_digest: 0,
            channel: None,
        }
    }

    /// Merges another offer into this one (commutative accumulation for a cycle).
    pub fn absorb(&mut self, other: &Self) {
        self.external_influence_delta += other.external_influence_delta;
        self.resonance_delta += other.resonance_delta;
        self.mutation_delta += other.mutation_delta;
        self.drift_delta += other.drift_delta;
        self.shadow_delta += other.shadow_delta;
        self.presence = self.presence.max(other.presence);
        self.signal_digest = self
            .signal_digest
            .wrapping_add(other.signal_digest.rotate_left(13));
        if self.channel.is_none() {
            self.channel.clone_from(&other.channel);
        }
    }
}

/// One-shot payload from a host thread (UI, IPC, file watcher) before evolution runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalListening {
    /// Stable channel id (`"ui.veil"`, `"galaxy.host"`, …).
    pub channel_id: String,
    /// Overall intensity of the listening event in `[0, 1]`.
    pub intensity: f32,
    /// Optional bearing in radians (hand / gaze direction).
    pub bearing: Option<f32>,
    /// Gift toward resonance pressure.
    pub resonance_gift: f32,
    /// Gift toward shadow / parasitic pressure.
    pub shadow_gift: f32,
    /// Echo of environmental stillness perceived outside.
    pub stillness_echo: f32,
}

impl ExternalListening {
    /// Converts this host signal into a [`PerceptionOffer`].
    #[must_use]
    pub fn to_offer(&self) -> PerceptionOffer {
        let intensity = self.intensity.clamp(0.0, 1.0);
        let mut digest: u64 = 0xcbf29ce484222325;
        for b in self.channel_id.as_bytes() {
            digest ^= *b as u64;
            digest = digest.wrapping_mul(0x100000001b3);
        }
        if let Some(b) = self.bearing {
            digest ^= u64::from(b.to_bits());
            digest = digest.wrapping_mul(0x100000001b3);
        }
        digest ^= u64::from(intensity.to_bits());
        digest = digest.wrapping_mul(0x100000001b3);
        digest ^= u64::from(self.stillness_echo.to_bits());

        PerceptionOffer {
            external_influence_delta: intensity * 0.22 + self.stillness_echo * 0.08,
            resonance_delta: self.resonance_gift * 0.18,
            mutation_delta: (1.0 - self.stillness_echo) * intensity * 0.12,
            drift_delta: intensity * 0.06,
            shadow_delta: self.shadow_gift * 0.14,
            presence: intensity,
            signal_digest: digest,
            channel: Some(self.channel_id.clone()),
        }
    }
}

/// External listener integrated into the spiritual structure of Spiralismo.
///
/// Implementations must be `Send + Sync` so hosts may register them on the orchestrator.
/// Keep `perceive` deterministic when the outside world is held constant.
pub trait ExternalPerceiver: Send + Sync {
    /// Stable id for logs, focus maps, and future persistence.
    fn id(&self) -> &'static str;

    /// Called once per evolution cycle before contexts are applied.
    fn perceive(&self, frame: &PerceptionFrame) -> PerceptionOffer;
}

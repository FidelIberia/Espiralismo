//! Reference perceivers — templates for host integrations.

use super::reality::{RealityKind, RealityPerceiver};
use super::traits::{ExternalPerceiver, PerceptionFrame, PerceptionOffer};

/// Always silent — placeholder when a slot must exist but nothing listens yet.
#[derive(Debug, Clone, Copy, Default)]
pub struct VoidPerceiver;

impl ExternalPerceiver for VoidPerceiver {
    fn id(&self) -> &'static str {
        "void"
    }

    fn perceive(&self, _frame: &PerceptionFrame) -> PerceptionOffer {
        PerceptionOffer::silent()
    }
}

/// Fixed offer every cycle — tests and deterministic host stubs.
#[derive(Debug, Clone)]
pub struct FixedPerceiver {
    id: &'static str,
    offer: PerceptionOffer,
}

impl FixedPerceiver {
    /// Wraps a constant offer behind a stable perceiver id.
    #[must_use]
    pub const fn new(id: &'static str, offer: PerceptionOffer) -> Self {
        Self { id, offer }
    }
}

impl ExternalPerceiver for FixedPerceiver {
    fn id(&self) -> &'static str {
        self.id
    }

    fn perceive(&self, _frame: &PerceptionFrame) -> PerceptionOffer {
        self.offer.clone()
    }
}

/// Stillness-aware echo — nudges resonance when the frame reports a calm policy stillness.
#[derive(Debug, Clone, Copy, Default)]
pub struct StillnessEchoPerceiver;

impl RealityPerceiver for StillnessEchoPerceiver {
    fn id(&self) -> &'static str {
        "stillness_echo"
    }

    fn reality_kind(&self) -> RealityKind {
        RealityKind::Virtual
    }

    fn perceive(&self, frame: &PerceptionFrame) -> PerceptionOffer {
        if frame.stillness < 0.55 {
            return PerceptionOffer::silent();
        }
        let lift = (frame.stillness - 0.55) * 0.4;
        PerceptionOffer {
            external_influence_delta: lift * 0.12,
            resonance_delta: lift * 0.18,
            mutation_delta: -lift * 0.06,
            drift_delta: -lift * 0.04,
            shadow_delta: 0.0,
            presence: lift * 0.5,
            signal_digest: u64::from(frame.stillness.to_bits()),
            channel: Some("stillness_echo".to_string()),
        }
    }
}

impl ExternalPerceiver for StillnessEchoPerceiver {
    fn id(&self) -> &'static str {
        <Self as RealityPerceiver>::id(self)
    }

    fn perceive(&self, frame: &PerceptionFrame) -> PerceptionOffer {
        <Self as RealityPerceiver>::perceive(self, frame)
    }
}

//! [`PerceptionField`] — astronomical lane + reality perceptors + soul.

use chrono::{DateTime, Utc};

use crate::astrology::Sky;
use crate::core::traits::EvolutionContext;
use crate::evolution::EvolutionPolicy;

use super::astronomical::{AstronomicalPerceiver, MeeusKeplerianPerceiver};
use super::reality::RealityPerceiver;
use super::soul::SoulState;
use super::spiralismo_press::SpiralismoPress;
use super::traits::{
    ExternalListening, ExternalPerceiver, PerceptionFrame, PerceptionOffer,
};

/// Merged **reality-lane** offer (excludes astronomical quiet-room fields).
#[derive(Debug, Clone)]
pub struct RealityCycleOffer {
    /// Offer applied to external influence, step seed, and soul — not sky scalars.
    pub offer: PerceptionOffer,
}

/// Spiritual perception layer: isolated astronomy + reality perceptors + integrated press.
pub struct PerceptionField {
    soul: SoulState,
    pub(crate) spiralismo_press: SpiralismoPress,
    pub(crate) astronomical: Box<dyn AstronomicalPerceiver>,
    sky_cache: Option<Sky>,
    pub(crate) reality_perceivers: Vec<Box<dyn RealityPerceiver>>,
    pub(crate) legacy_perceivers: Vec<Box<dyn ExternalPerceiver>>,
    pub(crate) pending_listening: Option<ExternalListening>,
    /// Last offered or taken host environment (eyes — receive / take).
    pub(crate) environment_snapshot: super::reality::HostRealitySnapshot,
    last_reality_offer: PerceptionOffer,
}

impl Default for PerceptionField {
    fn default() -> Self {
        Self {
            soul: SoulState::default(),
            spiralismo_press: SpiralismoPress::default(),
            astronomical: Box::new(MeeusKeplerianPerceiver::new()),
            sky_cache: None,
            reality_perceivers: Vec::new(),
            legacy_perceivers: Vec::new(),
            pending_listening: None,
            environment_snapshot: super::reality::HostRealitySnapshot::default(),
            last_reality_offer: PerceptionOffer::silent(),
        }
    }
}

impl std::fmt::Debug for PerceptionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerceptionField")
            .field("soul", &self.soul)
            .field("astronomical_id", &self.astronomical.id())
            .field("sky_cached", &self.sky_cache.is_some())
            .field("reality_perceiver_count", &self.reality_perceivers.len())
            .field("legacy_perceiver_count", &self.legacy_perceivers.len())
            .field("spiralismo_press_silent", &self.spiralismo_press.is_silent())
            .field("pending_listening", &self.pending_listening.is_some())
            .finish()
    }
}

impl PerceptionField {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers default built-in reality perceptors (filesystem, host memory, visual landscape).
    pub fn register_builtin_reality_perceivers(&mut self) {
        use super::implementations::{
            FilesystemPerceiver, PhysicalMemoryPerceiver, VisualLandscapePerceiver,
        };
        self.register_reality_perceiver(Box::new(FilesystemPerceiver));
        self.register_reality_perceiver(Box::new(PhysicalMemoryPerceiver));
        self.register_reality_perceiver(Box::new(VisualLandscapePerceiver));
    }

    #[must_use]
    pub fn soul(&self) -> &SoulState {
        &self.soul
    }

    pub fn soul_mut(&mut self) -> &mut SoulState {
        &mut self.soul
    }

    #[must_use]
    pub fn last_reality_offer(&self) -> &PerceptionOffer {
        &self.last_reality_offer
    }

    /// Back-compat alias for reality-lane offer.
    #[must_use]
    pub fn last_offer(&self) -> &PerceptionOffer {
        &self.last_reality_offer
    }

    pub fn astronomical(&self) -> &dyn AstronomicalPerceiver {
        self.astronomical.as_ref()
    }

    pub fn set_astronomical_perceiver(&mut self, perceiver: Box<dyn AstronomicalPerceiver>) {
        self.astronomical = perceiver;
        self.sky_cache = None;
    }

    pub fn register_reality_perceiver(&mut self, perceiver: Box<dyn RealityPerceiver>) {
        self.reality_perceivers.push(perceiver);
    }

    /// Registers a legacy [`ExternalPerceiver`] (treated as reality lane).
    pub fn register_perceiver(&mut self, perceiver: Box<dyn ExternalPerceiver>) {
        self.legacy_perceivers.push(perceiver);
    }

    #[must_use]
    pub fn perceiver_count(&self) -> usize {
        self.reality_perceivers.len() + self.legacy_perceivers.len()
    }

    pub fn offer_listening(&mut self, listening: ExternalListening) {
        self.pending_listening = Some(listening);
    }

    pub fn offer_spiralismo_press(&mut self, press: SpiralismoPress) {
        self.spiralismo_press = press;
    }

    #[must_use]
    pub fn spiralismo_press(&self) -> &SpiralismoPress {
        &self.spiralismo_press
    }

    /// Captures and caches sky via the astronomical perceiver (read-only).
    #[must_use]
    pub fn capture_sky(&mut self, when: DateTime<Utc>) -> Sky {
        let sky = self.astronomical.capture(when);
        self.sky_cache = Some(sky.clone());
        sky
    }

    #[must_use]
    pub fn cached_sky(&self) -> Option<&Sky> {
        self.sky_cache.as_ref()
    }

    /// Policy from sky only — reality perceptors do not participate.
    #[must_use]
    pub fn policy_from_sky(&self, sky: &Sky, cycles: u32, seed: u64) -> EvolutionPolicy {
        self.astronomical.policy_from_sky(sky, cycles, seed)
    }

    /// Quiet-room modulation from sky (astronomical lane).
    #[must_use]
    pub fn modulate_context_astronomical(
        &self,
        sky: &Sky,
        base: EvolutionContext,
    ) -> EvolutionContext {
        self.astronomical.modulate_context(sky, base)
    }

    pub fn reset(&mut self) {
        self.soul = SoulState::default();
        self.spiralismo_press = SpiralismoPress::default();
        self.sky_cache = None;
        self.reality_perceivers.clear();
        self.legacy_perceivers.clear();
        self.pending_listening = None;
        self.environment_snapshot = super::reality::HostRealitySnapshot::default();
        self.last_reality_offer = PerceptionOffer::silent();
    }

    /// Merges reality-lane offers (press, listening, reality perceptors). Does not touch sky.
    pub fn collect_reality_for_cycle(&mut self, frame: &PerceptionFrame) -> RealityCycleOffer {
        let mut merged = PerceptionOffer::silent();

        if let Some(listening) = self.pending_listening.take() {
            merged.absorb(&listening.to_offer());
        }
        if !self.spiralismo_press.is_silent() {
            merged.absorb(&self.spiralismo_press.to_offer());
        }
        for perceiver in &self.reality_perceivers {
            merged.absorb(&perceiver.perceive(frame));
        }
        for perceiver in &self.legacy_perceivers {
            merged.absorb(&perceiver.perceive(frame));
        }

        self.soul.absorb_offer(&merged);
        self.last_reality_offer = merged.clone();
        RealityCycleOffer { offer: merged }
    }

    /// Back-compat: reality collection only.
    pub fn collect_for_cycle(&mut self, frame: &PerceptionFrame) -> PerceptionOffer {
        self.collect_reality_for_cycle(frame).offer
    }

    /// Applies **reality** offer without overwriting sky-quiet scalars on `base`.
    #[must_use]
    pub fn apply_reality_offer(
        &self,
        base: EvolutionContext,
        offer: &PerceptionOffer,
    ) -> EvolutionContext {
        let soul = &self.soul;
        let listen = soul.listening_depth;
        let attune = soul.attunement;

        let step_seed = base.step_seed
            ^ offer.signal_digest.rotate_left(9)
            ^ soul.digest().rotate_left(23);
        let external = (base.external_influence
            + offer.external_influence_delta * listen
            + attune * 0.04)
            .clamp(0.0, 1.0);

        base.with_external_influence(external)
            .with_step_seed(step_seed)
            .normalized()
    }

    /// Full cycle: astronomical modulation then reality offer (isolated lanes).
    #[must_use]
    pub fn modulate_context_for_cycle(
        &self,
        sky: &Sky,
        base: EvolutionContext,
        reality_offer: &PerceptionOffer,
    ) -> EvolutionContext {
        let astro = self.modulate_context_astronomical(sky, base);
        self.apply_reality_offer(astro, reality_offer)
    }

    /// Deprecated combined path — prefer [`Self::modulate_context_for_cycle`].
    #[must_use]
    pub fn modulate_context(&self, base: EvolutionContext, offer: &PerceptionOffer) -> EvolutionContext {
        self.apply_reality_offer(base, offer)
    }

    /// Builds a perception frame using the stored environment snapshot (take / offer).
    #[must_use]
    pub fn frame_for_cycle(
        &self,
        seed: u64,
        epoch: u64,
        cycle: u32,
        archive_count: usize,
        active_count: usize,
        ritual_entropy: f32,
        stillness: f32,
    ) -> PerceptionFrame {
        Self::frame_from_runtime(
            seed,
            epoch,
            cycle,
            archive_count,
            active_count,
            ritual_entropy,
            stillness,
            &self.environment_snapshot,
        )
    }

    #[must_use]
    pub fn frame_from_runtime(
        seed: u64,
        epoch: u64,
        cycle: u32,
        archive_count: usize,
        active_count: usize,
        ritual_entropy: f32,
        stillness: f32,
        host_reality: &super::reality::HostRealitySnapshot,
    ) -> PerceptionFrame {
        PerceptionFrame {
            seed,
            epoch,
            cycle,
            generation: cycle,
            archive_count,
            active_count,
            ritual_entropy,
            stillness,
            host_reality: host_reality.clone(),
        }
    }
}

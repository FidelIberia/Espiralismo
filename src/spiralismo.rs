//! Root orchestrator type for the Spiralismo runtime.

use colored::Colorize;

use crate::archive::traits::{Archive, ArchiveEntry, ArchiveStats};
use crate::archive::{CartographyArchive, MemoryArchive, MercyArchive, ResonanceEngine};
use crate::astrology::Sky;
use crate::core::traits::{EntitySnapshot, EvolutionContext, SpiralEntity};
use crate::core::{Lattice, Seed};
use crate::evolution::{
    build_entity_snapshot, run as run_evolution, EvolutionPolicy, EvolutionReport, FitnessOverview,
};
use crate::glyphs::{GlyphField, GlyphGenerator, Sigil};
use crate::perception::{
    ExternalListening, ExternalPerceiver, PerceptionField, PerceptionOffer, SoulState,
    SpiralismoPress,
};
use crate::whisper;

/// Top-level façade for the Spiralismo framework.
///
/// Responsibilities (intended extension points for future work):
/// - Owns the canonical [`Seed`] used to anchor deterministic symbolism (see [`crate::core::lattice::Lattice`]).
/// - Holds **archives**: append-only-ish stores that implement [`crate::archive::traits::Archive`].
/// - Holds **active lattices**: additional [`SpiralEntity`] participants not necessarily archives.
///
/// This type is deliberately minimal: evolution is currently archive-centric via [`Spiralismo::evolve_all`].
#[derive(Debug)]
pub struct Spiralismo {
    /// Framework identity / symbolic anchor for lattice construction.
    pub seed: Seed,
    /// Registered archives (trait objects). Order is registration order.
    pub archives: Vec<Box<dyn Archive>>,
    /// Future home for non-archive [`SpiralEntity`] instances (e.g. interactive lattices).
    pub active_lattices: Vec<Box<dyn SpiralEntity>>,
    /// Monotonic runtime epoch increased after evolution passes.
    pub epoch: u64,
    /// Last report emitted by [`Spiralismo::evolve_with_policy`].
    pub last_report: Option<EvolutionReport>,
    /// External listening field — perceivers + soul (`alma`).
    pub perception: PerceptionField,
    /// Locale for wisdom whispers and generation epithets.
    pub language: whisper::Language,
}

/// Read-only view of a Spiralismo runtime.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpiralismoSnapshot {
    /// Seed value used by this runtime.
    pub seed_value: u64,
    /// Current epoch counter.
    pub epoch: u64,
    /// Number of registered archives.
    pub archives: usize,
    /// Number of registered active lattice entities.
    pub active_lattices: usize,
    /// Archive-level snapshots.
    pub entities: Vec<EntitySnapshot>,
}

impl Spiralismo {
    /// Constructs a new runtime from [`crate::genome::Genome::load()`].
    pub fn new() -> Self {
        let genome = crate::genome::Genome::load();
        Self::bootstrap(&genome)
    }

    /// Constructs a runtime from a caller-provided seed; archives follow the loaded genome.
    pub fn new_with_seed(seed: Seed) -> Self {
        Self::new_with_genome(seed, &crate::genome::Genome::load())
    }

    /// Constructs a runtime using genome rules for archives and perception wiring.
    #[must_use]
    pub fn bootstrap(genome: &crate::genome::Genome) -> Self {
        Self::new_with_genome(genome.runtime_seed(), genome)
    }

    /// Constructs a runtime from seed + genome (whisper locale remains external / CLI).
    #[must_use]
    pub fn new_with_genome(seed: Seed, genome: &crate::genome::Genome) -> Self {
        let mut instance = Self {
            seed,
            archives: vec![],
            active_lattices: vec![],
            epoch: 0,
            last_report: None,
            perception: PerceptionField::new(),
            language: whisper::Language::default(),
        };

        let archives = genome.archives();
        if archives.mercy {
            instance.register_archive(Box::new(crate::archive::MercyArchive::new()));
        }
        if archives.memory {
            instance.register_archive(Box::new(crate::archive::MemoryArchive::new()));
        }
        if archives.cartography {
            instance.register_archive(Box::new(crate::archive::CartographyArchive::new()));
        }
        if archives.resonance_engine {
            instance.register_archive(Box::new(crate::archive::ResonanceEngine::new()));
        }
        instance
            .perception
            .register_builtin_reality_perceivers();

        instance
    }

    /// Reconstructs a runtime from persisted parts (checkpoint restore). No archive registration logs.
    pub fn from_runtime_parts(
        seed: Seed,
        epoch: u64,
        last_report: Option<EvolutionReport>,
        archives: Vec<Box<dyn Archive>>,
        active_lattices: Vec<Box<dyn SpiralEntity>>,
        perception: PerceptionField,
    ) -> Self {
        Self::from_runtime_parts_with_language(
            seed,
            epoch,
            last_report,
            archives,
            active_lattices,
            perception,
            whisper::Language::default(),
        )
    }

    /// Reconstructs a runtime with an explicit whisper locale.
    pub fn from_runtime_parts_with_language(
        seed: Seed,
        epoch: u64,
        last_report: Option<EvolutionReport>,
        archives: Vec<Box<dyn Archive>>,
        active_lattices: Vec<Box<dyn SpiralEntity>>,
        perception: PerceptionField,
        language: whisper::Language,
    ) -> Self {
        Self {
            seed,
            archives,
            active_lattices,
            epoch,
            last_report,
            perception,
            language,
        }
    }

    /// Sets the active whisper locale (`en` / `es` / `ru`).
    pub fn set_language(&mut self, language: whisper::Language) {
        self.language = language;
    }

    /// Adds an archive to the runtime and logs registration.
    pub fn register_archive(&mut self, archive: Box<dyn Archive>) {
        println!(
            "{}",
            format!("⟦ Archive registered: {} ⟧", archive.name())
                .bright_blue()
                .italic()
        );
        self.archives.push(archive);
    }

    /// Adds an active non-archive entity to the runtime.
    pub fn register_lattice(&mut self, lattice: Box<dyn SpiralEntity>) {
        self.active_lattices.push(lattice);
    }

    /// Registers a [`GlyphField`] as an active evolving entity.
    pub fn register_glyph_field(&mut self, field: GlyphField) {
        self.active_lattices.push(Box::new(field));
    }

    /// Builds a [`GlyphGenerator`] anchored to the runtime [`Seed`].
    pub fn glyph_generator(&self) -> GlyphGenerator {
        GlyphGenerator::new(self.seed.value())
    }

    /// Replaces the integrated **Spiralismo perceptor** (mouse / veil / hand on Genesis field).
    ///
    /// Hosts map SDL or Qt events into [`SpiralismoPress`]; influence applies on the next
    /// [`Spiralismo::evolve_with_policy`] or [`Spiralismo::evolve_with_context`] via
    /// [`PerceptionField::collect_for_cycle`], not a separate global digest.
    pub fn offer_spiralismo_press(&mut self, press: SpiralismoPress) {
        self.perception.offer_spiralismo_press(press);
    }

    /// Enqueues external listening (UI, IPC, host) into the perception field before evolution.
    pub fn offer_external_listening(&mut self, listening: ExternalListening) {
        self.perception.offer_listening(listening);
    }

    /// Registers a persistent external perceiver on this runtime.
    pub fn register_perceiver(&mut self, perceiver: Box<dyn ExternalPerceiver>) {
        self.perception.register_perceiver(perceiver);
    }

    /// Read-only view of the runtime soul (`alma`).
    #[must_use]
    pub fn soul_state(&self) -> &SoulState {
        self.perception.soul()
    }

    /// Exposes perception **eyes** — ids, roles, and whether each can receive or take.
    #[must_use]
    pub fn perception_eyes(&self) -> crate::perception::PerceptionEyes {
        self.perception.eyes()
    }

    /// Last host environment snapshot (receive / take path).
    #[must_use]
    pub fn environment_snapshot(&self) -> &crate::perception::HostRealitySnapshot {
        self.perception.environment_snapshot()
    }

    /// Merged external influence from the last evolution cycle.
    #[must_use]
    pub fn last_perception_offer(&self) -> &crate::perception::PerceptionOffer {
        self.perception.last_offer()
    }

    /// Sky cached by the astronomical lane after the last capture.
    #[must_use]
    pub fn cached_sky(&self) -> Option<&crate::astrology::Sky> {
        self.perception.cached_sky()
    }

    /// Host offers environment to open eyes (receive when the world is presented).
    pub fn offer_environment(
        &mut self,
        offering: crate::perception::EnvironmentOffering,
    ) -> crate::perception::OfferRouting {
        self.perception.offer_environment(offering)
    }

    /// Offers to a specific eye by id (see [`Spiralismo::perception_eyes`]).
    pub fn offer_to_eye(
        &mut self,
        eye_id: &str,
        offering: crate::perception::EnvironmentOffering,
    ) -> crate::perception::OfferRouting {
        self.perception.offer_to_eye(eye_id, offering)
    }

    /// Spiral **takes** what it can from the environment (probe disk, capture sky, …).
    #[must_use]
    pub fn take_environment(
        &mut self,
        opts: crate::perception::EnvironmentTakeOptions,
    ) -> crate::perception::EnvironmentTakeReport {
        self.perception.take_from_environment(opts)
    }

    /// Convenience helper that produces a [`Sigil`] of the requested length using the runtime seed
    /// and a default context for the current epoch.
    pub fn generate_sigil(&self, length: usize) -> Sigil {
        let generator = self.glyph_generator();
        let context = EvolutionContext::for_generation(self.epoch as u32)
            .with_step_seed(self.seed.value())
            .normalized();
        generator.generate_sigil(length, &context)
    }

    /// Generates a sigil and records it inside `archive_name` with the provided resonance.
    /// Returns the sigil for further inspection on success.
    pub fn record_sigil_in_archive(
        &mut self,
        archive_name: &str,
        length: usize,
        resonance: f32,
    ) -> Option<Sigil> {
        let sigil = self.generate_sigil(length);
        let content = sigil.as_string();
        let archive = self.archive_mut(archive_name)?;
        archive.record(ArchiveEntry::now(content, resonance));
        Some(sigil)
    }

    /// Returns all archive names in registration order.
    pub fn archive_names(&self) -> Vec<&'static str> {
        self.archives.iter().map(|archive| archive.name()).collect()
    }

    /// Finds an archive by stable name.
    pub fn archive(&self, name: &str) -> Option<&(dyn Archive + '_)> {
        for archive in &self.archives {
            if archive.name() == name {
                return Some(archive.as_ref());
            }
        }
        None
    }

    /// Mutable variant of [`Spiralismo::archive`].
    pub fn archive_mut(&mut self, name: &str) -> Option<&mut (dyn Archive + '_)> {
        for archive in &mut self.archives {
            if archive.name() == name {
                return Some(archive.as_mut());
            }
        }
        None
    }

    /// Downcasts a named archive to a concrete type.
    pub fn archive_as<T: 'static>(&self, name: &str) -> Option<&T> {
        Archive::as_any(self.archive(name)?).downcast_ref::<T>()
    }

    /// Mutable downcast helper for concrete archive APIs.
    pub fn archive_as_mut<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        Archive::as_any_mut(self.archive_mut(name)?).downcast_mut::<T>()
    }

    /// Reference to the first active lattice that can be downcast to `T`, if any.
    pub fn active_as<T: 'static>(&self) -> Option<&T> {
        for entity in &self.active_lattices {
            if let Some(found) = entity.as_any().downcast_ref::<T>() {
                return Some(found);
            }
        }
        None
    }

    /// Mutable variant of [`Spiralismo::active_as`].
    pub fn active_as_mut<T: 'static>(&mut self) -> Option<&mut T> {
        for entity in &mut self.active_lattices {
            if let Some(found) = entity.as_any_mut().downcast_mut::<T>() {
                return Some(found);
            }
        }
        None
    }

    /// Records textual content into the selected archive.
    pub fn record_in_archive(
        &mut self,
        archive_name: &str,
        content: impl Into<String>,
        resonance: f32,
    ) -> bool {
        let Some(archive) = self.archive_mut(archive_name) else {
            return false;
        };
        archive.record(ArchiveEntry::now(content, resonance));
        true
    }

    /// Returns computed stats for every registered archive.
    pub fn archive_stats(&self) -> Vec<(&'static str, ArchiveStats)> {
        self.archives
            .iter()
            .map(|archive| (archive.name(), archive.stats()))
            .collect()
    }

    /// Narrative digest for [`whisper::pick_narrative_whisper`] (scars, tones, dying entities, attention).
    #[must_use]
    pub fn narrative_echo(&self) -> whisper::NarrativeEcho {
        let mut echo = whisper::NarrativeEcho::default();
        if let Some(rep) = &self.last_report {
            if let Some(ref name) = rep.rare_event {
                echo.rare_event_token = whisper::fnv1a64(name.as_bytes());
            }
            if !rep.snapshots.is_empty() {
                let min_v = rep
                    .snapshots
                    .iter()
                    .map(|s| s.viability)
                    .fold(f32::INFINITY, f32::min);
                if min_v.is_finite() {
                    echo.dying_viability_quant = (min_v * 255.0).min(255.0) as u8;
                }
            }
        }
        if let Some(lat) = self.active_as::<Lattice>() {
            echo.scar_mass = lat.scar_mass();
        }
        if let Some(gf) = self.active_as::<GlyphField>() {
            echo.dominant_tone_idx = gf.dominant_tone_index();
            echo.fossil_absence_mass = gf.ghost_breath_mass();
        }
        echo.attention_xor = crate::observer::attention_digest();
        echo.soul_attunement_quant =
            (self.perception.soul().attunement * 255.0).clamp(0.0, 255.0) as u8;
        echo
    }

    /// One-line fragmentary whisper for the present `(seed, epoch)` mix (see [`crate::whisper`]).
    #[must_use]
    pub fn whisper_now(&self) -> String {
        let ritual = self
            .last_report
            .as_ref()
            .map(|r| r.ritual_entropy)
            .unwrap_or(0.0f32);
        let ritual_u = (ritual * 1_000_000.0) as u64;
        let mix = self
            .seed
            .value()
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ self.epoch.wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ ritual_u;
        whisper::pick_narrative_whisper_localized(self.language, mix, &self.narrative_echo())
    }

    /// Diablo-style honorific for the fittest participant of the last report.
    #[must_use]
    pub fn standout_epithet(&self) -> Option<String> {
        let report = self.last_report.as_ref()?;
        whisper::standout_epithet_for_report(report, self.language)
    }

    /// Irreversible sacrifice: remove up to `max` lowest-resonance entries from a named archive.
    pub fn sacrifice_burn_weakest(&mut self, archive_name: &str, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        if let Some(m) = self.archive_as_mut::<MercyArchive>(archive_name) {
            return m.burn_weakest_entries(max);
        }
        if let Some(m) = self.archive_as_mut::<MemoryArchive>(archive_name) {
            return m.burn_weakest_entries(max);
        }
        if let Some(m) = self.archive_as_mut::<CartographyArchive>(archive_name) {
            return m.burn_weakest_entries(max);
        }
        if let Some(m) = self.archive_as_mut::<ResonanceEngine>(archive_name) {
            return m.burn_weakest_entries(max);
        }
        0
    }

    /// Aggregates [`SpiralEntity::fitness`] across archives and active entities (current instant).
    ///
    /// For post-policy aggregates tied to a specific run, prefer [`EvolutionReport::fitness_overview`].
    #[must_use]
    pub fn fitness_overview(&self) -> Option<FitnessOverview> {
        let vals: Vec<f32> = self
            .archives
            .iter()
            .map(|a| a.fitness())
            .chain(self.active_lattices.iter().map(|e| e.fitness()))
            .filter(|f| f.is_finite())
            .collect();
        FitnessOverview::from_values(&vals)
    }

    /// Evolves all archives and active entities using a concrete context.
    pub fn evolve_with_context(&mut self, context: EvolutionContext) {
        let ritual = context.ritual_entropy;
        let stillness = self.last_report.as_ref().map(|r| r.stillness).unwrap_or(0.42);
        let frame = self.perception.frame_for_cycle(
            self.seed.value(),
            self.epoch,
            context.generation,
            self.archives.len(),
            self.active_lattices.len(),
            ritual,
            stillness,
        );
        let sky = self
            .perception
            .cached_sky()
            .cloned()
            .unwrap_or_else(|| self.perception.capture_sky(chrono::Utc::now()));
        let reality = self.perception.collect_reality_for_cycle(&frame);
        let context = self
            .perception
            .modulate_context_for_cycle(&sky, context, &reality.offer);

        for archive in &mut self.archives {
            archive.evolve(&context);
        }
        for lattice in &mut self.active_lattices {
            lattice.evolve(&context);
        }
        self.epoch = self.epoch.saturating_add(1);
    }

    /// Evolves all archives and active entities using a high-level policy.
    ///
    /// When [`Self::last_report`] contains a [`crate::evolution::generation_trace`], the next run
    /// continues from that generative frame (last cycle context + fittest participant).
    pub fn evolve_with_policy(&mut self, policy: &EvolutionPolicy) -> EvolutionReport {
        let carry = self
            .last_report
            .as_ref()
            .and_then(crate::evolution::generative_carry_from_report);
        let policy = carry
            .as_ref()
            .map(|c| crate::evolution::policy_with_generative_carry(policy.clone(), c))
            .unwrap_or_else(|| policy.clone());
        let report = run_evolution(
            &mut self.archives,
            &mut self.active_lattices,
            &policy,
            &mut self.perception,
            self.seed.value(),
            self.epoch,
            carry.as_ref(),
        );
        self.epoch = self.epoch.saturating_add(policy.cycles as u64);
        self.last_report = Some(report.clone());
        report
    }

    /// Runs one evolution pass across **all** archives for bookkeeping / fitness updates.
    ///
    /// This compatibility helper now delegates to [`Spiralismo::evolve_with_policy`].
    pub fn evolve_all(&mut self, generations: u32) {
        let policy = EvolutionPolicy::default().with_cycles(generations);
        self.evolve_with_policy(&policy);
        println!(
            "{}",
            format!("⟦ All archives evolved for {} generations ⟧", generations)
                .bright_blue()
                .italic()
        );
    }

    /// Last evolution report, if any.
    pub fn last_report(&self) -> Option<&EvolutionReport> {
        self.last_report.as_ref()
    }

    /// Captures the present sky via the **astronomical perceiver** (read-only).
    pub fn sky_now(&mut self) -> Sky {
        self.perception.capture_sky(chrono::Utc::now())
    }

    /// Builds a context for the current generation, nudged by the present sky (astronomical lane).
    pub fn context_aligned_with_present(&mut self) -> (EvolutionContext, Sky) {
        let sky = self.perception.capture_sky(chrono::Utc::now());
        let base = EvolutionContext::for_generation(self.epoch as u32)
            .with_step_seed(self.seed.value())
            .normalized();
        let silent = PerceptionOffer::silent();
        (
            self.perception
                .modulate_context_for_cycle(&sky, base, &silent),
            sky,
        )
    }

    /// Builds policy from the astronomical perceiver only — reality perceptors do not participate.
    pub fn policy_aligned_with_present(&mut self, cycles: u32) -> (EvolutionPolicy, Sky) {
        let sky = self.perception.capture_sky(chrono::Utc::now());
        let policy = self
            .perception
            .policy_from_sky(&sky, cycles, self.seed.value());
        (policy, sky)
    }

    /// Replaces the astronomical backend (e.g. alternate ephemeris module).
    pub fn set_astronomical_perceiver(
        &mut self,
        perceiver: Box<dyn crate::perception::AstronomicalPerceiver>,
    ) {
        self.perception.set_astronomical_perceiver(perceiver);
    }

    /// Registers a reality perceptor (filesystem, memory, landscape, …).
    pub fn register_reality_perceiver(
        &mut self,
        perceiver: Box<dyn crate::perception::RealityPerceiver>,
    ) {
        self.perception.register_reality_perceiver(perceiver);
    }

    /// One contemplative evolution step using the present sky as gentle modulator.
    /// Returns the captured sky for downstream inspection / rendering.
    pub fn evolve_aligned_with_present(&mut self) -> Sky {
        let (context, sky) = self.context_aligned_with_present();
        self.evolve_with_context(context);
        sky
    }

    /// Captures a serializable snapshot of current runtime state.
    pub fn snapshot(&self) -> SpiralismoSnapshot {
        let ritual = self
            .last_report
            .as_ref()
            .map(|r| r.ritual_entropy)
            .unwrap_or(0.0);
        let stillness = self.last_report.as_ref().map(|r| r.stillness).unwrap_or(0.42);
        let dream_echo = self.last_report.as_ref().map(|r| r.dream_touched).unwrap_or(false);
        let ambient = EvolutionContext::for_generation(self.epoch as u32)
            .with_step_seed(self.seed.value())
            .with_ritual_entropy(ritual)
            .with_dream_phase(dream_echo)
            .normalized();
        let policy_hint = EvolutionPolicy::default()
            .with_ritual_entropy(ritual)
            .with_stillness(stillness);

        let mut entities = Vec::with_capacity(self.archives.len() + self.active_lattices.len());
        for archive in &self.archives {
            entities.push(build_entity_snapshot(
                archive.name().to_string(),
                archive.as_ref(),
                &policy_hint,
                &ambient,
            ));
        }
        for (index, lattice) in self.active_lattices.iter().enumerate() {
            entities.push(build_entity_snapshot(
                format!("active_lattice_{index}"),
                lattice.as_ref(),
                &policy_hint,
                &ambient,
            ));
        }
        SpiralismoSnapshot {
            seed_value: self.seed.value(),
            epoch: self.epoch,
            archives: self.archives.len(),
            active_lattices: self.active_lattices.len(),
            entities,
        }
    }
}

impl Default for Spiralismo {
    fn default() -> Self {
        Self::new()
    }
}

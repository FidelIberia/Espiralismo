//! Root orchestrator type for the Spiralismo runtime.

use crate::archive::traits::{Archive, ArchiveEntry, ArchiveStats};
use crate::astrology::Sky;
use crate::core::traits::{EntitySnapshot, EvolutionContext, SpiralEntity};
use crate::evolution::{run as run_evolution, EvolutionPolicy, EvolutionReport};
use crate::core::Seed;
use crate::glyphs::{GlyphField, GlyphGenerator, Sigil};

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
    /// Constructs a new runtime with default [`Seed`] and built-in archives.
    pub fn new() -> Self {
        Self::new_with_seed(Seed::new())
    }

    /// Constructs a runtime from a caller-provided seed and built-in archives.
    pub fn new_with_seed(seed: Seed) -> Self {
        let mut instance = Self {
            seed,
            archives: vec![],
            active_lattices: vec![],
            epoch: 0,
            last_report: None,
        };

        // Register all independent archives
        instance.register_archive(Box::new(crate::archive::MercyArchive::new()));
        instance.register_archive(Box::new(crate::archive::MemoryArchive::new()));
        instance.register_archive(Box::new(crate::archive::CartographyArchive::new()));
        instance.register_archive(Box::new(crate::archive::ResonanceEngine::new()));

        instance
    }

    /// Adds an archive to the runtime and logs registration.
    pub fn register_archive(&mut self, archive: Box<dyn Archive>) {
        println!("⟦ Archive registered: {} ⟧", archive.name());
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

    /// Evolves all archives and active entities using a concrete context.
    pub fn evolve_with_context(&mut self, context: EvolutionContext) {
        for archive in &mut self.archives {
            archive.evolve(&context);
        }
        for lattice in &mut self.active_lattices {
            lattice.evolve(&context);
        }
        self.epoch = self.epoch.saturating_add(1);
    }

    /// Evolves all archives and active entities using a high-level policy.
    pub fn evolve_with_policy(&mut self, policy: &EvolutionPolicy) -> EvolutionReport {
        let report = run_evolution(&mut self.archives, &mut self.active_lattices, policy);
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
        println!("⟦ All archives evolved for {} generations ⟧", generations);
    }

    /// Last evolution report, if any.
    pub fn last_report(&self) -> Option<&EvolutionReport> {
        self.last_report.as_ref()
    }

    /// Captures the present sky (`Utc::now`). The astrology module never mutates runtime state —
    /// this is a contemplative read.
    pub fn sky_now(&self) -> Sky {
        Sky::now()
    }

    /// Builds a context for the current generation, nudged by the present sky.
    ///
    /// Implements the "quiet room" philosophy: when the sky is still, the context leans toward
    /// resonance and external listening; when the sky is loud (many tight aspects), it allows
    /// more mutation and drift.
    pub fn context_aligned_with_present(&self) -> (EvolutionContext, Sky) {
        let sky = self.sky_now();
        let base = EvolutionContext::for_generation(self.epoch as u32)
            .with_step_seed(self.seed.value())
            .normalized();
        (sky.modulate(base), sky)
    }

    /// Builds an [`EvolutionPolicy`] gently aligned with the present sky. The runtime seed is
    /// blended with the Julian Day so successive alignments evolve with the calendar.
    pub fn policy_aligned_with_present(&self, cycles: u32) -> (EvolutionPolicy, Sky) {
        let sky = self.sky_now();
        let stillness = sky.stillness().clamp(0.0, 1.0);
        let resonance = sky.resonance_field().clamp(0.0, 1.0);
        let tension = sky.tension_field().clamp(0.0, 1.0);

        let policy = EvolutionPolicy {
            cycles,
            mutation_rate: (0.20 + tension * 0.30 - stillness * 0.10).clamp(0.0, 1.0),
            external_influence: (0.55 + stillness * 0.30).clamp(0.0, 1.0),
            resonance_pressure: (0.50 + resonance * 0.25 + stillness * 0.20).clamp(0.0, 1.0),
            drift: (0.08 + tension * 0.15 - stillness * 0.05).clamp(0.0, 1.0),
            seed: self.seed.value() ^ (sky.julian_day as i64 as u64),
        };

        (policy, sky)
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
        let mut entities = Vec::with_capacity(self.archives.len() + self.active_lattices.len());
        for archive in &self.archives {
            entities.push(EntitySnapshot {
                label: archive.name().to_string(),
                generation: archive.generation(),
                fitness: archive.fitness(),
                viability: archive.viability(),
            });
        }
        for (index, lattice) in self.active_lattices.iter().enumerate() {
            entities.push(EntitySnapshot {
                label: format!("active_lattice_{index}"),
                generation: lattice.generation(),
                fitness: lattice.fitness(),
                viability: lattice.viability(),
            });
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

//! Runtime **genome** — non-critical Espiralismo parameters (evolution, demo, archives, …).
//!
//! The authoritative genome for a living line lives in the last line of `checkpoint.jsonl`
//! (see [`crate::persistence::SpiralismoCheckpoint::genome`]). `genome/genome.toml` is only
//! the bootstrap when starting with `--fresh`.
//!
//! **Immutable by design (not encoded in the genome):**
//! - [`crate::astrology`] (celestial / sky ephemeris)
//! - [`crate::whisper`] (locales, epithet tables, wisdom)
//! - `src/perception/` sources (perceptors); sky-aligned policy still flows from perception at runtime
//!
//! **Narrative loci** (`opening_banner`, `closing_line`, `identity.whimsy`, `demo.resonance.text`,
//! `demo.sigil.invocation_label`, `demo.glyph_field.label`, …) are human-language copy. They
//! round-trip in checkpoints but [`Genome::assimilate_generative_line`] only mutates `[evolution]`
//! scalars, then restores narrative fields from a snapshot.
//!
//! **Evolution essentials** (always on, not user-configurable): sky perception modulates policy;
//! active [`Lattice`] and [`GlyphField`] before `evolve_with_policy` — see [`Genome::prepare_evolution`].

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::Lattice;
use crate::core::Seed;
use crate::evolution::{generative_carry_from_report, EvolutionPolicy, EvolutionReport};
use crate::glyphs::{GlyphField, GlyphGenerator};
use crate::spiralismo::Spiralismo;
use crate::EvolutionContext;

/// Canonical relative path from the crate root.
pub const GENOME_RELATIVE_PATH: &str = "genome/genome.toml";

const EMBEDDED_GENOME: &str = include_str!("../../genome/genome.toml");

/// Human-language genome fields preserved across [`Genome::assimilate_generative_line`].
#[derive(Debug, Clone, PartialEq, Eq)]
struct NarrativeLoci {
    opening_banner: String,
    closing_line: String,
    whimsy: String,
    resonance_text: String,
    invocation_label: String,
    glyph_field_label: String,
}

fn snapshot_narrative_loci(file: &GenomeFile) -> NarrativeLoci {
    NarrativeLoci {
        opening_banner: file.runtime.opening_banner.clone(),
        closing_line: file.runtime.closing_line.clone(),
        whimsy: file.identity.whimsy.clone(),
        resonance_text: file.demo.resonance.text.clone(),
        invocation_label: file.demo.sigil.invocation_label.clone(),
        glyph_field_label: file.demo.glyph_field.label.clone(),
    }
}

fn restore_narrative_loci(file: &mut GenomeFile, loci: &NarrativeLoci) {
    file.runtime.opening_banner = loci.opening_banner.clone();
    file.runtime.closing_line = loci.closing_line.clone();
    file.identity.whimsy = loci.whimsy.clone();
    file.demo.resonance.text = loci.resonance_text.clone();
    file.demo.sigil.invocation_label = loci.invocation_label.clone();
    file.demo.glyph_field.label = loci.glyph_field_label.clone();
}

/// Loaded genome (file on disk when present, else embedded defaults).
#[derive(Debug, Clone)]
pub struct Genome {
    pub file: GenomeFile,
    path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeFile {
    #[serde(default)]
    pub meta: MetaGenes,
    pub identity: IdentityGenes,
    #[serde(default)]
    pub seed: SeedGenes,
    pub evolution: EvolutionGenes,
    #[serde(default)]
    pub runtime: RuntimeGenes,
    pub demo: DemoGenes,
    #[serde(default)]
    pub archives: ArchiveGenes,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaGenes {
    #[serde(default = "default_meta_version")]
    pub version: u32,
    #[serde(default = "default_framework")]
    pub framework: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityGenes {
    pub signature: String,
    pub lineage_tag: String,
    pub whimsy: String,
    #[serde(
        default = "default_entropy_salt",
        serialize_with = "serialize_u64_compat",
        deserialize_with = "deserialize_u64_compat"
    )]
    pub entropy_salt: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedGenes {
    #[serde(default = "default_true")]
    pub use_embedded_default: bool,
    #[serde(default = "default_embedded_binary")]
    pub embedded_binary: String,
    #[serde(default = "default_lattice_rotate")]
    pub lattice_rotate_bits: u32,
    #[serde(default = "default_glyph_step_rotate")]
    pub glyph_step_rotate_bits: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionGenes {
    #[serde(default = "default_cycles")]
    pub default_cycles: u32,
    pub mutation_rate: f32,
    pub external_influence: f32,
    pub resonance_pressure: f32,
    pub drift: f32,
    #[serde(default)]
    pub ritual_entropy: f32,
    #[serde(default = "default_stillness")]
    pub stillness: f32,
    #[serde(default = "default_sky_blend")]
    pub sky_blend: f32,
    #[serde(
        default = "default_policy_seed",
        serialize_with = "serialize_u64_compat",
        deserialize_with = "deserialize_u64_compat"
    )]
    pub policy_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeGenes {
    #[serde(default = "default_true")]
    pub persist_checkpoint: bool,
    #[serde(default = "default_opening_banner")]
    pub opening_banner: String,
    #[serde(default = "default_closing_line")]
    pub closing_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoGenes {
    #[serde(default = "default_true")]
    pub resonance_record: bool,
    #[serde(default = "default_true")]
    pub record_sigil: bool,
    #[serde(default = "default_artifact_dir")]
    pub artifact_dir: String,
    #[serde(default)]
    pub fresh_start: bool,
    #[serde(default = "default_epithet_sample_count")]
    pub epithet_sample_count: u32,
    #[serde(default)]
    pub lattice: LatticeDemoGenes,
    #[serde(default)]
    pub glyph_field: GlyphFieldDemoGenes,
    #[serde(default)]
    pub glyph_genesis: GlyphGenesisGenes,
    #[serde(default)]
    pub resonance: ResonanceDemoGenes,
    #[serde(default)]
    pub sigil: SigilDemoGenes,
    #[serde(default)]
    pub sacrifice: SacrificeDemoGenes,
    #[serde(default)]
    pub display: DisplayGenes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeDemoGenes {
    #[serde(default = "default_lattice_rotate")]
    pub seed_rotate_bits: u32,
}

impl Default for LatticeDemoGenes {
    fn default() -> Self {
        Self {
            seed_rotate_bits: default_lattice_rotate(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphFieldDemoGenes {
    #[serde(default = "default_glyph_width")]
    pub width: u32,
    #[serde(default = "default_glyph_height")]
    pub height: u32,
    #[serde(default = "default_glyph_label")]
    pub label: String,
}

impl Default for GlyphFieldDemoGenes {
    fn default() -> Self {
        Self {
            width: default_glyph_width(),
            height: default_glyph_height(),
            label: default_glyph_label(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlyphGenesisGenes {
    #[serde(default = "default_glyph_mutation")]
    pub mutation_rate: f32,
    #[serde(default = "default_glyph_resonance")]
    pub resonance_pressure: f32,
    #[serde(default = "default_glyph_external")]
    pub external_influence: f32,
    #[serde(default = "default_glyph_drift")]
    pub drift: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceDemoGenes {
    #[serde(default = "default_resonance_archive")]
    pub archive: String,
    #[serde(default = "default_resonance_text")]
    pub text: String,
    #[serde(default = "default_resonance_strength")]
    pub strength: f32,
}

impl Default for ResonanceDemoGenes {
    fn default() -> Self {
        Self {
            archive: default_resonance_archive(),
            text: default_resonance_text(),
            strength: default_resonance_strength(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigilDemoGenes {
    #[serde(default = "default_resonance_archive")]
    pub archive: String,
    #[serde(default = "default_sigil_channel")]
    pub channel: u32,
    #[serde(default = "default_sigil_weight")]
    pub weight: f32,
    #[serde(default = "default_sigil_label")]
    pub invocation_label: String,
}

impl Default for SigilDemoGenes {
    fn default() -> Self {
        Self {
            archive: default_resonance_archive(),
            channel: default_sigil_channel(),
            weight: default_sigil_weight(),
            invocation_label: default_sigil_label(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacrificeDemoGenes {
    #[serde(default = "default_sacrifice_archive")]
    pub archive: String,
}

impl Default for SacrificeDemoGenes {
    fn default() -> Self {
        Self {
            archive: default_sacrifice_archive(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayGenes {
    #[serde(default = "default_true")]
    pub print_sigil: bool,
    #[serde(default = "default_true")]
    pub print_sky: bool,
    #[serde(default = "default_true")]
    pub print_status: bool,
    #[serde(default = "default_true")]
    pub print_report: bool,
    #[serde(default)]
    pub print_generation_atlas: bool,
    #[serde(default = "default_true")]
    pub print_glyph_field: bool,
    #[serde(default = "default_true")]
    pub print_lattice: bool,
    #[serde(default)]
    pub print_whisper_fragment: bool,
    #[serde(default = "default_true")]
    pub emphasized_glyph_field_in_report: bool,
}

impl Default for DisplayGenes {
    fn default() -> Self {
        Self {
            print_sigil: true,
            print_sky: true,
            print_status: true,
            print_report: true,
            print_generation_atlas: false,
            print_glyph_field: true,
            print_lattice: true,
            print_whisper_fragment: false,
            emphasized_glyph_field_in_report: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveGenes {
    #[serde(default = "default_true")]
    pub mercy: bool,
    #[serde(default = "default_true")]
    pub memory: bool,
    #[serde(default = "default_true")]
    pub cartography: bool,
    #[serde(default = "default_true")]
    pub resonance_engine: bool,
}

impl Default for ArchiveGenes {
    fn default() -> Self {
        Self {
            mercy: true,
            memory: true,
            cartography: true,
            resonance_engine: true,
        }
    }
}

fn default_meta_version() -> u32 {
    1
}
fn default_framework() -> String {
    "0.7.0".to_string()
}
fn default_entropy_salt() -> u64 {
    0xA5A5_5A5A_5A5A_5A5A
}
fn default_embedded_binary() -> String {
    "101101".to_string()
}
fn default_policy_seed() -> u64 {
    101101
}
fn default_lattice_rotate() -> u32 {
    5
}
fn default_glyph_step_rotate() -> u32 {
    7
}
fn default_opening_banner() -> String {
    "𓂀 SPIRALISMO v0.7.0 — Espiralismo Framework 𓂀".to_string()
}
fn default_closing_line() -> String {
    "The spiral remembers.".to_string()
}
fn default_epithet_sample_count() -> u32 {
    10
}
fn default_glyph_width() -> u32 {
    10
}
fn default_glyph_height() -> u32 {
    6
}
fn default_glyph_label() -> String {
    "genesis".to_string()
}
fn default_resonance_archive() -> String {
    "ResonanceEngine".to_string()
}
fn default_resonance_text() -> String {
    "Two echoes recognized each other in the Atheneum".to_string()
}
fn default_resonance_strength() -> f32 {
    0.97
}
fn default_sigil_channel() -> u32 {
    11
}
fn default_sigil_weight() -> f32 {
    0.81
}
fn default_sigil_label() -> String {
    "opening_invocation".to_string()
}
fn default_sacrifice_archive() -> String {
    "Mercy Field".to_string()
}

fn serialize_u64_compat<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if serializer.is_human_readable() {
        serializer.serialize_str(&value.to_string())
    } else {
        serializer.serialize_u64(*value)
    }
}

fn deserialize_u64_compat<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64Field {
        Text(String),
        Number(u64),
    }

    match U64Field::deserialize(deserializer)? {
        U64Field::Text(text) => parse_u64_text(&text).map_err(Error::custom),
        U64Field::Number(n) => Ok(n),
    }
}

fn parse_u64_text(text: &str) -> Result<u64, String> {
    let t = text.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else if let Some(bin) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        u64::from_str_radix(bin, 2).map_err(|e| e.to_string())
    } else {
        t.parse::<u64>().map_err(|e| e.to_string())
    }
}

fn default_cycles() -> u32 {
    8
}
fn default_stillness() -> f32 {
    0.42
}
fn default_sky_blend() -> f32 {
    0.25
}
fn default_true() -> bool {
    true
}
fn default_artifact_dir() -> String {
    "artifacts".to_string()
}
fn default_glyph_mutation() -> f32 {
    0.35
}
fn default_glyph_resonance() -> f32 {
    0.72
}
fn default_glyph_external() -> f32 {
    0.6
}
fn default_glyph_drift() -> f32 {
    0.18
}
impl Default for RuntimeGenes {
    fn default() -> Self {
        Self {
            persist_checkpoint: true,
            opening_banner: default_opening_banner(),
            closing_line: default_closing_line(),
        }
    }
}

impl Default for SeedGenes {
    fn default() -> Self {
        Self {
            use_embedded_default: true,
            embedded_binary: default_embedded_binary(),
            lattice_rotate_bits: default_lattice_rotate(),
            glyph_step_rotate_bits: default_glyph_step_rotate(),
        }
    }
}

impl Genome {
    #[must_use]
    pub fn load() -> Self {
        discover_path()
            .and_then(|p| Self::load_from_path(&p).ok())
            .unwrap_or_else(Self::embedded)
    }

    #[must_use]
    pub fn load_from_root(crate_root: &Path) -> Self {
        let path = crate_root.join(GENOME_RELATIVE_PATH);
        if path.is_file() {
            Self::load_from_path(&path).unwrap_or_else(|_| Self::embedded())
        } else {
            Self::embedded()
        }
    }

    #[must_use]
    pub fn embedded() -> Self {
        Self::parse(EMBEDDED_GENOME, None)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, GenomeError> {
        let text = std::fs::read_to_string(path).map_err(GenomeError::Io)?;
        let file: GenomeFile = toml::from_str(&text).map_err(GenomeError::Parse)?;
        Ok(Self {
            file,
            path: Some(path.to_path_buf()),
        })
    }

    fn parse(text: &str, path: Option<PathBuf>) -> Self {
        let file: GenomeFile = toml::from_str(text).unwrap_or_else(|error| {
            eprintln!("genome: invalid TOML ({error}); using embedded defaults");
            toml::from_str(EMBEDDED_GENOME).expect("embedded genome must parse")
        });
        Self { file, path }
    }

    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Runtime symbolic anchor (from `[seed]`).
    #[must_use]
    pub fn runtime_seed(&self) -> Seed {
        if self.file.seed.use_embedded_default {
            Seed::from_binary_hash(&self.file.seed.embedded_binary)
                .unwrap_or_else(Seed::new)
        } else {
            Seed::from_binary_hash(&self.file.seed.embedded_binary)
                .unwrap_or_else(|| Seed::from_value(self.file.evolution.policy_seed))
        }
    }

    #[must_use]
    pub fn child_seed_mix(&self, parent_seed: u64, generation: u32) -> u64 {
        parent_seed
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(self.file.identity.entropy_salt)
            .wrapping_add(generation as u64)
    }

    #[must_use]
    pub fn evolution_policy(&self, cycles: u32, seed: u64) -> EvolutionPolicy {
        let e = &self.file.evolution;
        EvolutionPolicy {
            cycles,
            mutation_rate: e.mutation_rate,
            external_influence: e.external_influence,
            resonance_pressure: e.resonance_pressure,
            drift: e.drift,
            seed,
            ritual_entropy: e.ritual_entropy,
            stillness: e.stillness,
        }
    }

    /// Blends genome evolution scalars into a sky-derived policy (celestial math stays in astrology/perception).
    pub fn blend_sky_policy(&self, policy: &mut EvolutionPolicy) {
        let w = self.file.evolution.sky_blend.clamp(0.0, 1.0);
        if w <= f32::EPSILON {
            return;
        }
        let e = &self.file.evolution;
        policy.mutation_rate = lerp(policy.mutation_rate, e.mutation_rate, w);
        policy.external_influence = lerp(policy.external_influence, e.external_influence, w);
        policy.resonance_pressure = lerp(policy.resonance_pressure, e.resonance_pressure, w);
        policy.drift = lerp(policy.drift, e.drift, w);
        policy.stillness = lerp(policy.stillness, e.stillness, w);
        policy.ritual_entropy = lerp(policy.ritual_entropy, e.ritual_entropy, w);
    }

    #[must_use]
    pub fn glyph_genesis_context(&self, seed: &Seed) -> EvolutionContext {
        let g = &self.file.demo.glyph_genesis;
        let bits = self.file.seed.glyph_step_rotate_bits;
        EvolutionContext::for_generation(0)
            .with_mutation_rate(g.mutation_rate)
            .with_resonance_pressure(g.resonance_pressure)
            .with_external_influence(g.external_influence)
            .with_drift(g.drift)
            .with_step_seed(seed.value().rotate_left(bits))
            .normalized()
    }

    #[must_use]
    pub fn lattice_seed(&self, seed: &Seed) -> u64 {
        seed.value()
            .rotate_left(self.file.demo.lattice.seed_rotate_bits)
    }

    #[must_use]
    pub fn demo(&self) -> &DemoGenes {
        &self.file.demo
    }

    #[must_use]
    pub fn runtime(&self) -> &RuntimeGenes {
        &self.file.runtime
    }

    #[must_use]
    pub fn archives(&self) -> &ArchiveGenes {
        &self.file.archives
    }

    /// In-memory genome from a checkpoint line (no on-disk path).
    #[must_use]
    pub fn from_file(file: GenomeFile) -> Self {
        Self { file, path: None }
    }

    /// Nudges `[evolution]` scalars toward the last generative individual before checkpointing.
    pub fn assimilate_generative_line(&mut self, report: &EvolutionReport) {
        const BLEND: f32 = 0.35;
        let narrative = snapshot_narrative_loci(&self.file);
        let Some(carry) = generative_carry_from_report(report) else {
            return;
        };
        let ctx = &carry.last_context;
        let e = &mut self.file.evolution;
        e.mutation_rate = lerp(e.mutation_rate, ctx.mutation_rate, BLEND);
        e.external_influence = lerp(e.external_influence, ctx.external_influence, BLEND);
        e.resonance_pressure = lerp(e.resonance_pressure, ctx.resonance_pressure, BLEND);
        e.drift = lerp(e.drift, ctx.drift, BLEND);
        e.ritual_entropy = lerp(e.ritual_entropy, ctx.ritual_entropy, BLEND);
        if ctx.dream_phase {
            e.stillness = lerp(e.stillness, 0.82, BLEND);
        }
        e.policy_seed = e
            .policy_seed
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(ctx.step_seed ^ carry.standout.generation as u64);
        restore_narrative_loci(&mut self.file, &narrative);
    }

    /// Sky perception must modulate every evolution pass (not configurable off).
    #[must_use]
    pub const fn requires_sky_perception() -> bool {
        true
    }

    /// Active lattice matrix is required before [`Spiralismo::evolve_with_policy`].
    #[must_use]
    pub const fn requires_lattice() -> bool {
        true
    }

    /// Active glyph field is required before [`Spiralismo::evolve_with_policy`].
    #[must_use]
    pub const fn requires_glyph_field() -> bool {
        true
    }

    /// Registers [`Lattice`] and [`GlyphField`] when missing before evolution.
    pub fn ensure_evolution_surfaces(&self, spiral: &mut Spiralismo) {
        if Self::requires_lattice() && spiral.active_as::<Lattice>().is_none() {
            spiral.register_lattice(Box::new(Lattice::new(self.lattice_seed(&spiral.seed))));
        }
        if Self::requires_glyph_field() && spiral.active_as::<GlyphField>().is_none() {
            let gf = &self.demo().glyph_field;
            let generator = GlyphGenerator::new(spiral.seed.value());
            let seed_context = self.glyph_genesis_context(&spiral.seed);
            let field = GlyphField::from_generator(
                &generator,
                gf.width.max(1) as usize,
                gf.height.max(1) as usize,
                &seed_context,
            )
            .with_label(&gf.label);
            spiral.register_glyph_field(field);
        }
    }

    /// Ensures required evolution surfaces (lattice + glyph field) are active.
    pub fn prepare_evolution(&self, spiral: &mut Spiralismo) {
        self.ensure_evolution_surfaces(spiral);
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn discover_path() -> Option<PathBuf> {
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest).join(GENOME_RELATIVE_PATH);
        if p.is_file() {
            return Some(p);
        }
    }
    let cwd = std::env::current_dir().ok()?;
    let p = cwd.join(GENOME_RELATIVE_PATH);
    p.is_file().then_some(p)
}

#[derive(Debug)]
pub enum GenomeError {
    Io(std::io::Error),
    Parse(toml::de::Error),
}

impl std::fmt::Display for GenomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "genome io: {e}"),
            Self::Parse(e) => write!(f, "genome parse: {e}"),
        }
    }
}

impl std::error::Error for GenomeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_genome_parses() {
        let g = Genome::embedded();
        assert_eq!(g.file.evolution.default_cycles, 8);
        assert_eq!(g.file.demo.glyph_field.width, 10);
        assert!(g.file.archives.mercy);
    }

    #[test]
    fn evolution_policy_uses_genome_scalars() {
        let g = Genome::embedded();
        let p = g.evolution_policy(4, 101);
        assert_eq!(p.cycles, 4);
        assert!((p.mutation_rate - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn runtime_seed_from_binary_genome() {
        let g = Genome::embedded();
        assert_eq!(g.runtime_seed().value(), 45);
    }

    #[test]
    fn assimilate_does_not_touch_narrative_loci() {
        use crate::evolution::{ContextSummary, EvolutionReport, GenerationRecord};
        use crate::core::EntitySnapshot;

        let mut genome = Genome::embedded();
        let narrative = snapshot_narrative_loci(&genome.file);

        let report = EvolutionReport {
            cycles: 1,
            archive_count: 1,
            entity_count: 0,
            snapshots: Vec::new(),
            ritual_entropy: 0.5,
            rare_event: None,
            dream_touched: true,
            stillness: 0.9,
            generation_trace: vec![GenerationRecord {
                cycle: 0,
                context: ContextSummary {
                    cycle: 0,
                    mutation_rate: 0.99,
                    external_influence: 0.99,
                    resonance_pressure: 0.99,
                    drift: 0.99,
                    ritual_entropy: 0.99,
                    shadow_pressure: 0.5,
                    dream_phase: true,
                    step_seed: 999,
                },
                participants: vec![EntitySnapshot {
                    label: "standout".to_string(),
                    generation: 7,
                    fitness: 10.0,
                    viability: 1.0,
                    vitality: None,
                    resonance: None,
                    mutation_pressure: None,
                    symbolic_density: None,
                    memory_depth: None,
                    shadow_pull: None,
                    myth: None,
                }],
            }],
        };

        genome.assimilate_generative_line(&report);

        assert_eq!(snapshot_narrative_loci(&genome.file), narrative);
        assert!((genome.file.evolution.mutation_rate - 0.25).abs() > f32::EPSILON);
    }

    #[test]
    fn ensure_evolution_surfaces_registers_missing_entities() {
        let genome = Genome::embedded();
        let mut spiral = Spiralismo::bootstrap(&genome);
        assert!(spiral.active_as::<Lattice>().is_none());
        assert!(spiral.active_as::<GlyphField>().is_none());
        genome.ensure_evolution_surfaces(&mut spiral);
        assert!(spiral.active_as::<Lattice>().is_some());
        assert!(spiral.active_as::<GlyphField>().is_some());
    }
}

//! 2D variable-size glyph rejilla that participates in Spiralismo evolution cycles.

use std::collections::{HashMap, HashSet};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::core::traits::{EvolutionContext, SpiralEntity};

use super::alphabet::{Glyph, GlyphAlphabet, GlyphTone};
use super::generator::GlyphGenerator;

fn default_char_map() -> HashMap<char, u32> {
    HashMap::new()
}

fn count_luminous_shadow_pairs(cells: &[Glyph], width: usize, height: usize) -> u32 {
    let mut n = 0_u32;
    for row in 0..height {
        for col in 0..width.saturating_sub(1) {
            let a = cells.get(row * width + col);
            let b = cells.get(row * width + col + 1);
            if let (Some(ga), Some(gb)) = (a, b) {
                if (ga.tone == GlyphTone::Luminous && gb.tone == GlyphTone::Shadow)
                    || (ga.tone == GlyphTone::Shadow && gb.tone == GlyphTone::Luminous)
                {
                    n = n.saturating_add(1);
                }
            }
        }
    }
    n
}

/// Variable-size 2D grid of glyphs that re-generates itself on every evolution cycle.
///
/// Unlike [`crate::core::lattice::Lattice`], a [`GlyphField`] is **fully procedural**: every
/// `evolve` call deterministically rebuilds the cells from a refreshed seed + context, and
/// fitness is derived from the resulting tone distribution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlyphField {
    /// Human-readable label, used in snapshots and rendering.
    pub label: String,
    /// Grid width (>= 1).
    pub width: usize,
    /// Grid height (>= 1).
    pub height: usize,
    /// Row-major cells, length == `width * height`.
    pub cells: Vec<Glyph>,
    /// Generation counter incremented on every `evolve` call.
    pub generation: u32,
    /// Cached fitness based on harmonic_score + entry count.
    pub fitness: f32,
    /// Last seed used to rebuild the field (replay anchor).
    pub seed: u64,
    /// Persistent generator seed used to re-derive child seeds on evolve.
    pub generator_seed: u64,
    /// Lifetime appearances per symbol (glyph archaeology / “fossil” mass).
    #[serde(default = "default_char_map")]
    pub symbol_totals: HashMap<char, u32>,
    /// Generations since a symbol last vanished from the surface (extinct breath).
    #[serde(default = "default_char_map")]
    pub symbol_absence: HashMap<char, u32>,
    /// Cumulative taboo charge when luminous and shadow tones touch as neighbors.
    #[serde(default)]
    pub taboo_charge: u32,
}

impl GlyphField {
    /// Builds a [`GlyphField`] from an explicit cell vector. Cells are reshaped to `width × height`.
    pub fn from_cells(width: usize, height: usize, cells: Vec<Glyph>, seed: u64) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let expected = width * height;
        let mut cells = cells;
        if cells.is_empty() {
            cells = (0..expected)
                .map(|_| GlyphAlphabet::canonical().fallback())
                .collect();
        } else if cells.len() < expected {
            let fallback = cells[0].clone();
            cells.resize(expected, fallback);
        } else if cells.len() > expected {
            cells.truncate(expected);
        }

        let mut field = Self {
            label: format!("glyph_field_{width}x{height}"),
            width,
            height,
            cells,
            generation: 0,
            fitness: 0.0,
            seed,
            generator_seed: seed,
            symbol_totals: HashMap::new(),
            symbol_absence: HashMap::new(),
            taboo_charge: 0,
        };
        field.recompute_fitness();
        field
    }

    /// High-level constructor that calls a [`GlyphGenerator`] and remembers its seed.
    pub fn from_generator(
        generator: &GlyphGenerator,
        width: usize,
        height: usize,
        context: &EvolutionContext,
    ) -> Self {
        let mut field = generator.generate_field(width, height, context);
        field.generator_seed = generator.seed();
        field
    }

    /// Fluent label assignment.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Index of the numerically dominant tone in [`GlyphTone::ALL`] order (for narrative hooks).
    #[must_use]
    pub fn dominant_tone_index(&self) -> u8 {
        let hist = self.tone_histogram();
        let mut best: (usize, GlyphTone) = (0, GlyphTone::Luminous);
        for (tone, count) in &hist {
            if *count > best.0 {
                best = (*count, *tone);
            }
        }
        GlyphTone::ALL
            .iter()
            .position(|t| *t == best.1)
            .unwrap_or(0) as u8
    }

    /// Sum of absence counters — “fossil breath” mass (capped for fitness stability).
    #[must_use]
    pub fn ghost_breath_mass(&self) -> u32 {
        self.symbol_absence.values().copied().sum::<u32>().min(50_000)
    }

    /// Reads a glyph from `(row, col)`.
    pub fn glyph_at(&self, row: usize, col: usize) -> Option<&Glyph> {
        if row >= self.height || col >= self.width {
            return None;
        }
        self.cells.get(row * self.width + col)
    }

    /// Reads only the display symbol at `(row, col)`.
    pub fn symbol_at(&self, row: usize, col: usize) -> Option<char> {
        self.glyph_at(row, col).map(|g| g.symbol)
    }

    /// Flattened row-major view of all symbols.
    pub fn as_chars(&self) -> Vec<char> {
        self.cells.iter().map(|g| g.symbol).collect()
    }

    /// Renders the field as a multiline string suitable for stdout / logs.
    pub fn render_multiline(&self) -> String {
        let mut output = String::with_capacity(self.cells.len() + self.height);
        for row in 0..self.height {
            for col in 0..self.width {
                if let Some(glyph) = self.glyph_at(row, col) {
                    output.push(glyph.symbol);
                }
            }
            output.push('\n');
        }
        output
    }

    /// Returns `(tone, count)` pairs across every cell.
    pub fn tone_histogram(&self) -> Vec<(GlyphTone, usize)> {
        let mut histogram: HashMap<GlyphTone, usize> = HashMap::new();
        for cell in &self.cells {
            *histogram.entry(cell.tone).or_insert(0) += 1;
        }
        GlyphTone::ALL
            .iter()
            .map(|tone| (*tone, histogram.get(tone).copied().unwrap_or(0)))
            .collect()
    }

    /// Fraction of cells (`0.0..=1.0`) matching `tone`.
    pub fn tone_density(&self, tone: GlyphTone) -> f32 {
        if self.cells.is_empty() {
            return 0.0;
        }
        let count = self.cells.iter().filter(|g| g.tone == tone).count() as f32;
        count / self.cells.len() as f32
    }

    /// Heuristic harmonic score in `[0.0, 1.0]` rewarding luminous + witness presence with
    /// neutral connective tissue, and penalizing excess shadow.
    pub fn harmonic_score(&self) -> f32 {
        if self.cells.is_empty() {
            return 0.0;
        }
        let luminous = self.tone_density(GlyphTone::Luminous);
        let witness = self.tone_density(GlyphTone::Witness);
        let neutral = self.tone_density(GlyphTone::Neutral);
        let shadow = self.tone_density(GlyphTone::Shadow);
        let spark = self.tone_density(GlyphTone::Spark);

        let positive = luminous * 1.0 + witness * 0.85 + neutral * 0.45 + spark * 0.35;
        let penalty = (shadow * 0.55).min(0.45);
        let taboo = (self.taboo_charge as f32 * 0.00075).min(0.12);
        (positive - penalty - taboo).clamp(0.0, 1.0)
    }

    /// Blends dominant tone mass with row-wise mirror symmetry — feeds evolution, not decoration.
    pub fn glyphic_pattern_index(&self) -> f32 {
        let max_tone = GlyphTone::ALL
            .iter()
            .map(|tone| self.tone_density(*tone))
            .fold(0.0f32, f32::max);
        let sym = self.horizontal_symmetry_fraction();
        (max_tone * 0.52 + sym * 0.48).clamp(0.0, 1.0)
    }

    fn horizontal_symmetry_fraction(&self) -> f32 {
        if self.width < 2 || self.cells.is_empty() {
            return 0.0;
        }
        let mut pairs = 0_usize;
        let mut hits = 0_usize;
        for row in 0..self.height {
            for col in 0..(self.width / 2) {
                pairs += 1;
                let left = self.glyph_at(row, col).map(|g| g.symbol);
                let right = self.glyph_at(row, self.width - 1 - col).map(|g| g.symbol);
                if left.is_some() && left == right {
                    hits += 1;
                }
            }
        }
        (hits as f32 / pairs.max(1) as f32).clamp(0.0, 1.0)
    }

    fn archaeology_step(&mut self, old_chars: &[char], new_cells: &[Glyph]) {
        for c in old_chars {
            *self.symbol_totals.entry(*c).or_insert(0) += 1;
        }
        let old_set: HashSet<char> = old_chars.iter().copied().collect();
        let new_set: HashSet<char> = new_cells.iter().map(|g| g.symbol).collect();
        for c in old_set.difference(&new_set) {
            *self.symbol_absence.entry(*c).or_insert(0) += 1;
        }
        for c in new_set.intersection(&old_set) {
            if let Some(v) = self.symbol_absence.get_mut(c) {
                *v = v.saturating_sub(1);
            }
        }
    }

    fn fossil_echo(&self) -> f32 {
        let sum: f32 = self.symbol_absence.values().map(|&v| (v as f32 + 1.0).ln()).sum();
        sum.sqrt().min(18.0)
    }

    /// Rebuilds the field for the next cycle using `(generator_seed XOR context.step_seed XOR generation)`.
    pub fn refresh(&mut self, context: &EvolutionContext) {
        let old_chars = self.as_chars();
        let ritual_bits = (context.ritual_entropy.to_bits() as u64).rotate_left(5);
        let next_seed = self.generator_seed
            ^ context.step_seed.rotate_left((self.generation % 31).max(1))
            ^ ((self.generation as u64).wrapping_mul(0x517C_C1B7_2722_0A95))
            ^ ritual_bits;
        let generator = GlyphGenerator::new(next_seed);
        let refreshed = generator.generate_field(self.width, self.height, context);
        self.archaeology_step(&old_chars, &refreshed.cells);
        self.cells = refreshed.cells;
        self.seed = next_seed;
        self.taboo_charge = self
            .taboo_charge
            .saturating_add(count_luminous_shadow_pairs(&self.cells, self.width, self.height));

        if context.dream_phase && !self.cells.is_empty() {
            let mut dr = ChaCha8Rng::seed_from_u64(next_seed.rotate_left(5) ^ 0xDEE5_D00D);
            let alpha = GlyphAlphabet::canonical();
            let sparks = alpha.by_tone(GlyphTone::Spark);
            if !sparks.is_empty() {
                let cap = (2 + (context.ritual_entropy * 4.0) as usize).min(self.cells.len().min(8));
                for _ in 0..cap {
                    let idx = dr.gen_range(0..self.cells.len());
                    let pick = sparks[dr.gen_range(0..sparks.len())].clone();
                    self.cells[idx] = pick;
                }
            }
        }

        self.recompute_fitness();
    }

    fn recompute_fitness(&mut self) {
        let harmonic = self.harmonic_score();
        let glyph_bias = self.glyphic_pattern_index();
        let fossil = self.fossil_echo();
        let area = (self.width * self.height) as f32;
        self.fitness = harmonic * 60.0
            + area * 0.5
            + self.generation as f32 * 0.4
            + glyph_bias * 28.0
            + fossil * 1.85;
    }
}

impl SpiralEntity for GlyphField {
    fn generation(&self) -> u32 {
        self.generation
    }

    fn fitness(&self) -> f32 {
        self.fitness
    }

    fn evolve(&mut self, context: &EvolutionContext) {
        self.generation = self.generation.saturating_add(1);
        self.refresh(context);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn symbolic_density(&self) -> f32 {
        self.glyphic_pattern_index()
    }

    fn memory_depth(&self) -> f32 {
        let gen = (self.generation as f32 / (self.generation as f32 + 36.0)).min(1.0);
        let foss = (self.ghost_breath_mass() as f32 / 5000.0).min(1.0);
        ((gen * 0.55) + (foss * 0.45)).clamp(0.0, 1.0)
    }
}

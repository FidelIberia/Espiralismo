//! 2D variable-size glyph rejilla that participates in Spiralismo evolution cycles.

use std::collections::HashMap;

use crate::core::traits::{EvolutionContext, SpiralEntity};

use super::alphabet::{Glyph, GlyphAlphabet, GlyphTone};
use super::generator::GlyphGenerator;

/// Variable-size 2D grid of glyphs that re-generates itself on every evolution cycle.
///
/// Unlike [`crate::core::lattice::Lattice`], a [`GlyphField`] is **fully procedural**: every
/// `evolve` call deterministically rebuilds the cells from a refreshed seed + context, and
/// fitness is derived from the resulting tone distribution.
#[derive(Clone, Debug)]
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
        (positive - penalty).clamp(0.0, 1.0)
    }

    /// Rebuilds the field for the next cycle using `(generator_seed XOR context.step_seed XOR generation)`.
    pub fn refresh(&mut self, context: &EvolutionContext) {
        let next_seed = self.generator_seed
            ^ context.step_seed.rotate_left((self.generation % 31).max(1))
            ^ ((self.generation as u64).wrapping_mul(0x517C_C1B7_2722_0A95));
        let generator = GlyphGenerator::new(next_seed);
        let refreshed = generator.generate_field(self.width, self.height, context);
        self.cells = refreshed.cells;
        self.seed = next_seed;
        self.recompute_fitness();
    }

    fn recompute_fitness(&mut self) {
        let harmonic = self.harmonic_score();
        let area = (self.width * self.height) as f32;
        self.fitness = harmonic * 60.0 + area * 0.5 + self.generation as f32 * 0.4;
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
}

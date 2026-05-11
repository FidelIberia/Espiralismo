//! Minimal 3×3 symbolic lattice implementing [`crate::core::traits::SpiralEntity`].

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::core::traits::{EvolutionContext, SpiralEntity};
use crate::utils::symbols::*;

/// Canonical lattice width/height.
pub const LATTICE_SIZE: usize = 3;

/// A tiny glyph grid representing a “cellular identity” in Spiralismo symbolism.
///
/// Layout conventions:
/// - Center cell uses [`CORE_GLYPH`].
/// - Other cells are derived from [`crate::core::Seed::as_bits`] patterns (demo mapping).
///
/// `seed_val` is reserved for future deterministic initialization; the current demo ignores it and
/// always reads the default [`crate::core::Seed`] bit pattern.
#[derive(Clone, Debug)]
pub struct Lattice {
    /// 3×3 display grid of Unicode symbols.
    pub grid: [[char; LATTICE_SIZE]; LATTICE_SIZE],
    /// Generation counter.
    pub generation: u32,
    /// Cached fitness score.
    pub fitness: f32,
}

impl Lattice {
    /// Constructs a lattice using default symbolic rules.
    pub fn new(seed_val: u64) -> Self {
        let seed = crate::core::Seed::from_value(seed_val);
        Self::from_seed(&seed)
    }

    /// Constructs a lattice from a [`crate::core::Seed`].
    pub fn from_seed(seed: &crate::core::Seed) -> Self {
        Self::from_bits(&seed.as_bits_width(LATTICE_SIZE * LATTICE_SIZE))
    }

    /// Constructs a lattice from any 0/1 bit slice.
    pub fn from_bits(bits: &[u32]) -> Self {
        let bits = if bits.is_empty() {
            vec![1, 0, 1, 1, 0, 1]
        } else {
            bits.to_vec()
        };
        let mut grid = [[DEFAULT_GLYPH; LATTICE_SIZE]; LATTICE_SIZE];
        grid[LATTICE_SIZE / 2][LATTICE_SIZE / 2] = CORE_GLYPH;

        for i in 0..LATTICE_SIZE {
            for j in 0..LATTICE_SIZE {
                if i == LATTICE_SIZE / 2 && j == LATTICE_SIZE / 2 {
                    continue;
                }
                let idx = (i * LATTICE_SIZE + j) % bits.len().max(1);
                grid[i][j] = if bits[idx] == 1 { '◉' } else { '⟡' };
            }
        }
        let mut lat = Lattice {
            grid,
            generation: 0,
            fitness: 0.0,
        };
        lat.update_fitness();
        lat
    }

    /// Reads a glyph from the lattice.
    pub fn glyph_at(&self, row: usize, col: usize) -> Option<char> {
        self.grid.get(row).and_then(|line| line.get(col)).copied()
    }

    /// Writes a glyph in-bounds. Returns `true` if the update was applied.
    pub fn set_glyph(&mut self, row: usize, col: usize, glyph: char) -> bool {
        let Some(target_row) = self.grid.get_mut(row) else {
            return false;
        };
        let Some(cell) = target_row.get_mut(col) else {
            return false;
        };
        *cell = glyph;
        self.update_fitness();
        true
    }

    /// Returns all lattice symbols row-major.
    pub fn flatten(&self) -> Vec<char> {
        self.grid.iter().flatten().copied().collect()
    }

    /// Human-friendly multiline representation for logs/CLI.
    pub fn as_multiline(&self) -> String {
        self.grid
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Applies deterministic wave mutation driven by context + seed.
    pub fn apply_wave(&mut self, context: &EvolutionContext) {
        let mut rng = ChaCha8Rng::seed_from_u64(context.step_seed ^ self.generation as u64);
        for row in 0..LATTICE_SIZE {
            for col in 0..LATTICE_SIZE {
                if row == LATTICE_SIZE / 2 && col == LATTICE_SIZE / 2 {
                    continue;
                }
                let roll = rng.gen_range(0.0f32..1.0f32);
                if roll < context.mutation_rate {
                    self.grid[row][col] = if self.grid[row][col] == '◉' { '⟡' } else { '◉' };
                } else if roll < context.resonance_pressure {
                    self.grid[row][col] = DEFAULT_GLYPH;
                }
            }
        }
        self.update_fitness();
    }

    fn update_fitness(&mut self) {
        let mut luminous = 0.0f32;
        let mut neutral = 0.0f32;
        for row in 0..LATTICE_SIZE {
            for col in 0..LATTICE_SIZE {
                let glyph = self.grid[row][col];
                if row == LATTICE_SIZE / 2 && col == LATTICE_SIZE / 2 {
                    luminous += 2.5;
                    continue;
                }
                match glyph {
                    '◉' => luminous += 2.0,
                    DEFAULT_GLYPH => neutral += 0.8,
                    '⟡' => neutral += 1.1,
                    _ => neutral += 0.4,
                }
            }
        }
        self.fitness = luminous + neutral + self.generation as f32 * 0.25;
    }
}

impl SpiralEntity for Lattice {
    fn generation(&self) -> u32 {
        self.generation
    }

    fn fitness(&self) -> f32 {
        self.fitness
    }

    fn evolve(&mut self, context: &EvolutionContext) {
        self.generation = self.generation.saturating_add(1);
        self.apply_wave(context);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

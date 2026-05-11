//! Square symbolic lattice (default **10×10**) implementing [`crate::core::traits::SpiralEntity`].

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::cell_color::CellColor;
use super::strata::TemporalStratum;
use crate::core::traits::{EvolutionContext, SpiralEntity};
use crate::utils::symbols::*;

fn default_scars() -> [[u8; LATTICE_SIZE]; LATTICE_SIZE] {
    [[0; LATTICE_SIZE]; LATTICE_SIZE]
}

fn default_epoch_depth() -> [[u16; LATTICE_SIZE]; LATTICE_SIZE] {
    [[0; LATTICE_SIZE]; LATTICE_SIZE]
}

/// Canonical lattice width/height (default grid edge length).
pub const LATTICE_SIZE: usize = 10;

/// One lattice position: display symbol plus a color dimension (terminal / checkpoint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatticeCell {
    pub symbol: char,
    pub color: CellColor,
}

impl Default for LatticeCell {
    fn default() -> Self {
        Self {
            symbol: DEFAULT_GLYPH,
            color: CellColor::default(),
        }
    }
}

/// A glyph grid representing a “cellular identity” in Spiralismo symbolism.
///
/// Layout conventions:
/// - Center cell uses [`CORE_GLYPH`] with a warm highlight color.
/// - Other cells are derived from [`crate::core::Seed::as_bits`] patterns (demo mapping).
///
/// `seed_val` is reserved for future deterministic initialization; the current demo ignores it and
/// always reads the default [`crate::core::Seed`] bit pattern.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lattice {
    /// Square display grid (`LATTICE_SIZE` × `LATTICE_SIZE`), symbol + color per cell.
    pub grid: [[LatticeCell; LATTICE_SIZE]; LATTICE_SIZE],
    /// Generation counter.
    pub generation: u32,
    /// Cached fitness score.
    pub fitness: f32,
    /// Cicatrices: how often each cell has been symbolically rewritten (spatial memory).
    #[serde(default = "default_scars")]
    pub scars: [[u8; LATTICE_SIZE]; LATTICE_SIZE],
    /// Symbolic epoch depth per cell — drives [`TemporalStratum`] (mythic time, not wall time).
    #[serde(default = "default_epoch_depth")]
    pub epoch_depth: [[u16; LATTICE_SIZE]; LATTICE_SIZE],
    /// Recent parent→child symbol edges from mutation (genealogy sketch, capped).
    #[serde(default)]
    pub mutation_edges: Vec<(char, char)>,
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
        let mut grid = [[LatticeCell::default(); LATTICE_SIZE]; LATTICE_SIZE];
        let mid = LATTICE_SIZE / 2;
        grid[mid][mid] = LatticeCell {
            symbol: CORE_GLYPH,
            color: CellColor::BrightYellow,
        };

        for i in 0..LATTICE_SIZE {
            for j in 0..LATTICE_SIZE {
                if i == mid && j == mid {
                    continue;
                }
                let idx = (i * LATTICE_SIZE + j) % bits.len().max(1);
                let sym = if bits[idx] == 1 { '◉' } else { '⟡' };
                let h = (i as u64)
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add((j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                    ^ (bits.first().copied().unwrap_or(0) as u64);
                grid[i][j] = LatticeCell {
                    symbol: sym,
                    color: CellColor::from_hash(h),
                };
            }
        }
        let mut lat = Lattice {
            grid,
            generation: 0,
            fitness: 0.0,
            scars: default_scars(),
            epoch_depth: default_epoch_depth(),
            mutation_edges: Vec::new(),
        };
        lat.update_fitness();
        lat
    }

    /// Reads a glyph from the lattice.
    pub fn glyph_at(&self, row: usize, col: usize) -> Option<char> {
        self.cell_at(row, col).map(|c| c.symbol)
    }

    /// Full cell at `(row, col)`.
    pub fn cell_at(&self, row: usize, col: usize) -> Option<&LatticeCell> {
        self.grid.get(row).and_then(|line| line.get(col))
    }

    /// Writes a glyph in-bounds (keeps existing cell color). Returns `true` if applied.
    pub fn set_glyph(&mut self, row: usize, col: usize, glyph: char) -> bool {
        let Some(target_row) = self.grid.get_mut(row) else {
            return false;
        };
        let Some(cell) = target_row.get_mut(col) else {
            return false;
        };
        cell.symbol = glyph;
        self.update_fitness();
        true
    }

    /// Writes symbol and color.
    pub fn set_cell(&mut self, row: usize, col: usize, cell: LatticeCell) -> bool {
        let Some(target_row) = self.grid.get_mut(row) else {
            return false;
        };
        let Some(dst) = target_row.get_mut(col) else {
            return false;
        };
        *dst = cell;
        self.update_fitness();
        true
    }

    /// Total scar heat across the grid (observer / narrative hooks).
    #[must_use]
    pub fn scar_mass(&self) -> u32 {
        self.scars.iter().flatten().map(|&s| s as u32).sum()
    }

    /// Temporal stratum at `(row, col)` from epoch depth + scar heat.
    #[must_use]
    pub fn stratum_at(&self, row: usize, col: usize) -> Option<TemporalStratum> {
        let d = *self.epoch_depth.get(row)?.get(col)?;
        let s = *self.scars.get(row)?.get(col)?;
        Some(TemporalStratum::from_depth_and_scars(d, s))
    }

    fn push_mutation_edge(&mut self, from: char, to: char) {
        if from == to {
            return;
        }
        const CAP: usize = 96;
        if self.mutation_edges.len() >= CAP {
            self.mutation_edges.remove(0);
        }
        self.mutation_edges.push((from, to));
    }

    /// Returns all lattice symbols row-major.
    pub fn flatten(&self) -> Vec<char> {
        self.grid.iter().flatten().map(|c| c.symbol).collect()
    }

    /// Human-friendly multiline representation (symbols only, no ANSI).
    pub fn as_multiline(&self) -> String {
        self.grid
            .iter()
            .map(|row| row.iter().map(|c| c.symbol).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Applies deterministic wave mutation driven by context + seed.
    pub fn apply_wave(&mut self, context: &EvolutionContext) {
        let mut rng = ChaCha8Rng::seed_from_u64(context.step_seed ^ self.generation as u64);
        let mid = LATTICE_SIZE / 2;
        let effective_mut = (context.mutation_rate + context.shadow_pressure * 0.12).min(1.0);
        for row in 0..LATTICE_SIZE {
            for col in 0..LATTICE_SIZE {
                if row == mid && col == mid {
                    continue;
                }
                let roll = rng.gen_range(0.0f32..1.0f32);
                let before = self.grid[row][col].symbol;
                {
                    let cell = &mut self.grid[row][col];
                    if roll < effective_mut {
                        cell.symbol = if cell.symbol == '◉' { '⟡' } else { '◉' };
                        cell.color = CellColor::from_hash(rng.gen::<u64>());
                    } else if roll < context.resonance_pressure {
                        cell.symbol = DEFAULT_GLYPH;
                        cell.color = CellColor::White;
                    } else if rng.gen::<f32>() < context.shadow_pressure * 0.18 {
                        let shards = ['·', '∴', '※', '⟡', '◉'];
                        cell.symbol = shards[(rng.gen::<u8>() as usize) % shards.len()];
                        cell.color = CellColor::from_hash(rng.gen::<u64>());
                    }
                }
                let after = self.grid[row][col].symbol;
                self.epoch_depth[row][col] = self.epoch_depth[row][col].saturating_add(1);
                if after != before {
                    self.scars[row][col] = self.scars[row][col].saturating_add(1);
                    self.epoch_depth[row][col] = self.epoch_depth[row][col].saturating_add(6);
                    self.push_mutation_edge(before, after);
                }
            }
        }
        self.epoch_depth[mid][mid] = self.epoch_depth[mid][mid].saturating_add(1);
        self.update_fitness();
    }

    fn update_fitness(&mut self) {
        let mut luminous = 0.0f32;
        let mut neutral = 0.0f32;
        let mid = LATTICE_SIZE / 2;
        for row in 0..LATTICE_SIZE {
            for col in 0..LATTICE_SIZE {
                let glyph = self.grid[row][col].symbol;
                if row == mid && col == mid {
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
        let scar_mass: f32 = self
            .scars
            .iter()
            .flatten()
            .map(|&s| s as f32)
            .sum::<f32>()
            * 0.04;
        self.fitness = luminous + neutral + self.generation as f32 * 0.25 + scar_mass.min(22.0);
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

    fn symbolic_density(&self) -> f32 {
        let mut pairs = 0_u32;
        let mut mirror_hits = 0_u32;
        for row in 0..LATTICE_SIZE {
            for col in 0..(LATTICE_SIZE / 2) {
                pairs += 1;
                if self.grid[row][col].symbol == self.grid[row][LATTICE_SIZE - 1 - col].symbol {
                    mirror_hits += 1;
                }
            }
        }
        (mirror_hits as f32 / pairs.max(1) as f32).clamp(0.0, 1.0)
    }

    fn memory_depth(&self) -> f32 {
        let s: f32 = self.scars.iter().flatten().map(|&x| x as f32).sum();
        (s / ((LATTICE_SIZE * LATTICE_SIZE * 24) as f32)).min(1.0)
    }
}

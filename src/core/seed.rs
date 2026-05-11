//! Canonical symbolic seed used across the framework.

/// Binary string identity for the default [`Seed`]. Interpreted as radix-2 [`u64`] in [`Seed::new`].
const SEED_HASH: &str = "101101";

/// Opaque numeric seed wrapper used as a conceptual “DNA” / anchor for symbolic state.
///
/// In the current implementation, the default value is derived from [`SEED_HASH`]. Future versions
/// may thread user-provided entropy through here for reproducible simulations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Seed(u64);

impl Seed {
    /// Builds the default seed from [`SEED_HASH`].
    pub fn new() -> Self {
        let val = u64::from_str_radix(SEED_HASH, 2).unwrap_or(45);
        Self(val)
    }

    /// Builds a seed directly from a raw integer.
    pub fn from_value(value: u64) -> Self {
        Self(value)
    }

    /// Parses a binary hash (`0`/`1` only) into a new seed.
    pub fn from_binary_hash(hash: &str) -> Option<Self> {
        u64::from_str_radix(hash, 2).ok().map(Self)
    }

    /// Raw numeric value (currently stable for the default construction path).
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Human-stable string form of the seed bits (not necessarily equal to `value()` formatting).
    pub fn hash_str(&self) -> &'static str {
        SEED_HASH
    }

    /// Expands the seed into `0/1` digits with a default width derived from [`SEED_HASH`].
    pub fn as_bits(&self) -> Vec<u32> {
        self.as_bits_width(SEED_HASH.len())
    }

    /// Expands the seed into `0/1` digits with a caller-provided width.
    pub fn as_bits_width(&self, width: usize) -> Vec<u32> {
        let width = width.clamp(1, u64::BITS as usize);
        (0..width)
            .rev()
            .map(|offset| ((self.0 >> offset) & 1) as u32)
            .collect()
    }

    /// Rotates bits to produce a deterministic sibling seed.
    pub fn rotate_left(self, amount: u32) -> Self {
        Self(self.0.rotate_left(amount))
    }

    /// Lightweight deterministic mixing function for seed composition.
    pub fn mix(self, other: Seed) -> Self {
        let mixed = self
            .0
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .rotate_left(13)
            ^ other.0.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
        Self(mixed)
    }
}

impl Default for Seed {
    fn default() -> Self {
        Self::new()
    }
}

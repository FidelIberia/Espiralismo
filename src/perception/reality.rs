//! Reality perceptors — landscapes, memory, files, virtual/physical hosts.
//!
//! These implement [`RealityPerceiver`] and merge into the **reality lane** only. They must not
//! call [`super::astronomical::AstronomicalPerceiver`] or alter sky-derived policy.

use super::traits::{PerceptionFrame, PerceptionOffer};

/// Which facet of world the perceptor listens to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RealityKind {
    /// Visual field / framebuffer / landscape entropy (host-supplied or rendered).
    VisualLandscape,
    /// Process or machine memory pressure (RSS, heap, host snapshot).
    PhysicalMemory,
    /// Files, directories, artifact trees on disk.
    Filesystem,
    /// Simulated, game, or model worlds.
    Virtual,
    /// Sensors tied to physical apparatus (generic bucket).
    Physical,
}

impl RealityKind {
    /// Stable token for logs and channels.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::VisualLandscape => "visual_landscape",
            Self::PhysicalMemory => "physical_memory",
            Self::Filesystem => "filesystem",
            Self::Virtual => "virtual",
            Self::Physical => "physical",
        }
    }
}

/// Perceives a slice of virtual or physical reality and returns a **reality** [`PerceptionOffer`].
///
/// Influence is applied via [`super::field::PerceptionField::apply_reality_offer`], which does not
/// overwrite sky-quiet fields (mutation, resonance, drift, ritual, shadow, dream).
pub trait RealityPerceiver: Send + Sync {
    fn id(&self) -> &'static str;
    fn reality_kind(&self) -> RealityKind;
    fn perceive(&self, frame: &PerceptionFrame) -> PerceptionOffer;
}

/// Optional host snapshot (UI, OS bridge, Galaxy render) attached to a [`PerceptionFrame`].
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HostRealitySnapshot {
    /// Normalized visual complexity / landscape entropy in `[0, 1]`.
    pub visual_landscape: f32,
    /// Resident set size in bytes, when the host reports it.
    pub process_rss_bytes: Option<u64>,
    /// Entries in the working directory (shallow listing).
    pub cwd_entry_count: Option<usize>,
    /// Entries under artifact path when provided.
    pub artifact_entry_count: Option<usize>,
}

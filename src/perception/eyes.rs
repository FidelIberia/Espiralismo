//! Perception **eyes** — expose perceptors so the spiral can receive or take the environment.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::astrology::Sky;

use super::reality::HostRealitySnapshot;
use super::spiralismo_press::SpiralismoPress;
use super::traits::ExternalListening;

/// How an eye relates to the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EyeRole {
    /// Physical sky — astronomical lane (isolated from reality merge).
    Astronomical,
    /// Hand on the veil — integrated [`SpiralismoPress`].
    Hand,
    /// Files, RAM, landscape, virtual/physical facets.
    Reality,
}

/// One exposed perceptor the host may address by [`PerceptionEyeDescriptor::id`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerceptionEyeDescriptor {
    pub id: String,
    pub role: EyeRole,
    /// `RealityKind::token()`, `"astronomy"`, or `"hand"`.
    pub facet: String,
    /// Host may push data (`offer_environment`, `offer_to_eye`).
    pub can_receive: bool,
    /// Spiral may probe when able (`take_from_environment`).
    pub can_take: bool,
}

/// Catalog of open eyes on this runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionEyes {
    pub astronomical: PerceptionEyeDescriptor,
    pub hand: PerceptionEyeDescriptor,
    pub reality: Vec<PerceptionEyeDescriptor>,
}

/// What the host (or bridge) offers to a specific eye.
#[derive(Debug, Clone)]
pub enum EnvironmentOffering {
    /// Full host snapshot for reality eyes (landscape, memory fields, …).
    HostSnapshot(HostRealitySnapshot),
    /// Hand / mouse / veil state.
    Hand(SpiralismoPress),
    /// One-shot listening line.
    Listening(ExternalListening),
}

/// Options for active environment capture (`take_from_environment`).
#[derive(Debug, Clone)]
pub struct EnvironmentTakeOptions {
    /// Probe shallow filesystem (cwd + `./artifacts`).
    pub probe_filesystem: bool,
    /// Capture present sky via the astronomical perceiver.
    pub capture_sky: bool,
    /// When true, store results on the field for subsequent frames.
    pub commit_to_field: bool,
}

impl Default for EnvironmentTakeOptions {
    fn default() -> Self {
        Self {
            probe_filesystem: true,
            capture_sky: true,
            commit_to_field: true,
        }
    }
}

/// Report of what the spiral **took** when it could.
#[derive(Debug, Clone)]
pub struct EnvironmentTakeReport {
    pub taken_at: DateTime<Utc>,
    pub host_reality: HostRealitySnapshot,
    pub sky: Option<Sky>,
    pub eyes_engaged: Vec<String>,
}

impl EnvironmentTakeReport {
    #[must_use]
    pub fn any_taken(&self) -> bool {
        !self.eyes_engaged.is_empty()
    }
}

/// Result of routing an [`EnvironmentOffering`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfferRouting {
    Accepted,
    UnknownEye,
    EyeCannotReceive,
}

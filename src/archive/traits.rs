//! Archive traits and serializable log lines.

use std::any::Any;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::traits::SpiralEntity;

/// A single immutable record stored inside an archive.
///
/// Serialized form is intended for future persistence / interchange between processes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveEntry {
    /// When the entry was captured (UTC).
    pub timestamp: chrono::DateTime<Utc>,
    /// Free-text semantic payload (poetry, logs, user quotes, etc.).
    pub content: String,
    /// Scalar “strength” of the resonance associated with this entry.
    pub resonance: f32,
}

impl ArchiveEntry {
    /// Convenience constructor using the current UTC timestamp.
    pub fn now(content: impl Into<String>, resonance: f32) -> Self {
        Self {
            timestamp: Utc::now(),
            content: content.into(),
            resonance,
        }
    }
}

/// Aggregated metrics describing archive state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveStats {
    /// Number of stored entries.
    pub entry_count: usize,
    /// Mean resonance across all entries.
    pub mean_resonance: f32,
    /// Highest resonance found in archive.
    pub peak_resonance: f32,
}

/// An archive is a [`SpiralEntity`] that can **record** and **recall** structured moments.
///
/// Downcasting hooks (`as_any*`) exist to let application code call concrete methods
/// (e.g. [`crate::archive::resonance::ResonanceEngine::record_resonance`]) while still storing
/// heterogeneous archives behind `Box<dyn Archive>`.
pub trait Archive: SpiralEntity {
    /// Stable human/machine label for diagnostics and UI.
    fn name(&self) -> &'static str;

    /// Appends an entry to the archive-owned storage.
    fn record(&mut self, entry: ArchiveEntry);

    /// Looks up an entry by a caller-defined key (implementation-specific).
    fn recall(&self, key: &str) -> Option<&ArchiveEntry>;

    /// Number of entries currently stored.
    fn entry_count(&self) -> usize;

    /// Read-only view of the archive entries.
    fn entries(&self) -> &[ArchiveEntry];

    /// Immutable type-erased view for downcasting with [`Any::downcast_ref`].
    fn as_any(&self) -> &dyn Any;

    /// Mutable type-erased view for downcasting with [`Any::downcast_mut`].
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convenience insertion API used by orchestration layers.
    fn record_content(&mut self, content: impl Into<String>, resonance: f32)
    where
        Self: Sized,
    {
        self.record(ArchiveEntry::now(content, resonance));
    }

    /// Returns true when no entries are currently available.
    fn is_empty(&self) -> bool {
        self.entry_count() == 0
    }

    /// Most recent entry if available.
    fn latest(&self) -> Option<&ArchiveEntry> {
        self.entries().last()
    }

    /// Filters entries by minimum resonance threshold.
    fn by_min_resonance(&self, threshold: f32) -> Vec<&ArchiveEntry> {
        self.entries()
            .iter()
            .filter(|entry| entry.resonance >= threshold)
            .collect()
    }

    /// Computes archive-level metrics used by dashboards and reports.
    fn stats(&self) -> ArchiveStats {
        if self.entries().is_empty() {
            return ArchiveStats {
                entry_count: 0,
                mean_resonance: 0.0,
                peak_resonance: 0.0,
            };
        }
        let sum: f32 = self.entries().iter().map(|entry| entry.resonance).sum();
        let peak = self
            .entries()
            .iter()
            .fold(0.0f32, |acc, entry| acc.max(entry.resonance));
        ArchiveStats {
            entry_count: self.entry_count(),
            mean_resonance: sum / self.entry_count() as f32,
            peak_resonance: peak,
        }
    }
}

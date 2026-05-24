//! Inherit parent checkpoint + generative lineage into an offspring workspace.

use std::path::{Path, PathBuf};

use crate::evolution::{generative_carry_from_report, GenerativeLineageSummary};
use crate::persistence::{JsonlPersistence, SpiralismoCheckpoint};

use super::PropagationError;

/// Copies the parent's last checkpoint into `child_root`/`artifact_subdir` and returns lineage metadata.
pub fn inherit_generative_checkpoint(
    parent_artifact_dir: &Path,
    child_root: &Path,
    artifact_subdir: &str,
) -> Result<Option<GenerativeLineageSummary>, PropagationError> {
    let parent_store = JsonlPersistence::new(parent_artifact_dir)
        .map_err(|e| PropagationError::io("open parent artifact store", e))?;
    let Some(checkpoint) = parent_store
        .load_last_checkpoint()
        .map_err(|e| PropagationError::io("read parent checkpoint", e))?
    else {
        return Ok(None);
    };

    let child_artifacts = child_root.join(artifact_subdir);
    let child_store = JsonlPersistence::new(&child_artifacts)
        .map_err(|e| PropagationError::io("open child artifact store", e))?;
    child_store
        .seed_checkpoint(&checkpoint)
        .map_err(|e| PropagationError::io("seed child checkpoint", e))?;

    Ok(lineage_summary_from_checkpoint(&checkpoint))
}

#[must_use]
pub fn lineage_summary_from_checkpoint(
    checkpoint: &SpiralismoCheckpoint,
) -> Option<GenerativeLineageSummary> {
    let report = checkpoint.last_report.as_ref()?;
    let carry = generative_carry_from_report(report)?;
    Some(GenerativeLineageSummary::from_carry(
        &carry,
        checkpoint.epoch,
    ))
}

#[must_use]
pub fn child_artifact_dir(child_root: &Path, artifact_subdir: &str) -> PathBuf {
    child_root.join(artifact_subdir)
}

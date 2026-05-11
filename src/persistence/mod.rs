//! JSONL persistence helpers for evolution reports and runtime snapshots.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::archive::ArchiveStats;
use crate::{EvolutionReport, Spiralismo, SpiralismoSnapshot};

/// Persisted runtime envelope combining snapshot and archive metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStateRecord {
    /// UTC timestamp when the state was captured.
    pub captured_at: chrono::DateTime<Utc>,
    /// Structural runtime state.
    pub snapshot: SpiralismoSnapshot,
    /// Archive metrics at capture time.
    pub archive_stats: Vec<(String, ArchiveStats)>,
}

impl RuntimeStateRecord {
    /// Captures a serializable runtime view from a [`Spiralismo`] instance.
    pub fn from_spiralismo(spiral: &Spiralismo) -> Self {
        Self {
            captured_at: Utc::now(),
            snapshot: spiral.snapshot(),
            archive_stats: spiral
                .archive_stats()
                .into_iter()
                .map(|(name, stats)| (name.to_string(), stats))
                .collect(),
        }
    }
}

/// Filesystem-backed JSONL store for Spiralismo artifacts.
#[derive(Debug, Clone)]
pub struct JsonlPersistence {
    root: PathBuf,
    reports_path: PathBuf,
    snapshots_path: PathBuf,
    runtime_state_path: PathBuf,
}

impl JsonlPersistence {
    /// Creates a persistence store rooted at `root`.
    ///
    /// The directory is created if it does not exist.
    pub fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self {
            reports_path: root.join("evolution_reports.jsonl"),
            snapshots_path: root.join("spiral_snapshots.jsonl"),
            runtime_state_path: root.join("runtime_state.jsonl"),
            root,
        })
    }

    /// Root directory used by this store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Appends an [`EvolutionReport`] JSON line.
    pub fn append_report(&self, report: &EvolutionReport) -> io::Result<()> {
        append_json_line(&self.reports_path, report)
    }

    /// Appends a [`SpiralismoSnapshot`] JSON line.
    pub fn append_snapshot(&self, snapshot: &SpiralismoSnapshot) -> io::Result<()> {
        append_json_line(&self.snapshots_path, snapshot)
    }

    /// Appends a full [`RuntimeStateRecord`] JSON line.
    pub fn append_runtime_state_record(&self, state: &RuntimeStateRecord) -> io::Result<()> {
        append_json_line(&self.runtime_state_path, state)
    }

    /// Captures and appends runtime state for a [`Spiralismo`] instance.
    pub fn append_runtime_state(&self, spiral: &Spiralismo) -> io::Result<()> {
        self.append_runtime_state_record(&RuntimeStateRecord::from_spiralismo(spiral))
    }

    /// Reads all report entries from disk.
    pub fn load_reports(&self) -> io::Result<Vec<EvolutionReport>> {
        read_json_lines(&self.reports_path)
    }

    /// Reads all snapshot entries from disk.
    pub fn load_snapshots(&self) -> io::Result<Vec<SpiralismoSnapshot>> {
        read_json_lines(&self.snapshots_path)
    }

    /// Reads all runtime state records from disk.
    pub fn load_runtime_states(&self) -> io::Result<Vec<RuntimeStateRecord>> {
        read_json_lines(&self.runtime_state_path)
    }
}

fn append_json_line<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    file.write_all(b"\n")?;
    Ok(())
}

fn read_json_lines<T: DeserializeOwned>(path: &Path) -> io::Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let reader = BufReader::new(File::open(path)?);
    let mut records = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let parsed = serde_json::from_str::<T>(&line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid JSONL record at line {} in {}: {error}",
                    idx + 1,
                    path.display()
                ),
            )
        })?;
        records.push(parsed);
    }

    Ok(records)
}

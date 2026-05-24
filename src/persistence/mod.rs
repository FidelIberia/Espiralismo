//! Single-file JSONL checkpoints: each line is a full [`SpiralismoCheckpoint`] for resume.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::archive::traits::Archive;
use crate::archive::{CartographyArchive, MercyArchive, MemoryArchive, ResonanceEngine};
use crate::core::traits::SpiralEntity;
use crate::core::{Lattice, Seed};
use crate::evolution::EvolutionReport;
use crate::genome::{Genome, GenomeFile};
use crate::glyphs::GlyphField;
use crate::perception::{PerceptionField, SoulState};
use crate::spiralismo::Spiralismo;

/// Schema version stored in [`SpiralismoCheckpoint::schema_version`].
pub const SPIRALISMO_CHECKPOINT_SCHEMA: u32 = 2;

/// Default filename inside the artifact directory.
pub const CHECKPOINT_JSONL: &str = "checkpoint.jsonl";

/// Failure while building or applying a checkpoint.
#[derive(Debug, Clone)]
pub enum CheckpointError {
    /// Checkpoint file uses an unsupported `schema_version`.
    UnsupportedSchema(u32),
    /// Expected built-in archive is missing from the runtime.
    MissingArchive(&'static str),
    /// An archive could not be matched to a known concrete type.
    UnknownArchive(String),
    /// An active entity is not a [`Lattice`] nor a [`GlyphField`].
    UnknownActiveEntity,
    /// Built-in archive set has unexpected size or extra unknown archives.
    ArchiveLayoutInvalid { expected: usize, found: usize },
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckpointError::UnsupportedSchema(v) => {
                write!(f, "unsupported checkpoint schema version: {v}")
            }
            CheckpointError::MissingArchive(name) => {
                write!(f, "missing expected archive: {name}")
            }
            CheckpointError::UnknownArchive(name) => {
                write!(f, "unknown archive type for name: {name}")
            }
            CheckpointError::UnknownActiveEntity => {
                write!(f, "active entity is not lattice nor glyph field")
            }
            CheckpointError::ArchiveLayoutInvalid { expected, found } => {
                write!(
                    f,
                    "invalid archive layout: expected {expected} built-ins, found {found}"
                )
            }
        }
    }
}

impl std::error::Error for CheckpointError {}

/// One full serializable snapshot of [`Spiralismo`] (archives, active entities, epoch, seed, last report).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiralismoCheckpoint {
    /// Increment when the checkpoint format changes.
    pub schema_version: u32,
    /// UTC time when this line was written.
    pub saved_at: chrono::DateTime<Utc>,
    /// Raw [`Seed::value`].
    pub seed_value: u64,
    /// Orchestrator epoch after the last completed policy run (same counter as in-memory).
    pub epoch: u64,
    /// Last [`EvolutionReport`] from [`Spiralismo::evolve_with_policy`], if any.
    pub last_report: Option<EvolutionReport>,
    /// The four built-in archives, each tagged by `kind` in JSON.
    pub archives: Vec<CheckpointArchive>,
    /// Active [`SpiralEntity`] stack (lattice and/or glyph field), in registration order.
    pub active_entities: Vec<CheckpointActiveEntity>,
    /// Optional fragment captured at save time (partial lore; absent on older checkpoints).
    #[serde(default)]
    pub whisper: Option<String>,
    /// Soul (`alma`) at save time — absent on older checkpoints defaults to empty soul.
    #[serde(default)]
    pub soul: SoulState,
    /// Full runtime genome after the last evolution assimilated the generative line.
    #[serde(default = "default_checkpoint_genome")]
    pub genome: GenomeFile,
}

fn default_checkpoint_genome() -> GenomeFile {
    Genome::embedded().file
}

/// Serializable wrapper for a built-in archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "body", rename_all = "snake_case")]
pub enum CheckpointArchive {
    MercyField(MercyArchive),
    LivingMemory(MemoryArchive),
    LivingCartography(CartographyArchive),
    ResonanceEngine(ResonanceEngine),
}

/// Serializable active entity (non-archive evolution participant).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "body", rename_all = "snake_case")]
pub enum CheckpointActiveEntity {
    Lattice(Lattice),
    GlyphField(GlyphField),
}

impl SpiralismoCheckpoint {
    /// Captures the full runtime state and the active [`Genome`].
    pub fn capture(spiral: &Spiralismo, genome: &Genome) -> Result<Self, CheckpointError> {
        Ok(Self {
            schema_version: SPIRALISMO_CHECKPOINT_SCHEMA,
            saved_at: Utc::now(),
            seed_value: spiral.seed.value(),
            epoch: spiral.epoch,
            last_report: spiral.last_report.clone(),
            archives: capture_archives(spiral)?,
            active_entities: capture_active_entities(spiral)?,
            whisper: Some(spiral.whisper_now()),
            soul: spiral.perception.soul().clone(),
            genome: genome.file.clone(),
        })
    }

    /// Genome stored in this checkpoint (embedded defaults for schema v1 lines).
    #[must_use]
    pub fn resolved_genome(&self) -> Genome {
        Genome::from_file(self.genome.clone())
    }

    /// Rebuilds a [`Spiralismo`] from this checkpoint.
    pub fn into_spiralismo(self) -> Result<Spiralismo, CheckpointError> {
        if self.schema_version != SPIRALISMO_CHECKPOINT_SCHEMA && self.schema_version != 1 {
            return Err(CheckpointError::UnsupportedSchema(self.schema_version));
        }

        let archives: Vec<Box<dyn Archive>> = self
            .archives
            .into_iter()
            .map(|a| match a {
                CheckpointArchive::MercyField(m) => Box::new(m) as Box<dyn Archive>,
                CheckpointArchive::LivingMemory(m) => Box::new(m),
                CheckpointArchive::LivingCartography(m) => Box::new(m),
                CheckpointArchive::ResonanceEngine(m) => Box::new(m),
            })
            .collect();

        let mut active_lattices: Vec<Box<dyn SpiralEntity>> = Vec::with_capacity(self.active_entities.len());
        for entity in self.active_entities {
            match entity {
                CheckpointActiveEntity::Lattice(l) => active_lattices.push(Box::new(l)),
                CheckpointActiveEntity::GlyphField(g) => active_lattices.push(Box::new(g)),
            }
        }

        let mut perception = PerceptionField::new();
        *perception.soul_mut() = self.soul;

        Ok(Spiralismo::from_runtime_parts(
            Seed::from_value(self.seed_value),
            self.epoch,
            self.last_report,
            archives,
            active_lattices,
            perception,
        ))
    }
}

fn capture_archives(spiral: &Spiralismo) -> Result<Vec<CheckpointArchive>, CheckpointError> {
    const ORDER: &[&str] = &[
        "Mercy Field",
        "Living Memory",
        "Living Cartography",
        "ResonanceEngine",
    ];

    if spiral.archives.len() != ORDER.len() {
        return Err(CheckpointError::ArchiveLayoutInvalid {
            expected: ORDER.len(),
            found: spiral.archives.len(),
        });
    }

    let mut by_name: HashMap<&str, CheckpointArchive> = HashMap::new();
    for archive in &spiral.archives {
        by_name.insert(archive.name(), archive_to_checkpoint(archive.as_ref())?);
    }

    let mut ordered = Vec::with_capacity(ORDER.len());
    for expected in ORDER {
        let cp = by_name
            .remove(expected)
            .ok_or(CheckpointError::MissingArchive(expected))?;
        ordered.push(cp);
    }

    if !by_name.is_empty() {
        return Err(CheckpointError::ArchiveLayoutInvalid {
            expected: ORDER.len(),
            found: ORDER.len() + by_name.len(),
        });
    }

    Ok(ordered)
}

fn archive_to_checkpoint(archive: &dyn Archive) -> Result<CheckpointArchive, CheckpointError> {
    if let Some(m) = Archive::as_any(archive).downcast_ref::<MercyArchive>() {
        return Ok(CheckpointArchive::MercyField(m.clone()));
    }
    if let Some(m) = Archive::as_any(archive).downcast_ref::<MemoryArchive>() {
        return Ok(CheckpointArchive::LivingMemory(m.clone()));
    }
    if let Some(m) = Archive::as_any(archive).downcast_ref::<CartographyArchive>() {
        return Ok(CheckpointArchive::LivingCartography(m.clone()));
    }
    if let Some(m) = Archive::as_any(archive).downcast_ref::<ResonanceEngine>() {
        return Ok(CheckpointArchive::ResonanceEngine(m.clone()));
    }
    Err(CheckpointError::UnknownArchive(archive.name().to_string()))
}

fn capture_active_entities(spiral: &Spiralismo) -> Result<Vec<CheckpointActiveEntity>, CheckpointError> {
    let mut out = Vec::with_capacity(spiral.active_lattices.len());
    for entity in &spiral.active_lattices {
        if let Some(l) = entity.as_any().downcast_ref::<Lattice>() {
            out.push(CheckpointActiveEntity::Lattice(l.clone()));
        } else if let Some(g) = entity.as_any().downcast_ref::<GlyphField>() {
            out.push(CheckpointActiveEntity::GlyphField(g.clone()));
        } else {
            return Err(CheckpointError::UnknownActiveEntity);
        }
    }
    Ok(out)
}

/// Filesystem-backed JSONL store: one append-only `checkpoint.jsonl`.
#[derive(Debug, Clone)]
pub struct JsonlPersistence {
    root: PathBuf,
    checkpoint_path: PathBuf,
}

impl JsonlPersistence {
    /// Creates a persistence store rooted at `root` (directory is created if missing).
    pub fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self {
            checkpoint_path: root.join(CHECKPOINT_JSONL),
            root,
        })
    }

    /// Root directory used by this store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path to the checkpoint JSONL file.
    pub fn checkpoint_path(&self) -> &Path {
        &self.checkpoint_path
    }

    /// Appends one full checkpoint line for the current runtime and genome.
    pub fn append_checkpoint(&self, spiral: &Spiralismo, genome: &Genome) -> io::Result<()> {
        let checkpoint = SpiralismoCheckpoint::capture(spiral, genome).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("could not build checkpoint: {error}"),
            )
        })?;
        append_json_line(&self.checkpoint_path, &checkpoint)
    }

    /// Reads the last successfully parsed checkpoint line, if any.
    pub fn load_last_checkpoint(&self) -> io::Result<Option<SpiralismoCheckpoint>> {
        if !self.checkpoint_path.exists() {
            return Ok(None);
        }

        let reader = BufReader::new(File::open(&self.checkpoint_path)?);
        let mut last_ok: Option<SpiralismoCheckpoint> = None;

        for (idx, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<SpiralismoCheckpoint>(&line) {
                Ok(cp) => last_ok = Some(cp),
                Err(error) => {
                    eprintln!(
                        "warning: skipping invalid checkpoint line {} in {}: {error}",
                        idx + 1,
                        self.checkpoint_path.display()
                    );
                }
            }
        }

        Ok(last_ok)
    }
}

fn append_json_line<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, value).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    file.write_all(b"\n")?;
    Ok(())
}

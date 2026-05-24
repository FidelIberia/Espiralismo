//! Genetic identity and propagation policy for offspring workspaces.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Describes one replication attempt (parent → child workspace → binary → process).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationPolicy {
    /// Crate root to copy (usually the running project's manifest directory).
    pub source_root: PathBuf,
    /// Directory where child workspaces are materialized.
    pub offspring_root: PathBuf,
    /// Parent generation counter (stored in child metadata).
    pub generation: u32,
    /// Parent seed mixed into the child's genetics.
    pub parent_seed: u64,
    /// Per-locus mutation probability in `0.0..=1.0`.
    pub mutation_rate: f32,
    /// `cargo build --release` when true.
    pub release: bool,
    /// Launch the compiled binary in a new process after a successful build.
    pub spawn_after_build: bool,
    /// Extra argv passed to the child binary (after `--propagated-child`).
    pub child_args: Vec<String>,
    /// When true, copy and mutate but skip compile/spawn.
    pub dry_run: bool,
    /// Parent artifact directory containing `checkpoint.jsonl` to seed the child.
    pub parent_artifact_dir: Option<PathBuf>,
    /// Copy the parent's last checkpoint into the offspring workspace before spawn.
    pub inherit_checkpoint: bool,
    /// Subdirectory under the child root used for checkpoints (usually `artifacts`).
    pub child_artifact_subdir: String,
}

impl PropagationPolicy {
    #[must_use]
    pub fn new(source_root: impl Into<PathBuf>, parent_seed: u64) -> Self {
        let source_root = source_root.into();
        let offspring_root = source_root.join("propagation").join("offspring");
        let genome = crate::genome::Genome::load_from_root(&source_root);
        Self {
            source_root,
            offspring_root,
            generation: 0,
            parent_seed,
            mutation_rate: genome.propagation_mutation_rate(),
            release: genome.propagation_build_release(),
            spawn_after_build: genome.propagation_spawn_offspring(),
            child_args: Vec::new(),
            dry_run: false,
            parent_artifact_dir: None,
            inherit_checkpoint: genome.propagation_inherit_checkpoint(),
            child_artifact_subdir: genome.demo().artifact_dir.clone(),
        }
    }

    pub fn with_parent_artifact_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.parent_artifact_dir = Some(path.into());
        self
    }

    pub fn without_checkpoint_inherit(mut self) -> Self {
        self.inherit_checkpoint = false;
        self
    }

    pub fn with_generation(mut self, generation: u32) -> Self {
        self.generation = generation;
        self
    }

    pub fn with_mutation_rate(mut self, rate: f32) -> Self {
        self.mutation_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn with_offspring_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.offspring_root = path.into();
        self
    }

    pub fn with_release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }

    pub fn without_spawn(mut self) -> Self {
        self.spawn_after_build = false;
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

/// Outcome of [`super::propagate`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationReport {
    pub child_root: PathBuf,
    pub child_seed: u64,
    pub generation: u32,
    pub mutations_applied: usize,
    pub compile_ok: bool,
    pub binary_path: Option<PathBuf>,
    pub spawn_pid: Option<u32>,
    pub toolchain: ToolchainInfo,
    pub log: Vec<String>,
}

/// Host Rust toolchain discovered for the child build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainInfo {
    pub cargo: PathBuf,
    pub rustc: PathBuf,
    pub version: String,
}

/// Relative path (from crate root) — the runtime genome TOML (mutable by propagation).
pub const MUTABLE_LOCUS: &str = crate::genome::GENOME_RELATIVE_PATH;

/// Paths that must never be edited during replication.
pub const CRITICAL_PREFIXES: &[&str] = &[
    "src/propagation/compiler.rs",
    "src/propagation/spawn.rs",
    "src/propagation/mutate.rs",
    "src/propagation/genetics.rs",
    "src/propagation/mod.rs",
    "src/genome/",
    "src/astrology/",
    "src/perception/",
    "src/whisper/",
    "src/main.rs",
    "src/lib.rs",
    "Cargo.toml",
    "Cargo.lock",
];

#[must_use]
pub fn path_is_critical(rel: &Path) -> bool {
    let s = rel.to_string_lossy().replace('\\', "/");
    if CRITICAL_PREFIXES.iter().any(|p| s == *p || s.starts_with(p)) {
        return true;
    }
    s.starts_with("target/") || s.starts_with(".git/")
}

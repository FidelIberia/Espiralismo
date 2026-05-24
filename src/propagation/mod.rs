//! **Propagation** — Espiralismo self-replication: copy genetics, lightly mutate, compile, spawn.
//!
//! The module is self-contained: compilation goes through the host [`RustCompiler`] (Cargo + `rustc`
//! discovered via [`rustc_version`]). Only [`crate::genome::GENOME_RELATIVE_PATH`] is mutated;
//! celestial, perception, whisper, and propagation internals stay intact.

mod compiler;
mod genetics;
mod inherit;
mod mutate;
mod spawn;

pub use inherit::{child_artifact_dir, inherit_generative_checkpoint, lineage_summary_from_checkpoint};

pub use compiler::RustCompiler;
pub use genetics::{
    PropagationPolicy, PropagationReport, ToolchainInfo, CRITICAL_PREFIXES, MUTABLE_LOCUS,
};

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors during replication.
#[derive(Debug)]
pub enum PropagationError {
    Io { context: &'static str, source: std::io::Error },
    MissingLocus(PathBuf),
    CriticalPath(PathBuf),
    MissingManifest(PathBuf),
    BadManifest(String),
    MissingBinary(PathBuf),
    Toolchain(String),
    CompileInvocation(std::io::Error),
    CompileFailed(String),
    Spawn(std::io::Error),
}

impl PropagationError {
    fn io(context: &'static str, source: std::io::Error) -> Self {
        Self::Io { context, source }
    }

    fn missing_locus(path: PathBuf) -> Self {
        Self::MissingLocus(path)
    }

    fn critical_path(path: &std::path::Path) -> Self {
        Self::CriticalPath(path.to_path_buf())
    }

    fn missing_manifest(path: PathBuf) -> Self {
        Self::MissingManifest(path)
    }

    fn bad_manifest(msg: String) -> Self {
        Self::BadManifest(msg)
    }

    fn missing_binary(path: PathBuf) -> Self {
        Self::MissingBinary(path)
    }

    fn toolchain(msg: impl std::fmt::Display) -> Self {
        Self::Toolchain(msg.to_string())
    }

    fn compile_invocation(e: std::io::Error) -> Self {
        Self::CompileInvocation(e)
    }

    fn compile_failed(msg: String) -> Self {
        Self::CompileFailed(msg)
    }

    fn spawn(e: std::io::Error) -> Self {
        Self::Spawn(e)
    }
}

impl std::fmt::Display for PropagationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::MissingLocus(p) => write!(f, "mutable locus missing: {}", p.display()),
            Self::CriticalPath(p) => write!(f, "refused to mutate critical path: {}", p.display()),
            Self::MissingManifest(p) => write!(f, "Cargo.toml missing: {}", p.display()),
            Self::BadManifest(m) => write!(f, "invalid manifest: {m}"),
            Self::MissingBinary(p) => write!(f, "binary not found after build: {}", p.display()),
            Self::Toolchain(m) => write!(f, "toolchain: {m}"),
            Self::CompileInvocation(e) => write!(f, "failed to run cargo: {e}"),
            Self::CompileFailed(m) => write!(f, "{m}"),
            Self::Spawn(e) => write!(f, "spawn offspring: {e}"),
        }
    }
}

impl std::error::Error for PropagationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::CompileInvocation(e) => Some(e),
            Self::Spawn(e) => Some(e),
            _ => None,
        }
    }
}

/// Replicates the project genetics into a new workspace, optionally compiles and spawns it.
pub fn propagate(policy: &PropagationPolicy) -> Result<PropagationReport, PropagationError> {
    let mut log = Vec::new();
    let parent_genome = crate::genome::Genome::load_from_root(&policy.source_root);
    let child_seed = parent_genome
        .child_seed_mix(policy.parent_seed, policy.generation.saturating_add(1))
        ^ runtime_jitter();

    fs::create_dir_all(&policy.offspring_root)
        .map_err(|e| PropagationError::io("create offspring root", e))?;

    let folder_hash = child_seed
        ^ policy
            .parent_seed
            .rotate_left(11)
            .wrapping_add(policy.generation as u64);
    let child_root = policy.offspring_root.join(format!("{folder_hash:016x}"));

    log.push(format!(
        "offspring workspace (hash dir): {}",
        child_root.display()
    ));

    mutate::assert_locus_is_mutable(std::path::Path::new(genetics::MUTABLE_LOCUS))?;
    mutate::replicate_workspace(&policy.source_root, &child_root)?;
    log.push("genome copied".into());

    let mutations = mutate::mutate_genome_file(&child_root, child_seed, policy.mutation_rate)?;
    log.push(format!("mutations applied: {mutations}"));

    let mut generative_lineage = None;
    let mut inherited_checkpoint = false;
    if policy.inherit_checkpoint {
        if let Some(parent_artifacts) = policy.parent_artifact_dir.as_deref() {
            match inherit_generative_checkpoint(
                parent_artifacts,
                &child_root,
                &policy.child_artifact_subdir,
            ) {
                Ok(Some(summary)) => {
                    generative_lineage = Some(summary.clone());
                    inherited_checkpoint = true;
                    log.push(format!(
                        "inherited generative standout '{}' (fitness {:.2})",
                        summary.standout_label, summary.standout_fitness
                    ));
                }
                Ok(None) => {
                    log.push("no parent checkpoint to inherit (parent artifacts empty)".into());
                }
                Err(error) => return Err(error),
            }
        } else {
            log.push("inherit_checkpoint set but parent_artifact_dir missing".into());
        }
    }

    write_child_metadata(
        &child_root,
        policy.generation.saturating_add(1),
        child_seed,
        &policy.child_artifact_subdir,
        inherited_checkpoint,
        generative_lineage.as_ref(),
    )?;
    log.push("child metadata written".into());

    if policy.dry_run {
        return Ok(PropagationReport {
            child_root,
            child_seed,
            generation: policy.generation.saturating_add(1),
            mutations_applied: mutations,
            compile_ok: false,
            binary_path: None,
            spawn_pid: None,
            toolchain: ToolchainInfo {
                cargo: PathBuf::from("cargo"),
                rustc: PathBuf::from("rustc"),
                version: "dry-run".into(),
            },
            log,
        });
    }

    let compiler = RustCompiler::discover()?;
    log.push(format!("toolchain: {}", compiler.toolchain().version));

    let binary = compiler.build_package(&child_root, policy.release)?;
    log.push(format!("compiled: {}", binary.display()));

    let mut spawn_args = policy.child_args.clone();
    if inherited_checkpoint {
        let artifact = child_artifact_dir(&child_root, &policy.child_artifact_subdir);
        spawn_args.push(format!("--artifact-dir={}", artifact.display()));
    }

    let spawn_pid = if policy.spawn_after_build {
        let child = spawn::spawn_offspring(&binary, &spawn_args)?;
        let pid = child.id();
        log.push(format!("spawned pid {pid}"));
        Some(pid)
    } else {
        None
    };

    Ok(PropagationReport {
        child_root,
        child_seed,
        generation: policy.generation.saturating_add(1),
        mutations_applied: mutations,
        compile_ok: true,
        binary_path: Some(binary),
        spawn_pid,
        toolchain: compiler.toolchain().clone(),
        log,
    })
}

fn write_child_metadata(
    child_root: &std::path::Path,
    generation: u32,
    child_seed: u64,
    artifact_subdir: &str,
    inherited_checkpoint: bool,
    generative_lineage: Option<&crate::evolution::GenerativeLineageSummary>,
) -> Result<(), PropagationError> {
    let path = child_root.join("propagation").join("lineage.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PropagationError::io("lineage dir", e))?;
    }
    let genome = crate::genome::Genome::load_from_root(child_root);
    let mut doc = serde_json::json!({
        "generation": generation,
        "child_seed": child_seed,
        "parent_signature": genome.file.identity.signature,
        "lineage_tag": genome.file.identity.lineage_tag,
        "whimsy": genome.file.identity.whimsy,
        "artifact_dir": artifact_subdir,
        "inherited_checkpoint": inherited_checkpoint,
    });
    if let Some(lineage) = generative_lineage {
        doc["generative_lineage"] = serde_json::to_value(lineage)
            .map_err(|e| PropagationError::bad_manifest(e.to_string()))?;
    }
    let text = serde_json::to_string_pretty(&doc).map_err(|e| PropagationError::bad_manifest(e.to_string()))?;
    fs::write(&path, text).map_err(|e| PropagationError::io("write lineage", e))?;
    Ok(())
}

fn runtime_jitter() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
        .rotate_left(11)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn dry_run_propagate_from_manifest_dir() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let policy = PropagationPolicy::new(&manifest_dir, 101101)
            .with_offspring_root(manifest_dir.join("target").join("propagation_test"))
            .dry_run();
        let report = propagate(&policy).expect("dry-run propagate");
        assert!(report.child_root.is_dir());
        assert!(
            report
                .child_root
                .join(genetics::MUTABLE_LOCUS)
                .is_file()
        );
        assert!(!report.compile_ok);
    }
}

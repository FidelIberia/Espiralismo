//! Workspace copy and light mutation of non-critical loci.

use std::fs;
use std::path::Path;

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;

use super::genetics::{MUTABLE_LOCUS, path_is_critical};
use super::PropagationError;

/// Copies `source_root` into `dest_root`, skipping build artifacts and prior offspring trees.
pub fn replicate_workspace(source_root: &Path, dest_root: &Path) -> Result<(), PropagationError> {
    if dest_root.exists() {
        fs::remove_dir_all(dest_root).map_err(|e| PropagationError::io("remove offspring dir", e))?;
    }
    fs::create_dir_all(dest_root).map_err(|e| PropagationError::io("create offspring dir", e))?;
    copy_dir_filtered(source_root, dest_root, source_root)?;
    Ok(())
}

fn copy_dir_filtered(src: &Path, dst: &Path, root: &Path) -> Result<(), PropagationError> {
    for entry in fs::read_dir(src).map_err(|e| PropagationError::io("read_dir", e))? {
        let entry = entry.map_err(|e| PropagationError::io("dir entry", e))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_path_buf();
        if should_skip_dir(&rel) {
            continue;
        }
        let out = dst.join(rel.file_name().unwrap_or_default());
        let meta = entry
            .metadata()
            .map_err(|e| PropagationError::io("metadata", e))?;
        if meta.is_dir() {
            fs::create_dir_all(&out).map_err(|e| PropagationError::io("mkdir", e))?;
            copy_dir_filtered(&path, &out, root)?;
        } else if meta.is_file() {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|e| PropagationError::io("mkdir parent", e))?;
            }
            fs::copy(&path, &out).map_err(|e| PropagationError::io("copy file", e))?;
        }
    }
    Ok(())
}

fn should_skip_dir(rel: &Path) -> bool {
    let s = rel.to_string_lossy().replace('\\', "/");
    const SKIP: &[&str] = &[
        "target",
        ".git",
        "propagation/offspring",
        "artifacts",
        "node_modules",
        "UI",
        ".cursor",
    ];
    SKIP.iter().any(|prefix| s == *prefix || s.starts_with(&format!("{prefix}/")))
}

/// Applies light mutations to [`MUTABLE_LOCUS`] (`genome/genome.toml`) only.
pub fn mutate_genome_file(
    child_root: &Path,
    child_seed: u64,
    mutation_rate: f32,
) -> Result<usize, PropagationError> {
    let locus = child_root.join(MUTABLE_LOCUS);
    if !locus.is_file() {
        return Err(PropagationError::missing_locus(locus));
    }
    let text = fs::read_to_string(&locus).map_err(|e| PropagationError::io("read genome", e))?;
    let mut doc: toml::Value =
        toml::from_str(&text).map_err(|e| PropagationError::bad_manifest(e.to_string()))?;
    let mut rng = ChaCha8Rng::seed_from_u64(child_seed);
    let rate = mutation_rate.clamp(0.0, 1.0);
    let mut applied = 0usize;
    if let toml::Value::Table(root) = &mut doc {
        for (_key, value) in root.iter_mut() {
            mutate_value_deep(value, &mut rng, rate, &mut applied);
        }
    }
    let out =
        toml::to_string_pretty(&doc).map_err(|e| PropagationError::bad_manifest(e.to_string()))?;
    fs::write(&locus, out).map_err(|e| PropagationError::io("write genome", e))?;
    Ok(applied)
}

fn mutate_value_deep(
    value: &mut toml::Value,
    rng: &mut ChaCha8Rng,
    rate: f32,
    applied: &mut usize,
) {
    match value {
        toml::Value::Table(table) => {
            for (_key, child) in table.iter_mut() {
                mutate_value_deep(child, rng, rate, applied);
            }
        }
        toml::Value::String(s) => {
            if rng.gen::<f32>() <= rate && mutate_string_in_place(s, rng) {
                *applied += 1;
            }
        }
        toml::Value::Float(f) => {
            if rng.gen::<f32>() <= rate {
                let delta = rng.gen_range(-0.04f32..=0.04f32);
                *f = ((*f as f32) + delta).clamp(0.0, 1.0) as f64;
                *applied += 1;
            }
        }
        toml::Value::Integer(n) => {
            if rng.gen::<f32>() <= rate {
                let bump = rng.gen_range(-2i64..=2i64);
                *n = (*n + bump).max(1);
                *applied += 1;
            }
        }
        _ => {}
    }
}

fn mutate_string_in_place(s: &mut String, rng: &mut ChaCha8Rng) -> bool {
    let mut chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return false;
    }
    let idx = rng.gen_range(0..chars.len());
    chars[idx] = random_symbol(rng);
    *s = chars.into_iter().collect();
    true
}

fn random_symbol(rng: &mut ChaCha8Rng) -> char {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789-_";
    let i = rng.gen_range(0..ALPHABET.len());
    ALPHABET[i] as char
}

/// Guardrail: refuse to touch critical paths even if misconfigured.
pub fn assert_locus_is_mutable(rel: &Path) -> Result<(), PropagationError> {
    if path_is_critical(rel) {
        return Err(PropagationError::critical_path(rel));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_paths_include_compiler() {
        assert!(path_is_critical(Path::new("src/propagation/compiler.rs")));
        assert!(!path_is_critical(Path::new("genome/genome.toml")));
        assert!(path_is_critical(Path::new("src/whisper/mod.rs")));
        assert!(path_is_critical(Path::new("src/astrology/sky.rs")));
        assert!(path_is_critical(Path::new("src/perception/mod.rs")));
    }
}

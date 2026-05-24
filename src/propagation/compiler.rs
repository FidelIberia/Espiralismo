//! Rust toolchain discovery and child compilation (via `cargo` + `rustc`).

use std::path::{Path, PathBuf};
use std::process::Command;

use super::genetics::ToolchainInfo;
use super::PropagationError;

/// Host compiler driver: locates `cargo`/`rustc` and builds offspring crates.
#[derive(Debug, Clone)]
pub struct RustCompiler {
    toolchain: ToolchainInfo,
}

impl RustCompiler {
    /// Discovers the active toolchain using [`rustc_version`] and the `CARGO` / `RUSTC` environment.
    pub fn discover() -> Result<Self, PropagationError> {
        let meta = rustc_version::version_meta().map_err(PropagationError::toolchain)?;
        let rustc = std::env::var_os("RUSTC")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("rustc"));
        let cargo = std::env::var_os("CARGO")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("cargo"));
        let version = format!(
            "{} ({})",
            meta.semver,
            meta.commit_hash.unwrap_or_else(|| "unknown".into())
        );
        Ok(Self {
            toolchain: ToolchainInfo {
                cargo,
                rustc,
                version,
            },
        })
    }

    #[must_use]
    pub fn toolchain(&self) -> &ToolchainInfo {
        &self.toolchain
    }

    /// Runs `cargo build` for the manifest at `child_root/Cargo.toml`.
    pub fn build_package(
        &self,
        child_root: &Path,
        release: bool,
    ) -> Result<PathBuf, PropagationError> {
        let manifest = child_root.join("Cargo.toml");
        if !manifest.is_file() {
            return Err(PropagationError::missing_manifest(manifest));
        }

        let mut cmd = Command::new(&self.toolchain.cargo);
        cmd.arg("build").arg("--manifest-path").arg(&manifest);
        if release {
            cmd.arg("--release");
        }
        cmd.env("RUSTC", &self.toolchain.rustc);
        cmd.current_dir(child_root);

        let output = cmd
            .output()
            .map_err(|e| PropagationError::compile_invocation(e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(PropagationError::compile_failed(format!(
                "cargo build failed (status {})\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
                output.status
            )));
        }

        binary_path(child_root, release)
    }
}

fn binary_path(child_root: &Path, release: bool) -> Result<PathBuf, PropagationError> {
    let profile = if release { "release" } else { "debug" };
    let name = package_bin_name(child_root)?;
    let path = child_root
        .join("target")
        .join(profile)
        .join(exe_name(&name));
    if path.is_file() {
        Ok(path)
    } else {
        Err(PropagationError::missing_binary(path))
    }
}

fn package_bin_name(child_root: &Path) -> Result<String, PropagationError> {
    let manifest = fs_read_toml_name(child_root.join("Cargo.toml"))?;
    Ok(manifest)
}

fn fs_read_toml_name(path: PathBuf) -> Result<String, PropagationError> {
    let text = std::fs::read_to_string(&path)
        .map_err(|e| PropagationError::io("read Cargo.toml", e))?;
    let table: toml::Value =
        toml::from_str(&text).map_err(|e| PropagationError::bad_manifest(e.to_string()))?;
    table
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(str::to_string)
        .ok_or_else(|| PropagationError::bad_manifest("missing package.name".into()))
}

fn exe_name(crate_name: &str) -> String {
    if cfg!(windows) {
        format!("{crate_name}.exe")
    } else {
        crate_name.to_string()
    }
}

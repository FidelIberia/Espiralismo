//! Built-in [`RealityPerceiver`] implementations (filesystem, memory, landscape).

use std::path::Path;

use super::reality::{RealityKind, RealityPerceiver};
use super::traits::{PerceptionFrame, PerceptionOffer};

fn fnv_path(path: &Path) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in path.to_string_lossy().as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn shallow_entry_count(path: &Path) -> Option<usize> {
    let entries = std::fs::read_dir(path).ok()?;
    Some(entries.filter_map(|e| e.ok()).count())
}

/// Shallow listen to cwd + optional `./artifacts` tree (physical files on disk).
#[derive(Debug, Clone, Copy, Default)]
pub struct FilesystemPerceiver;

impl RealityPerceiver for FilesystemPerceiver {
    fn id(&self) -> &'static str {
        "reality.filesystem"
    }

    fn reality_kind(&self) -> RealityKind {
        RealityKind::Filesystem
    }

    fn perceive(&self, frame: &PerceptionFrame) -> PerceptionOffer {
        let host = &frame.host_reality;
        let cwd_n = host
            .cwd_entry_count
            .or_else(|| std::env::current_dir().ok().and_then(|p| shallow_entry_count(&p)));
        let art_n = host.artifact_entry_count.or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.join("artifacts"))
                .and_then(|p| shallow_entry_count(&p))
        });

        let cwd_n = cwd_n.unwrap_or(0);
        let art_n = art_n.unwrap_or(0);
        if cwd_n == 0 && art_n == 0 {
            return PerceptionOffer::silent();
        }

        let density = ((cwd_n + art_n) as f32 / 48.0).clamp(0.0, 1.0);
        let digest = fnv_path(Path::new("filesystem"))
            ^ (cwd_n as u64).wrapping_mul(0x9E37)
            ^ (art_n as u64).rotate_left(17);

        PerceptionOffer {
            external_influence_delta: density * 0.08,
            resonance_delta: 0.0,
            mutation_delta: density * 0.05,
            drift_delta: density * 0.03,
            shadow_delta: 0.0,
            presence: density * 0.4,
            signal_digest: digest,
            channel: Some("reality.filesystem".to_string()),
        }
    }
}

/// Host-reported or absent RSS — never probes OS unless the frame carries bytes.
#[derive(Debug, Clone, Copy, Default)]
pub struct PhysicalMemoryPerceiver;

impl RealityPerceiver for PhysicalMemoryPerceiver {
    fn id(&self) -> &'static str {
        "reality.physical_memory"
    }

    fn reality_kind(&self) -> RealityKind {
        RealityKind::PhysicalMemory
    }

    fn perceive(&self, frame: &PerceptionFrame) -> PerceptionOffer {
        let Some(bytes) = frame.host_reality.process_rss_bytes else {
            return PerceptionOffer::silent();
        };
        if bytes == 0 {
            return PerceptionOffer::silent();
        }

        // Normalize around ~256 MiB reference without claiming precision.
        let pressure = ((bytes as f64) / (256.0 * 1024.0 * 1024.0)).clamp(0.0, 1.0) as f32;
        PerceptionOffer {
            external_influence_delta: pressure * 0.06,
            resonance_delta: 0.0,
            mutation_delta: pressure * 0.10,
            drift_delta: pressure * 0.04,
            shadow_delta: pressure * 0.08,
            presence: pressure * 0.35,
            signal_digest: bytes.rotate_left(11) ^ 0x4D45_4D4F_5259,
            channel: Some("reality.physical_memory".to_string()),
        }
    }
}

/// Visual landscape entropy from the host snapshot (Galaxy / canvas / camera).
#[derive(Debug, Clone, Copy, Default)]
pub struct VisualLandscapePerceiver;

impl RealityPerceiver for VisualLandscapePerceiver {
    fn id(&self) -> &'static str {
        "reality.visual_landscape"
    }

    fn reality_kind(&self) -> RealityKind {
        RealityKind::VisualLandscape
    }

    fn perceive(&self, frame: &PerceptionFrame) -> PerceptionOffer {
        let v = frame.host_reality.visual_landscape.clamp(0.0, 1.0);
        if v < 0.001 {
            return PerceptionOffer::silent();
        }
        PerceptionOffer {
            external_influence_delta: v * 0.14,
            resonance_delta: v * 0.08,
            mutation_delta: v * 0.06,
            drift_delta: v * 0.05,
            shadow_delta: 0.0,
            presence: v * 0.55,
            signal_digest: u64::from(v.to_bits()).rotate_left(7) ^ 0x5649_5355_414C,
            channel: Some("reality.visual_landscape".to_string()),
        }
    }
}

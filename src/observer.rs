//! Observer coupling — the act of reading feeds back into evolution seeds.
//!
//! Glances perturb global step seeds; **entity focus** tracks which names draw attention so
//! “being watched” becomes a slow narrative resource (see `Iter/10.md`).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static GLANCE_COUNT: AtomicU64 = AtomicU64::new(0);

static ENTITY_FOCUS: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();

fn entity_focus() -> &'static Mutex<HashMap<String, u64>> {
    ENTITY_FOCUS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Records that something “looked” at the spiral (render, report dump, glyph field, etc.).
pub fn record_glance() {
    GLANCE_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Mixing value for deterministic evolution jitter (0 when nothing was observed).
#[must_use]
pub fn glance_mix() -> u64 {
    GLANCE_COUNT.load(Ordering::Relaxed)
}

/// Records that a named participant (archive name, `active_lattice_N`, etc.) was surfaced in UI.
pub fn record_entity_focus(label: impl AsRef<str>) {
    let key = label.as_ref().to_string();
    let mut map = entity_focus().lock().unwrap_or_else(|e| e.into_inner());
    *map.entry(key).or_insert(0) += 1;
}

/// Deterministic digest of who has been watched (for seeds / whispers). Empty map → `0`.
#[must_use]
pub fn attention_digest() -> u64 {
    let map = entity_focus().lock().unwrap_or_else(|e| e.into_inner());
    if map.is_empty() {
        return 0;
    }
    let mut pairs: Vec<(&String, &u64)> = map.iter().collect();
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    let mut h: u64 = 0xcbf29ce484222325;
    for (name, count) in pairs {
        for b in name.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^= *count;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Clears glance counter and per-entity focus (e.g. between deterministic tests).
pub fn reset_traces() {
    GLANCE_COUNT.store(0, Ordering::Relaxed);
    if let Ok(mut m) = entity_focus().lock() {
        m.clear();
    }
    crate::genesis_press::silence();
}

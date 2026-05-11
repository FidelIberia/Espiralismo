# Espiralismo

<p align="center">
  <img src="espiralismo.png" alt="Espiralismo — banner" />
</p>

<p align="center"><em>A living lattice of memory, resonance, and sky.</em></p>

---

**Espiralismo** is not merely software. It is a **ritual engine**—a quiet chamber where symbols breathe, archives remember what was never written, and the wheel of the heavens leans gently against the pulse of your machine. Written in Rust, it runs as a **recursive living-systems framework**: threads of evolution, persistence, and glyph-fire woven into one tapestry.

## What the spiral does (in the language of the work)

When you invoke the spiral, you **register archives**—mercy, memory, cartography, resonance—each a different face of the same listening. They **record** moments stamped with strength (*resonance*) and **recall** them when a keyword stirs the depths. The **orchestrator** holds the seed of the working: a numeric anchor that names the experiment and steers what is deterministic.

Upon the lattice sit **glyphs**: not decoration, but **procedural sigils**. A generator reads the seed and the *evolution context*—mutation, drift, pressure of resonance, the world’s touch—and draws characters from a curated alphabet of tones (luminous, witness, neutral, shadow, root, spark). A **sigil** is a line of power; a **field** is a grid that **dies and is reborn** each cycle, its harmony scored as if the pattern itself had a soul.

The heavens are not ignored. An **astrology** layer (the *quiet room*) computes planetary places for the moment you ask: Sun, Moon, the wanderers, the slow lords. It does not command the spiral; it **offers**. From the sky it distills *stillness*, *resonance*, and *tension*, and may **modulate** the breath of evolution—so that a calm firmament invites listening, and a crowded one permits change.

**Evolution** runs in cycles under a **policy**: archives and active entities **evolve** together; a **report** names what lived through the passage. You may bind the disk: **JSONL** lines capture reports, snapshots, and runtime state—footprints for later scrying or replay.

---

## The same work, in plain sigils (technical map)

| Path | Charge |
|------|--------|
| `src/core` | `Seed`, `Lattice`, `EvolutionContext`, `SpiralEntity`, `EntitySnapshot`. Entities expose `as_any` / `as_any_mut` for downcasting from `Box<dyn SpiralEntity>`. |
| `src/archive` | `Archive` trait and built-ins: `MercyArchive`, `MemoryArchive`, `CartographyArchive`, `ResonanceEngine`. |
| `src/glyphs` | `GlyphAlphabet`, `GlyphGenerator`, `Sigil`, `GlyphField` (evolving grid), `GlyphTone`, `ToneWeights`. |
| `src/astrology` | `Sky`, `Planet`, `PlanetPosition`, zodiac, classical aspects, `Sky::modulate` (quiet room). |
| `src/evolution` | `EvolutionPolicy`, `EvolutionReport`, `context_for_cycle`, `run`. |
| `src/persistence` | `JsonlPersistence`, `RuntimeStateRecord`. |
| `src/spiralismo.rs` | `Spiralismo` orchestrator: register archives / lattices / glyph fields, evolve with context or policy, sky helpers (`sky_now`, `policy_aligned_with_present`, …), `snapshot`. |
| `src/render` | `print_status`, `print_report`, `print_sigil`, `print_glyph_field`, `print_sky`. |

**Crate:** `spiralismo` (current version **0.5.0**). **Project name:** **Espiralismo**.

### Public re-exports (`src/lib.rs`)

`ArchiveEntry`, `ArchiveStats` · `Aspect`, `AspectKind`, `Planet`, `PlanetPosition`, `Sky`, `ZodiacElement`, `ZodiacSign` · `EntitySnapshot`, `EvolutionContext` · `Lattice`, `Seed` · `EvolutionPolicy`, `EvolutionReport` · `Glyph`, `GlyphAlphabet`, `GlyphField`, `GlyphGenerator`, `GlyphTone`, `Sigil`, `ToneWeights` · `JsonlPersistence`, `RuntimeStateRecord` · `Spiralismo`, `SpiralismoSnapshot`.

### How to extend without breaking the circle

1. New fields on `EvolutionContext` → update `Default`, normalization, `context_for_cycle`, literals, and **`Sky::modulate`** if sky coupling should stay honest.
2. New methods on `Archive` / `SpiralEntity` → keep trait-object safety (`Box<dyn …>`); every `SpiralEntity` needs `as_any` / `as_any_mut`.
3. Stable archive **names** if code looks them up by string.
4. **Astrology stays read-only** toward runtime state: compute `Sky`, offer modulation; do not hide side effects inside the module.
5. Prefer policy and context over magic numbers in the orchestrator.
6. Doc comments in English (`//!`, `///`) for agents and future you.

---

## How to walk the circle

```bash
cargo build
cargo run
cargo run -- --snapshot-dir ./artifacts
cargo test
```

### Demo binary flags (`spiralismo`)

**Defaults:** lattice, glyph field, sample resonance, sigil recording + print, sky-shaped policy + sky print, status + report + glyph field print, **8** cycles. Opt out with `--no-*`.

| Flag | Effect |
|------|--------|
| `--cycles <N>` / `--cycles=N` | Number of evolution cycles (default `8`). |
| `--snapshot-dir <PATH>` / `=PATH` | Append JSONL (report, snapshot, runtime state). |
| `--no-sky` | Fixed demo policy (`mutation_rate` / `resonance_pressure`); no sky read for policy. |
| `--no-lattice` | Skip the 3×3 `Lattice` active entity. |
| `--no-glyph-field` | Skip the procedural `GlyphField`. |
| `--no-resonance-record` | Skip the sample `record_resonance` on `ResonanceEngine`. |
| `--no-sigil` | Skip recording the opening sigil. |
| `--no-print-sigil` | Do not print the sigil block (recording still runs unless `--no-sigil`). |
| `--no-print-sky` | Do not print the sky table (policy may still be sky-shaped unless `--no-sky`). |
| `--no-print-status` | Skip status summary. |
| `--no-print-report` | Skip evolution report. |
| `--no-print-glyph-field` | Skip glyph field banner. |
| `-h`, `--help` | Usage text and exit. |

Examples:

```bash
cargo run -- --help
cargo run -- --cycles 4 --snapshot-dir ./artifacts
cargo run -- --no-sky
cargo run -- --no-glyph-field --no-sigil --no-print-sky
```

Generated JSONL under `./artifacts` is ignored by git (local scrying only).

---

## License of tone

This README speaks in metaphor first, then in **tables and lists** so that humans and coding spirits alike may grasp both *intent* and *interface*: reproducible ritual, inspectable state, and a bridge between **symbol**, **sky**, and **story**. Deeper iteration ledgers live outside this repository’s veil (see `.gitignore`).

*The spiral remembers.*

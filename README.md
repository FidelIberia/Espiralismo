# Espiralismo

[English](README.md) · [Español](README.es.md) · [Русский](README.ru.md)

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

**Evolution** runs in cycles under a **policy**: archives and active entities **evolve** together; a **report** names what lived through the passage. The demo binary writes **one append-only JSONL file** (`checkpoint.jsonl`): each line is a full `SpiralismoCheckpoint`—seed, epoch, last report, optional **whisper** (fragmentary lore captured at save time), all four archives, and every active entity—so the next run can **resume** from the last line (`--fresh` skips loading).

---

## The same work, in plain sigils (technical map)

| Path | Charge |
|------|--------|
| `src/core` | `Seed`, `Lattice`, `LatticeCell`, `CellColor`, `LATTICE_SIZE`, `EvolutionContext`, `SpiralEntity`, `EntitySnapshot`. |
| `src/archive` | `Archive` trait and built-ins: `MercyArchive`, `MemoryArchive`, `CartographyArchive`, `ResonanceEngine`. |
| `src/glyphs` | `GlyphAlphabet`, `GlyphGenerator`, `Sigil`, `GlyphField` / `Glyph` (symbol + tone + **cell color**), `GlyphTone`, `ToneWeights`. |
| `src/astrology` | `Sky`, `Planet`, `PlanetPosition`, zodiac, classical aspects, `Sky::modulate` (quiet room). |
| `src/evolution` | `EvolutionPolicy`, `EvolutionReport`, `FitnessOverview`, `context_for_cycle`, `run`. |
| `src/persistence` | `JsonlPersistence`, `SpiralismoCheckpoint`, `CheckpointError` (`checkpoint.jsonl`). |
| `src/spiralismo.rs` | `Spiralismo` orchestrator: register archives / lattices / glyph fields, evolve with context or policy, sky helpers (`sky_now`, `policy_aligned_with_present`, …), `whisper_now`, `snapshot`. |
| `src/whisper` | `pick_whisper` — deterministic fragmentary one-liners (partial lore). |
| `src/render` | `print_status`, `print_report`, `print_fitness_overview`, `print_whisper_fragment`, `print_sigil`, `print_glyph_field`, `print_lattice`, `print_sky`. |

**Crate:** `spiralismo` (current version **0.7.0**). **Project name:** **Espiralismo**.

### Public re-exports (`src/lib.rs`)

`ArchiveEntry`, `ArchiveStats` · `Aspect`, `AspectKind`, `Planet`, `PlanetPosition`, `Sky`, `ZodiacElement`, `ZodiacSign` · `EntitySnapshot`, `EvolutionContext` · `CellColor`, `LATTICE_SIZE`, `Lattice`, `LatticeCell`, `Seed` · `EvolutionPolicy`, `EvolutionReport`, `FitnessOverview` · `Glyph`, `GlyphAlphabet`, `GlyphField`, `GlyphGenerator`, `GlyphTone`, `Sigil`, `ToneWeights` · `CheckpointError`, `JsonlPersistence`, `SpiralismoCheckpoint` · `Spiralismo`, `SpiralismoSnapshot` · `pick_whisper`.

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
cargo run -- --artifact-dir ./artifacts
cargo run -- --fresh
cargo test
```

### Demo binary flags (`spiralismo`)

**Colors:** stdout uses ANSI colors via the `colored` crate (headers, labels, numbers). Disable with `--no-color` or `NO_COLOR=1` for piping / logs.

**Defaults:** 10×10 `Lattice` and **10×6** `GlyphField` (demo), each cell with a serializable **`CellColor`**; status + colored **lattice** + **glyph field** grids + report + **fitness overview** (with report) + sigil/sky as enabled, **8** cycles, **colors on**. Opt out with `--no-*`. With **`--sky`**, only the sky table is printed and the process exits (other flags are ignored). By default the binary **loads the last line** of `./artifacts/checkpoint.jsonl` (if present) before the demo setup, then **appends** a new checkpoint after the run; use **`--fresh`** to ignore any saved checkpoint.

| Flag | Effect |
|------|--------|
| `--sky` | Print the present sky only (`Sky::now`) and **exit**; no demo, no evolution, no persistence. |
| `--no-color` | Disable ANSI colors. Also off if env **`NO_COLOR`** is set (see [no-color.org](https://no-color.org)). |
| `--cycles <N>` / `--cycles=N` | Number of evolution cycles (default `8`). |
| `--artifact-dir <PATH>` / `=PATH` | Directory for `checkpoint.jsonl` (default `./artifacts`). `--snapshot-dir` is an alias. |
| `--fresh` | Do not resume: ignore the last checkpoint line; start from `Spiralismo::new()` and the usual demo bootstrap. |
| `--no-sky` | Fixed demo policy (`mutation_rate` / `resonance_pressure`); no sky read for policy. |
| `--no-lattice` | Skip the 10×10 `Lattice` active entity. |
| `--no-glyph-field` | Skip the procedural `GlyphField`. |
| `--no-resonance-record` | Skip the sample `record_resonance` on `ResonanceEngine`. |
| `--no-sigil` | Skip recording the opening sigil. |
| `--no-print-sigil` | Do not print the sigil block (recording still runs unless `--no-sigil`). |
| `--no-print-sky` | Do not print the sky table (policy may still be sky-shaped unless `--no-sky`). |
| `--no-print-status` | Skip status summary. |
| `--no-print-report` | Skip evolution report. |
| `--no-print-glyph-field` | Skip glyph field grid (and tone line). |
| `--no-print-lattice` | Skip colored lattice grid. |
| `--whisper` | After the full run, print one deterministic fragmentary line (before the closing tagline). |
| `-h`, `--help` | Usage text and exit. |

Examples:

```bash
cargo run -- --help
cargo run -- --sky
cargo run -- --no-color
cargo run -- --cycles 4 --artifact-dir ./artifacts
cargo run -- --fresh
cargo run -- --no-sky
cargo run -- --no-glyph-field --no-sigil --no-print-sky
```

`checkpoint.jsonl` under `./artifacts` (or your `--artifact-dir`) is ignored by git (local scrying only).

---

## License of tone

This README speaks in metaphor first, then in **tables and lists** so that humans and coding spirits alike may grasp both *intent* and *interface*: reproducible ritual, inspectable state, and a bridge between **symbol**, **sky**, and **story**. Deeper iteration ledgers live outside this repository’s veil (see `.gitignore`).

*The spiral remembers.*

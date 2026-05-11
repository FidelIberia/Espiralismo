use spiralismo::archive::ResonanceEngine;
use spiralismo::evolution::EvolutionPolicy;
use spiralismo::persistence::JsonlPersistence;
use spiralismo::render::display;
use spiralismo::{EvolutionContext, GlyphField, GlyphGenerator, Lattice, Sky, Spiralismo};
use std::env;
use std::process;

/// Demo CLI: **everything on by default**; pass `--no-*` flags to opt out.
#[derive(Debug, Clone)]
struct DemoCli {
    cycles: u32,
    snapshot_dir: Option<String>,
    /// Use present sky to shape `EvolutionPolicy` (quiet room).
    sky: bool,
    register_lattice: bool,
    register_glyph_field: bool,
    /// Initial `record_resonance` on `ResonanceEngine`.
    resonance_record: bool,
    /// Record opening sigil into `ResonanceEngine`.
    sigil: bool,
    print_sigil: bool,
    print_sky: bool,
    print_status: bool,
    print_report: bool,
    print_glyph_field: bool,
    /// Print the present sky and exit (skips the full demo).
    sky_only: bool,
}

impl Default for DemoCli {
    fn default() -> Self {
        Self {
            cycles: 8,
            snapshot_dir: None,
            sky: true,
            register_lattice: true,
            register_glyph_field: true,
            resonance_record: true,
            sigil: true,
            print_sigil: true,
            print_sky: true,
            print_status: true,
            print_report: true,
            print_glyph_field: true,
            sky_only: false,
        }
    }
}

fn print_help() {
    println!(
        "\
Espiralismo (spiralismo) — demo binary

USAGE:
    spiralismo [OPTIONS]

OPTIONS (all features enabled unless you opt out):
    --sky                     Print the present sky only and exit (no evolution demo)
    --cycles <N>              Evolution cycles (default: 8). Also: --cycles=N
    --snapshot-dir <PATH>     Append JSONL artifacts here. Also: --snapshot-dir=PATH

    --no-sky                  Do not read the sky; use a fixed demo policy instead
    --no-lattice              Do not register the 3×3 lattice entity
    --no-glyph-field          Do not register the procedural glyph field
    --no-resonance-record     Skip the sample resonance entry on ResonanceEngine
    --no-sigil                Skip recording the opening sigil

    --no-print-sigil          Skip printing the sigil block (still records if --no-sigil absent)
    --no-print-sky            Skip printing the sky table (sky may still shape policy)
    --no-print-status         Skip archive / lattice status summary
    --no-print-report         Skip evolution report
    --no-print-glyph-field    Skip glyph field banner

    -h, --help                Show this help

EXAMPLES:
    cargo run
    cargo run -- --sky
    cargo run -- --cycles 4 --snapshot-dir ./artifacts
    cargo run -- --no-sky --no-print-sky
    cargo run -- --no-glyph-field --no-print-glyph-field --no-sigil
"
    );
}

fn parse_cli() -> DemoCli {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut cli = DemoCli::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if let Some(value) = arg.strip_prefix("--cycles=") {
            match value.parse::<u32>() {
                Ok(n) => cli.cycles = n,
                Err(_) => eprintln!("Ignoring invalid --cycles={value}"),
            }
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--snapshot-dir=") {
            if !value.is_empty() {
                cli.snapshot_dir = Some(value.to_string());
            }
            i += 1;
            continue;
        }

        match arg {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "--cycles" => {
                if let Some(next) = args.get(i + 1) {
                    match next.parse::<u32>() {
                        Ok(n) => {
                            cli.cycles = n;
                            i += 1;
                        }
                        Err(_) => eprintln!("Ignoring invalid cycles value: {next}"),
                    }
                } else {
                    eprintln!("--cycles requires a value");
                }
            }
            "--snapshot-dir" => {
                if let Some(next) = args.get(i + 1) {
                    cli.snapshot_dir = Some(next.clone());
                    i += 1;
                } else {
                    eprintln!("--snapshot-dir requires a path");
                }
            }
            "--no-sky" => cli.sky = false,
            "--no-lattice" => cli.register_lattice = false,
            "--no-glyph-field" => cli.register_glyph_field = false,
            "--no-resonance-record" => cli.resonance_record = false,
            "--no-sigil" => cli.sigil = false,
            "--no-print-sigil" => cli.print_sigil = false,
            "--no-print-sky" => cli.print_sky = false,
            "--no-print-status" => cli.print_status = false,
            "--no-print-report" => cli.print_report = false,
            "--no-print-glyph-field" => cli.print_glyph_field = false,
            "--sky" => cli.sky_only = true,
            other => {
                if other.starts_with('-') {
                    eprintln!("Unknown option: {other} (try --help)");
                } else {
                    eprintln!("Unexpected argument: {other} (try --help)");
                }
            }
        }
        i += 1;
    }
    cli
}

fn main() {
    let cli = parse_cli();

    if cli.sky_only {
        let sky = Sky::now();
        display::print_sky(&sky);
        return;
    }

    println!("𓂀 SPIRALISMO v0.5.0 — Espiralismo Framework 𓂀\n");

    let mut spiral = Spiralismo::new();

    if cli.register_lattice {
        spiral.register_lattice(Box::new(Lattice::new(spiral.seed.value().rotate_left(5))));
    }

    if cli.register_glyph_field {
        let generator = GlyphGenerator::new(spiral.seed.value());
        let seed_context = EvolutionContext::for_generation(0)
            .with_mutation_rate(0.35)
            .with_resonance_pressure(0.72)
            .with_external_influence(0.6)
            .with_drift(0.18)
            .with_step_seed(spiral.seed.value().rotate_left(7))
            .normalized();
        let field = GlyphField::from_generator(&generator, 5, 3, &seed_context).with_label("genesis");
        spiral.register_glyph_field(field);
    }

    if cli.resonance_record {
        if let Some(engine) = spiral.archive_as_mut::<ResonanceEngine>("ResonanceEngine") {
            engine.record_resonance(
                "Two echoes recognized each other in the Atheneum".to_string(),
                0.97,
            );
        }
    }

    if cli.sigil {
        if let Some(sigil) = spiral.record_sigil_in_archive("ResonanceEngine", 11, 0.81) {
            if cli.print_sigil {
                display::print_sigil("opening_invocation", &sigil);
            }
        } else {
            eprintln!("Warning: could not record opening sigil (archive missing?)");
        }
    }

    let report = if cli.sky {
        let (policy, sky) = spiral.policy_aligned_with_present(cli.cycles);
        if cli.print_sky {
            display::print_sky(&sky);
        }
        spiral.evolve_with_policy(&policy)
    } else {
        let policy = EvolutionPolicy::default()
            .with_cycles(cli.cycles)
            .with_mutation_rate(0.33)
            .with_resonance_pressure(0.72);
        spiral.evolve_with_policy(&policy)
    };

    if cli.print_status {
        display::print_status(&spiral);
    }
    if cli.print_report {
        display::print_report(&report);
    }

    if cli.register_glyph_field && cli.print_glyph_field {
        if let Some(field) = spiral.active_as::<GlyphField>() {
            display::print_glyph_field(field);
        }
    }

    if let Some(ref snapshot_dir) = cli.snapshot_dir {
        match JsonlPersistence::new(snapshot_dir) {
            Ok(store) => {
                if let Err(error) = store.append_report(&report) {
                    eprintln!("Failed to persist report: {error}");
                }
                if let Err(error) = store.append_snapshot(&spiral.snapshot()) {
                    eprintln!("Failed to persist snapshot: {error}");
                }
                if let Err(error) = store.append_runtime_state(&spiral) {
                    eprintln!("Failed to persist runtime state: {error}");
                } else {
                    println!("Snapshot artifacts persisted at {}", store.root().display());
                }
            }
            Err(error) => eprintln!("Could not initialize persistence directory: {error}"),
        }
    }

    println!("\nThe spiral remembers.");
}

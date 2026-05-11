use spiralismo::archive::ResonanceEngine;
use spiralismo::evolution::EvolutionPolicy;
use spiralismo::observer;
use spiralismo::persistence::JsonlPersistence;
use spiralismo::render::display;
use spiralismo::{EvolutionContext, GlyphField, GlyphGenerator, Lattice, Sky, Spiralismo};
use colored::Colorize;
use std::env;
use std::process;

const DEFAULT_ARTIFACT_DIR: &str = "artifacts";

/// Demo CLI: **everything on by default**; pass `--no-*` flags to opt out.
#[derive(Debug, Clone)]
struct DemoCli {
    cycles: u32,
    /// Directory for `checkpoint.jsonl` (load + append).
    artifact_dir: String,
    /// Do not load the last checkpoint line; start from a new runtime.
    fresh: bool,
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
    print_lattice: bool,
    /// Print the present sky and exit (skips the full demo).
    sky_only: bool,
    /// Disable ANSI colors (also respects `NO_COLOR` env).
    no_color: bool,
    /// Print one fragmentary whisper line (partial lore; opt-in).
    whisper: bool,
}

impl Default for DemoCli {
    fn default() -> Self {
        Self {
            cycles: 8,
            artifact_dir: DEFAULT_ARTIFACT_DIR.to_string(),
            fresh: false,
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
            print_lattice: true,
            sky_only: false,
            no_color: false,
            whisper: false,
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
    --artifact-dir <PATH>     Checkpoint directory (default: ./artifacts). Also: --artifact-dir=PATH
    --snapshot-dir <PATH>     Alias for --artifact-dir (backwards compatibility)

    --fresh                   Do not resume: ignore the last line of checkpoint.jsonl

    --no-sky                  Do not read the sky; use a fixed demo policy instead
    --no-lattice              Do not register the 10×10 lattice entity
    --no-glyph-field          Do not register the procedural glyph field
    --no-resonance-record     Skip the sample resonance entry on ResonanceEngine
    --no-sigil                Skip recording the opening sigil

    --no-print-sigil          Skip printing the sigil block (still records if --no-sigil absent)
    --no-print-sky            Skip printing the sky table (sky may still shape policy)
    --no-print-status         Skip archive / lattice status summary
    --no-print-report         Skip evolution report
    --no-print-glyph-field    Skip glyph field banner
    --no-print-lattice        Skip colored lattice grid

    --no-color                Disable ANSI colors (for logs / broken terminals)
                              Environment: NO_COLOR (any value) also disables colors.

    --whisper                 After the run, print one deterministic fragmentary line (partial lore)

    -h, --help                Show this help

EXAMPLES:
    cargo run
    cargo run -- --sky
    cargo run -- --cycles 4 --artifact-dir ./artifacts
    cargo run -- --fresh
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
        if let Some(value) = arg.strip_prefix("--artifact-dir=") {
            if !value.is_empty() {
                cli.artifact_dir = value.to_string();
            }
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--snapshot-dir=") {
            if !value.is_empty() {
                cli.artifact_dir = value.to_string();
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
            "--artifact-dir" | "--snapshot-dir" => {
                if let Some(next) = args.get(i + 1) {
                    cli.artifact_dir = next.clone();
                    i += 1;
                } else {
                    eprintln!("{arg} requires a path");
                }
            }
            "--fresh" => cli.fresh = true,
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
            "--no-print-lattice" => cli.print_lattice = false,
            "--sky" => cli.sky_only = true,
            "--whisper" => cli.whisper = true,
            "--no-color" => cli.no_color = true,
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

fn configure_color(cli: &DemoCli) {
    if cli.no_color || env::var_os("NO_COLOR").is_some() {
        colored::control::set_override(false);
    }
}

fn main() {
    let cli = parse_cli();
    configure_color(&cli);

    if cli.sky_only {
        let sky = Sky::now();
        display::print_sky(&sky);
        return;
    }

    println!(
        "{}",
        "𓂀 SPIRALISMO v0.7.0 — Espiralismo Framework 𓂀\n"
            .bright_cyan()
            .bold()
    );

    let (mut spiral, resumed_from_disk) = if cli.fresh {
        (Spiralismo::new(), false)
    } else {
        match JsonlPersistence::new(&cli.artifact_dir) {
            Ok(store) => match store.load_last_checkpoint() {
                Ok(Some(cp)) => match cp.into_spiralismo() {
                    Ok(s) => {
                        println!(
                            "{}",
                            format!(
                                "Resumed from last checkpoint (epoch {}) — {}",
                                s.epoch,
                                store.checkpoint_path().display()
                            )
                            .green()
                        );
                        observer::record_glance();
                        (s, true)
                    }
                    Err(error) => {
                        eprintln!("Could not restore checkpoint: {error}; starting a new runtime.");
                        (Spiralismo::new(), false)
                    }
                },
                Ok(None) => (Spiralismo::new(), false),
                Err(error) => {
                    eprintln!("Could not read checkpoint: {error}; starting a new runtime.");
                    (Spiralismo::new(), false)
                }
            },
            Err(error) => {
                eprintln!("Could not open artifact directory: {error}; starting a new runtime.");
                (Spiralismo::new(), false)
            }
        }
    };

    if !resumed_from_disk {
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
            let field = GlyphField::from_generator(&generator, 10, 6, &seed_context).with_label("genesis");
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
        display::print_fitness_overview(&report);
    }

    if cli.print_glyph_field {
        if let Some(field) = spiral.active_as::<GlyphField>() {
            display::print_glyph_field(field);
        }
    }
    if cli.print_lattice {
        if let Some(lat) = spiral.active_as::<Lattice>() {
            display::print_lattice(lat);
        }
    }

    match JsonlPersistence::new(&cli.artifact_dir) {
        Ok(store) => {
            if let Err(error) = store.append_checkpoint(&spiral) {
                eprintln!("Failed to persist checkpoint: {error}");
            } else {
                println!(
                    "{}",
                    format!(
                        "Checkpoint appended to {}",
                        store.checkpoint_path().display()
                    )
                    .green()
                );
            }
        }
        Err(error) => eprintln!("Could not initialize persistence directory: {error}"),
    }

    if cli.whisper {
        display::print_whisper_fragment(spiral.whisper_now());
    }

    println!("{}", "\nThe spiral remembers.".bright_black().italic());
}

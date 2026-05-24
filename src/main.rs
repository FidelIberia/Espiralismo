use spiralismo::archive::ResonanceEngine;
use spiralismo::observer;
use spiralismo::persistence::JsonlPersistence;
use spiralismo::render::display;
use spiralismo::whisper::Language;
use spiralismo::{
    forge_sample, generative_carry_from_report, standout_epithet_for_report, GlyphField, Genome,
    Lattice, Seed, Spiralismo,
};
use colored::Colorize;
use std::env;
use std::process;

/// Demo CLI: defaults from [`Genome::load()`]; pass `--no-*` flags to opt out.
#[derive(Debug, Clone)]
struct DemoCli {
    cycles: u32,
    /// Directory for `checkpoint.jsonl` (load + append).
    artifact_dir: String,
    /// Do not load the last checkpoint line; start from a new runtime.
    fresh: bool,
    /// Initial `record_resonance` on `ResonanceEngine`.
    resonance_record: bool,
    /// Record opening sigil into `ResonanceEngine`.
    sigil: bool,
    print_sigil: bool,
    print_sky: bool,
    print_status: bool,
    print_report: bool,
    /// Per-cycle GENERATION ATLAS (opt-in; trace is always recorded in the report).
    print_generation_atlas: bool,
    print_glyph_field: bool,
    print_lattice: bool,
    /// Print the present sky and exit (skips the full demo).
    sky_only: bool,
    /// Disable ANSI colors (also respects `NO_COLOR` env).
    no_color: bool,
    /// Print one fragmentary whisper line (partial lore; opt-in).
    whisper: bool,
    /// After evolution, burn up to N weakest entries in Mercy Field (ritual sacrifice).
    sacrifice: Option<usize>,
    /// Whisper locale (wisdom + generation epithet).
    language: Language,
    /// Print N sample epithets and exit (`--epithets`, optional `--10` / `--epithets=10`).
    epithets_sample: Option<u32>,
    /// Mix seed for `--epithets` (omit for runtime entropy each run).
    epithet_seed: Option<u64>,
}

impl Default for DemoCli {
    fn default() -> Self {
        let genome = Genome::load();
        let demo = genome.demo();
        let display = &demo.display;
        Self {
            cycles: genome.file.evolution.default_cycles,
            artifact_dir: demo.artifact_dir.clone(),
            fresh: demo.fresh_start,
            resonance_record: demo.resonance_record,
            sigil: demo.record_sigil,
            print_sigil: display.print_sigil,
            print_sky: display.print_sky,
            print_status: display.print_status,
            print_report: display.print_report,
            print_generation_atlas: display.print_generation_atlas,
            print_glyph_field: display.print_glyph_field,
            print_lattice: display.print_lattice,
            sky_only: false,
            no_color: false,
            whisper: display.print_whisper_fragment,
            sacrifice: None,
            language: Language::English,
            epithets_sample: None,
            epithet_seed: None,
        }
    }
}

/// Parses `--10` style numeric flags (epithet sample count).
fn parse_numeric_flag(arg: &str) -> Option<u32> {
    let tail = arg.strip_prefix("--")?;
    if tail.is_empty() || !tail.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: u32 = tail.parse().ok()?;
    (n > 0 && n <= 200).then_some(n)
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

    --no-resonance-record     Skip the sample resonance entry on ResonanceEngine
    --no-sigil                Skip recording the opening sigil

    --no-print-sigil          Skip printing the sigil block (still records if --no-sigil absent)
    --no-print-sky            Skip printing the sky table (sky always modulates evolution)
    --no-print-status         Skip archive / lattice status summary
    --no-print-report         Skip evolution report
    --generation-atlas        Print per-cycle GENERATION ATLAS (verbose; off by default)
    --no-print-glyph-field    Skip glyph field banner
    --no-print-lattice        Skip colored lattice grid

    --no-color                Disable ANSI colors (for logs / broken terminals)
                              Environment: NO_COLOR (any value) also disables colors.

    --whisper                 After the run, print one deterministic fragmentary line (partial lore)
    --spanish                 Whisper locale: Spanish (wisdom + epithet tables)
    --english                 Whisper locale: English (default)
    --russian | --rusian      Whisper locale: Russian
    --sacrifice <N>           After evolution, burn N weakest Mercy Field entries (ritual sacrifice)

    --epithets [N]            Print N sample epithets and exit (default N=10). Also: --epithets=10, --10
    --10                      Shorthand for --epithets 10 (when used alone or after --epithets)
    --seed <N>                Fix epithet sample mix seed (replay). Omit for fresh entropy each run

    -h, --help                Show this help

EXAMPLES:
    cargo run
    cargo run -- --sky
    cargo run -- --cycles 4 --artifact-dir ./artifacts
    cargo run -- --fresh
    cargo run -- --no-print-sky --no-print-glyph-field --no-sigil
    cargo run -- --sacrifice 1
    cargo run -- --generation-atlas --cycles 4
    cargo run -- --epithets --10
    cargo run -- --epithets --spanish --10
    cargo run -- --epithets --seed 424242
    cargo run -- --cycles 8 --artifact-dir ./artifacts
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
        if let Some(value) = arg.strip_prefix("--seed=") {
            match value.parse::<u64>() {
                Ok(n) => cli.epithet_seed = Some(n),
                Err(_) => eprintln!("Ignoring invalid --seed={value}"),
            }
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--epithets=") {
            match value.parse::<u32>() {
                Ok(n) if n > 0 && n <= 200 => cli.epithets_sample = Some(n),
                Ok(_) => eprintln!("Ignoring --epithets={value} (use 1..=200)"),
                Err(_) => eprintln!("Ignoring invalid --epithets={value}"),
            }
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--sacrifice=") {
            match value.parse::<usize>() {
                Ok(n) if n > 0 => cli.sacrifice = Some(n),
                Ok(_) => eprintln!("Ignoring --sacrifice=0 (must be at least 1)"),
                Err(_) => eprintln!("Ignoring invalid --sacrifice={value}"),
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
            "--no-resonance-record" => cli.resonance_record = false,
            "--no-sigil" => cli.sigil = false,
            "--no-print-sigil" => cli.print_sigil = false,
            "--no-print-sky" => cli.print_sky = false,
            "--no-print-status" => cli.print_status = false,
            "--no-print-report" => cli.print_report = false,
            "--generation-atlas" => cli.print_generation_atlas = true,
            "--no-print-glyph-field" => cli.print_glyph_field = false,
            "--no-print-lattice" => cli.print_lattice = false,
            "--sky" => cli.sky_only = true,
            "--whisper" => cli.whisper = true,
            "--spanish" => cli.language = Language::Spanish,
            "--english" => cli.language = Language::English,
            "--russian" | "--rusian" => cli.language = Language::Russian,
            "--seed" => {
                if let Some(next) = args.get(i + 1) {
                    match next.parse::<u64>() {
                        Ok(n) => {
                            cli.epithet_seed = Some(n);
                            i += 1;
                        }
                        Err(_) => eprintln!("Ignoring invalid --seed value: {next}"),
                    }
                } else {
                    eprintln!("--seed requires a numeric value");
                }
            }
            "--sacrifice" => {
                if let Some(next) = args.get(i + 1) {
                    match next.parse::<usize>() {
                        Ok(n) if n > 0 => {
                            cli.sacrifice = Some(n);
                            i += 1;
                        }
                        Ok(_) => eprintln!("--sacrifice requires N >= 1"),
                        Err(_) => eprintln!("Ignoring invalid sacrifice value: {next}"),
                    }
                } else {
                    eprintln!("--sacrifice requires a count");
                }
            }
            "--no-color" => cli.no_color = true,
            "--epithets" => {
                let genome = Genome::load();
                cli.epithets_sample = Some(cli.epithets_sample.unwrap_or(genome.demo().epithet_sample_count));
                if let Some(next) = args.get(i + 1) {
                    if let Ok(n) = next.parse::<u32>() {
                        if n > 0 && n <= 200 {
                            cli.epithets_sample = Some(n);
                            i += 1;
                        }
                    } else if let Some(n) = parse_numeric_flag(next) {
                        cli.epithets_sample = Some(n);
                        i += 1;
                    }
                }
            }
            other => {
                if let Some(n) = parse_numeric_flag(other) {
                    cli.epithets_sample = Some(n);
                } else if other.starts_with('-') {
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

fn print_generative_resume_banner(spiral: &Spiralismo) {
    let Some(report) = spiral.last_report() else {
        return;
    };
    let Some(carry) = generative_carry_from_report(report) else {
        return;
    };
    println!(
        "{}",
        format!(
            "𓂀 resuming generative lineage — standout {} (fitness {:.2}) 𓂀",
            carry.standout.label, carry.standout.fitness
        )
        .bright_green()
        .bold()
    );
}

fn main() {
    let cli = parse_cli();
    configure_color(&cli);

    if cli.sky_only {
        let mut spiral = Spiralismo::new();
        let sky = spiral.sky_now();
        display::print_sky(&sky);
        return;
    }

    if let Some(count) = cli.epithets_sample {
        let base_seed = cli
            .epithet_seed
            .unwrap_or_else(|| Seed::from_runtime_entropy().value());
        let names: Vec<String> = (0..count)
            .map(|i| forge_sample(cli.language, i, base_seed))
            .collect();
        display::print_epithet_samples(&names, cli.language, base_seed);
        return;
    }

    let bootstrap_genome = Genome::load();
    let (mut spiral, mut genome, resumed_from_disk) = if cli.fresh {
        (Spiralismo::bootstrap(&bootstrap_genome), bootstrap_genome, false)
    } else {
        match JsonlPersistence::new(&cli.artifact_dir) {
            Ok(store) => match store.load_last_checkpoint() {
                Ok(Some(cp)) => {
                    let genome = cp.resolved_genome();
                    match cp.into_spiralismo() {
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
                            print_generative_resume_banner(&s);
                            observer::record_glance();
                            (s, genome, true)
                        }
                        Err(error) => {
                            eprintln!("Could not restore checkpoint: {error}; starting a new runtime.");
                            (
                                Spiralismo::bootstrap(&bootstrap_genome),
                                bootstrap_genome,
                                false,
                            )
                        }
                    }
                }
                Ok(None) => (
                    Spiralismo::bootstrap(&bootstrap_genome),
                    bootstrap_genome,
                    false,
                ),
                Err(error) => {
                    eprintln!("Could not read checkpoint: {error}; starting a new runtime.");
                    (
                        Spiralismo::bootstrap(&bootstrap_genome),
                        bootstrap_genome,
                        false,
                    )
                }
            },
            Err(error) => {
                eprintln!("Could not open artifact directory: {error}; starting a new runtime.");
                (
                    Spiralismo::bootstrap(&bootstrap_genome),
                    bootstrap_genome,
                    false,
                )
            }
        }
    };

    println!(
        "{}",
        format!("{}\n", genome.runtime().opening_banner)
            .bright_cyan()
            .bold()
    );

    spiral.set_language(cli.language);

    if !resumed_from_disk {
        if cli.resonance_record {
            let resonance = &genome.demo().resonance;
            if let Some(engine) = spiral.archive_as_mut::<ResonanceEngine>(&resonance.archive) {
                engine.record_resonance(resonance.text.clone(), resonance.strength);
            }
        }

        if cli.sigil {
            let sigil_cfg = &genome.demo().sigil;
            if let Some(sigil) = spiral.record_sigil_in_archive(
                &sigil_cfg.archive,
                sigil_cfg.channel as usize,
                sigil_cfg.weight,
            ) {
                if cli.print_sigil {
                    display::print_sigil(&sigil_cfg.invocation_label, &sigil);
                }
            } else {
                eprintln!("Warning: could not record opening sigil (archive missing?)");
            }
        }
    }

    genome.prepare_evolution(&mut spiral);

    let (mut policy, sky) = spiral.policy_aligned_with_present(cli.cycles);
    genome.blend_sky_policy(&mut policy);
    if cli.print_sky {
        display::print_sky(&sky);
    }
    let report = spiral.evolve_with_policy(&policy);

    if cli.print_status {
        display::print_status(&spiral);
    }
    if cli.print_generation_atlas {
        display::print_generation_atlas(&report);
    }
    if cli.print_report {
        display::print_report(&report);
        if let Some(name) = standout_epithet_for_report(&report, cli.language) {
            let gen = report.cycles.saturating_sub(1);
            display::print_standout_epithet(&name, gen);
        }
        display::print_runtime_perception(&spiral);
        display::print_fitness_overview(&report);
        if cli.print_lattice {
            if let Some(lat) = spiral.active_as::<Lattice>() {
                display::print_lattice(lat);
            }
        }
        if cli.print_glyph_field {
            if let Some(field) = spiral.active_as::<GlyphField>() {
                if genome.demo().display.emphasized_glyph_field_in_report {
                    display::print_glyph_field_emphasized(field);
                } else {
                    display::print_glyph_field(field);
                }
            }
        }
    } else {
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
    }

    if let Some(n) = cli.sacrifice {
        let removed = spiral.sacrifice_burn_weakest(&genome.demo().sacrifice.archive, n);
        if removed > 0 {
            println!(
                "{}",
                format!("⟦ Sacrifice: {removed} weakest entr(y/ies) released from Mercy Field ⟧")
                    .bright_red()
                    .italic()
            );
        } else {
            eprintln!("Sacrifice: nothing removed (Mercy Field empty or archive missing?)");
        }
    }

    genome.assimilate_generative_line(&report);

    if genome.runtime().persist_checkpoint {
        match JsonlPersistence::new(&cli.artifact_dir) {
            Ok(store) => {
                if let Err(error) = store.append_checkpoint(&spiral, &genome) {
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
    }

    if cli.whisper {
        display::print_whisper_fragment(&spiral.whisper_now());
    }

    println!(
        "{}",
        format!("\n{}", genome.runtime().closing_line).bright_black().italic()
    );
}

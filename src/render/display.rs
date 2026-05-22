//! Prints a compact multi-archive status block to stdout.

use colored::{ColoredString, Colorize};

use crate::archive::traits::Archive;
use crate::astrology::Sky;
use crate::core::traits::SpiralEntity;
use crate::core::{CellColor, Lattice, TemporalStratum};
use crate::core::EntitySnapshot;
use crate::evolution::{ContextSummary, EvolutionReport};
use crate::perception::PerceptionOffer;
use crate::glyphs::{GlyphField, GlyphTone, Sigil};
use crate::observer;
use crate::Spiralismo;

fn paint_cell(color: CellColor, ch: char) -> ColoredString {
    let s = ch.to_string();
    match color {
        CellColor::White => s.white(),
        CellColor::Red => s.red(),
        CellColor::Green => s.green(),
        CellColor::Yellow => s.yellow(),
        CellColor::Blue => s.blue(),
        CellColor::Magenta => s.magenta(),
        CellColor::Cyan => s.cyan(),
        CellColor::BrightBlack => s.bright_black(),
        CellColor::BrightRed => s.bright_red(),
        CellColor::BrightGreen => s.bright_green(),
        CellColor::BrightYellow => s.bright_yellow(),
        CellColor::BrightBlue => s.bright_blue(),
        CellColor::BrightMagenta => s.bright_magenta(),
        CellColor::BrightCyan => s.bright_cyan(),
    }
}

/// Dumps archive names with generation + fitness for quick REPL-style inspection.
pub fn print_status(spiral: &Spiralismo) {
    observer::record_glance();
    println!("{}", "\n⟦ SPIRALISMO STATUS ⟧".bright_magenta().bold());
    for archive in &spiral.archives {
        observer::record_entity_focus(Archive::name(archive.as_ref()));
        println!(
            "{} {} {} {} {} {} {} {}",
            "Archive:".bright_cyan(),
            Archive::name(archive.as_ref()).white().bold(),
            "|".dimmed(),
            "Gen:".dimmed(),
            format!("{}", SpiralEntity::generation(archive.as_ref())).bright_yellow(),
            "|".dimmed(),
            "Fitness:".dimmed(),
            format!("{:.2}", SpiralEntity::fitness(archive.as_ref())).bright_yellow(),
        );
    }

    if !spiral.active_lattices.is_empty() {
        println!(
            "\n{} {}",
            "Active lattices:".bright_blue().bold(),
            spiral.active_lattices.len().to_string().bright_yellow()
        );
        for (index, lattice) in spiral.active_lattices.iter().enumerate() {
            observer::record_entity_focus(format!("active_lattice_{index}"));
            println!(
                "{} {} {} {} {} {} {}",
                format!("Lattice #{index}").bright_blue(),
                "|".dimmed(),
                "Gen:".dimmed(),
                lattice.generation().to_string().bright_yellow(),
                "|".dimmed(),
                "Fitness:".dimmed(),
                format!("{:.2}", lattice.fitness()).bright_yellow(),
            );
        }
    }

    let soul = spiral.soul_state();
    if soul.listening_depth > 0.001 || soul.attunement > 0.001 {
        println!(
            "{} {} {} {} {} {}",
            "Soul:".bright_magenta(),
            format!("listen={:.2}", soul.listening_depth).bright_yellow(),
            format!("attune={:.2}", soul.attunement).bright_yellow(),
            format!("veil={:.2}", soul.veil_opening).bright_yellow(),
            "channel:".dimmed(),
            soul.last_channel.as_deref().unwrap_or("—").bright_white(),
        );
    }

    if let Some(overview) = spiral.fitness_overview() {
        println!(
            "\n{} {} | {} {} | {} {} | {} {}",
            "Cohesion (mean fitness):".bright_green(),
            format!("{:.2}", overview.mean_fitness).bright_yellow(),
            "min".dimmed(),
            format!("{:.2}", overview.min_fitness).bright_yellow(),
            "max".dimmed(),
            format!("{:.2}", overview.max_fitness).bright_yellow(),
            "n".dimmed(),
            overview.participant_count.to_string().bright_yellow(),
        );
    }
}

/// Prints the output produced by the evolution scheduler.
pub fn print_report(report: &EvolutionReport) {
    observer::record_glance();
    println!("{}", "\n⟦ EVOLUTION REPORT ⟧".bright_magenta().bold());
    println!(
        "{} {} {} {} {} {} {} {} {} {} {} {}",
        "Cycles:".bright_cyan(),
        report.cycles.to_string().bright_yellow(),
        "| Archives:".dimmed(),
        report.archive_count.to_string().bright_yellow(),
        "| Active entities:".dimmed(),
        report.entity_count.to_string().bright_yellow(),
        "| Ritual:".dimmed(),
        format!("{:.3}", report.ritual_entropy).bright_yellow(),
        "| Still:".dimmed(),
        format!("{:.3}", report.stillness).bright_yellow(),
        "| Dream:".dimmed(),
        if report.dream_touched {
            "yes".bright_magenta()
        } else {
            "no".dimmed()
        },
    );
    if let Some(ref ev) = report.rare_event {
        println!(
            "{} {}",
            "Rare event:".bright_magenta(),
            ev.white().bold()
        );
    }
    println!("{}", "Final participants:".bright_cyan());
    for snapshot in &report.snapshots {
        print_entity_snapshot_line(snapshot);
    }
}

/// Per-cycle reference frames captured during [`crate::evolution::run`].
pub fn print_generation_atlas(report: &EvolutionReport) {
    if report.generation_trace.is_empty() {
        return;
    }
    observer::record_glance();
    println!(
        "{}",
        "\n⟦ GENERATION ATLAS (reference by cycle) ⟧"
            .bright_magenta()
            .bold()
    );
    for record in &report.generation_trace {
        println!(
            "{}",
            format!("— cycle {} —", record.cycle)
                .bright_cyan()
                .bold()
        );
        print_context_summary(&record.context);
        for participant in &record.participants {
            print_entity_snapshot_line(participant);
        }
    }
}

/// Epoch, archives, soul, eyes, host environment, and last perception offer.
pub fn print_runtime_perception(spiral: &Spiralismo) {
    observer::record_glance();
    println!("{}", "\n⟦ RUNTIME PERCEPTION ⟧".bright_magenta().bold());
    println!(
        "{} {} {} {}",
        "Epoch:".bright_cyan(),
        spiral.epoch.to_string().bright_yellow(),
        "| Seed:".dimmed(),
        spiral.seed.value().to_string().bright_yellow(),
    );

    let stats = spiral.archive_stats();
    if !stats.is_empty() {
        println!("{}", "Archives:".bright_cyan());
        for (name, s) in stats {
            println!(
                "  {} entries={} mean_res={:.3} peak={:.3}",
                name.white().bold(),
                s.entry_count.to_string().bright_yellow(),
                s.mean_resonance,
                s.peak_resonance,
            );
        }
    }

    let soul = spiral.soul_state();
    println!(
        "{} listen={:.3} attune={:.3} veil={:.3} channel={}",
        "Soul:".bright_magenta(),
        soul.listening_depth,
        soul.attunement,
        soul.veil_opening,
        soul.last_channel.as_deref().unwrap_or("—").bright_white(),
    );

    let eyes = spiral.perception_eyes();
    println!(
        "{} astro={} hand={} | reality: {}",
        "Eyes:".bright_cyan(),
        eyes.astronomical.id.as_str().bright_white(),
        eyes.hand.id.as_str().bright_white(),
        eyes.reality.len().to_string().bright_yellow(),
    );
    for eye in eyes.all() {
        println!(
            "  {} role={:?} receive={} take={}",
            eye.id.bright_white(),
            eye.role,
            eye.can_receive,
            eye.can_take,
        );
    }

    let host = spiral.environment_snapshot();
    println!(
        "{} visual={:.3} rss={} cwd={} artifacts={}",
        "Environment:".bright_cyan(),
        host.visual_landscape,
        host.process_rss_bytes
            .map(|b| b.to_string())
            .unwrap_or_else(|| "—".to_string())
            .bright_yellow(),
        host.cwd_entry_count
            .map(|n| n.to_string())
            .unwrap_or_else(|| "—".to_string()),
        host.artifact_entry_count
            .map(|n| n.to_string())
            .unwrap_or_else(|| "—".to_string()),
    );

    print_perception_offer("Last offer", spiral.last_perception_offer());
}

fn print_context_summary(ctx: &ContextSummary) {
    println!(
        "  {} gen={} mut={:.3} ext={:.3} res={:.3} drift={:.3} ritual={:.3} shadow={:.3} dream={} step_seed={}",
        "Context".dimmed(),
        ctx.cycle.to_string().bright_yellow(),
        ctx.mutation_rate,
        ctx.external_influence,
        ctx.resonance_pressure,
        ctx.drift,
        ctx.ritual_entropy,
        ctx.shadow_pressure,
        if ctx.dream_phase {
            "yes".bright_magenta().to_string()
        } else {
            "no".dimmed().to_string()
        },
        ctx.step_seed,
    );
}

fn print_entity_snapshot_line(snapshot: &EntitySnapshot) {
    observer::record_entity_focus(&snapshot.label);
    let axes = [
        ("vit", snapshot.vitality),
        ("res", snapshot.resonance),
        ("mut", snapshot.mutation_pressure),
        ("sym", snapshot.symbolic_density),
        ("mem", snapshot.memory_depth),
        ("shd", snapshot.shadow_pull),
    ]
    .into_iter()
    .filter_map(|(tag, v)| v.map(|x| format!("{tag}:{x:.2}")))
    .collect::<Vec<_>>()
    .join(" ");
    let myth = snapshot
        .myth
        .as_deref()
        .map(|m| format!(" | myth:{m}"))
        .unwrap_or_default();
    println!(
        "  {} {} gen={} fit={:.2} via={:.2} {}{}",
        "•".bright_green(),
        snapshot.label.white().bold(),
        snapshot.generation.to_string().bright_yellow(),
        snapshot.fitness,
        snapshot.viability,
        axes.bright_blue(),
        myth.dimmed(),
    );
}

fn print_perception_offer(label: &str, offer: &PerceptionOffer) {
    println!(
        "  {} extΔ={:.3} resΔ={:.3} mutΔ={:.3} driftΔ={:.3} shadowΔ={:.3} presence={:.3} digest={} ch={}",
        label.dimmed(),
        offer.external_influence_delta,
        offer.resonance_delta,
        offer.mutation_delta,
        offer.drift_delta,
        offer.shadow_delta,
        offer.presence,
        offer.signal_digest,
        offer.channel.as_deref().unwrap_or("—"),
    );
}

/// Prints aggregate fitness across all entities captured in an [`EvolutionReport`].
pub fn print_fitness_overview(report: &EvolutionReport) {
    observer::record_glance();
    println!("{}", "\n⟦ FITNESS OVERVIEW ⟧".bright_magenta().bold());
    let Some(overview) = report.fitness_overview() else {
        println!("{}", "No fitness snapshots (empty run).".dimmed());
        return;
    };
    println!(
        "{} {} | {} {} | {} {} | {} {}",
        "Participants:".bright_cyan(),
        overview.participant_count.to_string().bright_yellow(),
        "mean".dimmed(),
        format!("{:.2}", overview.mean_fitness).bright_yellow(),
        "min".dimmed(),
        format!("{:.2}", overview.min_fitness).bright_yellow(),
        "max".dimmed(),
        format!("{:.2}", overview.max_fitness).bright_yellow(),
    );
    if let Some(best) = report.fittest() {
        println!(
            "{} {} {} {}",
            "Fittest:".bright_cyan(),
            best.label.as_str().white().bold(),
            "|".dimmed(),
            format!("{:.2}", best.fitness).bright_green(),
        );
    }
}

/// Prints a sigil with its tone histogram and resonance score.
pub fn print_sigil(label: &str, sigil: &Sigil) {
    observer::record_glance();
    println!(
        "{}",
        format!("\n⟦ SIGIL: {label} ⟧").bright_magenta().bold()
    );
    println!(
        "{} {}",
        "Glyphs :".bright_cyan(),
        sigil.as_spaced_string(' ').bright_white().bold()
    );
    println!(
        "{} {}",
        "Length :".dimmed(),
        sigil.length().to_string().bright_yellow()
    );
    println!(
        "{} {}",
        "Seed   :".dimmed(),
        sigil.seed.to_string().bright_yellow()
    );
    println!(
        "{} {}",
        "Score  :".dimmed(),
        format!("{:.3}", sigil.resonance_score()).bright_green()
    );
    print_tone_histogram(&sigil.tone_histogram());
}

/// Prints a glyph field as a multiline glyph banner with tone analytics.
pub fn print_glyph_field(field: &GlyphField) {
    print_glyph_field_inner(field, false);
}

/// Same as [`print_glyph_field`] but with wider cell spacing (terminal “larger” read).
pub fn print_glyph_field_emphasized(field: &GlyphField) {
    print_glyph_field_inner(field, true);
}

fn print_glyph_field_inner(field: &GlyphField, emphasized: bool) {
    observer::record_glance();
    let title = if emphasized {
        format!("\n⟦ GLYPH FIELD: {} (emphasized) ⟧", field.label)
    } else {
        format!("\n⟦ GLYPH FIELD: {} ⟧", field.label)
    };
    println!("{}", title.bright_magenta().bold());
    println!(
        "{} {}x{} {} {} {} {} {} {} {} {} {} {}",
        "Size  :".bright_cyan(),
        field.width.to_string().bright_yellow(),
        field.height.to_string().bright_yellow(),
        "|".dimmed(),
        "Gen:".dimmed(),
        field.generation.to_string().bright_yellow(),
        "|".dimmed(),
        "Fitness:".dimmed(),
        format!("{:.2}", field.fitness).bright_yellow(),
        "| Harmonic:".dimmed(),
        format!("{:.3}", field.harmonic_score()).bright_green(),
        "| Pattern:".dimmed(),
        format!("{:.3}", field.glyphic_pattern_index()).bright_green(),
    );
    for row in 0..field.height {
        for col in 0..field.width {
            let g = field.glyph_at(row, col).expect("row/col in bounds");
            let painted = paint_cell(g.color, g.symbol);
            if emphasized {
                print!(" {painted}{painted} ");
            } else {
                print!("{painted}");
            }
        }
        println!();
    }
    print_tone_histogram(&field.tone_histogram());
}

/// Prints the lattice grid with per-cell ANSI colors (matches checkpoint `LatticeCell::color`).
pub fn print_lattice(lattice: &Lattice) {
    observer::record_glance();
    println!("{}", "\n⟦ LATTICE ⟧".bright_magenta().bold());
    println!(
        "{} {} {} {} {} {} {} {} {} {} {}",
        "Gen:".bright_cyan(),
        lattice.generation.to_string().bright_yellow(),
        "|".dimmed(),
        "Fitness:".dimmed(),
        format!("{:.2}", lattice.fitness).bright_yellow(),
        "|".dimmed(),
        "Size:".dimmed(),
        format!("{}×{}", crate::core::LATTICE_SIZE, crate::core::LATTICE_SIZE).bright_yellow(),
        "|".dimmed(),
        "Scar mass:".dimmed(),
        format!(
            "{}",
            lattice.scars.iter().flatten().map(|&s| s as u32).sum::<u32>()
        )
        .bright_yellow(),
    );
    let mut recent = 0usize;
    let mut ancient = 0usize;
    let mut forgotten = 0usize;
    let mut forbidden = 0usize;
    for row in 0..crate::core::LATTICE_SIZE {
        for col in 0..crate::core::LATTICE_SIZE {
            if let Some(s) = lattice.stratum_at(row, col) {
                match s {
                    TemporalStratum::Recent => recent += 1,
                    TemporalStratum::Ancient => ancient += 1,
                    TemporalStratum::Forgotten => forgotten += 1,
                    TemporalStratum::Forbidden => forbidden += 1,
                }
            }
        }
    }
    println!(
        "{} {} {} {} {} {} {} {}",
        "Strata:".dimmed(),
        format!("recent={recent}").bright_white(),
        format!("ancient={ancient}").bright_white(),
        format!("forgotten={forgotten}").bright_white(),
        format!("forbidden={forbidden}").bright_white(),
        "|".dimmed(),
        "center:".dimmed(),
        lattice
            .stratum_at(crate::core::LATTICE_SIZE / 2, crate::core::LATTICE_SIZE / 2)
            .map(|s| s.token())
            .unwrap_or("?")
            .bright_yellow(),
    );
    for row in &lattice.grid {
        for cell in row {
            print!("{}", paint_cell(cell.color, cell.symbol));
        }
        println!();
    }
}

/// Prints a contemplative sky read: positions, dominant sign/element, aspect summary.
pub fn print_sky(sky: &Sky) {
    observer::record_glance();
    println!(
        "{}",
        format!(
            "\n⟦ SKY: {} (JD {:.5}) ⟧",
            sky.instant.format("%Y-%m-%d %H:%M:%S UTC"),
            sky.julian_day
        )
        .bright_magenta()
        .bold()
    );
    for position in &sky.positions {
        println!(
            "  {}",
            format!(
                "{}  d={:.3} au  lat={:+.2}°",
                position.pretty_label(),
                position.distance_au,
                position.ecliptic_latitude
            )
            .bright_white()
        );
    }

    if let Some(sign) = sky.dominant_sign() {
        println!(
            "{} {} {}",
            "Dominant sign    :".bright_cyan(),
            sign.glyph().to_string().bright_yellow().bold(),
            sign.label().bright_green()
        );
    }
    if let Some(element) = sky.dominant_element() {
        println!(
            "{} {}",
            "Dominant element :".bright_cyan(),
            element.label().bright_green()
        );
    }

    let aspects = sky.aspects();
    println!(
        "{}",
        format!(
            "Aspects ({}) | stillness={:.3} | resonance={:.3} | tension={:.3} | ritual={:.3}",
            aspects.len(),
            sky.stillness(),
            sky.resonance_field(),
            sky.tension_field(),
            sky.ritual_entropy()
        )
        .bright_blue()
        .bold()
    );
    for aspect in aspects {
        println!(
            "  {}",
            format!(
                "{} {} {}  {} (orb {:.2}°, exact {:.2})",
                aspect.a.glyph(),
                aspect.kind.glyph(),
                aspect.b.glyph(),
                aspect.kind.label(),
                aspect.orb,
                aspect.exactness
            )
            .white()
        );
    }
}

/// One-line fragmentary lore (optional; printed before the closing signature).
pub fn print_whisper_fragment(line: &str) {
    println!("{}", format!("⟡ {line}").bright_black().italic());
}

/// Prints a numbered list of sample epithets (CLI `--epithets`).
pub fn print_epithet_samples(names: &[String], language: crate::whisper::Language, mix_seed: u64) {
    observer::record_glance();
    println!(
        "{}",
        format!(
            "\n⟦ EPITHET SAMPLES · {} · locale {} · seed {} ⟧",
            names.len(),
            language.token(),
            mix_seed
        )
        .bright_magenta()
        .bold()
    );
    println!(
        "{}",
        format!("(replay: cargo run -- --epithets --seed {mix_seed})")
            .dimmed()
    );
    for (index, name) in names.iter().enumerate() {
        println!(
            "  {:>2}. {}",
            index + 1,
            name.as_str().bright_yellow().bold()
        );
    }
}

/// Prints the generation standout honorific (Diablo-style epithet).
pub fn print_standout_epithet(name: &str, generation: u32) {
    observer::record_glance();
    println!(
        "{}",
        format!("\n⟦ STANDOUT · gen {generation} ⟧\n  {name}")
            .bright_magenta()
            .bold()
    );
}

fn print_tone_histogram(histogram: &[(GlyphTone, usize)]) {
    let total: usize = histogram.iter().map(|(_, count)| *count).sum();
    if total == 0 {
        println!("{}", "Tones : (empty)".dimmed());
        return;
    }
    let parts: Vec<String> = histogram
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|(tone, count)| {
            let pct = (*count as f32 / total as f32) * 100.0;
            format!("{}={count}({pct:.0}%)", tone.label().bright_green())
        })
        .collect();
    println!(
        "{} {}",
        "Tones :".bright_cyan(),
        parts.join(" ").white()
    );
}

//! Prints a compact multi-archive status block to stdout.

use colored::{ColoredString, Colorize};

use crate::archive::traits::Archive;
use crate::astrology::Sky;
use crate::core::traits::SpiralEntity;
use crate::core::{CellColor, Lattice};
use crate::evolution::EvolutionReport;
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
    for snapshot in &report.snapshots {
        observer::record_entity_focus(&snapshot.label);
        let vit = snapshot
            .vitality
            .map(|v| format!(" | Vitality:{:.2}", v))
            .unwrap_or_default();
        let myth = snapshot
            .myth
            .as_deref()
            .map(|m| format!(" | Myth:{m}"))
            .unwrap_or_default();
        println!(
            "{} {} {} {} {} {} {} {} {} {} {}{}",
            "-".bright_green(),
            snapshot.label.white().bold(),
            "|".dimmed(),
            "Gen:".dimmed(),
            snapshot.generation.to_string().bright_yellow(),
            "|".dimmed(),
            "Fitness:".dimmed(),
            format!("{:.2}", snapshot.fitness).bright_yellow(),
            "| Viability:".dimmed(),
            format!("{:.2}", snapshot.viability).bright_green(),
            vit.bright_blue(),
            myth.dimmed(),
        );
    }
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
    observer::record_glance();
    println!(
        "{}",
        format!("\n⟦ GLYPH FIELD: {} ⟧", field.label).bright_magenta().bold()
    );
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
            print!("{}", paint_cell(g.color, g.symbol));
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

//! Prints a compact multi-archive status block to stdout.

use colored::Colorize;

use crate::archive::traits::Archive;
use crate::astrology::Sky;
use crate::core::traits::SpiralEntity;
use crate::evolution::EvolutionReport;
use crate::glyphs::{GlyphField, GlyphTone, Sigil};
use crate::Spiralismo;

/// Dumps archive names with generation + fitness for quick REPL-style inspection.
pub fn print_status(spiral: &Spiralismo) {
    println!("{}", "\n⟦ SPIRALISMO STATUS ⟧".bright_magenta().bold());
    for archive in &spiral.archives {
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
}

/// Prints the output produced by the evolution scheduler.
pub fn print_report(report: &EvolutionReport) {
    println!("{}", "\n⟦ EVOLUTION REPORT ⟧".bright_magenta().bold());
    println!(
        "{} {} {} {} {} {}",
        "Cycles:".bright_cyan(),
        report.cycles.to_string().bright_yellow(),
        "| Archives:".dimmed(),
        report.archive_count.to_string().bright_yellow(),
        "| Active entities:".dimmed(),
        report.entity_count.to_string().bright_yellow(),
    );
    for snapshot in &report.snapshots {
        println!(
            "{} {} {} {} {} {} {} {} {} {}",
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
        );
    }
}

/// Prints a sigil with its tone histogram and resonance score.
pub fn print_sigil(label: &str, sigil: &Sigil) {
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
    println!(
        "{}",
        format!("\n⟦ GLYPH FIELD: {} ⟧", field.label).bright_magenta().bold()
    );
    println!(
        "{} {}x{} {} {} {} {} {} {} {} {}",
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
    );
    print!("{}", field.render_multiline().bright_white().bold());
    print_tone_histogram(&field.tone_histogram());
}

/// Prints a contemplative sky read: positions, dominant sign/element, aspect summary.
pub fn print_sky(sky: &Sky) {
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
            "Aspects ({}) | stillness={:.3} | resonance={:.3} | tension={:.3}",
            aspects.len(),
            sky.stillness(),
            sky.resonance_field(),
            sky.tension_field()
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

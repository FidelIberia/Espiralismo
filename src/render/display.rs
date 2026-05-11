//! Prints a compact multi-archive status block to stdout.

use crate::archive::traits::Archive;
use crate::astrology::Sky;
use crate::core::traits::SpiralEntity;
use crate::evolution::EvolutionReport;
use crate::glyphs::{GlyphField, GlyphTone, Sigil};
use crate::Spiralismo;

/// Dumps archive names with generation + fitness for quick REPL-style inspection.
pub fn print_status(spiral: &Spiralismo) {
    println!("\n⟦ SPIRALISMO STATUS ⟧");
    for archive in &spiral.archives {
        println!(
            "Archive: {} | Gen: {} | Fitness: {:.2}",
            Archive::name(archive.as_ref()),
            SpiralEntity::generation(archive.as_ref()),
            SpiralEntity::fitness(archive.as_ref()),
        );
    }

    if !spiral.active_lattices.is_empty() {
        println!("\nActive lattices: {}", spiral.active_lattices.len());
        for (index, lattice) in spiral.active_lattices.iter().enumerate() {
            println!(
                "Lattice #{index} | Gen: {} | Fitness: {:.2}",
                lattice.generation(),
                lattice.fitness(),
            );
        }
    }
}

/// Prints the output produced by the evolution scheduler.
pub fn print_report(report: &EvolutionReport) {
    println!("\n⟦ EVOLUTION REPORT ⟧");
    println!(
        "Cycles: {} | Archives: {} | Active entities: {}",
        report.cycles, report.archive_count, report.entity_count
    );
    for snapshot in &report.snapshots {
        println!(
            "- {} | Gen: {} | Fitness: {:.2} | Viability: {:.2}",
            snapshot.label, snapshot.generation, snapshot.fitness, snapshot.viability
        );
    }
}

/// Prints a sigil with its tone histogram and resonance score.
pub fn print_sigil(label: &str, sigil: &Sigil) {
    println!("\n⟦ SIGIL: {label} ⟧");
    println!("Glyphs : {}", sigil.as_spaced_string(' '));
    println!("Length : {}", sigil.length());
    println!("Seed   : {}", sigil.seed);
    println!("Score  : {:.3}", sigil.resonance_score());
    print_tone_histogram(&sigil.tone_histogram());
}

/// Prints a glyph field as a multiline glyph banner with tone analytics.
pub fn print_glyph_field(field: &GlyphField) {
    println!("\n⟦ GLYPH FIELD: {} ⟧", field.label);
    println!(
        "Size  : {}x{} | Gen: {} | Fitness: {:.2} | Harmonic: {:.3}",
        field.width,
        field.height,
        field.generation,
        field.fitness,
        field.harmonic_score()
    );
    print!("{}", field.render_multiline());
    print_tone_histogram(&field.tone_histogram());
}

/// Prints a contemplative sky read: positions, dominant sign/element, aspect summary.
pub fn print_sky(sky: &Sky) {
    println!("\n⟦ SKY: {} (JD {:.5}) ⟧", sky.instant.format("%Y-%m-%d %H:%M:%S UTC"), sky.julian_day);
    for position in &sky.positions {
        println!(
            "  {}  d={:.3} au  lat={:+.2}°",
            position.pretty_label(),
            position.distance_au,
            position.ecliptic_latitude
        );
    }

    if let Some(sign) = sky.dominant_sign() {
        println!("Dominant sign    : {} {}", sign.glyph(), sign.label());
    }
    if let Some(element) = sky.dominant_element() {
        println!("Dominant element : {}", element.label());
    }

    let aspects = sky.aspects();
    println!(
        "Aspects ({}) | stillness={:.3} | resonance={:.3} | tension={:.3}",
        aspects.len(),
        sky.stillness(),
        sky.resonance_field(),
        sky.tension_field()
    );
    for aspect in aspects {
        println!(
            "  {} {} {}  {} (orb {:.2}°, exact {:.2})",
            aspect.a.glyph(),
            aspect.kind.glyph(),
            aspect.b.glyph(),
            aspect.kind.label(),
            aspect.orb,
            aspect.exactness
        );
    }
}

fn print_tone_histogram(histogram: &[(GlyphTone, usize)]) {
    let total: usize = histogram.iter().map(|(_, count)| *count).sum();
    if total == 0 {
        println!("Tones : (empty)");
        return;
    }
    let parts: Vec<String> = histogram
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|(tone, count)| {
            let pct = (*count as f32 / total as f32) * 100.0;
            format!("{}={count}({pct:.0}%)", tone.label())
        })
        .collect();
    println!("Tones : {}", parts.join(" "));
}

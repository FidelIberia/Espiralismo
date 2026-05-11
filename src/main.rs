use spiralismo::archive::ResonanceEngine;
use spiralismo::persistence::JsonlPersistence;
use spiralismo::render::display;
use spiralismo::{EvolutionContext, GlyphField, GlyphGenerator, Lattice, Spiralismo};
use std::env;

fn main() {
    println!("𓂀 SPIRALISMO v0.5.0 — Espiralismo Framework 𓂀\n");

    let mut spiral = Spiralismo::new();
    spiral.register_lattice(Box::new(Lattice::new(spiral.seed.value().rotate_left(5))));

    // Glyph generation: seed a 5x3 field anchored to the framework seed
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

    // Record a resonant moment alongside a freshly minted sigil
    if let Some(engine) = spiral.archive_as_mut::<ResonanceEngine>("ResonanceEngine") {
        engine.record_resonance(
            "Two echoes recognized each other in the Atheneum".to_string(),
            0.97,
        );
    }
    let opening_sigil = spiral
        .record_sigil_in_archive("ResonanceEngine", 11, 0.81)
        .expect("ResonanceEngine should accept the sigil");
    display::print_sigil("opening_invocation", &opening_sigil);

    // Quiet room: read the present sky and let it shape the policy
    let (policy, sky) = spiral.policy_aligned_with_present(8);
    display::print_sky(&sky);
    let report = spiral.evolve_with_policy(&policy);

    display::print_status(&spiral);
    display::print_report(&report);

    // After evolution, surface the current state of the registered glyph field
    if let Some(field) = spiral.active_as::<GlyphField>() {
        display::print_glyph_field(field);
    }

    if let Some(snapshot_dir) = snapshot_dir_from_args() {
        match JsonlPersistence::new(&snapshot_dir) {
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

fn snapshot_dir_from_args() -> Option<String> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--snapshot-dir" {
            return args.next();
        }
    }
    None
}

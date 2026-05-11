use chrono::TimeZone;
use spiralismo::archive::{Archive, MemoryArchive, ResonanceEngine};
use spiralismo::astrology::{angular_separation, match_aspect, AspectKind, Sky};
use spiralismo::core::SpiralEntity;
use spiralismo::evolution::context_for_cycle;
use spiralismo::{
    ArchiveEntry, EntitySnapshot, EvolutionContext, EvolutionPolicy, EvolutionReport, GlyphField,
    GlyphGenerator, GlyphTone, JsonlPersistence, Lattice, Planet, Seed, Sigil, Spiralismo,
    SpiralismoSnapshot, ZodiacSign,
};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn assert_close(left: f32, right: f32) {
    assert!(
        (left - right).abs() < 0.000_001,
        "expected {left} to be approximately {right}"
    );
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for tests")
        .as_nanos();
    std::env::temp_dir().join(format!("spiralismo_{label}_{nanos}"))
}

#[test]
fn evolution_context_normalization_clamps_values() {
    let context = EvolutionContext::for_generation(7)
        .with_mutation_rate(1.8)
        .with_external_influence(-0.2)
        .with_resonance_pressure(2.5)
        .with_drift(-0.7)
        .with_step_seed(99)
        .normalized();

    assert_eq!(context.generation, 7);
    assert_eq!(context.mutation_rate, 1.0);
    assert_eq!(context.external_influence, 0.0);
    assert_eq!(context.resonance_pressure, 1.0);
    assert_eq!(context.drift, 0.0);
    assert_eq!(context.step_seed, 99);
}

#[test]
fn seed_helpers_are_deterministic() {
    let parsed = Seed::from_binary_hash("101101").expect("binary hash should parse");
    assert_eq!(parsed.value(), 45);

    let bits = parsed.as_bits_width(6);
    assert_eq!(bits, vec![1, 0, 1, 1, 0, 1]);

    let rotated = parsed.rotate_left(2);
    assert_ne!(rotated.value(), parsed.value());

    let mixed_a = parsed.mix(Seed::from_value(123));
    let mixed_b = parsed.mix(Seed::from_value(123));
    assert_eq!(mixed_a, mixed_b);
}

#[test]
fn lattice_wave_is_deterministic_for_same_context() {
    let context = EvolutionContext::for_generation(1)
        .with_mutation_rate(0.35)
        .with_resonance_pressure(0.75)
        .with_step_seed(4242)
        .normalized();

    let mut left = Lattice::new(45);
    let mut right = left.clone();

    left.evolve(&context);
    right.evolve(&context);

    assert_eq!(left.grid, right.grid);
    assert_eq!(left.generation, 1);
    assert_eq!(left.fitness, right.fitness);
    assert!(left.fitness > 0.0);
}

#[test]
fn resonance_engine_exposes_archive_helpers() {
    let mut engine = ResonanceEngine::new();
    engine.record_content("first whisper", 0.4);
    engine.record_content("second echo", 0.9);

    assert_eq!(engine.entry_count(), 2);
    assert!(!engine.is_empty());
    assert!(engine.recall("echo").is_some());
    assert_eq!(engine.latest().expect("latest entry").content, "second echo");
    assert_eq!(engine.by_min_resonance(0.8).len(), 1);
    assert_eq!(
        engine
            .strongest()
            .expect("strongest entry should exist")
            .resonance,
        0.9
    );

    let stats = engine.stats();
    assert_eq!(stats.entry_count, 2);
    assert!((stats.mean_resonance - 0.65).abs() < 0.0001);
    assert!((stats.peak_resonance - 0.9).abs() < 0.0001);
}

#[test]
fn spiralismo_policy_evolution_produces_report_and_snapshot() {
    let mut spiral = Spiralismo::new_with_seed(Seed::from_value(2026));
    spiral.register_lattice(Box::new(Lattice::new(spiral.seed.value().rotate_left(3))));
    assert!(spiral.record_in_archive("ResonanceEngine", "harmonic checkpoint", 0.88));
    assert!(!spiral.record_in_archive("MissingArchive", "ignored", 0.5));

    let policy = EvolutionPolicy::default()
        .with_cycles(3)
        .with_seed(777)
        .with_mutation_rate(0.32)
        .with_resonance_pressure(0.71);
    let report = spiral.evolve_with_policy(&policy);

    assert_eq!(report.cycles, 3);
    assert_eq!(report.archive_count, 4);
    assert_eq!(report.entity_count, 1);
    assert_eq!(report.snapshots.len(), 5);
    assert_eq!(spiral.epoch, 3);
    assert!(spiral.last_report().is_some());

    let engine = spiral
        .archive_as::<ResonanceEngine>("ResonanceEngine")
        .expect("resonance engine should be present");
    assert_eq!(engine.entry_count(), 1);

    let snapshot = spiral.snapshot();
    assert_eq!(snapshot.seed_value, 2026);
    assert_eq!(snapshot.epoch, 3);
    assert_eq!(snapshot.archives, 4);
    assert_eq!(snapshot.active_lattices, 1);
    assert_eq!(snapshot.entities.len(), 5);

    let stats = spiral.archive_stats();
    assert_eq!(stats.len(), 4);
}

#[test]
fn context_for_cycle_is_deterministic_across_seed_matrix() {
    let seeds = [0_u64, 1, 7, 42, 101101, 2026, u64::MAX - 3];

    for seed in seeds {
        let policy = EvolutionPolicy::default()
            .with_seed(seed)
            .with_drift(0.33)
            .with_mutation_rate(0.44)
            .with_external_influence(0.58)
            .with_resonance_pressure(0.66);

        for cycle in 0..12 {
            let left = context_for_cycle(&policy, cycle);
            let right = context_for_cycle(&policy, cycle);

            assert_eq!(left.generation, cycle);
            assert_eq!(right.generation, cycle);
            assert_eq!(left.step_seed, right.step_seed);
            assert_close(left.mutation_rate, right.mutation_rate);
            assert_close(left.external_influence, right.external_influence);
            assert_close(left.resonance_pressure, right.resonance_pressure);
            assert_close(left.drift, right.drift);

            for value in [
                left.mutation_rate,
                left.external_influence,
                left.resonance_pressure,
                left.drift,
            ] {
                assert!(
                    (0.0..=1.0).contains(&value),
                    "expected normalized value in [0, 1], got {value}"
                );
            }
        }
    }
}

#[test]
fn evolution_reports_remain_deterministic_for_multiple_policy_seeds() {
    for seed in [3_u64, 11, 97, 777, 2026] {
        let policy = EvolutionPolicy::default()
            .with_cycles(4)
            .with_seed(seed)
            .with_mutation_rate(0.31)
            .with_external_influence(0.63)
            .with_resonance_pressure(0.74)
            .with_drift(0.22);

        let mut left = Spiralismo::new_with_seed(Seed::from_value(seed.rotate_left(1)));
        let mut right = Spiralismo::new_with_seed(Seed::from_value(seed.rotate_left(1)));
        left.register_lattice(Box::new(Lattice::new(seed ^ 0xA5A5)));
        right.register_lattice(Box::new(Lattice::new(seed ^ 0xA5A5)));
        assert!(left.record_in_archive("ResonanceEngine", "matrix checkpoint", 0.84));
        assert!(right.record_in_archive("ResonanceEngine", "matrix checkpoint", 0.84));

        let left_report = left.evolve_with_policy(&policy);
        let right_report = right.evolve_with_policy(&policy);

        assert_eq!(left_report.cycles, right_report.cycles);
        assert_eq!(left_report.archive_count, right_report.archive_count);
        assert_eq!(left_report.entity_count, right_report.entity_count);
        assert_eq!(left_report.snapshots.len(), right_report.snapshots.len());
        for (left_snapshot, right_snapshot) in
            left_report.snapshots.iter().zip(right_report.snapshots.iter())
        {
            assert_eq!(left_snapshot.label, right_snapshot.label);
            assert_eq!(left_snapshot.generation, right_snapshot.generation);
            assert_close(left_snapshot.fitness, right_snapshot.fitness);
            assert_close(left_snapshot.viability, right_snapshot.viability);
        }

        let left_snapshot = left.snapshot();
        let right_snapshot = right.snapshot();
        assert_eq!(left_snapshot.seed_value, right_snapshot.seed_value);
        assert_eq!(left_snapshot.epoch, right_snapshot.epoch);
        assert_eq!(left_snapshot.archives, right_snapshot.archives);
        assert_eq!(left_snapshot.active_lattices, right_snapshot.active_lattices);
        assert_eq!(left_snapshot.entities.len(), right_snapshot.entities.len());
    }
}

#[test]
fn serialization_roundtrip_keeps_reports_and_snapshots_consistent() {
    let mut spiral = Spiralismo::new_with_seed(Seed::from_value(3030));
    spiral.register_lattice(Box::new(Lattice::new(0xF0F0)));
    assert!(spiral.record_in_archive("ResonanceEngine", "roundtrip start", 0.73));
    assert!(spiral.record_in_archive("Living Memory", "roundtrip memory", 0.42));

    let report = spiral.evolve_with_policy(
        &EvolutionPolicy::default()
            .with_cycles(2)
            .with_seed(5150)
            .with_mutation_rate(0.27),
    );
    let snapshot = spiral.snapshot();
    let sample_entity = EntitySnapshot {
        label: "serialization_probe".to_string(),
        generation: 9,
        fitness: 12.5,
        viability: 0.75,
    };

    let report_json = serde_json::to_string(&report).expect("report should serialize");
    let snapshot_json = serde_json::to_string(&snapshot).expect("snapshot should serialize");
    let entity_json = serde_json::to_string(&sample_entity).expect("entity should serialize");

    let roundtrip_report: EvolutionReport =
        serde_json::from_str(&report_json).expect("report should deserialize");
    let roundtrip_snapshot: SpiralismoSnapshot =
        serde_json::from_str(&snapshot_json).expect("snapshot should deserialize");
    let roundtrip_entity: EntitySnapshot =
        serde_json::from_str(&entity_json).expect("entity should deserialize");

    assert_eq!(roundtrip_report.cycles, report.cycles);
    assert_eq!(roundtrip_report.archive_count, report.archive_count);
    assert_eq!(roundtrip_report.entity_count, report.entity_count);
    assert_eq!(roundtrip_report.snapshots.len(), report.snapshots.len());
    for (left_snapshot, right_snapshot) in report.snapshots.iter().zip(roundtrip_report.snapshots.iter()) {
        assert_eq!(left_snapshot.label, right_snapshot.label);
        assert_eq!(left_snapshot.generation, right_snapshot.generation);
        assert_close(left_snapshot.fitness, right_snapshot.fitness);
        assert_close(left_snapshot.viability, right_snapshot.viability);
    }

    assert_eq!(roundtrip_snapshot.seed_value, snapshot.seed_value);
    assert_eq!(roundtrip_snapshot.epoch, snapshot.epoch);
    assert_eq!(roundtrip_snapshot.archives, snapshot.archives);
    assert_eq!(roundtrip_snapshot.active_lattices, snapshot.active_lattices);
    assert_eq!(roundtrip_snapshot.entities.len(), snapshot.entities.len());

    assert_eq!(roundtrip_entity.label, sample_entity.label);
    assert_eq!(roundtrip_entity.generation, sample_entity.generation);
    assert_close(roundtrip_entity.fitness, sample_entity.fitness);
    assert_close(roundtrip_entity.viability, sample_entity.viability);
}

#[test]
fn archive_mut_and_downcasting_remain_consistent_after_repeated_mutation() {
    let mut spiral = Spiralismo::new();

    assert!(spiral.archive_as::<ResonanceEngine>("ResonanceEngine").is_some());
    assert!(spiral.archive_as::<MemoryArchive>("ResonanceEngine").is_none());
    assert!(spiral.archive_mut("MissingArchive").is_none());

    for idx in 0..5 {
        {
            let archive = spiral
                .archive_mut("ResonanceEngine")
                .expect("archive lookup should succeed");
            archive.record(ArchiveEntry::now(
                format!("dyn mutation {idx}"),
                0.20 + idx as f32 * 0.1,
            ));
        }
        {
            let engine = spiral
                .archive_as_mut::<ResonanceEngine>("ResonanceEngine")
                .expect("downcast should succeed");
            engine.record_resonance(format!("typed mutation {idx}"), 0.45 + idx as f32 * 0.1);
            assert!(engine.recall(&format!("typed mutation {idx}")).is_some());
        }
    }

    let engine = spiral
        .archive_as::<ResonanceEngine>("ResonanceEngine")
        .expect("engine should still be available");
    assert_eq!(engine.entry_count(), 10);
    assert!(engine.recall("dyn mutation 4").is_some());
    assert!(engine.recall("typed mutation 4").is_some());

    let dyn_archive = spiral
        .archive("ResonanceEngine")
        .expect("dyn archive should still be available");
    assert_eq!(dyn_archive.entry_count(), engine.entry_count());
}

#[test]
fn glyph_generator_is_deterministic_for_same_seed_and_context() {
    let context = EvolutionContext::for_generation(2)
        .with_mutation_rate(0.4)
        .with_resonance_pressure(0.7)
        .with_external_influence(0.55)
        .with_drift(0.2)
        .with_step_seed(2026)
        .normalized();
    let generator = GlyphGenerator::new(101101);

    let left_sigil = generator.generate_sigil(16, &context);
    let right_sigil = generator.generate_sigil(16, &context);
    assert_eq!(left_sigil.length(), 16);
    assert_eq!(left_sigil.glyphs, right_sigil.glyphs);
    assert_eq!(left_sigil.tones, right_sigil.tones);
    assert_eq!(left_sigil.seed, right_sigil.seed);

    let left_field = generator.generate_field(4, 3, &context);
    let right_field = generator.generate_field(4, 3, &context);
    assert_eq!(left_field.width, 4);
    assert_eq!(left_field.height, 3);
    assert_eq!(left_field.cells.len(), 12);
    let left_chars = left_field.as_chars();
    let right_chars = right_field.as_chars();
    assert_eq!(left_chars, right_chars);
}

#[test]
fn glyph_generator_tone_weights_respond_to_context() {
    let generator = GlyphGenerator::new(7);
    let calm = EvolutionContext::for_generation(0)
        .with_mutation_rate(0.05)
        .with_resonance_pressure(0.95)
        .with_external_influence(0.8)
        .with_drift(0.05)
        .normalized();
    let chaotic = EvolutionContext::for_generation(0)
        .with_mutation_rate(0.95)
        .with_resonance_pressure(0.1)
        .with_external_influence(0.1)
        .with_drift(0.95)
        .normalized();

    let calm_weights = generator.tone_weights(&calm);
    let chaotic_weights = generator.tone_weights(&chaotic);

    assert!(
        calm_weights.fraction(GlyphTone::Luminous) > chaotic_weights.fraction(GlyphTone::Luminous),
        "calm context should favor luminous tones"
    );
    assert!(
        chaotic_weights.fraction(GlyphTone::Spark) > calm_weights.fraction(GlyphTone::Spark),
        "chaotic context should favor spark tones"
    );
    assert!(
        chaotic_weights.fraction(GlyphTone::Shadow) > calm_weights.fraction(GlyphTone::Shadow),
        "chaotic context should favor shadow tones"
    );
}

#[test]
fn glyph_field_evolves_and_changes_state_deterministically() {
    let context = EvolutionContext::for_generation(0)
        .with_mutation_rate(0.4)
        .with_resonance_pressure(0.65)
        .with_drift(0.2)
        .with_step_seed(404)
        .normalized();
    let generator = GlyphGenerator::new(2026);
    let mut field_a = GlyphField::from_generator(&generator, 5, 3, &context);
    let mut field_b = GlyphField::from_generator(&generator, 5, 3, &context);
    assert_eq!(field_a.as_chars(), field_b.as_chars());

    let before = field_a.as_chars();
    field_a.evolve(&context);
    field_b.evolve(&context);

    assert_eq!(field_a.generation, 1);
    assert_eq!(field_a.as_chars(), field_b.as_chars());
    assert_ne!(field_a.as_chars(), before, "evolve should mutate the field");
    assert!(field_a.fitness > 0.0);

    let histogram = field_a.tone_histogram();
    let total: usize = histogram.iter().map(|(_, count)| *count).sum();
    assert_eq!(total, field_a.cells.len());

    let harmonic = field_a.harmonic_score();
    assert!((0.0..=1.0).contains(&harmonic));
}

#[test]
fn spiralismo_integrates_glyph_fields_and_sigils() {
    let mut spiral = Spiralismo::new_with_seed(Seed::from_value(424242));
    let generator = spiral.glyph_generator();
    let context = EvolutionContext::for_generation(0)
        .with_mutation_rate(0.32)
        .with_resonance_pressure(0.7)
        .with_step_seed(spiral.seed.value())
        .normalized();
    let field = GlyphField::from_generator(&generator, 4, 4, &context).with_label("integration");
    spiral.register_glyph_field(field);

    assert!(spiral.active_as::<GlyphField>().is_some());

    let sigil = spiral
        .record_sigil_in_archive("ResonanceEngine", 9, 0.66)
        .expect("ResonanceEngine should record the sigil");
    assert_eq!(sigil.length(), 9);
    assert!((0.0..=1.0).contains(&sigil.resonance_score()));

    let policy = EvolutionPolicy::default()
        .with_cycles(2)
        .with_seed(909)
        .with_mutation_rate(0.31)
        .with_resonance_pressure(0.72);
    let report = spiral.evolve_with_policy(&policy);
    assert_eq!(report.entity_count, 1);

    let field = spiral
        .active_as::<GlyphField>()
        .expect("glyph field should still be registered");
    assert_eq!(field.generation, 2);
    assert_eq!(field.width, 4);
    assert_eq!(field.height, 4);
    assert!(field.fitness > 0.0);

    let sigil_entry = spiral
        .archive("ResonanceEngine")
        .and_then(|archive| archive.latest())
        .expect("resonance archive should contain the recorded sigil");
    let _: char = sigil_entry
        .content
        .chars()
        .next()
        .expect("sigil content should not be empty");
}

#[test]
fn sigil_serialization_roundtrip_preserves_glyphs_and_tones() {
    let context = EvolutionContext::for_generation(3)
        .with_mutation_rate(0.5)
        .with_resonance_pressure(0.6)
        .with_step_seed(7777)
        .normalized();
    let generator = GlyphGenerator::new(31415);
    let sigil = generator.generate_sigil(12, &context);

    let json = serde_json::to_string(&sigil).expect("sigil should serialize");
    let restored: Sigil = serde_json::from_str(&json).expect("sigil should deserialize");

    assert_eq!(restored.glyphs, sigil.glyphs);
    assert_eq!(restored.tones, sigil.tones);
    assert_eq!(restored.seed, sigil.seed);
    assert_close(restored.resonance_score(), sigil.resonance_score());
}

#[test]
fn astrology_julian_day_matches_j2000_reference() {
    let j2000 = chrono::Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
    let jd = spiralismo::astrology::julian_day(j2000);
    assert!((jd - 2_451_545.0).abs() < 1e-6, "expected J2000 = 2451545.0, got {jd}");
}

#[test]
fn astrology_sky_places_sun_in_capricorn_at_j2000() {
    let j2000 = chrono::Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
    let sky = Sky::at(j2000);
    let sun = sky.position(Planet::Sun).expect("sun position should exist");
    // Sun's ecliptic longitude at J2000 is ~280.5° (Capricorn 10°)
    assert!(
        sun.ecliptic_longitude > 270.0 && sun.ecliptic_longitude < 285.0,
        "expected Sun longitude near 280°, got {}",
        sun.ecliptic_longitude
    );
    assert_eq!(sun.sign, ZodiacSign::Capricorn);
}

#[test]
fn astrology_zodiac_longitude_mapping_covers_all_signs() {
    for (idx, sign) in ZodiacSign::ALL.iter().enumerate() {
        let middle_longitude = idx as f64 * 30.0 + 15.0;
        assert_eq!(ZodiacSign::from_longitude(middle_longitude), *sign);
    }
    assert_eq!(ZodiacSign::from_longitude(-15.0), ZodiacSign::Pisces);
    assert_eq!(ZodiacSign::from_longitude(720.0), ZodiacSign::Aries);
}

#[test]
fn astrology_aspect_detection_matches_classical_angles() {
    assert_eq!(angular_separation(10.0, 350.0), 20.0);
    assert_eq!(angular_separation(0.0, 180.0), 180.0);
    let trine = match_aspect(120.5).expect("near 120° should match trine");
    assert_eq!(trine.0, AspectKind::Trine);
    assert!(trine.1 < 1.0);
    assert!(match_aspect(45.0).is_none(), "semi-square is not a Ptolemaic aspect");
}

#[test]
fn astrology_sky_is_deterministic_for_fixed_instant() {
    let when = chrono::Utc.with_ymd_and_hms(2026, 5, 11, 12, 0, 0).unwrap();
    let sky_left = Sky::at(when);
    let sky_right = Sky::at(when);
    assert_eq!(sky_left.positions.len(), sky_right.positions.len());
    for (l, r) in sky_left.positions.iter().zip(sky_right.positions.iter()) {
        assert_eq!(l.planet, r.planet);
        assert_close_64(l.ecliptic_longitude, r.ecliptic_longitude);
        assert_close_64(l.ecliptic_latitude, r.ecliptic_latitude);
        assert_close_64(l.distance_au, r.distance_au);
    }
    assert_close(sky_left.stillness(), sky_right.stillness());
}

#[test]
fn astrology_modulation_pulls_toward_resonance_when_sky_is_still() {
    // Fabricate an "empty" sky scenario by picking an instant and verifying the modulation
    // doesn't blow up; checks invariants only (clamped fields).
    let when = chrono::Utc.with_ymd_and_hms(2024, 8, 15, 3, 30, 0).unwrap();
    let sky = Sky::at(when);
    let base = EvolutionContext::for_generation(7)
        .with_mutation_rate(0.3)
        .with_resonance_pressure(0.4)
        .with_external_influence(0.5)
        .with_drift(0.2)
        .normalized();
    let modulated = sky.modulate(base.clone());
    assert!((0.0..=1.0).contains(&modulated.mutation_rate));
    assert!((0.0..=1.0).contains(&modulated.resonance_pressure));
    assert!((0.0..=1.0).contains(&modulated.external_influence));
    assert!((0.0..=1.0).contains(&modulated.drift));
    assert_eq!(modulated.generation, base.generation);

    let stillness = sky.stillness();
    if stillness > 0.7 {
        // High-stillness sky should at least amplify external influence relative to base.
        assert!(
            modulated.external_influence >= base.external_influence,
            "stillness should not reduce external influence (was {}, now {})",
            base.external_influence,
            modulated.external_influence
        );
    }
}

#[test]
fn spiralismo_evolves_aligned_with_present_without_panicking() {
    let mut spiral = Spiralismo::new_with_seed(Seed::from_value(1111));
    let (policy, sky) = spiral.policy_aligned_with_present(2);
    assert_eq!(policy.cycles, 2);
    assert!(policy.mutation_rate >= 0.0 && policy.mutation_rate <= 1.0);
    assert!(policy.external_influence >= 0.0 && policy.external_influence <= 1.0);
    assert!(!sky.positions.is_empty());

    let captured = spiral.evolve_aligned_with_present();
    assert_eq!(captured.positions.len(), sky.positions.len());
    assert_eq!(spiral.epoch, 1);
}

fn assert_close_64(left: f64, right: f64) {
    assert!(
        (left - right).abs() < 1e-9,
        "expected {left} to be approximately {right}"
    );
}

#[test]
fn jsonl_persistence_roundtrip_keeps_runtime_state_and_archive_stats() {
    let mut spiral = Spiralismo::new_with_seed(Seed::from_value(8080));
    spiral.register_lattice(Box::new(Lattice::new(0xA0A0)));
    assert!(spiral.record_in_archive("ResonanceEngine", "persisted resonance", 0.91));
    assert!(spiral.record_in_archive("Living Memory", "persisted memory", 0.37));

    let policy = EvolutionPolicy::default()
        .with_cycles(3)
        .with_seed(404)
        .with_mutation_rate(0.29)
        .with_resonance_pressure(0.77);
    let report = spiral.evolve_with_policy(&policy);
    let snapshot = spiral.snapshot();

    let dir = unique_temp_dir("persistence");
    let store = JsonlPersistence::new(&dir).expect("persistence directory should initialize");
    store
        .append_report(&report)
        .expect("report should persist as jsonl");
    store
        .append_snapshot(&snapshot)
        .expect("snapshot should persist as jsonl");
    store
        .append_runtime_state(&spiral)
        .expect("runtime state should persist as jsonl");

    let reports = store.load_reports().expect("reports should load");
    let snapshots = store.load_snapshots().expect("snapshots should load");
    let runtime_states = store
        .load_runtime_states()
        .expect("runtime states should load");

    assert_eq!(reports.len(), 1);
    assert_eq!(snapshots.len(), 1);
    assert_eq!(runtime_states.len(), 1);
    assert_eq!(reports[0].cycles, report.cycles);
    assert_eq!(snapshots[0].seed_value, snapshot.seed_value);
    assert_eq!(runtime_states[0].snapshot.epoch, spiral.epoch);

    let loaded_stats = &runtime_states[0].archive_stats;
    assert_eq!(loaded_stats.len(), spiral.archive_stats().len());
    let loaded_resonance = loaded_stats
        .iter()
        .find(|(name, _)| name == "ResonanceEngine")
        .expect("resonance stats should exist");
    assert!(loaded_resonance.1.entry_count >= 1);
    assert!(loaded_resonance.1.peak_resonance >= 0.91);

    let _ = std::fs::remove_dir_all(&dir);
}

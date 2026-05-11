//! `Sky` snapshot — planetary positions for a given instant and the analytics derived from them.
//!
//! Math sources:
//! - JPL Keplerian elements (heliocentric ecliptic, J2000) for the major planets.
//! - Meeus low-precision lunar theory (Astronomical Algorithms, Ch. 47) for the Moon.
//! - Geocentric Sun = `-Earth_heliocentric`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::traits::EvolutionContext;

use super::aspect::{angular_separation, match_aspect, Aspect, AspectKind};
use super::planet::{KeplerianElements, Planet};
use super::time::{julian_centuries, julian_day, J2000_JD};
use super::zodiac::{ZodiacElement, ZodiacSign};

const AU_PER_KM: f64 = 1.0 / 149_597_870.7;

/// Position of a single planet in geocentric ecliptic coordinates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanetPosition {
    /// Planet identity.
    pub planet: Planet,
    /// Geocentric ecliptic longitude, normalized to `[0, 360)`.
    pub ecliptic_longitude: f64,
    /// Geocentric ecliptic latitude (degrees, can be negative).
    pub ecliptic_latitude: f64,
    /// Geocentric distance (astronomical units).
    pub distance_au: f64,
    /// Zodiac sign containing `ecliptic_longitude`.
    pub sign: ZodiacSign,
    /// Position within `sign`, in `[0, 30)` degrees.
    pub degree_in_sign: f64,
}

impl PlanetPosition {
    /// Pretty single-line label, e.g. `☉ 21°28' ♑ Capricorn`.
    pub fn pretty_label(&self) -> String {
        let degrees = self.degree_in_sign.floor() as u32;
        let minutes_fp = (self.degree_in_sign - degrees as f64) * 60.0;
        let minutes = minutes_fp.floor() as u32;
        format!(
            "{} {:>2}°{:02}' {} {}",
            self.planet.glyph(),
            degrees,
            minutes,
            self.sign.glyph(),
            self.sign.label()
        )
    }
}

/// Snapshot of every tracked planet at a specific UTC instant.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sky {
    /// UTC instant the snapshot describes.
    pub instant: DateTime<Utc>,
    /// Julian Day corresponding to `instant`.
    pub julian_day: f64,
    /// Geocentric positions for every planet in [`Planet::ALL`].
    pub positions: Vec<PlanetPosition>,
}

impl Sky {
    /// Snapshot of the present moment (UTC `now`).
    pub fn now() -> Self {
        Self::at(Utc::now())
    }

    /// Snapshot at the given UTC instant.
    pub fn at(when: DateTime<Utc>) -> Self {
        let jd = julian_day(when);
        Self::at_jd_with_instant(when, jd)
    }

    /// Snapshot for an arbitrary Julian Day. The `instant` field is set to the closest UTC date.
    pub fn at_jd(jd: f64) -> Self {
        let instant = jd_to_utc(jd);
        Self::at_jd_with_instant(instant, jd)
    }

    fn at_jd_with_instant(instant: DateTime<Utc>, jd: f64) -> Self {
        let t = julian_centuries(jd);
        let earth = earth_heliocentric_xyz(t);
        let mut positions = Vec::with_capacity(Planet::ALL.len());
        for planet in Planet::ALL {
            positions.push(compute_position(planet, t, &earth));
        }
        Self { instant, julian_day: jd, positions }
    }

    /// Looks up a single planet's position.
    pub fn position(&self, planet: Planet) -> Option<&PlanetPosition> {
        self.positions.iter().find(|position| position.planet == planet)
    }

    /// Detects every Ptolemaic aspect in the sky (unique unordered pairs).
    pub fn aspects(&self) -> Vec<Aspect> {
        let mut result = Vec::new();
        for left_idx in 0..self.positions.len() {
            for right_idx in (left_idx + 1)..self.positions.len() {
                let left = &self.positions[left_idx];
                let right = &self.positions[right_idx];
                let sep = angular_separation(left.ecliptic_longitude, right.ecliptic_longitude);
                if let Some((kind, orb)) = match_aspect(sep) {
                    result.push(Aspect::new(left.planet, right.planet, kind, orb));
                }
            }
        }
        result
    }

    /// Most populated zodiac sign across all planets. `None` when the sky is empty.
    pub fn dominant_sign(&self) -> Option<ZodiacSign> {
        let mut counts: HashMap<ZodiacSign, usize> = HashMap::new();
        for position in &self.positions {
            *counts.entry(position.sign).or_insert(0) += 1;
        }
        counts.into_iter().max_by_key(|(_, count)| *count).map(|(sign, _)| sign)
    }

    /// Most prevalent classical element across all planets. `None` when empty.
    pub fn dominant_element(&self) -> Option<ZodiacElement> {
        let mut counts: HashMap<ZodiacElement, usize> = HashMap::new();
        for position in &self.positions {
            *counts.entry(position.sign.element()).or_insert(0) += 1;
        }
        counts.into_iter().max_by_key(|(_, count)| *count).map(|(element, _)| element)
    }

    /// Mean exactness (`0..1`) of harmonic aspects currently present.
    pub fn resonance_field(&self) -> f32 {
        mean_exactness(&self.aspects(), |aspect| aspect.kind.is_harmonic())
    }

    /// Mean exactness (`0..1`) of tensional aspects currently present.
    pub fn tension_field(&self) -> f32 {
        mean_exactness(&self.aspects(), |aspect| aspect.kind.is_tensional())
    }

    /// Angular separation (degrees) between Sun and Moon geocentric longitudes, if both exist.
    #[must_use]
    pub fn lunar_solar_elongation_degrees(&self) -> Option<f64> {
        let sun = self.position(Planet::Sun)?;
        let moon = self.position(Planet::Moon)?;
        Some(angular_separation(sun.ecliptic_longitude, moon.ecliptic_longitude))
    }

    /// Ritual entropy in `[0, 1]`: conjunction clusters, tension spikes, lunar phase tension, anti-stillness.
    ///
    /// Deterministic from the sky geometry — not independent RNG.
    #[must_use]
    pub fn ritual_entropy(&self) -> f32 {
        let aspects = self.aspects();
        let conj = aspects
            .iter()
            .filter(|a| matches!(a.kind, AspectKind::Conjunction))
            .count() as f32;
        let planet_count = self.positions.len() as f32;
        let max_pairs = (planet_count * (planet_count - 1.0) * 0.5).max(1.0);
        let conj_density = (conj / max_pairs).min(1.0);

        let still = self.stillness();
        let chaos = (1.0 - still).clamp(0.0, 1.0);
        let tens = self.tension_field();
        let res = self.resonance_field();

        let lunar = self
            .lunar_solar_elongation_degrees()
            .map(|deg| {
                let rad = deg.to_radians();
                // New / full tension peaks (symbolic “phase shift” without a full ephemeris).
                rad.sin().abs() as f32
            })
            .unwrap_or(0.45);

        (0.20 * chaos + 0.26 * tens + 0.18 * conj_density + 0.14 * res + 0.22 * lunar).clamp(0.0, 1.0)
    }

    /// "Quiet room" coefficient in `[0.0, 1.0]`: high when aspect chatter is low.
    ///
    /// Computed as `1 - mean_exactness(all_aspects) * coverage_ratio`, where `coverage_ratio`
    /// is the fraction of planet pairs participating in any aspect.
    pub fn stillness(&self) -> f32 {
        let aspects = self.aspects();
        let planet_count = self.positions.len() as f32;
        if planet_count < 2.0 {
            return 1.0;
        }
        let max_pairs = planet_count * (planet_count - 1.0) * 0.5;
        if aspects.is_empty() {
            return 1.0;
        }
        let mean = aspects.iter().map(|a| a.exactness).sum::<f32>() / aspects.len() as f32;
        let coverage = (aspects.len() as f32 / max_pairs).clamp(0.0, 1.0);
        (1.0 - mean * coverage).clamp(0.0, 1.0)
    }

    /// Gently nudges an [`EvolutionContext`] using the sky's stillness and aspect fields.
    ///
    /// The "quiet room" rule: high stillness pulls toward steadiness (more resonance, less mutation
    /// and drift). Tension pulls toward mutation/drift. External influence is amplified by
    /// stillness, modeling a calm sky that lets the runtime listen.
    pub fn modulate(&self, context: EvolutionContext) -> EvolutionContext {
        let stillness = self.stillness().clamp(0.0, 1.0);
        let tension = self.tension_field().clamp(0.0, 1.0);
        let resonance = self.resonance_field().clamp(0.0, 1.0);

        let mutation_rate = (context.mutation_rate + tension * 0.25 - stillness * 0.15).clamp(0.0, 1.0);
        let resonance_pressure =
            (context.resonance_pressure + resonance * 0.20 + stillness * 0.15).clamp(0.0, 1.0);
        let drift = (context.drift + tension * 0.15 - stillness * 0.10).clamp(0.0, 1.0);
        let external = (context.external_influence + stillness * 0.20).clamp(0.0, 1.0);
        let ritual = self.ritual_entropy();
        let shadow = ((1.0 - stillness) * 0.32 + tension * 0.48 + ritual * 0.22).clamp(0.0, 1.0);
        let dream = stillness > 0.82;

        context
            .with_mutation_rate(mutation_rate)
            .with_external_influence(external)
            .with_resonance_pressure(resonance_pressure)
            .with_drift(drift)
            .with_ritual_entropy(ritual)
            .with_shadow_pressure(shadow)
            .with_dream_phase(dream)
            .normalized()
    }
}

fn mean_exactness<F>(aspects: &[Aspect], filter: F) -> f32
where
    F: Fn(&Aspect) -> bool,
{
    let filtered: Vec<&Aspect> = aspects.iter().filter(|a| filter(a)).collect();
    if filtered.is_empty() {
        return 0.0;
    }
    filtered.iter().map(|a| a.exactness).sum::<f32>() / filtered.len() as f32
}

fn jd_to_utc(jd: f64) -> DateTime<Utc> {
    let delta_seconds = ((jd - J2000_JD) * 86_400.0) as i64;
    let j2000 = DateTime::<Utc>::from_timestamp(946_728_000, 0).unwrap_or_else(|| Utc::now());
    j2000 + chrono::Duration::seconds(delta_seconds)
}

fn solve_kepler(mean_anomaly_rad: f64, eccentricity: f64) -> f64 {
    let mut eccentric = mean_anomaly_rad;
    for _ in 0..16 {
        let f = eccentric - eccentricity * eccentric.sin() - mean_anomaly_rad;
        let f_prime = 1.0 - eccentricity * eccentric.cos();
        let delta = f / f_prime;
        eccentric -= delta;
        if delta.abs() < 1e-10 {
            break;
        }
    }
    eccentric
}

fn heliocentric_xyz(elements: &KeplerianElements) -> [f64; 3] {
    let a = elements.a;
    let e = elements.e;
    let i_rad = elements.i.to_radians();
    let mean_long = elements.mean_long.rem_euclid(360.0);
    let long_peri = elements.long_peri.rem_euclid(360.0);
    let long_node = elements.long_node.rem_euclid(360.0);

    let mean_anomaly = (mean_long - long_peri).rem_euclid(360.0).to_radians();
    let arg_perihelion = (long_peri - long_node).to_radians();
    let long_node_rad = long_node.to_radians();

    let ecc_anom = solve_kepler(mean_anomaly, e);

    let x_orb = a * (ecc_anom.cos() - e);
    let y_orb = a * (1.0 - e * e).max(0.0).sqrt() * ecc_anom.sin();

    let cos_w = arg_perihelion.cos();
    let sin_w = arg_perihelion.sin();
    let cos_n = long_node_rad.cos();
    let sin_n = long_node_rad.sin();
    let cos_i = i_rad.cos();
    let sin_i = i_rad.sin();

    let x_ecl =
        (cos_w * cos_n - sin_w * sin_n * cos_i) * x_orb + (-sin_w * cos_n - cos_w * sin_n * cos_i) * y_orb;
    let y_ecl =
        (cos_w * sin_n + sin_w * cos_n * cos_i) * x_orb + (-sin_w * sin_n + cos_w * cos_n * cos_i) * y_orb;
    let z_ecl = (sin_w * sin_i) * x_orb + (cos_w * sin_i) * y_orb;

    [x_ecl, y_ecl, z_ecl]
}

fn earth_heliocentric_xyz(t: f64) -> [f64; 3] {
    // JPL "Earth-Moon Barycenter" Keplerian elements at J2000.
    let earth_em = KeplerianElements {
        a: 1.000_002_61,
        e: 0.016_711_23,
        i: -0.000_015_31,
        mean_long: 100.464_571_66,
        long_peri: 102.937_681_93,
        long_node: 0.0,
        a_rate: 0.000_005_62,
        e_rate: -0.000_043_92,
        i_rate: -0.012_946_68,
        mean_long_rate: 35_999.372_449_81,
        long_peri_rate: 0.323_273_64,
        long_node_rate: 0.0,
    };
    heliocentric_xyz(&earth_em.at(t))
}

fn compute_position(planet: Planet, t: f64, earth: &[f64; 3]) -> PlanetPosition {
    let (longitude, latitude, distance) = match planet {
        Planet::Sun => {
            let x = -earth[0];
            let y = -earth[1];
            let z = -earth[2];
            spherical_from_xyz([x, y, z])
        }
        Planet::Moon => moon_geocentric(t),
        _ => {
            let elements = planet.elements().expect("major planet has elements").at(t);
            let helio = heliocentric_xyz(&elements);
            let geo = [helio[0] - earth[0], helio[1] - earth[1], helio[2] - earth[2]];
            spherical_from_xyz(geo)
        }
    };

    let normalized_longitude = longitude.rem_euclid(360.0);
    let sign = ZodiacSign::from_longitude(normalized_longitude);
    let degree_in_sign = normalized_longitude.rem_euclid(30.0);

    PlanetPosition {
        planet,
        ecliptic_longitude: normalized_longitude,
        ecliptic_latitude: latitude,
        distance_au: distance,
        sign,
        degree_in_sign,
    }
}

fn spherical_from_xyz(xyz: [f64; 3]) -> (f64, f64, f64) {
    let [x, y, z] = xyz;
    let r = (x * x + y * y + z * z).sqrt();
    let longitude = y.atan2(x).to_degrees();
    let latitude = if r > 0.0 {
        (z / r).clamp(-1.0, 1.0).asin().to_degrees()
    } else {
        0.0
    };
    (longitude, latitude, r)
}

fn moon_geocentric(t: f64) -> (f64, f64, f64) {
    // Meeus, Astronomical Algorithms, low-precision series for the Moon.
    let mean_long = 218.316_447_7 + 481_267.881_234_21 * t - 0.001_578_6 * t * t;
    let elongation = 297.850_192_1 + 445_267.111_403_4 * t - 0.001_881_9 * t * t;
    let sun_mean = 357.529_109_2 + 35_999.050_290_9 * t - 0.000_153_6 * t * t;
    let moon_mean = 134.963_396_4 + 477_198.867_505_5 * t + 0.008_741_4 * t * t;
    let arg_lat = 93.272_095_0 + 483_202.017_523_3 * t - 0.003_653_9 * t * t;

    let d_rad = elongation.to_radians();
    let m_rad = sun_mean.to_radians();
    let mp_rad = moon_mean.to_radians();
    let f_rad = arg_lat.to_radians();

    let longitude_corr = 6.288_774 * mp_rad.sin()
        + 1.274_027 * (2.0 * d_rad - mp_rad).sin()
        + 0.658_314 * (2.0 * d_rad).sin()
        + 0.213_618 * (2.0 * mp_rad).sin()
        - 0.185_116 * m_rad.sin()
        - 0.114_332 * (2.0 * f_rad).sin()
        + 0.058_793 * (2.0 * d_rad - 2.0 * mp_rad).sin()
        + 0.057_066 * (2.0 * d_rad - m_rad - mp_rad).sin()
        + 0.053_322 * (2.0 * d_rad + mp_rad).sin()
        + 0.045_758 * (2.0 * d_rad - m_rad).sin();

    let latitude_corr = 5.128_122 * f_rad.sin()
        + 0.280_602 * (mp_rad + f_rad).sin()
        + 0.277_693 * (mp_rad - f_rad).sin()
        + 0.173_237 * (2.0 * d_rad - f_rad).sin();

    let distance_km = 385_000.56 - 20_905.355 * mp_rad.cos()
        - 3_699.111 * (2.0 * d_rad - mp_rad).cos()
        - 2_955.968 * (2.0 * d_rad).cos();

    let longitude = (mean_long + longitude_corr).rem_euclid(360.0);
    let latitude = latitude_corr;
    let distance_au = distance_km.max(1.0) * AU_PER_KM;

    (longitude, latitude, distance_au)
}

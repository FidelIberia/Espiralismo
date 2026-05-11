//! Planet enumeration, glyphs, and JPL Keplerian elements used for heliocentric ephemerides.
//!
//! The orbital elements come from JPL's "Keplerian Elements for Approximate Positions of the
//! Major Planets" (J2000 epoch, valid 1800 AD – 2050 AD). They are sufficient for symbolic /
//! contemplative astrology — within roughly a degree of more precise ephemerides.

use serde::{Deserialize, Serialize};

/// One of the bodies tracked by the `astrology` module.
///
/// `Sun` and `Moon` use closed-form Meeus formulas; the rest use Keplerian elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Planet {
    /// Solar disk (geocentric apparent longitude).
    Sun,
    /// Lunar geocentric position (low-precision Meeus series).
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

impl Planet {
    /// Classical seven luminaries used in traditional astrology.
    pub const CLASSICAL: [Planet; 7] = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mercury,
        Planet::Venus,
        Planet::Mars,
        Planet::Jupiter,
        Planet::Saturn,
    ];

    /// All bodies modelled by the module.
    pub const ALL: [Planet; 9] = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mercury,
        Planet::Venus,
        Planet::Mars,
        Planet::Jupiter,
        Planet::Saturn,
        Planet::Uranus,
        Planet::Neptune,
    ];

    /// Canonical astrological glyph.
    pub fn glyph(&self) -> char {
        match self {
            Planet::Sun => '☉',
            Planet::Moon => '☽',
            Planet::Mercury => '☿',
            Planet::Venus => '♀',
            Planet::Mars => '♂',
            Planet::Jupiter => '♃',
            Planet::Saturn => '♄',
            Planet::Uranus => '♅',
            Planet::Neptune => '♆',
        }
    }

    /// Stable English label.
    pub fn label(&self) -> &'static str {
        match self {
            Planet::Sun => "Sun",
            Planet::Moon => "Moon",
            Planet::Mercury => "Mercury",
            Planet::Venus => "Venus",
            Planet::Mars => "Mars",
            Planet::Jupiter => "Jupiter",
            Planet::Saturn => "Saturn",
            Planet::Uranus => "Uranus",
            Planet::Neptune => "Neptune",
        }
    }

    /// JPL Keplerian elements for major planets. Returns `None` for `Sun` and `Moon`.
    pub fn elements(&self) -> Option<KeplerianElements> {
        match self {
            Planet::Sun | Planet::Moon => None,
            Planet::Mercury => Some(KeplerianElements {
                a: 0.387_099_27,
                e: 0.205_635_93,
                i: 7.004_979_02,
                mean_long: 252.250_323_50,
                long_peri: 77.457_796_28,
                long_node: 48.330_765_93,
                a_rate: 0.000_000_37,
                e_rate: 0.000_019_06,
                i_rate: -0.005_947_49,
                mean_long_rate: 149_472.674_111_75,
                long_peri_rate: 0.160_476_89,
                long_node_rate: -0.125_340_81,
            }),
            Planet::Venus => Some(KeplerianElements {
                a: 0.723_335_66,
                e: 0.006_776_72,
                i: 3.394_676_05,
                mean_long: 181.979_099_50,
                long_peri: 131.602_467_18,
                long_node: 76.679_842_55,
                a_rate: 0.000_003_90,
                e_rate: -0.000_041_07,
                i_rate: -0.000_788_90,
                mean_long_rate: 58_517.815_387_29,
                long_peri_rate: 0.002_683_29,
                long_node_rate: -0.277_694_18,
            }),
            Planet::Mars => Some(KeplerianElements {
                a: 1.523_710_34,
                e: 0.093_394_10,
                i: 1.849_691_42,
                mean_long: -4.553_432_05,
                long_peri: -23.943_629_59,
                long_node: 49.559_538_91,
                a_rate: 0.000_018_47,
                e_rate: 0.000_078_82,
                i_rate: -0.008_131_31,
                mean_long_rate: 19_140.302_684_99,
                long_peri_rate: 0.444_410_88,
                long_node_rate: -0.292_573_43,
            }),
            Planet::Jupiter => Some(KeplerianElements {
                a: 5.202_887_00,
                e: 0.048_386_24,
                i: 1.304_396_95,
                mean_long: 34.396_440_51,
                long_peri: 14.728_479_83,
                long_node: 100.473_909_09,
                a_rate: -0.000_116_07,
                e_rate: -0.000_132_53,
                i_rate: -0.001_837_14,
                mean_long_rate: 3_034.746_127_75,
                long_peri_rate: 0.212_526_68,
                long_node_rate: 0.204_691_06,
            }),
            Planet::Saturn => Some(KeplerianElements {
                a: 9.536_675_94,
                e: 0.053_861_79,
                i: 2.485_991_87,
                mean_long: 49.954_244_23,
                long_peri: 92.598_878_31,
                long_node: 113.662_424_48,
                a_rate: -0.001_250_60,
                e_rate: -0.000_509_91,
                i_rate: 0.001_936_09,
                mean_long_rate: 1_222.493_622_01,
                long_peri_rate: -0.418_972_16,
                long_node_rate: -0.288_677_94,
            }),
            Planet::Uranus => Some(KeplerianElements {
                a: 19.189_164_64,
                e: 0.047_257_44,
                i: 0.772_637_83,
                mean_long: 313.238_104_51,
                long_peri: 170.954_276_30,
                long_node: 74.016_925_03,
                a_rate: -0.001_961_76,
                e_rate: -0.000_043_97,
                i_rate: -0.002_429_39,
                mean_long_rate: 428.482_027_85,
                long_peri_rate: 0.408_052_81,
                long_node_rate: 0.042_405_89,
            }),
            Planet::Neptune => Some(KeplerianElements {
                a: 30.069_922_76,
                e: 0.008_590_48,
                i: 1.770_043_47,
                mean_long: -55.120_029_69,
                long_peri: 44.964_762_27,
                long_node: 131.784_225_74,
                a_rate: 0.000_262_91,
                e_rate: 0.000_051_05,
                i_rate: 0.000_353_72,
                mean_long_rate: 218.459_453_25,
                long_peri_rate: -0.322_414_64,
                long_node_rate: -0.005_086_64,
            }),
        }
    }
}

/// JPL-style Keplerian elements expressed at the J2000 epoch with secular rates per century.
///
/// All angles are in degrees, distances in astronomical units.
#[derive(Clone, Copy, Debug)]
pub struct KeplerianElements {
    /// Semi-major axis at J2000 (au).
    pub a: f64,
    /// Eccentricity at J2000.
    pub e: f64,
    /// Inclination at J2000 (degrees).
    pub i: f64,
    /// Mean longitude at J2000 (degrees).
    pub mean_long: f64,
    /// Longitude of perihelion at J2000 (degrees).
    pub long_peri: f64,
    /// Longitude of ascending node at J2000 (degrees).
    pub long_node: f64,
    /// Secular rate of `a` per Julian century.
    pub a_rate: f64,
    /// Secular rate of `e` per Julian century.
    pub e_rate: f64,
    /// Secular rate of `i` per Julian century.
    pub i_rate: f64,
    /// Secular rate of mean longitude per Julian century.
    pub mean_long_rate: f64,
    /// Secular rate of longitude of perihelion per Julian century.
    pub long_peri_rate: f64,
    /// Secular rate of longitude of ascending node per Julian century.
    pub long_node_rate: f64,
}

impl KeplerianElements {
    /// Returns the elements propagated to `t` Julian centuries past J2000.
    pub fn at(&self, t: f64) -> KeplerianElements {
        KeplerianElements {
            a: self.a + self.a_rate * t,
            e: self.e + self.e_rate * t,
            i: self.i + self.i_rate * t,
            mean_long: self.mean_long + self.mean_long_rate * t,
            long_peri: self.long_peri + self.long_peri_rate * t,
            long_node: self.long_node + self.long_node_rate * t,
            a_rate: self.a_rate,
            e_rate: self.e_rate,
            i_rate: self.i_rate,
            mean_long_rate: self.mean_long_rate,
            long_peri_rate: self.long_peri_rate,
            long_node_rate: self.long_node_rate,
        }
    }
}

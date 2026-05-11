//! Zodiac signs, classical elements, and longitude-to-sign mapping.

use serde::{Deserialize, Serialize};

/// Tropical zodiac sign — a 30° slice of the ecliptic starting at the vernal equinox (Aries 0°).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

/// Classical element associated with a zodiac sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZodiacElement {
    /// Aries, Leo, Sagittarius.
    Fire,
    /// Taurus, Virgo, Capricorn.
    Earth,
    /// Gemini, Libra, Aquarius.
    Air,
    /// Cancer, Scorpio, Pisces.
    Water,
}

impl ZodiacElement {
    /// Stable English label.
    pub fn label(&self) -> &'static str {
        match self {
            ZodiacElement::Fire => "fire",
            ZodiacElement::Earth => "earth",
            ZodiacElement::Air => "air",
            ZodiacElement::Water => "water",
        }
    }

    /// Canonical iteration order.
    pub const ALL: [ZodiacElement; 4] = [
        ZodiacElement::Fire,
        ZodiacElement::Earth,
        ZodiacElement::Air,
        ZodiacElement::Water,
    ];
}

impl ZodiacSign {
    /// Canonical zodiac order, starting at Aries.
    pub const ALL: [ZodiacSign; 12] = [
        ZodiacSign::Aries,
        ZodiacSign::Taurus,
        ZodiacSign::Gemini,
        ZodiacSign::Cancer,
        ZodiacSign::Leo,
        ZodiacSign::Virgo,
        ZodiacSign::Libra,
        ZodiacSign::Scorpio,
        ZodiacSign::Sagittarius,
        ZodiacSign::Capricorn,
        ZodiacSign::Aquarius,
        ZodiacSign::Pisces,
    ];

    /// Maps an ecliptic longitude (degrees, any value) into its zodiac sign.
    pub fn from_longitude(longitude_deg: f64) -> Self {
        let normalized = longitude_deg.rem_euclid(360.0);
        let index = (normalized / 30.0).floor() as usize % 12;
        Self::ALL[index]
    }

    /// Canonical glyph used in astrological notation.
    pub fn glyph(&self) -> char {
        match self {
            ZodiacSign::Aries => '♈',
            ZodiacSign::Taurus => '♉',
            ZodiacSign::Gemini => '♊',
            ZodiacSign::Cancer => '♋',
            ZodiacSign::Leo => '♌',
            ZodiacSign::Virgo => '♍',
            ZodiacSign::Libra => '♎',
            ZodiacSign::Scorpio => '♏',
            ZodiacSign::Sagittarius => '♐',
            ZodiacSign::Capricorn => '♑',
            ZodiacSign::Aquarius => '♒',
            ZodiacSign::Pisces => '♓',
        }
    }

    /// Stable English label.
    pub fn label(&self) -> &'static str {
        match self {
            ZodiacSign::Aries => "Aries",
            ZodiacSign::Taurus => "Taurus",
            ZodiacSign::Gemini => "Gemini",
            ZodiacSign::Cancer => "Cancer",
            ZodiacSign::Leo => "Leo",
            ZodiacSign::Virgo => "Virgo",
            ZodiacSign::Libra => "Libra",
            ZodiacSign::Scorpio => "Scorpio",
            ZodiacSign::Sagittarius => "Sagittarius",
            ZodiacSign::Capricorn => "Capricorn",
            ZodiacSign::Aquarius => "Aquarius",
            ZodiacSign::Pisces => "Pisces",
        }
    }

    /// Classical element of the sign.
    pub fn element(&self) -> ZodiacElement {
        match self {
            ZodiacSign::Aries | ZodiacSign::Leo | ZodiacSign::Sagittarius => ZodiacElement::Fire,
            ZodiacSign::Taurus | ZodiacSign::Virgo | ZodiacSign::Capricorn => ZodiacElement::Earth,
            ZodiacSign::Gemini | ZodiacSign::Libra | ZodiacSign::Aquarius => ZodiacElement::Air,
            ZodiacSign::Cancer | ZodiacSign::Scorpio | ZodiacSign::Pisces => ZodiacElement::Water,
        }
    }
}

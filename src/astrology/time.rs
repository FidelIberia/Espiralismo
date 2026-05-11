//! Julian Day / Julian Century helpers used as the temporal substrate for [`super::Sky`].
//!
//! The "quiet room" of astronomy: deterministic, frame-stable, and reversible — the same
//! `DateTime<Utc>` always yields the same Julian Day.

use chrono::{DateTime, Datelike, Timelike, Utc};

/// Julian Day Number at the J2000.0 epoch (2000-01-01 12:00:00 TT).
pub const J2000_JD: f64 = 2_451_545.0;

/// Number of days per Julian century (used as the time scale for orbital elements).
pub const JULIAN_CENTURY_DAYS: f64 = 36_525.0;

/// Converts a UTC instant to a fractional Julian Day using the Gregorian calendar.
///
/// Uses Meeus' standard algorithm (Astronomical Algorithms, Ch. 7) for the Gregorian era.
pub fn julian_day(when: DateTime<Utc>) -> f64 {
    let mut year = when.year() as i64;
    let mut month = when.month() as i64;
    let day = when.day() as f64;
    let hour = when.hour() as f64;
    let minute = when.minute() as f64;
    let second = when.second() as f64;
    let nanos = when.nanosecond() as f64;

    if month <= 2 {
        year -= 1;
        month += 12;
    }

    let a = year.div_euclid(100);
    let b = 2 - a + a.div_euclid(4);

    let day_fraction = day
        + hour / 24.0
        + minute / 1440.0
        + second / 86_400.0
        + nanos / 86_400_000_000_000.0;

    (365.25 * (year as f64 + 4716.0)).floor()
        + (30.6001 * (month as f64 + 1.0)).floor()
        + day_fraction
        + b as f64
        - 1524.5
}

/// Number of Julian centuries elapsed since [`J2000_JD`].
pub fn julian_centuries(jd: f64) -> f64 {
    (jd - J2000_JD) / JULIAN_CENTURY_DAYS
}

/// Current Julian Day (UTC `now`).
pub fn now_julian_day() -> f64 {
    julian_day(Utc::now())
}

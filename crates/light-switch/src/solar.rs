use chrono::{Datelike, NaiveTime};

/// Calculate sunrise and sunset times for a given date and location.
///
/// Uses the NOAA solar calculator simplified algorithm.
/// Returns `(sunrise, sunset)` as `NaiveTime` in local solar time.
/// The caller should apply timezone offset separately.
pub fn sunrise_sunset(year: i32, month: u32, day: u32, lat: f64, lon: f64) -> (NaiveTime, NaiveTime) {
    let day_of_year = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap())
        .ordinal() as f64;

    // Fractional year (radians)
    let gamma = 2.0 * std::f64::consts::PI / 365.0 * (day_of_year - 1.0);

    // Equation of time (minutes)
    let eqtime = 229.18
        * (0.000075 + 0.001868 * gamma.cos() - 0.032077 * gamma.sin()
            - 0.014615 * (2.0 * gamma).cos()
            - 0.040849 * (2.0 * gamma).sin());

    // Solar declination (radians)
    let decl = 0.006918 - 0.399912 * gamma.cos() + 0.070257 * gamma.sin()
        - 0.006758 * (2.0 * gamma).cos()
        + 0.000907 * (2.0 * gamma).sin()
        - 0.002697 * (3.0 * gamma).cos()
        + 0.00148 * (3.0 * gamma).sin();

    let lat_rad = lat.to_radians();

    // Hour angle (degrees)
    // cos(90.833°) ≈ -0.01454 — the cosine of the official zenith for sunrise/sunset
    let cos_zenith: f64 = -0.01454;
    let cos_ha = (cos_zenith - lat_rad.sin() * decl.sin())
        / (lat_rad.cos() * decl.cos());

    // Clamp for polar regions
    let cos_ha = cos_ha.clamp(-1.0, 1.0);
    let ha = cos_ha.acos().to_degrees();

    // Sunrise and sunset in minutes from midnight UTC
    let sunrise_min = 720.0 - 4.0 * (lon + ha) - eqtime;
    let sunset_min = 720.0 - 4.0 * (lon - ha) - eqtime;

    (minutes_to_time(sunrise_min), minutes_to_time(sunset_min))
}

fn minutes_to_time(mut mins: f64) -> NaiveTime {
    // Wrap into 0..1440
    mins = mins.rem_euclid(1440.0);
    let h = (mins / 60.0).floor() as u32;
    let m = (mins % 60.0).floor() as u32;
    NaiveTime::from_hms_opt(h.min(23), m.min(59), 0).unwrap_or(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn paris_equinox() {
        // Paris ~48.86°N, 2.35°E, March 20
        let (rise, set) = sunrise_sunset(2025, 3, 20, 48.86, 2.35);
        // Sunrise should be roughly 6:00-7:00 UTC
        assert!(rise.hour() >= 5 && rise.hour() <= 7, "sunrise: {rise}");
        // Sunset should be roughly 18:00-19:00 UTC
        assert!(set.hour() >= 17 && set.hour() <= 19, "sunset: {set}");
    }

    #[test]
    fn polar_doesnt_panic() {
        // North pole in summer — sun doesn't set, should not panic
        let (_rise, _set) = sunrise_sunset(2025, 6, 21, 89.0, 0.0);
    }
}

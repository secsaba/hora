use crate::*;

/// Ch. 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialLagna {
    HoraLagna,
    GhatiLagna,
    SreeLagna,
    BhaavaLagna,
}

/// The special lagnas by how much the book uses them: HL and GL widely (ch. 5 §5.1), SL (§5.7), BL not at all (footnote 10).
pub const SPECIAL_LAGNAS: [SpecialLagna; 4] = [
    SpecialLagna::HoraLagna,
    SpecialLagna::GhatiLagna,
    SpecialLagna::SreeLagna,
    SpecialLagna::BhaavaLagna,
];

/// The bhaava lagna `hours` after sunrise, from the Sun's `sun_at_sunrise` at 15° an hour (ch. 5 §5.2).
pub fn bhaava_lagna(sun_at_sunrise: f64, hours: f64) -> f64 {
    reduce(sun_at_sunrise + 15.0 * hours)
}

/// The hora lagna `hours` after sunrise, from the Sun's `sun_at_sunrise` at 30° an hour (ch. 5 §5.3).
pub fn hora_lagna(sun_at_sunrise: f64, hours: f64) -> f64 {
    reduce(sun_at_sunrise + 30.0 * hours)
}

/// The ghati lagna `hours` after sunrise, from the Sun's `sun_at_sunrise` at 30° per 24 minutes (ch. 5 §5.4).
pub fn ghati_lagna(sun_at_sunrise: f64, hours: f64) -> f64 {
    reduce(sun_at_sunrise + 30.0 * 60.0 / 24.0 * hours)
}

/// The sree lagna: `lagna` plus 360° × the fraction of its nakshatra `moon` has covered (ch. 5 §5.7).
pub fn sree_lagna(lagna: f64, moon: f64) -> f64 {
    // 360° per 13°20′ is 27×, and 27 × a nakshatra's start is a whole circle, so the start drops out.
    reduce(lagna + 27.0 * moon)
}

/// The special lagnas of `sky` with their longitudes: the hora, ghati and bhaava lagnas count the exact time from the
/// Vedic day's sunrise, so where the Sky has none only the sree lagna is there (ch. 5, ADR 0003).
pub fn special_lagnas(sky: &Sky) -> Vec<(SpecialLagna, f64)> {
    let sree = (
        SpecialLagna::SreeLagna,
        sree_lagna(sky.lagna, sky.grahas.moon.longitude),
    );
    let Some(day) = &sky.vedic_day else {
        return vec![sree];
    };
    let (sun, hours) = (day.sun_at_sunrise, (sky.jd - day.sunrise) * 24.0);
    vec![
        (SpecialLagna::HoraLagna, hora_lagna(sun, hours)),
        (SpecialLagna::GhatiLagna, ghati_lagna(sun, hours)),
        sree,
        (SpecialLagna::BhaavaLagna, bhaava_lagna(sun, hours)),
    ]
}

#[cfg(test)]
mod tests;

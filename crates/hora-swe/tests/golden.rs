//! Golden charts from P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*,
//! pinned exactly as the book prints them.

use hora_swe::{
    Ayanamsa, Calendar, NodeType, Request, SunriseDefinition, UtcOffset, calculate, julian_day,
};

/// Sidereal longitude from a sign (0 = Aries) and degrees, minutes, seconds within it.
fn dms(sign: u8, d: u8, m: u8, s: f64) -> f64 {
    f64::from(sign) * 30.0 + f64::from(d) + f64::from(m) / 60.0 + s / 3600.0
}

#[track_caller]
fn assert_arcsec(name: &str, got: f64, book: f64, tolerance: f64) {
    let diff = (got - book) * 3600.0;
    assert!(
        diff.abs() <= tolerance,
        "{name}: got {got}°, book {book}°, off by {diff:.4}″"
    );
}

/// The book's Sun has no aberration (about 1.5 s), which `swe_rise_trans` cannot drop,
/// and the book prints whole seconds, possibly truncated (up to 1 s).
const SUN_EVENT_SECONDS: f64 = 3.0;

/// `book` is the printed local date and time; `utc_offset` the printed offset in hours.
#[track_caller]
fn assert_ut(
    name: &str,
    got: f64,
    [year, month, day, hour, minute, second]: [i32; 6],
    utc_offset: f64,
) {
    let hour = f64::from(hour) + f64::from(minute) / 60.0 + f64::from(second) / 3600.0;
    let book = julian_day(year, month, day, hour, Calendar::Gregorian).unwrap() - utc_offset / 24.0;
    let diff = (got - book) * 86400.0;
    assert!(
        diff.abs() <= SUN_EVENT_SECONDS,
        "{name}: got JD {got}, book JD {book}, off by {diff:.1} s"
    );
}

/// Chart 1: ch. 9 *Arudha Padas*, p. 87 (the book's chart 2 is the same chart).
#[tokio::test]
async fn chart_1_wilmington_2000() {
    // Printed: 2000-04-09 17:55 at UTC−4:00. 71°W12′, 42°N30′. No altitude printed, so 0.
    let sky = calculate(Request {
        year: 2000,
        month: 4,
        day: 9,
        hour: 17,
        minute: 55,
        second: 0,
        calendar: Calendar::Gregorian,
        utc_offset: UtcOffset::Stated(-4 * 3600),
        latitude: 42.5,
        longitude: -(71.0 + 12.0 / 60.0),
        altitude: 0.0,
        ayanamsa: Ayanamsa::TrueChitra,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap();

    // One unit of the book's last printed digit: it rounds some values and truncates others.
    let tol = 0.01;
    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 10, 49, 44.53), tol);
    assert_arcsec("sun", g.sun.longitude, dms(11, 26, 30, 31.79), tol);
    assert_arcsec("moon", g.moon.longitude, dms(2, 4, 45, 58.43), tol);
    assert_arcsec("mars", g.mars.longitude, dms(0, 19, 10, 12.75), tol);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 1, 37, 2.96), tol);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 17, 22, 39.78), tol);
    assert_arcsec("venus", g.venus.longitude, dms(11, 10, 2, 30.56), tol);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 22, 42, 46.33), tol);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 5, 56, 6.19), tol);
    assert_arcsec("ayanamsa", sky.ayanamsa, dms(0, 23, 50, 42.57), tol);
    let day = sky.vedic_day.unwrap();
    assert_ut("sunrise", day.sunrise, [2000, 4, 9, 6, 13, 8], -4.0);
    assert_ut("sunset", day.sunset, [2000, 4, 9, 19, 20, 9], -4.0);
}

/// Chart 3 (A.B. Vajpayee): ch. 9 *Arudha Padas*, p. 97.
#[tokio::test]
async fn chart_3_gwalior_1926() {
    // Printed: 1926-12-25 05:12 IST (UTC+5:30). 78°E10′, 26°N14′.
    let sky = calculate(Request {
        year: 1926,
        month: 12,
        day: 25,
        hour: 5,
        minute: 12,
        second: 0,
        calendar: Calendar::Gregorian,
        utc_offset: UtcOffset::Stated(5 * 3600 + 30 * 60),
        latitude: 26.0 + 14.0 / 60.0,
        longitude: 78.0 + 10.0 / 60.0,
        altitude: 0.0,
        ayanamsa: Ayanamsa::TrueChitra,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap();

    // The book's ΔT for 1926 is ~0.27 s above ours (Moon 0.15″ off) and its ayanamsa 0.014″ smaller (older Spica data).
    let tol = 0.2;
    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 14, 13, 4.95), tol);
    assert_arcsec("sun", g.sun.longitude, dms(8, 9, 36, 47.60), tol);
    assert_arcsec("moon", g.moon.longitude, dms(4, 15, 29, 36.26), tol);
    assert_arcsec("mars", g.mars.longitude, dms(0, 13, 40, 32.96), tol);
    assert_arcsec("mercury", g.mercury.longitude, dms(7, 21, 0, 29.70), tol);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 2, 6, 36.83), tol);
    assert_arcsec("venus", g.venus.longitude, dms(8, 17, 42, 58.68), tol);
    assert_arcsec("saturn", g.saturn.longitude, dms(7, 9, 42, 15.18), tol);
    assert_arcsec("rahu", g.rahu.longitude, dms(2, 14, 31, 38.45), tol);
    assert_arcsec("ayanamsa", sky.ayanamsa, dms(0, 22, 49, 20.58), tol);
    // Born before sunrise on 25 Dec, so the Vedic day (ch. 1, p. 17) is 24 Dec's, as the book prints.
    let day = sky.vedic_day.unwrap();
    assert_ut("sunrise", day.sunrise, [1926, 12, 24, 7, 1, 50], 5.5);
    assert_ut("sunset", day.sunset, [1926, 12, 24, 17, 31, 27], 5.5);
}

/// Chart 4 (an engineer in the US): ch. 9 *Arudha Padas*, p. 98.
#[tokio::test]
async fn chart_4_pithapuram_1968() {
    // Printed: 1968-03-19 18:30 IST (UTC+5:30). 82°E19′, 17°N07′.
    let sky = calculate(Request {
        year: 1968,
        month: 3,
        day: 19,
        hour: 18,
        minute: 30,
        second: 0,
        calendar: Calendar::Gregorian,
        utc_offset: UtcOffset::Stated(5 * 3600 + 30 * 60),
        latitude: 17.0 + 7.0 / 60.0,
        longitude: 82.0 + 19.0 / 60.0,
        altitude: 0.0,
        ayanamsa: Ayanamsa::TrueChitra,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap();

    // ΔT for 1968 is well measured, so the tolerance is one unit of the last printed digit, as in chart 1.
    let tol = 0.01;
    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 10, 52, 52.91), tol);
    assert_arcsec("sun", g.sun.longitude, dms(11, 5, 36, 3.87), tol);
    assert_arcsec("moon", g.moon.longitude, dms(7, 10, 16, 23.62), tol);
    assert_arcsec("mars", g.mars.longitude, dms(0, 0, 21, 8.03), tol);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 8, 56, 58.73), tol);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 4, 6, 57.69), tol);
    assert_arcsec("venus", g.venus.longitude, dms(10, 11, 31, 1.54), tol);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 19, 54, 0.04), tol);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 26, 26, 16.85), tol);
    assert_arcsec("ayanamsa", sky.ayanamsa, dms(0, 23, 23, 51.89), tol);
    let day = sky.vedic_day.unwrap();
    assert_ut("sunrise", day.sunrise, [1968, 3, 19, 6, 6, 1], 5.5);
    assert_ut("sunset", day.sunset, [1968, 3, 19, 18, 11, 25], 5.5);
}

/// Not a book value: chart 1 checked against the author's software, Jagannatha Hora, under each sunrise definition.
#[tokio::test]
async fn chart_1_sunrise_definitions_match_jagannatha_hora() {
    // Jagannatha Hora's sunrise and sunset, local clock time (UTC−4:00), as [hour, minute, second].
    let jhora = [
        (SunriseDefinition::CentreTrue, [6, 17, 37], [19, 15, 40]),
        (SunriseDefinition::UpperLimbTrue, [6, 16, 9], [19, 17, 8]),
        (SunriseDefinition::CentreApparent, [6, 14, 36], [19, 18, 40]),
        (
            SunriseDefinition::UpperLimbApparent,
            [6, 13, 8],
            [19, 20, 9],
        ),
    ];
    for (sunrise_definition, [rh, rm, rs], [sh, sm, ss]) in jhora {
        let sky = calculate(Request {
            year: 2000,
            month: 4,
            day: 9,
            hour: 17,
            minute: 55,
            second: 0,
            calendar: Calendar::Gregorian,
            utc_offset: UtcOffset::Stated(-4 * 3600),
            latitude: 42.5,
            longitude: -(71.0 + 12.0 / 60.0),
            altitude: 0.0,
            ayanamsa: Ayanamsa::TrueChitra,
            node: NodeType::Mean,
            sunrise_definition,
        })
        .await
        .unwrap();
        let day = sky.vedic_day.unwrap();
        let name = format!("{sunrise_definition:?}");
        assert_ut(&name, day.sunrise, [2000, 4, 9, rh, rm, rs], -4.0);
        assert_ut(&name, day.sunset, [2000, 4, 9, sh, sm, ss], -4.0);
    }
}

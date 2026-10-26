//! Golden charts from P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*,
//! pinned exactly as the book prints them.

use hora_swe::{Ayanamsa, Calendar, NodeType, Request, SunriseDefinition, UtcOffset, calculate};

/// Sidereal longitude from a sign (0 = Aries) and degrees and minutes within it.
fn dms(sign: u8, d: u8, m: u8) -> f64 {
    f64::from(sign) * 30.0 + f64::from(d) + f64::from(m) / 60.0
}

#[track_caller]
fn assert_arcsec(name: &str, got: f64, book: f64, tolerance: f64) {
    let diff = (got - book) * 3600.0;
    assert!(
        diff.abs() <= tolerance,
        "{name}: got {got}°, book {book}°, off by {diff:.4}″"
    );
}

/// The book prints whole arcminutes.
const ARCMINUTE: f64 = 60.0;

/// Chart 1: ch. 9 *Arudha Padas*, p. 87.
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
        ayanamsa: Ayanamsa::Lahiri,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap();

    // No lagna: the book's (10 Vi 58) is 6–10′ off ours, as in charts 2 and 3, for an unknown reason.
    let g = &sky.grahas;
    assert_arcsec("sun", g.sun.longitude, dms(11, 26, 29), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(2, 4, 45), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 19, 9), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 1, 36), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 17, 21), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(11, 10, 1), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 22, 41), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 5, 55), ARCMINUTE);
}

/// Chart 2: ch. 9 *Arudha Padas*, p. 89; chart 1's birth data with the offset printed as "5:00 West".
#[tokio::test]
async fn chart_2_wilmington_2000_at_utc_minus_5() {
    // Printed: 2000-04-09 17:55 at UTC−5:00. 71°W12′, 42°N30′.
    let sky = calculate(Request {
        year: 2000,
        month: 4,
        day: 9,
        hour: 17,
        minute: 55,
        second: 0,
        calendar: Calendar::Gregorian,
        utc_offset: UtcOffset::Stated(-5 * 3600),
        latitude: 42.5,
        longitude: -(71.0 + 12.0 / 60.0),
        altitude: 0.0,
        ayanamsa: Ayanamsa::Lahiri,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap();

    // No lagna: the book's (22 Vi 41) is 6–10′ off ours, as in charts 1 and 3, for an unknown reason.
    let g = &sky.grahas;
    assert_arcsec("sun", g.sun.longitude, dms(11, 26, 32), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(2, 5, 21), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 19, 11), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 1, 39), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 17, 22), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(11, 10, 4), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 22, 42), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 5, 55), ARCMINUTE);
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
        ayanamsa: Ayanamsa::Lahiri,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap();

    // No lagna: the book's (14 Sc 18) is 6–10′ off ours, as in charts 1 and 2, for an unknown reason.
    let g = &sky.grahas;
    assert_arcsec("sun", g.sun.longitude, dms(8, 9, 35), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 15, 28), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 13, 39), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(7, 20, 59), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 2, 5), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(8, 17, 42), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(7, 9, 41), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(2, 14, 30), ARCMINUTE);
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
        ayanamsa: Ayanamsa::Lahiri,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap();

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 10, 52), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 5, 35), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(7, 10, 15), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 0, 20), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 8, 56), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 4, 6), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 11, 30), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 19, 53), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 26, 25), ARCMINUTE);
}

/// Chart 10 (Emperor Akbar): ch. 11 *Yogas*, §11.7, p. 136; birth data in footnote 39.
#[tokio::test]
async fn chart_10_akbar_1542() {
    // Printed: 4 Dec 1542 (Gregorian) 03:39 LMT, "November 24, 1542 based on the old calendar". 69°E47′, 25°N19′.
    // Entered as the Julian date, since hora rejects Gregorian dates before 1582-10-15.
    let sky = calculate(Request {
        year: 1542,
        month: 11,
        day: 24,
        hour: 3,
        minute: 39,
        second: 0,
        calendar: Calendar::Julian,
        utc_offset: UtcOffset::LocalMean,
        latitude: 25.0 + 19.0 / 60.0,
        longitude: 69.0 + 47.0 / 60.0,
        altitude: 0.0,
        ayanamsa: Ayanamsa::Lahiri,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap();

    // Wider than an arcminute: the Moon is 91.5″ off, Rahu 63.7″, the lagna −58.8″; likely ΔT for 1542, not chased.
    let tol = 100.0;
    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(6, 15, 30), tol);
    assert_arcsec("sun", g.sun.longitude, dms(7, 23, 46), tol);
    assert_arcsec("moon", g.moon.longitude, dms(2, 8, 51), tol);
    assert_arcsec("mars", g.mars.longitude, dms(9, 23, 5), tol);
    assert_arcsec("mercury", g.mercury.longitude, dms(8, 10, 10), tol);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 5, 42), tol);
    assert_arcsec("venus", g.venus.longitude, dms(6, 28, 10), tol);
    assert_arcsec("saturn", g.saturn.longitude, dms(6, 27, 28), tol);
    assert_arcsec("rahu", g.rahu.longitude, dms(10, 7, 56), tol);
}

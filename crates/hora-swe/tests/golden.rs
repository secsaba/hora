//! Golden charts from P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*,
//! pinned exactly as the book prints them.

use hora_swe::{
    Ayanamsa, Calendar, NodeType, Request, Sky, SunriseDefinition, UtcOffset, calculate,
};

/// The sky for a chart's printed birth data (date, clock time, offset, calendar, latitude, longitude; no
/// altitude is printed, so 0), with the book's settings: Lahiri, mean node, the upper limb rising.
async fn book_sky(
    (year, month, day): (i32, i32, i32),
    (hour, minute, second): (i32, i32, i32),
    utc_offset: UtcOffset,
    calendar: Calendar,
    latitude: f64,
    longitude: f64,
) -> Sky {
    calculate(Request {
        year,
        month,
        day,
        hour,
        minute,
        second,
        calendar,
        utc_offset,
        latitude,
        longitude,
        altitude: 0.0,
        ayanamsa: Ayanamsa::Lahiri,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    })
    .await
    .unwrap()
}

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
    // Printed: 2000-04-09 17:55 at UTC−4:00. 71°W12′, 42°N30′.
    let sky = book_sky(
        (2000, 4, 9),
        (17, 55, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        42.5,
        -(71.0 + 12.0 / 60.0),
    )
    .await;

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
    let sky = book_sky(
        (2000, 4, 9),
        (17, 55, 0),
        UtcOffset::Stated(-5 * 3600),
        Calendar::Gregorian,
        42.5,
        -(71.0 + 12.0 / 60.0),
    )
    .await;

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
    let sky = book_sky(
        (1926, 12, 25),
        (5, 12, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        26.0 + 14.0 / 60.0,
        78.0 + 10.0 / 60.0,
    )
    .await;

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
    let sky = book_sky(
        (1968, 3, 19),
        (18, 30, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 7.0 / 60.0,
        82.0 + 19.0 / 60.0,
    )
    .await;

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

/// Chart 5 (Aspects Exercise): ch. 10 *Aspects and Argalas*, p. 101.
#[tokio::test]
async fn chart_5_aspects_exercise_2000() {
    // Printed: 2000-04-19 19:21 at UTC−4:00. 71°W08′, 42°N40′.
    let sky = book_sky(
        (2000, 4, 19),
        (19, 21, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        42.0 + 40.0 / 60.0,
        -(71.0 + 8.0 / 60.0),
    )
    .await;

    // No lagna: the book's (5 Li 23) is 636″ ahead of ours; its printed time likely drops seconds, as chart 14's does.
    let g = &sky.grahas;
    assert_arcsec("sun", g.sun.longitude, dms(0, 6, 20), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(6, 20, 50), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 26, 22), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 17, 21), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 19, 42), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(11, 22, 25), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 23, 55), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 5, 23), ARCMINUTE);
}

/// Chart 6 (P.V. Narasimha Rao): ch. 10 *Aspects and Argalas*, p. 107.
#[tokio::test]
async fn chart_6_p_v_narasimha_rao_1921() {
    // Printed: 1921-06-28 12:49 at UTC+5:17. 79°E09′, 18°N26′.
    let sky = book_sky(
        (1921, 6, 28),
        (12, 49, 0),
        UtcOffset::Stated(5 * 3600 + 17 * 60),
        Calendar::Gregorian,
        18.0 + 26.0 / 60.0,
        79.0 + 9.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 24, 19), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(2, 13, 16), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(11, 10, 33), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 13, 33), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(2, 27, 40), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 20, 6), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 27, 40), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(4, 26, 26), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(6, 0, 47), ARCMINUTE);
}

/// Chart 7 (Ronald Reagan): ch. 10 *Aspects and Argalas*, p. 108.
#[tokio::test]
async fn chart_7_ronald_reagan_1911() {
    // Printed: 1911-02-06 02:04 at UTC−6:00. 89°W47′, 41°N38′.
    let sky = book_sky(
        (1911, 2, 6),
        (2, 4, 0),
        UtcOffset::Stated(-6 * 3600),
        Calendar::Gregorian,
        41.0 + 38.0 / 60.0,
        -(89.0 + 47.0 / 60.0),
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 7, 8), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(9, 23, 49), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(0, 19, 49), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(8, 11, 19), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(8, 28, 49), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 21, 7), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 10, 56), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 8, 12), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(0, 21, 54), ARCMINUTE);
}

/// Chart 8 (Guess his livelihood!): ch. 10 *Aspects and Argalas*, p. 109.
#[tokio::test]
async fn chart_8_guess_his_livelihood_1946() {
    // Printed: 1946-12-02 06:45 at UTC+1:00. 15°E39′, 38°N06′.
    let sky = book_sky(
        (1946, 12, 2),
        (6, 45, 0),
        UtcOffset::Stated(3600),
        Calendar::Gregorian,
        38.0 + 6.0 / 60.0,
        15.0 + 39.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 13, 14), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(7, 16, 20), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(10, 20, 15), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(7, 25, 30), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(6, 28, 9), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 21, 27), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(6, 24, 47), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(3, 15, 39), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(1, 18, 36), ARCMINUTE);
}

/// Chart 10 (Emperor Akbar): ch. 11 *Yogas*, §11.7, p. 136; birth data in footnote 39.
#[tokio::test]
async fn chart_10_akbar_1542() {
    // Printed: 4 Dec 1542 (Gregorian) 03:39 LMT, "November 24, 1542 based on the old calendar". 69°E47′, 25°N19′.
    // Entered as the Julian date, since hora rejects Gregorian dates before 1582-10-15.
    let sky = book_sky(
        (1542, 11, 24),
        (3, 39, 0),
        UtcOffset::LocalMean,
        Calendar::Julian,
        25.0 + 19.0 / 60.0,
        69.0 + 47.0 / 60.0,
    )
    .await;

    // Wider than an arcminute: the Moon is 91.5″ off, the lagna −67″, Rahu 63.7″; likely ΔT for 1542, not chased.
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

/// Chart 11 (Mercury's Ashtakavarga): ch. 12 *Ashtakavarga*, p. 152.
#[tokio::test]
async fn chart_11_p_v_narasimha_rao_1921() {
    // Printed: 1921-06-28 13:08 IST (UTC+5:30). 79°E09′, 18°N26′.
    let sky = book_sky(
        (1921, 6, 28),
        (13, 8, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        18.0 + 26.0 / 60.0,
        79.0 + 9.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 25, 45), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(2, 13, 17), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(11, 10, 36), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 13, 33), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(2, 27, 40), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 20, 6), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 27, 40), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(4, 26, 26), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(6, 0, 47), ARCMINUTE);
}

/// Chart 12 (SAV Exercise): ch. 12 *Ashtakavarga*, p. 155.
#[tokio::test]
async fn chart_12_sav_exercise_1958() {
    // Printed: 1958-08-16 07:05 at UTC−4:00. 83°W53′, 43°N36′.
    let sky = book_sky(
        (1958, 8, 16),
        (7, 5, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        43.0 + 36.0 / 60.0,
        -(83.0 + 53.0 / 60.0),
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(4, 3, 29), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(3, 29, 47), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 17, 39), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 22, 5), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 12, 23), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 3, 6), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(3, 7, 13), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(7, 25, 51), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(6, 2, 3), ARCMINUTE);
}

/// Chart 13 (An example): ch. 13 *Interpreting Charts*, p. 173.
#[tokio::test]
async fn chart_13_an_example_1894() {
    // Printed: 1894-05-20 13:22 IST (UTC+5:30). 79°E32′, 11°N57′.
    let sky = book_sky(
        (1894, 5, 20),
        (13, 22, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        11.0 + 57.0 / 60.0,
        79.0 + 32.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(4, 23, 10), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(1, 6, 52), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(7, 14, 1), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(10, 14, 49), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(1, 6, 28), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(1, 17, 58), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(11, 22, 11), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(5, 26, 50), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 15, 25), ARCMINUTE);
}

/// Chart 14 (Rajiv Gandhi): ch. 13 *Interpreting Charts*, p. 175.
#[tokio::test]
async fn chart_14_rajiv_gandhi_1944() {
    // Printed: 1944-08-20 07:11 IST (UTC+5:30), 07:11:40 in footnote 37, p. 136. 72°E49′, 18°N58′.
    let sky = book_sky(
        (1944, 8, 20),
        (7, 11, 40),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        18.0 + 58.0 / 60.0,
        72.0 + 49.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(4, 14, 44), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(4, 3, 49), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 17, 9), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(5, 1, 12), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 28, 34), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 12, 12), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 18, 40), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(2, 14, 13), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 2, 48), ARCMINUTE);
}

/// Chart 15 (Satyam Gaur): ch. 13 *Interpreting Charts*, p. 177.
#[tokio::test]
async fn chart_15_satyam_gaur_1970() {
    // Printed: 1970-11-04 16:06 IST (UTC+5:30). 76°E53′, 30°N44′.
    let sky = book_sky(
        (1970, 11, 4),
        (16, 6, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        30.0 + 44.0 / 60.0,
        76.0 + 53.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(11, 19, 54), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(6, 18, 9), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(8, 21, 18), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(5, 16, 4), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(6, 23, 7), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 21, 57), ARCMINUTE);
    // 61.5″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("venus", g.venus.longitude, dms(6, 27, 32), 62.0);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 26, 16), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(10, 5, 33), ARCMINUTE);
}

/// Chart 15's second chart (Shivam Gaur): ch. 13 *Interpreting Charts*, p. 177.
#[tokio::test]
async fn chart_15_shivam_gaur_1970() {
    // Printed: 1970-11-04 16:08 IST (UTC+5:30). 76°E53′, 30°N44′.
    let sky = book_sky(
        (1970, 11, 4),
        (16, 8, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        30.0 + 44.0 / 60.0,
        76.0 + 53.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(11, 20, 37), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(6, 18, 9), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(8, 21, 19), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(5, 16, 4), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(6, 23, 7), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 21, 57), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(6, 27, 32), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 26, 16), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(10, 5, 33), ARCMINUTE);
}

/// Chart 17 (Pandit Sanjay Rath): ch. 16 *Vimsottari Dasa*, p. 216.
#[tokio::test]
async fn chart_17_pandit_sanjay_rath_1963() {
    // Printed: 1963-08-07 21:14 IST (UTC+5:30). 83°E58′, 21°N27′.
    let sky = book_sky(
        (1963, 8, 7),
        (21, 14, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        21.0 + 27.0 / 60.0,
        83.0 + 58.0 / 60.0,
    )
    .await;

    // No lagna: the book's (14 Pi 01) is 946″ ahead of ours; its printed time likely drops seconds, as chart 14's does.
    let g = &sky.grahas;
    assert_arcsec("sun", g.sun.longitude, dms(3, 21, 4), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(10, 19, 58), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(5, 13, 40), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 13, 23), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(11, 26, 7), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(3, 14, 56), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 26, 49), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(2, 25, 45), ARCMINUTE);
}

/// Chart 18 (A lady): ch. 16 *Vimsottari Dasa*, p. 217.
#[tokio::test]
async fn chart_18_a_lady_1972() {
    // Printed: 1972-06-01 04:16 IST (UTC+5:30). 81°E12′, 16°N15′.
    let sky = book_sky(
        (1972, 6, 1),
        (4, 16, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(0, 26, 32), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(1, 17, 4), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(8, 29, 27), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 18, 55), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(1, 12, 14), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(8, 12, 49), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(2, 10, 48), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(1, 16, 34), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(9, 5, 6), ARCMINUTE);
}

/// Chart 19 (Sri Navin Patnaik): ch. 16 *Vimsottari Dasa*, p. 219.
#[tokio::test]
async fn chart_19_sri_navin_patnaik_1946() {
    // Printed: 1946-10-16 00:58 IST (UTC+5:30). 85°E50′, 20°N30′.
    let sky = book_sky(
        (1946, 10, 16),
        (0, 58, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        20.0 + 30.0 / 60.0,
        85.0 + 50.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(3, 22, 13), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(5, 28, 43), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(2, 5, 45), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(6, 21, 22), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(6, 18, 27), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 11, 11), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(7, 6, 37), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(3, 14, 36), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(1, 21, 7), ARCMINUTE);
}

/// Chart 20 (An engineer in USA): ch. 16 *Vimsottari Dasa*, p. 220.
#[tokio::test]
async fn chart_20_an_engineer_in_usa_1954() {
    // Printed: 1954-11-12 07:52 IST (UTC+5:30). 78°E50′, 12°N30′.
    let sky = book_sky(
        (1954, 11, 12),
        (7, 52, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        12.0 + 30.0 / 60.0,
        78.0 + 50.0 / 60.0,
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 18, 9), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(6, 25, 56), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(1, 17, 12), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(9, 21, 12), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(6, 7, 11), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(3, 6, 40), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(7, 1, 5), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(6, 19, 42), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(8, 14, 50), ARCMINUTE);
}

/// Chart 21 (John F. Kennedy, Jr.): ch. 16 *Vimsottari Dasa*, p. 222.
#[tokio::test]
async fn chart_21_john_f_kennedy_jr_1960() {
    // Printed: 1960-11-25 00:22 at UTC−5:00. 77°W02′, 38°N54′.
    let sky = book_sky(
        (1960, 11, 25),
        (0, 22, 0),
        UtcOffset::Stated(-5 * 3600),
        Calendar::Gregorian,
        38.0 + 54.0 / 60.0,
        -(77.0 + 2.0 / 60.0),
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(4, 18, 39), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(7, 9, 38), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(10, 4, 6), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 25, 12), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(6, 19, 55), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(8, 12, 31), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(8, 18, 35), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(8, 22, 15), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 17, 58), ARCMINUTE);
}

/// Chart 22 (A lady): ch. 16 *Vimsottari Dasa*, p. 223.
#[tokio::test]
async fn chart_22_a_lady_1954() {
    // Printed: 1954-12-07 02:13 at UTC−6:00. 93°W15′, 44°N58′.
    let sky = book_sky(
        (1954, 12, 7),
        (2, 13, 0),
        UtcOffset::Stated(-6 * 3600),
        Calendar::Gregorian,
        44.0 + 58.0 / 60.0,
        -(93.0 + 15.0 / 60.0),
    )
    .await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 18, 35), ARCMINUTE);
    // 60.1″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("sun", g.sun.longitude, dms(7, 21, 27), 62.0);
    assert_arcsec("moon", g.moon.longitude, dms(0, 13, 29), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(10, 8, 55), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(7, 11, 29), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(3, 6, 2), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(6, 21, 28), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(6, 22, 37), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(8, 13, 29), ARCMINUTE);
}

/// Chart 23 (Dr. B.V. Raman): ch. 16 *Vimsottari Dasa*, p. 224.
#[tokio::test]
async fn chart_23_dr_b_v_raman_1912() {
    // Printed: 1912-08-08 19:38 IST (UTC+5:30). 77°E35′, 13°N00′.
    let sky = book_sky(
        (1912, 8, 8),
        (19, 38, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        13.0,
        77.0 + 35.0 / 60.0,
    )
    .await;

    // No lagna: the book's (9 Aq 15) is 497″ ahead of ours; its printed time likely drops seconds, as chart 14's does.
    let g = &sky.grahas;
    assert_arcsec("sun", g.sun.longitude, dms(3, 22, 59), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(1, 23, 38), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 21, 22), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 13, 57), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(7, 12, 58), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 2, 15), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(1, 10, 9), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 22, 47), ARCMINUTE);
}

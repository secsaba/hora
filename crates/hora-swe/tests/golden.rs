//! Golden charts from P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*,
//! pinned exactly as the book prints them.

use hora_swe::{Request, calculate};

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

/// Chart 1: ch. 9 *Arudha Padas*, p. 87 (the book's chart 2 is the same chart).
#[tokio::test]
async fn chart_1_wilmington_2000() {
    // Printed: 2000-04-09 17:55 at UTC−4:00, so 21:55 UT. 71°W12′, 42°N30′. No altitude printed, so 0.
    let chart = calculate(Request {
        year: 2000,
        month: 4,
        day: 9,
        hour: 17 + 4,
        minute: 55,
        second: 0,
        latitude: 42.5,
        longitude: -(71.0 + 12.0 / 60.0),
        altitude: 0.0,
    })
    .await
    .unwrap();

    // One unit of the book's last printed digit: it rounds some values and truncates others.
    let tol = 0.01;
    let g = &chart.grahas;
    assert_arcsec("lagna", chart.lagna, dms(5, 10, 49, 44.53), tol);
    assert_arcsec("sun", g.sun.longitude, dms(11, 26, 30, 31.79), tol);
    assert_arcsec("moon", g.moon.longitude, dms(2, 4, 45, 58.43), tol);
    assert_arcsec("mars", g.mars.longitude, dms(0, 19, 10, 12.75), tol);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 1, 37, 2.96), tol);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 17, 22, 39.78), tol);
    assert_arcsec("venus", g.venus.longitude, dms(11, 10, 2, 30.56), tol);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 22, 42, 46.33), tol);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 5, 56, 6.19), tol);
    assert_arcsec("ayanamsha", chart.ayanamsha, dms(0, 23, 50, 42.57), tol);
}

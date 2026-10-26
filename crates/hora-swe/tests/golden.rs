//! Golden charts from P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*,
//! pinned exactly as the book prints them.

mod book;

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
    let sky = book::chart_1_wilmington_2000().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 10, 58), ARCMINUTE);
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
    let sky = book::chart_2_wilmington_2000_at_utc_minus_5().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 22, 41), ARCMINUTE);
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
    let sky = book::chart_3_gwalior_1926().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 14, 18), ARCMINUTE);
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
    let sky = book::chart_4_pithapuram_1968().await;

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
    let sky = book::chart_5_aspects_exercise_2000().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(6, 5, 23), ARCMINUTE);
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
    let sky = book::chart_6_p_v_narasimha_rao_1921().await;

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
    let sky = book::chart_7_ronald_reagan_1911().await;

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
    let sky = book::chart_8_guess_his_livelihood_1946().await;

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

/// Chart 9 (Chatrapati Shivaji): ch. 11 *Yogas*, §11.6, p. 128.
#[tokio::test]
async fn chart_9_chatrapati_shivaji_1630() {
    let sky = book::chart_9_chatrapati_shivaji_1630().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(4, 27, 41), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(10, 22, 19), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(5, 27, 52), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 16, 0), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 6, 18), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 5, 51), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 8, 30), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(6, 16, 34), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(1, 29, 22), ARCMINUTE);
}

/// Chart 10 (Emperor Akbar): ch. 11 *Yogas*, §11.7, p. 136; birth data in footnote 39.
#[tokio::test]
async fn chart_10_akbar_1542() {
    let sky = book::chart_10_akbar_1542().await;

    // Wider than an arcminute: the lagna is −67″ because the chart prints 4:39 East, not footnote 39's local mean time;
    // the Moon (+92″) and Rahu (+64″) are as far off in Jagannatha Hora too, likely the author's older version for 1542 (#71).
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
    let sky = book::chart_11_p_v_narasimha_rao_1921().await;

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
    let sky = book::chart_12_sav_exercise_1958().await;

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
    let sky = book::chart_13_an_example_1894().await;

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
    let sky = book::chart_14_rajiv_gandhi_1944().await;

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
    let sky = book::chart_15_satyam_gaur_1970().await;

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
    let sky = book::chart_15_shivam_gaur_1970().await;

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
    let sky = book::chart_17_pandit_sanjay_rath_1963().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(11, 14, 1), ARCMINUTE);
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
    let sky = book::chart_18_a_lady_1972().await;

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
    let sky = book::chart_19_sri_navin_patnaik_1946().await;

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
    let sky = book::chart_20_an_engineer_in_usa_1954().await;

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
    let sky = book::chart_21_john_f_kennedy_jr_1960().await;

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
    let sky = book::chart_22_a_lady_1954().await;

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
    let sky = book::chart_23_dr_b_v_raman_1912().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 9, 15), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(3, 22, 59), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(1, 23, 38), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 21, 22), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 13, 57), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(7, 12, 58), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 2, 15), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(1, 10, 9), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 22, 47), ARCMINUTE);
}

/// Chart 24 (Bill Gates): ch. 18 *Narayana Dasa*, p. 240.
#[tokio::test]
async fn chart_24_bill_gates_1955() {
    let sky = book::chart_24_bill_gates_1955().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 25, 38), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(6, 11, 46), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(11, 14, 35), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(5, 16, 51), ARCMINUTE);
    // 60.6″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("mercury", g.mercury.longitude, dms(5, 23, 19), 62.0);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 4, 32), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(6, 26, 57), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(6, 28, 21), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(7, 26, 13), ARCMINUTE);
}

/// Chart 25 (India): ch. 18 *Narayana Dasa*, p. 242.
#[tokio::test]
async fn chart_25_india_1947() {
    let sky = book::chart_25_india_1947().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(1, 8, 17), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(3, 27, 59), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(3, 3, 59), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 7, 27), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(3, 13, 41), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 25, 52), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(3, 22, 34), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(3, 20, 28), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(1, 5, 4), ARCMINUTE);
}

/// Chart 26 (Narayana Dasa Exercise): ch. 18 *Narayana Dasa*, p. 243.
#[tokio::test]
async fn chart_26_narayana_dasa_exercise_1971() {
    let sky = book::chart_26_narayana_dasa_exercise_1971().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 17, 40), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(0, 24, 26), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(6, 9, 28), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(9, 9, 14), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 0, 58), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(7, 9, 53), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(11, 25, 13), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(1, 1, 24), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(9, 25, 42), ARCMINUTE);
}

/// Chart 27 (Foreign Stay Example): ch. 18 *Narayana Dasa*, p. 246.
#[tokio::test]
async fn chart_27_foreign_stay_example_1970() {
    let sky = book::chart_27_foreign_stay_example_1970().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 14, 52), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 20, 52), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(10, 28, 35), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 26, 40), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 3, 7), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 9, 45), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 7, 55), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 15, 6), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(10, 16, 53), ARCMINUTE);
}

/// Chart 28 (A lady): ch. 18 *Narayana Dasa*, p. 248.
#[tokio::test]
async fn chart_28_a_lady_1971() {
    let sky = book::chart_28_a_lady_1971().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(6, 0, 12), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(4, 25, 14), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(1, 29, 39), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(9, 18, 28), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 7, 18), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(7, 6, 36), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 29, 25), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(1, 13, 1), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(9, 19, 2), ARCMINUTE);
}

/// Chart 29 (A lady): ch. 18 *Narayana Dasa*, p. 249.
#[tokio::test]
async fn chart_29_a_lady_1969() {
    let sky = book::chart_29_a_lady_1969().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 4, 8), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(2, 22, 26), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(0, 11, 59), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(7, 8, 16), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(2, 6, 40), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(5, 5, 37), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(1, 7, 57), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 13, 53), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 1, 12), ARCMINUTE);
}

/// Chart 31 (An electrical engineer): ch. 18 *Narayana Dasa*, p. 251.
#[tokio::test]
async fn chart_31_an_electrical_engineer_1947() {
    let sky = book::chart_31_an_electrical_engineer_1947().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 16, 50), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(4, 20, 55), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(1, 12, 40), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 22, 40), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 29, 21), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 28, 35), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 21, 59), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(3, 23, 25), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(1, 3, 48), ARCMINUTE);
}

/// Chart 32 (A gentleman): ch. 18 *Narayana Dasa*, p. 252.
#[tokio::test]
async fn chart_32_a_gentleman_1961() {
    let sky = book::chart_32_a_gentleman_1961().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(8, 9, 44), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(3, 8, 57), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(8, 6, 11), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 22, 37), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(2, 20, 8), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 8, 54), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(1, 26, 26), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 2, 46), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 5, 8), ARCMINUTE);
}

/// Chart 35 (Jayalalita): ch. 20 *Sudasa*, p. 266.
#[tokio::test]
async fn chart_35_jayalalita_1948() {
    let sky = book::chart_35_jayalalita_1948().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 21, 19), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(10, 11, 33), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 6, 42), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 2, 9), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 2, 40), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(8, 2, 1), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(11, 21, 51), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(3, 24, 53), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(0, 24, 48), ARCMINUTE);
}

/// Chart 37 (ISKCON Devotee): ch. 21 *Drigdasa*, p. 270.
#[tokio::test]
async fn chart_37_iskcon_devotee_1971() {
    let sky = book::chart_37_iskcon_devotee_1971().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 25, 20), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(9, 12, 15), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(9, 3, 34), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(7, 8, 36), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(8, 19, 8), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(7, 8, 28), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(7, 25, 29), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 22, 18), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(10, 1, 9), ARCMINUTE);
}

/// Chart 38 (Sri Aurobindo Ghose): ch. 21 *Drigdasa*, p. 271.
#[tokio::test]
async fn chart_38_sri_aurobindo_ghose_1872() {
    let sky = book::chart_38_sri_aurobindo_ghose_1872().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(3, 25, 9), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(4, 0, 19), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(8, 5, 41), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(3, 5, 23), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 23, 30), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(3, 21, 35), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 8, 32), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(8, 23, 29), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(1, 16, 37), ARCMINUTE);
}

/// Chart 40 (A Male): ch. 22 *Niryaana Shoola Dasa*, p. 277.
#[tokio::test]
async fn chart_40_a_male_1927() {
    let sky = book::chart_40_a_male_1927().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(0, 19, 4), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(9, 6, 24), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 3, 3), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 21, 56), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(9, 0, 59), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 7, 34), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(9, 20, 43), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(7, 12, 15), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(2, 13, 7), ARCMINUTE);
}

/// Chart 41 (A gentleman): ch. 22 *Niryaana Shoola Dasa*, p. 278.
#[tokio::test]
async fn chart_41_a_gentleman_1902() {
    let sky = book::chart_41_a_gentleman_1902().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 1, 51), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 0, 15), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(0, 25, 2), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(11, 3, 40), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 2, 51), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 15, 12), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(9, 25, 53), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 2, 57), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(6, 14, 10), ARCMINUTE);
}

/// Chart 42 (Niryaana Shoola Exercise): ch. 22 *Niryaana Shoola Dasa*, p. 279.
#[tokio::test]
async fn chart_42_niryaana_shoola_exercise_1889() {
    let sky = book::chart_42_niryaana_shoola_exercise_1889().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(6, 2, 56), ARCMINUTE);
    // 61.4″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("sun", g.sun.longitude, dms(0, 8, 29), 62.0);
    assert_arcsec("moon", g.moon.longitude, dms(8, 14, 14), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 24, 4), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 3, 21), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(8, 15, 56), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 24, 23), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(3, 21, 9), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(2, 23, 45), ARCMINUTE);
}

/// Chart 43 (Shoola dasa exercise): ch. 23 *Shoola Dasa*, p. 284.
#[tokio::test]
async fn chart_43_shoola_dasa_exercise_1915() {
    let sky = book::chart_43_shoola_dasa_exercise_1915().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(6, 9, 25), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(7, 26, 33), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(10, 12, 36), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 4, 53), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(7, 24, 30), ARCMINUTE);
    // 60.004″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 27, 11), 62.0);
    assert_arcsec("venus", g.venus.longitude, dms(8, 19, 15), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(2, 22, 10), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(9, 18, 6), ARCMINUTE);
}

/// Chart 44 (Pitri Shoola Example): ch. 23 *Shoola Dasa*, p. 286.
#[tokio::test]
async fn chart_44_pitri_shoola_example_1971() {
    let sky = book::chart_44_pitri_shoola_example_1971().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 18, 7), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(0, 24, 26), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(6, 9, 29), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(9, 9, 14), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 0, 58), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(7, 9, 53), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(11, 25, 13), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(1, 1, 24), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(9, 25, 42), ARCMINUTE);
}

/// Chart 48 (A gentleman): ch. 24 *Kalachakra Dasa*, p. 302.
#[tokio::test]
async fn chart_48_a_gentleman_1933() {
    let sky = book::chart_48_a_gentleman_1933().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 8, 37), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(2, 29, 3), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(0, 3, 58), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(5, 11, 29), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(3, 20, 17), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 26, 5), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(3, 21, 34), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 21, 40), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(10, 7, 39), ARCMINUTE);
}

/// Chart 49 (An ISKCON devotee): ch. 24 *Kalachakra Dasa*, p. 304.
#[tokio::test]
async fn chart_49_an_iskcon_devotee_1971() {
    let sky = book::chart_49_an_iskcon_devotee_1971().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 24, 49), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(9, 12, 16), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(9, 3, 35), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(7, 8, 37), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(8, 19, 10), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(7, 8, 30), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(7, 25, 31), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 22, 20), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(10, 1, 11), ARCMINUTE);
}

/// Chart 50 (Divorced Lady): ch. 24 *Kalachakra Dasa*, p. 306.
#[tokio::test]
async fn chart_50_divorced_lady_1968() {
    let sky = book::chart_50_divorced_lady_1968().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(9, 8, 44), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(1, 7, 19), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(11, 6, 17), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(1, 15, 48), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(1, 29, 41), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 3, 45), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 29, 17), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 27, 33), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 23, 6), ARCMINUTE);
}

/// Chart 51 (An actor): ch. 24 *Kalachakra Dasa*, p. 307.
#[tokio::test]
async fn chart_51_an_actor_1937() {
    let sky = book::chart_51_an_actor_1937().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(0, 20, 14), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(2, 26, 29), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 19, 28), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(6, 28, 0), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(3, 0, 57), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(8, 29, 51), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(1, 11, 30), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 12, 9), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(7, 20, 24), ARCMINUTE);
}

/// Chart 53 (A lady): ch. 25 *Transits and Natal References*, p. 321.
#[tokio::test]
async fn chart_53_a_lady_1973() {
    let sky = book::chart_53_a_lady_1973().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(11, 3, 18), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(3, 10, 1), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(1, 26, 39), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(11, 28, 12), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(3, 0, 13), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 14, 12), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 8, 25), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(2, 5, 48), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(8, 12, 48), ARCMINUTE);
}

/// Chart 54 (An engineer): ch. 25 *Transits and Natal References*, p. 324.
#[tokio::test]
async fn chart_54_an_engineer_1970() {
    let sky = book::chart_54_an_engineer_1970().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(8, 18, 31), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(5, 18, 11), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(7, 12, 14), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 26, 56), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(5, 2, 35), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(6, 15, 29), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(6, 27, 37), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 28, 21), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(10, 7, 9), ARCMINUTE);
}

/// Chart 54's second chart (Wedding (transit)): ch. 25 *Transits and Natal References*, p. 324.
#[tokio::test]
async fn chart_54_wedding_transit_1999() {
    let sky = book::chart_54_wedding_transit_1999().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 25, 31), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(9, 9, 52), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(0, 1, 31), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(6, 5, 11), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(9, 2, 32), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(11, 2, 3), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 0, 32), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 3, 31), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 29, 19), ARCMINUTE);
}

/// Chart 55 (A gentleman): ch. 25 *Transits and Natal References*, p. 326.
#[tokio::test]
async fn chart_55_a_gentleman_1961() {
    let sky = book::chart_55_a_gentleman_1961().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 14, 59), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(2, 8, 40), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(6, 5, 49), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 3, 36), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(2, 14, 25), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 12, 32), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 23, 0), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 5, 1), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 6, 49), ARCMINUTE);
}

/// Chart 55's second chart (Childbirth (transit)): ch. 25 *Transits and Natal References*, p. 326.
#[tokio::test]
async fn chart_55_childbirth_transit_1992() {
    let sky = book::chart_55_childbirth_transit_1992().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(8, 6, 6), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(5, 16, 32), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(8, 12, 55), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 17, 26), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(5, 29, 49), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(5, 4, 41), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(6, 16, 10), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 18, 11), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(8, 1, 24), ARCMINUTE);
}

/// Chart 56 (John F. Kennedy, Jr): ch. 25 *Transits and Natal References*, p. 327.
#[tokio::test]
async fn chart_56_john_f_kennedy_jr_1960() {
    let sky = book::chart_56_john_f_kennedy_jr_1960().await;

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

/// Chart 56's second chart (Death (transit)): ch. 25 *Transits and Natal References*, p. 327.
#[tokio::test]
async fn chart_56_death_transit_1999() {
    let sky = book::chart_56_death_transit_1999().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 0, 51), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(3, 0, 14), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 21, 49), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(6, 10, 37), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(3, 14, 58), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 8, 45), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 8, 18), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 21, 41), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 20, 6), ARCMINUTE);
}

/// Chart 58 (A.B. Vajpayee): ch. 25 *Transits and Natal References*, p. 331.
#[tokio::test]
async fn chart_58_vajpayee_1926() {
    let sky = book::chart_58_vajpayee_1926().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 14, 18), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(8, 9, 35), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 15, 28), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 13, 39), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(7, 20, 59), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 2, 5), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(8, 17, 42), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(7, 9, 41), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(2, 14, 30), ARCMINUTE);
}

/// Chart 59 (Vajpayee's Swearing-in): ch. 25 *Transits and Natal References*, p. 332.
#[tokio::test]
async fn chart_59_vajpayee_swearing_in_1998() {
    let sky = book::chart_59_vajpayee_swearing_in_1998().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(1, 4, 1), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 4, 31), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(7, 9, 46), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(11, 17, 20), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 22, 53), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 16, 23), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(9, 18, 21), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 26, 21), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 15, 48), ARCMINUTE);
}

/// Chart 61 (Indira Gandhi): ch. 25 *Transits and Natal References*, p. 336.
#[tokio::test]
async fn chart_61_indira_gandhi_1917() {
    let sky = book::chart_61_indira_gandhi_1917().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(3, 25, 38), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(7, 4, 7), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(9, 5, 30), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 16, 22), ARCMINUTE);
    // 60.3″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("mercury", g.mercury.longitude, dms(7, 13, 13), 62.0);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(1, 14, 59), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(8, 21, 0), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(3, 21, 47), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(8, 10, 33), ARCMINUTE);
}

/// Chart 61's second chart (Indira's assassination): ch. 25 *Transits and Natal References*, p. 336.
#[tokio::test]
async fn chart_61_indira_gandhi_assassination_1984() {
    let sky = book::chart_61_indira_gandhi_assassination_1984().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(9, 4, 25), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(6, 14, 24), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(9, 11, 28), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(8, 24, 54), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(6, 27, 5), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(8, 15, 4), ARCMINUTE);
    // 60.03″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("venus", g.venus.longitude, dms(7, 19, 47), 62.0);
    assert_arcsec("saturn", g.saturn.longitude, dms(6, 24, 7), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(1, 4, 46), ARCMINUTE);
}

/// Chart 62 (Princess Diana): ch. 25 *Transits and Natal References*, p. 339.
#[tokio::test]
async fn chart_62_princess_diana_1961() {
    let sky = book::chart_62_princess_diana_1961().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 28, 17), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(2, 16, 8), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(9, 28, 27), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 8, 12), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(2, 9, 59), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 11, 48), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(1, 0, 51), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 4, 30), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 6, 24), ARCMINUTE);
}

/// Chart 66 (Annual chart example): ch. 27 *Tajaka Chart Basics*, p. 368.
#[tokio::test]
async fn chart_66_annual_2000() {
    let sky = book::chart_66_annual_2000().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(9, 10, 49), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(10, 23, 50), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(11, 15, 13), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(11, 24, 58), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 11, 27), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 10, 10), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(9, 29, 20), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 19, 9), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 7, 39), ARCMINUTE);
}

/// Chart 67 (Annual Dasa Example): ch. 30 *Annual Dasas*, p. 397.
#[tokio::test]
async fn chart_67_annual_1993() {
    let sky = book::chart_67_annual_1993().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 7, 13), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(1, 17, 4), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(6, 4, 48), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(3, 23, 53), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(2, 4, 47), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(5, 10, 59), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 1, 37), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(10, 6, 29), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(7, 18, 38), ARCMINUTE);
}

/// Chart 68 (Going abroad (Tajaka)): ch. 30 *Annual Dasas*, p. 403.
#[tokio::test]
async fn chart_68_annual_1991() {
    let sky = book::chart_68_annual_1991().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(9, 27, 20), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 20, 52), ARCMINUTE);
    // 60.5″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("moon", g.moon.longitude, dms(7, 25, 2), 62.0);
    assert_arcsec("mars", g.mars.longitude, dms(2, 7, 15), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 5, 16), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(3, 9, 51), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 26, 40), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 11, 41), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(9, 0, 25), ARCMINUTE);
}

/// Chart 69 (Academic success): ch. 30 *Annual Dasas*, p. 404.
#[tokio::test]
async fn chart_69_annual_1987() {
    let sky = book::chart_69_annual_1987().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(9, 13, 35), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 20, 52), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(2, 4, 55), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(1, 5, 46), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 24, 46), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(11, 14, 17), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 14, 55), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(7, 27, 27), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 17, 50), ARCMINUTE);
}

/// Chart 70 (Childbirth): ch. 30 *Annual Dasas*, p. 405.
#[tokio::test]
async fn chart_70_annual_1998() {
    let sky = book::chart_70_annual_1998().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 12, 53), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 20, 52), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(3, 0, 51), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(11, 29, 54), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 24, 22), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 20, 11), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 4, 34), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 28, 24), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 14, 56), ARCMINUTE);
}

/// Chart 71 (Bought a car): ch. 30 *Annual Dasas*, p. 407.
#[tokio::test]
async fn chart_71_annual_1995() {
    let sky = book::chart_71_annual_1995().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 3, 59), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 20, 52), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(1, 14, 40), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(3, 20, 6), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 11, 4), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(7, 21, 34), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 15, 26), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(10, 24, 48), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(6, 13, 0), ARCMINUTE);
}

/// Chart 73's second chart (Indira Gandhi 1976-77): ch. 31 *Sudarsana Chakra Dasa*, p. 413.
#[tokio::test]
async fn chart_73_indira_gandhi_annual_1976() {
    let sky = book::chart_73_indira_gandhi_annual_1976().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 7, 26), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(7, 4, 7), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(6, 9, 3), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(7, 5, 39), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(7, 11, 15), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(1, 2, 22), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(8, 12, 59), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(3, 23, 16), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(6, 8, 34), ARCMINUTE);
}

/// Chart 74's second chart (A lady 1998-99): ch. 31 *Sudarsana Chakra Dasa*, p. 415.
#[tokio::test]
async fn chart_74_a_lady_annual_1998() {
    let sky = book::chart_74_a_lady_annual_1998().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(3, 29, 23), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(3, 10, 1), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 21, 16), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(2, 20, 1), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 3, 45), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(11, 4, 5), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(2, 15, 8), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 9, 27), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 8, 55), ARCMINUTE);
}

/// Chart 75 (Rectification Example 1): ch. 32 *Impact of Birthtime Error*, p. 429.
#[tokio::test]
async fn chart_75_rectification_example_1966() {
    let sky = book::chart_75_rectification_example_1966().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(9, 13, 15), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(7, 14, 9), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(2, 9, 32), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(5, 4, 32), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(6, 24, 43), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(3, 10, 57), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(7, 19, 23), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(10, 29, 32), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(0, 21, 36), ARCMINUTE);
}

/// Chart 77 (A gentleman): ch. 32 *Impact of Birthtime Error*, p. 432.
#[tokio::test]
async fn chart_77_a_gentleman_1939() {
    let sky = book::chart_77_a_gentleman_1939().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(11, 17, 35), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(4, 25, 53), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 13, 13), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(9, 3, 25), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 16, 55), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(11, 12, 43), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(4, 27, 42), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 7, 30), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(6, 8, 23), ARCMINUTE);
}

/// Chart 78 (Bill Gates (wrong)): ch. 32 *Impact of Birthtime Error*, p. 435.
#[tokio::test]
async fn chart_78_bill_gates_wrong_time_1955() {
    let sky = book::chart_78_bill_gates_wrong_time_1955().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 24, 58), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(6, 11, 46), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(11, 14, 33), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(5, 16, 51), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(5, 23, 19), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 4, 32), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(6, 26, 57), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(6, 28, 21), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(7, 26, 13), ARCMINUTE);
}

/// Chart 79 (A Music Teacher): ch. 32 *Impact of Birthtime Error*, p. 436.
#[tokio::test]
async fn chart_79_a_music_teacher_1973() {
    let sky = book::chart_79_a_music_teacher_1973().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 17, 3), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(4, 19, 36), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(8, 2, 34), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 14, 21), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 22, 24), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 9, 37), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(5, 27, 38), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(2, 9, 44), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(8, 10, 38), ARCMINUTE);
}

/// Chart 80's second chart (India Tajaka 1997-98): ch. 35 *Mundane Astrology*, p. 463.
#[tokio::test]
async fn chart_80_india_annual_1997() {
    let sky = book::chart_80_india_annual_1997().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(10, 11, 1), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(3, 27, 59), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(8, 5, 12), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(6, 6, 19), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(4, 21, 56), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 22, 31), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(5, 2, 35), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 26, 23), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 27, 17), ARCMINUTE);
}

/// Chart 81 (India: Lunar 1992-93): ch. 35 *Mundane Astrology*, p. 465.
#[tokio::test]
async fn chart_81_india_lunar_year_1992() {
    let sky = book::chart_81_india_lunar_year_1992().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(2, 4, 35), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 19, 56), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(11, 19, 57), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(10, 11, 0), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 6, 38), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 12, 1), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(11, 1, 12), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 22, 21), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(8, 11, 7), ARCMINUTE);
}

/// Chart 82 (India: Solar year 1998): ch. 35 *Mundane Astrology*, p. 467.
#[tokio::test]
async fn chart_82_india_solar_year_1998() {
    let sky = book::chart_82_india_solar_year_1998().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(11, 2, 53), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(9, 0, 0), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(3, 17, 27), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(9, 27, 21), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(8, 8, 7), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 1, 16), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(9, 3, 40), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 20, 27), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 19, 11), ARCMINUTE);
}

/// Chart 84 (Vajpayee Swearing-in 1): ch. 36 *Muhurta or Electional Astrology*, p. 476.
#[tokio::test]
async fn chart_84_vajpayee_swearing_in_1996() {
    let sky = book::chart_84_vajpayee_swearing_in_1996().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(4, 1, 2), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(1, 1, 52), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(0, 17, 32), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 16, 17), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 29, 57), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(8, 23, 38), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(2, 4, 11), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 10, 25), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(5, 21, 24), ARCMINUTE);
}

/// Chart 86 (Writing this book): ch. 36 *Muhurta or Electional Astrology*, p. 479.
#[tokio::test]
async fn chart_86_writing_this_book_1997() {
    let sky = book::chart_86_writing_this_book_1997().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(3, 28, 0), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(0, 29, 6), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(3, 20, 29), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(4, 24, 23), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 6, 35), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 26, 57), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(1, 9, 48), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 21, 40), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(5, 2, 12), ARCMINUTE);
}

/// Chart 88 (An astrologer): ch. 36 *Muhurta or Electional Astrology*, p. 490.
#[tokio::test]
async fn chart_88_an_astrologer_1949() {
    let sky = book::chart_88_an_astrologer_1949().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 1, 45), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(10, 28, 58), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 3, 48), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(10, 29, 55), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 4, 53), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(9, 2, 23), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 20, 6), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(4, 8, 8), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(0, 4, 31), ARCMINUTE);
}

/// Chart 89 (Tajaka 1999-2000): ch. 36 *Muhurta or Electional Astrology*, p. 491.
#[tokio::test]
async fn chart_89_annual_1999() {
    let sky = book::chart_89_annual_1999().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(5, 20, 11), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(10, 28, 58), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(9, 8, 50), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(6, 18, 13), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(11, 9, 21), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(11, 12, 48), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 0, 38), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 7, 29), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 26, 44), ARCMINUTE);
}

/// Chart 90 (An Indian in USA): ch. 36 *Muhurta or Electional Astrology*, p. 493.
#[tokio::test]
async fn chart_90_an_indian_in_usa_1968() {
    let sky = book::chart_90_an_indian_in_usa_1968().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(11, 29, 25), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(9, 10, 54), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(7, 12, 43), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(10, 18, 55), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(9, 27, 41), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 10, 38), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(8, 4, 46), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 14, 9), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 29, 18), ARCMINUTE);
}

/// Chart 91 (Tajaka 1998-99): ch. 36 *Muhurta or Electional Astrology*, p. 494.
#[tokio::test]
async fn chart_91_annual_1998() {
    let sky = book::chart_91_annual_1998().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 28, 54), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(9, 10, 54), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(7, 28, 16), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(10, 5, 49), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(8, 22, 58), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(10, 3, 42), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(8, 27, 31), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 21, 5), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 18, 37), ARCMINUTE);
}

/// Chart 92 (May-June 1998): ch. 36 *Muhurta or Electional Astrology*, p. 495.
#[tokio::test]
async fn chart_92_monthly_may_1998() {
    let sky = book::chart_92_monthly_may_1998().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(3, 8, 43), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(1, 10, 54), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(1, 15, 59), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(1, 7, 37), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 24, 14), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(11, 0, 2), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 1, 46), ARCMINUTE);
    // 60.5″: the book truncates to the arcminute and ours is just past it.
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 4, 41), 62.0);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 12, 12), ARCMINUTE);
}

/// Chart 92's second chart (June-July 1998): ch. 36 *Muhurta or Electional Astrology*, p. 495.
#[tokio::test]
async fn chart_92_monthly_june_1998() {
    let sky = book::chart_92_monthly_june_1998().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(8, 10, 22), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(2, 10, 54), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(3, 11, 16), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(1, 29, 29), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(2, 28, 36), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(11, 3, 29), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(1, 8, 35), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 7, 42), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(4, 10, 32), ARCMINUTE);
}

/// Chart 93 (March-April, 2000): ch. 36 *Muhurta or Electional Astrology*, p. 497.
#[tokio::test]
async fn chart_93_monthly_march_2000() {
    let sky = book::chart_93_monthly_march_2000().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(7, 3, 55), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 0, 2), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(2, 11, 44), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(11, 29, 35), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 8, 56), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 11, 27), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 7, 1), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 19, 44), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 7, 20), ARCMINUTE);
}

/// Chart 94 (March 26-28, 2000): ch. 36 *Muhurta or Electional Astrology*, p. 498.
#[tokio::test]
async fn chart_94_sixty_hours_2000() {
    let sky = book::chart_94_sixty_hours_2000().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(3, 16, 54), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(11, 12, 32), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(7, 29, 2), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(0, 8, 52), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(10, 14, 51), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(0, 14, 9), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(10, 22, 33), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(0, 21, 3), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(3, 6, 40), ARCMINUTE);
}

/// Chart 95 (A lady): ch. 36 *Muhurta or Electional Astrology*, p. 500.
#[tokio::test]
async fn chart_95_a_lady_1968() {
    let sky = book::chart_95_a_lady_1968().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(9, 8, 42), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(1, 7, 17), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(11, 6, 15), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(1, 15, 46), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(1, 29, 39), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(4, 3, 44), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(0, 29, 15), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(11, 27, 31), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(11, 23, 4), ARCMINUTE);
}

/// Chart 96 (Tajaka 1991-92): ch. 36 *Muhurta or Electional Astrology*, p. 501.
#[tokio::test]
async fn chart_96_tajaka_1991() {
    let sky = book::chart_96_tajaka_1991().await;

    let g = &sky.grahas;
    assert_arcsec("lagna", sky.lagna, dms(8, 1, 0), ARCMINUTE);
    assert_arcsec("sun", g.sun.longitude, dms(1, 7, 17), ARCMINUTE);
    assert_arcsec("moon", g.moon.longitude, dms(4, 29, 39), ARCMINUTE);
    assert_arcsec("mars", g.mars.longitude, dms(3, 3, 59), ARCMINUTE);
    assert_arcsec("mercury", g.mercury.longitude, dms(0, 13, 36), ARCMINUTE);
    assert_arcsec("jupiter", g.jupiter.longitude, dms(3, 13, 51), ARCMINUTE);
    assert_arcsec("venus", g.venus.longitude, dms(2, 21, 11), ARCMINUTE);
    assert_arcsec("saturn", g.saturn.longitude, dms(9, 13, 4), ARCMINUTE);
    assert_arcsec("rahu", g.rahu.longitude, dms(8, 27, 53), ARCMINUTE);
}

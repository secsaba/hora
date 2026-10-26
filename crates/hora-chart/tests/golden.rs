//! Facts the book shows for its example charts, derived from hora-swe's positions:
//! P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*.

use hora_chart::{Body, chart};
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

/// Chart 4 (an engineer in the US): ch. 9 *Arudha Padas*, p. 98.
#[tokio::test]
async fn chart_4_marks_only_jupiter_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Jupiter]);
}

/// Chart 6 (P.V. Narasimha Rao): ch. 10 *Aspects and Argalas*, p. 107.
#[tokio::test]
async fn chart_6_p_v_narasimha_rao_1921_marks_only_mercury_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Mercury]);
}

/// Chart 8 (Guess his livelihood!): ch. 10 *Aspects and Argalas*, p. 109.
#[tokio::test]
async fn chart_8_guess_his_livelihood_1946_marks_venus_saturn_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Venus, Body::Saturn]);
}

/// Chart 11 (Mercury's Ashtakavarga): ch. 12 *Ashtakavarga*, p. 152.
#[tokio::test]
async fn chart_11_p_v_narasimha_rao_1921_marks_only_mercury_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Mercury]);
}

/// Chart 12 (SAV Exercise): ch. 12 *Ashtakavarga*, p. 155.
#[tokio::test]
async fn chart_12_sav_exercise_1958_marks_mercury_saturn_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Mercury, Body::Saturn]);
}

/// Chart 13 (An example): ch. 13 *Interpreting Charts*, p. 173.
#[tokio::test]
async fn chart_13_an_example_1894_marks_only_saturn_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Saturn]);
}

/// Chart 15 (Satyam Gaur): ch. 13 *Interpreting Charts*, p. 177.
#[tokio::test]
async fn chart_15_satyam_gaur_1970_marks_venus_saturn_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Venus, Body::Saturn]);
}

/// Chart 15's second chart (Shivam Gaur): ch. 13 *Interpreting Charts*, p. 177.
#[tokio::test]
async fn chart_15_shivam_gaur_1970_marks_venus_saturn_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Venus, Body::Saturn]);
}

/// Chart 17 (Pandit Sanjay Rath): ch. 16 *Vimsottari Dasa*, p. 216.
#[tokio::test]
async fn chart_17_pandit_sanjay_rath_1963_marks_only_saturn_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Saturn]);
}

/// Chart 18 (A lady): ch. 16 *Vimsottari Dasa*, p. 217.
#[tokio::test]
async fn chart_18_a_lady_1972_marks_jupiter_venus_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Jupiter, Body::Venus]);
}

/// Chart 20 (An engineer in USA): ch. 16 *Vimsottari Dasa*, p. 220.
#[tokio::test]
async fn chart_20_an_engineer_in_usa_1954_marks_only_venus_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Venus]);
}

/// Chart 21 (John F. Kennedy, Jr.): ch. 16 *Vimsottari Dasa*, p. 222.
#[tokio::test]
async fn chart_21_john_f_kennedy_jr_1960_marks_only_mars_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Mars]);
}

/// Chart 22 (A lady): ch. 16 *Vimsottari Dasa*, p. 223.
#[tokio::test]
async fn chart_22_a_lady_1954_marks_only_jupiter_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Jupiter]);
}

/// Chart 23 (Dr. B.V. Raman): ch. 16 *Vimsottari Dasa*, p. 224.
#[tokio::test]
async fn chart_23_dr_b_v_raman_1912_marks_only_mercury_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Mercury]);
}

/// Chart 26 (Narayana Dasa Exercise): ch. 18 *Narayana Dasa*, p. 243.
#[tokio::test]
async fn chart_26_narayana_dasa_exercise_1971_marks_only_jupiter_retrograde() {
    // Printed: 1971-05-09 09:20 IST (UTC+5:30). 81°E12′, 16°N15′.
    let sky = book_sky(
        (1971, 5, 9),
        (9, 20, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    )
    .await;

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Jupiter]);
}

/// Chart 27 (Foreign Stay Example): ch. 18 *Narayana Dasa*, p. 246.
#[tokio::test]
async fn chart_27_foreign_stay_example_1970_marks_only_jupiter_retrograde() {
    // Printed: 1970-04-04 17:50 IST (UTC+5:30). 81°E12′, 16°N15′.
    let sky = book_sky(
        (1970, 4, 4),
        (17, 50, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    )
    .await;

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Jupiter]);
}

/// Chart 29 (A lady): ch. 18 *Narayana Dasa*, p. 249.
#[tokio::test]
async fn chart_29_a_lady_1969_marks_only_mars_retrograde() {
    // Printed: 1969-07-08 10:47 IST (UTC+5:30). 82°E15′, 16°N57′.
    let sky = book_sky(
        (1969, 7, 8),
        (10, 47, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 57.0 / 60.0,
        82.0 + 15.0 / 60.0,
    )
    .await;

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Mars]);
}

/// Chart 32 (A gentleman): ch. 18 *Narayana Dasa*, p. 252.
#[tokio::test]
async fn chart_32_a_gentleman_1961_marks_jupiter_saturn_retrograde() {
    // Printed: 1961-07-25 17:10 IST (UTC+5:30). 75°E50′, 22°N44′.
    let sky = book_sky(
        (1961, 7, 25),
        (17, 10, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        22.0 + 44.0 / 60.0,
        75.0 + 50.0 / 60.0,
    )
    .await;

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Jupiter, Body::Saturn]);
}

/// Chart 35 (Jayalalita): ch. 20 *Sudasa*, p. 266.
#[tokio::test]
async fn chart_35_jayalalita_1948_marks_mars_mercury_saturn_retrograde() {
    // Printed: 1948-02-24 14:36 IST (UTC+5:30). 80°E18′, 13°N05′.
    let sky = book_sky(
        (1948, 2, 24),
        (14, 36, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        13.0 + 5.0 / 60.0,
        80.0 + 18.0 / 60.0,
    )
    .await;

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Mars, Body::Mercury, Body::Saturn]);
}

/// Chart 38 (Sri Aurobindo Ghose): ch. 21 *Drigdasa*, p. 271.
#[tokio::test]
async fn chart_38_sri_aurobindo_ghose_1872_marks_only_saturn_retrograde() {
    // Printed: 1872-08-15 05:17 at UTC+5:53. 88°E20′, 22°N30′.
    let sky = book_sky(
        (1872, 8, 15),
        (5, 17, 0),
        UtcOffset::Stated(5 * 3600 + 53 * 60),
        Calendar::Gregorian,
        22.0 + 30.0 / 60.0,
        88.0 + 20.0 / 60.0,
    )
    .await;

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Saturn]);
}

/// Chart 42 (Niryaana Shoola Exercise): ch. 22 *Niryaana Shoola Dasa*, p. 279.
#[tokio::test]
async fn chart_42_niryaana_shoola_exercise_1889_marks_only_venus_retrograde() {
    // Printed: 1889-04-20 18:30 at UTC+1:00. 13°E02′, 48°N15′.
    let sky = book_sky(
        (1889, 4, 20),
        (18, 30, 0),
        UtcOffset::Stated(3600),
        Calendar::Gregorian,
        48.0 + 15.0 / 60.0,
        13.0 + 2.0 / 60.0,
    )
    .await;

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Venus]);
}

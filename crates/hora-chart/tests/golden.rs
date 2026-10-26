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

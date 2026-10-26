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

//! Facts the book shows for its example charts, derived from hora-swe's positions:
//! P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*.

use hora_chart::{Body, chart};
use hora_swe::{Ayanamsa, Calendar, NodeType, Request, SunriseDefinition, UtcOffset, calculate};

/// Chart 4 (an engineer in the US): ch. 9 *Arudha Padas*, p. 98.
#[tokio::test]
async fn chart_4_marks_only_jupiter_retrograde() {
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

    let marked: Vec<_> = chart(&sky)
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    assert_eq!(marked, [Body::Jupiter]);
}

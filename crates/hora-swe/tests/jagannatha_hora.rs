//! Not book values: checks against the author's software, Jagannatha Hora.

use hora_swe::{
    Ayanamsa, Calendar, NodeType, Request, SunriseDefinition, UtcOffset, calculate, julian_day,
};

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

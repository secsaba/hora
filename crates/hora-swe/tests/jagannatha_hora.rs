//! Not book values: checks against the author's software, Jagannatha Hora.

use hora_swe::{
    Ayanamsa, Calendar, NodeType, Request, SunriseDefinition, UtcOffset, calculate, julian_day,
};

/// Jagannatha Hora's Sun has no aberration (about 1.5 s), which `swe_rise_trans` cannot drop,
/// and it prints whole seconds, possibly truncated (up to 1 s).
const SUN_EVENT_SECONDS: f64 = 3.0;

/// `jhora` is its local date and time; `utc_offset` the offset in hours.
#[track_caller]
fn assert_ut(
    name: &str,
    got: f64,
    [year, month, day, hour, minute, second]: [i32; 6],
    utc_offset: f64,
) {
    let hour = f64::from(hour) + f64::from(minute) / 60.0 + f64::from(second) / 3600.0;
    let jhora =
        julian_day(year, month, day, hour, Calendar::Gregorian).unwrap() - utc_offset / 24.0;
    let diff = (got - jhora) * 86400.0;
    assert!(
        diff.abs() <= SUN_EVENT_SECONDS,
        "{name}: got JD {got}, Jagannatha Hora JD {jhora}, off by {diff:.1} s"
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

/// Sidereal longitude from a sign (0 = Aries) and degrees, minutes, seconds within it.
fn dms(sign: u8, d: u8, m: u8, s: f64) -> f64 {
    f64::from(sign) * 30.0 + f64::from(d) + f64::from(m) / 60.0 + s / 3600.0
}

/// Jagannatha Hora's grahas for charts 1, 3 and 4. It ran on True Chitra, so these do too: the
/// ayanamsa is checked elsewhere; this catches a change to the flags (TRUEPOS), node or ephemeris.
#[tokio::test]
async fn charts_1_3_4_grahas_match_jagannatha_hora() {
    let chart =
        |[year, month, day, hour, minute]: [i32; 5], offset: i32, latitude, longitude| Request {
            year,
            month,
            day,
            hour,
            minute,
            second: 0,
            calendar: Calendar::Gregorian,
            utc_offset: UtcOffset::Stated(offset * 60),
            latitude,
            longitude,
            altitude: 0.0,
            ayanamsa: Ayanamsa::TrueChitra,
            node: NodeType::Mean,
            sunrise_definition: SunriseDefinition::UpperLimbApparent,
        };
    // Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu; tolerance in arcseconds.
    let charts = [
        (
            "chart 1",
            chart([2000, 4, 9, 17, 55], -240, 42.5, -(71.0 + 12.0 / 60.0)),
            [
                dms(11, 26, 30, 31.79),
                dms(2, 4, 45, 58.43),
                dms(0, 19, 10, 12.75),
                dms(11, 1, 37, 2.96),
                dms(0, 17, 22, 39.78),
                dms(11, 10, 2, 30.56),
                dms(0, 22, 42, 46.33),
                dms(3, 5, 56, 6.19),
            ],
            0.01,
        ),
        (
            "chart 3",
            chart(
                [1926, 12, 25, 5, 12],
                330,
                26.0 + 14.0 / 60.0,
                78.0 + 10.0 / 60.0,
            ),
            [
                dms(8, 9, 36, 47.60),
                dms(4, 15, 29, 36.26),
                dms(0, 13, 40, 32.96),
                dms(7, 21, 0, 29.70),
                dms(10, 2, 6, 36.83),
                dms(8, 17, 42, 58.68),
                dms(7, 9, 42, 15.18),
                dms(2, 14, 31, 38.45),
            ],
            // Its ΔT for 1926 is ~0.27 s above ours (Moon 0.15″ off).
            0.2,
        ),
        (
            "chart 4",
            chart(
                [1968, 3, 19, 18, 30],
                330,
                17.0 + 7.0 / 60.0,
                82.0 + 19.0 / 60.0,
            ),
            [
                dms(11, 5, 36, 3.87),
                dms(7, 10, 16, 23.62),
                dms(0, 0, 21, 8.03),
                dms(10, 8, 56, 58.73),
                dms(4, 4, 6, 57.69),
                dms(10, 11, 31, 1.54),
                dms(11, 19, 54, 0.04),
                dms(11, 26, 26, 16.85),
            ],
            0.01,
        ),
    ];
    for (name, request, jhora, tolerance) in charts {
        let g = calculate(request).await.unwrap().grahas;
        let got = [
            g.sun, g.moon, g.mars, g.mercury, g.jupiter, g.venus, g.saturn, g.rahu,
        ];
        let names = [
            "sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu",
        ];
        for ((graha, got), jhora) in names.iter().zip(got).zip(jhora) {
            let diff = (got.longitude - jhora) * 3600.0;
            assert!(
                diff.abs() <= tolerance,
                "{name} {graha}: got {}°, Jagannatha Hora {jhora}°, off by {diff:.4}″",
                got.longitude
            );
        }
    }
}

//! Not book values: hora's own lagna for the book's charts 1–4, pinned so any change to how it is computed fails here.

use hora_swe::{Ayanamsa, Calendar, NodeType, Request, SunriseDefinition, UtcOffset, calculate};

/// The printed local date and time, offset in minutes, latitude and longitude.
fn at(
    [year, month, day, hour, minute]: [i32; 5],
    offset: i32,
    latitude: f64,
    longitude: f64,
) -> Request {
    Request {
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
        ayanamsa: Ayanamsa::Lahiri,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    }
}

#[tokio::test]
async fn the_lagna_of_charts_1_to_4_is_unchanged() {
    let (lat, lon) = (42.5, -(71.0 + 12.0 / 60.0));
    let charts = [
        (
            "chart 1",
            at([2000, 4, 9, 17, 55], -240, lat, lon),
            160.817710058,
        ),
        (
            "chart 2",
            at([2000, 4, 9, 17, 55], -300, lat, lon),
            172.526708955,
        ),
        (
            "chart 3",
            at(
                [1926, 12, 25, 5, 12],
                330,
                26.0 + 14.0 / 60.0,
                78.0 + 10.0 / 60.0,
            ),
            224.207847240,
        ),
        (
            "chart 4",
            at(
                [1968, 3, 19, 18, 30],
                330,
                17.0 + 7.0 / 60.0,
                82.0 + 19.0 / 60.0,
            ),
            160.867594042,
        ),
    ];
    for (name, request, pinned) in charts {
        let lagna = calculate(request).await.unwrap().lagna;
        let diff = (lagna - pinned) * 3600.0;
        // 0.01″: far below any method change (nutation alone is ~16″), above libm differences between platforms.
        assert!(
            diff.abs() <= 0.01,
            "{name}: got {lagna}°, pinned {pinned}°, off by {diff:.4}″"
        );
    }
}

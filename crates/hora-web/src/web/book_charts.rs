//! The book's numbered charts for the knowledge base's appendix, in the book's order (docs/book-charts.md).

use fluent_templates::Loader;
use fluent_templates::fluent_bundle::FluentValue;
use hora_swe::{Ayanamsa, Calendar, NodeType, Request, SunriseDefinition, UtcOffset};

use super::{AYANAMSAS, CALENDARS, Choices, LOCALES, Lang, NODES, SUNRISES};

/// Where a chart's link gets its birth data.
pub(super) enum Data {
    /// As the golden fixture of this chart, seconds worked out included; a test holds the two together.
    Birth(Request),
    /// The same data as the listed chart with this key.
    Same(&'static str),
    /// Not enough printed to cast it.
    Missing,
}

pub(super) struct Chart {
    /// The book's number, with a suffix for a further chart printed under that number; names its description, `book-chart-{key}`.
    pub(super) key: &'static str,
    chapter: u8,
    page: u16,
    pub(super) data: Data,
}

/// The book's settings as in the golden fixtures: Lahiri, mean node, the upper limb rising, no altitude printed.
const fn birth(
    (year, month, day): (i32, i32, i32),
    (hour, minute, second): (i32, i32, i32),
    utc_offset: UtcOffset,
    calendar: Calendar,
    latitude: f64,
    longitude: f64,
) -> Request {
    Request {
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
    }
}

const fn own(key: &'static str, chapter: u8, page: u16, request: Request) -> Chart {
    Chart {
        key,
        chapter,
        page,
        data: Data::Birth(request),
    }
}

const fn same(key: &'static str, chapter: u8, page: u16, source: &'static str) -> Chart {
    Chart {
        key,
        chapter,
        page,
        data: Data::Same(source),
    }
}

const fn missing(key: &'static str, chapter: u8, page: u16) -> Chart {
    Chart {
        key,
        chapter,
        page,
        data: Data::Missing,
    }
}

#[rustfmt::skip]
pub(super) const CHARTS: &[Chart] = &[
    own("1", 9, 87, birth((2000, 4, 9), (17, 55, 48), UtcOffset::Stated(-4 * 3600), Calendar::Gregorian, 42.5, -(71.0 + 12.0 / 60.0))),
    own("2", 9, 89, birth((2000, 4, 9), (17, 55, 51), UtcOffset::Stated(-5 * 3600), Calendar::Gregorian, 42.5, -(71.0 + 12.0 / 60.0))),
    own("3", 9, 97, birth((1926, 12, 25), (5, 12, 30), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 26.0 + 14.0 / 60.0, 78.0 + 10.0 / 60.0)),
    own("4", 9, 98, birth((1968, 3, 19), (18, 30, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0 + 7.0 / 60.0, 82.0 + 19.0 / 60.0)),
    own("5", 10, 101, birth((2000, 4, 19), (19, 21, 57), UtcOffset::Stated(-4 * 3600), Calendar::Gregorian, 42.0 + 40.0 / 60.0, -(71.0 + 8.0 / 60.0))),
    own("6", 10, 107, birth((1921, 6, 28), (12, 49, 0), UtcOffset::Stated(5 * 3600 + 17 * 60), Calendar::Gregorian, 18.0 + 26.0 / 60.0, 79.0 + 9.0 / 60.0)),
    own("7", 10, 108, birth((1911, 2, 6), (2, 4, 0), UtcOffset::Stated(-6 * 3600), Calendar::Gregorian, 41.0 + 38.0 / 60.0, -(89.0 + 47.0 / 60.0))),
    own("8", 10, 109, birth((1946, 12, 2), (6, 45, 0), UtcOffset::Stated(3600), Calendar::Gregorian, 38.0 + 6.0 / 60.0, 15.0 + 39.0 / 60.0)),
    own("9", 11, 128, birth((1630, 2, 19), (18, 25, 32), UtcOffset::LocalMean, Calendar::Julian, 18.0 + 32.0 / 60.0, 73.0 + 53.0 / 60.0)),
    own("10", 11, 136, birth((1542, 11, 24), (3, 39, 0), UtcOffset::LocalMean, Calendar::Julian, 25.0 + 19.0 / 60.0, 69.0 + 47.0 / 60.0)),
    own("11", 12, 152, birth((1921, 6, 28), (13, 8, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 18.0 + 26.0 / 60.0, 79.0 + 9.0 / 60.0)),
    own("12", 12, 155, birth((1958, 8, 16), (7, 5, 0), UtcOffset::Stated(-4 * 3600), Calendar::Gregorian, 43.0 + 36.0 / 60.0, -(83.0 + 53.0 / 60.0))),
    own("13", 13, 173, birth((1894, 5, 20), (13, 22, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 11.0 + 57.0 / 60.0, 79.0 + 32.0 / 60.0)),
    own("14", 13, 175, birth((1944, 8, 20), (7, 11, 40), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 18.0 + 58.0 / 60.0, 72.0 + 49.0 / 60.0)),
    own("15", 13, 177, birth((1970, 11, 4), (16, 6, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 30.0 + 44.0 / 60.0, 76.0 + 53.0 / 60.0)),
    own("15-shivam", 13, 177, birth((1970, 11, 4), (16, 8, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 30.0 + 44.0 / 60.0, 76.0 + 53.0 / 60.0)),
    same("16", 13, 178, "15"),
    own("17", 16, 216, birth((1963, 8, 7), (21, 14, 50), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 21.0 + 27.0 / 60.0, 83.0 + 58.0 / 60.0)),
    own("18", 16, 217, birth((1972, 6, 1), (4, 16, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("19", 16, 219, birth((1946, 10, 16), (0, 58, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 20.0 + 30.0 / 60.0, 85.0 + 50.0 / 60.0)),
    own("20", 16, 220, birth((1954, 11, 12), (7, 52, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 12.0 + 30.0 / 60.0, 78.0 + 50.0 / 60.0)),
    own("21", 16, 222, birth((1960, 11, 25), (0, 22, 0), UtcOffset::Stated(-5 * 3600), Calendar::Gregorian, 38.0 + 54.0 / 60.0, -(77.0 + 2.0 / 60.0))),
    own("22", 16, 223, birth((1954, 12, 7), (2, 13, 0), UtcOffset::Stated(-6 * 3600), Calendar::Gregorian, 44.0 + 58.0 / 60.0, -(93.0 + 15.0 / 60.0))),
    own("23", 16, 224, birth((1912, 8, 8), (19, 38, 30), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 13.0, 77.0 + 35.0 / 60.0)),
    own("24", 18, 240, birth((1955, 10, 28), (21, 18, 22), UtcOffset::Stated(-8 * 3600), Calendar::Gregorian, 47.0 + 36.0 / 60.0, -(122.0 + 20.0 / 60.0))),
    own("25", 18, 242, birth((1947, 8, 15), (0, 0, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 27.0, 78.0 + 30.0 / 60.0)),
    own("26", 18, 243, birth((1971, 5, 9), (9, 20, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("27", 18, 246, birth((1970, 4, 4), (17, 50, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("28", 18, 248, birth((1971, 9, 12), (8, 25, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 13.0 / 60.0, 80.0 + 23.0 / 60.0)),
    own("29", 18, 249, birth((1969, 7, 8), (10, 47, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 57.0 / 60.0, 82.0 + 15.0 / 60.0)),
    missing("30", 18, 250),
    own("31", 18, 251, birth((1947, 9, 7), (18, 0, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 15.0 + 54.0 / 60.0, 80.0 + 30.0 / 60.0)),
    own("32", 18, 252, birth((1961, 7, 25), (17, 10, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 22.0 + 44.0 / 60.0, 75.0 + 50.0 / 60.0)),
    same("33", 18, 253, "27"),
    same("34", 19, 261, "7"),
    own("35", 20, 266, birth((1948, 2, 24), (14, 36, 40), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 13.0 + 5.0 / 60.0, 80.0 + 18.0 / 60.0)),
    missing("36", 21, 268),
    own("37", 21, 270, birth((1971, 1, 26), (10, 44, 0), UtcOffset::Stated(3 * 3600), Calendar::Gregorian, 49.0 + 49.0 / 60.0, 24.0 + 1.0 / 60.0)),
    own("38", 21, 271, birth((1872, 8, 15), (5, 17, 0), UtcOffset::Stated(5 * 3600 + 53 * 60), Calendar::Gregorian, 22.0 + 30.0 / 60.0, 88.0 + 20.0 / 60.0)),
    same("39", 22, 275, "14"),
    own("40", 22, 277, birth((1927, 1, 20), (12, 30, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("41", 22, 278, birth((1902, 3, 14), (11, 48, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("42", 22, 279, birth((1889, 4, 20), (18, 30, 0), UtcOffset::Stated(3600), Calendar::Gregorian, 48.0 + 15.0 / 60.0, 13.0 + 2.0 / 60.0)),
    own("43", 23, 284, birth((1915, 12, 12), (3, 19, 0), UtcOffset::Stated(-5 * 3600), Calendar::Gregorian, 40.0 + 45.0 / 60.0, -(74.0 + 2.0 / 60.0))),
    own("44", 23, 286, birth((1971, 5, 9), (9, 22, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    missing("45", 23, 287),
    same("46", 24, 300, "44"),
    same("47", 24, 301, "27"),
    own("48", 24, 302, birth((1933, 7, 15), (4, 15, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 5.0 / 60.0, 80.0 + 55.0 / 60.0)),
    own("49", 24, 304, Request { ayanamsa: Ayanamsa::TrueChitra, ..birth((1971, 1, 26), (10, 43, 0), UtcOffset::Stated(3 * 3600), Calendar::Gregorian, 49.0 + 49.0 / 60.0, 24.0 + 1.0 / 60.0) }),
    own("50", 24, 306, Request { ayanamsa: Ayanamsa::TrueChitra, ..birth((1968, 5, 21), (23, 5, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 18.0 + 40.0 / 60.0, 78.0 + 10.0 / 60.0) }),
    own("51", 24, 307, Request { ayanamsa: Ayanamsa::TrueChitra, ..birth((1937, 7, 12), (0, 30, 0), UtcOffset::Stated(-5 * 3600), Calendar::Gregorian, 39.0 + 57.0 / 60.0, -(75.0 + 10.0 / 60.0)) }),
    missing("52", 25, 318),
    own("53", 25, 321, birth((1973, 7, 26), (21, 41, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 13.0 / 60.0, 80.0 + 28.0 / 60.0)),
    own("54", 25, 324, birth((1970, 10, 5), (12, 34, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 15.0 + 49.0 / 60.0, 80.0 + 21.0 / 60.0)),
    own("54-transit", 25, 324, birth((1999, 1, 24), (9, 30, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0 + 20.0 / 60.0, 78.0 + 30.0 / 60.0)),
    own("55", 25, 326, birth((1961, 6, 23), (22, 46, 30), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0, 81.0 + 48.0 / 60.0)),
    own("55-transit", 25, 326, birth((1992, 10, 3), (11, 58, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0 + 29.0 / 60.0, 78.0 + 30.0 / 60.0)),
    own("56", 25, 327, birth((1960, 11, 25), (0, 22, 0), UtcOffset::Stated(-5 * 3600), Calendar::Gregorian, 38.0 + 53.0 / 60.0, -(77.0 + 2.0 / 60.0))),
    own("56-transit", 25, 327, birth((1999, 7, 16), (21, 45, 0), UtcOffset::Stated(-4 * 3600), Calendar::Gregorian, 42.0 + 30.0 / 60.0, -(71.0 + 12.0 / 60.0))),
    same("57", 25, 328, "56"),
    same("57-transit", 25, 328, "56-transit"),
    own("58", 25, 331, birth((1926, 12, 25), (5, 12, 26), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 26.0 + 13.0 / 60.0, 78.0 + 10.0 / 60.0)),
    own("59", 25, 332, birth((1998, 3, 19), (9, 32, 34), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 28.0 + 36.0 / 60.0, 77.0 + 12.0 / 60.0)),
    same("60", 25, 334, "14"),
    own("61", 25, 336, birth((1917, 11, 19), (23, 3, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 25.0 + 28.0 / 60.0, 81.0 + 52.0 / 60.0)),
    own("61-transit", 25, 336, birth((1984, 10, 31), (12, 30, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 28.0 + 36.0 / 60.0, 77.0 + 12.0 / 60.0)),
    own("62", 25, 339, birth((1961, 7, 1), (14, 25, 31), UtcOffset::Stated(3600), Calendar::Gregorian, 52.0 + 50.0 / 60.0, 0.0 + 30.0 / 60.0)),
    same("63", 25, 342, "27"),
    same("64", 25, 343, "27"),
    same("65", 25, 345, "27"),
    own("66", 27, 368, birth((2000, 3, 8), (4, 41, 21), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 26.0 + 18.0 / 60.0, 73.0 + 4.0 / 60.0)),
    own("67", 30, 397, birth((1993, 6, 1), (13, 30, 4), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("68", 30, 403, birth((1991, 4, 5), (3, 5, 33), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("69", 30, 404, birth((1987, 4, 5), (2, 15, 41), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("70", 30, 405, birth((1998, 4, 4), (21, 59, 49), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    own("71", 30, 407, birth((1995, 4, 5), (3, 28, 36), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 15.0 / 60.0, 81.0 + 12.0 / 60.0)),
    same("72", 31, 408, "17"),
    same("73", 31, 413, "61"),
    own("73-annual", 31, 413, birth((1976, 11, 20), (2, 10, 50), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 25.0 + 28.0 / 60.0, 81.0 + 52.0 / 60.0)),
    same("74", 31, 415, "53"),
    own("74-annual", 31, 415, birth((1998, 7, 27), (7, 15, 50), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 16.0 + 13.0 / 60.0, 80.0 + 28.0 / 60.0)),
    own("75", 32, 429, birth((1966, 11, 30), (10, 36, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 11.0 + 19.0 / 60.0, 77.0 + 47.0 / 60.0)),
    same("76", 32, 430, "75"),
    own("77", 32, 432, birth((1939, 9, 12), (19, 30, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 30.0 + 29.0 / 60.0, 76.0 + 36.0 / 60.0)),
    own("78", 32, 435, birth((1955, 10, 28), (21, 15, 0), UtcOffset::Stated(-8 * 3600), Calendar::Gregorian, 47.0 + 36.0 / 60.0, -(122.0 + 20.0 / 60.0))),
    own("79", 32, 436, birth((1973, 9, 6), (1, 32, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0 + 29.0 / 60.0, 78.0 + 30.0 / 60.0)),
    same("80", 35, 463, "25"),
    own("80-annual", 35, 463, birth((1997, 8, 14), (19, 30, 43), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 27.0, 78.0 + 30.0 / 60.0)),
    own("81", 35, 465, birth((1992, 4, 3), (10, 32, 11), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 28.0 + 36.0 / 60.0, 77.0 + 12.0 / 60.0)),
    own("82", 35, 467, birth((1998, 1, 14), (10, 38, 8), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 28.0 + 36.0 / 60.0, 77.0 + 12.0 / 60.0)),
    same("83", 35, 468, "59"),
    own("84", 36, 476, birth((1996, 5, 16), (12, 0, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 27.0, 78.0 + 30.0 / 60.0)),
    same("85", 36, 477, "59"),
    own("86", 36, 479, birth((1997, 5, 13), (12, 1, 28), UtcOffset::Stated(-4 * 3600), Calendar::Gregorian, 42.0 + 40.0 / 60.0, -(71.0 + 8.0 / 60.0))),
    same("87", 36, 488, "53"),
    own("88", 36, 490, birth((1949, 3, 13), (0, 32, 0), UtcOffset::Stated(0), Calendar::Gregorian, 53.0 + 20.0 / 60.0, -(8.0 + 13.0 / 60.0))),
    own("89", 36, 491, birth((1999, 3, 13), (20, 30, 20), UtcOffset::Stated(0), Calendar::Gregorian, 53.0 + 20.0 / 60.0, -(8.0 + 13.0 / 60.0))),
    own("90", 36, 493, birth((1968, 1, 25), (11, 14, 45), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0 + 29.0 / 60.0, 78.0 + 30.0 / 60.0)),
    own("91", 36, 494, birth((1998, 1, 25), (3, 59, 18), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0 + 29.0 / 60.0, 78.0 + 30.0 / 60.0)),
    own("92", 36, 495, birth((1998, 5, 26), (9, 55, 25), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0 + 29.0 / 60.0, 78.0 + 30.0 / 60.0)),
    own("92-june", 36, 495, birth((1998, 6, 26), (18, 47, 46), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 17.0 + 29.0 / 60.0, 78.0 + 30.0 / 60.0)),
    own("93", 36, 497, birth((2000, 3, 14), (0, 21, 20), UtcOffset::Stated(-4 * 3600), Calendar::Gregorian, 40.0 + 43.0 / 60.0, -74.0)),
    own("94", 36, 498, birth((2000, 3, 26), (14, 27, 45), UtcOffset::Stated(-4 * 3600), Calendar::Gregorian, 40.0 + 43.0 / 60.0, -74.0)),
    own("95", 36, 500, birth((1968, 5, 21), (23, 5, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 18.0 + 40.0 / 60.0, 78.0 + 10.0 / 60.0)),
    own("96", 36, 501, birth((1991, 5, 22), (20, 29, 0), UtcOffset::Stated(5 * 3600 + 30 * 60), Calendar::Gregorian, 18.0 + 40.0 / 60.0, 78.0 + 10.0 / 60.0)),
];

impl Chart {
    /// The number the book prints for it.
    fn number(&self) -> &'static str {
        self.key.split('-').next().unwrap_or(self.key)
    }

    /// The birth data its link opens; `None` when the book prints too little.
    pub(super) fn request(&'static self) -> Option<&'static Request> {
        match &self.data {
            Data::Birth(request) => Some(request),
            Data::Same(key) => CHARTS.iter().find(|chart| chart.key == *key)?.request(),
            Data::Missing => None,
        }
    }
}

/// The value a chart form field sends for `choice`.
fn form_value<T: Copy + PartialEq>(choices: &Choices<T>, choice: T) -> &'static str {
    choices
        .iter()
        .find(|(_, value, _)| *value == choice)
        .map_or("", |(value, ..)| value)
}

/// The chart page's query for a request, every setting stated, so a change of the form's defaults cannot move a chart.
fn query(r: &Request) -> String {
    let (utc, offset) = match r.utc_offset {
        UtcOffset::Stated(seconds) => {
            // A bare + in a query is a space.
            let sign = if seconds < 0 { "-" } else { "%2B" };
            let s = seconds.unsigned_abs();
            let text = format!("{sign}{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60);
            ("stated", text)
        }
        UtcOffset::LocalMean => ("local-mean", String::new()),
        UtcOffset::LocalApparent => ("local-apparent", String::new()),
    };
    format!(
        "calendar={}&date={:04}-{:02}-{:02}&time={:02}:{:02}:{:02}&utc={utc}&offset={offset}&latitude={}&longitude={}&altitude={}&ayanamsa={}&node={}&sunrise={}",
        form_value(CALENDARS, r.calendar),
        r.year,
        r.month,
        r.day,
        r.hour,
        r.minute,
        r.second,
        r.latitude,
        r.longitude,
        r.altitude,
        form_value(AYANAMSAS, r.ayanamsa),
        form_value(NODES, r.node),
        form_value(SUNRISES, r.sunrise_definition),
    )
}

/// One table of charts per book chapter, as Markdown.
pub(super) fn tables(lang: Lang) -> String {
    let t = |id: &str| LOCALES.lookup(&lang.id(), id);
    let with = |id: &str, name: &'static str, value: String| {
        let args = [(name.into(), FluentValue::from(value))].into();
        LOCALES.lookup_with_args(&lang.id(), id, &args)
    };
    let label = |number: &str| with("book-charts-chart", "chart", number.to_owned());
    let mut markdown = String::new();
    for charts in CHARTS.chunk_by(|a, b| a.chapter == b.chapter) {
        markdown += &format!(
            "\n\n## {}\n\n| {} | {} | {} |\n|---|---|---|\n",
            with(
                "book-charts-chapter",
                "chapter",
                charts[0].chapter.to_string()
            ),
            t("book-charts-column-chart"),
            t("book-charts-column-page"),
            t("book-charts-column-shows"),
        );
        for chart in charts {
            let name = label(chart.number());
            let cell = match chart.request() {
                Some(request) => format!("[{name}](/{}/chart?{})", lang.code(), query(request)),
                None => name,
            };
            let mut shows = t(&format!("book-chart-{}", chart.key));
            if let Data::Same(source) = chart.data {
                let source = label(source.split('-').next().unwrap_or(source));
                shows += &with("book-charts-same-data", "chart", source);
            }
            markdown += &format!("| {cell} | {} | {shows} |\n", chart.page);
        }
    }
    markdown
}

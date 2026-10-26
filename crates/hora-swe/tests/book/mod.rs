//! The printed birth data of the book's example charts, shared by the golden tests of hora-swe and
//! hora-chart so a chart is computed from the same data in both.

use hora_swe::{
    Ayanamsa, Calendar, NodeType, Request, Sky, SunriseDefinition, UtcOffset, calculate,
};

/// A chart's printed birth data (date, clock time, offset, calendar, latitude, longitude; no altitude is
/// printed, so 0), with the book's settings: Lahiri, mean node, the upper limb rising.
fn book_request(
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

async fn sky(request: Request) -> Sky {
    calculate(request).await.unwrap()
}

pub async fn chart_1_wilmington_2000() -> Sky {
    // Printed: 2000-04-09 17:55 at UTC−4:00. 71°W12′, 42°N30′.
    sky(book_request(
        (2000, 4, 9),
        (17, 55, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        42.5,
        -(71.0 + 12.0 / 60.0),
    ))
    .await
}

pub async fn chart_2_wilmington_2000_at_utc_minus_5() -> Sky {
    // Printed: 2000-04-09 17:55 at UTC−5:00. 71°W12′, 42°N30′.
    sky(book_request(
        (2000, 4, 9),
        (17, 55, 0),
        UtcOffset::Stated(-5 * 3600),
        Calendar::Gregorian,
        42.5,
        -(71.0 + 12.0 / 60.0),
    ))
    .await
}

pub async fn chart_3_gwalior_1926() -> Sky {
    // Printed: 1926-12-25 05:12 IST (UTC+5:30). 78°E10′, 26°N14′.
    sky(book_request(
        (1926, 12, 25),
        (5, 12, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        26.0 + 14.0 / 60.0,
        78.0 + 10.0 / 60.0,
    ))
    .await
}

pub async fn chart_4_pithapuram_1968() -> Sky {
    // Printed: 1968-03-19 18:30 IST (UTC+5:30). 82°E19′, 17°N07′.
    sky(book_request(
        (1968, 3, 19),
        (18, 30, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 7.0 / 60.0,
        82.0 + 19.0 / 60.0,
    ))
    .await
}

pub async fn chart_5_aspects_exercise_2000() -> Sky {
    // Printed: 2000-04-19 19:21 at UTC−4:00. 71°W08′, 42°N40′.
    sky(book_request(
        (2000, 4, 19),
        (19, 21, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        42.0 + 40.0 / 60.0,
        -(71.0 + 8.0 / 60.0),
    ))
    .await
}

pub async fn chart_6_p_v_narasimha_rao_1921() -> Sky {
    // Printed: 1921-06-28 12:49 at UTC+5:17. 79°E09′, 18°N26′.
    sky(book_request(
        (1921, 6, 28),
        (12, 49, 0),
        UtcOffset::Stated(5 * 3600 + 17 * 60),
        Calendar::Gregorian,
        18.0 + 26.0 / 60.0,
        79.0 + 9.0 / 60.0,
    ))
    .await
}

pub async fn chart_7_ronald_reagan_1911() -> Sky {
    // Printed: 1911-02-06 02:04 at UTC−6:00. 89°W47′, 41°N38′.
    sky(book_request(
        (1911, 2, 6),
        (2, 4, 0),
        UtcOffset::Stated(-6 * 3600),
        Calendar::Gregorian,
        41.0 + 38.0 / 60.0,
        -(89.0 + 47.0 / 60.0),
    ))
    .await
}

pub async fn chart_8_guess_his_livelihood_1946() -> Sky {
    // Printed: 1946-12-02 06:45 at UTC+1:00. 15°E39′, 38°N06′.
    sky(book_request(
        (1946, 12, 2),
        (6, 45, 0),
        UtcOffset::Stated(3600),
        Calendar::Gregorian,
        38.0 + 6.0 / 60.0,
        15.0 + 39.0 / 60.0,
    ))
    .await
}

pub async fn chart_9_chatrapati_shivaji_1630() -> Sky {
    // Printed: "Phalguna Bahula Tritiya 1630 AD, 12:05 hrs after sunrise". 73°E53′, 18°N32′.
    // Worked out: that tithi falls on 19 Feb 1630 Julian (as Gregorian, the Sun is 10° short of the book's);
    // the book's upper-limb sunrise is 06:19:48 LMT there, plus 12:05.
    sky(book_request(
        (1630, 2, 19),
        (18, 24, 48),
        UtcOffset::LocalMean,
        Calendar::Julian,
        18.0 + 32.0 / 60.0,
        73.0 + 53.0 / 60.0,
    ))
    .await
}

pub async fn chart_10_akbar_1542() -> Sky {
    // Printed: 4 Dec 1542 (Gregorian) 03:39 LMT, "November 24, 1542 based on the old calendar". 69°E47′, 25°N19′.
    // Entered as the Julian date, since hora rejects Gregorian dates before 1582-10-15.
    sky(book_request(
        (1542, 11, 24),
        (3, 39, 0),
        UtcOffset::LocalMean,
        Calendar::Julian,
        25.0 + 19.0 / 60.0,
        69.0 + 47.0 / 60.0,
    ))
    .await
}

pub async fn chart_11_p_v_narasimha_rao_1921() -> Sky {
    // Printed: 1921-06-28 13:08 IST (UTC+5:30). 79°E09′, 18°N26′.
    sky(book_request(
        (1921, 6, 28),
        (13, 8, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        18.0 + 26.0 / 60.0,
        79.0 + 9.0 / 60.0,
    ))
    .await
}

pub async fn chart_12_sav_exercise_1958() -> Sky {
    // Printed: 1958-08-16 07:05 at UTC−4:00. 83°W53′, 43°N36′.
    sky(book_request(
        (1958, 8, 16),
        (7, 5, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        43.0 + 36.0 / 60.0,
        -(83.0 + 53.0 / 60.0),
    ))
    .await
}

pub async fn chart_13_an_example_1894() -> Sky {
    // Printed: 1894-05-20 13:22 IST (UTC+5:30). 79°E32′, 11°N57′.
    sky(book_request(
        (1894, 5, 20),
        (13, 22, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        11.0 + 57.0 / 60.0,
        79.0 + 32.0 / 60.0,
    ))
    .await
}

pub async fn chart_14_rajiv_gandhi_1944() -> Sky {
    // Printed: 1944-08-20 07:11 IST (UTC+5:30), 07:11:40 in footnote 37, p. 136. 72°E49′, 18°N58′.
    sky(book_request(
        (1944, 8, 20),
        (7, 11, 40),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        18.0 + 58.0 / 60.0,
        72.0 + 49.0 / 60.0,
    ))
    .await
}

pub async fn chart_15_satyam_gaur_1970() -> Sky {
    // Printed: 1970-11-04 16:06 IST (UTC+5:30). 76°E53′, 30°N44′.
    sky(book_request(
        (1970, 11, 4),
        (16, 6, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        30.0 + 44.0 / 60.0,
        76.0 + 53.0 / 60.0,
    ))
    .await
}

pub async fn chart_15_shivam_gaur_1970() -> Sky {
    // Printed: 1970-11-04 16:08 IST (UTC+5:30). 76°E53′, 30°N44′.
    sky(book_request(
        (1970, 11, 4),
        (16, 8, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        30.0 + 44.0 / 60.0,
        76.0 + 53.0 / 60.0,
    ))
    .await
}

pub async fn chart_17_pandit_sanjay_rath_1963() -> Sky {
    // Printed: 1963-08-07 21:14 IST (UTC+5:30). 83°E58′, 21°N27′.
    sky(book_request(
        (1963, 8, 7),
        (21, 14, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        21.0 + 27.0 / 60.0,
        83.0 + 58.0 / 60.0,
    ))
    .await
}

pub async fn chart_18_a_lady_1972() -> Sky {
    // Printed: 1972-06-01 04:16 IST (UTC+5:30). 81°E12′, 16°N15′.
    sky(book_request(
        (1972, 6, 1),
        (4, 16, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_19_sri_navin_patnaik_1946() -> Sky {
    // Printed: 1946-10-16 00:58 IST (UTC+5:30). 85°E50′, 20°N30′.
    sky(book_request(
        (1946, 10, 16),
        (0, 58, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        20.0 + 30.0 / 60.0,
        85.0 + 50.0 / 60.0,
    ))
    .await
}

pub async fn chart_20_an_engineer_in_usa_1954() -> Sky {
    // Printed: 1954-11-12 07:52 IST (UTC+5:30). 78°E50′, 12°N30′.
    sky(book_request(
        (1954, 11, 12),
        (7, 52, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        12.0 + 30.0 / 60.0,
        78.0 + 50.0 / 60.0,
    ))
    .await
}

pub async fn chart_21_john_f_kennedy_jr_1960() -> Sky {
    // Printed: 1960-11-25 00:22 at UTC−5:00. 77°W02′, 38°N54′.
    sky(book_request(
        (1960, 11, 25),
        (0, 22, 0),
        UtcOffset::Stated(-5 * 3600),
        Calendar::Gregorian,
        38.0 + 54.0 / 60.0,
        -(77.0 + 2.0 / 60.0),
    ))
    .await
}

pub async fn chart_22_a_lady_1954() -> Sky {
    // Printed: 1954-12-07 02:13 at UTC−6:00. 93°W15′, 44°N58′.
    sky(book_request(
        (1954, 12, 7),
        (2, 13, 0),
        UtcOffset::Stated(-6 * 3600),
        Calendar::Gregorian,
        44.0 + 58.0 / 60.0,
        -(93.0 + 15.0 / 60.0),
    ))
    .await
}

pub async fn chart_23_dr_b_v_raman_1912() -> Sky {
    // Printed: 1912-08-08 19:38 IST (UTC+5:30). 77°E35′, 13°N00′.
    sky(book_request(
        (1912, 8, 8),
        (19, 38, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        13.0,
        77.0 + 35.0 / 60.0,
    ))
    .await
}

pub async fn chart_24_bill_gates_1955() -> Sky {
    // Printed: 1955-10-28 21:18 at UTC−8:00. 122°W20′, 47°N36′.
    sky(book_request(
        (1955, 10, 28),
        (21, 18, 0),
        UtcOffset::Stated(-8 * 3600),
        Calendar::Gregorian,
        47.0 + 36.0 / 60.0,
        -(122.0 + 20.0 / 60.0),
    ))
    .await
}

pub async fn chart_25_india_1947() -> Sky {
    // Printed: 1947-08-15 00:00 IST (UTC+5:30). 78°E30′, 27°N00′.
    sky(book_request(
        (1947, 8, 15),
        (0, 0, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        27.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_26_narayana_dasa_exercise_1971() -> Sky {
    // Printed: 1971-05-09 09:20 IST (UTC+5:30). 81°E12′, 16°N15′.
    sky(book_request(
        (1971, 5, 9),
        (9, 20, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_27_foreign_stay_example_1970() -> Sky {
    // Printed: 1970-04-04 17:50 IST (UTC+5:30). 81°E12′, 16°N15′.
    sky(book_request(
        (1970, 4, 4),
        (17, 50, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_28_a_lady_1971() -> Sky {
    // Printed: 1971-09-12 08:25 IST (UTC+5:30). 80°E23′, 16°N13′.
    sky(book_request(
        (1971, 9, 12),
        (8, 25, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 13.0 / 60.0,
        80.0 + 23.0 / 60.0,
    ))
    .await
}

pub async fn chart_29_a_lady_1969() -> Sky {
    // Printed: 1969-07-08 10:47 IST (UTC+5:30). 82°E15′, 16°N57′.
    sky(book_request(
        (1969, 7, 8),
        (10, 47, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 57.0 / 60.0,
        82.0 + 15.0 / 60.0,
    ))
    .await
}

pub async fn chart_31_an_electrical_engineer_1947() -> Sky {
    // Printed: 1947-09-07 18:00 IST (UTC+5:30). 80°E30′, 15°N54′.
    sky(book_request(
        (1947, 9, 7),
        (18, 0, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        15.0 + 54.0 / 60.0,
        80.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_32_a_gentleman_1961() -> Sky {
    // Printed: 1961-07-25 17:10 IST (UTC+5:30). 75°E50′, 22°N44′.
    sky(book_request(
        (1961, 7, 25),
        (17, 10, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        22.0 + 44.0 / 60.0,
        75.0 + 50.0 / 60.0,
    ))
    .await
}

pub async fn chart_35_jayalalita_1948() -> Sky {
    // Printed: 1948-02-24 14:36 IST (UTC+5:30). 80°E18′, 13°N05′.
    sky(book_request(
        (1948, 2, 24),
        (14, 36, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        13.0 + 5.0 / 60.0,
        80.0 + 18.0 / 60.0,
    ))
    .await
}

pub async fn chart_37_iskcon_devotee_1971() -> Sky {
    // Printed: 1971-01-26 10:44 at UTC+3:00. 24°E01′, 49°N49′.
    sky(book_request(
        (1971, 1, 26),
        (10, 44, 0),
        UtcOffset::Stated(3 * 3600),
        Calendar::Gregorian,
        49.0 + 49.0 / 60.0,
        24.0 + 1.0 / 60.0,
    ))
    .await
}

pub async fn chart_38_sri_aurobindo_ghose_1872() -> Sky {
    // Printed: 1872-08-15 05:17 at UTC+5:53. 88°E20′, 22°N30′.
    sky(book_request(
        (1872, 8, 15),
        (5, 17, 0),
        UtcOffset::Stated(5 * 3600 + 53 * 60),
        Calendar::Gregorian,
        22.0 + 30.0 / 60.0,
        88.0 + 20.0 / 60.0,
    ))
    .await
}

pub async fn chart_40_a_male_1927() -> Sky {
    // Printed: 1927-01-20 12:30 IST (UTC+5:30). 81°E12′, 16°N15′.
    sky(book_request(
        (1927, 1, 20),
        (12, 30, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_41_a_gentleman_1902() -> Sky {
    // Printed: 1902-03-14 11:48 IST (UTC+5:30). 81°E12′, 16°N15′.
    sky(book_request(
        (1902, 3, 14),
        (11, 48, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_42_niryaana_shoola_exercise_1889() -> Sky {
    // Printed: 1889-04-20 18:30 at UTC+1:00. 13°E02′, 48°N15′.
    sky(book_request(
        (1889, 4, 20),
        (18, 30, 0),
        UtcOffset::Stated(3600),
        Calendar::Gregorian,
        48.0 + 15.0 / 60.0,
        13.0 + 2.0 / 60.0,
    ))
    .await
}

pub async fn chart_43_shoola_dasa_exercise_1915() -> Sky {
    // Printed: 1915-12-12 03:19 at UTC−5:00. 74°W02′, 40°N45′.
    sky(book_request(
        (1915, 12, 12),
        (3, 19, 0),
        UtcOffset::Stated(-5 * 3600),
        Calendar::Gregorian,
        40.0 + 45.0 / 60.0,
        -(74.0 + 2.0 / 60.0),
    ))
    .await
}

pub async fn chart_44_pitri_shoola_example_1971() -> Sky {
    // Printed: 1971-05-09 09:22 IST (UTC+5:30). 81°E12′, 16°N15′.
    sky(book_request(
        (1971, 5, 9),
        (9, 22, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_48_a_gentleman_1933() -> Sky {
    // Printed: 1933-07-15 04:15 IST (UTC+5:30). 80°E55′, 16°N05′.
    sky(book_request(
        (1933, 7, 15),
        (4, 15, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 5.0 / 60.0,
        80.0 + 55.0 / 60.0,
    ))
    .await
}

pub async fn chart_49_an_iskcon_devotee_1971() -> Sky {
    // Printed: 1971-01-26 10:43 at UTC+3:00. 24°E01′, 49°N49′.
    // True Chitra, the nearest ayanamsa to the book's: under Lahiri every value is −52…−107″ off, as in each of charts 49–51 (ch. 24).
    // Truncated, the tables fit an ayanamsa 107–110″ below Lahiri, matched by no Swiss Ephemeris ayanamsa (True Chitra is 55″ below; #71).
    sky(Request {
        ayanamsa: Ayanamsa::TrueChitra,
        ..book_request(
            (1971, 1, 26),
            (10, 43, 0),
            UtcOffset::Stated(3 * 3600),
            Calendar::Gregorian,
            49.0 + 49.0 / 60.0,
            24.0 + 1.0 / 60.0,
        )
    })
    .await
}

pub async fn chart_50_divorced_lady_1968() -> Sky {
    // Printed: 1968-05-21 23:05 IST (UTC+5:30). 78°E10′, 18°N40′.
    // True Chitra, the nearest ayanamsa to the book's: under Lahiri every value is −50…−105″ off, as in each of charts 49–51 (ch. 24).
    // Truncated, the tables fit an ayanamsa 107–110″ below Lahiri, matched by no Swiss Ephemeris ayanamsa (True Chitra is 55″ below; #71).
    sky(Request {
        ayanamsa: Ayanamsa::TrueChitra,
        ..book_request(
            (1968, 5, 21),
            (23, 5, 0),
            UtcOffset::Stated(5 * 3600 + 30 * 60),
            Calendar::Gregorian,
            18.0 + 40.0 / 60.0,
            78.0 + 10.0 / 60.0,
        )
    })
    .await
}

pub async fn chart_51_an_actor_1937() -> Sky {
    // Printed: 1937-07-12 00:30 at UTC−5:00. 75°W10′, 39°N57′.
    // True Chitra, the nearest ayanamsa to the book's: under Lahiri every value is −58…−106″ off, as in each of charts 49–51 (ch. 24).
    // Truncated, the tables fit an ayanamsa 107–110″ below Lahiri, matched by no Swiss Ephemeris ayanamsa (True Chitra is 55″ below; #71).
    sky(Request {
        ayanamsa: Ayanamsa::TrueChitra,
        ..book_request(
            (1937, 7, 12),
            (0, 30, 0),
            UtcOffset::Stated(-5 * 3600),
            Calendar::Gregorian,
            39.0 + 57.0 / 60.0,
            -(75.0 + 10.0 / 60.0),
        )
    })
    .await
}

pub async fn chart_53_a_lady_1973() -> Sky {
    // Printed: 1973-07-26 21:41 IST (UTC+5:30). 80°E28′, 16°N13′.
    sky(book_request(
        (1973, 7, 26),
        (21, 41, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 13.0 / 60.0,
        80.0 + 28.0 / 60.0,
    ))
    .await
}

pub async fn chart_54_an_engineer_1970() -> Sky {
    // Printed: 1970-10-05 12:34 IST (UTC+5:30). 80°E21′, 15°N49′.
    sky(book_request(
        (1970, 10, 5),
        (12, 34, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        15.0 + 49.0 / 60.0,
        80.0 + 21.0 / 60.0,
    ))
    .await
}

pub async fn chart_54_wedding_transit_1999() -> Sky {
    // Printed: 1999-01-24 09:30 IST (UTC+5:30). 78°E30′, 17°N20′.
    sky(book_request(
        (1999, 1, 24),
        (9, 30, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 20.0 / 60.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_55_a_gentleman_1961() -> Sky {
    // Printed: 1961-06-23 22:46 IST (UTC+5:30). 81°E48′, 17°N00′.
    sky(book_request(
        (1961, 6, 23),
        (22, 46, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0,
        81.0 + 48.0 / 60.0,
    ))
    .await
}

pub async fn chart_55_childbirth_transit_1992() -> Sky {
    // Printed: 1992-10-03 11:58 IST (UTC+5:30). 78°E30′, 17°N29′.
    sky(book_request(
        (1992, 10, 3),
        (11, 58, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 29.0 / 60.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_56_john_f_kennedy_jr_1960() -> Sky {
    // Printed: 1960-11-25 00:22 at UTC−5:00. 77°W02′, 38°N53′.
    sky(book_request(
        (1960, 11, 25),
        (0, 22, 0),
        UtcOffset::Stated(-5 * 3600),
        Calendar::Gregorian,
        38.0 + 53.0 / 60.0,
        -(77.0 + 2.0 / 60.0),
    ))
    .await
}

pub async fn chart_56_death_transit_1999() -> Sky {
    // Printed: 1999-07-16 21:45 at UTC−4:00. 71°W12′, 42°N30′.
    sky(book_request(
        (1999, 7, 16),
        (21, 45, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        42.0 + 30.0 / 60.0,
        -(71.0 + 12.0 / 60.0),
    ))
    .await
}

pub async fn chart_58_vajpayee_1926() -> Sky {
    // Printed: 1926-12-25 05:12 IST (UTC+5:30). 78°E10′, 26°N13′.
    sky(book_request(
        (1926, 12, 25),
        (5, 12, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        26.0 + 13.0 / 60.0,
        78.0 + 10.0 / 60.0,
    ))
    .await
}

pub async fn chart_59_vajpayee_swearing_in_1998() -> Sky {
    // Printed: 1998-03-19 09:32 IST (UTC+5:30). 77°E12′, 28°N36′.
    sky(book_request(
        (1998, 3, 19),
        (9, 32, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        28.0 + 36.0 / 60.0,
        77.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_61_indira_gandhi_1917() -> Sky {
    // Printed: 1917-11-19 23:03 IST (UTC+5:30). 81°E52′, 25°N28′.
    sky(book_request(
        (1917, 11, 19),
        (23, 3, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        25.0 + 28.0 / 60.0,
        81.0 + 52.0 / 60.0,
    ))
    .await
}

pub async fn chart_61_indira_gandhi_assassination_1984() -> Sky {
    // Printed: 1984-10-31 12:30 IST (UTC+5:30). 77°E12′, 28°N36′.
    sky(book_request(
        (1984, 10, 31),
        (12, 30, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        28.0 + 36.0 / 60.0,
        77.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_62_princess_diana_1961() -> Sky {
    // Printed: 1961-07-01 14:25 at UTC+1:00. 0°E30′, 52°N50′.
    sky(book_request(
        (1961, 7, 1),
        (14, 25, 0),
        UtcOffset::Stated(3600),
        Calendar::Gregorian,
        52.0 + 50.0 / 60.0,
        0.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_66_annual_2000() -> Sky {
    // Printed: 2000-03-08 04:41 IST (UTC+5:30), 04:41:21 in Example 118, p. 367. 73°E04′, 26°N18′.
    sky(book_request(
        (2000, 3, 8),
        (4, 41, 21),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        26.0 + 18.0 / 60.0,
        73.0 + 4.0 / 60.0,
    ))
    .await
}

pub async fn chart_67_annual_1993() -> Sky {
    // Printed: 1993-06-01 13:30 IST (UTC+5:30), 13:30:04 in Example 122, p. 396. 81°E12′, 16°N15′.
    sky(book_request(
        (1993, 6, 1),
        (13, 30, 4),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_68_annual_1991() -> Sky {
    // Printed: 1991-04-05 03:05 IST (UTC+5:30), 03:05:33 in Example 123, p. 402. 81°E12′, 16°N15′.
    sky(book_request(
        (1991, 4, 5),
        (3, 5, 33),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_69_annual_1987() -> Sky {
    // Printed: 1987-04-05 02:15 IST (UTC+5:30), 02:15:41 in Example 124, p. 403. 81°E12′, 16°N15′.
    sky(book_request(
        (1987, 4, 5),
        (2, 15, 41),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_70_annual_1998() -> Sky {
    // Printed: 1998-04-04 21:59 IST (UTC+5:30), 21:59:49 in Example 125, p. 405. 81°E12′, 16°N15′.
    sky(book_request(
        (1998, 4, 4),
        (21, 59, 49),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_71_annual_1995() -> Sky {
    // Printed: 1995-04-05 03:28 IST (UTC+5:30), 03:28:36 in the answer to Exercise 49, p. 406. 81°E12′, 16°N15′.
    sky(book_request(
        (1995, 4, 5),
        (3, 28, 36),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 15.0 / 60.0,
        81.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_73_indira_gandhi_annual_1976() -> Sky {
    // Printed: 1976-11-20 02:10 IST (UTC+5:30). 81°E52′, 25°N28′.
    sky(book_request(
        (1976, 11, 20),
        (2, 10, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        25.0 + 28.0 / 60.0,
        81.0 + 52.0 / 60.0,
    ))
    .await
}

pub async fn chart_74_a_lady_annual_1998() -> Sky {
    // Printed: 1998-07-27 07:15 IST (UTC+5:30). 80°E28′, 16°N13′.
    sky(book_request(
        (1998, 7, 27),
        (7, 15, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        16.0 + 13.0 / 60.0,
        80.0 + 28.0 / 60.0,
    ))
    .await
}

pub async fn chart_75_rectification_example_1966() -> Sky {
    // Printed: 1966-11-30 10:36 IST (UTC+5:30). 77°E47′, 11°N19′.
    sky(book_request(
        (1966, 11, 30),
        (10, 36, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        11.0 + 19.0 / 60.0,
        77.0 + 47.0 / 60.0,
    ))
    .await
}

pub async fn chart_77_a_gentleman_1939() -> Sky {
    // Printed: 1939-09-12 19:30 IST (UTC+5:30). 76°E36′, 30°N29′.
    sky(book_request(
        (1939, 9, 12),
        (19, 30, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        30.0 + 29.0 / 60.0,
        76.0 + 36.0 / 60.0,
    ))
    .await
}

pub async fn chart_78_bill_gates_wrong_time_1955() -> Sky {
    // Printed: 1955-10-28 21:15 at UTC−8:00. 122°W20′, 47°N36′.
    sky(book_request(
        (1955, 10, 28),
        (21, 15, 0),
        UtcOffset::Stated(-8 * 3600),
        Calendar::Gregorian,
        47.0 + 36.0 / 60.0,
        -(122.0 + 20.0 / 60.0),
    ))
    .await
}

pub async fn chart_79_a_music_teacher_1973() -> Sky {
    // Printed: 1973-09-06 01:32 IST (UTC+5:30). 78°E30′, 17°N29′.
    sky(book_request(
        (1973, 9, 6),
        (1, 32, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 29.0 / 60.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_80_india_annual_1997() -> Sky {
    // Printed: 1997-08-14 19:30 IST (UTC+5:30). 78°E30′, 27°N00′.
    sky(book_request(
        (1997, 8, 14),
        (19, 30, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        27.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_81_india_lunar_year_1992() -> Sky {
    // Printed: 1992-04-03 10:32 IST (UTC+5:30). 77°E12′, 28°N36′.
    sky(book_request(
        (1992, 4, 3),
        (10, 32, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        28.0 + 36.0 / 60.0,
        77.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_82_india_solar_year_1998() -> Sky {
    // Printed: 1998-01-14 10:38 IST (UTC+5:30). 77°E12′, 28°N36′.
    sky(book_request(
        (1998, 1, 14),
        (10, 38, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        28.0 + 36.0 / 60.0,
        77.0 + 12.0 / 60.0,
    ))
    .await
}

pub async fn chart_84_vajpayee_swearing_in_1996() -> Sky {
    // Printed: 1996-05-16 12:00 IST (UTC+5:30). 78°E30′, 27°N00′.
    sky(book_request(
        (1996, 5, 16),
        (12, 0, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        27.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_86_writing_this_book_1997() -> Sky {
    // Printed: 1997-05-13 12:01 at UTC−4:00. 71°W08′, 42°N40′.
    sky(book_request(
        (1997, 5, 13),
        (12, 1, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        42.0 + 40.0 / 60.0,
        -(71.0 + 8.0 / 60.0),
    ))
    .await
}

pub async fn chart_88_an_astrologer_1949() -> Sky {
    // Printed: 1949-03-13 00:32 at UTC+0:00. 8°W13′, 53°N20′.
    sky(book_request(
        (1949, 3, 13),
        (0, 32, 0),
        UtcOffset::Stated(0),
        Calendar::Gregorian,
        53.0 + 20.0 / 60.0,
        -(8.0 + 13.0 / 60.0),
    ))
    .await
}

pub async fn chart_89_annual_1999() -> Sky {
    // Printed: 1999-03-13 20:30 at UTC+0:00. 8°W13′, 53°N20′.
    sky(book_request(
        (1999, 3, 13),
        (20, 30, 0),
        UtcOffset::Stated(0),
        Calendar::Gregorian,
        53.0 + 20.0 / 60.0,
        -(8.0 + 13.0 / 60.0),
    ))
    .await
}

pub async fn chart_90_an_indian_in_usa_1968() -> Sky {
    // Printed: 1968-01-25 11:14 IST (UTC+5:30). 78°E30′, 17°N29′.
    sky(book_request(
        (1968, 1, 25),
        (11, 14, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 29.0 / 60.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_91_annual_1998() -> Sky {
    // Printed: 1998-01-25 03:59 IST (UTC+5:30). 78°E30′, 17°N29′.
    sky(book_request(
        (1998, 1, 25),
        (3, 59, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 29.0 / 60.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_92_monthly_may_1998() -> Sky {
    // Printed: 1998-05-26 09:55 IST (UTC+5:30). 78°E30′, 17°N29′.
    sky(book_request(
        (1998, 5, 26),
        (9, 55, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 29.0 / 60.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_92_monthly_june_1998() -> Sky {
    // Printed: 1998-06-26 18:47 IST (UTC+5:30). 78°E30′, 17°N29′.
    sky(book_request(
        (1998, 6, 26),
        (18, 47, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        17.0 + 29.0 / 60.0,
        78.0 + 30.0 / 60.0,
    ))
    .await
}

pub async fn chart_93_monthly_march_2000() -> Sky {
    // Printed: 2000-03-14 00:21 at UTC−4:00. 74°W00′, 40°N43′.
    sky(book_request(
        (2000, 3, 14),
        (0, 21, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        40.0 + 43.0 / 60.0,
        -74.0,
    ))
    .await
}

pub async fn chart_94_sixty_hours_2000() -> Sky {
    // Printed: 2000-03-26 14:27 at UTC−4:00. 74°W00′, 40°N43′.
    sky(book_request(
        (2000, 3, 26),
        (14, 27, 0),
        UtcOffset::Stated(-4 * 3600),
        Calendar::Gregorian,
        40.0 + 43.0 / 60.0,
        -74.0,
    ))
    .await
}

pub async fn chart_95_a_lady_1968() -> Sky {
    // Printed: 1968-05-21 23:05 IST (UTC+5:30). 78°E10′, 18°N40′.
    sky(book_request(
        (1968, 5, 21),
        (23, 5, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        18.0 + 40.0 / 60.0,
        78.0 + 10.0 / 60.0,
    ))
    .await
}

pub async fn chart_96_tajaka_1991() -> Sky {
    // Printed: 1991-05-22 20:29 IST (UTC+5:30). 78°E10′, 18°N40′.
    sky(book_request(
        (1991, 5, 22),
        (20, 29, 0),
        UtcOffset::Stated(5 * 3600 + 30 * 60),
        Calendar::Gregorian,
        18.0 + 40.0 / 60.0,
        78.0 + 10.0 / 60.0,
    ))
    .await
}

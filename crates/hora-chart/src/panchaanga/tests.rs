use super::*;
use crate::tests::*;

#[test]
fn each_limb_is_named_alike_across_the_part_hora_swe_searches() {
    // The limb named at `degrees` of the measure, with the Sun at 0°.
    let names: [fn(f64) -> u32; 4] = [
        |d| tithi(0.0, d).0.into(),
        |d| nakshatra(d).0 as u32,
        |d| sun_moon_yoga(0.0, d) as u32,
        |d| karana(0.0, d) as u32,
    ];
    for (size, name) in [TITHI, NAKSHATRA, NAKSHATRA, KARANA].into_iter().zip(names) {
        for boundary in (0..=360 * 3600 / size).map(|i| f64::from(i * size) / 3600.0) {
            // The +1µ″ in `arcseconds` makes 1e-12° below a boundary the next part; rem_euclid can round to 360°.
            for d in [-1e-5 / 3600.0, -1e-12, 0.0, 1e-12].map(|e| (boundary + e).clamp(0.0, 360.0))
            {
                let part = arcseconds_from_aries(d) / size;
                let middle = (f64::from(part) + 0.5) * f64::from(size) / 3600.0;
                assert_eq!(name(d), name(middle), "{d}° in parts of {size}″");
            }
        }
    }
}

#[test]
fn every_nakshatra_starts_where_table_2_says() {
    // Table 2's "Starts at" (ch. 1 §1.3.6), as a rasi and degrees, minutes.
    let starts = [
        (0, 0, 0),
        (0, 13, 20),
        (0, 26, 40),
        (1, 10, 0),
        (1, 23, 20),
        (2, 6, 40),
        (2, 20, 0),
        (3, 3, 20),
        (3, 16, 40),
        (4, 0, 0),
        (4, 13, 20),
        (4, 26, 40),
        (5, 10, 0),
        (5, 23, 20),
        (6, 6, 40),
        (6, 20, 0),
        (7, 3, 20),
        (7, 16, 40),
        (8, 0, 0),
        (8, 13, 20),
        (8, 26, 40),
        (9, 10, 0),
        (9, 23, 20),
        (10, 6, 40),
        (10, 20, 0),
        (11, 3, 20),
        (11, 16, 40),
    ];
    for (i, (r, d, m)) in starts.into_iter().enumerate() {
        let start = dms(r, d, m, 0.0);
        assert_eq!(nakshatra(start), (NAKSHATRAS[i], 1), "{start}");
        let before = (start - 1.0 / 3600.0).rem_euclid(360.0);
        assert_eq!(nakshatra(before), (NAKSHATRAS[(i + 26) % 27], 4), "{start}");
    }
    assert_eq!(nakshatra(dms(11, 29, 59, 59.0)), (Nakshatra::Revati, 4));
    assert_eq!(nakshatra(360.0 - 1e-12), (Nakshatra::Revati, 4));
    assert_eq!(nakshatra(-1e-20), (Nakshatra::Aswini, 1));
}

#[test]
fn every_pada_is_3_20() {
    for (pada, d, m) in [(1, 0, 0), (2, 3, 20), (3, 6, 40), (4, 10, 0)] {
        assert_eq!(nakshatra(dms(0, d, m, 0.0)), (Nakshatra::Aswini, pada));
    }
    assert_eq!(nakshatra(dms(0, 3, 19, 59.0)), (Nakshatra::Aswini, 1));
    assert_eq!(nakshatra(dms(0, 3, 20, 0.0)), (Nakshatra::Aswini, 2));
}

#[test]
fn the_books_kalachakra_examples_name_the_moons_pada() {
    // Ch. 24 *Kalachakra Dasa*, examples 95 and 96 and exercise 34's answer.
    assert_eq!(nakshatra(dms(1, 15, 50, 0.0)), (Nakshatra::Rohini, 2));
    assert_eq!(nakshatra(dms(3, 3, 0, 0.0)), (Nakshatra::Punarvasu, 4));
    assert_eq!(nakshatra(dms(10, 5, 50, 0.0)), (Nakshatra::Dhanishtha, 4));
}

#[test]
fn vimsottari_lords_are_table_2s() {
    use Body::*;
    // Table 2's "Vimsottari Lord" column, row by row (ch. 1 §1.3.6).
    let table_2 = [
        Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury, //
        Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury, //
        Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury,
    ];
    let lords: Vec<_> = NAKSHATRAS.into_iter().map(vimsottari_lord).collect();
    assert_eq!(lords, table_2);
}

#[test]
fn tithis_15_and_30_are_table_3s_two_last_rows() {
    let at = |moon_ahead: f64| tithi(100.0, 100.0 + moon_ahead);
    assert_eq!(at(0.0), (1, Paksha::Sukla, Tithi::Pratipat));
    assert_eq!(at(179.0), (15, Paksha::Sukla, Tithi::Paurnami));
    assert_eq!(at(180.0), (16, Paksha::Krishna, Tithi::Pratipat));
    assert_eq!(at(359.0), (30, Paksha::Krishna, Tithi::Amavasya));
    assert_eq!(tithi(0.0, -1e-20), (1, Paksha::Sukla, Tithi::Pratipat));
}

#[test]
fn tithi_lords_are_table_3s() {
    use Body::*;
    let table_3 = [
        Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, //
        Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu,
    ];
    let lords: Vec<_> = TITHIS.into_iter().map(tithi_lord).collect();
    assert_eq!(lords, table_3);
    assert_eq!(TITHIS[14], Tithi::Paurnami);
    assert_eq!(TITHIS[15], Tithi::Amavasya);
}

#[test]
fn lunar_months_are_table_4s_and_the_first_of_two_alike_is_adhika() {
    let at = |longitude: f64| NewMoon { jd: 0.0, longitude };
    let month = |[a, b, c]: [f64; 3]| lunar_month(&[at(a), at(b), at(c)]);
    assert_eq!(month([320.0, 349.0, 18.0]), (LunarMonth::Chaitra, None));
    assert_eq!(month([290.0, 319.9, 348.0]), (LunarMonth::Phaalguna, None));
    assert_eq!(
        month([2.0, 30.4, 58.5]),
        (LunarMonth::Jyeshtha, Some(TwinMonth::Adhika))
    );
    assert_eq!(
        month([30.4, 58.5, 87.0]),
        (LunarMonth::Jyeshtha, Some(TwinMonth::Nija))
    );
    // A month in which the Sun enters two rasis skips a name (kshaya), and is neither.
    assert_eq!(month([28.0, 59.9, 90.1]), (LunarMonth::Jyeshtha, None));
    let named: Vec<_> = (0..12)
        .map(|r| month([0.0, f64::from(r) * 30.0 + 15.0, 180.0]).0)
        .collect();
    let mut table_4 = LUNAR_MONTHS.to_vec();
    table_4.rotate_left(1);
    assert_eq!(named, table_4);
    // Table 4's full-Moon nakshatras, by the month's place: Chitra is the 14th nakshatra.
    let full_moons: Vec<Vec<u8>> = LUNAR_MONTHS
        .map(|month| {
            full_moon_nakshatras(month)
                .iter()
                .map(|&n| n as u8 + 1)
                .collect()
        })
        .to_vec();
    let table_4 = [
        vec![14],
        vec![16],
        vec![18],
        vec![20, 21],
        vec![22],
        vec![25, 26],
        vec![1],
        vec![3],
        vec![5],
        vec![8],
        vec![10],
        vec![11, 12],
    ];
    assert_eq!(full_moons, table_4);
}

#[test]
fn sun_moon_yogas_wrap_at_360() {
    assert_eq!(sun_moon_yoga(200.0, 160.0), SunMoonYoga::Vishkambha);
    assert_eq!(sun_moon_yoga(200.0, 160.0 - 1e-9), SunMoonYoga::Vaidhriti);
    assert_eq!(SUN_MOON_YOGAS[9], SunMoonYoga::Ganda);
}

#[test]
fn karanas_follow_section_1_3_10() {
    use Karana::*;
    let at = |moon_ahead: f64| karana(100.0, 100.0 + moon_ahead);
    for (moon_ahead, want) in [
        (0.0, Kimstughna),
        (5.99, Kimstughna),
        (6.0, Bava),
        (11.99, Bava),
        (12.0, Balava),
        (42.0, Vishti),
        (48.0, Bava),
        (336.0, Vishti),
        (341.99, Vishti),
        (342.0, Sakuna),
        (348.0, Chatushpada),
        (354.0, Naga),
        (359.99, Naga),
    ] {
        assert_eq!(at(moon_ahead), want, "{moon_ahead}");
    }
    let halves: Vec<_> = (0..60).map(|h| at(f64::from(h) * 6.0 + 1.0)).collect();
    assert_eq!(halves.iter().filter(|&&k| k == Bava).count(), 8);
    assert_eq!(halves.iter().filter(|&&k| k == Vishti).count(), 8);
}

#[test]
fn the_vaara_is_the_weekday_of_the_vedic_days_sunrise_in_its_clock() {
    // 2000-01-01 was a Saturday.
    assert_eq!(vaara(&day(2451545.0, 2451546.0)), Vaara::Saturday);
    assert_eq!(
        vaara(&day(WEDNESDAY + hours(6, 10), WEDNESDAY + 1.25)),
        Vaara::Wednesday
    );
    // Just before midnight in the clock, still Tuesday.
    assert_eq!(
        vaara(&day(WEDNESDAY - 1e-6, WEDNESDAY + 1.0)),
        Vaara::Tuesday
    );
    use Body::*;
    let lords = VAARAS.map(vaara_lord);
    assert_eq!(lords, [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn]);
}

#[test]
fn horas_are_24_equal_parts_of_a_shorter_vedic_day() {
    // Sunrise at 6:00, the next at 5:54: each hora is 59 min 45 s.
    let sunrise = WEDNESDAY + hours(6, 0);
    let day = day(sunrise, WEDNESDAY + 1.0 + hours(5, 54));
    // 17 h 57 min after sunrise is in the 19th hora, not the 18th hour.
    assert_eq!(hora(&day, sunrise + hours(17, 57)), (19, Body::Mars));
    assert_eq!(hora(&day, sunrise + hours(17, 55)), (18, Body::Jupiter));
    assert_eq!(hora(&day, sunrise), (1, Body::Mercury));
    assert_eq!(hora(&day, day.next_sunrise - 1e-9), (24, Body::Saturn));
    assert_eq!(hora(&day, day.next_sunrise), (24, Body::Saturn));
}

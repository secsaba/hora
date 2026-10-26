//! The chart prepared from a Sky, in the book's terms (ch. 1 *Basic Concepts*); no web code.

use hora_swe::{Position, Sky};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Body {
    Lagna,
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rasi {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

/// The lagna or a graha, its sidereal longitude (degrees) and whether it is marked retrograde.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    pub body: Body,
    pub longitude: f64,
    pub retrograde: bool,
}

/// The chart of `sky`: the lagna, then the nine grahas in the book's order.
pub fn chart(sky: &Sky) -> [Entry; 10] {
    let g = &sky.grahas;
    let entry = |body, longitude, retrograde| Entry {
        body,
        longitude,
        retrograde,
    };
    // Only Mars to Saturn are marked retrograde; Rahu and Ketu move backward by nature (ch. 1 *Basic Concepts*).
    let retrograde = |body, p: Position| entry(body, p.longitude, p.speed < 0.0);
    [
        entry(Body::Lagna, sky.lagna, false),
        entry(Body::Sun, g.sun.longitude, false),
        entry(Body::Moon, g.moon.longitude, false),
        retrograde(Body::Mars, g.mars),
        retrograde(Body::Mercury, g.mercury),
        retrograde(Body::Jupiter, g.jupiter),
        retrograde(Body::Venus, g.venus),
        retrograde(Body::Saturn, g.saturn),
        entry(Body::Rahu, g.rahu.longitude, false),
        // Ketu stands exactly opposite Rahu (ch. 1 *Basic Concepts*).
        entry(
            Body::Ketu,
            (g.rahu.longitude + 180.0).rem_euclid(360.0),
            false,
        ),
    ]
}

/// The twelve rasis in zodiac order, from 0° Aries (ch. 1 *Basic Concepts*).
pub const RASIS: [Rasi; 12] = [
    Rasi::Aries,
    Rasi::Taurus,
    Rasi::Gemini,
    Rasi::Cancer,
    Rasi::Leo,
    Rasi::Virgo,
    Rasi::Libra,
    Rasi::Scorpio,
    Rasi::Sagittarius,
    Rasi::Capricorn,
    Rasi::Aquarius,
    Rasi::Pisces,
];

/// Each rasi's span in degrees (ch. 1 *Basic Concepts*).
pub const RASI_SPAN: f64 = 30.0;

/// The rasi `longitude` falls in, and its degrees within that rasi, in [0, 30): each rasi is 30° from 0° Aries (ch. 1 *Basic Concepts*).
pub fn rasi(longitude: f64) -> (Rasi, f64) {
    let longitude = longitude.rem_euclid(360.0);
    // `%` is exact, so the rasi and the degrees agree; rem_euclid can round up to 360.0 itself, which is 0° Aries.
    let degrees = longitude % RASI_SPAN;
    let index = ((longitude - degrees) / RASI_SPAN).round() as usize % 12;
    (RASIS[index], degrees)
}

/// Degrees within a rasi, in [0, 30), as whole arcseconds, truncated: 29°59′59.9″ stays 29°59′59″.
pub fn arcseconds(degrees: f64) -> u32 {
    // +1µ″: a longitude built as 3°20′ can land a hair below it. min: just under 30° can round up to 30 × 3600.
    ((degrees * 3600.0 + 1e-6) as u32).min(30 * 3600 - 1)
}

/// The bhava, 1 to 12, that `rasi` is counted from the rasi holding the reference point; Pisces is followed by Aries (ch. 1 §1.3.3).
pub fn bhava(reference: Rasi, rasi: Rasi) -> u8 {
    (rasi as u8 + 12 - reference as u8) % 12 + 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nakshatra {
    Aswini,
    Bharani,
    Krittika,
    Rohini,
    Mrigasira,
    Aardra,
    Punarvasu,
    Pushyami,
    Aasresha,
    Makha,
    PoorvaPhalguni,
    UttaraPhalguni,
    Hasta,
    Chitra,
    Swaati,
    Visaakha,
    Anooraadha,
    Jyeshtha,
    Moola,
    Poorvaashaadha,
    Uttaraashaadha,
    Sravanam,
    Dhanishtha,
    Satabhishak,
    Poorvaabhaadra,
    Uttaraabhaadra,
    Revati,
}

/// The 27 nakshatras in zodiac order, each 13°20′ from 0° Aries; Abhijit is not one of them (ch. 1 §1.3.6, Table 2).
pub const NAKSHATRAS: [Nakshatra; 27] = [
    Nakshatra::Aswini,
    Nakshatra::Bharani,
    Nakshatra::Krittika,
    Nakshatra::Rohini,
    Nakshatra::Mrigasira,
    Nakshatra::Aardra,
    Nakshatra::Punarvasu,
    Nakshatra::Pushyami,
    Nakshatra::Aasresha,
    Nakshatra::Makha,
    Nakshatra::PoorvaPhalguni,
    Nakshatra::UttaraPhalguni,
    Nakshatra::Hasta,
    Nakshatra::Chitra,
    Nakshatra::Swaati,
    Nakshatra::Visaakha,
    Nakshatra::Anooraadha,
    Nakshatra::Jyeshtha,
    Nakshatra::Moola,
    Nakshatra::Poorvaashaadha,
    Nakshatra::Uttaraashaadha,
    Nakshatra::Sravanam,
    Nakshatra::Dhanishtha,
    Nakshatra::Satabhishak,
    Nakshatra::Poorvaabhaadra,
    Nakshatra::Uttaraabhaadra,
    Nakshatra::Revati,
];

/// The nakshatra `longitude` falls in and its pada, 1 to 4, each pada 3°20′ (ch. 1 §1.3.6, Table 2).
pub fn nakshatra(longitude: f64) -> (Nakshatra, u8) {
    let (rasi, degrees) = rasi(longitude);
    // 3°20′ is no f64 but is 12000″, so whole arcseconds split exactly; a rasi holds 9 padas.
    let pada = rasi as usize * 9 + arcseconds(degrees) as usize / 12000;
    (NAKSHATRAS[pada / 4], (pada % 4) as u8 + 1)
}

/// The graha ruling `nakshatra` in the Vimsottari dasa: nine lords from Ketu, repeated (ch. 1 §1.3.6, Table 2).
pub fn vimsottari_lord(nakshatra: Nakshatra) -> Body {
    use Body::*;
    [Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury][nakshatra as usize % 9]
}

#[cfg(test)]
mod tests {
    use hora_swe::Grahas;

    use super::*;

    /// Sidereal longitude from a rasi (0 = Aries) and degrees, minutes, seconds within it.
    fn dms(rasi: u8, d: u8, m: u8, s: f64) -> f64 {
        f64::from(rasi) * 30.0 + f64::from(d) + f64::from(m) / 60.0 + s / 3600.0
    }

    /// A Sky with the lagna and `[longitude, speed]` for Sun to Rahu; the speeds' signs are all a chart reads.
    fn sky(lagna: f64, g: [[f64; 2]; 8]) -> Sky {
        let p = |[longitude, speed]: [f64; 2]| Position { longitude, speed };
        Sky {
            grahas: Grahas {
                sun: p(g[0]),
                moon: p(g[1]),
                mars: p(g[2]),
                mercury: p(g[3]),
                jupiter: p(g[4]),
                venus: p(g[5]),
                saturn: p(g[6]),
                rahu: p(g[7]),
            },
            lagna,
            ayanamsa: 0.0,
            vedic_day: None,
        }
    }

    /// Chart 1: ch. 9 *Arudha Padas*, p. 87, to the arcminute the book prints. It prints no speeds; the mean node always moves backward.
    fn chart_1() -> Sky {
        sky(
            dms(5, 10, 58, 0.0),
            [
                [dms(11, 26, 29, 0.0), 1.0],
                [dms(2, 4, 45, 0.0), 13.0],
                [dms(0, 19, 9, 0.0), 0.7],
                [dms(11, 1, 36, 0.0), 1.5],
                [dms(0, 17, 21, 0.0), 0.2],
                [dms(11, 10, 1, 0.0), 1.2],
                [dms(0, 22, 41, 0.0), 0.1],
                [dms(3, 5, 55, 0.0), -0.05],
            ],
        )
    }

    #[test]
    fn rasi_splits_the_books_chart_1_values() {
        use Body::*;
        use Rasi::*;
        // No printed value lies within an arcminute of a rasi boundary, so the book's rounding decides nothing here.
        let books = [
            (Lagna, Virgo, (10, 58)),
            (Sun, Pisces, (26, 29)),
            (Moon, Gemini, (4, 45)),
            (Mars, Aries, (19, 9)),
            (Mercury, Pisces, (1, 36)),
            (Jupiter, Aries, (17, 21)),
            (Venus, Pisces, (10, 1)),
            (Saturn, Aries, (22, 41)),
            (Rahu, Cancer, (5, 55)),
            (Ketu, Capricorn, (5, 55)),
        ];
        let chart = chart(&chart_1());
        for (entry, (body, book_rasi, (d, m))) in chart.iter().zip(books) {
            assert_eq!(entry.body, body);
            let (got_rasi, degrees) = rasi(entry.longitude);
            assert_eq!(got_rasi, book_rasi, "{body:?}");
            assert!(
                (degrees - dms(0, d, m, 0.0)).abs() < 1e-9,
                "{body:?}: {degrees}"
            );
            assert!(!entry.retrograde, "{body:?}");
        }
    }

    #[test]
    fn only_mars_to_saturn_can_be_retrograde_and_ketu_is_opposite_rahu() {
        let chart = chart(&sky(-1.0, [[350.0, -1.0]; 8]));
        let marked: Vec<_> = chart
            .iter()
            .filter(|e| e.retrograde)
            .map(|e| e.body)
            .collect();
        use Body::*;
        assert_eq!(marked, [Mars, Mercury, Jupiter, Venus, Saturn]);
        assert_eq!(chart[9].body, Ketu);
        assert_eq!(chart[9].longitude, 170.0);
    }

    #[test]
    fn a_longitude_stays_in_its_rasi_up_to_the_boundary() {
        let (r, degrees) = rasi(dms(1, 29, 59, 59.9));
        assert_eq!(r, Rasi::Taurus);
        assert!((29.99..30.0).contains(&degrees), "{degrees}");
        assert_eq!(rasi(60.0), (Rasi::Gemini, 0.0));
        let (r, degrees) = rasi(360.0 - 1e-12);
        assert_eq!(r, Rasi::Pisces);
        assert!((29.99..30.0).contains(&degrees), "{degrees}");
        // rem_euclid rounds this up to exactly 360.0.
        assert_eq!((-1e-20f64).rem_euclid(360.0), 360.0);
        assert_eq!(rasi(-1e-20), (Rasi::Aries, 0.0));
        assert_eq!(rasi(360.0), (Rasi::Aries, 0.0));
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
    fn bhavas_from_a_horalagna_in_cancer_wrap_past_pisces() {
        use Rasi::*;
        // Ch. 1 §1.3.3's example.
        for (rasi, house) in [
            (Cancer, 1),
            (Leo, 2),
            (Virgo, 3),
            (Pisces, 9),
            (Aries, 10),
            (Taurus, 11),
            (Gemini, 12),
        ] {
            assert_eq!(bhava(Cancer, rasi), house, "{rasi:?}");
        }
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
}

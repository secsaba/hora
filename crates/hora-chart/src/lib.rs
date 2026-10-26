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

/// The rasi `longitude` falls in, and its degrees within that rasi, in [0, 30): each rasi is 30° from 0° Aries (ch. 1 *Basic Concepts*).
pub fn rasi(longitude: f64) -> (Rasi, f64) {
    use Rasi::*;
    const RASIS: [Rasi; 12] = [
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
    ];
    let longitude = longitude.rem_euclid(360.0);
    // `%` is exact, so the rasi and the degrees agree; rem_euclid can round up to 360.0 itself, which is 0° Aries.
    let degrees = longitude % 30.0;
    let index = ((longitude - degrees) / 30.0).round() as usize % 12;
    (RASIS[index], degrees)
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
}

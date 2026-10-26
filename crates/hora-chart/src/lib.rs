//! The chart prepared from a Sky, in the book's terms (ch. 1 *Basic Concepts*); no web code.

use hora_swe::{NewMoon, Position, Sky, VedicDay};

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

/// The rasi holding `body` in `chart`.
pub fn rasi_of(chart: &[Entry; 10], body: Body) -> Rasi {
    // chart() holds every Body.
    let entry = chart.iter().find(|e| e.body == body).unwrap();
    rasi(entry.longitude).0
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

/// Whole arcseconds the Moon is ahead of the Sun, in [0, 360°); truncated like `arcseconds`, so a boundary built from printed values holds.
fn moon_ahead(sun: f64, moon: f64) -> u32 {
    let (rasi, degrees) = rasi(moon - sun);
    rasi as u32 * 30 * 3600 + arcseconds(degrees)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paksha {
    Sukla,
    Krishna,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tithi {
    Pratipat,
    Dwitiya,
    Tritiya,
    Chaturthi,
    Panchami,
    Shashti,
    Saptami,
    Ashtami,
    Navami,
    Dasami,
    Ekadasi,
    Dwadasi,
    Trayodasi,
    Chaturdasi,
    Paurnami,
    Amavasya,
}

/// Table 3's rows: the first fourteen in both pakshas, Paurnami the 15th tithi, Amavasya the 30th (ch. 1 §1.3.8.1).
pub const TITHIS: [Tithi; 16] = [
    Tithi::Pratipat,
    Tithi::Dwitiya,
    Tithi::Tritiya,
    Tithi::Chaturthi,
    Tithi::Panchami,
    Tithi::Shashti,
    Tithi::Saptami,
    Tithi::Ashtami,
    Tithi::Navami,
    Tithi::Dasami,
    Tithi::Ekadasi,
    Tithi::Dwadasi,
    Tithi::Trayodasi,
    Tithi::Chaturdasi,
    Tithi::Paurnami,
    Tithi::Amavasya,
];

/// The tithi running, 1 to 30, its paksha and name: one per 12° the Moon is ahead of the Sun (ch. 1 §1.3.8.1).
pub fn tithi(sun: f64, moon: f64) -> (u8, Paksha, Tithi) {
    let index = (moon_ahead(sun, moon) / (12 * 3600)) as u8 + 1;
    let (paksha, name) = match index {
        1..=15 => (Paksha::Sukla, TITHIS[usize::from(index) - 1]),
        30 => (Paksha::Krishna, Tithi::Amavasya),
        _ => (Paksha::Krishna, TITHIS[usize::from(index) - 16]),
    };
    (index, paksha, name)
}

/// The graha ruling `tithi`: Sun to Rahu, repeated down Table 3 (ch. 1 §1.3.8.1).
pub fn tithi_lord(tithi: Tithi) -> Body {
    use Body::*;
    [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu][tithi as usize % 8]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LunarMonth {
    Chaitra,
    Vaisaakha,
    Jyeshtha,
    Aashaadha,
    Sraavana,
    Bhaadrapada,
    Aaswayuja,
    Kaarteeka,
    Maargasira,
    Pushya,
    Maagha,
    Phaalguna,
}

/// Table 4's lunar months, from the one a new Moon in Pisces starts (ch. 1 §1.3.8.2).
pub const LUNAR_MONTHS: [LunarMonth; 12] = [
    LunarMonth::Chaitra,
    LunarMonth::Vaisaakha,
    LunarMonth::Jyeshtha,
    LunarMonth::Aashaadha,
    LunarMonth::Sraavana,
    LunarMonth::Bhaadrapada,
    LunarMonth::Aaswayuja,
    LunarMonth::Kaarteeka,
    LunarMonth::Maargasira,
    LunarMonth::Pushya,
    LunarMonth::Maagha,
    LunarMonth::Phaalguna,
];

/// Which of two consecutive lunar months started in the same rasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwinMonth {
    Adhika,
    Nija,
}

/// The lunar month `new_moons[1]` starts (ch. 1 §1.3.8.2, Table 4); not from the book: the first of two alike is Adhika.
pub fn lunar_month([previous, start, next]: &[NewMoon; 3]) -> (LunarMonth, Option<TwinMonth>) {
    let rasi_of = |new_moon: &NewMoon| rasi(new_moon.longitude).0;
    let twin = if rasi_of(start) == rasi_of(next) {
        Some(TwinMonth::Adhika)
    } else if rasi_of(previous) == rasi_of(start) {
        Some(TwinMonth::Nija)
    } else {
        None
    };
    // Pisces, the twelfth rasi, starts the first month.
    (LUNAR_MONTHS[(rasi_of(start) as usize + 1) % 12], twin)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SunMoonYoga {
    Vishkambha,
    Preeti,
    Aayushmaan,
    Saubhaagya,
    Sobhana,
    Atiganda,
    Sukarman,
    Dhriti,
    Shoola,
    Ganda,
    Vriddhi,
    Dhruva,
    Vyaaghaata,
    Harshana,
    Vajra,
    Siddhi,
    Vyatipaata,
    Variyan,
    Parigha,
    Shiva,
    Siddha,
    Saadhya,
    Subha,
    Sukla,
    Brahma,
    Indra,
    Vaidhriti,
}

/// Table 5's Sun-Moon yogas in order (ch. 1 §1.3.9).
pub const SUN_MOON_YOGAS: [SunMoonYoga; 27] = [
    SunMoonYoga::Vishkambha,
    SunMoonYoga::Preeti,
    SunMoonYoga::Aayushmaan,
    SunMoonYoga::Saubhaagya,
    SunMoonYoga::Sobhana,
    SunMoonYoga::Atiganda,
    SunMoonYoga::Sukarman,
    SunMoonYoga::Dhriti,
    SunMoonYoga::Shoola,
    SunMoonYoga::Ganda,
    SunMoonYoga::Vriddhi,
    SunMoonYoga::Dhruva,
    SunMoonYoga::Vyaaghaata,
    SunMoonYoga::Harshana,
    SunMoonYoga::Vajra,
    SunMoonYoga::Siddhi,
    SunMoonYoga::Vyatipaata,
    SunMoonYoga::Variyan,
    SunMoonYoga::Parigha,
    SunMoonYoga::Shiva,
    SunMoonYoga::Siddha,
    SunMoonYoga::Saadhya,
    SunMoonYoga::Subha,
    SunMoonYoga::Sukla,
    SunMoonYoga::Brahma,
    SunMoonYoga::Indra,
    SunMoonYoga::Vaidhriti,
];

/// The Sun-Moon yoga running: one per 13°20′ of the sum of their longitudes (ch. 1 §1.3.9).
pub fn sun_moon_yoga(sun: f64, moon: f64) -> SunMoonYoga {
    // The same 13°20′ parts as the nakshatras.
    SUN_MOON_YOGAS[nakshatra(sun + moon).0 as usize]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Karana {
    Bava,
    Balava,
    Kaulava,
    Taitula,
    Garija,
    Vanija,
    Vishti,
    Sakuna,
    Chatushpada,
    Naga,
    Kimstughna,
}

/// The eleven karanas in the book's order: seven repeating, then four once a month (ch. 1 §1.3.10).
pub const KARANAS: [Karana; 11] = [
    Karana::Bava,
    Karana::Balava,
    Karana::Kaulava,
    Karana::Taitula,
    Karana::Garija,
    Karana::Vanija,
    Karana::Vishti,
    Karana::Sakuna,
    Karana::Chatushpada,
    Karana::Naga,
    Karana::Kimstughna,
];

/// The karana running: each tithi's halves are 6° of Moon − Sun; the first half of the month is Kimstughna,
/// the next 56 the first seven karanas eight times, the last three Sakuna, Chatushpada, Naga (ch. 1 §1.3.10).
pub fn karana(sun: f64, moon: f64) -> Karana {
    match (moon_ahead(sun, moon) / (6 * 3600)) as usize {
        0 => Karana::Kimstughna,
        half @ 1..=56 => KARANAS[(half - 1) % 7],
        half => KARANAS[half - 57 + 7],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vaara {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

/// The weekday of the Vedic day's sunrise in the birth time's clock, so a birth before sunrise has the previous one (ch. 1 §1.3.11).
pub fn vaara(day: &VedicDay) -> Vaara {
    use Vaara::*;
    // A Julian day starts at noon, and Julian day 0 was a Monday.
    let days = (day.clock_sunrise + 0.5).floor() as i64;
    [
        Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday,
    ][(days + 1).rem_euclid(7) as usize]
}

/// The graha ruling `vaara`, Sun for Sunday to Saturn for Saturday (ch. 1 §1.3.11).
pub fn vaara_lord(vaara: Vaara) -> Body {
    use Body::*;
    [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn][vaara as usize]
}

/// The hora running at `birth` (UT Julian day), 1 to 24, and its lord: the Vedic day in 24 equal parts, the first ruled by
/// the vaara's graha, then on in the order Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon (ch. 1 §1.3.11).
pub fn hora(day: &VedicDay, birth: f64) -> (u8, Body) {
    use Body::*;
    const ORDER: [Body; 7] = [Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon];
    let part = (birth - day.sunrise) / (day.next_sunrise - day.sunrise) * 24.0;
    // The Vedic day holds the birth, so only rounding can push it out of 0..24.
    let index = (part as u8).min(23);
    // ORDER holds every vaara lord.
    let first = ORDER
        .iter()
        .position(|&g| g == vaara_lord(vaara(day)))
        .unwrap();
    (index + 1, ORDER[(first + usize::from(index)) % 7])
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
            jd: 0.0,
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
            new_moons: None,
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

    /// A Vedic day from `sunrise` to `next_sunrise`, its clock the same as UT.
    fn day(sunrise: f64, next_sunrise: f64) -> VedicDay {
        VedicDay {
            sunrise,
            sunset: sunrise + 0.5,
            next_sunrise,
            clock_sunrise: sunrise,
            clock_sunset: sunrise + 0.5,
            clock_next_sunrise: next_sunrise,
        }
    }

    /// 2000-01-05, a Wednesday, at 00:00.
    const WEDNESDAY: f64 = 2451548.5;

    fn hours(h: u8, m: u8) -> f64 {
        (f64::from(h) + f64::from(m) / 60.0) / 24.0
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
        let lords = [
            Vaara::Sunday,
            Vaara::Monday,
            Vaara::Tuesday,
            Vaara::Wednesday,
            Vaara::Thursday,
            Vaara::Friday,
            Vaara::Saturday,
        ]
        .map(vaara_lord);
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
}

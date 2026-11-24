use crate::*;

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
    // 3°20′ is no f64 but is 12000″, so whole arcseconds split exactly.
    let pada = (arcseconds_from_aries(reduce(longitude)) / (NAKSHATRA / 4)) as usize;
    (NAKSHATRAS[pada / 4], (pada % 4) as u8 + 1)
}

/// The graha ruling `nakshatra` in the Vimsottari dasa: nine lords from Ketu, repeated (ch. 1 §1.3.6, Table 2).
pub fn vimsottari_lord(nakshatra: Nakshatra) -> Body {
    use Body::*;
    [Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury][nakshatra as usize % 9]
}

/// Whole arcseconds the Moon is ahead of the Sun, in [0, 360°); truncated like `arcseconds`, so a boundary built from printed values holds.
fn moon_ahead(sun: f64, moon: f64) -> u32 {
    arcseconds_from_aries(reduce(moon - sun))
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
    let index = (moon_ahead(sun, moon) / TITHI) as u8 + 1;
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

/// The nakshatras the Moon most likely holds at `month`'s full Moon, which name the month (ch. 1 §1.3.8.2, Table 4).
pub fn full_moon_nakshatras(month: LunarMonth) -> &'static [Nakshatra] {
    use Nakshatra::*;
    match month {
        LunarMonth::Chaitra => &[Chitra],
        LunarMonth::Vaisaakha => &[Visaakha],
        LunarMonth::Jyeshtha => &[Jyeshtha],
        LunarMonth::Aashaadha => &[Poorvaashaadha, Uttaraashaadha],
        LunarMonth::Sraavana => &[Sravanam],
        LunarMonth::Bhaadrapada => &[Poorvaabhaadra, Uttaraabhaadra],
        LunarMonth::Aaswayuja => &[Aswini],
        LunarMonth::Kaarteeka => &[Krittika],
        LunarMonth::Maargasira => &[Mrigasira],
        LunarMonth::Pushya => &[Pushyami],
        LunarMonth::Maagha => &[Makha],
        LunarMonth::Phaalguna => &[PoorvaPhalguni, UttaraPhalguni],
    }
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
    match (moon_ahead(sun, moon) / KARANA) as usize {
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

/// The seven vaaras, Sunday first (ch. 1 §1.3.11).
pub const VAARAS: [Vaara; 7] = [
    Vaara::Sunday,
    Vaara::Monday,
    Vaara::Tuesday,
    Vaara::Wednesday,
    Vaara::Thursday,
    Vaara::Friday,
    Vaara::Saturday,
];

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
mod tests;

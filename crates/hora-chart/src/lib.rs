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

/// The nine grahas in the book's order (ch. 1 §1.3.1).
pub const GRAHAS: [Body; 9] = [
    Body::Sun,
    Body::Moon,
    Body::Mars,
    Body::Mercury,
    Body::Jupiter,
    Body::Venus,
    Body::Saturn,
    Body::Rahu,
    Body::Ketu,
];

/// The seven planets, the grahas without the nodes; only they have natural relationships (ch. 3 §3.4.1, Table 7).
pub const PLANETS: [Body; 7] = [
    Body::Sun,
    Body::Moon,
    Body::Mars,
    Body::Mercury,
    Body::Jupiter,
    Body::Venus,
    Body::Saturn,
];

/// The chart of `sky`: the lagna, then the nine grahas in the book's order.
pub fn chart(sky: &Sky) -> [Entry; 10] {
    let g = &sky.grahas;
    let entry = |body, longitude, retrograde| Entry {
        body,
        longitude,
        retrograde,
    };
    // The same rule for all nine, so the mean nodes always carry R (glossary: Retrograde; #43).
    let graha = |body, p: Position| entry(body, p.longitude, p.speed < 0.0);
    [
        entry(Body::Lagna, sky.lagna, false),
        graha(Body::Sun, g.sun),
        graha(Body::Moon, g.moon),
        graha(Body::Mars, g.mars),
        graha(Body::Mercury, g.mercury),
        graha(Body::Jupiter, g.jupiter),
        graha(Body::Venus, g.venus),
        graha(Body::Saturn, g.saturn),
        graha(Body::Rahu, g.rahu),
        // Ketu stands exactly opposite Rahu (ch. 1 *Basic Concepts*), so moves with it.
        entry(
            Body::Ketu,
            (g.rahu.longitude + 180.0).rem_euclid(360.0),
            g.rahu.speed < 0.0,
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

/// `longitude` reduced to [0°, 360°) (ch. 4 §4.2, footnote 8).
fn reduce(longitude: f64) -> f64 {
    // rem_euclid can round a tiny negative up to 360.0 itself, which is 0° Aries.
    longitude.rem_euclid(360.0) % 360.0
}

/// The rasi `longitude` falls in, and its degrees within that rasi, in [0, 30): each rasi is 30° from 0° Aries (ch. 1 *Basic Concepts*).
pub fn rasi(longitude: f64) -> (Rasi, f64) {
    let longitude = reduce(longitude);
    // `%` is exact, so the rasi and the degrees agree.
    let degrees = longitude % RASI_SPAN;
    let index = ((longitude - degrees) / RASI_SPAN).round() as usize % 12;
    (RASIS[index], degrees)
}

/// Degrees within a rasi, in [0, 30), as whole arcseconds, truncated: 29°59′59.9″ stays 29°59′59″.
/// `hora_swe`'s `part` splits the limbs the same way; its tests check that they agree.
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

/// A limb of Vishnu, whose body the zodiac is (ch. 2 §2.2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limb {
    Head,
    Face,
    Arms,
    Heart,
    Stomach,
    Hip,
    BelowNavel,
    PrivateParts,
    Thighs,
    Knees,
    Ankles,
    Feet,
}

/// Odd (male) or even (female) (ch. 2 §2.2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    Vishama,
    Sama,
}

/// Odd-footed or even-footed (ch. 2 §2.2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Footedness {
    Vishamapada,
    Samapada,
}

/// Movable, fixed or dual (ch. 2 §2.2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mobility {
    Chara,
    Sthira,
    Dwiswabhava,
}

/// Fire, earth, air, water or ether; aakaasa is in every rasi (ch. 2 §2.2.5) but is the element of only one graha, Jupiter (ch. 3 §3.2.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Agni,
    Bhoo,
    Vaayu,
    Jala,
    Aakaasa,
}

/// The Ayurvedic nature (ch. 2 §2.2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    Pitta,
    Vaata,
    Kapha,
    Mixed,
}

/// The guna (ch. 2 §2.2.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Sattwa,
    Rajas,
    Tamas,
}

/// The direction a rasi shows (ch. 2 §2.2.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    East,
    South,
    West,
    North,
}

/// The colour a rasi or graha shows; Taurus and Leo share white, Virgo and Capricorn variegated (ch. 2 §2.2.9, ch. 3 §3.2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    BloodRed,
    White,
    GrassGreen,
    PaleRed,
    Variegated,
    Black,
    ReddishBrown,
    HuskOfGrass,
    /// That of a mongoose.
    Brown,
    /// Or the colour of fish.
    Cream,
    Tawny,
}

/// Whether a rasi or graha is strong in the daytime (divaa) or at night (nishaa); Mercury always is (ch. 2 §2.2.10, ch. 3 §3.2.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrongBy {
    Divaa,
    Nishaa,
    Always,
}

/// Whether a rasi rises head first, feet first, or both (ch. 2 §2.2.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rising {
    Seershodaya,
    Prishthodaya,
    Both,
}

/// The varna (ch. 2 §2.2.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Brahmana,
    Kshatriya,
    Vaisya,
    Sudra,
}

impl Rasi {
    /// Ch. 2 §2.2.1.
    pub fn limb(self) -> Limb {
        use Rasi::*;
        match self {
            Aries => Limb::Head,
            Taurus => Limb::Face,
            Gemini => Limb::Arms,
            Cancer => Limb::Heart,
            Leo => Limb::Stomach,
            Virgo => Limb::Hip,
            Libra => Limb::BelowNavel,
            Scorpio => Limb::PrivateParts,
            Sagittarius => Limb::Thighs,
            Capricorn => Limb::Knees,
            Aquarius => Limb::Ankles,
            Pisces => Limb::Feet,
        }
    }

    /// Ch. 2 §2.2.2.
    pub fn parity(self) -> Parity {
        use Rasi::*;
        match self {
            Aries | Gemini | Leo | Libra | Sagittarius | Aquarius => Parity::Vishama,
            Taurus | Cancer | Virgo | Scorpio | Capricorn | Pisces => Parity::Sama,
        }
    }

    /// Ch. 2 §2.2.3.
    pub fn footedness(self) -> Footedness {
        use Rasi::*;
        match self {
            Aries | Taurus | Gemini | Libra | Scorpio | Sagittarius => Footedness::Vishamapada,
            Cancer | Leo | Virgo | Capricorn | Aquarius | Pisces => Footedness::Samapada,
        }
    }

    /// Ch. 2 §2.2.4.
    pub fn mobility(self) -> Mobility {
        use Rasi::*;
        match self {
            Aries | Cancer | Libra | Capricorn => Mobility::Chara,
            Taurus | Leo | Scorpio | Aquarius => Mobility::Sthira,
            Gemini | Virgo | Sagittarius | Pisces => Mobility::Dwiswabhava,
        }
    }

    /// Ch. 2 §2.2.5.
    pub fn element(self) -> Element {
        use Rasi::*;
        match self {
            Aries | Leo | Sagittarius => Element::Agni,
            Taurus | Virgo | Capricorn => Element::Bhoo,
            Gemini | Libra | Aquarius => Element::Vaayu,
            Cancer | Scorpio | Pisces => Element::Jala,
        }
    }

    /// Ch. 2 §2.2.6.
    pub fn nature(self) -> Nature {
        use Rasi::*;
        match self {
            Aries | Leo | Sagittarius => Nature::Pitta,
            Taurus | Virgo | Capricorn => Nature::Vaata,
            Cancer | Scorpio | Pisces => Nature::Kapha,
            Gemini | Libra | Aquarius => Nature::Mixed,
        }
    }

    /// Ch. 2 §2.2.7.
    pub fn quality(self) -> Quality {
        use Rasi::*;
        match self {
            Cancer | Leo | Sagittarius | Pisces => Quality::Sattwa,
            Aries | Taurus | Libra | Scorpio => Quality::Rajas,
            Gemini | Virgo | Capricorn | Aquarius => Quality::Tamas,
        }
    }

    /// Ch. 2 §2.2.8.
    pub fn direction(self) -> Direction {
        use Rasi::*;
        match self {
            Aries | Leo | Sagittarius => Direction::East,
            Taurus | Virgo | Capricorn => Direction::South,
            Gemini | Libra | Aquarius => Direction::West,
            Cancer | Scorpio | Pisces => Direction::North,
        }
    }

    /// Ch. 2 §2.2.9.
    pub fn colour(self) -> Colour {
        use Rasi::*;
        match self {
            Aries => Colour::BloodRed,
            Taurus | Leo => Colour::White,
            Gemini => Colour::GrassGreen,
            Cancer => Colour::PaleRed,
            Virgo | Capricorn => Colour::Variegated,
            Libra => Colour::Black,
            Scorpio => Colour::ReddishBrown,
            Sagittarius => Colour::HuskOfGrass,
            Aquarius => Colour::Brown,
            Pisces => Colour::Cream,
        }
    }

    /// Ch. 2 §2.2.10.
    pub fn strong_by(self) -> StrongBy {
        use Rasi::*;
        match self {
            Leo | Virgo | Libra | Scorpio | Aquarius | Pisces => StrongBy::Divaa,
            Aries | Taurus | Gemini | Cancer | Sagittarius | Capricorn => StrongBy::Nishaa,
        }
    }

    /// Ch. 2 §2.2.11.
    pub fn rising(self) -> Rising {
        use Rasi::*;
        match self {
            Gemini | Leo | Virgo | Libra | Scorpio | Aquarius => Rising::Seershodaya,
            Aries | Taurus | Cancer | Sagittarius | Capricorn => Rising::Prishthodaya,
            Pisces => Rising::Both,
        }
    }

    /// Ch. 2 §2.2.12.
    pub fn class(self) -> Class {
        use Rasi::*;
        match self {
            Cancer | Scorpio | Pisces => Class::Brahmana,
            Aries | Leo | Sagittarius => Class::Kshatriya,
            Taurus | Virgo | Capricorn => Class::Vaisya,
            Gemini | Libra | Aquarius => Class::Sudra,
        }
    }

    /// The seven-planet lord (Table 6's own rasis, ch. 3 §3.3); Rahu's Aquarius and Ketu's Scorpio count only in their own dignity (#123).
    pub fn lord(self) -> Body {
        use Rasi::*;
        match self {
            Aries | Scorpio => Body::Mars,
            Taurus | Libra => Body::Venus,
            Gemini | Virgo => Body::Mercury,
            Cancer => Body::Moon,
            Leo => Body::Sun,
            Sagittarius | Pisces => Body::Jupiter,
            Capricorn | Aquarius => Body::Saturn,
        }
    }
}

/// The avatara of Vishnu that came from a graha (ch. 3 §3.2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Avatara {
    /// Also Meena.
    Matsya,
    Koorma,
    /// Also Sookara.
    Varaaha,
    /// Also Nrisimha.
    Narasimha,
    Vaamana,
    /// Also Bhaargava Rama.
    ParasuRama,
    Rama,
    Krishna,
    Buddha,
}

/// Natural benefic (saumya) or malefic (kroora); Mercury's turns on his company, the Moon's on the paksha (ch. 3 §3.2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    Saumya,
    Kroora,
    /// Saumya alone or with more natural benefics, kroora when joined by more natural malefics.
    ByCompany,
    /// Saumya waxing in Sukla paksha, kroora waning in Krishna paksha.
    ByPaksha,
}

/// What a graha mainly governs (ch. 3 §3.2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Governance {
    Soul,
    Mind,
    Strength,
    Speech,
    KnowledgeAndHappiness,
    Potency,
    Grief,
}

/// A graha's place in the planetary cabinet (ch. 3 §3.2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cabinet {
    King,
    /// The army chief.
    Leader,
    Prince,
    Minister,
    Servant,
    Army,
}

/// A graha's ruling deity (ch. 3 §3.2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deity {
    Agni,
    Varuna,
    Subrahmanya,
    MahaVishnu,
    Indra,
    SachiDevi,
    Brahma,
}

/// A graha's sex; Mercury and Saturn are female as printed (ch. 3 §3.2.7, #123).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

/// Where a graha lives (ch. 3 §3.2.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abode {
    Temple,
    WateryPlace,
    SportsGround,
    TreasureHouse,
    Bedroom,
    FilthyArea,
}

/// One of the sapta dhaatus, the seven matters of the human body (ch. 3 §3.2.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaptaDhaatu {
    Bones,
    Blood,
    Marrow,
    Skin,
    Fat,
    Semen,
    Muscles,
}

/// The span of time a graha rules (ch. 3 §3.2.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Ayana,
    Minute,
    Week,
    Ritu,
    Month,
    Fortnight,
    Year,
}

/// The taste a graha governs (ch. 3 §3.2.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taste {
    Pungent,
    Saline,
    Bitter,
    Mixed,
    Sweet,
    Sour,
    Astringent,
}

/// Uttara while the Sun moves from Capricorn to Gemini, Dakshina from Cancer to Sagittarius (ch. 3 §3.2.13, note 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ayana {
    Uttara,
    Dakshina,
}

/// The six two-month seasons (ch. 3 §3.2.13, note 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ritu {
    Vasanta,
    Greeshma,
    Varsha,
    Hemanta,
    Seeta,
    Sisira,
}

/// What a graha rules: dhatus (metals and materials), moolas (roots and vegetables) or jeevas (living beings) (ch. 3 §3.2.17).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Realm {
    Dhatu,
    Moola,
    Jeeva,
}

/// The §3.2 lists of ch. 3; each is `None` for the lagna and for a graha the book leaves out, never filled from elsewhere (#123).
impl Body {
    /// Ch. 3 §3.2.1.
    pub fn avatara(self) -> Option<Avatara> {
        use Body::*;
        Some(match self {
            Lagna => return None,
            Sun => Avatara::Rama,
            Moon => Avatara::Krishna,
            Mars => Avatara::Narasimha,
            Mercury => Avatara::Buddha,
            Jupiter => Avatara::Vaamana,
            Venus => Avatara::ParasuRama,
            Saturn => Avatara::Koorma,
            Rahu => Avatara::Varaaha,
            Ketu => Avatara::Matsya,
        })
    }

    /// Ch. 3 §3.2.2, with Saturn from ch. 13 §13.2.
    pub fn disposition(self) -> Option<Disposition> {
        use Body::*;
        Some(match self {
            Lagna => return None,
            Jupiter | Venus => Disposition::Saumya,
            // §3.2.2 leaves Saturn out; §13.2 restates the list with him a natural malefic.
            Sun | Mars | Saturn | Rahu | Ketu => Disposition::Kroora,
            Mercury => Disposition::ByCompany,
            Moon => Disposition::ByPaksha,
        })
    }

    /// Ch. 3 §3.2.3.
    pub fn governance(self) -> Option<Governance> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Sun => Governance::Soul,
            Moon => Governance::Mind,
            Mars => Governance::Strength,
            Mercury => Governance::Speech,
            Jupiter => Governance::KnowledgeAndHappiness,
            Venus => Governance::Potency,
            Saturn => Governance::Grief,
        })
    }

    /// Ch. 3 §3.2.4.
    pub fn colour(self) -> Option<Colour> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Sun | Mars => Colour::BloodRed,
            Moon | Jupiter => Colour::Tawny,
            Mercury => Colour::GrassGreen,
            Venus => Colour::Variegated,
            Saturn => Colour::Black,
        })
    }

    /// Ch. 3 §3.2.5.
    pub fn cabinet(self) -> Option<Cabinet> {
        use Body::*;
        Some(match self {
            Lagna => return None,
            Sun | Moon => Cabinet::King,
            Mars => Cabinet::Leader,
            Mercury => Cabinet::Prince,
            Jupiter | Venus => Cabinet::Minister,
            Saturn => Cabinet::Servant,
            Rahu | Ketu => Cabinet::Army,
        })
    }

    /// Ch. 3 §3.2.6.
    pub fn deity(self) -> Option<Deity> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Sun => Deity::Agni,
            Moon => Deity::Varuna,
            Mars => Deity::Subrahmanya,
            Mercury => Deity::MahaVishnu,
            Jupiter => Deity::Indra,
            Venus => Deity::SachiDevi,
            Saturn => Deity::Brahma,
        })
    }

    /// Ch. 3 §3.2.7.
    pub fn sex(self) -> Option<Sex> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Sun | Mars | Jupiter => Sex::Male,
            Moon | Venus | Saturn | Mercury => Sex::Female,
        })
    }

    /// Ch. 3 §3.2.8: the Sun shares Mars's nature, the Moon Venus's.
    pub fn element(self) -> Option<Element> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Mars | Sun => Element::Agni,
            Mercury => Element::Bhoo,
            Saturn => Element::Vaayu,
            Jupiter => Element::Aakaasa,
            Venus | Moon => Element::Jala,
        })
    }

    /// Ch. 3 §3.2.9.
    pub fn class(self) -> Option<Class> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Jupiter | Venus => Class::Brahmana,
            Sun | Mars => Class::Kshatriya,
            Moon | Mercury => Class::Vaisya,
            Saturn => Class::Sudra,
        })
    }

    /// Ch. 3 §3.2.10.
    pub fn quality(self) -> Option<Quality> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Sun | Moon | Jupiter => Quality::Sattwa,
            Mercury | Venus => Quality::Rajas,
            Mars | Saturn => Quality::Tamas,
        })
    }

    /// Ch. 3 §3.2.11; the book gives Mars none.
    pub fn abode(self) -> Option<Abode> {
        use Body::*;
        Some(match self {
            Lagna | Mars | Rahu | Ketu => return None,
            Sun => Abode::Temple,
            Moon => Abode::WateryPlace,
            Mercury => Abode::SportsGround,
            Jupiter => Abode::TreasureHouse,
            Venus => Abode::Bedroom,
            Saturn => Abode::FilthyArea,
        })
    }

    /// Ch. 3 §3.2.12.
    pub fn sapta_dhaatu(self) -> Option<SaptaDhaatu> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Sun => SaptaDhaatu::Bones,
            Moon => SaptaDhaatu::Blood,
            Mars => SaptaDhaatu::Marrow,
            Mercury => SaptaDhaatu::Skin,
            Jupiter => SaptaDhaatu::Fat,
            Venus => SaptaDhaatu::Semen,
            Saturn => SaptaDhaatu::Muscles,
        })
    }

    /// Ch. 3 §3.2.13.
    pub fn period(self) -> Option<Period> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Sun => Period::Ayana,
            Moon => Period::Minute,
            Mars => Period::Week,
            Mercury => Period::Ritu,
            Jupiter => Period::Month,
            Venus => Period::Fortnight,
            Saturn => Period::Year,
        })
    }

    /// Ch. 3 §3.2.14.
    pub fn taste(self) -> Option<Taste> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Sun => Taste::Pungent,
            Moon => Taste::Saline,
            Mars => Taste::Bitter,
            Mercury => Taste::Mixed,
            Jupiter => Taste::Sweet,
            Venus => Taste::Sour,
            Saturn => Taste::Astringent,
        })
    }

    /// The direction of a graha's digbala (ch. 3 §3.2.15).
    pub fn digbala(self) -> Option<Direction> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Mercury | Jupiter => Direction::East,
            Sun | Mars => Direction::South,
            Moon | Venus => Direction::North,
            Saturn => Direction::West,
        })
    }

    /// Ch. 3 §3.2.15.
    pub fn strong_by(self) -> Option<StrongBy> {
        use Body::*;
        Some(match self {
            Lagna | Rahu | Ketu => return None,
            Moon | Mars | Saturn => StrongBy::Nishaa,
            Sun | Jupiter | Venus => StrongBy::Divaa,
            Mercury => StrongBy::Always,
        })
    }

    /// Natural benefics are strong in Sukla paksha, malefics in Krishna (ch. 3 §3.2.15); none for Mercury and the Moon, whose nature turns, nor the nodes.
    pub fn strong_paksha(self) -> Option<Paksha> {
        // Time strength is the seven planets' (#128), though §3.2.2 makes the nodes natural malefics.
        if matches!(self, Body::Rahu | Body::Ketu) {
            return None;
        }
        match self.disposition()? {
            Disposition::Saumya => Some(Paksha::Sukla),
            Disposition::Kroora => Some(Paksha::Krishna),
            Disposition::ByCompany | Disposition::ByPaksha => None,
        }
    }

    /// Natural benefics are strong in Uttara ayana, malefics in Dakshina (ch. 3 §3.2.15); none for Mercury and the Moon, whose nature turns, nor the nodes.
    pub fn strong_ayana(self) -> Option<Ayana> {
        if matches!(self, Body::Rahu | Body::Ketu) {
            return None;
        }
        match self.disposition()? {
            Disposition::Saumya => Some(Ayana::Uttara),
            Disposition::Kroora => Some(Ayana::Dakshina),
            Disposition::ByCompany | Disposition::ByPaksha => None,
        }
    }

    /// Ch. 3 §3.2.16; the book gives the Sun none.
    pub fn ritu(self) -> Option<Ritu> {
        use Body::*;
        Some(match self {
            Lagna | Sun | Rahu | Ketu => return None,
            Venus => Ritu::Vasanta,
            Mars => Ritu::Greeshma,
            Moon => Ritu::Varsha,
            Mercury => Ritu::Hemanta,
            Jupiter => Ritu::Seeta,
            Saturn => Ritu::Sisira,
        })
    }

    /// Ch. 3 §3.2.17.
    pub fn realm(self) -> Option<Realm> {
        use Body::*;
        Some(match self {
            Lagna => return None,
            Rahu | Mars | Saturn | Moon => Realm::Dhatu,
            Sun | Venus => Realm::Moola,
            Mercury | Jupiter | Ketu => Realm::Jeeva,
        })
    }

    /// Table 6 (ch. 3 §3.3), with §3.3's degree splits in the moolatrikona rasi; none for the lagna.
    pub fn dignities(self) -> Option<Dignities> {
        use Rasi::*;
        let row = |own, uchcha, neecha, moolatrikona| Dignities {
            own,
            uchcha,
            neecha,
            moolatrikona,
        };
        Some(match self {
            Body::Lagna => return None,
            Body::Sun => row(&[Leo], (Aries, Some(10)), (Libra, Some(10)), (Leo, 0, 20)),
            Body::Moon => row(
                &[Cancer],
                (Taurus, Some(3)),
                (Scorpio, Some(3)),
                (Taurus, 3, 30),
            ),
            // "First 12º of Le" in §3.3 (3) is a typo for Aries (#123).
            Body::Mars => row(
                &[Aries, Scorpio],
                (Capricorn, Some(28)),
                (Cancer, Some(28)),
                (Aries, 0, 12),
            ),
            Body::Mercury => row(
                &[Gemini, Virgo],
                (Virgo, Some(15)),
                (Pisces, Some(15)),
                (Virgo, 15, 20),
            ),
            Body::Jupiter => row(
                &[Sagittarius, Pisces],
                (Cancer, Some(5)),
                (Capricorn, Some(5)),
                (Sagittarius, 0, 10),
            ),
            Body::Venus => row(
                &[Taurus, Libra],
                (Pisces, Some(27)),
                (Virgo, Some(27)),
                (Libra, 0, 15),
            ),
            Body::Saturn => row(
                &[Capricorn, Aquarius],
                (Libra, Some(20)),
                (Aries, Some(20)),
                (Aquarius, 0, 20),
            ),
            // ADR 0003: the book's node dignities only.
            Body::Rahu => row(
                &[Aquarius],
                (Gemini, None),
                (Sagittarius, None),
                (Virgo, 0, 30),
            ),
            Body::Ketu => row(
                &[Scorpio],
                (Sagittarius, None),
                (Gemini, None),
                (Pisces, 0, 30),
            ),
        })
    }
}

/// A graha's row of Table 6 (ch. 3 §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dignities {
    pub own: &'static [Rasi],
    /// The uchcha rasi and its deep exaltation degree; the book prints none for the nodes.
    pub uchcha: (Rasi, Option<u8>),
    /// The neecha rasi and its deep debilitation degree.
    pub neecha: (Rasi, Option<u8>),
    /// The moolatrikona rasi and the degrees, from and up to but not including, that give its results (§3.3 notes 1–7).
    pub moolatrikona: (Rasi, u8, u8),
}

/// Ch. 3 §3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dignity {
    Uchcha,
    Moolatrikona,
    OwnRasi,
    Neecha,
}

/// `body`'s dignity at `longitude`, none in any other rasi; where its rasi is two, §3.3's degree splits decide (ch. 3 §3.3).
pub fn dignity(body: Body, longitude: f64) -> Option<Dignity> {
    let d = body.dignities()?;
    let (r, degrees) = rasi(longitude);
    // Whole arcseconds, as everywhere a printed boundary must hold: 2°59′59″ Taurus is the Moon's uchcha, 3°00′ not.
    let at = arcseconds(degrees);
    let deg = |d: u8| u32::from(d) * 3600;
    let (mt, from, to) = d.moolatrikona;
    Some(if r == mt && (deg(from)..deg(to)).contains(&at) {
        Dignity::Moolatrikona
    } else if r == d.uchcha.0 && (r != mt || at < deg(from)) {
        // Mercury's Virgo is uchcha only before his moolatrikona span, own rasi after it.
        Dignity::Uchcha
    } else if d.own.contains(&r) {
        Dignity::OwnRasi
    } else if r == d.neecha.0 {
        Dignity::Neecha
    } else {
        return None;
    })
}

/// Ch. 3 §3.4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Natural {
    Mitra,
    Sama,
    Satru,
}

/// How `graha` sees `other` by nature: from its moolatrikona, the lords of its uchcha rasi and of the 2nd, 4th, 5th, 8th, 9th
/// and 12th rasis are friends, other lords enemies, both neutral (ch. 3 §3.4.1). None unless both are the seven planets, and differ.
pub fn natural(graha: Body, other: Body) -> Option<Natural> {
    if graha == other || !PLANETS.contains(&graha) || !PLANETS.contains(&other) {
        return None;
    }
    let d = graha.dignities()?;
    let mut friend = d.uchcha.0.lord() == other;
    let mut enemy = false;
    for r in RASIS.into_iter().filter(|r| r.lord() == other) {
        match bhava(d.moolatrikona.0, r) {
            2 | 4 | 5 | 8 | 9 | 12 => friend = true,
            _ => enemy = true,
        }
    }
    Some(match (friend, enemy) {
        (true, false) => Natural::Mitra,
        (false, true) => Natural::Satru,
        _ => Natural::Sama,
    })
}

/// Ch. 3 §3.4.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tatkaala {
    Mitra,
    Satru,
}

/// How a graha in `rasi` sees one in `other` in a chart: a friend in the 2nd, 3rd, 4th, 10th, 11th or 12th, else an enemy (ch. 3 §3.4.2).
/// Takes rasis, so a chart without birth data (Rama's) can be tested.
pub fn tatkaala(rasi: Rasi, other: Rasi) -> Tatkaala {
    match bhava(rasi, other) {
        2 | 3 | 4 | 10 | 11 | 12 => Tatkaala::Mitra,
        _ => Tatkaala::Satru,
    }
}

/// Ch. 3 §3.4.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compound {
    Adhimitra,
    Mitra,
    Sama,
    Satru,
    Adhisatru,
}

/// Table 8 (ch. 3 §3.4.3).
pub fn compound(natural: Natural, tatkaala: Tatkaala) -> Compound {
    match (natural, tatkaala) {
        (Natural::Mitra, Tatkaala::Mitra) => Compound::Adhimitra,
        (Natural::Mitra, Tatkaala::Satru) | (Natural::Satru, Tatkaala::Mitra) => Compound::Sama,
        (Natural::Sama, Tatkaala::Mitra) => Compound::Mitra,
        (Natural::Sama, Tatkaala::Satru) => Compound::Satru,
        (Natural::Satru, Tatkaala::Satru) => Compound::Adhisatru,
    }
}

/// `body`'s compound relationship with the lord of the rasi it occupies in `chart`; none in its own rasi, or for Rahu, Ketu
/// and the lagna (ch. 3 §3.4.3). Separate from its dignity: Saturn in Aries is neecha whatever he is to Mars.
pub fn lord_relationship(chart: &[Entry; 10], body: Body) -> Option<Compound> {
    let r = rasi_of(chart, body);
    let lord = r.lord();
    Some(compound(
        natural(body, lord)?,
        tatkaala(r, rasi_of(chart, lord)),
    ))
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

/// Ch. 4 §4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Upagraha {
    Dhuma,
    Vyatipaata,
    Parivesha,
    Indrachaapa,
    Upaketu,
    Kaala,
    Mrityu,
    Arthaprahaara,
    Yamaghantaka,
    Gulika,
    Maandi,
}

/// The eleven upagrahas in the book's order: five Sun-based, then six time-based (ch. 4 §4.2–4.3).
pub const UPAGRAHAS: [Upagraha; 11] = [
    Upagraha::Dhuma,
    Upagraha::Vyatipaata,
    Upagraha::Parivesha,
    Upagraha::Indrachaapa,
    Upagraha::Upaketu,
    Upagraha::Kaala,
    Upagraha::Mrityu,
    Upagraha::Arthaprahaara,
    Upagraha::Yamaghantaka,
    Upagraha::Gulika,
    Upagraha::Maandi,
];

/// What the book says of an upagraha's nature, no more: only Kaala and Mrityu are called malefic among the six (ch. 4 §4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpagrahaNature {
    VeryMalefic,
    MaleficLikeSun,
    MaleficLikeMars,
    LikeMercury,
    LikeJupiter,
    LikeSaturn,
}

impl Upagraha {
    /// Ch. 4 §4.2–4.3.
    pub fn nature(self) -> UpagrahaNature {
        use Upagraha::*;
        match self {
            Dhuma | Vyatipaata | Parivesha | Indrachaapa | Upaketu => UpagrahaNature::VeryMalefic,
            Kaala => UpagrahaNature::MaleficLikeSun,
            Mrityu => UpagrahaNature::MaleficLikeMars,
            Arthaprahaara => UpagrahaNature::LikeMercury,
            Yamaghantaka => UpagrahaNature::LikeJupiter,
            Gulika | Maandi => UpagrahaNature::LikeSaturn,
        }
    }
}

/// The five Sun-based upagrahas from the Sun's `sun` longitude, each reduced to [0°, 360°) (ch. 4 §4.2, Table 9, footnote 8).
pub fn sun_based(sun: f64) -> [(Upagraha, f64); 5] {
    let dhuma = reduce(sun + 133.0 + 20.0 / 60.0);
    let vyatipaata = reduce(360.0 - dhuma);
    let parivesha = reduce(vyatipaata + 180.0);
    let indrachaapa = reduce(360.0 - parivesha);
    let upaketu = reduce(indrachaapa + 16.0 + 40.0 / 60.0);
    use Upagraha::*;
    [
        (Dhuma, dhuma),
        (Vyatipaata, vyatipaata),
        (Parivesha, parivesha),
        (Indrachaapa, indrachaapa),
        (Upaketu, upaketu),
    ]
}

/// The graha ruling each eighth of `vaara`'s day, or its `night`, `None` for the lord-less part: by day the vaara's graha
/// first, by night the 5th from it, then on in weekday order with a lord-less part after Saturn's (ch. 4 §4.3, Table 10).
pub fn part_rulers(vaara: Vaara, night: bool) -> [Option<Body>; 8] {
    use Body::*;
    const ORDER: [Option<Body>; 8] = [
        Some(Sun),
        Some(Moon),
        Some(Mars),
        Some(Mercury),
        Some(Jupiter),
        Some(Venus),
        Some(Saturn),
        None,
    ];
    // The 5th graha is counted among the seven, without the lord-less part.
    let first = (vaara as usize + if night { 4 } else { 0 }) % 7;
    std::array::from_fn(|k| ORDER[(first + k) % 8])
}

/// The upagrahas of `sky` with their longitudes, in the book's order: the five Sun-based for every chart, then the six
/// time-based where the Sky has a Vedic day, Gulika at the start of Saturn's part and the rest at a middle (ch. 4, ADR 0003).
pub fn upagrahas(sky: &Sky) -> Vec<(Upagraha, f64)> {
    let mut upagrahas = sun_based(sky.grahas.sun.longitude).to_vec();
    if let (Some(day), Some(lagnas)) = (&sky.vedic_day, &sky.part_lagnas) {
        let rulers = part_rulers(vaara(day), day.is_night(sky.jd));
        // Each graha rules one of the eight parts.
        let lagna = |graha, middle: bool| {
            lagnas[rulers.iter().position(|&r| r == Some(graha)).unwrap()][usize::from(middle)]
        };
        use Upagraha::*;
        upagrahas.extend([
            (Kaala, lagna(Body::Sun, true)),
            (Mrityu, lagna(Body::Mars, true)),
            (Arthaprahaara, lagna(Body::Mercury, true)),
            (Yamaghantaka, lagna(Body::Jupiter, true)),
            (Gulika, lagna(Body::Saturn, false)),
            (Maandi, lagna(Body::Saturn, true)),
        ]);
    }
    upagrahas
}

#[cfg(test)]
mod tests {
    use hora_swe::{Grahas, Limbs};

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
            limbs: Limbs::default(),
            part_lagnas: None,
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
    fn grahas_follow_the_charts_order() {
        let bodies = chart(&chart_1()).map(|entry| entry.body);
        assert_eq!(bodies[1..], GRAHAS);
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
            assert_eq!(entry.retrograde, matches!(body, Rahu | Ketu), "{body:?}");
        }
    }

    #[test]
    fn every_graha_moving_backward_is_retrograde_and_ketu_is_opposite_rahu() {
        let chart = chart(&sky(-1.0, [[350.0, -1.0]; 8]));
        let marked: Vec<_> = chart
            .iter()
            .filter(|e| e.retrograde)
            .map(|e| e.body)
            .collect();
        use Body::*;
        assert_eq!(
            marked,
            [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu]
        );
        assert_eq!(chart[9].body, Ketu);
        assert_eq!(chart[9].longitude, 170.0);
        // A true node moving forward loses its R, Ketu with it.
        let forward = super::chart(&sky(-1.0, [[350.0, 1.0]; 8]));
        assert!(forward.iter().all(|e| !e.retrograde));
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

    const VAARAS: [Vaara; 7] = [
        Vaara::Sunday,
        Vaara::Monday,
        Vaara::Tuesday,
        Vaara::Wednesday,
        Vaara::Thursday,
        Vaara::Friday,
        Vaara::Saturday,
    ];

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

    /// The rasis, in zodiac order, whose `characteristic` is `value`; the book lists most characteristics so.
    fn rasis_with<T: PartialEq>(characteristic: fn(Rasi) -> T, value: T) -> Vec<Rasi> {
        RASIS
            .into_iter()
            .filter(|&r| characteristic(r) == value)
            .collect()
    }

    #[test]
    fn limbs_of_vishnu_are_section_2_2_1s() {
        use Limb::*;
        let book = [
            Head,
            Face,
            Arms,
            Heart,
            Stomach,
            Hip,
            BelowNavel,
            PrivateParts,
            Thighs,
            Knees,
            Ankles,
            Feet,
        ];
        assert_eq!(RASIS.map(Rasi::limb), book);
    }

    #[test]
    fn odd_and_even_rasis_are_section_2_2_2s() {
        use Rasi::*;
        assert_eq!(
            rasis_with(Rasi::parity, Parity::Vishama),
            [Aries, Gemini, Leo, Libra, Sagittarius, Aquarius]
        );
        assert_eq!(
            rasis_with(Rasi::parity, Parity::Sama),
            [Taurus, Cancer, Virgo, Scorpio, Capricorn, Pisces]
        );
    }

    #[test]
    fn odd_and_even_footed_rasis_are_section_2_2_3s() {
        use Rasi::*;
        assert_eq!(
            rasis_with(Rasi::footedness, Footedness::Vishamapada),
            [Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius]
        );
        assert_eq!(
            rasis_with(Rasi::footedness, Footedness::Samapada),
            [Cancer, Leo, Virgo, Capricorn, Aquarius, Pisces]
        );
    }

    #[test]
    fn movable_fixed_and_dual_rasis_are_section_2_2_4s() {
        use Rasi::*;
        let with = |value| rasis_with(Rasi::mobility, value);
        assert_eq!(with(Mobility::Chara), [Aries, Cancer, Libra, Capricorn]);
        assert_eq!(with(Mobility::Sthira), [Taurus, Leo, Scorpio, Aquarius]);
        assert_eq!(
            with(Mobility::Dwiswabhava),
            [Gemini, Virgo, Sagittarius, Pisces]
        );
    }

    #[test]
    fn elements_are_section_2_2_5s() {
        use Rasi::*;
        let with = |value| rasis_with(Rasi::element, value);
        assert_eq!(with(Element::Agni), [Aries, Leo, Sagittarius]);
        assert_eq!(with(Element::Bhoo), [Taurus, Virgo, Capricorn]);
        assert_eq!(with(Element::Vaayu), [Gemini, Libra, Aquarius]);
        assert_eq!(with(Element::Jala), [Cancer, Scorpio, Pisces]);
    }

    #[test]
    fn pitta_vaata_and_kapha_are_section_2_2_6s() {
        use Rasi::*;
        let with = |value| rasis_with(Rasi::nature, value);
        assert_eq!(with(Nature::Pitta), [Aries, Leo, Sagittarius]);
        assert_eq!(with(Nature::Vaata), [Taurus, Virgo, Capricorn]);
        assert_eq!(with(Nature::Kapha), [Cancer, Scorpio, Pisces]);
        assert_eq!(with(Nature::Mixed), [Gemini, Libra, Aquarius]);
    }

    #[test]
    fn gunas_are_section_2_2_7s() {
        use Rasi::*;
        let with = |value| rasis_with(Rasi::quality, value);
        assert_eq!(with(Quality::Sattwa), [Cancer, Leo, Sagittarius, Pisces]);
        assert_eq!(with(Quality::Rajas), [Aries, Taurus, Libra, Scorpio]);
        assert_eq!(with(Quality::Tamas), [Gemini, Virgo, Capricorn, Aquarius]);
    }

    #[test]
    fn directions_are_section_2_2_8s() {
        use Rasi::*;
        let with = |value| rasis_with(Rasi::direction, value);
        assert_eq!(with(Direction::East), [Aries, Leo, Sagittarius]);
        assert_eq!(with(Direction::South), [Taurus, Virgo, Capricorn]);
        assert_eq!(with(Direction::West), [Gemini, Libra, Aquarius]);
        assert_eq!(with(Direction::North), [Cancer, Scorpio, Pisces]);
    }

    #[test]
    fn colours_are_section_2_2_9s() {
        use Colour::*;
        let book = [
            BloodRed,
            White,
            GrassGreen,
            PaleRed,
            White,
            Variegated,
            Black,
            ReddishBrown,
            HuskOfGrass,
            Variegated,
            Brown,
            Cream,
        ];
        assert_eq!(RASIS.map(Rasi::colour), book);
    }

    #[test]
    fn day_and_night_rasis_are_section_2_2_10s() {
        use Rasi::*;
        assert_eq!(
            rasis_with(Rasi::strong_by, StrongBy::Nishaa),
            [Aries, Taurus, Gemini, Cancer, Sagittarius, Capricorn]
        );
        assert_eq!(
            rasis_with(Rasi::strong_by, StrongBy::Divaa),
            [Leo, Virgo, Libra, Scorpio, Aquarius, Pisces]
        );
    }

    #[test]
    fn rising_is_section_2_2_11s() {
        use Rasi::*;
        let with = |value| rasis_with(Rasi::rising, value);
        assert_eq!(
            with(Rising::Seershodaya),
            [Gemini, Leo, Virgo, Libra, Scorpio, Aquarius]
        );
        assert_eq!(
            with(Rising::Prishthodaya),
            [Aries, Taurus, Cancer, Sagittarius, Capricorn]
        );
        assert_eq!(with(Rising::Both), [Pisces]);
    }

    #[test]
    fn varnas_are_section_2_2_12s() {
        use Rasi::*;
        let with = |value| rasis_with(Rasi::class, value);
        assert_eq!(with(Class::Brahmana), [Cancer, Scorpio, Pisces]);
        assert_eq!(with(Class::Kshatriya), [Aries, Leo, Sagittarius]);
        assert_eq!(with(Class::Vaisya), [Taurus, Virgo, Capricorn]);
        assert_eq!(with(Class::Sudra), [Gemini, Libra, Aquarius]);
    }

    // Each §3.2 test writes the book's list out for the nine grahas in GRAHAS's order: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu.

    #[test]
    fn avataras_are_section_3_2_1s() {
        use Avatara::*;
        let book = [
            Rama, Krishna, Narasimha, Buddha, Vaamana, ParasuRama, Koorma, Varaaha, Matsya,
        ];
        assert_eq!(GRAHAS.map(Body::avatara), book.map(Some));
    }

    #[test]
    fn saumya_and_kroora_grahas_are_section_3_2_2s_with_saturn_from_13_2() {
        use Disposition::*;
        let book = [
            Some(Kroora),
            Some(ByPaksha),
            Some(Kroora),
            Some(ByCompany),
            Some(Saumya),
            Some(Saumya),
            Some(Kroora),
            Some(Kroora),
            Some(Kroora),
        ];
        assert_eq!(GRAHAS.map(Body::disposition), book);
    }

    #[test]
    fn main_governances_are_section_3_2_3s() {
        use Governance::*;
        let book = [
            Some(Soul),
            Some(Mind),
            Some(Strength),
            Some(Speech),
            Some(KnowledgeAndHappiness),
            Some(Potency),
            Some(Grief),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::governance), book);
    }

    #[test]
    fn graha_colours_are_section_3_2_4s() {
        use Colour::*;
        let book = [
            Some(BloodRed),
            Some(Tawny),
            Some(BloodRed),
            Some(GrassGreen),
            Some(Tawny),
            Some(Variegated),
            Some(Black),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::colour), book);
    }

    #[test]
    fn the_planetary_cabinet_is_section_3_2_5s() {
        use Cabinet::*;
        let book = [
            King, King, Leader, Prince, Minister, Minister, Servant, Army, Army,
        ];
        assert_eq!(GRAHAS.map(Body::cabinet), book.map(Some));
    }

    #[test]
    fn deities_are_section_3_2_6s() {
        use Deity::*;
        let book = [
            Some(Agni),
            Some(Varuna),
            Some(Subrahmanya),
            Some(MahaVishnu),
            Some(Indra),
            Some(SachiDevi),
            Some(Brahma),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::deity), book);
    }

    #[test]
    fn sexes_are_section_3_2_7s_with_mercury_and_saturn_female() {
        use Sex::*;
        let book = [
            Some(Male),
            Some(Female),
            Some(Male),
            Some(Female),
            Some(Male),
            Some(Female),
            Some(Female),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::sex), book);
    }

    #[test]
    fn graha_elements_are_section_3_2_8s() {
        use Element::*;
        let book = [
            Some(Agni),
            Some(Jala),
            Some(Agni),
            Some(Bhoo),
            Some(Aakaasa),
            Some(Jala),
            Some(Vaayu),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::element), book);
    }

    #[test]
    fn graha_varnas_are_section_3_2_9s() {
        use Class::*;
        let book = [
            Some(Kshatriya),
            Some(Vaisya),
            Some(Kshatriya),
            Some(Vaisya),
            Some(Brahmana),
            Some(Brahmana),
            Some(Sudra),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::class), book);
    }

    #[test]
    fn graha_gunas_are_section_3_2_10s() {
        use Quality::*;
        let book = [
            Some(Sattwa),
            Some(Sattwa),
            Some(Tamas),
            Some(Rajas),
            Some(Sattwa),
            Some(Rajas),
            Some(Tamas),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::quality), book);
    }

    #[test]
    fn abodes_are_section_3_2_11s_with_none_for_mars() {
        use Abode::*;
        let book = [
            Some(Temple),
            Some(WateryPlace),
            None,
            Some(SportsGround),
            Some(TreasureHouse),
            Some(Bedroom),
            Some(FilthyArea),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::abode), book);
    }

    #[test]
    fn sapta_dhaatus_are_section_3_2_12s() {
        use SaptaDhaatu::*;
        let book = [
            Some(Bones),
            Some(Blood),
            Some(Marrow),
            Some(Skin),
            Some(Fat),
            Some(Semen),
            Some(Muscles),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::sapta_dhaatu), book);
    }

    #[test]
    fn time_periods_are_section_3_2_13s() {
        use Period::*;
        let book = [
            Some(Ayana),
            Some(Minute),
            Some(Week),
            Some(Ritu),
            Some(Month),
            Some(Fortnight),
            Some(Year),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::period), book);
    }

    #[test]
    fn tastes_are_section_3_2_14s() {
        use Taste::*;
        let book = [
            Some(Pungent),
            Some(Saline),
            Some(Bitter),
            Some(Mixed),
            Some(Sweet),
            Some(Sour),
            Some(Astringent),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::taste), book);
    }

    #[test]
    fn digbalas_are_section_3_2_15s() {
        use Direction::*;
        let book = [
            Some(South),
            Some(North),
            Some(South),
            Some(East),
            Some(East),
            Some(North),
            Some(West),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::digbala), book);
    }

    #[test]
    fn day_and_night_strength_is_section_3_2_15s() {
        use StrongBy::*;
        let book = [
            Some(Divaa),
            Some(Nishaa),
            Some(Nishaa),
            Some(Always),
            Some(Divaa),
            Some(Divaa),
            Some(Nishaa),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::strong_by), book);
    }

    #[test]
    fn paksha_and_ayana_strength_is_section_3_2_15s() {
        // Natural malefics in Krishna paksha and Dakshina ayana, benefics in Sukla and Uttara; Mercury's and the Moon's nature turns (§3.2.2); the seven planets only.
        use Ayana::*;
        use Paksha::*;
        let paksha = [
            Some(Krishna),
            None,
            Some(Krishna),
            None,
            Some(Sukla),
            Some(Sukla),
            Some(Krishna),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::strong_paksha), paksha);
        let ayana = [
            Some(Dakshina),
            None,
            Some(Dakshina),
            None,
            Some(Uttara),
            Some(Uttara),
            Some(Dakshina),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::strong_ayana), ayana);
    }

    #[test]
    fn ritus_are_section_3_2_16s_with_none_for_the_sun() {
        use Ritu::*;
        let book = [
            None,
            Some(Varsha),
            Some(Greeshma),
            Some(Hemanta),
            Some(Seeta),
            Some(Vasanta),
            Some(Sisira),
            None,
            None,
        ];
        assert_eq!(GRAHAS.map(Body::ritu), book);
    }

    #[test]
    fn dhatu_moola_and_jeeva_are_section_3_2_17s() {
        use Realm::*;
        let book = [
            Moola, Dhatu, Dhatu, Jeeva, Jeeva, Moola, Dhatu, Dhatu, Jeeva,
        ];
        assert_eq!(GRAHAS.map(Body::realm), book.map(Some));
    }

    #[test]
    fn the_lagna_has_no_graha_characteristics() {
        let l = Body::Lagna;
        assert_eq!(
            (
                l.avatara(),
                l.disposition(),
                l.governance(),
                l.colour(),
                l.cabinet(),
                l.deity()
            ),
            (None, None, None, None, None, None)
        );
        assert_eq!(
            (
                l.sex(),
                l.element(),
                l.class(),
                l.quality(),
                l.abode(),
                l.sapta_dhaatu()
            ),
            (None, None, None, None, None, None)
        );
        assert_eq!(
            (
                l.period(),
                l.taste(),
                l.digbala(),
                l.strong_by(),
                l.strong_paksha(),
                l.strong_ayana()
            ),
            (None, None, None, None, None, None)
        );
        assert_eq!((l.ritu(), l.realm()), (None, None));
    }

    #[test]
    fn dignities_are_table_6s() {
        use Rasi::*;
        // Table 6 (ch. 3 §3.3) row by row: own rasis, uchcha (deep exaltation), neecha (deep debilitation), moolatrikona.
        let table_6: [(&[Rasi], _, _, _); 9] = [
            (&[Leo], (Aries, Some(10)), (Libra, Some(10)), Leo),
            (&[Cancer], (Taurus, Some(3)), (Scorpio, Some(3)), Taurus),
            (
                &[Aries, Scorpio],
                (Capricorn, Some(28)),
                (Cancer, Some(28)),
                Aries,
            ),
            (
                &[Gemini, Virgo],
                (Virgo, Some(15)),
                (Pisces, Some(15)),
                Virgo,
            ),
            (
                &[Sagittarius, Pisces],
                (Cancer, Some(5)),
                (Capricorn, Some(5)),
                Sagittarius,
            ),
            (
                &[Taurus, Libra],
                (Pisces, Some(27)),
                (Virgo, Some(27)),
                Libra,
            ),
            (
                &[Capricorn, Aquarius],
                (Libra, Some(20)),
                (Aries, Some(20)),
                Aquarius,
            ),
            (&[Aquarius], (Gemini, None), (Sagittarius, None), Virgo),
            (&[Scorpio], (Sagittarius, None), (Gemini, None), Pisces),
        ];
        for (graha, (own, uchcha, neecha, moolatrikona)) in GRAHAS.into_iter().zip(table_6) {
            let d = graha.dignities().unwrap();
            assert_eq!(
                (d.own, d.uchcha, d.neecha, d.moolatrikona.0),
                (own, uchcha, neecha, moolatrikona),
                "{graha:?}"
            );
        }
        assert_eq!(Body::Lagna.dignities(), None);
    }

    #[test]
    fn every_rasis_lord_owns_it_in_table_6() {
        for r in RASIS {
            assert!(r.lord().dignities().unwrap().own.contains(&r), "{r:?}");
        }
        assert_eq!(Rasi::Scorpio.lord(), Body::Mars);
        assert_eq!(Rasi::Aquarius.lord(), Body::Saturn);
    }

    #[test]
    fn of_the_two_rasis_a_graha_owns_one_is_divaa_and_one_nishaa() {
        // Ch. 2 §2.2.10, left for chapter 3 (#115).
        for graha in GRAHAS {
            if let [a, b] = graha.dignities().unwrap().own {
                assert_ne!(a.strong_by(), b.strong_by(), "{graha:?}");
            }
        }
    }

    #[test]
    fn degree_splits_are_section_3_3s_at_their_boundaries() {
        use Body::*;
        use Dignity::*;
        // (graha, rasi, degree of the boundary, dignity just below it, dignity from it), §3.3 notes 1–7.
        for (graha, r, at, below, from) in [
            (Moon, 1, 3, Uchcha, Moolatrikona),
            (Mercury, 5, 15, Uchcha, Moolatrikona),
            (Mercury, 5, 20, Moolatrikona, OwnRasi),
            (Sun, 4, 20, Moolatrikona, OwnRasi),
            (Mars, 0, 12, Moolatrikona, OwnRasi),
            (Jupiter, 8, 10, Moolatrikona, OwnRasi),
            (Venus, 6, 15, Moolatrikona, OwnRasi),
            (Saturn, 10, 20, Moolatrikona, OwnRasi),
        ] {
            let at = dms(r, at, 0, 0.0);
            assert_eq!(
                dignity(graha, at - 1.0 / 60.0),
                Some(below),
                "{graha:?} {at}"
            );
            assert_eq!(dignity(graha, at), Some(from), "{graha:?} {at}");
            assert_eq!(
                dignity(graha, at + 1.0 / 60.0),
                Some(from),
                "{graha:?} {at}"
            );
        }
        // Each rasi's first and last arcsecond.
        assert_eq!(dignity(Moon, dms(1, 0, 0, 0.0)), Some(Uchcha));
        assert_eq!(dignity(Moon, dms(1, 2, 59, 59.0)), Some(Uchcha));
        assert_eq!(dignity(Moon, dms(1, 29, 59, 59.0)), Some(Moolatrikona));
        assert_eq!(dignity(Mercury, dms(5, 29, 59, 59.0)), Some(OwnRasi));
        assert_eq!(dignity(Mars, dms(0, 0, 0, 0.0)), Some(Moolatrikona));
        assert_eq!(dignity(Mars, dms(7, 0, 0, 0.0)), Some(OwnRasi));
        assert_eq!(dignity(Saturn, dms(0, 22, 41, 0.0)), Some(Neecha));
        assert_eq!(dignity(Saturn, dms(5, 0, 0, 0.0)), None);
        assert_eq!(dignity(Rahu, dms(5, 29, 59, 59.0)), Some(Moolatrikona));
        assert_eq!(dignity(Ketu, dms(2, 0, 0, 0.0)), Some(Neecha));
        assert_eq!(dignity(Lagna, 0.0), None);
    }

    #[test]
    fn natural_relationships_are_table_7s() {
        use Body::*;
        // Table 7 (ch. 3 §3.4.1): each planet's friends, neutrals, enemies.
        let table_7 = [
            (
                Sun,
                vec![Moon, Mars, Jupiter],
                vec![Mercury],
                vec![Venus, Saturn],
            ),
            (
                Moon,
                vec![Sun, Mercury],
                vec![Mars, Jupiter, Venus, Saturn],
                vec![],
            ),
            (
                Mars,
                vec![Sun, Moon, Jupiter],
                vec![Venus, Saturn],
                vec![Mercury],
            ),
            (
                Mercury,
                vec![Sun, Venus],
                vec![Mars, Jupiter, Saturn],
                vec![Moon],
            ),
            (
                Jupiter,
                vec![Sun, Moon, Mars],
                vec![Saturn],
                vec![Mercury, Venus],
            ),
            (
                Venus,
                vec![Mercury, Saturn],
                vec![Mars, Jupiter],
                vec![Sun, Moon],
            ),
            (
                Saturn,
                vec![Mercury, Venus],
                vec![Jupiter],
                vec![Sun, Moon, Mars],
            ),
        ];
        for (graha, friends, neutrals, enemies) in table_7 {
            let with = |value| -> Vec<Body> {
                GRAHAS
                    .into_iter()
                    .filter(|&other| natural(graha, other) == Some(value))
                    .collect()
            };
            assert_eq!(with(Natural::Mitra), friends, "{graha:?}");
            assert_eq!(with(Natural::Sama), neutrals, "{graha:?}");
            assert_eq!(with(Natural::Satru), enemies, "{graha:?}");
        }
        assert_eq!(natural(Sun, Sun), None);
        assert_eq!(natural(Sun, Rahu), None);
        assert_eq!(natural(Ketu, Moon), None);
        assert_eq!(natural(Lagna, Moon), None);
    }

    #[test]
    fn tatkaala_friends_are_in_the_2nd_3rd_4th_10th_11th_and_12th() {
        let friends: Vec<u8> = RASIS
            .into_iter()
            .filter(|&r| tatkaala(Rasi::Cancer, r) == Tatkaala::Mitra)
            .map(|r| bhava(Rasi::Cancer, r))
            .collect();
        assert_eq!(friends, [10, 11, 12, 2, 3, 4]);
    }

    #[test]
    fn compound_relationships_are_table_8s() {
        use Compound::*;
        // Table 8 (ch. 3 §3.4.3): rows natural friend, neutral, enemy; columns temporary friend, enemy.
        let table_8 = [[Adhimitra, Sama], [Mitra, Satru], [Sama, Adhisatru]];
        let got = [Natural::Mitra, Natural::Sama, Natural::Satru]
            .map(|n| [Tatkaala::Mitra, Tatkaala::Satru].map(|t| compound(n, t)));
        assert_eq!(got, table_8);
    }

    #[test]
    fn saturn_in_aries_is_neecha_and_sama_with_mars_in_taurus() {
        // Mars in Taurus is Saturn's tatkaala friend, a natural enemy: sama (Table 8); neecha all the same (§3.3).
        let mut g = [[dms(4, 1, 0, 0.0), 1.0]; 8];
        g[2] = [dms(1, 5, 0, 0.0), 1.0];
        g[6] = [dms(0, 22, 41, 0.0), 1.0];
        let chart = chart(&sky(dms(4, 0, 0, 0.0), g));
        assert_eq!(
            dignity(Body::Saturn, chart[7].longitude),
            Some(Dignity::Neecha)
        );
        assert_eq!(
            lord_relationship(&chart, Body::Saturn),
            Some(Compound::Sama)
        );
        // The rest are in Leo: the Sun in his own rasi has no relationship, the others have the Sun's.
        assert_eq!(lord_relationship(&chart, Body::Sun), None);
        assert_eq!(lord_relationship(&chart, Body::Moon), Some(Compound::Sama));
        assert_eq!(lord_relationship(&chart, Body::Lagna), None);
        assert_eq!(lord_relationship(&chart, Body::Rahu), None);
        assert_eq!(lord_relationship(&chart, Body::Ketu), None);
        // Nor in the nodes' own rasis, whose lords are Saturn and Mars; Ketu opposite Rahu, so one at a time.
        for (rahu, node, at) in [(10, Body::Rahu, 8), (1, Body::Ketu, 9)] {
            let mut g = [[dms(4, 1, 0, 0.0), 1.0]; 8];
            g[7] = [dms(rahu, 5, 0, 0.0), -0.05];
            let chart = super::chart(&sky(0.0, g));
            assert_eq!(dignity(node, chart[at].longitude), Some(Dignity::OwnRasi));
            assert_eq!(lord_relationship(&chart, node), None);
        }
    }

    /// The rasi and whole arcminutes within it, as the book prints them.
    fn rasi_dm(longitude: f64) -> (Rasi, u32) {
        let (rasi, degrees) = rasi(longitude);
        (rasi, arcseconds(degrees) / 60)
    }

    #[test]
    fn example_6_sun_9_36_sg_has_dhuma_22_56_ar() {
        // Ch. 4 §4.2, Example 6; the book works out four of the five.
        use Rasi::*;
        use Upagraha::*;
        let got = sun_based(dms(8, 9, 36, 0.0)).map(|(u, l)| (u, rasi_dm(l)));
        for (upagraha, at) in [
            (Dhuma, (Aries, 22 * 60 + 56)),
            (Vyatipaata, (Pisces, 7 * 60 + 4)),
            (Parivesha, (Virgo, 7 * 60 + 4)),
            (Indrachaapa, (Libra, 22 * 60 + 56)),
        ] {
            assert!(got.contains(&(upagraha, at)), "{upagraha:?}: {got:?}");
        }
    }

    #[test]
    fn exercise_7_sun_13_19_ta_has_dhuma_26_39_vi() {
        // Ch. 4 §4.4 prints Upaketu at 19°1′ Sc, a misprint: Table 9's Sun − 30° is 13°19′ Ar (#134).
        use Rasi::*;
        use Upagraha::*;
        let got = sun_based(dms(1, 13, 19, 0.0)).map(|(u, l)| (u, rasi_dm(l)));
        assert_eq!(
            got,
            [
                (Dhuma, (Virgo, 26 * 60 + 39)),
                (Vyatipaata, (Libra, 3 * 60 + 21)),
                (Parivesha, (Aries, 3 * 60 + 21)),
                (Indrachaapa, (Pisces, 26 * 60 + 39)),
                (Upaketu, (Aries, 13 * 60 + 19)),
            ]
        );
    }

    #[test]
    fn upaketu_is_the_sun_less_30_and_the_pairs_are_opposite() {
        // Ch. 4 §4.2: Table 9's Upaketu row, and the note under it.
        let apart = |a: f64, b: f64| (a - b).rem_euclid(360.0);
        let close = |a: f64, b: f64| {
            let d = apart(a, b);
            d.min(360.0 - d) < 1e-9
        };
        for tenth in 0..3600 {
            let sun = f64::from(tenth) / 10.0;
            let [dhuma, vyatipaata, parivesha, indrachaapa, upaketu] =
                sun_based(sun).map(|(_, l)| {
                    assert!((0.0..360.0).contains(&l), "{sun}: {l}");
                    l
                });
            assert!(close(upaketu, sun - 30.0), "{sun}: {upaketu}");
            assert!(close(apart(dhuma, indrachaapa), 180.0), "{sun}");
            assert!(close(apart(vyatipaata, parivesha), 180.0), "{sun}");
        }
    }

    #[test]
    fn part_rulers_are_table_10s() {
        // Ch. 4 §4.3, Table 10: the day's rows, then the night's, Sunday to Saturday; `—` is lord-less.
        use Body::*;
        let (su, mo, ma, me, ju, ve, sa) = (
            Some(Sun),
            Some(Moon),
            Some(Mars),
            Some(Mercury),
            Some(Jupiter),
            Some(Venus),
            Some(Saturn),
        );
        let day = [
            [su, mo, ma, me, ju, ve, sa, None],
            [mo, ma, me, ju, ve, sa, None, su],
            [ma, me, ju, ve, sa, None, su, mo],
            [me, ju, ve, sa, None, su, mo, ma],
            [ju, ve, sa, None, su, mo, ma, me],
            [ve, sa, None, su, mo, ma, me, ju],
            [sa, None, su, mo, ma, me, ju, ve],
        ];
        let night = [
            [ju, ve, sa, None, su, mo, ma, me],
            [ve, sa, None, su, mo, ma, me, ju],
            [sa, None, su, mo, ma, me, ju, ve],
            [su, mo, ma, me, ju, ve, sa, None],
            [mo, ma, me, ju, ve, sa, None, su],
            [ma, me, ju, ve, sa, None, su, mo],
            [me, ju, ve, sa, None, su, mo, ma],
        ];
        assert_eq!(VAARAS.map(|v| part_rulers(v, false)), day);
        assert_eq!(VAARAS.map(|v| part_rulers(v, true)), night);
    }

    /// A Sky on `vaara`'s Vedic day, born in its day or `night`, whose part lagnas are `lagna(part, middle)`.
    fn timed_sky(vaara: Vaara, night: bool, lagna: impl Fn(usize, bool) -> f64) -> Sky {
        // 06:00 on 2000-01-02, a Sunday, then on to `vaara`.
        let sunrise = WEDNESDAY - 3.0 + hours(6, 0) + vaara as usize as f64;
        let day = day(sunrise, sunrise + 1.0);
        Sky {
            jd: if night { day.sunset } else { day.sunrise },
            vedic_day: Some(day),
            part_lagnas: Some(std::array::from_fn(|k| [lagna(k, false), lagna(k, true)])),
            ..sky(0.0, [[0.0, 1.0]; 8])
        }
    }

    /// The time-based upagrahas of `sky`, in the book's order.
    fn time_based_of(sky: &Sky) -> Vec<(Upagraha, f64)> {
        upagrahas(sky).split_off(5)
    }

    #[test]
    fn a_thursday_night_from_18_00_to_06_00_has_yamaghantaka_at_23_15() {
        // Ch. 4 §4.3's worked case: Jupiter rules the 4th part, 22:30 to 24:00; a part lagna here is its moment's hour.
        let sky = timed_sky(Vaara::Thursday, true, |k, middle| {
            18.0 + 1.5 * (k as f64 + if middle { 0.5 } else { 0.0 })
        });
        assert_eq!(part_rulers(Vaara::Thursday, true)[3], Some(Body::Jupiter));
        assert!(time_based_of(&sky).contains(&(Upagraha::Yamaghantaka, 23.25)));
    }

    #[test]
    fn each_time_based_upagraha_rises_at_the_start_or_middle_of_its_part() {
        // A Thursday's day: Jup, Ven, Sat, —, Sun, Moon, Mars, Merc; a part lagna here is 10 × its part, + 5 at its middle.
        let sky = timed_sky(Vaara::Thursday, false, |k, middle| {
            10.0 * (k + 1) as f64 + if middle { 5.0 } else { 0.0 }
        });
        use Upagraha::*;
        assert_eq!(
            time_based_of(&sky),
            [
                (Kaala, 55.0),
                (Mrityu, 75.0),
                (Arthaprahaara, 85.0),
                (Yamaghantaka, 15.0),
                (Gulika, 30.0),
                (Maandi, 35.0),
            ]
        );
    }

    #[test]
    fn a_sky_without_a_vedic_day_has_only_the_sun_based_upagrahas() {
        let sky = sky(0.0, [[0.0, 1.0]; 8]);
        assert_eq!(upagrahas(&sky), sun_based(0.0));
    }
}

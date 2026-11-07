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
    in_varga(Varga::RasiChart, chart, body)
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

    /// Whether it is one of the five found from the Sun's longitude, not from a part of the day or night (ch. 4 §4.2).
    pub fn is_sun_based(self) -> bool {
        use Upagraha::*;
        matches!(self, Dhuma | Vyatipaata | Parivesha | Indrachaapa | Upaketu)
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

/// Ch. 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialLagna {
    HoraLagna,
    GhatiLagna,
    SreeLagna,
    BhaavaLagna,
}

/// The special lagnas by how much the book uses them: HL and GL widely (ch. 5 §5.1), SL (§5.7), BL not at all (footnote 10).
pub const SPECIAL_LAGNAS: [SpecialLagna; 4] = [
    SpecialLagna::HoraLagna,
    SpecialLagna::GhatiLagna,
    SpecialLagna::SreeLagna,
    SpecialLagna::BhaavaLagna,
];

/// The bhaava lagna `hours` after sunrise, from the Sun's `sun_at_sunrise` at 15° an hour (ch. 5 §5.2).
pub fn bhaava_lagna(sun_at_sunrise: f64, hours: f64) -> f64 {
    reduce(sun_at_sunrise + 15.0 * hours)
}

/// The hora lagna `hours` after sunrise, from the Sun's `sun_at_sunrise` at 30° an hour (ch. 5 §5.3).
pub fn hora_lagna(sun_at_sunrise: f64, hours: f64) -> f64 {
    reduce(sun_at_sunrise + 30.0 * hours)
}

/// The ghati lagna `hours` after sunrise, from the Sun's `sun_at_sunrise` at 30° per 24 minutes (ch. 5 §5.4).
pub fn ghati_lagna(sun_at_sunrise: f64, hours: f64) -> f64 {
    reduce(sun_at_sunrise + 30.0 * 60.0 / 24.0 * hours)
}

/// The sree lagna: `lagna` plus 360° × the fraction of its nakshatra `moon` has covered (ch. 5 §5.7).
pub fn sree_lagna(lagna: f64, moon: f64) -> f64 {
    // 360° per 13°20′ is 27×, and 27 × a nakshatra's start is a whole circle, so the start drops out.
    reduce(lagna + 27.0 * moon)
}

/// The special lagnas of `sky` with their longitudes: the hora, ghati and bhaava lagnas count the exact time from the
/// Vedic day's sunrise, so where the Sky has none only the sree lagna is there (ch. 5, ADR 0003).
pub fn special_lagnas(sky: &Sky) -> Vec<(SpecialLagna, f64)> {
    let sree = (
        SpecialLagna::SreeLagna,
        sree_lagna(sky.lagna, sky.grahas.moon.longitude),
    );
    let Some(day) = &sky.vedic_day else {
        return vec![sree];
    };
    let (sun, hours) = (day.sun_at_sunrise, (sky.jd - day.sunrise) * 24.0);
    vec![
        (SpecialLagna::HoraLagna, hora_lagna(sun, hours)),
        (SpecialLagna::GhatiLagna, ghati_lagna(sun, hours)),
        sree,
        (SpecialLagna::BhaavaLagna, bhaava_lagna(sun, hours)),
    ]
}

/// The twenty varga chakras of ch. 6 §6.2, in its order; named in the glossary's spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Varga {
    RasiChart = 1,
    Hora = 2,
    Drekkana = 3,
    Chaturthamsa = 4,
    Panchamsa = 5,
    Shashthamsa = 6,
    Saptamsa = 7,
    Ashtamsa = 8,
    Navamsa = 9,
    Dasamsa = 10,
    Rudramsa = 11,
    Dwadasamsa = 12,
    Shodasamsa = 16,
    Vimsamsa = 20,
    Chaturvimsamsa = 24,
    Nakshatramsa = 27,
    Trimsamsa = 30,
    Khavedamsa = 40,
    Akshavedamsa = 45,
    Shashtyamsa = 60,
}

/// The twenty varga chakras in the book's order, D-1 to D-60 (ch. 6 §6.2).
pub const VARGAS: [Varga; 20] = [
    Varga::RasiChart,
    Varga::Hora,
    Varga::Drekkana,
    Varga::Chaturthamsa,
    Varga::Panchamsa,
    Varga::Shashthamsa,
    Varga::Saptamsa,
    Varga::Ashtamsa,
    Varga::Navamsa,
    Varga::Dasamsa,
    Varga::Rudramsa,
    Varga::Dwadasamsa,
    Varga::Shodasamsa,
    Varga::Vimsamsa,
    Varga::Chaturvimsamsa,
    Varga::Nakshatramsa,
    Varga::Trimsamsa,
    Varga::Khavedamsa,
    Varga::Akshavedamsa,
    Varga::Shashtyamsa,
];

impl Varga {
    /// The n of D-n: the parts each rasi is divided into (ch. 6 §6.1).
    pub fn parts(self) -> u8 {
        self as u8
    }
}

/// The kth rasi from `from`, counted zodiacally: the 13th is the 1st again (§6.2.13).
fn kth(from: Rasi, k: usize) -> Rasi {
    RASIS[(from as usize + k - 1) % 12]
}

/// The rasi `longitude` falls in in `varga`, by the part of its rasi it is in (ch. 6 §6.2, ADR 0009).
pub fn varga(varga: Varga, longitude: f64) -> Rasi {
    use Rasi::*;
    use Varga::*;
    let (r, degrees) = rasi(longitude);
    // Whole arcseconds, as for dignities: 9°59′59.9″ is still the 1st drekkana, 10°00′00″ the 2nd.
    let at = arcseconds(degrees);
    let part = (at * u32::from(varga.parts()) / (30 * 3600)) as usize;
    let odd = r.parity() == Parity::Vishama;
    let by_parity = |odd_rasi, even_rasi| if odd { odd_rasi } else { even_rasi };
    let by_mobility = |[chara, sthira, dwiswabhava]: [Rasi; 3]| match r.mobility() {
        Mobility::Chara => chara,
        Mobility::Sthira => sthira,
        Mobility::Dwiswabhava => dwiswabhava,
    };
    let by_element = |[agni, bhoo, vaayu, jala]: [Rasi; 4]| match r.element() {
        Element::Agni => agni,
        Element::Bhoo => bhoo,
        Element::Vaayu => vaayu,
        // Aakaasa is no rasi's element of its own (ch. 2 §2.2.5).
        Element::Jala | Element::Aakaasa => jala,
    };
    let (start, step) = match varga {
        RasiChart | Dwadasamsa | Shashtyamsa => (r, 1),
        // The Sun's hora is Leo, the Moon's Cancer (Example 27, ADR 0003).
        Hora => return if odd { [Leo, Cancer] } else { [Cancer, Leo] }[part],
        // The 1st, 5th and 9th from the rasi.
        Drekkana => (r, 4),
        // The 1st, 4th, 7th and 10th from the rasi.
        Chaturthamsa => (r, 3),
        Panchamsa => {
            return if odd {
                [Aries, Aquarius, Sagittarius, Gemini, Libra]
            } else {
                [Taurus, Virgo, Pisces, Capricorn, Scorpio]
            }[part];
        }
        Shashthamsa | Khavedamsa => (by_parity(Aries, Libra), 1),
        Saptamsa => (by_parity(r, kth(r, 7)), 1),
        Ashtamsa | Vimsamsa => (by_mobility([Aries, Sagittarius, Leo]), 1),
        Navamsa => (by_element([Aries, Capricorn, Libra, Cancer]), 1),
        Dasamsa => (by_parity(r, kth(r, 9)), 1),
        // As many rasis back from Aries as the rasi is from it (§6.2.11, footnote 11).
        Rudramsa => (RASIS[(12 - r as usize) % 12], 1),
        Shodasamsa | Akshavedamsa => (by_mobility([Aries, Leo, Sagittarius]), 1),
        Chaturvimsamsa => (by_parity(Leo, Cancer), 1),
        Nakshatramsa => (by_element([Aries, Cancer, Libra, Capricorn]), 1),
        // Unequal parts: each rasi up to the degree mark it is below (§6.2.17).
        Trimsamsa => {
            let marks = if odd {
                [
                    (5, Aries),
                    (10, Aquarius),
                    (18, Sagittarius),
                    (25, Gemini),
                    (30, Libra),
                ]
            } else {
                [
                    (5, Taurus),
                    (12, Virgo),
                    (20, Pisces),
                    (25, Capricorn),
                    (30, Scorpio),
                ]
            };
            // The last mark is 30°, beyond every `at`.
            return marks
                .into_iter()
                .find(|&(mark, _)| at < mark * 3600)
                .unwrap()
                .1;
        }
    };
    kth(start, part * step + 1)
}

/// The four groups of varga chakras amsabala is counted over (ch. 6 §6.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VargaGroup {
    Shadvarga,
    SaptaVarga,
    DasaVarga,
    ShodasaVarga,
}

/// The varga groups in the book's order (ch. 6 §6.6.1–6.6.4).
pub const VARGA_GROUPS: [VargaGroup; 4] = [
    VargaGroup::Shadvarga,
    VargaGroup::SaptaVarga,
    VargaGroup::DasaVarga,
    VargaGroup::ShodasaVarga,
];

/// The amsas of §6.6, named for a graha's count of good varga chakras in a group, in the book's spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Amsa {
    Kimsukaamsa,
    Vyanjanaamsa,
    Chaamaraamsa,
    Chatraamsa,
    Kundalaamsa,
    Mukutaamsa,
    Paarijaataamsa,
    Uttamaamsa,
    Gopuraamsa,
    Simhaasanaamsa,
    Paaraavataamsa,
    Devalokaamsa,
    Brahmalokamsa,
    Airaavataamsa,
    Sreedhaamaamsa,
    Bhedakaamsa,
    Kusumaamsa,
    Nagapurushaamsa,
    Kandukaamsa,
    Keralaamsa,
    Kalpavrikshaamsa,
    Chandanavanaamsa,
    Poornachandraamsa,
    Uchchaisravaamsa,
    Dhanvantaryamsa,
    Sooryakaantaamsa,
    Vidrumaamsa,
    Indraasanaamsa,
    Golokaamsa,
    SreeVallabhaamsa,
}

impl VargaGroup {
    /// The group's varga chakras (§6.6.1–6.6.4).
    pub fn vargas(self) -> &'static [Varga] {
        use Varga::*;
        match self {
            VargaGroup::Shadvarga => &[RasiChart, Hora, Drekkana, Navamsa, Dwadasamsa, Trimsamsa],
            VargaGroup::SaptaVarga => &[
                RasiChart, Hora, Drekkana, Saptamsa, Navamsa, Dwadasamsa, Trimsamsa,
            ],
            VargaGroup::DasaVarga => &[
                RasiChart,
                Hora,
                Drekkana,
                Saptamsa,
                Navamsa,
                Dasamsa,
                Dwadasamsa,
                Shodasamsa,
                Trimsamsa,
                Shashtyamsa,
            ],
            VargaGroup::ShodasaVarga => &[
                RasiChart,
                Hora,
                Drekkana,
                Chaturthamsa,
                Saptamsa,
                Navamsa,
                Dasamsa,
                Dwadasamsa,
                Shodasamsa,
                Vimsamsa,
                Chaturvimsamsa,
                Nakshatramsa,
                Trimsamsa,
                Khavedamsa,
                Akshavedamsa,
                Shashtyamsa,
            ],
        }
    }

    /// The group's amsas, for a count of 2 up to all its vargas (§6.6.1–6.6.4).
    pub fn amsas(self) -> &'static [Amsa] {
        use Amsa::*;
        const SAPTA: [Amsa; 6] = [
            Kimsukaamsa,
            Vyanjanaamsa,
            Chaamaraamsa,
            Chatraamsa,
            Kundalaamsa,
            Mukutaamsa,
        ];
        match self {
            // Shadvarga's six vargas stop one short of sapta varga's mukutaamsa.
            VargaGroup::Shadvarga => &SAPTA[..5],
            VargaGroup::SaptaVarga => &SAPTA,
            VargaGroup::DasaVarga => &[
                Paarijaataamsa,
                Uttamaamsa,
                Gopuraamsa,
                Simhaasanaamsa,
                Paaraavataamsa,
                Devalokaamsa,
                Brahmalokamsa,
                Airaavataamsa,
                Sreedhaamaamsa,
            ],
            VargaGroup::ShodasaVarga => &[
                Bhedakaamsa,
                Kusumaamsa,
                Nagapurushaamsa,
                Kandukaamsa,
                Keralaamsa,
                Kalpavrikshaamsa,
                Chandanavanaamsa,
                Poornachandraamsa,
                Uchchaisravaamsa,
                Dhanvantaryamsa,
                Sooryakaantaamsa,
                Vidrumaamsa,
                Indraasanaamsa,
                Golokaamsa,
                SreeVallabhaamsa,
            ],
        }
    }

    /// The amsa named for `count` good vargas in the group; none below 2.
    pub fn amsa(self, count: u8) -> Option<Amsa> {
        self.amsas()
            .get(usize::from(count.checked_sub(2)?))
            .copied()
    }
}

/// How many of `group`'s vargas put `graha` in its uchcha, moolatrikona or own rasi, and the amsa named for it
/// (ch. 6 §6.6). By whole rasi: Example 27 counts Jupiter at 29°49′ Sg, past §3.3's moolatrikona degrees. None for
/// the lagna.
pub fn amsabala(graha: Body, longitude: f64, group: VargaGroup) -> (u8, Option<Amsa>) {
    let Some(d) = graha.dignities() else {
        return (0, None);
    };
    let count = group
        .vargas()
        .iter()
        .map(|&v| varga(v, longitude))
        .filter(|r| *r == d.uchcha.0 || *r == d.moolatrikona.0 || d.own.contains(r))
        .count() as u8;
    (count, group.amsa(count))
}

/// Paaka lagna: the rasi the lagna's lord occupies, both read in `d`'s own chart, since it "applies to all
/// divisional charts" (ch. 7 §7.3.5, ADR 0003).
pub fn paaka_lagna(d: Varga, chart: &[Entry; 10]) -> Rasi {
    let in_d = |body| in_varga(d, chart, body);
    // shortcut: Scorpio takes Mars and Aquarius Saturn, not the stronger co-lord; revisit with ch. 15's decision (ADR 0003).
    in_d(in_d(Body::Lagna).lord())
}

/// The rasi holding `body` in `d`'s chart.
fn in_varga(d: Varga, chart: &[Entry; 10], body: Body) -> Rasi {
    // chart() holds every Body.
    varga(d, chart.iter().find(|e| e.body == body).unwrap().longitude)
}

/// Whether rasi `a` aspects rasi `b`, which is then also so the other way: movable and fixed rasis aspect each other
/// except the adjacent ones, dual rasis the other dual rasis (ch. 10 §10.3).
pub fn rasi_drishti(a: Rasi, b: Rasi) -> bool {
    use Mobility::*;
    match (a.mobility(), b.mobility()) {
        (Chara, Sthira) | (Sthira, Chara) => !matches!(bhava(a, b), 2 | 12),
        (Dwiswabhava, Dwiswabhava) => a != b,
        _ => false,
    }
}

/// The rasis `graha` in `at` aspects with graha drishti, in count order from `at` (ch. 10 §10.2). The nodes and a lagna have none,
/// as in Exercise 14 (ADR 0013).
pub fn graha_drishti(graha: Body, at: Rasi) -> Vec<Rasi> {
    let houses: &[usize] = match graha {
        Body::Sun | Body::Moon | Body::Mercury | Body::Venus => &[7],
        Body::Mars => &[4, 7, 8],
        Body::Jupiter => &[5, 7, 9],
        Body::Saturn => &[3, 7, 10],
        Body::Rahu | Body::Ketu | Body::Lagna => &[],
    };
    houses.iter().map(|&k| kth(at, k)).collect()
}

/// The rasis `at` aspects with rasi drishti, in count order from `at` (ch. 10 §10.3).
pub fn rasi_drishtis(at: Rasi) -> Vec<Rasi> {
    (2..=12)
        .map(|k| kth(at, k))
        .filter(|&r| rasi_drishti(at, r))
        .collect()
}

/// The nine grahas in `rasi` in `d`'s chart, in the book's order; a lagna is never aspected as a graha (ADR 0013).
pub fn grahas_in(d: Varga, chart: &[Entry; 10], rasi: Rasi) -> Vec<Body> {
    GRAHAS
        .into_iter()
        .filter(|&g| in_varga(d, chart, g) == rasi)
        .collect()
}

/// One argala on a rasi and what obstructs it. An empty `virodhargala` is the "unobstructed" the book speaks of; there is
/// no verdict (ADR 0013).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argala {
    /// The count the argala comes from: 2, 4, 5 or 11.
    pub from: u8,
    /// The count its obstruction comes from: 12, 10, 9 or 3.
    pub obstructed_from: u8,
    pub argala: Vec<Body>,
    pub virodhargala: Vec<Body>,
}

/// Whether `rasi`'s argala counts run anti-zodiacally in `d`'s chart: it holds Ketu there (ch. 10 §10.6's note).
pub fn argala_reversed(d: Varga, chart: &[Entry; 10], rasi: Rasi) -> bool {
    in_varga(d, chart, Body::Ketu) == rasi
}

/// The argalas on `rasi` in `d`'s chart, in the order 2nd/12th, 4th/10th, 5th/9th, 11th/3rd (ch. 10 §10.5–10.6), counted as
/// [`argala_reversed`] says. Several malefics in the 3rd stay virodhargala (ADR 0013).
pub fn argala(d: Varga, chart: &[Entry; 10], rasi: Rasi) -> [Argala; 4] {
    let reversed = argala_reversed(d, chart, rasi);
    let held = |count: u8| {
        // The k-th counted in reverse is the (14 − k)th counted forward.
        let count = if reversed { 14 - count } else { count };
        grahas_in(d, chart, kth(rasi, count.into()))
    };
    [(2, 12), (4, 10), (5, 9), (11, 3)].map(|(from, obstructed_from)| Argala {
        from,
        obstructed_from,
        argala: held(from),
        virodhargala: held(obstructed_from),
    })
}

/// `rasi`'s owners in Table 6, so Mars and Ketu for Scorpio, Saturn and Rahu for Aquarius (ch. 15 §15.5.1).
fn lords(rasi: Rasi) -> impl Iterator<Item = Body> {
    GRAHAS
        .into_iter()
        .filter(move |g| g.dignities().is_some_and(|d| d.own.contains(&rasi)))
}

/// How many of Jupiter, Mercury and `lords` occupy or aspect `rasi` in `d`'s chart. These are roles, not grahas, so
/// Mercury as a lord counts twice (ch. 15 §15.5.1 and §15.5.2, rule 2; ADR 0012).
fn roles_on(d: Varga, chart: &[Entry; 10], rasi: Rasi, lords: impl Iterator<Item = Body>) -> usize {
    [Body::Jupiter, Body::Mercury]
        .into_iter()
        .chain(lords)
        .map(|g| in_varga(d, chart, g))
        .filter(|&at| at == rasi || rasi_drishti(at, rasi))
        .count()
}

/// Exalted by whole rasi, since a varga has no degrees (ADR 0012).
fn exalted(graha: Body, at: Rasi) -> bool {
    graha.dignities().is_some_and(|d| d.uchcha.0 == at)
}

/// `graha`'s advancement in its rasi in the rasi chart, whichever chart is being read, in whole arcseconds; the nodes'
/// measured back from the rasi's end (ch. 15 §15.5.1 rule 5b, ch. 8 §8.2, ADR 0011, ADR 0012).
fn advancement(chart: &[Entry; 10], graha: Body) -> u32 {
    // chart() holds every Body.
    let entry = chart.iter().find(|e| e.body == graha).unwrap();
    let at = arcseconds(rasi(entry.longitude).1);
    if matches!(graha, Body::Rahu | Body::Ketu) {
        30 * 3600 - at
    } else {
        at
    }
}

/// `rasi`'s lord in `d`'s chart: its only one, or the stronger of Scorpio's or Aquarius's co-lords by §15.5.1's basic rule,
/// then rules 1, 2, 3, 4 and 5b (ch. 15 §15.5.1, ADR 0012). Rule 5a, for dasa lengths, waits for the dasas.
pub fn stronger_lord(d: Varga, chart: &[Entry; 10], rasi: Rasi) -> Body {
    let lord = rasi.lord();
    let Some(co_lord) = lords(rasi).find(|&g| g != lord) else {
        return lord;
    };
    let in_d = |body| in_varga(d, chart, body);
    match (in_d(lord) == rasi, in_d(co_lord) == rasi) {
        (true, false) => return co_lord,
        (false, true) => return lord,
        _ => {}
    }
    // Compared in order, so a rule is reached only on a tie in those before it.
    let strength = |graha: Body| {
        let at = in_d(graha);
        (
            GRAHAS
                .into_iter()
                .filter(|&g| g != graha && in_d(g) == at)
                .count(),
            // No co-lord joins itself as its own dispositor (ADR 0012).
            roles_on(d, chart, at, lords(at).filter(|&g| g != graha)),
            exalted(graha, at),
            // Mobility runs from movable to dual, weakest to strongest.
            at.mobility() as u8,
            advancement(chart, graha),
        )
    };
    // shortcut: the book has no rule for a tie after 5b, so the seven-planet lord; upgrade if a later chapter gives one.
    if strength(co_lord) > strength(lord) {
        co_lord
    } else {
        lord
    }
}

/// The stronger of two rasis in `d`'s chart, by §15.5.2's rules 1 to 6 in order, which also run for a graha's own two rasis
/// (ch. 15 §15.5.2, ADR 0012). Not for ayur dasas, where §15.5.2's warning drops the lord's aspect: chart 44's Aq beats Le (p. 285), but this gives Le.
pub fn stronger_rasi(d: Varga, chart: &[Entry; 10], a: Rasi, b: Rasi) -> Rasi {
    let in_d = |body| in_varga(d, chart, body);
    // Compared in order, so a rule is reached only on a tie in those before it.
    let strength = |r: Rasi| {
        let occupants = || GRAHAS.into_iter().filter(|&g| in_d(g) == r);
        let stronger = stronger_lord(d, chart, r);
        (
            occupants().count(),
            // Both co-lords, not the stronger lord alone (Exercise 26(5), ADR 0012).
            roles_on(d, chart, r, lords(r)),
            occupants().any(|g| exalted(g, r)),
            in_d(stronger).parity() != r.parity(),
            // Mobility runs from movable to dual, weakest to strongest.
            r.mobility() as u8,
            advancement(chart, stronger),
        )
    };
    // shortcut: the book has no rule for a tie after rule 6, so the first rasi from Aries; upgrade if a later chapter gives one.
    let (first, second) = if (a as u8) < (b as u8) {
        (a, b)
    } else {
        (b, a)
    };
    if strength(second) > strength(first) {
        second
    } else {
        first
    }
}

/// The arudha pada counted from `start` through `reached`, always zodiacally: as many rasis on as `reached` is from `start`,
/// but if that lands 1st or 7th from `start`, the 10th from where it landed (ch. 9 §9.2 and §9.5, steps 3–5).
fn arudha_pada(start: Rasi, reached: Rasi) -> Rasi {
    let landed = kth(reached, bhava(start, reached) as usize);
    if matches!(bhava(start, landed), 1 | 7) {
        kth(landed, 10)
    } else {
        landed
    }
}

/// The bhava arudhas of `d`'s chart, from AL (the 1st bhava) to UL (the 12th), counted from that chart's lagna, each bhava's
/// lord being its stronger lord (ch. 9 §9.2, ADR 0012).
pub fn bhava_arudhas(d: Varga, chart: &[Entry; 10]) -> [Rasi; 12] {
    let lagna = in_varga(d, chart, Body::Lagna);
    std::array::from_fn(|i| {
        let house = kth(lagna, i + 1);
        arudha_pada(house, in_varga(d, chart, stronger_lord(d, chart, house)))
    })
}

/// The graha arudhas of `d`'s chart, the Sun to Ketu, each counted through the rasi its graha owns, or the stronger of its two
/// (ch. 9 §9.5, ADR 0012).
pub fn graha_arudhas(d: Varga, chart: &[Entry; 10]) -> [(Body, Rasi); 9] {
    GRAHAS.map(|graha| {
        // Every graha has dignities and owns a rasi (Table 6).
        let rasis = graha.dignities().unwrap().own.iter().copied();
        let owned = rasis.reduce(|a, b| stronger_rasi(d, chart, a, b)).unwrap();
        (graha, arudha_pada(in_varga(d, chart, graha), owned))
    })
}

/// The eight chara karakas, from the most advanced graha to the least (ch. 8 §8.2, Table 13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharaKaraka {
    Atma,
    Amatya,
    Bhratri,
    Matri,
    Pitri,
    Putra,
    Jnaati,
    Dara,
}

/// The chara karakas in Table 13's order.
pub const CHARA_KARAKAS: [CharaKaraka; 8] = [
    CharaKaraka::Atma,
    CharaKaraka::Amatya,
    CharaKaraka::Bhratri,
    CharaKaraka::Matri,
    CharaKaraka::Pitri,
    CharaKaraka::Putra,
    CharaKaraka::Jnaati,
    CharaKaraka::Dara,
];

/// The chara karaka of the Sun to Saturn and Rahu, in the book's graha order, ranked by advancement in their rasis
/// in whole arcseconds, Rahu's measured back from his rasi's end (ch. 8 §8.2, ADR 0011). From the rasi chart only;
/// Ketu and the lagna have none.
pub fn chara_karakas(chart: &[Entry; 10]) -> [(Body, CharaKaraka); 8] {
    // chart() holds the lagna first, then Sun to Rahu, then Ketu.
    let grahas = &chart[1..9];
    let advanced: [u32; 8] = std::array::from_fn(|i| advancement(chart, grahas[i].body));
    // Ranking by how many are strictly ahead: grahas tied share a karakatwa and the next is left empty (ADR 0011).
    std::array::from_fn(|i| {
        let ahead = advanced.iter().filter(|&&a| a > advanced[i]).count();
        (grahas[i].body, CHARA_KARAKAS[ahead])
    })
}

/// A rasi's lords as a yoga counts them: its lord, or either co-lord of Scorpio or Aquarius, but only the other if one is in
/// the rasi, and either if both are (Parasara as the book gives him on p. 218, ADR 0014).
pub fn lords_in_yoga(d: Varga, chart: &[Entry; 10], rasi: Rasi) -> Vec<Body> {
    let outside: Vec<Body> = lords(rasi)
        .filter(|&g| in_varga(d, chart, g) != rasi)
        .collect();
    if outside.is_empty() {
        lords(rasi).collect()
    } else {
        outside
    }
}

/// The lords a §11.6 rule tries, in turn: [`lords_in_yoga`]'s, then a seven-planet co-lord at home, since "Saturn also owns
/// 5th" (p. 218, ADR 0014). A node at home is not tried: the book says that only of Saturn.
fn lords_to_try(d: Varga, chart: &[Entry; 10], rasi: Rasi) -> Vec<Body> {
    let mut lords_tried = lords_in_yoga(d, chart, rasi);
    for lord in lords(rasi) {
        if PLANETS.contains(&lord) && !lords_tried.contains(&lord) {
            lords_tried.push(lord);
        }
    }
    lords_tried
}

/// `graha`'s natural nature in `d`'s chart, only ever `Saumya` or `Kroora`: the Moon's by paksha, Mercury's by his company
/// there with the nodes as malefics, and none on a tie, which §3.2.2 has no rule for, or for a lagna (ch. 3 §3.2.2, ADR 0014).
pub fn natural_nature(d: Varga, chart: &[Entry; 10], graha: Body) -> Option<Disposition> {
    use Disposition::*;
    match graha.disposition()? {
        ByPaksha => {
            // chart() holds every Body.
            let longitude = |body| chart.iter().find(|e| e.body == body).unwrap().longitude;
            // The paksha is the day's, the same in every varga.
            Some(match tithi(longitude(Body::Sun), longitude(Body::Moon)).1 {
                Paksha::Sukla => Saumya,
                Paksha::Krishna => Kroora,
            })
        }
        ByCompany => {
            let company = grahas_in(d, chart, in_varga(d, chart, graha));
            let count = |nature| {
                company
                    .iter()
                    .filter(|&&g| g != graha && natural_nature(d, chart, g) == Some(nature))
                    .count()
            };
            let (saumya, kroora) = (count(Saumya), count(Kroora));
            match saumya.cmp(&kroora) {
                std::cmp::Ordering::Greater => Some(Saumya),
                std::cmp::Ordering::Less => Some(Kroora),
                std::cmp::Ordering::Equal if saumya == 0 => Some(Saumya),
                std::cmp::Ordering::Equal => None,
            }
        }
        fixed => Some(fixed),
    }
}

/// The classes ch. 11 §11.1 divides the yogas into. `Ravi` is the class of solar yogas, not the single [`YogaName::Ravi`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YogaClass {
    Ravi,
    Chandra,
    Mahaapurusha,
    Naabhasa,
    OtherPopular,
    Raaja,
    RaajaSambandha,
    Dhana,
    Daridra,
}

/// Every entry of ch. 11 in the book's order, its numbered ones by class and number. The Aakriti yoga `Vajra` and the Sankhya
/// yoga `Soola` are not the Sun-Moon yoga's values, and `Chaapa` is not `Chapa`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YogaName {
    // §11.2
    Vesi,
    Vosi,
    Ubhayachara,
    BudhaAaditya,
    // §11.3
    Sunaphaa,
    Anaphaa,
    Duradhara,
    Kemadruma,
    ChandraMangala,
    Adhi,
    // §11.4
    Ruchaka,
    Bhadra,
    Sasa,
    Maalavya,
    Hamsa,
    // §11.5.1
    Rajju,
    Musala,
    Nala,
    // §11.5.2
    Maalaa,
    Sarpa,
    // §11.5.3
    Gadaa,
    Sakata,
    Vihanga,
    Sringaataka,
    Hala,
    Vajra,
    Yava,
    Kamala,
    Vaapi,
    Yoopa,
    Sara,
    Sakti,
    Danda,
    Naukaa,
    Koota,
    Chatra,
    Chaapa,
    Ardhachandra,
    Chakra,
    Samudra,
    // §11.5.4
    Veenaa,
    Daama,
    Paasa,
    Kedaara,
    Soola,
    Yuga,
    Gola,
    // §11.6
    Subha,
    Asubha,
    GajaKesari,
    GuruMangala,
    Amala,
    Parvata,
    Kaahala,
    Chaamara,
    Sankha,
    Bheri,
    Mridanga,
    Sreenaatha,
    Matsya,
    Koorma,
    Khadga,
    Kusuma,
    Kalaanidhi,
    Kalpadruma,
    Lagnaadhi,
    Hari,
    Hara,
    Brahma,
    Vishnu,
    Siva,
    Trilochana,
    Gouri,
    Chandikaa,
    Lakshmi,
    Saarada,
    Bhaarathi,
    Saraswathi,
    Amsaavatara,
    Devendra,
    Indra,
    Ravi,
    Bhaaskara,
    Kulavardhana,
    Vasumati,
    Gandharva,
    Go,
    Vidyut,
    Chapa,
    Pushkala,
    Makuta,
    Jaya,
    Harsha,
    Sarala,
    Vimala,
    // §11.7.1
    BasicRaaja,
    DharmaKarmadhipati,
    VipareetaRaaja,
    /// §11.7.3 (1) to (18).
    Raaja(u8),
    /// §11.8 (1) to (15).
    RaajaSambandha(u8),
    /// §11.9 (1) to (12).
    Dhana(u8),
    /// §11.10 (1) to (13).
    Daridra(u8),
}

impl YogaName {
    /// The class ch. 11 lists it under.
    pub fn class(self) -> YogaClass {
        use YogaClass as C;
        use YogaName::*;
        match self {
            Vesi | Vosi | Ubhayachara | BudhaAaditya => C::Ravi,
            Sunaphaa | Anaphaa | Duradhara | Kemadruma | ChandraMangala | Adhi => C::Chandra,
            Ruchaka | Bhadra | Sasa | Maalavya | Hamsa => C::Mahaapurusha,
            Rajju | Musala | Nala | Maalaa | Sarpa | Gadaa | Sakata | Vihanga | Sringaataka
            | Hala | Vajra | Yava | Kamala | Vaapi | Yoopa | Sara | Sakti | Danda | Naukaa
            | Koota | Chatra | Chaapa | Ardhachandra | Chakra | Samudra | Veenaa | Daama
            | Paasa | Kedaara | Soola | Yuga | Gola => C::Naabhasa,
            Subha | Asubha | GajaKesari | GuruMangala | Amala | Parvata | Kaahala | Chaamara
            | Sankha | Bheri | Mridanga | Sreenaatha | Matsya | Koorma | Khadga | Kusuma
            | Kalaanidhi | Kalpadruma | Lagnaadhi | Hari | Hara | Brahma | Vishnu | Siva
            | Trilochana | Gouri | Chandikaa | Lakshmi | Saarada | Bhaarathi | Saraswathi
            | Amsaavatara | Devendra | Indra | Ravi | Bhaaskara | Kulavardhana | Vasumati
            | Gandharva | Go | Vidyut | Chapa | Pushkala | Makuta | Jaya | Harsha | Sarala
            | Vimala => C::OtherPopular,
            BasicRaaja | DharmaKarmadhipati | VipareetaRaaja | Raaja(_) => C::Raaja,
            RaajaSambandha(_) => C::RaajaSambandha,
            Dhana(_) => C::Dhana,
            Daridra(_) => C::Daridra,
        }
    }
}

/// A condition of a yoga with no measure in the book, left to the reader (ADR 0014).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToJudge {
    /// Mercury's company ties and the yoga holds only with one of his natures.
    MercurysNature,
    /// A graha of the opposite nature is in a kendra of a Dala yoga, which "may not operate well" (§11.5.2).
    MayNotOperateWell,
    /// A rule's "strong": no threshold anywhere in the book.
    Strong(Body),
    /// A stated "not combust": this graha shares the Sun's rasi in the rasi chart, and the book gives no number.
    Combust(Body),
    /// A stated "deep exaltation": this graha is in his uchcha rasi, and how near the degree must be has no number (ADR 0014).
    DeepExaltation(Body),
    /// Vasumati's "full results" want no malefic in the upachayas (§11.6), and this one is there.
    MaleficInUpachaya(Body),
}

/// A yoga present in a chart, or to judge if `to_judge` holds anything; `name.class()` gives its class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yoga {
    pub name: YogaName,
    /// The grahas forming it: both of a conjunction, only those placed from the Sun or Moon, all seven of a shape or count,
    /// only those of the stated nature in the kendras for Maalaa, Sarpa, Vajra and Yava, none for Kemadruma, the four in the
    /// book's order, repeats included, for Kalpadruma, and otherwise every graha a rule names, in graha order.
    pub grahas: Vec<Body>,
    pub to_judge: Vec<ToJudge>,
    /// Which of the book's forms held, 1 or 2, for a yoga with an "alternately" form or Brahma's second; a present form before a
    /// to-judge one, then the book's order.
    pub form: Option<u8>,
}

/// The Sankhya yoga by how many rasis the seven planets occupy (§11.5.4); [`yogas`] adds the proviso, so never lists `Gola`.
pub fn sankhya_yoga(planets: [Rasi; 7]) -> YogaName {
    use YogaName::*;
    let occupied = RASIS.iter().filter(|r| planets.contains(r)).count();
    [Gola, Yuga, Soola, Kedaara, Paasa, Daama, Veenaa][occupied - 1]
}

/// The yogas of `d`'s chart, counted from that chart's lagna, in the book's order; absent ones are not listed (ADR 0014).
pub fn yogas(d: Varga, chart: &[Entry; 10]) -> Vec<Yoga> {
    use Body::*;
    use YogaName as N;

    let at = |body| in_varga(d, chart, body);
    let yoga = |name, grahas| Yoga {
        name,
        grahas,
        to_judge: Vec::new(),
        form: None,
    };
    // Mars to Saturn: the Sun's and Moon's 2nd and 12th leave out the other light and the nodes (ADR 0014).
    let others_in = |rasi| -> Vec<Body> {
        PLANETS[2..]
            .iter()
            .copied()
            .filter(|&g| at(g) == rasi)
            .collect()
    };
    // Both sides are the third yoga alone, not the two single ones as well.
    let flanking = |from, [second, twelfth, both]: [YogaName; 3]| {
        let (s, t) = (others_in(kth(from, 2)), others_in(kth(from, 12)));
        match (s.is_empty(), t.is_empty()) {
            (true, true) => None,
            (false, true) => Some(yoga(second, s)),
            (true, false) => Some(yoga(twelfth, t)),
            (false, false) => Some(yoga(both, [s, t].concat())),
        }
    };

    let mut found = Vec::new();

    // §11.2.1–11.2.4. Budha-Aaditya's combustion is a note on its power (ADR 0014).
    let sun = at(Sun);
    found.extend(flanking(sun, [N::Vesi, N::Vosi, N::Ubhayachara]));
    if at(Mercury) == sun {
        found.push(yoga(N::BudhaAaditya, vec![Sun, Mercury]));
    }

    // §11.3.1–11.3.6.
    let moon = at(Moon);
    let lagna = at(Lagna);
    let kendra = |rasi| matches!(bhava(lagna, rasi), 1 | 4 | 7 | 10);
    found.extend(flanking(moon, [N::Sunaphaa, N::Anaphaa, N::Duradhara]));
    let beside_moon = [1, 2, 12]
        .iter()
        .any(|&k| !others_in(kth(moon, k)).is_empty());
    let in_kendra = PLANETS.iter().any(|&g| g != Moon && kendra(at(g)));
    if !beside_moon && !in_kendra {
        found.push(yoga(N::Kemadruma, Vec::new()));
    }
    if at(Mars) == moon {
        found.push(yoga(N::ChandraMangala, vec![Moon, Mars]));
    }
    // Two or more natural benefics in the places, in any arrangement; a kroora Mercury is not one, and the Moon is counted from (ADR 0014).
    let places = [6, 7, 8].map(|k| kth(moon, k));
    found.extend(with_mercury_each_way(d, chart, N::Adhi, |mercury| {
        let held: Vec<Body> = [Mercury, Jupiter, Venus]
            .into_iter()
            .filter(|&g| g != Mercury || mercury == Disposition::Saumya)
            .filter(|&g| places.contains(&at(g)))
            .collect();
        (held.len() >= 2).then_some(held)
    }));

    // §11.4.1–11.4.5, by whole rasi. Hamsa's "Ruchaka" is a misprint (ADR 0014).
    for (name, graha) in [
        (N::Ruchaka, Mars),
        (N::Bhadra, Mercury),
        (N::Sasa, Saturn),
        (N::Maalavya, Venus),
        (N::Hamsa, Jupiter),
    ] {
        let rasi = at(graha);
        if kendra(rasi) && (lords(rasi).any(|g| g == graha) || exalted(graha, rasi)) {
            found.push(yoga(name, vec![graha]));
        }
    }

    // §11.5: over the seven planets, as "all the planets" means (ADR 0014).
    let planets = PLANETS.map(at);
    let houses = planets.map(|r| bhava(lagna, r));
    let within = |places: &[u8]| houses.iter().all(|h| places.contains(h));
    let forms = |places: &[u8]| within(places) && places.iter().all(|p| houses.contains(p));
    let seven_from = |k: u8| -> [u8; 7] { std::array::from_fn(|i| (k - 1 + i as u8) % 12 + 1) };
    let first_naabhasa = found.len();
    // Among the grahas only a tied Mercury has no nature.
    let of_nature =
        |g: Body, nature, mercury_as| natural_nature(d, chart, g).unwrap_or(mercury_as) == nature;

    // §11.5.1.
    for (name, mobility) in [
        (N::Rajju, Mobility::Chara),
        (N::Musala, Mobility::Sthira),
        (N::Nala, Mobility::Dwiswabhava),
    ] {
        if planets.iter().all(|r| r.mobility() == mobility) {
            found.push(yoga(name, PLANETS.to_vec()));
        }
    }

    // §11.5.2. A tied Mercury always has both natures beside him, so his reading never decides a kendra or the note.
    for (name, forming, opposite) in [
        (N::Maalaa, Disposition::Saumya, Disposition::Kroora),
        (N::Sarpa, Disposition::Kroora, Disposition::Saumya),
    ] {
        let row = with_mercury_each_way(d, chart, name, |mercury_as| {
            let grahas: Vec<Body> = GRAHAS
                .into_iter()
                .filter(|&g| kendra(at(g)) && of_nature(g, forming, mercury_as))
                .collect();
            let held = [1, 4, 7, 10]
                .iter()
                .filter(|&&k| grahas.iter().any(|&g| bhava(lagna, at(g)) == k))
                .count();
            (held >= 3).then_some(grahas)
        });
        found.extend(row.map(|mut row| {
            if GRAHAS
                .iter()
                .any(|&g| kendra(at(g)) && natural_nature(d, chart, g) == Some(opposite))
            {
                row.to_judge.push(ToJudge::MayNotOperateWell);
            }
            row
        }));
    }

    // §11.5.3: every place occupied, but for Kamala and Vaapi (ADR 0014).
    for (name, formed) in [
        (
            N::Gadaa,
            [[1, 4], [4, 7], [7, 10], [10, 1]].iter().any(|p| forms(p)),
        ),
        (N::Sakata, forms(&[1, 7])),
        (N::Vihanga, forms(&[4, 10])),
        (N::Sringaataka, forms(&[1, 5, 9])),
        (
            N::Hala,
            [[2, 6, 10], [3, 7, 11], [4, 8, 12]]
                .iter()
                .any(|p| forms(p)),
        ),
    ] {
        if formed {
            found.push(yoga(name, PLANETS.to_vec()));
        }
    }
    // Each named house holds at least one of its nature, others beside it or not (ADR 0014).
    for (name, in_1st_7th, in_4th_10th) in [
        (N::Vajra, Disposition::Saumya, Disposition::Kroora),
        (N::Yava, Disposition::Kroora, Disposition::Saumya),
    ] {
        found.extend(with_mercury_each_way(d, chart, name, |mercury_as| {
            let grahas: Vec<Body> = GRAHAS
                .into_iter()
                .filter(|&g| match bhava(lagna, at(g)) {
                    1 | 7 => of_nature(g, in_1st_7th, mercury_as),
                    4 | 10 => of_nature(g, in_4th_10th, mercury_as),
                    _ => false,
                })
                .collect();
            [1, 4, 7, 10]
                .iter()
                .all(|&k| grahas.iter().any(|&g| bhava(lagna, at(g)) == k))
                .then_some(grahas)
        }));
    }
    for (name, formed) in [
        (N::Kamala, within(&[1, 4, 7, 10])),
        (N::Vaapi, within(&[2, 5, 8, 11]) || within(&[3, 6, 9, 12])),
        (N::Yoopa, forms(&[1, 2, 3, 4])),
        (N::Sara, forms(&[4, 5, 6, 7])),
        (N::Sakti, forms(&[7, 8, 9, 10])),
        (N::Danda, forms(&[10, 11, 12, 1])),
        (N::Naukaa, forms(&seven_from(1))),
        (N::Koota, forms(&seven_from(4))),
        (N::Chatra, forms(&seven_from(7))),
        (N::Chaapa, forms(&seven_from(10))),
        (
            N::Ardhachandra,
            [2, 5, 8, 11, 3, 6, 9, 12]
                .iter()
                .any(|&k| forms(&seven_from(k))),
        ),
        (N::Chakra, forms(&[1, 3, 5, 7, 9, 11])),
        (N::Samudra, forms(&[2, 4, 6, 8, 10, 12])),
    ] {
        if formed {
            found.push(yoga(name, PLANETS.to_vec()));
        }
    }

    // §11.5.4: a to-judge row is applicable too, so it also stops these (ADR 0014).
    if found.len() == first_naabhasa {
        found.push(yoga(sankhya_yoga(planets), PLANETS.to_vec()));
    }

    found.extend(other_popular_yogas(d, chart));

    found
}

/// §11.6, Subha to Vimala. "Planets" are the seven, "benefics" and "malefics" by [`natural_nature`] (ADR 0014).
fn other_popular_yogas(d: Varga, chart: &[Entry; 10]) -> Vec<Yoga> {
    use Body::*;
    use Disposition::{Kroora, Saumya};
    use YogaName as N;

    let at = |body| in_varga(d, chart, body);
    let lagna = at(Lagna);
    let kendra = |rasi| matches!(bhava(lagna, rasi), 1 | 4 | 7 | 10);
    let trikona = |rasi| matches!(bhava(lagna, rasi), 1 | 5 | 9);
    let yoga = |name, grahas| Yoga {
        name,
        grahas,
        to_judge: Vec::new(),
        form: None,
    };
    // Among the grahas only a tied Mercury has no nature.
    let of_nature =
        |g, nature, mercury_as| natural_nature(d, chart, g).unwrap_or(mercury_as) == nature;
    let of_nature_in = |rasi, nature, mercury_as| -> Vec<Body> {
        GRAHAS
            .into_iter()
            .filter(|&g| at(g) == rasi && of_nature(g, nature, mercury_as))
            .collect()
    };
    // "Only" benefics: a tied Mercury always has a malefic beside him, so failing him here is right either way (§3.2.2).
    let only_benefics = |rasi: Rasi| {
        grahas_in(d, chart, rasi)
            .iter()
            .all(|&g| natural_nature(d, chart, g) == Some(Saumya))
    };
    let conjoins_or_graha_drishti_on =
        |g, other| at(g) == at(other) || graha_drishti(g, at(g)).contains(&at(other));
    // The seven-planet lord and the compound relationship, as lord_relationship reads them, but in `d`'s chart (§3.4.3).
    let with_rasi_lord = |g: Body| {
        let lord = at(g).lord();
        Some(compound(natural(g, lord)?, tatkaala(at(g), at(lord))))
    };
    // A graha is never in a kendra from himself: these are relations between two (ADR 0014).
    let kendra_from = |a, b| a != b && matches!(bhava(at(a), at(b)), 1 | 4 | 7 | 10);
    let own_or_uchcha = |g| lords(at(g)).any(|l| l == g) || exalted(g, at(g));
    let row = |name, form, grahas: &[Body], to_judge| Yoga {
        name,
        grahas: in_graha_order(grahas),
        to_judge,
        form,
    };
    let strong = |g| vec![ToJudge::Strong(g)];
    // A graha in his own rasi is his own dispositor, whatever the co-lord rule says: "Saturn himself" of Aq in chart 9's navamsa.
    let dispositors = |v, g| {
        let rasi = in_varga(v, chart, g);
        if lords(rasi).any(|l| l == g) {
            vec![g]
        } else {
            lords_to_try(v, chart, rasi)
        }
    };
    let in_house = |g, k| at(g) == kth(lagna, k);
    let friendly = |g| {
        own_or_uchcha(g)
            || matches!(
                with_rasi_lord(g),
                Some(Compound::Mitra | Compound::Adhimitra)
            )
    };
    // A graha never joins himself: these are relations between two (ADR 0014).
    let joins = |a, b| a != b && at(a) == at(b);
    let mut found = Vec::new();

    for (name, nature) in [(N::Subha, Saumya), (N::Asubha, Kroora)] {
        found.extend(with_mercury_each_way(d, chart, name, |mercury_as| {
            let [first, second, twelfth] =
                [1, 2, 12].map(|k| of_nature_in(kth(lagna, k), nature, mercury_as));
            let kartari = if second.is_empty() || twelfth.is_empty() {
                Vec::new()
            } else {
                [second, twelfth].concat()
            };
            let grahas = in_graha_order(&[first, kartari].concat());
            (!grahas.is_empty()).then_some(grahas)
        }));
    }

    // "Some authors'" form from the lagna is not computed (ADR 0014).
    let jupiter = at(Jupiter);
    let in_enemys_house = matches!(
        with_rasi_lord(Jupiter),
        Some(Compound::Satru | Compound::Adhisatru)
    );
    let debilitated = Jupiter.dignities().is_some_and(|x| x.neecha.0 == jupiter);
    if matches!(bhava(at(Moon), jupiter), 1 | 4 | 7 | 10) && !in_enemys_house && !debilitated {
        let row = with_mercury_each_way(d, chart, N::GajaKesari, |mercury_as| {
            let benefics: Vec<Body> = [Moon, Mercury, Venus]
                .into_iter()
                .filter(|&g| {
                    of_nature(g, Saumya, mercury_as) && conjoins_or_graha_drishti_on(g, Jupiter)
                })
                .collect();
            (!benefics.is_empty())
                .then(|| in_graha_order(&[&[Moon, Jupiter], &benefics[..]].concat()))
        });
        found.extend(row.map(|mut row| {
            if d == Varga::RasiChart && at(Sun) == jupiter {
                row.to_judge.push(ToJudge::Combust(Jupiter));
            }
            row
        }));
    }

    if matches!(bhava(jupiter, at(Mars)), 1 | 7) {
        found.push(yoga(N::GuruMangala, vec![Mars, Jupiter]));
    }

    let amala: Vec<Body> = [lagna, at(Moon)]
        .map(|from| kth(from, 10))
        .into_iter()
        .filter(|&rasi| only_benefics(rasi))
        .flat_map(|rasi| grahas_in(d, chart, rasi))
        .collect();
    if !amala.is_empty() {
        found.push(yoga(N::Amala, in_graha_order(&amala)));
    }

    // (2) leaves the 7th free to be vacant, so a kendra may be, but one must be occupied (ADR 0014).
    let parvata = [1, 4, 7, 10, 8].map(|k| kth(lagna, k));
    let in_kendras = [1, 4, 7, 10]
        .iter()
        .any(|&k| !grahas_in(d, chart, kth(lagna, k)).is_empty());
    if in_kendras && parvata.iter().all(|&rasi| only_benefics(rasi)) {
        let grahas: Vec<Body> = parvata
            .into_iter()
            .flat_map(|rasi| grahas_in(d, chart, rasi))
            .collect();
        found.push(yoga(N::Parvata, in_graha_order(&grahas)));
    }

    // "Some say 9th lord" is not computed (ADR 0014).
    let mut rows = Vec::new();
    for [fourth, first] in house_lords(d, chart, [4, 1]) {
        if kendra_from(fourth, Jupiter) {
            let grahas = [fourth, Jupiter, first];
            rows.push(row(N::Kaahala, Some(1), &grahas, strong(first)));
        }
    }
    for [fourth, tenth] in house_lords(d, chart, [4, 10]) {
        if own_or_uchcha(fourth) && at(tenth) == at(fourth) {
            rows.push(row(N::Kaahala, Some(2), &[fourth, tenth], Vec::new()));
        }
    }
    found.extend(the_best_of(rows));

    found.extend(with_mercury_each_way(d, chart, N::Chaamara, |mercury_as| {
        let mut grahas = Vec::new();
        for first in lords_to_try(d, chart, lagna) {
            let place = at(first);
            if exalted(first, place)
                && kendra(place)
                && graha_drishti(Jupiter, jupiter).contains(&place)
            {
                grahas.extend([first, Jupiter]);
            }
        }
        for k in [7, 9, 10] {
            let benefics = of_nature_in(kth(lagna, k), Saumya, mercury_as);
            if benefics.len() >= 2 {
                grahas.extend(benefics);
            }
        }
        (!grahas.is_empty()).then(|| in_graha_order(&grahas))
    }));

    let mut rows = Vec::new();
    for [first, fifth, sixth] in house_lords(d, chart, [1, 5, 6]) {
        if kendra_from(fifth, sixth) {
            let grahas = [first, fifth, sixth];
            rows.push(row(N::Sankha, Some(1), &grahas, strong(first)));
        }
    }
    for [first, tenth, ninth] in house_lords(d, chart, [1, 10, 9]) {
        if first != tenth && at(first) == at(tenth) && at(first).mobility() == Mobility::Chara {
            let grahas = [first, tenth, ninth];
            rows.push(row(N::Sankha, Some(2), &grahas, strong(ninth)));
        }
    }
    found.extend(the_best_of(rows));

    let planets_in =
        |rasi| -> Vec<Body> { PLANETS.iter().copied().filter(|&g| at(g) == rasi).collect() };
    let bheris_houses = [1, 2, 7, 12].map(|k| planets_in(kth(lagna, k)));
    let mut rows = Vec::new();
    for [ninth] in house_lords(d, chart, [9]) {
        if bheris_houses.iter().all(|planets| !planets.is_empty()) {
            let grahas = [&bheris_houses.concat()[..], &[ninth]].concat();
            rows.push(row(N::Bheri, Some(1), &grahas, strong(ninth)));
        }
    }
    for [ninth, first] in house_lords(d, chart, [9, 1]) {
        // The lagna's lord may be Jupiter or Venus himself, leaving the one pair.
        if kendra_from(Jupiter, Venus)
            && [Jupiter, Venus]
                .iter()
                .all(|&g| g == first || kendra_from(g, first))
        {
            let grahas = [Jupiter, Venus, first, ninth];
            rows.push(row(N::Bheri, Some(2), &grahas, strong(ninth)));
        }
    }
    found.extend(the_best_of(rows));

    // At least one, as "benefics occupy" and "some planets" read (ADR 0014).
    let dignified: Vec<Body> = PLANETS
        .iter()
        .copied()
        .filter(|&g| own_or_uchcha(g) && (kendra(at(g)) || trikona(at(g))))
        .collect();
    if !dignified.is_empty() {
        let rows = lords_to_try(d, chart, lagna).into_iter().map(|first| {
            let grahas = [&dignified[..], &[first]].concat();
            row(N::Mridanga, None, &grahas, strong(first))
        });
        found.extend(the_best_of(rows.collect()));
    }

    // Applied strictly, as footnote 33 says: so only for a Sagittarius lagna.
    let rows = house_lords(d, chart, [7, 10, 9])
        .into_iter()
        .filter(|&[seventh, tenth, ninth]| {
            at(seventh) == kth(lagna, 10) && exalted(seventh, at(seventh)) && at(tenth) == at(ninth)
        })
        .map(|[seventh, tenth, ninth]| {
            row(N::Sreenaatha, None, &[seventh, tenth, ninth], Vec::new())
        });
    found.extend(the_best_of(rows.collect()));

    // The book glosses its chaturasras as the 4th and 8th.
    found.extend(with_mercury_each_way(d, chart, N::Matsya, |mercury_as| {
        let houses = [
            of_nature_in(kth(lagna, 1), Saumya, mercury_as),
            of_nature_in(kth(lagna, 9), Saumya, mercury_as),
            planets_in(kth(lagna, 5)),
            of_nature_in(kth(lagna, 4), Kroora, mercury_as),
            of_nature_in(kth(lagna, 8), Kroora, mercury_as),
        ];
        houses
            .iter()
            .all(|held| !held.is_empty())
            .then(|| in_graha_order(&houses.concat()))
    }));

    found.extend(with_mercury_each_way(d, chart, N::Koorma, |mercury_as| {
        let benefics = [5, 6, 7].map(|k| {
            let mut held = of_nature_in(kth(lagna, k), Saumya, mercury_as);
            held.retain(|&g| friendly(g));
            held
        });
        let malefics = [1, 3, 11].map(|k| {
            let mut held = of_nature_in(kth(lagna, k), Kroora, mercury_as);
            held.retain(|&g| own_or_uchcha(g));
            held
        });
        benefics
            .iter()
            .chain(&malefics)
            .all(|held| !held.is_empty())
            .then(|| in_graha_order(&[benefics.concat(), malefics.concat()].concat()))
    }));

    let rows = house_lords(d, chart, [2, 9, 1])
        .into_iter()
        .filter(|&[second, ninth, first]| {
            at(second) == kth(lagna, 9)
                && at(ninth) == kth(lagna, 2)
                && (kendra(at(first)) || trikona(at(first)))
        })
        .map(|lords| row(N::Khadga, None, &lords, Vec::new()));
    found.extend(the_best_of(rows.collect()));

    if lagna.mobility() == Mobility::Sthira
        && kendra(at(Venus))
        && trikona(at(Moon))
        && at(Saturn) == kth(lagna, 10)
    {
        found.extend(with_mercury_each_way(d, chart, N::Kusuma, |mercury_as| {
            let mut company = of_nature_in(at(Moon), Saumya, mercury_as);
            company.retain(|&g| g != Moon);
            (!company.is_empty())
                .then(|| in_graha_order(&[&[Moon, Venus, Saturn], &company[..]].concat()))
        }));
    }

    if matches!(bhava(lagna, jupiter), 2 | 5)
        && conjoins_or_graha_drishti_on(Mercury, Jupiter)
        && conjoins_or_graha_drishti_on(Venus, Jupiter)
    {
        found.push(yoga(N::Kalaanidhi, vec![Mercury, Jupiter, Venus]));
    }

    // Rasi-only: the 4th graha is taken in the navamsa and placed in the rasi chart (ADR 0014). pp. 128–129.
    if d == Varga::RasiChart {
        let placed = |g| kendra(at(g)) || trikona(at(g)) || exalted(g, at(g));
        let mut rows = Vec::new();
        for first in lords_to_try(d, chart, lagna) {
            for second in dispositors(d, first) {
                for third in dispositors(d, second) {
                    for fourth in dispositors(Varga::Navamsa, second) {
                        let four = [first, second, third, fourth];
                        if four.iter().all(|&g| placed(g)) {
                            rows.push(yoga(N::Kalpadruma, four.to_vec()));
                        }
                    }
                }
            }
        }
        found.extend(the_best_of(rows));
    }

    // A kroora Mercury is a malefic here; "Adhi Yoga from lagna" does not add the 6th (ADR 0014).
    found.extend(with_mercury_each_way(
        d,
        chart,
        N::Lagnaadhi,
        |mercury_as| {
            let places = [kth(lagna, 7), kth(lagna, 8)];
            let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
                .into_iter()
                .filter(|&g| of_nature(g, Saumya, mercury_as) && places.contains(&at(g)))
                .collect();
            let afflicted = benefics.iter().any(|&b| {
                GRAHAS
                    .into_iter()
                    .any(|m| of_nature(m, Kroora, mercury_as) && conjoins_or_graha_drishti_on(m, b))
            });
            (benefics.len() >= 2 && !afflicted).then_some(benefics)
        },
    ));

    // Sets of places: every benefic but the graha counted from, and so at least one, Jupiter or Venus (ADR 0014).
    let trimurthi = |name, house, places: [usize; 3]| -> Vec<Yoga> {
        lords_to_try(d, chart, kth(lagna, house))
            .into_iter()
            .filter_map(|lord| {
                with_mercury_each_way(d, chart, name, |mercury_as| {
                    let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
                        .into_iter()
                        .filter(|&g| g != lord && of_nature(g, Saumya, mercury_as))
                        .collect();
                    benefics
                        .iter()
                        .all(|&g| places.iter().any(|&k| kth(at(lord), k) == at(g)))
                        .then(|| in_graha_order(&[&benefics[..], &[lord]].concat()))
                })
            })
            .collect()
    };
    found.extend(the_best_of(trimurthi(N::Hari, 2, [2, 12, 8])));
    found.extend(the_best_of(trimurthi(N::Hara, 7, [4, 9, 8])));

    let mut rows = trimurthi(N::Brahma, 1, [4, 10, 11]);
    for row in &mut rows {
        row.form = Some(1);
    }
    // The NOTE's variation, p. 130.
    for [ninth, eleventh, first, tenth] in house_lords(d, chart, [9, 11, 1, 10]) {
        let mercurys: Vec<Body> = [first, tenth]
            .into_iter()
            .filter(|&l| kendra_from(Mercury, l))
            .collect();
        if kendra_from(Jupiter, ninth) && kendra_from(Venus, eleventh) && !mercurys.is_empty() {
            let grahas = [&[Jupiter, ninth, Venus, eleventh, Mercury], &mercurys[..]].concat();
            rows.push(row(N::Brahma, Some(2), &grahas, Vec::new()));
        }
    }
    found.extend(the_best_of(rows));

    // Rasi-only: the 9th lord's navamsa picks a graha placed in the rasi chart, like those after it (ADR 0014).
    if d == Varga::RasiChart {
        let mut rows = Vec::new();
        for [ninth, tenth] in house_lords(d, chart, [9, 10]) {
            for lord in dispositors(Varga::Navamsa, ninth) {
                let grahas = [ninth, tenth, lord];
                if grahas.iter().all(|&g| in_house(g, 2)) {
                    rows.push(row(N::Vishnu, None, &grahas, Vec::new()));
                }
            }
        }
        found.extend(the_best_of(rows));
    }

    let rows = house_lords(d, chart, [5, 9, 10])
        .into_iter()
        .filter(|&[fifth, ninth, tenth]| {
            in_house(fifth, 9) && in_house(ninth, 10) && in_house(tenth, 5)
        })
        .map(|lords| row(N::Siva, None, &lords, Vec::new()));
    found.extend(the_best_of(rows.collect()));

    // The same rasi counts as a trine, as it does as a kendra, and a trine's trines are its own (ADR 0014).
    let trine_from = |a, b| matches!(bhava(at(a), at(b)), 1 | 5 | 9);
    if trine_from(Sun, Moon) && trine_from(Sun, Mars) {
        found.push(yoga(N::Trilochana, vec![Sun, Moon, Mars]));
    }

    if d == Varga::RasiChart {
        let mut rows = Vec::new();
        for [tenth, first] in house_lords(d, chart, [10, 1]) {
            for lord in dispositors(Varga::Navamsa, tenth) {
                if in_house(lord, 10) && exalted(lord, at(lord)) && joins(first, lord) {
                    rows.push(row(N::Gouri, None, &[tenth, lord, first], Vec::new()));
                }
            }
        }
        found.extend(the_best_of(rows));
    }

    if d == Varga::RasiChart && lagna.mobility() == Mobility::Sthira {
        let mut rows = Vec::new();
        for [sixth, ninth] in house_lords(d, chart, [6, 9]) {
            for a in dispositors(Varga::Navamsa, sixth) {
                for b in dispositors(Varga::Navamsa, ninth) {
                    // The Sun as one of them leaves the other, as Bheri's lagna lord does, but joins no one as both (ADR 0014).
                    if graha_drishti(sixth, at(sixth)).contains(&lagna)
                        && (a != Sun || b != Sun)
                        && [a, b].iter().all(|&g| g == Sun || joins(g, Sun))
                    {
                        let grahas = [sixth, ninth, Sun, a, b];
                        rows.push(row(N::Chandikaa, None, &grahas, Vec::new()));
                    }
                }
            }
        }
        found.extend(the_best_of(rows));
    }

    // "Quadrant" holds of the own rasi and the uchcha alike.
    let rows = house_lords(d, chart, [9, 1])
        .into_iter()
        .filter(|&[ninth, _]| own_or_uchcha(ninth) && kendra(at(ninth)))
        .map(|[ninth, first]| row(N::Lakshmi, None, &[ninth, first], strong(first)));
    found.extend(the_best_of(rows.collect()));

    let in_moons_trines: Vec<Body> = [Mercury, Jupiter]
        .into_iter()
        .filter(|&g| trine_from(Moon, g))
        .collect();
    if kendra(at(Mercury))
        && at(Sun) == Rasi::Leo
        && !in_moons_trines.is_empty()
        && in_house(Mars, 11)
    {
        let rows = house_lords(d, chart, [10])
            .into_iter()
            .filter(|&[tenth]| in_house(tenth, 5))
            .map(|[tenth]| {
                let grahas = [&[tenth, Mercury, Sun, Moon, Mars], &in_moons_trines[..]].concat();
                row(N::Saarada, None, &grahas, strong(Sun))
            });
        found.extend(the_best_of(rows.collect()));
    }

    if d == Varga::RasiChart {
        let mut rows = Vec::new();
        for k in [2, 5, 11] {
            for [lord, ninth] in house_lords(d, chart, [k, 9]) {
                for disposing in dispositors(Varga::Navamsa, lord) {
                    if exalted(disposing, at(disposing)) && joins(disposing, ninth) {
                        let grahas = [lord, disposing, ninth];
                        rows.push(row(N::Bhaarathi, None, &grahas, Vec::new()));
                    }
                }
            }
        }
        found.extend(the_best_of(rows));
    }

    let saraswathi = [Mercury, Jupiter, Venus];
    if saraswathi
        .iter()
        .all(|&g| matches!(bhava(lagna, at(g)), 1 | 2 | 4 | 5 | 7 | 9 | 10))
        && friendly(Jupiter)
    {
        found.push(yoga(N::Saraswathi, saraswathi.to_vec()));
    }

    let amsaavatara = [Jupiter, Venus, Saturn];
    if amsaavatara.iter().all(|&g| kendra(at(g))) && exalted(Saturn, at(Saturn)) {
        found.push(yoga(N::Amsaavatara, amsaavatara.to_vec()));
    }

    // Footnote 35: each lord in the other's house.
    let exchange = |a, i, b, j| in_house(a, i) && in_house(b, j);
    if lagna.mobility() == Mobility::Sthira {
        let rows = house_lords(d, chart, [2, 10, 1, 11])
            .into_iter()
            .filter(|&[second, tenth, first, eleventh]| {
                exchange(second, 10, tenth, 2) && exchange(first, 11, eleventh, 1)
            })
            .map(|lords| row(N::Devendra, None, &lords, Vec::new()));
        found.extend(the_best_of(rows.collect()));
    }

    let rows = house_lords(d, chart, [5, 11])
        .into_iter()
        .filter(|&[fifth, eleventh]| exchange(fifth, 11, eleventh, 5) && in_house(Moon, 5))
        .map(|[fifth, eleventh]| row(N::Indra, None, &[fifth, eleventh, Moon], Vec::new()));
    found.extend(the_best_of(rows.collect()));

    let rows = house_lords(d, chart, [10])
        .into_iter()
        .filter(|&[tenth]| in_house(Sun, 10) && in_house(tenth, 3) && joins(tenth, Saturn))
        .map(|[tenth]| row(N::Ravi, None, &[Sun, tenth, Saturn], Vec::new()));
    found.extend(the_best_of(rows.collect()));

    if bhava(at(Sun), at(Moon)) == 12
        && bhava(at(Sun), at(Mercury)) == 2
        && matches!(bhava(at(Moon), jupiter), 5 | 9)
    {
        found.push(yoga(N::Bhaaskara, vec![Sun, Moon, Mercury, Jupiter]));
    }

    let fifths = [lagna, at(Moon), at(Sun)].map(|from| kth(from, 5));
    if PLANETS.iter().all(|&g| fifths.contains(&at(g))) {
        found.push(yoga(N::Kulavardhana, PLANETS.to_vec()));
    }

    // Every benefic and so at least one, Jupiter or Venus. A tied Mercury has a benefic beside him, in or out with him, so his
    // reading never decides, and has a malefic too, so the malefics are named either way (ADR 0014).
    let upachaya = |g| matches!(bhava(lagna, at(g)), 3 | 6 | 10 | 11);
    let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
        .into_iter()
        .filter(|&g| of_nature(g, Saumya, Saumya))
        .collect();
    if benefics.iter().all(|&g| upachaya(g)) {
        let malefics = GRAHAS
            .into_iter()
            .filter(|&g| upachaya(g) && natural_nature(d, chart, g) == Some(Kroora));
        let to_judge = benefics
            .iter()
            .map(|&g| ToJudge::Strong(g))
            .chain(malefics.map(ToJudge::MaleficInUpachaya))
            .collect();
        found.push(row(N::Vasumati, None, &benefics, to_judge));
    }

    if exalted(Sun, at(Sun)) && in_house(Moon, 9) {
        let rows = house_lords(d, chart, [10, 1])
            .into_iter()
            .filter(|&[tenth, first]| {
                matches!(bhava(kth(lagna, 7), at(tenth)), 1 | 5 | 9)
                    && first != Jupiter
                    && conjoins_or_graha_drishti_on(Jupiter, first)
            })
            .map(|[tenth, first]| {
                let grahas = [tenth, first, Jupiter, Sun, Moon];
                row(N::Gandharva, None, &grahas, strong(Sun))
            });
        found.extend(the_best_of(rows.collect()));
    }

    // §3.3 note (5) gives Jupiter's span in the rasi chart; a varga has only the rasi (ADR 0014).
    let in_moolatrikona = if d == Varga::RasiChart {
        // chart() holds every Body.
        let longitude = chart.iter().find(|e| e.body == Jupiter).unwrap().longitude;
        dignity(Jupiter, longitude) == Some(Dignity::Moolatrikona)
    } else {
        jupiter == Rasi::Sagittarius
    };
    if in_moolatrikona {
        let rows = house_lords(d, chart, [2, 1])
            .into_iter()
            .filter(|&[second, first]| joins(second, Jupiter) && exalted(first, at(first)))
            .map(|[second, first]| row(N::Go, None, &[second, Jupiter, first], strong(Jupiter)));
        found.extend(the_best_of(rows.collect()));
    }

    if d == Varga::RasiChart {
        let rows = house_lords(d, chart, [11, 1])
            .into_iter()
            .filter(|&[eleventh, first]| {
                exalted(eleventh, at(eleventh))
                    && joins(eleventh, Venus)
                    // A Venus who is the lagna lord leaves the 11th lord, as Bheri's does (ADR 0014).
                    && [eleventh, Venus]
                        .iter()
                        .all(|&g| g == first || kendra_from(first, g))
            })
            .map(|[eleventh, first]| {
                let deep = vec![ToJudge::DeepExaltation(eleventh)];
                row(N::Vidyut, None, &[eleventh, Venus, first], deep)
            });
        found.extend(the_best_of(rows.collect()));
    }

    let rows = house_lords(d, chart, [4, 10, 1])
        .into_iter()
        .filter(|&[fourth, tenth, first]| {
            exchange(fourth, 10, tenth, 4) && exalted(first, at(first))
        })
        .map(|lords| row(N::Chapa, None, &lords, Vec::new()));
    found.extend(the_best_of(rows.collect()));

    // The book numbers its four conditions (1), (2), (2), (4).
    let lagna_planets = planets_in(lagna);
    if !lagna_planets.is_empty() {
        let mut rows = Vec::new();
        for first in lords_to_try(d, chart, lagna) {
            for lord in dispositors(d, Moon) {
                if joins(first, Moon)
                    && (kendra(at(lord)) || with_rasi_lord(lord) == Some(Compound::Adhimitra))
                    && graha_drishti(lord, at(lord)).contains(&lagna)
                {
                    let grahas = [&[first, Moon, lord], &lagna_planets[..]].concat();
                    rows.push(row(N::Pushkala, None, &grahas, Vec::new()));
                }
            }
        }
        found.extend(the_best_of(rows));
    }

    // A tied Mercury has a benefic beside him, so his reading never decides; a saumya reading lists him (ADR 0014).
    let beyond_jupiter = of_nature_in(kth(jupiter, 9), Saumya, Saumya);
    if !beyond_jupiter.is_empty() && in_house(Saturn, 10) {
        let rows = house_lords(d, chart, [9])
            .into_iter()
            .filter(|&[ninth]| bhava(at(ninth), jupiter) == 9)
            .map(|[ninth]| {
                let grahas = [&[ninth, Jupiter, Saturn], &beyond_jupiter[..]].concat();
                row(N::Makuta, None, &grahas, Vec::new())
            });
        found.extend(the_best_of(rows.collect()));
    }

    if d == Varga::RasiChart {
        let rows = house_lords(d, chart, [10, 6])
            .into_iter()
            .filter(|&[tenth, sixth]| {
                exalted(tenth, at(tenth))
                    && sixth.dignities().is_some_and(|x| x.neecha.0 == at(sixth))
            })
            .map(|[tenth, sixth]| {
                let deep = vec![ToJudge::DeepExaltation(tenth)];
                row(N::Jaya, None, &[tenth, sixth], deep)
            });
        found.extend(the_best_of(rows.collect()));
    }

    for (name, k) in [(N::Harsha, 6), (N::Sarala, 8), (N::Vimala, 12)] {
        let rows = lords_to_try(d, chart, kth(lagna, k))
            .into_iter()
            .filter(|&lord| in_house(lord, k))
            .map(|lord| row(name, None, &[lord], Vec::new()));
        found.extend(the_best_of(rows.collect()));
    }

    found
}

/// Each way of choosing a lord for `houses` from `d`'s lagna, a co-lord giving two (ADR 0014).
fn house_lords<const N: usize>(
    d: Varga,
    chart: &[Entry; 10],
    houses: [usize; N],
) -> Vec<[Body; N]> {
    let lagna = in_varga(d, chart, Body::Lagna);
    let mut ways = vec![[Body::Lagna; N]];
    for (i, k) in houses.into_iter().enumerate() {
        let of_house = lords_to_try(d, chart, kth(lagna, k));
        ways = ways
            .iter()
            .flat_map(|way| {
                of_house.iter().map(move |&lord| {
                    let mut way = *way;
                    way[i] = lord;
                    way
                })
            })
            .collect();
    }
    ways
}

/// The row of a yoga with several forms or lords that hold: the least left to judge, then the first (ADR 0014).
fn the_best_of(rows: Vec<Yoga>) -> Option<Yoga> {
    rows.into_iter().min_by_key(|row| row.to_judge.len())
}

/// `grahas` once each, in the book's order.
fn in_graha_order(grahas: &[Body]) -> Vec<Body> {
    GRAHAS.into_iter().filter(|g| grahas.contains(g)).collect()
}

/// The yoga `rule` gives its grahas for, Mercury being `Saumya` or `Kroora`: as he is in `d`'s chart, or each way on a tie, when
/// agreement is the result and a difference is to judge (ch. 3 §3.2.2, ADR 0014). On agreement the saumya reading's grahas stand.
fn with_mercury_each_way(
    d: Varga,
    chart: &[Entry; 10],
    name: YogaName,
    rule: impl Fn(Disposition) -> Option<Vec<Body>>,
) -> Option<Yoga> {
    use Disposition::{Kroora, Saumya};
    let (grahas, to_judge) = match natural_nature(d, chart, Body::Mercury) {
        Some(nature) => (rule(nature)?, Vec::new()),
        None => match (rule(Saumya), rule(Kroora)) {
            (Some(grahas), Some(_)) => (grahas, Vec::new()),
            (Some(grahas), None) | (None, Some(grahas)) => (grahas, vec![ToJudge::MercurysNature]),
            (None, None) => return None,
        },
    };
    Some(Yoga {
        name,
        grahas,
        to_judge,
        form: None,
    })
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
            sun_at_sunrise: 0.0,
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
    fn the_sun_based_upagrahas_are_sun_baseds_five() {
        let five: Vec<_> = UPAGRAHAS.into_iter().filter(|u| u.is_sun_based()).collect();
        assert_eq!(five, sun_based(0.0).map(|(u, _)| u));
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

    #[test]
    fn a_polar_sky_without_a_vedic_day_has_only_the_sree_lagna() {
        let sky = sky(dms(5, 25, 5, 0.0), [[dms(6, 13, 6, 0.0), 13.0]; 8]);
        let got = special_lagnas(&sky);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, SpecialLagna::SreeLagna);
        assert!((got[0].1 - dms(11, 18, 47, 0.0)).abs() < 1e-9, "{got:?}");
    }

    #[test]
    fn special_lagnas_count_the_exact_time_since_sunrise_not_the_days_length() {
        // A 23-hour Vedic day, born 90 minutes and 1 second after sunrise.
        let sunrise = WEDNESDAY + hours(6, 0);
        let mut day = day(sunrise, sunrise + 23.0 / 24.0);
        day.sun_at_sunrise = dms(9, 24, 17, 0.0);
        let sky = Sky {
            jd: sunrise + hours(1, 30) + 1.0 / 86400.0,
            vedic_day: Some(day),
            ..sky(0.0, [[0.0, 1.0]; 8])
        };
        let got = special_lagnas(&sky);
        let want = [
            (SpecialLagna::HoraLagna, dms(11, 9, 17, 30.0)),
            (SpecialLagna::GhatiLagna, dms(1, 16, 48, 15.0)),
            (SpecialLagna::SreeLagna, 0.0),
            (SpecialLagna::BhaavaLagna, dms(10, 16, 47, 15.0)),
        ];
        assert_eq!(got.iter().map(|g| g.0).collect::<Vec<_>>(), SPECIAL_LAGNAS);
        for ((lagna, got), (_, want)) in got.into_iter().zip(want) {
            // A Julian day near 2.45e6 holds about 40 µs, which GL turns into about 0.01″.
            assert!(
                (got - want).abs() * 3600.0 < 0.05,
                "{lagna:?}: {got} {want}"
            );
        }
    }

    #[test]
    fn example_11_drekkanas_of_mercury_jupiter_venus_at_3_19_21_gemini() {
        use Rasi::*;
        let got = [3, 19, 21].map(|d| varga(Varga::Drekkana, dms(2, d, 0, 0.0)));
        assert_eq!(got, [Gemini, Libra, Aquarius]);
    }

    #[test]
    fn example_12_chaturthamsas_of_mercury_jupiter_venus_at_3_14_23_taurus() {
        use Rasi::*;
        let got = [3, 14, 23].map(|d| varga(Varga::Chaturthamsa, dms(1, d, 0, 0.0)));
        assert_eq!(got, [Taurus, Leo, Aquarius]);
    }

    #[test]
    fn examples_13_to_25_place_mercury_in_gemini_and_jupiter_at_19_scorpio_or_virgo() {
        use Rasi::*;
        use Varga::*;
        // (example, varga, Mercury's degrees in Gemini, Jupiter's rasi at 19°, Mercury's and Jupiter's rasis in the varga).
        let examples = [
            (13, Shashthamsa, 11, Scorpio, [Gemini, Capricorn]),
            (14, Saptamsa, 10, Virgo, [Leo, Cancer]),
            // Erratum: the example ends "Mercury is in Le", a Le/Li typo; its own count, the 3rd from Le, gives Li.
            (15, Ashtamsa, 10, Scorpio, [Libra, Taurus]),
            (16, Navamsa, 11, Scorpio, [Capricorn, Sagittarius]),
            (17, Dasamsa, 10, Scorpio, [Virgo, Capricorn]),
            (18, Rudramsa, 11, Scorpio, [Gemini, Pisces]),
            (19, Dwadasamsa, 11, Scorpio, [Libra, Gemini]),
            (20, Shodasamsa, 11, Scorpio, [Taurus, Gemini]),
            (21, Vimsamsa, 11, Scorpio, [Pisces, Sagittarius]),
            (22, Chaturvimsamsa, 11, Scorpio, [Aries, Libra]),
            // Erratum: the example counts "the 10th from Li is Le"; from Libra the 10th is Cancer (§6.2.13).
            (23, Nakshatramsa, 11, Scorpio, [Cancer, Gemini]),
            (24, Khavedamsa, 11, Scorpio, [Gemini, Scorpio]),
            (25, Akshavedamsa, 11, Scorpio, [Aries, Sagittarius]),
        ];
        for (example, v, mercury, jupiter, want) in examples {
            let got =
                [(Gemini, mercury), (jupiter, 19)].map(|(r, d)| varga(v, dms(r as u8, d, 0, 0.0)));
            assert_eq!(got, want, "Example {example}");
        }
    }

    #[test]
    fn example_26_jupiter_at_222_58_is_in_the_26th_part_of_scorpio_sagittarius_in_d60() {
        assert_eq!(
            varga(Varga::Shashtyamsa, 222.0 + 58.0 / 60.0),
            Rasi::Sagittarius
        );
    }

    #[test]
    fn movable_and_earthy_rasis_start_their_count_where_6_2_says() {
        use Rasi::*;
        use Varga::*;
        // No example of §6.2 has a body in a movable or earthy rasi; the 2nd part of Cancer and of Taurus.
        let in_cancer = [Ashtamsa, Shodasamsa, Vimsamsa, Akshavedamsa]
            .map(|v| varga(v, dms(3, 0, 0, 0.0) + RASI_SPAN / f64::from(v.parts())));
        assert_eq!(in_cancer, [Taurus, Taurus, Taurus, Taurus]);
        let in_taurus = [Navamsa, Nakshatramsa]
            .map(|v| varga(v, dms(1, 0, 0, 0.0) + RASI_SPAN / f64::from(v.parts())));
        assert_eq!(in_taurus, [Aquarius, Leo]);
    }

    #[test]
    fn the_rasi_chart_is_the_rasi() {
        for r in 0..12 {
            for d in [0.0, 15.0, 29.999] {
                let longitude = f64::from(r) * RASI_SPAN + d;
                assert_eq!(varga(Varga::RasiChart, longitude), rasi(longitude).0);
            }
        }
    }

    #[test]
    fn a_part_starts_at_its_boundary_to_the_whole_arcsecond() {
        use Rasi::*;
        let drekkana = |d, m, s| varga(Varga::Drekkana, dms(0, d, m, s));
        assert_eq!([drekkana(9, 59, 59.9), drekkana(10, 0, 0.0)], [Aries, Leo]);
        // D-30's marks, just before and at each, in Aries (odd) and Taurus (even).
        let trimsamsa = |rasi, d, s| varga(Varga::Trimsamsa, dms(rasi, d, 0, 0.0) - s / 3600.0);
        let odd = [5, 10, 18, 25].map(|d| [trimsamsa(0, d, 0.1), trimsamsa(0, d, 0.0)]);
        assert_eq!(
            odd,
            [
                [Aries, Aquarius],
                [Aquarius, Sagittarius],
                [Sagittarius, Gemini],
                [Gemini, Libra]
            ]
        );
        let even = [5, 12, 20, 25].map(|d| [trimsamsa(1, d, 0.1), trimsamsa(1, d, 0.0)]);
        assert_eq!(
            even,
            [
                [Taurus, Virgo],
                [Virgo, Pisces],
                [Pisces, Capricorn],
                [Capricorn, Scorpio]
            ]
        );
        assert_eq!(
            [trimsamsa(1, 30, 0.1), trimsamsa(0, 0, 0.0)],
            [Scorpio, Aries]
        );
    }

    #[test]
    fn hora_and_panchamsa_differ_for_odd_and_even_rasis() {
        use Rasi::*;
        let hora = [(0, 14), (0, 15), (1, 14), (1, 15)]
            .map(|(r, d)| varga(Varga::Hora, dms(r, d, 0, 0.0)));
        assert_eq!(hora, [Leo, Cancer, Cancer, Leo]);
        let panchamsa = |r| [0, 6, 12, 18, 24].map(|d| varga(Varga::Panchamsa, dms(r, d, 0, 0.0)));
        assert_eq!(panchamsa(0), [Aries, Aquarius, Sagittarius, Gemini, Libra]);
        assert_eq!(panchamsa(1), [Taurus, Virgo, Pisces, Capricorn, Scorpio]);
    }

    #[test]
    fn each_varga_group_names_an_amsa_from_a_count_of_two_up_to_all_its_vargas() {
        use Amsa::*;
        use VargaGroup::*;
        let amsas = |group: VargaGroup| (0..=17).map(|count| group.amsa(count)).collect::<Vec<_>>();
        let named = |names: &[Amsa]| {
            [None, None]
                .into_iter()
                .chain(names.iter().copied().map(Some))
                .chain([None].repeat(16 - names.len()))
                .collect::<Vec<_>>()
        };
        // §6.6.1–6.6.4.
        let sapta = [
            Kimsukaamsa,
            Vyanjanaamsa,
            Chaamaraamsa,
            Chatraamsa,
            Kundalaamsa,
            Mukutaamsa,
        ];
        assert_eq!(amsas(Shadvarga), named(&sapta[..5]));
        assert_eq!(amsas(SaptaVarga), named(&sapta));
        assert_eq!(
            amsas(DasaVarga),
            named(&[
                Paarijaataamsa,
                Uttamaamsa,
                Gopuraamsa,
                Simhaasanaamsa,
                Paaraavataamsa,
                Devalokaamsa,
                Brahmalokamsa,
                Airaavataamsa,
                Sreedhaamaamsa,
            ])
        );
        assert_eq!(
            amsas(ShodasaVarga),
            named(&[
                Bhedakaamsa,
                Kusumaamsa,
                Nagapurushaamsa,
                Kandukaamsa,
                Keralaamsa,
                Kalpavrikshaamsa,
                Chandanavanaamsa,
                Poornachandraamsa,
                Uchchaisravaamsa,
                Dhanvantaryamsa,
                Sooryakaantaamsa,
                Vidrumaamsa,
                Indraasanaamsa,
                Golokaamsa,
                SreeVallabhaamsa,
            ])
        );
        assert_eq!(VARGA_GROUPS.map(|g| g.vargas().len()), [6, 7, 10, 16]);
    }

    #[test]
    fn the_lagna_has_no_amsabala() {
        assert_eq!(
            amsabala(Body::Lagna, 255.0, VargaGroup::ShodasaVarga),
            (0, None)
        );
    }

    /// A chart with the lagna and Sun to Rahu at `g`, all direct; a paaka lagna reads no speeds.
    fn placed(lagna: f64, g: [f64; 8]) -> [Entry; 10] {
        chart(&sky(lagna, g.map(|longitude| [longitude, 1.0])))
    }

    #[test]
    fn paaka_lagna_is_the_lagna_lords_rasi() {
        // §7.3.5: a Pisces lagna with Jupiter in Cancer gives Cancer; a Leo lagna with the Sun in Virgo gives Virgo.
        let mut g = [dms(0, 15, 0, 0.0); 8];
        g[4] = dms(3, 15, 0, 0.0);
        assert_eq!(
            paaka_lagna(Varga::RasiChart, &placed(dms(11, 10, 0, 0.0), g)),
            Rasi::Cancer
        );
        let mut g = [dms(0, 15, 0, 0.0); 8];
        g[0] = dms(5, 15, 0, 0.0);
        assert_eq!(
            paaka_lagna(Varga::RasiChart, &placed(dms(4, 10, 0, 0.0), g)),
            Rasi::Virgo
        );
    }

    #[test]
    fn a_vargas_paaka_lagna_takes_that_vargas_lagna_lord() {
        // Pisces 1° is Cancer in D-9, so the Moon, not the rasi chart's lord Jupiter, decides (ADR 0003).
        let mut g = [dms(0, 1, 0, 0.0); 8];
        // Aries 5°, the 2nd navamsa: Taurus. Jupiter at Aries 1° is Aries in D-9, the carried-over reading.
        g[1] = dms(0, 5, 0, 0.0);
        assert_eq!(
            paaka_lagna(Varga::Navamsa, &placed(dms(11, 1, 0, 0.0), g)),
            Rasi::Taurus
        );
    }

    #[test]
    fn scorpio_and_aquarius_lagnas_take_mars_and_saturn() {
        // Rahu in Aries puts Ketu in Libra; §15.5.1's stronger lord is not used here until ch. 15 decides (ADR 0003).
        let mut g = [dms(0, 15, 0, 0.0); 8];
        g[2] = dms(2, 15, 0, 0.0);
        g[6] = dms(4, 15, 0, 0.0);
        let chart = |lagna| placed(lagna, g);
        assert_eq!(
            paaka_lagna(Varga::RasiChart, &chart(dms(7, 10, 0, 0.0))),
            Rasi::Gemini
        );
        assert_eq!(
            paaka_lagna(Varga::RasiChart, &chart(dms(10, 10, 0, 0.0))),
            Rasi::Leo
        );
    }

    #[test]
    fn the_rasi_charts_paaka_lagna_is_the_lagna_lords_rasi() {
        // Sun to Saturn each in a different rasi, Aries to Libra, so every lord is told apart.
        let g = std::array::from_fn(|i| dms(i as u8, 15, 0, 0.0));
        for r in RASIS {
            let chart = placed(dms(r as u8, 10, 0, 0.0), g);
            assert_eq!(
                paaka_lagna(Varga::RasiChart, &chart),
                rasi_of(&chart, r.lord()),
                "{r:?}"
            );
        }
    }

    #[test]
    fn example_28_has_the_chara_karakas_of_table_14() {
        use Body::*;
        use CharaKaraka::*;
        // Ch. 8 §8.2, Example 28. Erratum: Table 14's last row prints "Venus 2°28'" for Saturn, which is at 2Ta28.
        let g = [
            dms(2, 12, 47, 0.0),
            dms(0, 20, 28, 0.0),
            dms(2, 13, 51, 0.0),
            dms(2, 25, 18, 0.0),
            dms(1, 5, 40, 0.0),
            dms(2, 17, 21, 0.0),
            dms(1, 2, 28, 0.0),
            dms(3, 1, 43, 0.0),
        ];
        assert_eq!(
            chara_karakas(&placed(0.0, g)),
            [
                (Sun, Putra),
                (Moon, Bhratri),
                (Mars, Pitri),
                (Mercury, Amatya),
                (Jupiter, Jnaati),
                (Venus, Matri),
                (Saturn, Dara),
                (Rahu, Atma),
            ]
        );
    }

    #[test]
    fn exercise_11_has_the_chara_karakas_of_table_17() {
        use Body::*;
        use CharaKaraka::*;
        // Ch. 8 §8.5 answers Exercise 11 (§8.2), though Table 17's caption says "Example 28".
        let g = [
            dms(8, 9, 36, 0.0),
            dms(4, 15, 29, 0.0),
            dms(0, 13, 40, 0.0),
            dms(7, 21, 0, 0.0),
            dms(10, 2, 6, 0.0),
            dms(8, 17, 42, 0.0),
            dms(7, 9, 41, 0.0),
            dms(2, 14, 30, 0.0),
        ];
        assert_eq!(
            chara_karakas(&placed(0.0, g)),
            [
                (Sun, Jnaati),
                (Moon, Matri),
                (Mars, Pitri),
                (Mercury, Atma),
                (Jupiter, Dara),
                (Venus, Amatya),
                (Saturn, Putra),
                (Rahu, Bhratri),
            ]
        );
    }

    #[test]
    fn grahas_at_the_same_arcsecond_share_a_karakatwa_and_the_next_is_left_empty() {
        use CharaKaraka::*;
        // §8.2: an exact tie shares a karakatwa and the next has no ruler (ADR 0011). Moon and Mars tie for 2nd.
        let g = [
            dms(0, 29, 0, 0.0),
            dms(1, 20, 0, 30.0),
            dms(2, 20, 0, 30.0),
            dms(3, 15, 0, 0.0),
            dms(4, 10, 0, 0.0),
            dms(5, 5, 0, 0.0),
            dms(6, 1, 0, 0.0),
            dms(7, 29, 30, 0.0),
        ];
        let karakas = chara_karakas(&placed(0.0, g)).map(|(_, k)| k);
        assert_eq!(
            karakas,
            [Atma, Amatya, Amatya, Matri, Pitri, Putra, Jnaati, Dara]
        );
        assert!(!karakas.contains(&Bhratri));
    }

    #[test]
    fn rahu_at_the_start_of_a_rasi_is_the_atma_karaka() {
        // ADR 0011: Rahu at 0°00′00″ has advanced 30°, ahead of any graha at 29°59′59″.
        let mut g = [dms(0, 29, 59, 59.0); 8];
        g[7] = dms(3, 0, 0, 0.0);
        let karakas = chara_karakas(&placed(0.0, g));
        assert_eq!(karakas[7], (Body::Rahu, CharaKaraka::Atma));
        assert!(karakas[..7].iter().all(|&(_, k)| k == CharaKaraka::Amatya));
    }

    #[test]
    fn rasi_drishti_follows_the_examples_of_section_10_3() {
        use Rasi::*;
        // Ch. 10 §10.3: the rasis Ar, Ta, Ge and Li are said to aspect.
        for (from, aspected) in [
            (Aries, [Leo, Scorpio, Aquarius]),
            (Taurus, [Cancer, Libra, Capricorn]),
            (Gemini, [Virgo, Sagittarius, Pisces]),
            (Libra, [Aquarius, Taurus, Leo]),
        ] {
            for to in RASIS {
                assert_eq!(
                    rasi_drishti(from, to),
                    aspected.contains(&to),
                    "{from:?} {to:?}"
                );
            }
        }
    }

    #[test]
    fn rasi_drishti_is_symmetric() {
        // §10.3: "sign Y will aspect sign X if sign X aspects sign Y".
        for a in RASIS {
            for b in RASIS {
                assert_eq!(rasi_drishti(a, b), rasi_drishti(b, a), "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn graha_drishti_follows_the_examples_of_section_10_2() {
        use Rasi::*;
        // §10.2's list of 7th-house examples.
        assert_eq!(graha_drishti(Body::Sun, Taurus), [Scorpio]);
        assert!(graha_drishti(Body::Mars, Gemini).contains(&Sagittarius));
        assert_eq!(graha_drishti(Body::Moon, Leo), [Aquarius]);
        assert!(graha_drishti(Body::Jupiter, Pisces).contains(&Virgo));
        assert!(graha_drishti(Body::Saturn, Capricorn).contains(&Cancer));
        // §10.2: Jupiter in Ta aspects Cp, "the 9th house from Ta".
        assert!(graha_drishti(Body::Jupiter, Taurus).contains(&Capricorn));
    }

    #[test]
    fn graha_drishti_is_example_34_in_count_order() {
        use Rasi::*;
        assert_eq!(
            graha_drishti(Body::Jupiter, Gemini),
            [Libra, Sagittarius, Aquarius]
        );
        assert_eq!(graha_drishti(Body::Mars, Leo), [Scorpio, Aquarius, Pisces]);
        assert_eq!(
            graha_drishti(Body::Saturn, Sagittarius),
            [Aquarius, Gemini, Virgo]
        );
    }

    #[test]
    fn the_nodes_and_the_lagna_have_no_graha_drishti() {
        // ADR 0013: Exercise 14 leaves out Rahu and Ketu, despite §10.2's "all planets".
        for body in [Body::Rahu, Body::Ketu, Body::Lagna] {
            assert!(graha_drishti(body, Rasi::Aries).is_empty());
        }
    }

    #[test]
    fn rasi_drishtis_lists_section_10_3s_examples_in_count_order() {
        use Rasi::*;
        assert_eq!(rasi_drishtis(Aries), [Leo, Scorpio, Aquarius]);
        assert_eq!(rasi_drishtis(Taurus), [Cancer, Libra, Capricorn]);
        assert_eq!(rasi_drishtis(Gemini), [Virgo, Sagittarius, Pisces]);
        // "a planet in Libra will aspect the houses and planets in Aq, Ta and Le".
        assert_eq!(rasi_drishtis(Libra), [Aquarius, Taurus, Leo]);
    }

    #[test]
    fn grahas_in_gives_the_aspected_saturn_of_section_10_2() {
        use Rasi::*;
        let chart = scene(
            Aries,
            &[(Body::Saturn, mid(Capricorn)), (Body::Jupiter, mid(Taurus))],
        );
        // §10.2: "Jupiter in Ta will aspect Saturn in Cp".
        let aspected: Vec<Body> = graha_drishti(Body::Jupiter, Taurus)
            .into_iter()
            .flat_map(|r| grahas_in(Varga::RasiChart, &chart, r))
            .collect();
        assert_eq!(aspected, [Body::Saturn]);
    }

    #[test]
    fn grahas_in_runs_in_the_books_order_and_ignores_the_lagna() {
        use Rasi::*;
        let chart = scene(
            Aries,
            &[
                (Body::Saturn, mid(Leo)),
                (Body::Lagna, mid(Leo)),
                (Body::Sun, mid(Leo)),
            ],
        );
        assert_eq!(
            grahas_in(Varga::RasiChart, &chart, Leo),
            [Body::Sun, Body::Saturn]
        );
        let lagna_alone = scene(Aries, &[(Body::Lagna, mid(Virgo))]);
        assert!(grahas_in(Varga::RasiChart, &lagna_alone, Virgo).is_empty());
    }

    #[test]
    fn argala_follows_the_example_of_section_10_6() {
        use Body::*;
        use Rasi::*;
        // Everything else in Scorpio, which Gemini's argalas don't count, Ketu moved there off its place opposite Rahu.
        let chart = scene(
            Scorpio,
            &[
                (Mercury, mid(Gemini)),
                (Jupiter, mid(Pisces)),
                (Venus, mid(Aries)),
                (Saturn, mid(Virgo)),
                (Ketu, mid(Scorpio)),
            ],
        );
        let [second, fourth, fifth, eleventh] = argala(Varga::RasiChart, &chart, Gemini);
        assert_eq!((fourth.from, fourth.obstructed_from), (4, 10));
        assert_eq!(fourth.argala, [Saturn]);
        assert_eq!(fourth.virodhargala, [Jupiter]);
        assert_eq!((eleventh.from, eleventh.obstructed_from), (11, 3));
        assert_eq!(eleventh.argala, [Venus]);
        assert!(eleventh.virodhargala.is_empty());
        for empty in [second, fifth] {
            assert!(empty.argala.is_empty() && empty.virodhargala.is_empty());
        }
    }

    #[test]
    fn argala_on_a_rasi_holding_ketu_counts_anti_zodiacally_as_in_section_10_6s_note() {
        use Body::*;
        use Rasi::*;
        // Le, Ge, Ta, Sc are the 2nd, 4th, 5th, 11th from Vi in reverse; Li, Sg, Cp, Cn the 12th, 10th, 9th, 3rd.
        let chart = scene(
            Pisces,
            &[
                (Ketu, mid(Virgo)),
                (Sun, mid(Leo)),
                (Moon, mid(Gemini)),
                (Venus, mid(Taurus)),
                (Mars, mid(Scorpio)),
                (Jupiter, mid(Libra)),
                (Mercury, mid(Sagittarius)),
                (Saturn, mid(Capricorn)),
                (Rahu, mid(Cancer)),
            ],
        );
        let rows = argala(Varga::RasiChart, &chart, Virgo);
        let cells = rows.iter().map(|a| {
            (
                a.from,
                a.argala.clone(),
                a.obstructed_from,
                a.virodhargala.clone(),
            )
        });
        assert!(cells.eq([
            (2, vec![Sun], 12, vec![Jupiter]),
            (4, vec![Moon], 10, vec![Mercury]),
            (5, vec![Venus], 9, vec![Saturn]),
            (11, vec![Mars], 3, vec![Rahu]),
        ]));
    }

    #[test]
    fn ketu_causing_argala_on_another_rasi_does_not_reverse_it() {
        use Rasi::*;
        // Ketu in Cancer, the 2nd from Gemini; Saturn in Virgo stays the 4th.
        let chart = scene(
            Scorpio,
            &[(Body::Ketu, mid(Cancer)), (Body::Saturn, mid(Virgo))],
        );
        let [second, fourth, ..] = argala(Varga::RasiChart, &chart, Gemini);
        assert_eq!(second.argala, [Body::Ketu]);
        assert_eq!(fourth.argala, [Body::Saturn]);
    }

    #[test]
    fn argalas_reversal_follows_ketus_rasi_in_the_varga_not_in_the_rasi_chart() {
        use Rasi::*;
        // Ketu is in Aries in D-1 and Leo in D-9; Mars at 27° Pisces is in Pisces in both, the Sun at 2° Pisces in Cancer in D-9.
        let chart = scene(
            Gemini,
            &[
                (Body::Ketu, mid(Aries)),
                (Body::Mars, dms(Pisces as u8, 27, 0, 0.0)),
                (Body::Sun, dms(Pisces as u8, 2, 0, 0.0)),
            ],
        );
        let [second, ..] = argala(Varga::RasiChart, &chart, Aries);
        assert!(second.argala.contains(&Body::Mars));
        let [second, ..] = argala(Varga::Navamsa, &chart, Aries);
        assert!(!second.argala.contains(&Body::Mars));
        assert!(second.virodhargala.contains(&Body::Mars));
        let [second, ..] = argala(Varga::Navamsa, &chart, Leo);
        assert!(second.argala.contains(&Body::Sun));
    }

    #[test]
    fn a_lagna_in_a_counted_rasi_causes_no_argala() {
        use Rasi::*;
        let chart = scene(
            Scorpio,
            &[(Body::Ketu, mid(Scorpio)), (Body::Lagna, mid(Cancer))],
        );
        for a in argala(Varga::RasiChart, &chart, Gemini) {
            assert!(a.argala.is_empty() && a.virodhargala.is_empty());
        }
    }

    #[test]
    fn a_rasis_own_grahas_never_appear_in_its_argalas() {
        for r in RASIS {
            // All nine in `r`, Ketu either opposite Rahu's place or with them: counted forwards and in reverse.
            for chart in [scene(r, &[]), scene(r, &[(Body::Ketu, mid(r))])] {
                let rows = argala(Varga::RasiChart, &chart, r);
                for a in rows {
                    assert!(a.argala.is_empty() && a.virodhargala.is_empty(), "{r:?}");
                }
            }
        }
    }

    /// `r` at 15°.
    fn mid(r: Rasi) -> f64 {
        dms(r as u8, 15, 0, 0.0)
    }

    /// A chart with everything at 15° of `rest` but for the bodies in `at`. The nodes move apart freely: only their rasis and
    /// advancement are read, and §15.5's examples never place both.
    fn scene(rest: Rasi, at: &[(Body, f64)]) -> [Entry; 10] {
        let mut chart = placed(mid(rest), [mid(rest); 8]);
        for (body, longitude) in at {
            chart
                .iter_mut()
                .find(|e| e.body == *body)
                .unwrap()
                .longitude = *longitude;
        }
        chart
    }

    fn lord_of(chart: &[Entry; 10], r: Rasi) -> Body {
        stronger_lord(Varga::RasiChart, chart, r)
    }

    #[test]
    fn a_bhava_arudha_follows_the_example_of_section_9_2() {
        // A Gemini bhava with Mercury in Aquarius: 9 rasis, then 9 on, to Libra.
        let chart = scene(Rasi::Gemini, &[(Body::Mercury, mid(Rasi::Aquarius))]);
        assert_eq!(bhava_arudhas(Varga::RasiChart, &chart)[0], Rasi::Libra);
    }

    #[test]
    fn a_graha_arudha_follows_the_example_of_section_9_5() {
        // The Sun in Gemini: 3 rasis to Leo, then 3 on, to Libra.
        let chart = scene(Rasi::Gemini, &[]);
        assert_eq!(
            graha_arudhas(Varga::RasiChart, &chart)[0],
            (Body::Sun, Rasi::Libra)
        );
    }

    /// Rahu's and Ketu's arudha padas with Rahu in `rahu` and Ketu in `ketu`.
    fn node_arudha_padas(rahu: Rasi, ketu: Rasi) -> (Rasi, Rasi) {
        let at = [(Body::Rahu, mid(rahu)), (Body::Ketu, mid(ketu))];
        let [.., (_, rahu), (_, ketu)] = graha_arudhas(Varga::RasiChart, &scene(Rasi::Aries, &at));
        (rahu, ketu)
    }

    #[test]
    fn ketu_7th_from_rahu_shares_rahus_arudha_pada() {
        // §9.5's note.
        for rahu in RASIS {
            let (a, b) = node_arudha_padas(rahu, kth(rahu, 7));
            assert_eq!(a, b, "{rahu:?}");
        }
    }

    #[test]
    fn the_nodes_together_have_opposite_arudha_padas() {
        // §9.5's note.
        for rasi in RASIS {
            let (a, b) = node_arudha_padas(rasi, rasi);
            assert_eq!(bhava(a, b), 7, "{rasi:?}");
        }
    }

    #[test]
    fn a_rasi_with_one_lord_has_that_lord() {
        let chart = scene(Rasi::Gemini, &[]);
        for r in RASIS {
            if !matches!(r, Rasi::Scorpio | Rasi::Aquarius) {
                assert_eq!(lord_of(&chart, r), r.lord(), "{r:?}");
            }
        }
    }

    #[test]
    fn basic_rule_a_co_lord_in_the_rasi_gives_way_to_the_other() {
        use Body::*;
        use Rasi::*;
        // §15.5.1's basic rule, in either order.
        for (inside, other, r) in [
            (Saturn, Rahu, Aquarius),
            (Rahu, Saturn, Aquarius),
            (Mars, Ketu, Scorpio),
            (Ketu, Mars, Scorpio),
        ] {
            assert_eq!(lord_of(&scene(Gemini, &[(inside, mid(r))]), r), other);
        }
    }

    #[test]
    fn both_co_lords_in_the_rasi_go_through_the_rules() {
        use Body::*;
        use Rasi::*;
        // "Exactly one" is the basic rule; here 5b decides, Rahu's 25° over Saturn's 20°, then Saturn's 20° over Rahu's 15°.
        let at = [(Saturn, dms(10, 20, 0, 0.0)), (Rahu, dms(10, 5, 0, 0.0))];
        assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Rahu);
        let at = [(Saturn, dms(10, 20, 0, 0.0)), (Rahu, dms(10, 15, 0, 0.0))];
        assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Saturn);
    }

    #[test]
    fn rule_1_saturn_joined_by_more_planets_beats_rahu() {
        use Body::*;
        use Rasi::*;
        // §15.5.1 rule 1: Saturn in Pi with Mars and Sun, Rahu in Ar with Jupiter.
        let at = [
            (Saturn, mid(Pisces)),
            (Mars, mid(Pisces)),
            (Sun, mid(Pisces)),
            (Rahu, mid(Aries)),
            (Jupiter, mid(Aries)),
        ];
        assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Saturn);
    }

    #[test]
    fn a_lagna_with_a_co_lord_does_not_count_as_joining_it() {
        use Body::*;
        use Rasi::*;
        // ADR 0012. Were the lagna a planet Rahu would win rule 1; Saturn wins rule 4 instead.
        let at = [
            (Saturn, mid(Pisces)),
            (Rahu, mid(Aries)),
            (Lagna, mid(Aries)),
        ];
        assert_eq!(lord_of(&scene(Taurus, &at), Aquarius), Saturn);
    }

    #[test]
    fn rule_2_saturn_counts_mercury_as_dispositor_twice() {
        use Body::*;
        use Rasi::*;
        // §15.5.1 rule 2: Saturn in Ge with Mercury is 2, Rahu in Ar aspected by Mars from Le is 1. The book leaves rule 1
        // to tie, so the Sun joins Rahu.
        let at = [
            (Saturn, mid(Gemini)),
            (Mercury, mid(Gemini)),
            (Rahu, mid(Aries)),
            (Sun, mid(Aries)),
            (Mars, mid(Leo)),
            (Jupiter, mid(Taurus)),
        ];
        assert_eq!(lord_of(&scene(Capricorn, &at), Aquarius), Saturn);

        // Counted once Mercury would only tie, and Rahu, as dual and more advanced, would win.
        let at = [
            (Saturn, mid(Virgo)),
            (Mercury, mid(Virgo)),
            (Rahu, dms(8, 1, 0, 0.0)),
            (Sun, mid(Sagittarius)),
        ];
        assert_eq!(lord_of(&scene(Capricorn, &at), Aquarius), Saturn);
    }

    #[test]
    fn a_co_lord_that_is_its_own_dispositor_gets_no_count() {
        use Body::*;
        use Rasi::*;
        // ADR 0012. Saturn's Capricorn gives him 0 to Rahu's 1 from Mars; counting himself, he would win 5b.
        let at = [
            (Saturn, dms(9, 25, 0, 0.0)),
            (Rahu, dms(0, 25, 0, 0.0)),
            (Mars, mid(Leo)),
        ];
        assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Rahu);
    }

    #[test]
    fn a_co_lord_in_scorpio_counts_both_of_its_dispositors() {
        use Body::*;
        use Rasi::*;
        // ADR 0012. Saturn in Sc has Mars and Ketu, 2, to Rahu's 1 from Jupiter; with Mars alone, Rahu would win rule 4.
        let at = [
            (Saturn, mid(Scorpio)),
            (Mars, mid(Cancer)),
            (Ketu, mid(Aries)),
            (Rahu, mid(Virgo)),
            (Jupiter, mid(Sagittarius)),
        ];
        assert_eq!(lord_of(&scene(Taurus, &at), Aquarius), Saturn);
    }

    #[test]
    fn rule_3_exalted_saturn_beats_rahu() {
        use Body::*;
        use Rasi::*;
        // §15.5.1 rule 3: Saturn in Li, Rahu in Cn. Rahu at 5° has 25° to Saturn's 15°, so only exaltation gives it to Saturn.
        let at = [(Saturn, mid(Libra)), (Rahu, dms(3, 5, 0, 0.0))];
        assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Saturn);
    }

    #[test]
    fn the_nodes_are_exalted_in_gemini_and_sagittarius() {
        use Body::*;
        use Rasi::*;
        // ADR 0012, by whole rasi: at 25° each has advanced only 5°, and their rivals 29° in a dual rasi too.
        let at = [(Rahu, dms(2, 25, 0, 0.0)), (Saturn, dms(5, 29, 0, 0.0))];
        assert_eq!(lord_of(&scene(Taurus, &at), Aquarius), Rahu);
        let at = [(Ketu, dms(8, 25, 0, 0.0)), (Mars, dms(11, 29, 0, 0.0))];
        assert_eq!(lord_of(&scene(Taurus, &at), Scorpio), Ketu);
    }

    #[test]
    fn rule_4_mars_in_a_dual_rasi_beats_ketu_in_a_fixed_one() {
        use Body::*;
        use Rasi::*;
        // §15.5.1 rule 4: Mars in Ge, Ketu in Aq. Ketu is the more advanced, so only the rasi gives it to Mars.
        let at = [(Mars, mid(Gemini)), (Ketu, dms(10, 1, 0, 0.0))];
        assert_eq!(lord_of(&scene(Taurus, &at), Scorpio), Mars);
    }

    #[test]
    fn rule_5b_ketu_more_advanced_from_the_rasis_end_beats_mars() {
        use Body::*;
        use Rasi::*;
        // §15.5.1 rule 5b: Mars at 23Li17 has 23°17′, Ketu at 5Cn54 has 24°06′.
        let at = [(Mars, dms(6, 23, 17, 0.0)), (Ketu, dms(3, 5, 54, 0.0))];
        assert_eq!(lord_of(&scene(Gemini, &at), Scorpio), Ketu);
    }

    #[test]
    fn a_last_tie_goes_to_the_seven_planet_lord() {
        use Body::*;
        use Rasi::*;
        // ADR 0012's shortcut: both at 15° of a movable rasi, with nothing else to tell them apart.
        let at = [(Mars, mid(Libra)), (Ketu, mid(Cancer))];
        assert_eq!(lord_of(&scene(Gemini, &at), Scorpio), Mars);
        let at = [(Saturn, mid(Cancer)), (Rahu, mid(Aries))];
        assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Saturn);
    }

    /// `winner` is the stronger rasi whichever is named first.
    #[track_caller]
    fn assert_stronger(chart: &[Entry; 10], winner: Rasi, loser: Rasi) {
        let d = Varga::RasiChart;
        assert_eq!(stronger_rasi(d, chart, winner, loser), winner);
        assert_eq!(stronger_rasi(d, chart, loser, winner), winner);
    }

    #[test]
    fn rule_1_the_rasi_with_more_planets_is_stronger() {
        use Body::*;
        use Rasi::*;
        // §15.5.2 rule 1: Saturn and Jupiter in Ar, Venus in Li.
        let at = [
            (Saturn, mid(Aries)),
            (Jupiter, mid(Aries)),
            (Venus, mid(Libra)),
        ];
        assert_stronger(&scene(Gemini, &at), Aries, Libra);
    }

    #[test]
    fn a_lagna_in_a_rasi_does_not_count_as_a_planet_in_it() {
        use Body::*;
        use Rasi::*;
        // ADR 0012. Were the lagna a planet rule 1 would tie, and the last tie would give Aries.
        let at = [(Lagna, mid(Aries)), (Moon, mid(Libra))];
        assert_stronger(&scene(Gemini, &at), Libra, Aries);
    }

    #[test]
    fn rule_2_the_rasi_with_more_of_jupiter_mercury_and_its_lord_is_stronger() {
        use Body::*;
        use Rasi::*;
        // §15.5.2 rule 2: Ar has Jupiter, Li is aspected by Mercury and Venus from Ta. The book leaves rule 1 to tie, so
        // the Moon is in Li.
        let at = [
            (Jupiter, mid(Aries)),
            (Mercury, mid(Taurus)),
            (Venus, mid(Taurus)),
            (Mars, mid(Virgo)),
            (Moon, mid(Libra)),
        ];
        assert_stronger(&scene(Gemini, &at), Libra, Aries);
    }

    #[test]
    fn rahus_aspect_counts_for_aquarius_though_saturn_is_its_stronger_lord() {
        use Body::*;
        use Rasi::*;
        // Exercise 26(5), ADR 0012. Saturn is the lord by rule 1, so without Rahu's aspect rule 6 would give Le to the Sun.
        let at = [(Sun, dms(2, 25, 0, 0.0)), (Rahu, mid(Libra))];
        let chart = scene(Gemini, &at);
        assert_eq!(lord_of(&chart, Aquarius), Saturn);
        assert_stronger(&chart, Aquarius, Leo);
    }

    #[test]
    fn rule_3_the_rasi_with_an_exalted_planet_is_stronger() {
        use Body::*;
        use Rasi::*;
        // §15.5.2 rule 3: Saturn and Mercury in Li, Jupiter and Venus in Ar.
        let at = [
            (Saturn, mid(Libra)),
            (Mercury, mid(Libra)),
            (Jupiter, mid(Aries)),
            (Venus, mid(Aries)),
        ];
        assert_stronger(&scene(Gemini, &at), Libra, Aries);
    }

    #[test]
    fn rule_3_reads_exaltation_by_whole_rasi() {
        use Body::*;
        use Rasi::*;
        // ADR 0012: Mercury at 25° Vi is own rasi by §3.3's degrees, yet exalted here. Each rasi counts 3 in rule 2, and rule 6 would give Pi.
        let at = [
            (Mercury, dms(5, 25, 0, 0.0)),
            (Jupiter, dms(11, 28, 0, 0.0)),
        ];
        assert_stronger(&scene(Taurus, &at), Virgo, Pisces);
    }

    #[test]
    fn rule_4_the_rasi_whose_lord_is_in_a_rasi_of_other_oddity_is_stronger() {
        use Body::*;
        use Rasi::*;
        // §15.5.2 rule 4: Venus in Cn, Mars in Le. Mars aspects Ar, so the book's tie needs Jupiter aspecting Li from Ta.
        let at = [
            (Venus, mid(Cancer)),
            (Mars, mid(Leo)),
            (Jupiter, mid(Taurus)),
        ];
        assert_stronger(&scene(Gemini, &at), Libra, Aries);
    }

    #[test]
    fn a_rule_4_tie_on_a_grahas_own_rasis_goes_on_to_rule_5() {
        use Body::*;
        use Rasi::*;
        // ADR 0012: Scorpio's lord is Ketu, by rule 1. Mars in odd Le and Ketu in even Cn, both like the rasi they rule.
        let at = [(Mars, mid(Leo)), (Ketu, mid(Cancer)), (Moon, mid(Cancer))];
        let chart = scene(Gemini, &at);
        assert_eq!(lord_of(&chart, Scorpio), Ketu);
        assert_stronger(&chart, Scorpio, Aries);

        // Both unlike their rasi: with Mars as Scorpio's lord too, rule 4 would give Aries. Jupiter in Le evens rule 2.
        let at = [
            (Mars, mid(Cancer)),
            (Ketu, mid(Gemini)),
            (Moon, mid(Gemini)),
            (Jupiter, mid(Leo)),
        ];
        let chart = scene(Pisces, &at);
        assert_eq!(lord_of(&chart, Scorpio), Ketu);
        assert_stronger(&chart, Scorpio, Aries);
    }

    #[test]
    fn rule_6_the_rasi_whose_lord_is_more_advanced_is_stronger() {
        use Body::*;
        use Rasi::*;
        // §15.5.2 rule 6: Mars at 23Ge17 over Venus at 19Le51. Venus aspects Li, so the book's tie needs Jupiter aspecting
        // Ar from Sc. Then the other way round, which the last tie could not give.
        let at = |mars, venus| [(Mars, mars), (Venus, venus), (Jupiter, mid(Scorpio))];
        let chart = scene(Gemini, &at(dms(2, 23, 17, 0.0), dms(4, 19, 51, 0.0)));
        assert_stronger(&chart, Aries, Libra);
        let chart = scene(Gemini, &at(dms(2, 19, 51, 0.0), dms(4, 23, 17, 0.0)));
        assert_stronger(&chart, Libra, Aries);
    }

    #[test]
    fn rule_6_takes_the_stronger_lord_and_measures_rahu_from_the_rasis_end() {
        use Body::*;
        use Rasi::*;
        // §15.5.2 rule 6's note. Rahu at 5Ge joined by the rest is Aquarius's lord with 25° to the Sun's 20°.
        let at = [
            (Rahu, dms(2, 5, 0, 0.0)),
            (Sun, dms(2, 20, 0, 0.0)),
            (Saturn, mid(Taurus)),
        ];
        let chart = scene(Gemini, &at);
        assert_eq!(lord_of(&chart, Aquarius), Rahu);
        assert_stronger(&chart, Aquarius, Leo);
    }

    #[test]
    fn a_last_tie_goes_to_the_first_rasi_counted_from_aries() {
        use Rasi::*;
        // ADR 0012's shortcut: Mars and Venus at the same degree in Ge, nothing else to tell Ar and Li apart.
        assert_stronger(&scene(Gemini, &[]), Aries, Libra);
    }

    #[test]
    fn an_earlier_rule_wins_though_the_next_rule_says_otherwise() {
        use Body::*;
        use Rasi::*;
        // Rule 1 over 2, 2 over 3, 3 over 4 of §15.5.1.
        for (at, lord) in [
            (
                vec![
                    (Saturn, mid(Cancer)),
                    (Sun, mid(Cancer)),
                    (Rahu, mid(Libra)),
                    (Venus, mid(Taurus)),
                ],
                Saturn,
            ),
            (
                vec![(Saturn, mid(Libra)), (Rahu, mid(Aries)), (Mars, mid(Leo))],
                Rahu,
            ),
            (vec![(Saturn, mid(Libra)), (Rahu, mid(Leo))], Saturn),
        ] {
            assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), lord);
        }
        // Rule 2 over 3, 3 over 4, 4 over 5, 5 over 6 of §15.5.2.
        for (at, winner, loser) in [
            (
                vec![(Saturn, mid(Libra)), (Jupiter, mid(Aries))],
                Aries,
                Libra,
            ),
            (
                vec![(Saturn, mid(Libra)), (Moon, mid(Aries)), (Mars, mid(Virgo))],
                Libra,
                Aries,
            ),
            (vec![(Mars, mid(Virgo))], Aries, Leo),
            (vec![(Mars, dms(2, 25, 0, 0.0))], Leo, Aries),
        ] {
            assert_stronger(&scene(Gemini, &at), winner, loser);
        }
    }

    #[test]
    fn a_rule_tries_the_main_owner_then_a_seven_planet_co_lord_at_home() {
        use Body::*;
        use Rasi::*;
        let tried = |at: &[(Body, f64)], r| lords_to_try(Varga::RasiChart, &scene(Gemini, at), r);
        assert_eq!(tried(&[(Saturn, mid(Aquarius))], Aquarius), [Rahu, Saturn]);
        assert_eq!(tried(&[(Mars, mid(Scorpio))], Scorpio), [Ketu, Mars]);
        // A node at home is not tried.
        assert_eq!(tried(&[(Rahu, mid(Aquarius))], Aquarius), [Saturn]);
        assert_eq!(tried(&[(Ketu, mid(Scorpio))], Scorpio), [Mars]);
        // Neither, or both, at home: as lords_in_yoga has them.
        assert_eq!(tried(&[], Aquarius), [Saturn, Rahu]);
        let both = [(Saturn, mid(Aquarius)), (Rahu, mid(Aquarius))];
        assert_eq!(tried(&both, Aquarius), [Saturn, Rahu]);
        assert_eq!(tried(&[], Aries), [Mars]);
    }

    #[test]
    fn saturn_at_home_in_an_aquarius_lagna_can_form_a_yoga_but_rahu_at_home_cannot() {
        use Body::*;
        use Rasi::*;
        let khadga = |at: &[(Body, Rasi)]| {
            let mut at = at.to_vec();
            at.extend([(Jupiter, Libra), (Venus, Pisces)]);
            from_lagna(Aquarius, Gemini, &at, YogaName::Khadga)
        };
        // Rahu in the 3rd fails as main owner, so Saturn in the lagna forms it.
        let row = khadga(&[(Saturn, Aquarius), (Rahu, Aries)]);
        assert_eq!(present(row), [Jupiter, Venus, Saturn]);
        // Saturn in the 3rd is the only lord tried, though Rahu is in the lagna.
        assert_eq!(khadga(&[(Rahu, Aquarius), (Saturn, Aries)]), None);
        // The main owner is named when he works: Rahu in the 5th.
        let row = khadga(&[(Saturn, Aquarius), (Rahu, Gemini)]);
        assert_eq!(present(row), [Jupiter, Venus, Rahu]);
    }

    #[test]
    fn a_yogas_lord_is_the_co_lord_outside_the_rasi_or_either() {
        use Body::*;
        use Rasi::*;
        let lords = |at: &[(Body, f64)], r| lords_in_yoga(Varga::RasiChart, &scene(Gemini, at), r);
        // Parasara, p. 218, and chart 18's Rahu as 5th lord with Saturn in Aquarius (ADR 0014).
        assert_eq!(lords(&[(Saturn, mid(Aquarius))], Aquarius), [Rahu]);
        assert_eq!(lords(&[(Rahu, mid(Aquarius))], Aquarius), [Saturn]);
        assert_eq!(lords(&[], Aquarius), [Saturn, Rahu]);
        assert_eq!(
            lords(&[(Saturn, mid(Aquarius)), (Rahu, mid(Aquarius))], Aquarius),
            [Saturn, Rahu]
        );
        assert_eq!(lords(&[(Mars, mid(Scorpio))], Scorpio), [Ketu]);
        assert_eq!(lords(&[], Scorpio), [Mars, Ketu]);
        // A rasi with one lord keeps it, in the rasi or not.
        assert_eq!(lords(&[], Aries), [Mars]);
        assert_eq!(lords(&[(Mars, mid(Aries))], Aries), [Mars]);
    }

    /// A chart with Mercury in Virgo, `company` beside him, and everything else in Aquarius but for the nodes in Leo and Aquarius.
    fn with_mercury(company: &[Body]) -> [Entry; 10] {
        let mut at = vec![
            (Body::Mercury, mid(Rasi::Virgo)),
            (Body::Rahu, mid(Rasi::Aquarius)),
            (Body::Ketu, mid(Rasi::Leo)),
        ];
        at.extend(company.iter().map(|&g| (g, mid(Rasi::Virgo))));
        scene(Rasi::Aquarius, &at)
    }

    #[test]
    fn mercurys_nature_turns_on_his_company_in_the_chart_read() {
        use Body::*;
        use Disposition::*;
        // §3.2.2; a tie has no rule (ADR 0014).
        for (company, nature) in [
            (&[][..], Some(Saumya)),
            (&[Jupiter], Some(Saumya)),
            (&[Sun, Mars], Some(Kroora)),
            (&[Jupiter, Venus, Sun], Some(Saumya)),
            (&[Sun, Mars, Jupiter], Some(Kroora)),
            (&[Sun, Jupiter], None),
            // The nodes are malefics in his company.
            (&[Rahu], Some(Kroora)),
            (&[Ketu, Venus], None),
        ] {
            let chart = with_mercury(company);
            assert_eq!(
                natural_nature(Varga::RasiChart, &chart, Mercury),
                nature,
                "{company:?}"
            );
        }
    }

    #[test]
    fn mercurys_company_is_counted_in_the_varga_read() {
        use Body::*;
        // Together at Aries 1° and 5° in the rasi chart, apart in D-9: Aries and Taurus.
        let chart = scene(
            Rasi::Aquarius,
            &[(Mercury, dms(0, 5, 0, 0.0)), (Mars, dms(0, 1, 0, 0.0))],
        );
        assert_eq!(
            natural_nature(Varga::RasiChart, &chart, Mercury),
            Some(Disposition::Kroora)
        );
        assert_eq!(
            natural_nature(Varga::Navamsa, &chart, Mercury),
            Some(Disposition::Saumya)
        );
    }

    #[test]
    fn the_moon_is_saumya_in_sukla_and_kroora_in_krishna_and_counts_so_beside_mercury() {
        use Body::*;
        use Disposition::*;
        // The Moon 30° ahead of the Sun is waxing, 30° behind waning.
        let beside_mercury = |sun| {
            scene(
                Rasi::Aquarius,
                &[
                    (Sun, mid(sun)),
                    (Moon, mid(Rasi::Taurus)),
                    (Mercury, mid(Rasi::Taurus)),
                ],
            )
        };
        let waxing = beside_mercury(Rasi::Aries);
        assert_eq!(
            natural_nature(Varga::RasiChart, &waxing, Moon),
            Some(Saumya)
        );
        assert_eq!(
            natural_nature(Varga::RasiChart, &waxing, Mercury),
            Some(Saumya)
        );
        let waning = beside_mercury(Rasi::Gemini);
        assert_eq!(
            natural_nature(Varga::RasiChart, &waning, Moon),
            Some(Kroora)
        );
        assert_eq!(
            natural_nature(Varga::RasiChart, &waning, Mercury),
            Some(Kroora)
        );
    }

    #[test]
    fn the_other_natures_are_fixed_and_a_lagna_has_none() {
        use Body::*;
        use Disposition::*;
        let chart = with_mercury(&[]);
        let nature = |g| natural_nature(Varga::RasiChart, &chart, g);
        assert_eq!([Jupiter, Venus].map(nature), [Some(Saumya), Some(Saumya)]);
        assert_eq!(
            [Sun, Mars, Saturn, Rahu, Ketu].map(nature),
            [Some(Kroora); 5]
        );
        assert_eq!(nature(Lagna), None);
    }

    fn yoga_in(chart: &[Entry; 10], d: Varga, name: YogaName) -> Option<Yoga> {
        yogas(d, chart).into_iter().find(|y| y.name == name)
    }

    /// `name`'s row in the rasi chart of a scene with everything at 15° of `rest` but for `at`.
    fn yoga_of(rest: Rasi, at: &[(Body, Rasi)], name: YogaName) -> Option<Yoga> {
        let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
        yoga_in(&scene(rest, &at), Varga::RasiChart, name)
    }

    fn grahas_of(rest: Rasi, at: &[(Body, Rasi)], name: YogaName) -> Option<Vec<Body>> {
        yoga_of(rest, at, name).map(|y| y.grahas)
    }

    #[test]
    fn vesi_is_a_planet_other_than_the_moon_in_the_suns_2nd() {
        use Body::*;
        use Rasi::*;
        let at = [(Sun, Gemini), (Jupiter, Cancer), (Mercury, Cancer)];
        let vesi = yoga_of(Aquarius, &at, YogaName::Vesi).unwrap();
        assert_eq!(vesi.name.class(), YogaClass::Ravi);
        assert_eq!(vesi.grahas, [Mercury, Jupiter]);
        assert_eq!(vesi.to_judge, []);
        let at = [(Sun, Gemini), (Moon, Cancer)];
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Vesi), None);
    }

    #[test]
    fn a_node_alone_in_the_suns_2nd_or_12th_forms_nothing() {
        use Body::*;
        use Rasi::*;
        let at = [(Sun, Gemini), (Rahu, Cancer), (Ketu, Taurus)];
        for name in [YogaName::Vesi, YogaName::Vosi, YogaName::Ubhayachara] {
            assert_eq!(grahas_of(Aquarius, &at, name), None, "{name:?}");
        }
    }

    #[test]
    fn vosi_is_a_planet_other_than_the_moon_in_the_suns_12th() {
        use Body::*;
        use Rasi::*;
        let at = [(Sun, Aries), (Jupiter, Pisces), (Venus, Pisces)];
        assert_eq!(
            grahas_of(Aquarius, &at, YogaName::Vosi),
            Some(vec![Jupiter, Venus])
        );
        let at = [(Sun, Aries), (Moon, Pisces)];
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Vosi), None);
    }

    #[test]
    fn ubhayachara_replaces_vesi_and_vosi_with_planets_on_both_sides() {
        use Body::*;
        use Rasi::*;
        let at = [(Sun, Cancer), (Mars, Leo), (Venus, Gemini)];
        assert_eq!(
            grahas_of(Aquarius, &at, YogaName::Ubhayachara),
            Some(vec![Mars, Venus])
        );
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Vesi), None);
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Vosi), None);
        let at = [(Sun, Cancer), (Mars, Leo)];
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Ubhayachara), None);
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Vesi), Some(vec![Mars]));
    }

    #[test]
    fn budha_aaditya_is_the_sun_and_mercury_in_one_rasi_however_close() {
        use Body::*;
        use Rasi::*;
        let at = [(Sun, Scorpio), (Mercury, Scorpio)];
        assert_eq!(
            grahas_of(Aquarius, &at, YogaName::BudhaAaditya),
            Some(vec![Sun, Mercury])
        );
        let at = [(Sun, Scorpio), (Mercury, Libra)];
        assert_eq!(grahas_of(Aquarius, &at, YogaName::BudhaAaditya), None);
        // §11.2.4's combustion is a note on its power, not a condition (ADR 0014).
        let close = scene(
            Aquarius,
            &[(Sun, dms(7, 10, 0, 0.0)), (Mercury, dms(7, 10, 6, 0.0))],
        );
        let row = yoga_in(&close, Varga::RasiChart, YogaName::BudhaAaditya).unwrap();
        assert_eq!(row.to_judge, []);
    }

    #[test]
    fn a_yoga_in_a_varga_reads_the_vargas_places() {
        use Body::*;
        use Rasi::*;
        // Aries 1° and 5° share Aries in the rasi chart, not D-9: Aries and Taurus.
        let split = scene(
            Aquarius,
            &[(Sun, dms(0, 1, 0, 0.0)), (Mercury, dms(0, 5, 0, 0.0))],
        );
        assert!(yoga_in(&split, Varga::RasiChart, YogaName::BudhaAaditya).is_some());
        assert_eq!(
            yoga_in(&split, Varga::Navamsa, YogaName::BudhaAaditya),
            None
        );
        // Aries 1° and Leo 1° each begin a fiery rasi's navamsas, at Aries.
        let joined = scene(
            Aquarius,
            &[(Sun, dms(0, 1, 0, 0.0)), (Mercury, dms(4, 1, 0, 0.0))],
        );
        assert_eq!(
            yoga_in(&joined, Varga::RasiChart, YogaName::BudhaAaditya),
            None
        );
        assert!(yoga_in(&joined, Varga::Navamsa, YogaName::BudhaAaditya).is_some());
    }

    #[test]
    fn sunaphaa_is_a_planet_other_than_the_sun_in_the_moons_2nd() {
        use Body::*;
        use Rasi::*;
        let at = [(Moon, Gemini), (Jupiter, Cancer), (Mercury, Cancer)];
        let row = yoga_of(Aquarius, &at, YogaName::Sunaphaa).unwrap();
        assert_eq!(row.name.class(), YogaClass::Chandra);
        assert_eq!(row.grahas, [Mercury, Jupiter]);
        for other in [Sun, Rahu] {
            let at = [(Moon, Gemini), (other, Cancer)];
            assert_eq!(grahas_of(Aquarius, &at, YogaName::Sunaphaa), None);
        }
    }

    #[test]
    fn anaphaa_is_a_planet_other_than_the_sun_in_the_moons_12th() {
        use Body::*;
        use Rasi::*;
        let at = [(Moon, Aries), (Jupiter, Pisces), (Venus, Pisces)];
        assert_eq!(
            grahas_of(Aquarius, &at, YogaName::Anaphaa),
            Some(vec![Jupiter, Venus])
        );
        for other in [Sun, Ketu] {
            let at = [(Moon, Aries), (other, Pisces)];
            assert_eq!(grahas_of(Aquarius, &at, YogaName::Anaphaa), None);
        }
    }

    #[test]
    fn duradhara_replaces_sunaphaa_and_anaphaa_with_planets_on_both_sides() {
        use Body::*;
        use Rasi::*;
        let at = [(Moon, Cancer), (Mars, Leo), (Venus, Gemini)];
        assert_eq!(
            grahas_of(Aquarius, &at, YogaName::Duradhara),
            Some(vec![Mars, Venus])
        );
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Sunaphaa), None);
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Anaphaa), None);
        let at = [(Moon, Cancer), (Venus, Gemini)];
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Duradhara), None);
        assert_eq!(
            grahas_of(Aquarius, &at, YogaName::Anaphaa),
            Some(vec![Venus])
        );
    }

    /// Kemadruma's row with the book's example, a Taurus lagna and the Moon in Virgo, everything else in Capricorn but for `more`.
    fn kemadruma_with(more: &[(Body, Rasi)]) -> Option<Vec<Body>> {
        let mut at = vec![(Body::Lagna, Rasi::Taurus), (Body::Moon, Rasi::Virgo)];
        at.extend_from_slice(more);
        grahas_of(Rasi::Capricorn, &at, YogaName::Kemadruma)
    }

    #[test]
    fn kemadruma_is_no_planet_beside_the_moon_nor_in_a_kendra_from_the_lagna() {
        use Body::*;
        use Rasi::*;
        assert_eq!(kemadruma_with(&[]), Some(vec![]));
        // The Sun does not count beside the Moon, but does in a kendra.
        assert_eq!(kemadruma_with(&[(Sun, Libra)]), Some(vec![]));
        assert_eq!(kemadruma_with(&[(Sun, Taurus)]), None);
        // Mars in the Moon's 1st, 2nd and 12th, and in the other kendras.
        for rasi in [Virgo, Libra, Leo, Taurus, Scorpio, Aquarius] {
            assert_eq!(kemadruma_with(&[(Mars, rasi)]), None, "{rasi:?}");
        }
        // The Moon itself in a kendra counts for nothing.
        let at = [(Lagna, Taurus), (Moon, Leo)];
        assert_eq!(grahas_of(Capricorn, &at, YogaName::Kemadruma), Some(vec![]));
    }

    #[test]
    fn a_node_counts_in_none_of_kemadrumas_checks() {
        use Body::*;
        use Rasi::*;
        // In a kendra, then beside the Moon (ADR 0014).
        assert_eq!(
            kemadruma_with(&[(Rahu, Taurus), (Ketu, Scorpio)]),
            Some(vec![])
        );
        assert_eq!(
            kemadruma_with(&[(Rahu, Libra), (Ketu, Virgo)]),
            Some(vec![])
        );
    }

    #[test]
    fn chandra_mangala_is_the_moon_and_mars_in_one_rasi() {
        use Body::*;
        use Rasi::*;
        let row = yoga_of(
            Aquarius,
            &[(Moon, Virgo), (Mars, Virgo)],
            YogaName::ChandraMangala,
        );
        assert_eq!(row.map(|y| y.grahas), Some(vec![Moon, Mars]));
        let at = [(Moon, Virgo), (Mars, Libra)];
        assert_eq!(grahas_of(Aquarius, &at, YogaName::ChandraMangala), None);
    }

    /// Adhi's row with the Moon in Taurus and Jupiter and Venus at the given rasis.
    fn adhi_with(jupiter: Rasi, venus: Rasi, more: &[(Body, Rasi)]) -> Option<Yoga> {
        let mut at = vec![
            (Body::Moon, Rasi::Taurus),
            (Body::Jupiter, jupiter),
            (Body::Venus, venus),
        ];
        at.extend_from_slice(more);
        yoga_of(Rasi::Aquarius, &at, YogaName::Adhi)
    }

    #[test]
    fn adhi_is_two_or_more_natural_benefics_in_the_moons_6th_7th_and_8th() {
        use Body::*;
        use Rasi::*;
        // All three: across the places, then all in one (ADR 0014).
        let at = [(Mercury, Sagittarius)];
        let row = adhi_with(Libra, Scorpio, &at).unwrap();
        assert_eq!(row.name.class(), YogaClass::Chandra);
        assert_eq!(row.grahas, [Mercury, Jupiter, Venus]);
        assert_eq!(row.to_judge, []);
        let row = adhi_with(Libra, Libra, &[(Mercury, Libra)]).unwrap();
        assert_eq!(row.grahas, [Mercury, Jupiter, Venus]);
        // Any two, with the third outside: a saumya Mercury in the 11th and the 9th, then Venus and Jupiter outside.
        for rasi in [Pisces, Capricorn] {
            let row = adhi_with(Libra, Scorpio, &[(Mercury, rasi)]).unwrap();
            assert_eq!(row.grahas, [Jupiter, Venus]);
            assert_eq!(row.to_judge, []);
        }
        let row = adhi_with(Libra, Cancer, &at).unwrap();
        assert_eq!(row.grahas, [Mercury, Jupiter]);
        let row = adhi_with(Capricorn, Scorpio, &at).unwrap();
        assert_eq!(row.grahas, [Mercury, Venus]);
        // One alone is not enough: Mercury, Jupiter, Venus, with a kroora Mercury left in Aquarius for the last two.
        assert_eq!(adhi_with(Cancer, Pisces, &at), None);
        assert_eq!(adhi_with(Libra, Cancer, &[]), None);
        assert_eq!(adhi_with(Cancer, Scorpio, &[]), None);
        // The rule, not its example's misprinted 5th and 4th (ADR 0014).
        assert_eq!(adhi_with(Virgo, Leo, &[(Mercury, Virgo)]), None);
    }

    #[test]
    fn adhi_leaves_out_a_kroora_mercury() {
        use Body::*;
        use Rasi::*;
        let beside_mars = [(Mercury, Scorpio), (Mars, Scorpio)];
        // Jupiter and Venus alone form it, with him in the places or among the malefics of Aquarius outside them (ADR 0014).
        for more in [&beside_mars[..], &[]] {
            let row = adhi_with(Libra, Sagittarius, more).unwrap();
            assert_eq!(row.grahas, [Jupiter, Venus]);
            assert_eq!(row.to_judge, []);
        }
        // He does not stand in for either of them.
        assert_eq!(adhi_with(Libra, Cancer, &beside_mars), None);
        assert_eq!(adhi_with(Capricorn, Cancer, &beside_mars), None);
    }

    #[test]
    fn a_kroora_mercury_outside_adhi_keeps_his_bhadra() {
        use Body::*;
        use Rasi::*;
        let at = [
            (Lagna, Gemini),
            (Moon, Pisces),
            (Jupiter, Leo),
            (Mercury, Virgo),
            (Mars, Virgo),
            (Sun, Libra),
            (Venus, Libra),
            (Saturn, Libra),
        ];
        assert_eq!(
            grahas_of(Aquarius, &at, YogaName::Adhi),
            Some(vec![Jupiter, Venus])
        );
        assert_eq!(
            grahas_of(Aquarius, &at, YogaName::Bhadra),
            Some(vec![Mercury])
        );
    }

    #[test]
    fn adhi_is_to_judge_only_if_a_tied_mercury_is_the_second_benefic() {
        use Body::*;
        use Rasi::*;
        // Tied in the places with Jupiter or Venus beside him and the other outside: it holds only if he is saumya.
        let row = adhi_with(Libra, Cancer, &[(Mercury, Libra), (Sun, Libra)]).unwrap();
        assert_eq!(row.grahas, [Mercury, Jupiter]);
        assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
        let row = adhi_with(Cancer, Libra, &[(Mercury, Libra), (Sun, Libra)]).unwrap();
        assert_eq!(row.grahas, [Mercury, Venus]);
        assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
        // With both in the places, it holds both ways, and the saumya reading lists him.
        let row = adhi_with(Libra, Scorpio, &[(Mercury, Libra), (Sun, Libra)]).unwrap();
        assert_eq!(row.grahas, [Mercury, Jupiter, Venus]);
        assert_eq!(row.to_judge, []);
        // Tied outside them by the waxing Moon and Mars: it holds both ways without him.
        let more = [(Sun, Aries), (Mars, Taurus), (Mercury, Taurus)];
        let row = adhi_with(Libra, Scorpio, &more).unwrap();
        assert_eq!(row.grahas, [Jupiter, Venus]);
        assert_eq!(row.to_judge, []);
    }

    #[test]
    fn a_yogas_class_is_the_section_of_ch_11_listing_it() {
        use YogaClass as C;
        use YogaName as N;
        for (name, class) in [
            (N::Vesi, C::Ravi),
            (N::BudhaAaditya, C::Ravi),
            (N::Sunaphaa, C::Chandra),
            (N::Adhi, C::Chandra),
            (N::Ruchaka, C::Mahaapurusha),
            (N::Hamsa, C::Mahaapurusha),
            (N::Rajju, C::Naabhasa),
            (N::Vajra, C::Naabhasa),
            (N::Chaapa, C::Naabhasa),
            (N::Gola, C::Naabhasa),
            (N::Subha, C::OtherPopular),
            (N::Ravi, C::OtherPopular),
            (N::Chapa, C::OtherPopular),
            (N::Vimala, C::OtherPopular),
            (N::BasicRaaja, C::Raaja),
            (N::VipareetaRaaja, C::Raaja),
            (N::Raaja(1), C::Raaja),
            (N::RaajaSambandha(15), C::RaajaSambandha),
            (N::Dhana(12), C::Dhana),
            (N::Daridra(13), C::Daridra),
        ] {
            assert_eq!(name.class(), class, "{name:?}");
        }
    }

    #[test]
    fn a_rule_differing_on_a_tied_mercury_is_to_judge_whichever_way_it_holds() {
        use Disposition::{Kroora, Saumya};
        let tied = with_mercury(&[Body::Sun, Body::Jupiter]);
        let each_way = |rule: fn(Disposition) -> Option<Vec<Body>>| {
            with_mercury_each_way(Varga::RasiChart, &tied, YogaName::Maalaa, rule)
                .map(|y| (y.grahas, y.to_judge))
        };
        let judged = Some((vec![Body::Mercury], vec![ToJudge::MercurysNature]));
        assert_eq!(
            each_way(|m| (m == Saumya).then(|| vec![Body::Mercury])),
            judged
        );
        assert_eq!(
            each_way(|m| (m == Kroora).then(|| vec![Body::Mercury])),
            judged
        );
        assert_eq!(each_way(|_| None), None);
        // Agreeing on the yoga but not its grahas, the saumya reading's stand (ADR 0014).
        assert_eq!(
            each_way(|m| Some(vec![if m == Saumya {
                Body::Mercury
            } else {
                Body::Sun
            }])),
            Some((vec![Body::Mercury], vec![]))
        );
        // Off a tie, only his nature is read.
        let kroora = with_mercury(&[Body::Sun, Body::Mars]);
        let row = with_mercury_each_way(Varga::RasiChart, &kroora, YogaName::Maalaa, |m| {
            (m == Kroora).then(|| vec![Body::Mercury])
        })
        .unwrap();
        assert_eq!(row.grahas, [Body::Mercury]);
        assert_eq!(row.to_judge, []);
    }

    #[test]
    fn mahaapurusha_yogas_are_a_planet_in_own_or_uchcha_in_a_kendra_from_the_lagna() {
        use Body::*;
        use Rasi::*;
        // §11.4.1–11.4.5's examples.
        for (name, lagna, graha, rasi) in [
            (YogaName::Ruchaka, Libra, Mars, Aries),
            (YogaName::Bhadra, Gemini, Mercury, Virgo),
            (YogaName::Sasa, Capricorn, Saturn, Libra),
            (YogaName::Maalavya, Gemini, Venus, Pisces),
            (YogaName::Hamsa, Sagittarius, Jupiter, Pisces),
        ] {
            let row = yoga_of(Gemini, &[(Lagna, lagna), (graha, rasi)], name).unwrap();
            assert_eq!(row.name.class(), YogaClass::Mahaapurusha);
            assert_eq!(row.grahas, [graha]);
        }
        // The rest of each planet's own and uchcha rasis.
        for (name, lagna, graha, rasi) in [
            (YogaName::Ruchaka, Libra, Mars, Capricorn),
            (YogaName::Ruchaka, Taurus, Mars, Scorpio),
            (YogaName::Bhadra, Gemini, Mercury, Gemini),
            (YogaName::Sasa, Aries, Saturn, Capricorn),
            (YogaName::Sasa, Aquarius, Saturn, Aquarius),
            (YogaName::Maalavya, Taurus, Venus, Taurus),
            (YogaName::Maalavya, Cancer, Venus, Libra),
            (YogaName::Hamsa, Aries, Jupiter, Cancer),
            (YogaName::Hamsa, Virgo, Jupiter, Sagittarius),
        ] {
            let at = [(Lagna, lagna), (graha, rasi)];
            assert_eq!(grahas_of(Gemini, &at, name), Some(vec![graha]), "{name:?}");
        }
        // For each, its own or uchcha rasi outside a kendra, then a kendra outside them.
        for (name, lagna, graha, rasi) in [
            (YogaName::Ruchaka, Gemini, Mars, Aries),
            (YogaName::Ruchaka, Libra, Mars, Cancer),
            (YogaName::Bhadra, Taurus, Mercury, Gemini),
            (YogaName::Bhadra, Gemini, Mercury, Sagittarius),
            (YogaName::Sasa, Gemini, Saturn, Aquarius),
            (YogaName::Sasa, Capricorn, Saturn, Aries),
            (YogaName::Maalavya, Gemini, Venus, Libra),
            (YogaName::Maalavya, Gemini, Venus, Virgo),
            (YogaName::Hamsa, Gemini, Jupiter, Cancer),
            (YogaName::Hamsa, Cancer, Jupiter, Capricorn),
        ] {
            let at = [(Lagna, lagna), (graha, rasi)];
            assert_eq!(grahas_of(Gemini, &at, name), None, "{name:?} {rasi:?}");
        }
    }

    #[test]
    fn a_mahaapurusha_yoga_in_a_varga_counts_from_that_vargas_lagna() {
        use Body::*;
        // Pisces 1° is Cancer in D-9 and Aries 1° is Aries: the 2nd in the rasi chart, the 10th in D-9.
        let chart = scene(
            Rasi::Gemini,
            &[(Lagna, dms(11, 1, 0, 0.0)), (Mars, dms(0, 1, 0, 0.0))],
        );
        assert_eq!(yoga_in(&chart, Varga::RasiChart, YogaName::Ruchaka), None);
        assert!(yoga_in(&chart, Varga::Navamsa, YogaName::Ruchaka).is_some());
    }

    /// The Naabhasa rows of a chart with its lagna in Aries, the seven planets in these houses in the book's order, and the nodes in `nodes`.
    fn naabhasa_rows(houses: [u8; 7], nodes: u8) -> Vec<Yoga> {
        let house = |h: u8| mid(kth(Rasi::Aries, h as usize));
        let mut at = vec![
            (Body::Lagna, mid(Rasi::Aries)),
            (Body::Rahu, house(nodes)),
            (Body::Ketu, house(nodes)),
        ];
        at.extend(PLANETS.into_iter().zip(houses.map(house)));
        yogas(Varga::RasiChart, &scene(Rasi::Aries, &at))
            .into_iter()
            .filter(|y| y.name.class() == YogaClass::Naabhasa)
            .collect()
    }

    fn naabhasa_names(houses: [u8; 7], nodes: u8) -> Vec<YogaName> {
        naabhasa_rows(houses, nodes)
            .into_iter()
            .map(|y| y.name)
            .collect()
    }

    /// A house none of the seven planets is in, so the nodes stand outside the shape: Rahu and Ketu are not among "all the planets" (ADR 0014).
    fn empty_house(houses: [u8; 7]) -> u8 {
        (1..=12).find(|h| !houses.contains(h)).unwrap()
    }

    #[test]
    fn aasraya_yogas_are_all_seven_in_movable_fixed_or_dual_rasis() {
        use YogaName::*;
        // §11.5.1. From an Aries lagna, the 1st, 4th, 7th and 10th are chara, the next sthira, the last dwiswabhava.
        for (name, houses, broken) in [
            (Rajju, [1, 4, 7, 10, 1, 4, 7], [1, 4, 7, 10, 1, 4, 2]),
            (Musala, [2, 5, 8, 11, 2, 5, 8], [2, 5, 8, 11, 2, 5, 3]),
            (Nala, [3, 6, 9, 12, 3, 6, 9], [3, 6, 9, 12, 3, 6, 1]),
        ] {
            let row = naabhasa_rows(houses, empty_house(houses))
                .into_iter()
                .find(|y| y.name == name)
                .unwrap();
            assert_eq!(row.grahas, PLANETS);
            assert_eq!(row.to_judge, []);
            assert!(!naabhasa_names(broken, empty_house(broken)).contains(&name));
        }
    }

    #[test]
    fn aakriti_shapes_hold_all_seven_with_every_place_occupied() {
        use YogaName::*;
        // §11.5.3, each with the charts forming it, then the charts that do not.
        type Charts = &'static [[u8; 7]];
        let cases: [(YogaName, Charts, Charts); 16] = [
            (
                Gadaa,
                &[
                    [1, 1, 1, 1, 4, 4, 4],
                    [4, 4, 4, 7, 7, 7, 7],
                    [7, 7, 7, 10, 10, 10, 10],
                    [10, 10, 10, 1, 1, 1, 1],
                ],
                &[[1; 7], [1, 1, 1, 1, 4, 4, 7], [1, 1, 1, 7, 7, 7, 7]],
            ),
            (
                Sakata,
                &[[1, 1, 1, 1, 7, 7, 7]],
                &[[1; 7], [1, 1, 1, 7, 7, 7, 4]],
            ),
            (
                Vihanga,
                &[[4, 4, 4, 10, 10, 10, 10]],
                &[[4; 7], [4, 4, 4, 10, 10, 10, 1]],
            ),
            (
                Sringaataka,
                &[[1, 1, 1, 5, 5, 9, 9]],
                &[[1, 1, 1, 1, 5, 5, 5], [1, 1, 5, 5, 9, 9, 2]],
            ),
            (
                Hala,
                &[
                    [2, 2, 2, 6, 6, 10, 10],
                    [3, 3, 3, 7, 7, 11, 11],
                    [4, 4, 4, 8, 8, 12, 12],
                ],
                // Trikonas from the lagna are Sringaataka's.
                &[
                    [1, 1, 1, 5, 5, 9, 9],
                    [2, 2, 2, 2, 6, 6, 6],
                    [2, 2, 6, 6, 10, 10, 3],
                ],
            ),
            (
                Yoopa,
                &[[1, 2, 3, 4, 1, 2, 3]],
                // §11.5.3 with the 4th left empty, then a planet outside.
                &[[1, 1, 1, 2, 2, 3, 3], [1, 2, 3, 4, 1, 2, 5]],
            ),
            (
                Sara,
                &[[4, 5, 6, 7, 4, 5, 6]],
                &[[4, 4, 4, 5, 5, 6, 6], [4, 5, 6, 7, 4, 5, 8]],
            ),
            (
                Sakti,
                &[[7, 8, 9, 10, 7, 8, 9]],
                &[[7, 7, 7, 8, 8, 9, 9], [7, 8, 9, 10, 7, 8, 11]],
            ),
            (
                Danda,
                &[[10, 11, 12, 1, 10, 11, 12]],
                &[[10, 10, 10, 11, 11, 12, 12], [10, 11, 12, 1, 10, 11, 2]],
            ),
            (
                Naukaa,
                &[[1, 2, 3, 4, 5, 6, 7]],
                &[[1, 1, 2, 3, 4, 5, 6], [1, 2, 3, 4, 5, 6, 8]],
            ),
            (
                Koota,
                &[[4, 5, 6, 7, 8, 9, 10]],
                &[[4, 4, 5, 6, 7, 8, 9], [4, 5, 6, 7, 8, 9, 11]],
            ),
            (
                Chatra,
                &[[7, 8, 9, 10, 11, 12, 1]],
                &[[7, 7, 8, 9, 10, 11, 12], [7, 8, 9, 10, 11, 12, 2]],
            ),
            (
                Chaapa,
                &[[10, 11, 12, 1, 2, 3, 4]],
                &[[10, 10, 11, 12, 1, 2, 3], [10, 11, 12, 1, 2, 3, 5]],
            ),
            (
                Ardhachandra,
                // The seven rasis from each panaphara, then each apoklima.
                &[
                    [2, 3, 4, 5, 6, 7, 8],
                    [5, 6, 7, 8, 9, 10, 11],
                    [8, 9, 10, 11, 12, 1, 2],
                    [11, 12, 1, 2, 3, 4, 5],
                    [3, 4, 5, 6, 7, 8, 9],
                    [6, 7, 8, 9, 10, 11, 12],
                    [9, 10, 11, 12, 1, 2, 3],
                    [12, 1, 2, 3, 4, 5, 6],
                ],
                // From a kendra are the four above; six of the seven rasis.
                &[
                    [1, 2, 3, 4, 5, 6, 7],
                    [4, 5, 6, 7, 8, 9, 10],
                    [7, 8, 9, 10, 11, 12, 1],
                    [10, 11, 12, 1, 2, 3, 4],
                    [2, 2, 3, 4, 5, 6, 7],
                ],
            ),
            (
                Chakra,
                &[[1, 3, 5, 7, 9, 11, 1]],
                &[[1, 3, 5, 7, 9, 1, 1], [1, 3, 5, 7, 9, 11, 2]],
            ),
            (
                Samudra,
                &[[2, 4, 6, 8, 10, 12, 2]],
                &[[2, 4, 6, 8, 10, 2, 2], [2, 4, 6, 8, 10, 12, 1]],
            ),
        ];
        for (name, forming, not_forming) in cases {
            for &houses in forming {
                let row = naabhasa_rows(houses, empty_house(houses))
                    .into_iter()
                    .find(|y| y.name == name);
                let row = row.unwrap_or_else(|| panic!("{name:?} {houses:?}"));
                assert_eq!(row.grahas, PLANETS);
            }
            for &houses in not_forming {
                let names = naabhasa_names(houses, empty_house(houses));
                assert!(!names.contains(&name), "{name:?} {houses:?}");
            }
        }
    }

    #[test]
    fn kamala_and_vaapi_only_need_all_seven_within_their_places() {
        use YogaName::*;
        let has = |houses, name| naabhasa_names(houses, empty_house(houses)).contains(&name);
        // §11.5.3 says "in", so one kendra is enough (ADR 0014).
        for houses in [[1; 7], [1, 1, 1, 4, 4, 4, 4], [1, 4, 7, 10, 1, 4, 7]] {
            assert!(has(houses, Kamala), "{houses:?}");
        }
        assert!(!has([1, 4, 7, 10, 1, 4, 2], Kamala));
        for houses in [[2; 7], [2, 5, 8, 11, 2, 5, 8], [3, 6, 9, 12, 3, 6, 9]] {
            assert!(has(houses, Vaapi), "{houses:?}");
        }
        // All in panapharas or all in apoklimas, not a mix of the two (ADR 0014).
        assert!(!has([2, 2, 2, 2, 3, 3, 3], Vaapi));
        assert!(!has([2, 5, 8, 11, 2, 5, 1], Vaapi));
    }

    #[test]
    fn a_chart_may_show_more_than_one_shape() {
        use YogaName::*;
        let houses = [1, 1, 1, 1, 7, 7, 7];
        // Both kendras are chara from an Aries lagna, so its Aasraya yoga lists too.
        assert_eq!(naabhasa_names(houses, 2), [Rajju, Sakata, Kamala]);
    }

    #[test]
    fn maalaa_is_three_kendras_held_by_saumya_grahas() {
        use Body::*;
        use Rasi::*;
        // §11.5.2's example.
        let example = [
            (Lagna, Aries),
            (Jupiter, Cancer),
            (Venus, Capricorn),
            (Mercury, Libra),
        ];
        let row = yoga_of(Taurus, &example, YogaName::Maalaa).unwrap();
        assert_eq!(row.grahas, [Mercury, Jupiter, Venus]);
        assert_eq!(row.to_judge, []);
        let at = [(Lagna, Aries), (Jupiter, Cancer), (Venus, Capricorn)];
        assert_eq!(yoga_of(Taurus, &at, YogaName::Maalaa), None);
        let at = [
            (Lagna, Aries),
            (Jupiter, Cancer),
            (Venus, Capricorn),
            (Mercury, Libra),
            (Mars, Libra),
        ];
        assert_eq!(yoga_of(Taurus, &at, YogaName::Maalaa), None);
        // The Moon counts by paksha: waxing ahead of the Sun in Pisces, waning behind him in Taurus.
        let with_moon = |sun| {
            [
                (Lagna, Aries),
                (Moon, Aries),
                (Jupiter, Cancer),
                (Venus, Capricorn),
                (Sun, sun),
            ]
        };
        assert_eq!(
            grahas_of(Taurus, &with_moon(Pisces), YogaName::Maalaa),
            Some(vec![Moon, Jupiter, Venus])
        );
        assert_eq!(
            grahas_of(Taurus, &with_moon(Taurus), YogaName::Maalaa),
            None
        );
    }

    #[test]
    fn sarpa_is_three_kendras_held_by_kroora_grahas_nodes_among_them() {
        use Body::*;
        use Rasi::*;
        // §11.5.2's example.
        let example = [
            (Lagna, Scorpio),
            (Mars, Taurus),
            (Rahu, Leo),
            (Ketu, Aquarius),
        ];
        let row = yoga_of(Sagittarius, &example, YogaName::Sarpa).unwrap();
        assert_eq!(row.grahas, [Mars, Rahu, Ketu]);
        assert_eq!(row.to_judge, []);
        let at = [(Lagna, Scorpio), (Rahu, Leo), (Ketu, Aquarius)];
        assert_eq!(yoga_of(Sagittarius, &at, YogaName::Sarpa), None);
    }

    #[test]
    fn the_opposite_nature_in_any_of_the_four_kendras_may_stop_a_dala_yoga_operating_well() {
        use Body::*;
        use Rasi::*;
        let judged = [ToJudge::MayNotOperateWell];
        // §11.5.2: Saturn in the Maalaa's 4th, then in the kendra it leaves empty, then Rahu there.
        for (graha, rasi) in [(Saturn, Cancer), (Saturn, Aries), (Rahu, Aries)] {
            let at = [
                (Lagna, Aries),
                (Jupiter, Cancer),
                (Venus, Capricorn),
                (Mercury, Libra),
                (graha, rasi),
            ];
            let row = yoga_of(Taurus, &at, YogaName::Maalaa).unwrap();
            assert_eq!(row.to_judge, judged, "{graha:?} {rasi:?}");
            assert_eq!(row.grahas, [Mercury, Jupiter, Venus]);
        }
        // Jupiter in the Sarpa's 7th, then in the kendra it leaves empty, then Mercury there, saumya alone with nothing else to trip the note.
        for (graha, rasi) in [(Jupiter, Taurus), (Jupiter, Scorpio), (Mercury, Scorpio)] {
            let at = [
                (Lagna, Scorpio),
                (Mars, Taurus),
                (Rahu, Leo),
                (Ketu, Aquarius),
                (graha, rasi),
            ];
            let row = yoga_of(Sagittarius, &at, YogaName::Sarpa).unwrap();
            assert_eq!(row.to_judge, judged, "{graha:?} {rasi:?}");
        }
    }

    #[test]
    fn a_tied_mercury_in_a_dala_kendra_leaves_only_the_opposite_nature_to_judge() {
        use Body::*;
        use Rasi::*;
        // Tied beside the Sun and Jupiter, with a waxing Moon: the saumya reading lists him (ADR 0014).
        let at = [
            (Lagna, Aries),
            (Sun, Cancer),
            (Jupiter, Cancer),
            (Mercury, Cancer),
            (Venus, Capricorn),
            (Moon, Libra),
        ];
        let row = yoga_of(Taurus, &at, YogaName::Maalaa).unwrap();
        assert_eq!(row.grahas, [Moon, Mercury, Jupiter, Venus]);
        assert_eq!(row.to_judge, [ToJudge::MayNotOperateWell]);
        // Tied beside Ketu and Jupiter: the same saumya reading leaves him out of the Sarpa.
        let at = [
            (Lagna, Scorpio),
            (Mars, Taurus),
            (Rahu, Leo),
            (Ketu, Aquarius),
            (Jupiter, Aquarius),
            (Mercury, Aquarius),
        ];
        let row = yoga_of(Sagittarius, &at, YogaName::Sarpa).unwrap();
        assert_eq!(row.grahas, [Mars, Rahu, Ketu]);
        assert_eq!(row.to_judge, [ToJudge::MayNotOperateWell]);
    }

    #[test]
    fn vajra_and_yava_need_each_named_house_to_hold_its_nature() {
        use Body::*;
        use Rasi::*;
        // §11.5.3: saumya grahas in the 1st and 7th with kroora ones in the 4th and 10th, then the other way round.
        let vajra = [
            (Lagna, Aries),
            (Jupiter, Aries),
            (Venus, Libra),
            (Sun, Cancer),
            (Saturn, Capricorn),
        ];
        assert_eq!(
            grahas_of(Taurus, &vajra, YogaName::Vajra),
            Some(vec![Sun, Jupiter, Venus, Saturn])
        );
        assert_eq!(grahas_of(Taurus, &vajra, YogaName::Yava), None);
        let yava = [
            (Lagna, Aries),
            (Sun, Aries),
            (Saturn, Libra),
            (Jupiter, Cancer),
            (Venus, Capricorn),
        ];
        assert_eq!(
            grahas_of(Taurus, &yava, YogaName::Yava),
            Some(vec![Sun, Jupiter, Venus, Saturn])
        );
        assert_eq!(grahas_of(Taurus, &yava, YogaName::Vajra), None);
        // The nodes are kroora (ADR 0014).
        let at = [
            (Lagna, Aries),
            (Jupiter, Aries),
            (Venus, Libra),
            (Rahu, Cancer),
            (Ketu, Capricorn),
        ];
        assert_eq!(
            grahas_of(Taurus, &at, YogaName::Vajra),
            Some(vec![Jupiter, Venus, Rahu, Ketu])
        );
        // A house without its nature breaks it; another graha beside it does not, and is not listed (ADR 0014).
        let at = [
            (Lagna, Aries),
            (Jupiter, Aries),
            (Venus, Libra),
            (Sun, Cancer),
        ];
        assert_eq!(grahas_of(Taurus, &at, YogaName::Vajra), None);
        let mut at = vajra.to_vec();
        at.push((Mars, Aries));
        assert_eq!(
            grahas_of(Taurus, &at, YogaName::Vajra),
            Some(vec![Sun, Jupiter, Venus, Saturn])
        );
    }

    #[test]
    fn sankhya_yogas_count_the_rasis_of_the_seven_planets() {
        use YogaName::*;
        // §11.5.4, each the only Naabhasa row of its chart, the nodes not counted.
        for (name, houses) in [
            (Veenaa, [2, 3, 5, 6, 8, 9, 12]),
            (Daama, [2, 2, 3, 5, 6, 8, 9]),
            (Paasa, [2, 2, 3, 3, 5, 6, 8]),
            (Kedaara, [2, 2, 2, 3, 3, 5, 6]),
            (Soola, [2, 2, 2, 3, 3, 3, 5]),
            (Yuga, [2, 2, 2, 2, 3, 3, 3]),
        ] {
            assert_eq!(naabhasa_names(houses, 11), [name]);
            assert_eq!(naabhasa_rows(houses, 11)[0].grahas, PLANETS);
        }
    }

    #[test]
    fn a_kroora_mercury_forms_sarpa_vajra_and_yava_and_is_listed() {
        use Body::*;
        use Rasi::*;
        // Beside the Sun alone he is kroora (§3.2.2).
        let at = [
            (Lagna, Scorpio),
            (Mars, Taurus),
            (Rahu, Leo),
            (Sun, Aquarius),
            (Mercury, Aquarius),
        ];
        let row = yoga_of(Sagittarius, &at, YogaName::Sarpa).unwrap();
        assert_eq!(row.grahas, [Sun, Mars, Mercury, Rahu]);
        assert_eq!(row.to_judge, []);
        let at = [
            (Lagna, Aries),
            (Jupiter, Aries),
            (Venus, Libra),
            (Sun, Cancer),
            (Mercury, Cancer),
            (Saturn, Capricorn),
        ];
        assert_eq!(
            grahas_of(Taurus, &at, YogaName::Vajra),
            Some(vec![Sun, Mercury, Jupiter, Venus, Saturn])
        );
        let at = [
            (Lagna, Aries),
            (Sun, Aries),
            (Mercury, Aries),
            (Saturn, Libra),
            (Jupiter, Cancer),
            (Venus, Capricorn),
        ];
        assert_eq!(
            grahas_of(Taurus, &at, YogaName::Yava),
            Some(vec![Sun, Mercury, Jupiter, Venus, Saturn])
        );
    }

    #[test]
    fn a_waning_moon_is_kroora_in_a_dala_yoga() {
        use Body::*;
        use Rasi::*;
        // Behind the Sun in Taurus she is the opposite nature on a Maalaa; ahead of him in Pisces she is one of its grahas.
        let maalaa = |sun| {
            [
                (Lagna, Aries),
                (Jupiter, Cancer),
                (Venus, Capricorn),
                (Mercury, Libra),
                (Moon, Aries),
                (Sun, sun),
            ]
        };
        let row = yoga_of(Taurus, &maalaa(Taurus), YogaName::Maalaa).unwrap();
        assert_eq!(row.grahas, [Mercury, Jupiter, Venus]);
        assert_eq!(row.to_judge, [ToJudge::MayNotOperateWell]);
        let row = yoga_of(Taurus, &maalaa(Pisces), YogaName::Maalaa).unwrap();
        assert_eq!(row.grahas, [Moon, Mercury, Jupiter, Venus]);
        assert_eq!(row.to_judge, []);
        // She holds the Sarpa's third kendra only waning, behind the Sun in Pisces and not ahead of him in Sagittarius.
        let sarpa = |sun| {
            [
                (Lagna, Scorpio),
                (Mars, Taurus),
                (Rahu, Leo),
                (Moon, Aquarius),
                (Sun, sun),
            ]
        };
        let row = yoga_of(Sagittarius, &sarpa(Pisces), YogaName::Sarpa).unwrap();
        assert_eq!(row.grahas, [Moon, Mars, Rahu]);
        assert_eq!(row.to_judge, []);
        assert_eq!(
            yoga_of(Sagittarius, &sarpa(Sagittarius), YogaName::Sarpa),
            None
        );
    }

    #[test]
    fn another_naabhasa_yoga_stops_each_sankhya_yoga_its_count_would_give() {
        use YogaName::*;
        // §11.5.4, each chart with that many rasis forming the other yoga.
        for (sankhya, other, houses) in [
            (Veenaa, Naukaa, [1, 2, 3, 4, 5, 6, 7]),
            (Daama, Chakra, [1, 3, 5, 7, 9, 11, 1]),
            (Paasa, Sarpa, [1, 2, 4, 2, 3, 3, 7]),
            (Kedaara, Yoopa, [1, 2, 3, 4, 1, 2, 3]),
            (Soola, Sringaataka, [1, 1, 1, 5, 5, 9, 9]),
            (Yuga, Sakata, [1, 1, 1, 1, 7, 7, 7]),
        ] {
            assert_eq!(
                sankhya_yoga(houses.map(|h| kth(Rasi::Aries, h as usize))),
                sankhya
            );
            let names = naabhasa_names(houses, 11);
            assert!(names.contains(&other), "{other:?}");
            assert!(!names.contains(&sankhya), "{sankhya:?}");
        }
    }

    #[test]
    fn gola_is_the_count_of_one_rasi_but_its_aasraya_yoga_always_stops_it() {
        use YogaName::*;
        assert_eq!(sankhya_yoga([Rasi::Leo; 7]), Gola);
        assert_eq!(
            sankhya_yoga([
                Rasi::Aries,
                Rasi::Leo,
                Rasi::Leo,
                Rasi::Leo,
                Rasi::Leo,
                Rasi::Leo,
                Rasi::Leo
            ]),
            Yuga
        );
        for (house, aasraya) in [(1, Rajju), (2, Musala), (3, Nala)] {
            let names = naabhasa_names([house; 7], 11);
            assert!(names.contains(&aasraya), "{house}");
            assert!(!names.contains(&Gola), "{house}");
        }
    }

    #[test]
    fn rama_lists_sarpa_to_judge_and_so_no_daama() {
        use Body::*;
        use Rasi::*;
        // Ch. 1 Example 1: Saturn, Mars and the Sun hold his 4th, 7th and 10th; Jupiter and a waxing Moon his 1st (ADR 0014).
        let at = [
            (Lagna, Cancer),
            (Sun, Aries),
            (Mercury, Taurus),
            (Ketu, Gemini),
            (Moon, Cancer),
            (Jupiter, Cancer),
            (Saturn, Libra),
            (Rahu, Sagittarius),
            (Mars, Capricorn),
            (Venus, Pisces),
        ];
        let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
        let rows: Vec<Yoga> = yogas(Varga::RasiChart, &scene(Aries, &at))
            .into_iter()
            .filter(|y| y.name.class() == YogaClass::Naabhasa)
            .collect();
        assert_eq!(
            rows,
            [Yoga {
                name: YogaName::Sarpa,
                grahas: vec![Sun, Mars, Saturn],
                to_judge: vec![ToJudge::MayNotOperateWell],
                form: None,
            }]
        );
    }

    #[test]
    fn a_sankhya_row_stands_alone_and_a_chart_with_no_other_has_exactly_one() {
        use YogaName::*;
        const SANKHYA: [YogaName; 7] = [Veenaa, Daama, Paasa, Kedaara, Soola, Yuga, Gola];
        // A fixed stream of charts, clustered so that shapes form as well as scatter.
        let mut state: u32 = 221;
        let mut next = |n: u32| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 16) % n
        };
        let (mut alone, mut others) = (0, 0);
        for _ in 0..20_000 {
            let spread = next(12) + 1;
            let houses: [u8; 7] = std::array::from_fn(|_| (next(spread) + 1) as u8);
            let rows = naabhasa_rows(houses, (next(12) + 1) as u8);
            assert!(
                rows.iter()
                    .all(|y| !y.to_judge.contains(&ToJudge::MercurysNature)),
                "{houses:?}"
            );
            let (sankhya, rest): (Vec<YogaName>, Vec<YogaName>) = rows
                .into_iter()
                .map(|y| y.name)
                .partition(|n| SANKHYA.contains(n));
            if rest.is_empty() {
                let planets = houses.map(|h| kth(Rasi::Aries, h as usize));
                assert_eq!(sankhya, [sankhya_yoga(planets)], "{houses:?}");
                alone += 1;
            } else {
                assert_eq!(sankhya, [], "{houses:?} {rest:?}");
                others += 1;
            }
        }
        assert!(alone > 1000 && others > 1000, "{alone} {others}");
    }

    #[test]
    fn naabhasa_yogas_in_a_varga_read_the_vargas_places() {
        use Body::*;
        use Rasi::*;
        // All seven in Aries in the rasi chart; Jupiter at 5° is in Taurus in D-9, the others at 1° in Aries.
        let mut at: Vec<(Body, f64)> = PLANETS.iter().map(|&g| (g, dms(0, 1, 0, 0.0))).collect();
        at.push((Jupiter, dms(0, 5, 0, 0.0)));
        let chart = scene(Aquarius, &at);
        assert!(yoga_in(&chart, Varga::RasiChart, YogaName::Rajju).is_some());
        assert_eq!(yoga_in(&chart, Varga::Navamsa, YogaName::Rajju), None);
        assert!(yoga_in(&chart, Varga::Navamsa, YogaName::Yuga).is_some());
        assert_eq!(rasi_of(&chart, Jupiter), Aries);
    }

    /// `name`'s row in the rasi chart of a scene with the lagna in Aries and the rest piled in the 3rd, Gemini, but for `at`.
    fn from_aries(at: &[(Body, Rasi)], name: YogaName) -> Option<Yoga> {
        from_lagna(Rasi::Aries, Rasi::Gemini, at, name)
    }

    /// `name`'s row with the lagna at `lagna` and the rest piled in `rest`, but for `at`.
    fn from_lagna(lagna: Rasi, rest: Rasi, at: &[(Body, Rasi)], name: YogaName) -> Option<Yoga> {
        let mut at = at.to_vec();
        at.push((Body::Lagna, lagna));
        yoga_of(rest, &at, name)
    }

    fn present(row: Option<Yoga>) -> Vec<Body> {
        let row = row.expect("the yoga is listed");
        assert_eq!(row.to_judge, []);
        assert_eq!(row.form, None);
        row.grahas
    }

    #[test]
    fn subha_is_benefics_in_the_lagna_or_on_both_its_sides() {
        use Body::*;
        use Rasi::*;
        let subha = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Subha);
        let row = subha(&[(Jupiter, Aries)]);
        assert_eq!(
            row.as_ref().map(|y| y.name.class()),
            Some(YogaClass::OtherPopular)
        );
        assert_eq!(present(row), [Jupiter]);
        assert_eq!(
            present(subha(&[(Venus, Pisces), (Jupiter, Taurus)])),
            [Jupiter, Venus]
        );
        // One side, or a malefic in the lagna, is not it.
        assert_eq!(subha(&[(Venus, Pisces)]), None);
        assert_eq!(subha(&[(Venus, Pisces), (Saturn, Taurus)]), None);
        assert_eq!(subha(&[(Mars, Aries)]), None);
    }

    #[test]
    fn asubha_is_malefics_in_the_lagna_or_on_both_its_sides() {
        use Body::*;
        use Rasi::*;
        let asubha = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Asubha);
        assert_eq!(present(asubha(&[(Mars, Aries)])), [Mars]);
        assert_eq!(present(asubha(&[(Rahu, Aries)])), [Rahu]);
        assert_eq!(
            present(asubha(&[(Saturn, Pisces), (Sun, Taurus)])),
            [Sun, Saturn]
        );
        assert_eq!(asubha(&[(Saturn, Pisces)]), None);
        assert_eq!(asubha(&[(Jupiter, Aries)]), None);
    }

    #[test]
    fn the_moons_paksha_decides_subha_or_asubha() {
        use Body::*;
        use Rasi::*;
        let waxing = [(Moon, Aries), (Sun, Capricorn)];
        assert_eq!(present(from_aries(&waxing, YogaName::Subha)), [Moon]);
        assert_eq!(from_aries(&waxing, YogaName::Asubha), None);
        let waning = [(Moon, Aries)];
        assert_eq!(from_aries(&waning, YogaName::Subha), None);
        assert_eq!(present(from_aries(&waning, YogaName::Asubha)), [Moon]);
    }

    #[test]
    fn a_tied_mercury_in_the_lagna_changes_neither_subha_nor_asubha() {
        use Body::*;
        use Rasi::*;
        let at = [(Mercury, Aries), (Sun, Aries), (Jupiter, Aries)];
        assert_eq!(
            present(from_aries(&at, YogaName::Subha)),
            [Mercury, Jupiter]
        );
        // The saumya reading's grahas stand (ADR 0014).
        assert_eq!(present(from_aries(&at, YogaName::Asubha)), [Sun]);
    }

    #[test]
    fn gaja_kesari_is_jupiter_in_a_kendra_from_the_moon_with_a_benefic_on_him() {
        use Body::*;
        use Rasi::*;
        let gaja = |at: &[(Body, Rasi)]| from_aries(at, YogaName::GajaKesari);
        // Venus's aspect, then Mercury alone beside him; the Moon here is waning, so no benefic herself.
        let row = gaja(&[(Moon, Aries), (Jupiter, Cancer), (Venus, Capricorn)]);
        assert_eq!(present(row), [Moon, Jupiter, Venus]);
        let row = gaja(&[(Moon, Aries), (Jupiter, Cancer), (Mercury, Cancer)]);
        assert_eq!(present(row), [Moon, Mercury, Jupiter]);
        // Not in a kendra; no benefic on him; a malefic's aspect is no benefic's.
        assert_eq!(
            gaja(&[(Moon, Aries), (Jupiter, Leo), (Venus, Aquarius)]),
            None
        );
        assert_eq!(gaja(&[(Moon, Aries), (Jupiter, Cancer)]), None);
        assert_eq!(
            gaja(&[(Moon, Aries), (Jupiter, Cancer), (Saturn, Capricorn)]),
            None
        );
        // A waxing Moon is a benefic, and aspects him from the 7th.
        let row = gaja(&[(Moon, Capricorn), (Sun, Leo), (Jupiter, Cancer)]);
        assert_eq!(present(row), [Moon, Jupiter]);
        // A tied Mercury beside Venus, who is on Jupiter just the same, so his reading decides nothing here.
        let row = gaja(&[
            (Moon, Aries),
            (Jupiter, Cancer),
            (Mercury, Capricorn),
            (Venus, Capricorn),
            (Mars, Capricorn),
        ]);
        assert_eq!(present(row), [Moon, Mercury, Jupiter, Venus]);
        // Tied by Jupiter himself, he is the only benefic on Jupiter, so his reading decides it, as in chart 13.
        let row = gaja(&[
            (Moon, Aries),
            (Jupiter, Cancer),
            (Mercury, Cancer),
            (Mars, Cancer),
        ])
        .unwrap();
        assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
        assert_eq!(row.grahas, [Moon, Mercury, Jupiter]);
    }

    #[test]
    fn gaja_kesari_fails_on_a_debilitated_jupiter_or_in_an_enemys_house() {
        use Body::*;
        use Rasi::*;
        let gaja = |at: &[(Body, Rasi)]| from_aries(at, YogaName::GajaKesari);
        assert_eq!(
            gaja(&[(Moon, Aries), (Jupiter, Capricorn), (Venus, Cancer)]),
            None
        );
        // Venus in the 8th from Taurus is his bad enemy; the 2nd, a temporary friend, makes her only neutral (§3.4.3).
        let enemy = [(Moon, Leo), (Jupiter, Taurus), (Venus, Scorpio)];
        assert_eq!(gaja(&enemy), None);
        let neutral = [
            (Moon, Leo),
            (Jupiter, Taurus),
            (Venus, Gemini),
            (Mercury, Taurus),
        ];
        assert_eq!(present(gaja(&neutral)), [Moon, Mercury, Jupiter]);
    }

    #[test]
    fn gaja_kesaris_combustion_is_to_judge_beside_the_sun_in_the_rasi_chart_only() {
        use Body::*;
        use Rasi::*;
        let at = [
            (Moon, Aries),
            (Jupiter, Cancer),
            (Sun, Cancer),
            (Venus, Capricorn),
        ];
        let row = from_aries(&at, YogaName::GajaKesari).unwrap();
        assert_eq!(row.to_judge, [ToJudge::Combust(Jupiter)]);
        // Everything at 15°: in the navamsa Ar is Le, Cn is Sc, Ta is Ta, so the same shape beside the Sun (ADR 0014).
        let at = [
            (Lagna, Aries),
            (Moon, Aries),
            (Jupiter, Cancer),
            (Sun, Cancer),
            (Venus, Taurus),
        ]
        .map(|(b, r)| (b, mid(r)));
        let row = yoga_in(&scene(Gemini, &at), Varga::Navamsa, YogaName::GajaKesari).unwrap();
        assert_eq!(row.to_judge, []);
    }

    #[test]
    fn guru_mangala_is_jupiter_and_mars_together_or_opposite() {
        use Body::*;
        use Rasi::*;
        let guru = |at: &[(Body, Rasi)]| from_aries(at, YogaName::GuruMangala);
        assert_eq!(
            present(guru(&[(Jupiter, Leo), (Mars, Leo)])),
            [Mars, Jupiter]
        );
        assert_eq!(
            present(guru(&[(Jupiter, Leo), (Mars, Aquarius)])),
            [Mars, Jupiter]
        );
        assert_eq!(guru(&[(Jupiter, Leo), (Mars, Virgo)]), None);
    }

    #[test]
    fn amala_is_only_benefics_in_the_10th_from_the_lagna_or_the_moon() {
        use Body::*;
        use Rasi::*;
        let amala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Amala);
        // The Moon in the pile counts from Gemini, so her 10th is Pisces.
        assert_eq!(present(amala(&[(Venus, Capricorn)])), [Venus]);
        assert_eq!(present(amala(&[(Jupiter, Pisces)])), [Jupiter]);
        assert_eq!(amala(&[(Venus, Capricorn), (Saturn, Capricorn)]), None);
        assert_eq!(amala(&[(Rahu, Capricorn)]), None);
        // Empty, then the wrong house.
        assert_eq!(amala(&[]), None);
        assert_eq!(amala(&[(Venus, Aquarius)]), None);
        // A waxing Moon in the lagna's 10th, then a waning one.
        assert_eq!(present(amala(&[(Moon, Capricorn), (Sun, Leo)])), [Moon]);
        assert_eq!(amala(&[(Moon, Capricorn)]), None);
        // Mercury alone is saumya; beside a malefic, kroora; tied, a malefic is beside him anyway.
        assert_eq!(present(amala(&[(Mercury, Capricorn)])), [Mercury]);
        assert_eq!(amala(&[(Mercury, Capricorn), (Mars, Capricorn)]), None);
        let tied = [(Mercury, Capricorn), (Mars, Capricorn), (Venus, Capricorn)];
        assert_eq!(amala(&tied), None);
    }

    #[test]
    fn parvata_is_only_benefics_in_the_kendras_and_the_8th() {
        use Body::*;
        use Rasi::*;
        let parvata = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Parvata);
        assert_eq!(
            present(parvata(&[(Jupiter, Aries), (Venus, Cancer)])),
            [Jupiter, Venus]
        );
        let row = parvata(&[(Jupiter, Aries), (Mercury, Scorpio)]);
        assert_eq!(present(row), [Mercury, Jupiter]);
        // A malefic in a kendra, a node, a kroora Mercury, or a malefic in the 8th.
        assert_eq!(parvata(&[(Jupiter, Aries), (Mars, Cancer)]), None);
        assert_eq!(parvata(&[(Jupiter, Aries), (Ketu, Libra)]), None);
        assert_eq!(
            parvata(&[(Jupiter, Aries), (Mercury, Libra), (Saturn, Libra)]),
            None
        );
        assert_eq!(parvata(&[(Jupiter, Aries), (Sun, Scorpio)]), None);
        // Vacant kendras all: (2) leaves the 7th free to be vacant, but something must occupy a kendra.
        assert_eq!(parvata(&[]), None);
        assert_eq!(parvata(&[(Jupiter, Scorpio)]), None);
    }

    /// A row's form, what is left to judge and grahas.
    fn formed(row: Option<Yoga>) -> (Option<u8>, Vec<ToJudge>, Vec<Body>) {
        let row = row.expect("the yoga is listed");
        (row.form, row.to_judge, row.grahas)
    }

    #[test]
    fn kaahalas_first_form_is_the_4th_lord_and_jupiter_in_mutual_kendras_with_a_strong_lagna_lord()
    {
        use Body::*;
        use Rasi::*;
        let kaahala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Kaahala);
        // The Moon rules Aries's 4th and Mars its lagna.
        let row = kaahala(&[(Moon, Libra), (Jupiter, Cancer)]);
        assert_eq!(
            formed(row),
            (
                Some(1),
                vec![ToJudge::Strong(Mars)],
                vec![Moon, Mars, Jupiter]
            )
        );
        assert_eq!(kaahala(&[(Moon, Libra), (Jupiter, Leo)]), None);
        // For Virgo he is the 4th lord himself: no pair, so no first form. "Some say 9th lord" is not computed.
        assert_eq!(from_lagna(Virgo, Gemini, &[], YogaName::Kaahala), None);
    }

    #[test]
    fn kaahalas_second_form_is_the_4th_lord_exalted_or_in_own_rasi_joined_by_the_10th_lord() {
        use Body::*;
        use Rasi::*;
        let kaahala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Kaahala);
        for rasi in [Taurus, Cancer] {
            let row = kaahala(&[(Moon, rasi), (Saturn, rasi), (Jupiter, Virgo)]);
            assert_eq!(
                formed(row),
                (Some(2), vec![], vec![Moon, Saturn]),
                "{rasi:?}"
            );
        }
        assert_eq!(
            kaahala(&[(Moon, Taurus), (Saturn, Aries), (Jupiter, Virgo)]),
            None
        );
        assert_eq!(
            kaahala(&[(Moon, Leo), (Saturn, Leo), (Jupiter, Virgo)]),
            None
        );
        // Both forms hold: the present one is the row, not the to-judge one.
        let row = kaahala(&[(Moon, Taurus), (Saturn, Taurus), (Jupiter, Leo)]);
        assert_eq!(formed(row), (Some(2), vec![], vec![Moon, Saturn]));
    }

    #[test]
    fn kaahalas_4th_lord_in_his_own_scorpio_or_aquarius_counts_wherever_his_co_lord_is() {
        use Body::*;
        use Rasi::*;
        // Scorpio's 4th is Aq and 10th Le, with Jupiter out of a kendra from Rahu: only the second form could hold.
        let kaahala = |sun, rahu| {
            let at = [
                (Saturn, Aquarius),
                (Sun, sun),
                (Rahu, rahu),
                (Jupiter, Virgo),
            ];
            from_lagna(Scorpio, Gemini, &at, YogaName::Kaahala)
        };
        for rahu in [Taurus, Aquarius] {
            assert_eq!(
                formed(kaahala(Aquarius, rahu)),
                (Some(2), vec![], vec![Sun, Saturn]),
                "{rahu:?}"
            );
        }
        // Without the 10th lord beside him.
        assert_eq!(kaahala(Gemini, Taurus), None);
    }

    #[test]
    fn chaamaras_lagna_lord_is_exalted_in_a_kendra_with_jupiters_aspect() {
        use Body::*;
        use Rasi::*;
        let chaamara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Chaamara);
        assert_eq!(
            present(chaamara(&[(Mars, Capricorn), (Jupiter, Cancer)])),
            [Mars, Jupiter]
        );
        assert_eq!(chaamara(&[(Mars, Capricorn)]), None);
        // At home, not exalted; then exalted outside a kendra, in Aquarius's 9th.
        assert_eq!(chaamara(&[(Mars, Aries), (Jupiter, Leo)]), None);
        let at = [(Saturn, Libra), (Jupiter, Aries)];
        assert_eq!(from_lagna(Aquarius, Gemini, &at, YogaName::Chaamara), None);
    }

    #[test]
    fn chaamaras_two_benefics_join_in_the_7th_9th_or_10th() {
        use Body::*;
        use Rasi::*;
        let chaamara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Chaamara);
        for rasi in [Libra, Sagittarius, Capricorn] {
            let row = chaamara(&[(Venus, rasi), (Jupiter, rasi)]);
            assert_eq!(present(row), [Jupiter, Venus], "{rasi:?}");
        }
        assert_eq!(chaamara(&[(Venus, Libra), (Jupiter, Sagittarius)]), None);
        assert_eq!(chaamara(&[(Venus, Scorpio), (Jupiter, Scorpio)]), None);
        assert_eq!(chaamara(&[(Venus, Libra)]), None);
        // A tied Mercury is the second benefic only if saumya.
        let row = chaamara(&[(Mercury, Libra), (Venus, Libra), (Mars, Libra)]).unwrap();
        assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
        assert_eq!(row.grahas, [Mercury, Venus]);
    }

    #[test]
    fn sankhas_first_form_is_the_5th_and_6th_lords_in_mutual_kendras_with_a_strong_lagna_lord() {
        use Body::*;
        use Rasi::*;
        let sankha = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Sankha);
        for mercury in [Scorpio, Leo] {
            let row = sankha(&[(Sun, Leo), (Mercury, mercury)]);
            assert_eq!(
                formed(row),
                (
                    Some(1),
                    vec![ToJudge::Strong(Mars)],
                    vec![Sun, Mars, Mercury]
                )
            );
        }
        assert_eq!(sankha(&[(Sun, Leo), (Mercury, Virgo)]), None);
    }

    #[test]
    fn sankhas_second_form_is_the_lagna_and_10th_lords_in_a_movable_rasi_with_a_strong_9th_lord() {
        use Body::*;
        use Rasi::*;
        let sankha = |mars, saturn| {
            // The 5th and 6th lords apart, to fail the first form.
            from_aries(
                &[(Sun, Leo), (Mercury, Virgo), (Mars, mars), (Saturn, saturn)],
                YogaName::Sankha,
            )
        };
        assert_eq!(
            formed(sankha(Libra, Libra)),
            (
                Some(2),
                vec![ToJudge::Strong(Jupiter)],
                vec![Mars, Jupiter, Saturn]
            )
        );
        assert_eq!(sankha(Taurus, Taurus), None);
        assert_eq!(sankha(Libra, Cancer), None);
    }

    #[test]
    fn bheris_first_form_is_planets_in_the_1st_2nd_7th_and_12th_with_a_strong_9th_lord() {
        use Body::*;
        use Rasi::*;
        let bheri = |twelfth: (Body, Rasi)| {
            from_aries(
                &[(Sun, Aries), (Moon, Taurus), (Mars, Libra), twelfth],
                YogaName::Bheri,
            )
        };
        assert_eq!(
            formed(bheri((Saturn, Pisces))),
            (
                Some(1),
                vec![ToJudge::Strong(Jupiter)],
                vec![Sun, Moon, Mars, Jupiter, Saturn]
            )
        );
        assert_eq!(bheri((Saturn, Cancer)), None);
        // A node is no planet here (ADR 0014).
        assert_eq!(bheri((Rahu, Pisces)), None);
    }

    #[test]
    fn bheris_second_form_is_jupiter_venus_and_the_lagna_lord_in_mutual_kendras_with_a_strong_9th_lord()
     {
        use Body::*;
        use Rasi::*;
        let bheri = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Bheri);
        let row = bheri(&[(Jupiter, Aries), (Venus, Cancer), (Mars, Libra)]);
        assert_eq!(
            formed(row),
            (
                Some(2),
                vec![ToJudge::Strong(Jupiter)],
                vec![Mars, Jupiter, Venus]
            )
        );
        assert_eq!(
            bheri(&[(Jupiter, Aries), (Venus, Leo), (Mars, Libra)]),
            None
        );
        // Venus is both for Taurus: Jupiter and Venus are the only pair. Saturn is its 9th lord.
        let row = from_lagna(
            Taurus,
            Gemini,
            &[(Jupiter, Aries), (Venus, Cancer)],
            YogaName::Bheri,
        );
        assert_eq!(
            formed(row),
            (
                Some(2),
                vec![ToJudge::Strong(Saturn)],
                vec![Jupiter, Venus, Saturn]
            )
        );
    }

    #[test]
    fn mridanga_is_a_planet_in_own_or_uchcha_in_a_kendra_or_trikona_with_a_strong_lagna_lord() {
        use Body::*;
        use Rasi::*;
        let mridanga = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Mridanga);
        for at in [
            (Saturn, Libra),
            (Moon, Cancer),
            (Sun, Leo),
            (Jupiter, Sagittarius),
        ] {
            let row = mridanga(&[at]);
            assert_eq!(
                formed(row),
                (
                    None,
                    vec![ToJudge::Strong(Mars)],
                    in_graha_order(&[at.0, Mars])
                ),
                "{at:?}"
            );
        }
        // Mercury's own rasi in the pile is the 3rd; uchcha in the 6th is outside too; neecha is no dignity.
        assert_eq!(mridanga(&[]), None);
        assert_eq!(mridanga(&[(Mercury, Virgo)]), None);
        assert_eq!(mridanga(&[(Saturn, Aries)]), None);
        // A node is no planet here (ADR 0014): Rahu's own Aquarius in Taurus's 10th.
        assert_eq!(
            from_lagna(
                Taurus,
                Pisces,
                &[(Rahu, Aquarius), (Mars, Aries)],
                YogaName::Mridanga
            ),
            None
        );
    }

    #[test]
    fn sreenaatha_is_the_7th_lord_exalted_in_the_10th_and_the_10th_lord_with_the_9th() {
        use Body::*;
        use Rasi::*;
        let row = from_lagna(
            Sagittarius,
            Gemini,
            &[(Mercury, Virgo), (Sun, Virgo)],
            YogaName::Sreenaatha,
        );
        assert_eq!(present(row), [Sun, Mercury]);
        let row = from_lagna(
            Sagittarius,
            Gemini,
            &[(Mercury, Virgo), (Sun, Libra)],
            YogaName::Sreenaatha,
        );
        assert_eq!(row, None);
        // Footnote 33: only Sagittarius can have it, co-lords and all.
        for lagna in RASIS {
            let row = from_lagna(lagna, kth(lagna, 10), &[], YogaName::Sreenaatha);
            assert_eq!(row.is_some(), lagna == Sagittarius, "{lagna:?}");
        }
    }

    #[test]
    fn matsya_is_benefics_in_the_lagna_and_9th_a_planet_in_the_5th_and_malefics_in_the_4th_and_8th()
    {
        use Body::*;
        use Rasi::*;
        let whole = [
            (Jupiter, Aries),
            (Venus, Sagittarius),
            (Sun, Leo),
            (Mars, Cancer),
            (Saturn, Scorpio),
        ];
        let matsya = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Matsya);
        assert_eq!(present(matsya(&whole)), [Sun, Mars, Jupiter, Venus, Saturn]);
        // Each named house emptied in turn, all to the pile.
        for missing in 0..whole.len() {
            let mut at = whole.to_vec();
            at[missing].1 = Gemini;
            assert_eq!(matsya(&at), None, "{:?}", whole[missing]);
        }
        // A node is a malefic in the 4th (ADR 0014) but no planet in the 5th.
        let mut node = whole.to_vec();
        node[3] = (Rahu, Cancer);
        assert!(matsya(&node).is_some());
        let mut node = whole.to_vec();
        node[2] = (Rahu, Leo);
        assert_eq!(matsya(&node), None);
    }

    #[test]
    fn koorma_is_dignified_benefics_in_the_5th_to_7th_and_malefics_in_own_or_uchcha_in_the_1st_3rd_and_11th()
     {
        use Body::*;
        use Rasi::*;
        let koorma = |at: &[(Body, Rasi)]| from_lagna(Aries, Taurus, at, YogaName::Koorma);
        let whole = [
            // Jupiter's friend the Sun is in the pile, in Leo's 10th, so his close friend. Mercury exalted, Venus at home.
            (Jupiter, Leo),
            (Mercury, Virgo),
            (Venus, Libra),
            // Mars at home, Rahu exalted, Saturn at home.
            (Mars, Aries),
            (Rahu, Gemini),
            (Saturn, Aquarius),
        ];
        assert_eq!(
            present(koorma(&whole)),
            [Mars, Mercury, Jupiter, Venus, Saturn, Rahu]
        );
        // Each house emptied in turn.
        for missing in 0..whole.len() {
            let mut at = whole.to_vec();
            at[missing].1 = Taurus;
            assert_eq!(koorma(&at), None, "{:?}", whole[missing]);
        }
        // The Sun in Leo's 9th only neutral to Jupiter (§3.4.3), so not friendly, though himself exalted in the 1st.
        let mut neutral = whole.to_vec();
        neutral.push((Sun, Aries));
        assert_eq!(koorma(&neutral), None);
        // A malefic neither at home nor exalted: the Sun alone in the 3rd, Jupiter's friend still.
        let mut undignified = whole.to_vec();
        undignified[4] = (Rahu, Taurus);
        undignified.push((Sun, Gemini));
        assert_eq!(koorma(&undignified), None);
    }

    #[test]
    fn khadga_is_the_2nd_and_9th_lords_exchanged_with_the_lagna_lord_in_a_kendra_or_trikona() {
        use Body::*;
        use Rasi::*;
        let khadga = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Khadga);
        for mars in [Cancer, Leo] {
            let row = khadga(&[(Venus, Sagittarius), (Jupiter, Taurus), (Mars, mars)]);
            assert_eq!(present(row), [Mars, Jupiter, Venus], "{mars:?}");
        }
        assert_eq!(khadga(&[(Venus, Sagittarius), (Jupiter, Taurus)]), None);
        assert_eq!(
            khadga(&[(Venus, Libra), (Jupiter, Taurus), (Mars, Cancer)]),
            None
        );
        assert_eq!(
            khadga(&[(Venus, Sagittarius), (Jupiter, Leo), (Mars, Cancer)]),
            None
        );
    }

    #[test]
    fn kusuma_is_a_fixed_lagna_venus_in_a_kendra_the_moon_in_a_trikona_with_a_benefic_and_saturn_in_the_10th()
     {
        use Body::*;
        use Rasi::*;
        let whole = [
            (Venus, Leo),
            (Moon, Virgo),
            (Jupiter, Virgo),
            (Saturn, Aquarius),
        ];
        let kusuma = |at: &[(Body, Rasi)]| from_lagna(Taurus, Gemini, at, YogaName::Kusuma);
        assert_eq!(present(kusuma(&whole)), [Moon, Jupiter, Venus, Saturn]);
        for missing in 0..whole.len() {
            let mut at = whole.to_vec();
            at[missing].1 = Gemini;
            assert_eq!(kusuma(&at), None, "{:?}", whole[missing]);
        }
        // A malefic is no company for her; the same shape from a movable lagna is not it.
        let malefic = [
            (Venus, Leo),
            (Moon, Virgo),
            (Mars, Virgo),
            (Saturn, Aquarius),
        ];
        assert_eq!(kusuma(&malefic), None);
        let movable = [
            (Venus, Cancer),
            (Moon, Leo),
            (Jupiter, Leo),
            (Saturn, Capricorn),
        ];
        assert_eq!(from_aries(&movable, YogaName::Kusuma), None);
    }

    #[test]
    fn kalaanidhi_is_jupiter_in_the_2nd_or_5th_with_both_mercury_and_venus_on_him() {
        use Body::*;
        use Rasi::*;
        let kalaanidhi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Kalaanidhi);
        let all = [Mercury, Jupiter, Venus];
        assert_eq!(
            present(kalaanidhi(&[
                (Jupiter, Taurus),
                (Mercury, Taurus),
                (Venus, Scorpio)
            ])),
            all
        );
        assert_eq!(
            present(kalaanidhi(&[
                (Jupiter, Leo),
                (Mercury, Aquarius),
                (Venus, Leo)
            ])),
            all
        );
        assert_eq!(kalaanidhi(&[(Jupiter, Taurus), (Mercury, Taurus)]), None);
        assert_eq!(kalaanidhi(&[(Jupiter, Taurus), (Venus, Scorpio)]), None);
        assert_eq!(
            kalaanidhi(&[(Jupiter, Cancer), (Mercury, Cancer), (Venus, Cancer)]),
            None
        );
    }

    /// Kalpadruma's row in the given chart with the lagna in Aries and the rest in Gemini.
    fn kalpadruma_in(d: Varga, at: &[(Body, f64)]) -> Option<Vec<Body>> {
        let mut at = at.to_vec();
        at.push((Body::Lagna, mid(Rasi::Aries)));
        yoga_in(&scene(Rasi::Gemini, &at), d, YogaName::Kalpadruma).map(|y| y.grahas)
    }

    #[test]
    fn kalpadrumas_fourth_graha_is_the_dispositors_dispositor_in_the_navamsa() {
        use Body::*;
        use Rasi::*;
        // Mars in Cn, so the Moon; she in Li, so Venus in the rasi chart. Her navamsa picks the fourth.
        let with_moon_at = |moon: f64, more: &[(Body, f64)]| {
            let mut at = vec![(Mars, mid(Cancer)), (Moon, moon), (Venus, mid(Leo))];
            at.extend_from_slice(more);
            kalpadruma_in(Varga::RasiChart, &at)
        };
        // At 15° Li her navamsa is Aq, so Saturn, who must then stand in a kendra of the rasi chart.
        let saturn_placed = [(Saturn, mid(Libra))];
        assert_eq!(
            with_moon_at(mid(Libra), &saturn_placed),
            Some(vec![Mars, Moon, Venus, Saturn])
        );
        // Saturn in the 3rd, and Rahu, the co-lord, out of his uchcha Gemini.
        assert_eq!(with_moon_at(mid(Libra), &[(Rahu, mid(Taurus))]), None);
        // At 1° Li her navamsa is Li, so Venus again, with Saturn left in the 3rd.
        assert_eq!(
            with_moon_at(dms(6, 1, 0, 0.0), &[]),
            Some(vec![Mars, Moon, Venus, Venus])
        );
    }

    #[test]
    fn kalpadruma_needs_all_four_placed_and_a_graha_at_home_disposes_of_himself() {
        use Body::*;
        use Rasi::*;
        let kalpadruma = |at: &[(Body, f64)]| kalpadruma_in(Varga::RasiChart, at);
        // Venus in the 3rd.
        let at = [(Mars, mid(Cancer)), (Moon, dms(6, 1, 0, 0.0))];
        assert_eq!(kalpadruma(&at), None);
        // Mars in Gemini, the 3rd, is not placed, whatever follows.
        let at = [(Moon, mid(Libra)), (Venus, mid(Leo)), (Saturn, mid(Libra))];
        assert_eq!(kalpadruma(&at), None);
        // Mars at home three times over; at 15° Ar his navamsa is Le, so the Sun.
        let at = [(Mars, mid(Aries)), (Sun, mid(Leo))];
        assert_eq!(kalpadruma(&at), Some(vec![Mars, Mars, Mars, Sun]));
        // Mars at home in the lagna is tried once Ketu, the main owner, fails in the 8th.
        let at = [
            (Lagna, mid(Scorpio)),
            (Mars, mid(Scorpio)),
            (Ketu, mid(Gemini)),
        ];
        let row = yoga_in(&scene(Gemini, &at), Varga::RasiChart, YogaName::Kalpadruma);
        assert_eq!(row.map(|y| y.grahas), Some(vec![Mars, Mars, Mars, Mars]));
    }

    #[test]
    fn kalpadruma_counts_an_uchcha_rasi_outside_the_kendras_and_trikonas() {
        use Body::*;
        use Rasi::*;
        // Leo's lagna lord in Aq, Saturn exalted in its 3rd, and Venus at 1° Li's navamsa: chart 9's shape, with her in Ar.
        let at = [
            (Lagna, mid(Leo)),
            (Sun, mid(Aquarius)),
            (Saturn, dms(6, 1, 0, 0.0)),
            (Venus, mid(Aries)),
        ];
        let row = yoga_in(&scene(Gemini, &at), Varga::RasiChart, YogaName::Kalpadruma);
        assert_eq!(row.map(|y| y.grahas), Some(vec![Sun, Saturn, Venus, Venus]));
    }

    #[test]
    fn kalpadruma_is_only_in_the_rasi_chart_list() {
        use Body::*;
        use Rasi::*;
        let at = [(Mars, mid(Aries)), (Sun, mid(Leo))];
        assert!(kalpadruma_in(Varga::RasiChart, &at).is_some());
        for d in VARGAS.into_iter().filter(|&d| d != Varga::RasiChart) {
            assert_eq!(kalpadruma_in(d, &at), None, "{d:?}");
        }
    }

    #[test]
    fn lagnaadhi_is_two_or_more_benefics_in_the_7th_and_8th_in_any_arrangement() {
        use Body::*;
        use Rasi::*;
        let lagnaadhi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Lagnaadhi);
        assert_eq!(
            present(lagnaadhi(&[(Jupiter, Libra), (Venus, Libra)])),
            [Jupiter, Venus]
        );
        assert_eq!(
            present(lagnaadhi(&[(Jupiter, Scorpio), (Venus, Scorpio)])),
            [Jupiter, Venus]
        );
        assert_eq!(
            present(lagnaadhi(&[(Jupiter, Libra), (Venus, Scorpio)])),
            [Jupiter, Venus]
        );
        // One alone, and the 6th not added (ADR 0014).
        assert_eq!(lagnaadhi(&[(Jupiter, Libra)]), None);
        assert_eq!(lagnaadhi(&[(Jupiter, Virgo), (Venus, Libra)]), None);
        assert_eq!(lagnaadhi(&[(Jupiter, Virgo), (Venus, Virgo)]), None);
    }

    #[test]
    fn a_malefic_conjoining_either_benefic_or_on_it_by_graha_drishti_stops_lagnaadhi() {
        use Body::*;
        use Rasi::*;
        let lagnaadhi = |more: &[(Body, Rasi)]| {
            let mut at = vec![(Jupiter, Libra), (Venus, Scorpio)];
            at.extend_from_slice(more);
            from_aries(&at, YogaName::Lagnaadhi)
        };
        // Conjoining each, a node among them, which has no drishti but is a malefic.
        assert_eq!(lagnaadhi(&[(Mars, Libra)]), None);
        assert_eq!(lagnaadhi(&[(Saturn, Scorpio)]), None);
        assert_eq!(lagnaadhi(&[(Rahu, Libra)]), None);
        // Graha drishti: Saturn's 7th on Li from Ar, his 3rd on Li from Le, and the Sun's 7th on Sc from Ta.
        assert_eq!(lagnaadhi(&[(Saturn, Aries)]), None);
        assert_eq!(lagnaadhi(&[(Saturn, Leo)]), None);
        assert_eq!(lagnaadhi(&[(Sun, Taurus)]), None);
        // A node's rasi drishti is not graha drishti.
        assert!(lagnaadhi(&[(Rahu, Aries)]).is_some());
    }

    #[test]
    fn lagnaadhi_reads_the_moon_by_paksha_and_mercury_by_his_company() {
        use Body::*;
        use Rasi::*;
        let lagnaadhi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Lagnaadhi);
        // A waxing Moon is the second benefic, or a third.
        assert_eq!(
            present(lagnaadhi(&[(Moon, Scorpio), (Jupiter, Libra)])),
            [Moon, Jupiter]
        );
        let row = lagnaadhi(&[(Moon, Scorpio), (Jupiter, Libra), (Venus, Libra)]);
        assert_eq!(present(row), [Moon, Jupiter, Venus]);
        // A waning one is no benefic, and a malefic beside him or on him by graha drishti.
        assert_eq!(
            lagnaadhi(&[(Moon, Scorpio), (Sun, Capricorn), (Jupiter, Libra)]),
            None
        );
        let beside = [
            (Moon, Scorpio),
            (Sun, Capricorn),
            (Jupiter, Libra),
            (Venus, Scorpio),
        ];
        assert_eq!(lagnaadhi(&beside), None);
        let aspecting = [(Moon, Aries), (Jupiter, Libra), (Venus, Libra)];
        assert_eq!(lagnaadhi(&aspecting), None);
        // A saumya Mercury is a benefic.
        assert_eq!(
            present(lagnaadhi(&[(Mercury, Libra), (Jupiter, Scorpio)])),
            [Mercury, Jupiter]
        );
        // A kroora Mercury's graha drishti, with Rahu beside him, stops it: Rahu has none, so it is his alone.
        let aspecting = [
            (Mercury, Aries),
            (Rahu, Aries),
            (Jupiter, Libra),
            (Venus, Libra),
        ];
        assert_eq!(lagnaadhi(&aspecting), None);
        // Alone there, he is saumya and his graha drishti is no harm.
        let harmless = [(Mercury, Aries), (Jupiter, Libra), (Venus, Libra)];
        assert_eq!(present(lagnaadhi(&harmless)), [Jupiter, Venus]);
    }

    #[test]
    fn a_tied_mercurys_reading_of_lagnaadhi_is_to_judge_only_if_it_changes_the_result() {
        use Body::*;
        use Rasi::*;
        let lagnaadhi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Lagnaadhi);
        // In Ar with Venus and Rahu, he is tied: saumya, his graha drishti is harmless; kroora, it stops the Moon and Jupiter in Li.
        let at = [
            (Mercury, Aries),
            (Venus, Aries),
            (Rahu, Aries),
            (Moon, Libra),
            (Jupiter, Libra),
        ];
        let row = lagnaadhi(&at).unwrap();
        assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
        assert_eq!(row.grahas, [Moon, Jupiter]);
        // Tied in Pi, his graha drishti falls on Vi, so it stands either way.
        let at = [
            (Mercury, Pisces),
            (Venus, Pisces),
            (Rahu, Pisces),
            (Moon, Libra),
            (Jupiter, Libra),
        ];
        assert_eq!(present(lagnaadhi(&at)), [Moon, Jupiter]);
    }

    #[test]
    fn hari_is_every_benefic_in_the_2nd_12th_and_8th_from_the_2nd_lord() {
        use Body::*;
        use Rasi::*;
        let hari = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Hari);
        // Venus rules Aries's 2nd; from Li they are Sc, Vi and Ta. The Moon waxes in Sc, Mercury in the pile is kroora.
        let row = hari(&[(Venus, Libra), (Jupiter, Virgo), (Moon, Scorpio)]);
        assert_eq!(present(row), [Moon, Jupiter, Venus]);
        // Every benefic, not any: Jupiter outside; a waxing Moon outside.
        assert_eq!(hari(&[(Venus, Libra), (Moon, Scorpio)]), None);
        assert_eq!(hari(&[(Venus, Libra), (Jupiter, Virgo)]), None);
        // A waning Moon need not be there.
        let row = hari(&[(Venus, Libra), (Jupiter, Virgo), (Sun, Leo)]);
        assert_eq!(present(row), [Jupiter, Venus]);
        // A saumya Mercury must be, and a kroora one need not.
        let saumya = [
            (Mercury, Cancer),
            (Venus, Libra),
            (Jupiter, Virgo),
            (Moon, Scorpio),
        ];
        assert_eq!(hari(&saumya), None);
        let saumya = [
            (Mercury, Taurus),
            (Venus, Libra),
            (Jupiter, Virgo),
            (Moon, Scorpio),
        ];
        assert_eq!(present(hari(&saumya)), [Moon, Mercury, Jupiter, Venus]);
    }

    #[test]
    fn a_tied_mercury_outside_haris_places_is_to_judge() {
        use Body::*;
        use Rasi::*;
        // Beside Venus and Mars in Li he is tied, and Li is none of its own places.
        let at = [
            (Mercury, Libra),
            (Venus, Libra),
            (Mars, Libra),
            (Jupiter, Virgo),
            (Moon, Scorpio),
        ];
        let row = from_aries(&at, YogaName::Hari).unwrap();
        assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
        assert_eq!(row.grahas, [Moon, Jupiter, Venus]);
    }

    #[test]
    fn hari_works_from_either_co_lord_and_names_him() {
        use Body::*;
        use Rasi::*;
        // Scorpio is Libra's 2nd. From Ketu in Ar the places are Ta, Pi and Sc; from Mars in the Cp pile, Aq, Sg and Le.
        let at = [
            (Ketu, Aries),
            (Jupiter, Taurus),
            (Venus, Pisces),
            (Moon, Virgo),
        ];
        let row = from_lagna(Libra, Capricorn, &at, YogaName::Hari);
        assert_eq!(present(row), [Jupiter, Venus, Ketu]);
        // And from Mars, the same benefics placed from him.
        let at = [
            (Mars, Aries),
            (Ketu, Capricorn),
            (Jupiter, Taurus),
            (Venus, Pisces),
            (Moon, Virgo),
        ];
        let row = from_lagna(Libra, Capricorn, &at, YogaName::Hari);
        assert_eq!(present(row), [Mars, Jupiter, Venus]);
    }

    #[test]
    fn hara_is_every_benefic_in_the_4th_9th_and_8th_from_the_7th_lord() {
        use Body::*;
        use Rasi::*;
        let hara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Hara);
        // Venus rules Aries's 7th; from Li they are Cp, Ge and Ta, with the waxing Moon and Jupiter in the Ge pile.
        assert_eq!(present(hara(&[(Venus, Libra)])), [Moon, Jupiter, Venus]);
        assert_eq!(hara(&[(Venus, Libra), (Jupiter, Leo)]), None);
    }

    #[test]
    fn brahmas_first_form_is_every_benefic_in_the_4th_10th_and_11th_from_the_lagna_lord() {
        use Body::*;
        use Rasi::*;
        let brahma = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Brahma);
        // Mars in Ar: Cn, Cp and Aq. The Moon wanes in Aq. Jupiter is Aries's 9th lord, so has no second form.
        let at = [
            (Mars, Aries),
            (Jupiter, Cancer),
            (Venus, Capricorn),
            (Moon, Aquarius),
        ];
        assert_eq!(
            formed(brahma(&at)),
            (Some(1), vec![], vec![Mars, Jupiter, Venus])
        );
        let at = [
            (Mars, Aries),
            (Jupiter, Leo),
            (Venus, Capricorn),
            (Moon, Aquarius),
        ];
        assert_eq!(brahma(&at), None);
    }

    #[test]
    fn brahmas_second_form_is_jupiter_venus_and_mercury_each_in_a_kendra_from_its_lord() {
        use Body::*;
        use Rasi::*;
        // Taurus: the 9th lord is Saturn, the 11th Jupiter, the 1st Venus, the 10th Saturn or Rahu.
        let brahma = |at: &[(Body, Rasi)]| from_lagna(Taurus, Gemini, at, YogaName::Brahma);
        let row = brahma(&[
            (Saturn, Aries),
            (Jupiter, Cancer),
            (Venus, Libra),
            (Mercury, Libra),
        ]);
        assert_eq!(
            formed(row),
            (Some(2), vec![], vec![Mercury, Jupiter, Venus, Saturn])
        );
        // Mercury from the 10th lord alone: Rahu's kendra, 4th from Le, is none of Venus's or Saturn's.
        let row = brahma(&[
            (Saturn, Aries),
            (Rahu, Leo),
            (Jupiter, Cancer),
            (Venus, Libra),
            (Mercury, Scorpio),
        ]);
        assert_eq!(
            formed(row),
            (Some(2), vec![], vec![Mercury, Jupiter, Venus, Saturn, Rahu])
        );
        // Each graha out of its kendra in turn.
        let whole = [
            (Saturn, Aries),
            (Jupiter, Cancer),
            (Venus, Libra),
            (Mercury, Libra),
        ];
        assert_eq!(
            brahma(&[whole[0], (Jupiter, Leo), whole[2], whole[3]]),
            None
        );
        assert_eq!(
            brahma(&[whole[0], whole[1], (Venus, Scorpio), (Mercury, Scorpio)]),
            None
        );
        assert_eq!(
            brahma(&[whole[0], whole[1], whole[2], (Mercury, Scorpio)]),
            None
        );
    }

    #[test]
    fn harsha_sarala_and_vimala_are_the_dusthana_lord_in_his_own_dusthana() {
        use Body::*;
        use Rasi::*;
        // From Aries: Mercury rules the 6th, Mars or Ketu the 8th, Jupiter the 12th.
        assert_eq!(
            present(from_aries(&[(Mercury, Virgo)], YogaName::Harsha)),
            [Mercury]
        );
        assert_eq!(from_aries(&[], YogaName::Harsha), None);
        assert_eq!(
            present(from_aries(&[(Jupiter, Pisces)], YogaName::Vimala)),
            [Jupiter]
        );
        assert_eq!(from_aries(&[], YogaName::Vimala), None);
        assert_eq!(
            present(from_aries(&[(Mars, Scorpio)], YogaName::Sarala)),
            [Mars]
        );
        assert_eq!(from_aries(&[], YogaName::Sarala), None);
        // A node at home is not tried (ADR 0014).
        assert_eq!(from_aries(&[(Ketu, Scorpio)], YogaName::Sarala), None);
    }

    #[test]
    fn harsha_in_a_varga_reads_that_vargas_lagna_and_places() {
        use Body::*;
        use Rasi::*;
        // 1° Ar is D-9's Aries and 8° Cn its Virgo.
        let at = [(Lagna, dms(0, 1, 0, 0.0)), (Mercury, dms(3, 8, 0, 0.0))];
        let chart = scene(Gemini, &at);
        assert_eq!(yoga_in(&chart, Varga::RasiChart, YogaName::Harsha), None);
        let row = yoga_in(&chart, Varga::Navamsa, YogaName::Harsha);
        assert_eq!(present(row), [Mercury]);
    }

    #[test]
    fn siva_is_the_5th_lord_in_the_9th_the_9th_in_the_10th_and_the_10th_in_the_5th() {
        use Body::*;
        use Rasi::*;
        let siva = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Siva);
        let at = [(Sun, Sagittarius), (Jupiter, Capricorn), (Saturn, Leo)];
        assert_eq!(present(siva(&at)), [Sun, Jupiter, Saturn]);
        assert_eq!(siva(&[at[0], at[1]]), None);
        assert_eq!(siva(&[at[0], at[2]]), None);
        assert_eq!(siva(&[at[1], at[2]]), None);
    }

    #[test]
    fn trilochana_is_the_sun_moon_and_mars_in_mutual_trines_the_same_rasi_counting() {
        use Body::*;
        use Rasi::*;
        let trilochana = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Trilochana);
        let at = [(Sun, Aries), (Moon, Leo), (Mars, Sagittarius)];
        assert_eq!(present(trilochana(&at)), [Sun, Moon, Mars]);
        // All three in the pile (ADR 0014).
        assert_eq!(present(trilochana(&[])), [Sun, Moon, Mars]);
        assert_eq!(
            trilochana(&[(Sun, Aries), (Moon, Leo), (Mars, Virgo)]),
            None
        );
        assert_eq!(
            trilochana(&[(Sun, Aries), (Moon, Taurus), (Mars, Sagittarius)]),
            None
        );
    }

    #[test]
    fn lakshmi_is_the_9th_lord_own_or_uchcha_in_a_kendra_with_a_strong_lagna_lord() {
        use Body::*;
        use Rasi::*;
        // Uchcha in the 4th from Aries.
        let row = from_aries(&[(Jupiter, Cancer)], YogaName::Lakshmi);
        assert_eq!(
            formed(row),
            (None, vec![ToJudge::Strong(Mars)], vec![Mars, Jupiter])
        );
        // Own in the 4th from Leo, whose 9th is Aries.
        let row = from_lagna(Leo, Gemini, &[(Mars, Scorpio)], YogaName::Lakshmi);
        assert_eq!(
            formed(row),
            (None, vec![ToJudge::Strong(Sun)], vec![Sun, Mars])
        );
        // Own in the 9th and the 12th, in no kendra; neither own nor uchcha in the 1st.
        for jupiter in [Sagittarius, Pisces, Aries] {
            assert_eq!(
                from_aries(&[(Jupiter, jupiter)], YogaName::Lakshmi),
                None,
                "{jupiter:?}"
            );
        }
        // Uchcha in Pi, the 2nd from Aquarius, whose 9th is Libra.
        let row = from_lagna(Aquarius, Gemini, &[(Venus, Pisces)], YogaName::Lakshmi);
        assert_eq!(row, None);
    }

    #[test]
    fn saarada_needs_all_five_of_its_conditions_and_leaves_the_suns_strength_to_judge() {
        use Body::*;
        use Rasi::*;
        let saarada = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Saarada);
        // Mercury in the 9th from the Moon in Sc, and Jupiter in the pile, her 8th, is left out.
        let whole = [
            (Saturn, Leo),
            (Mercury, Cancer),
            (Sun, Leo),
            (Mars, Aquarius),
            (Moon, Scorpio),
        ];
        assert_eq!(
            formed(saarada(&whole)),
            (
                None,
                vec![ToJudge::Strong(Sun)],
                vec![Sun, Moon, Mars, Mercury, Saturn]
            )
        );
        // Jupiter in the 9th from the Moon in Li, and Mercury in her 10th.
        let row = saarada(&[whole[0], whole[1], whole[2], whole[3], (Moon, Libra)]);
        assert_eq!(formed(row).2, [Sun, Moon, Mars, Mercury, Jupiter, Saturn]);
        // Neither is in a trine from the Moon in Vi.
        let row = saarada(&[whole[0], whole[1], whole[2], whole[3], (Moon, Virgo)]);
        assert_eq!(row, None);
        // Each other condition out in turn.
        for (i, moved) in [
            (0, (Saturn, Virgo)),
            (1, (Mercury, Virgo)),
            (2, (Sun, Virgo)),
            (3, (Mars, Virgo)),
        ] {
            let mut at = whole;
            at[i] = moved;
            assert_eq!(saarada(&at), None, "{moved:?}");
        }
    }

    #[test]
    fn saraswathi_is_all_three_in_kendras_trikonas_or_the_2nd_with_jupiter_well_placed() {
        use Body::*;
        use Rasi::*;
        let saraswathi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Saraswathi);
        // Jupiter's own rasi in the 9th.
        let at = [(Mercury, Cancer), (Jupiter, Sagittarius), (Venus, Taurus)];
        assert_eq!(present(saraswathi(&at)), [Mercury, Jupiter, Venus]);
        // Uchcha in the 4th.
        let row = saraswathi(&[(Mercury, Cancer), (Jupiter, Cancer), (Venus, Taurus)]);
        assert_eq!(present(row), [Mercury, Jupiter, Venus]);
        // Adhimitra: Mars is his friend, and in the pile in the 3rd from Ar a friend for now.
        let row = saraswathi(&[(Mercury, Cancer), (Jupiter, Aries), (Venus, Taurus)]);
        assert_eq!(present(row), [Mercury, Jupiter, Venus]);
        // Adhisatru: beside Venus, an enemy by nature, in Ta.
        let row = saraswathi(&[(Mercury, Cancer), (Jupiter, Taurus), (Venus, Taurus)]);
        assert_eq!(row, None);
        // Each of the three in the 3rd, the pile.
        assert_eq!(saraswathi(&[at[1], at[2]]), None);
        assert_eq!(saraswathi(&[at[0], at[1]]), None);
        assert_eq!(saraswathi(&[at[0], at[2]]), None);
    }

    #[test]
    fn amsaavatara_is_jupiter_venus_and_an_exalted_saturn_in_kendras() {
        use Body::*;
        use Rasi::*;
        let amsaavatara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Amsaavatara);
        let at = [(Jupiter, Cancer), (Venus, Aries), (Saturn, Libra)];
        assert_eq!(present(amsaavatara(&at)), [Jupiter, Venus, Saturn]);
        // In a kendra but not exalted.
        assert_eq!(amsaavatara(&[at[0], at[1], (Saturn, Capricorn)]), None);
        // Jupiter and Venus out of the kendras in turn.
        assert_eq!(amsaavatara(&[(Jupiter, Leo), at[1], at[2]]), None);
        assert_eq!(amsaavatara(&[at[0], (Venus, Leo), at[2]]), None);
        // Exalted, but in the 6th from Taurus.
        let at = [(Jupiter, Leo), (Venus, Taurus), (Saturn, Libra)];
        let row = from_lagna(Taurus, Gemini, &at, YogaName::Amsaavatara);
        assert_eq!(row, None);
    }

    #[test]
    fn devendra_is_a_fixed_lagna_with_two_parivartanas() {
        use Body::*;
        use Rasi::*;
        // Taurus: the 2nd and 10th lords, Mercury and Saturn; the lagna and 11th lords, Venus and Jupiter.
        let devendra = |at: &[(Body, Rasi)]| from_lagna(Taurus, Cancer, at, YogaName::Devendra);
        let whole = [
            (Mercury, Aquarius),
            (Saturn, Gemini),
            (Venus, Pisces),
            (Jupiter, Taurus),
        ];
        assert_eq!(present(devendra(&whole)), [Mercury, Jupiter, Venus, Saturn]);
        // Half of either exchange.
        assert_eq!(devendra(&whole[1..]), None);
        assert_eq!(devendra(&whole[..3]), None);
        // Both exchanges from Cancer, a movable lagna.
        let at = [(Sun, Aries), (Mars, Leo), (Moon, Taurus), (Venus, Cancer)];
        assert_eq!(from_lagna(Cancer, Gemini, &at, YogaName::Devendra), None);
    }

    #[test]
    fn indra_is_the_5th_and_11th_lords_exchanged_with_the_moon_in_the_5th() {
        use Body::*;
        use Rasi::*;
        let indra = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Indra);
        let at = [(Sun, Aquarius), (Saturn, Leo), (Moon, Leo)];
        assert_eq!(present(indra(&at)), [Sun, Moon, Saturn]);
        assert_eq!(indra(&[at[0], at[1]]), None);
        assert_eq!(indra(&[at[1], at[2]]), None);
        assert_eq!(indra(&[at[0], at[2]]), None);
    }

    #[test]
    fn the_single_ravi_yoga_is_the_sun_in_the_10th_and_its_lord_in_the_3rd_with_saturn() {
        use Body::*;
        use Rasi::*;
        let ravi = |at: &[(Body, Rasi)]| from_lagna(Gemini, Cancer, at, YogaName::Ravi);
        let at = [(Sun, Pisces), (Jupiter, Leo), (Saturn, Leo)];
        assert_eq!(present(ravi(&at)), [Sun, Jupiter, Saturn]);
        assert_eq!(ravi(&[at[0], at[1]]), None);
        assert_eq!(ravi(&[at[1], at[2]]), None);
        assert_eq!(ravi(&[at[0], at[2]]), None);
        // Taurus: Rahu, the 10th's other lord, stands with Saturn, who cannot stand with himself.
        let at = [(Sun, Aquarius), (Rahu, Cancer), (Saturn, Cancer)];
        let row = from_lagna(Taurus, Gemini, &at, YogaName::Ravi);
        assert_eq!(present(row), [Sun, Saturn, Rahu]);
        let at = [(Sun, Aquarius), (Saturn, Cancer)];
        assert_eq!(from_lagna(Taurus, Gemini, &at, YogaName::Ravi), None);
    }

    #[test]
    fn the_single_ravi_yoga_is_listed_apart_from_the_class_of_ravi_yogas() {
        use Body::*;
        use Rasi::*;
        // Venus in the Sun's 2nd adds a Vesi, of the class, to the single yoga.
        let at = [
            (Sun, Pisces),
            (Jupiter, Leo),
            (Saturn, Leo),
            (Venus, Aries),
            (Lagna, Gemini),
        ];
        let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
        let rows = yogas(Varga::RasiChart, &scene(Cancer, &at));
        let class_of = |name| rows.iter().find(|y| y.name == name).unwrap().name.class();
        assert_eq!(class_of(YogaName::Vesi), YogaClass::Ravi);
        assert_eq!(class_of(YogaName::Ravi), YogaClass::OtherPopular);
    }

    #[test]
    fn bhaaskara_counts_the_moon_and_mercury_from_the_sun_and_jupiter_from_the_moon() {
        use Body::*;
        use Rasi::*;
        let bhaaskara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Bhaaskara);
        // The Moon stays in the pile in Ge, the Sun's 12th.
        for jupiter in [Libra, Aquarius] {
            let at = [(Sun, Cancer), (Mercury, Leo), (Jupiter, jupiter)];
            assert_eq!(present(bhaaskara(&at)), [Sun, Moon, Mercury, Jupiter]);
        }
        let at = [(Sun, Cancer), (Mercury, Leo), (Jupiter, Scorpio)];
        assert_eq!(bhaaskara(&at), None);
        // Mercury in the Sun's 12th, and then the Moon in his 2nd.
        let at = [(Sun, Cancer), (Jupiter, Libra)];
        assert_eq!(bhaaskara(&at), None);
        let at = [
            (Sun, Cancer),
            (Mercury, Leo),
            (Moon, Leo),
            (Jupiter, Sagittarius),
        ];
        assert_eq!(bhaaskara(&at), None);
    }

    #[test]
    fn kulavardhana_is_each_planet_in_the_5th_from_the_lagna_moon_or_sun() {
        use Body::*;
        use Rasi::*;
        let kulavardhana = |at: &[(Body, Rasi)]| from_lagna(Aries, Leo, at, YogaName::Kulavardhana);
        // Everything in Le, the lagna's 5th; then the Sun in the Moon's 5th and Mars in the Sun's.
        assert_eq!(present(kulavardhana(&[])), PLANETS);
        let at = [(Sun, Sagittarius), (Mars, Aries)];
        assert_eq!(present(kulavardhana(&at)), PLANETS);
        // The nodes are no planets here (ADR 0014).
        let at = [(Rahu, Taurus), (Ketu, Scorpio)];
        assert_eq!(present(kulavardhana(&at)), PLANETS);
        assert_eq!(kulavardhana(&[(Jupiter, Cancer)]), None);
    }

    #[test]
    fn vasumati_is_every_benefic_in_the_upachayas_from_the_lagna_its_full_results_to_judge() {
        use Body::*;
        use Rasi::*;
        use ToJudge::{MaleficInUpachaya, Strong};
        let vasumati = |at: &[(Body, Rasi)]| from_lagna(Aries, Aries, at, YogaName::Vasumati);
        let whole = [
            (Moon, Gemini),
            (Mercury, Virgo),
            (Jupiter, Capricorn),
            (Venus, Aquarius),
        ];
        assert_eq!(
            formed(vasumati(&whole)),
            (
                None,
                vec![
                    Strong(Moon),
                    Strong(Mercury),
                    Strong(Jupiter),
                    Strong(Venus)
                ],
                vec![Moon, Mercury, Jupiter, Venus]
            )
        );
        // Every benefic, not any.
        assert_eq!(vasumati(&whole[1..]), None);
        assert_eq!(vasumati(&whole[..3]), None);
        // A malefic there is named, and Mars beside him makes Mercury kroora, so not a benefic to place.
        let at = [whole[0], whole[1], whole[2], whole[3], (Mars, Virgo)];
        assert_eq!(
            formed(vasumati(&at)),
            (
                None,
                vec![
                    Strong(Moon),
                    Strong(Jupiter),
                    Strong(Venus),
                    MaleficInUpachaya(Mars),
                    MaleficInUpachaya(Mercury)
                ],
                vec![Moon, Jupiter, Venus]
            )
        );
        // A waning Moon need not be there; the Sun in Ta has her 30° behind him, and a kroora Mercury may stay out.
        let at = [
            (Sun, Taurus),
            (Mercury, Aries),
            (Jupiter, Capricorn),
            (Venus, Aquarius),
        ];
        assert_eq!(formed(vasumati(&at)).2, [Jupiter, Venus]);
        // Counted from the lagna, not the Moon: Vi and Ta are her 3rd and 11th.
        let at = [(Moon, Cancer), (Jupiter, Virgo), (Venus, Taurus)];
        assert_eq!(vasumati(&at), None);
    }

    #[test]
    fn a_tied_mercury_never_decides_vasumati() {
        use Body::*;
        use Rasi::*;
        use ToJudge::{MaleficInUpachaya, Strong};
        // Tied beside Venus and Mars in Vi he has a benefic there, and is listed with no to-judge of his nature.
        let at = [
            (Moon, Gemini),
            (Mercury, Virgo),
            (Venus, Virgo),
            (Mars, Virgo),
            (Jupiter, Capricorn),
        ];
        assert_eq!(
            formed(from_lagna(Aries, Aries, &at, YogaName::Vasumati)),
            (
                None,
                vec![
                    Strong(Moon),
                    Strong(Mercury),
                    Strong(Jupiter),
                    Strong(Venus),
                    MaleficInUpachaya(Mars)
                ],
                vec![Moon, Mercury, Jupiter, Venus]
            )
        );
        // Tied out of them, the benefic beside him is out of them too.
        let at = [
            (Moon, Gemini),
            (Mercury, Taurus),
            (Venus, Taurus),
            (Mars, Taurus),
            (Jupiter, Capricorn),
        ];
        assert_eq!(from_lagna(Aries, Aries, &at, YogaName::Vasumati), None);
    }

    #[test]
    fn gandharva_needs_its_four_conditions_and_leaves_the_suns_strength_to_judge() {
        use Body::*;
        use Rasi::*;
        let gandharva = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Gandharva);
        // Saturn in the pile, the 9th from the 7th; Mars conjoined there by Jupiter.
        let whole = [(Sun, Aries), (Moon, Sagittarius)];
        assert_eq!(
            formed(gandharva(&whole)),
            (
                None,
                vec![ToJudge::Strong(Sun)],
                vec![Sun, Moon, Mars, Jupiter, Saturn]
            )
        );
        // Jupiter's 5th on Mars in Le.
        let at = [whole[0], whole[1], (Jupiter, Aries), (Mars, Leo)];
        assert_eq!(formed(gandharva(&at)).2, [Sun, Moon, Mars, Jupiter, Saturn]);
        // Neither conjoined nor aspected, with Jupiter's aspects on Li, Sg and Aq.
        assert_eq!(gandharva(&[whole[0], whole[1], (Mars, Taurus)]), None);
        // The 10th lord in the 2nd, no trine of the 7th.
        assert_eq!(gandharva(&[whole[0], whole[1], (Saturn, Taurus)]), None);
        // The Sun not exalted, and the Moon out of the 9th.
        assert_eq!(gandharva(&[whole[1]]), None);
        assert_eq!(gandharva(&[whole[0]]), None);
        // A Jupiter who is the lagna lord does not conjoin himself.
        let at = [(Sun, Aries), (Moon, Leo)];
        let row = from_lagna(Sagittarius, Gemini, &at, YogaName::Gandharva);
        assert_eq!(row, None);
    }

    #[test]
    fn go_is_jupiter_in_his_moolatrikona_span_with_the_2nd_lord_and_an_exalted_lagna_lord() {
        use Body::*;
        use Rasi::*;
        let go = |jupiter: f64, more: &[(Body, f64)]| {
            let mut at = vec![
                (Lagna, mid(Aries)),
                (Jupiter, jupiter),
                (Venus, mid(Sagittarius)),
                (Mars, mid(Capricorn)),
            ];
            at.extend_from_slice(more);
            yoga_in(&scene(Gemini, &at), Varga::RasiChart, YogaName::Go)
        };
        // §3.3 note (5): the first 10° of Sg, the rest his own rasi.
        assert_eq!(
            formed(go(dms(8, 5, 0, 0.0), &[])),
            (
                None,
                vec![ToJudge::Strong(Jupiter)],
                vec![Mars, Jupiter, Venus]
            )
        );
        assert!(go(dms(8, 9, 59, 0.0), &[]).is_some());
        assert_eq!(go(dms(8, 10, 0, 0.0), &[]), None);
        // The 2nd lord away; the lagna lord not exalted.
        assert_eq!(go(dms(8, 5, 0, 0.0), &[(Venus, mid(Aries))]), None);
        assert_eq!(go(dms(8, 5, 0, 0.0), &[(Mars, mid(Aries))]), None);
        // A Jupiter who is the 2nd lord is not with himself.
        let at = [
            (Lagna, mid(Scorpio)),
            (Jupiter, dms(8, 5, 0, 0.0)),
            (Mars, mid(Capricorn)),
        ];
        let chart = scene(Gemini, &at);
        assert_eq!(yoga_in(&chart, Varga::RasiChart, YogaName::Go), None);
    }

    #[test]
    fn go_in_a_varga_takes_the_whole_rasi_having_no_degrees() {
        use Body::*;
        // 28° Ar is D-9's Sagittarius, 22° Cn its Capricorn and 1° Ar its Aries.
        let at = [
            (Lagna, dms(0, 1, 0, 0.0)),
            (Jupiter, dms(0, 28, 0, 0.0)),
            (Venus, dms(0, 28, 0, 0.0)),
            (Mars, dms(3, 22, 0, 0.0)),
        ];
        let row = yoga_in(&scene(Rasi::Gemini, &at), Varga::Navamsa, YogaName::Go);
        assert_eq!(
            formed(row),
            (
                None,
                vec![ToJudge::Strong(Jupiter)],
                vec![Mars, Jupiter, Venus]
            )
        );
    }

    #[test]
    fn chapa_is_the_4th_and_10th_lords_exchanged_with_an_exalted_lagna_lord() {
        use Body::*;
        use Rasi::*;
        let chapa = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Chapa);
        let at = [(Moon, Capricorn), (Saturn, Cancer), (Mars, Capricorn)];
        assert_eq!(present(chapa(&at)), [Moon, Mars, Saturn]);
        assert_eq!(chapa(&[at[0], at[1]]), None);
        assert_eq!(chapa(&[at[0], at[2]]), None);
        assert_eq!(chapa(&[at[1], at[2]]), None);
    }

    #[test]
    fn chaapa_of_the_naabhasa_is_not_chapa_of_the_other_popular_yogas() {
        assert_ne!(YogaName::Chaapa, YogaName::Chapa);
        assert_eq!(YogaName::Chaapa.class(), YogaClass::Naabhasa);
        assert_eq!(YogaName::Chapa.class(), YogaClass::OtherPopular);
    }

    #[test]
    fn pushkala_is_its_four_stated_conditions() {
        use Body::*;
        use Rasi::*;
        let pushkala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Pushkala);
        // Mars with the Moon in Li; Venus, her dispositor, in the 7th aspecting the lagna; the Sun in it.
        let whole = [(Mars, Libra), (Moon, Libra), (Venus, Libra), (Sun, Aries)];
        assert_eq!(present(pushkala(&whole)), [Sun, Moon, Mars, Venus]);
        // Each condition out: Mars away, Venus not aspecting, and no planet in the lagna, though nodes may be.
        assert_eq!(pushkala(&whole[1..]), None);
        let at = [whole[0], whole[1], (Venus, Cancer), whole[3]];
        assert_eq!(pushkala(&at), None);
        assert_eq!(pushkala(&whole[..3]), None);
        assert_eq!(
            pushkala(&[whole[0], whole[1], whole[2], (Rahu, Aries)]),
            None
        );
        // A Cancer lagna's lord is the Moon, who is not with herself.
        let at = [(Sun, Cancer), (Moon, Scorpio), (Mars, Capricorn)];
        assert_eq!(from_lagna(Cancer, Gemini, &at, YogaName::Pushkala), None);
    }

    #[test]
    fn pushkalas_dispositor_may_stand_out_of_a_kendra_in_an_adhimitras_house() {
        use Body::*;
        use Rasi::*;
        let pushkala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Pushkala);
        // Jupiter, the Moon's dispositor from Sg, in the 5th: his 9th is on the lagna, and his friend the Sun is in Le's 2nd.
        let at = [
            (Moon, Sagittarius),
            (Mars, Sagittarius),
            (Jupiter, Leo),
            (Venus, Aries),
            (Sun, Virgo),
        ];
        assert_eq!(present(pushkala(&at)), [Moon, Mars, Jupiter, Venus]);
        // In Le's 5th the Sun is a friend by nature but an enemy for now, and so only sama.
        let at = [at[0], at[1], at[2], at[3], (Sun, Sagittarius)];
        assert_eq!(pushkala(&at), None);
    }

    #[test]
    fn makuta_is_jupiter_9th_from_the_9th_lord_a_benefic_9th_from_him_and_saturn_in_the_10th() {
        use Body::*;
        use Rasi::*;
        // Taurus: Saturn, the 9th lord, in Aq, Jupiter 9th from him in Li, and Ge 9th from Jupiter.
        let makuta = |rest, at: &[(Body, Rasi)]| from_lagna(Taurus, rest, at, YogaName::Makuta);
        let whole = [(Saturn, Aquarius), (Jupiter, Libra)];
        assert_eq!(
            present(makuta(Gemini, &whole)),
            [Moon, Jupiter, Venus, Saturn]
        );
        // No benefic in Ge.
        assert_eq!(makuta(Cancer, &whole), None);
        // A Mercury alone there is one, and with Rahu beside him is not.
        let at = [whole[0], whole[1], (Mercury, Gemini)];
        assert_eq!(present(makuta(Cancer, &at)), [Mercury, Jupiter, Saturn]);
        let at = [whole[0], whole[1], (Mercury, Gemini), (Rahu, Gemini)];
        assert_eq!(makuta(Cancer, &at), None);
        // Jupiter elsewhere; Saturn out of the 10th, with Jupiter still 9th from him.
        assert_eq!(makuta(Gemini, &[whole[0], (Jupiter, Virgo)]), None);
        let at = [(Saturn, Pisces), (Jupiter, Scorpio), (Venus, Cancer)];
        assert_eq!(makuta(Gemini, &at), None);
        // An Aries lagna's 9th lord is Jupiter himself, never in his own 9th.
        assert_eq!(
            from_aries(&[(Saturn, Capricorn), (Jupiter, Leo)], YogaName::Makuta),
            None
        );
    }

    /// `name`'s row in the rasi chart with the lagna at 15° of `lagna`, the rest in Gemini, but for `at`.
    fn with_degrees(lagna: Rasi, at: &[(Body, f64)], name: YogaName) -> Option<Yoga> {
        let mut at = at.to_vec();
        at.push((Body::Lagna, mid(lagna)));
        yoga_in(&scene(Rasi::Gemini, &at), Varga::RasiChart, name)
    }

    #[test]
    fn vishnu_is_the_9th_and_10th_lords_and_the_9th_lords_navamsa_dispositor_in_the_2nd() {
        use Body::*;
        use Rasi::*;
        let vishnu = |at: &[(Body, f64)]| with_degrees(Aries, at, YogaName::Vishnu);
        // Jupiter at 15° Ta is in D-9's Taurus, so Venus.
        let whole = [
            (Jupiter, mid(Taurus)),
            (Saturn, mid(Taurus)),
            (Venus, mid(Taurus)),
        ];
        assert_eq!(present(vishnu(&whole)), [Jupiter, Venus, Saturn]);
        assert_eq!(vishnu(&whole[1..]), None);
        assert_eq!(vishnu(&[whole[0], whole[2]]), None);
        assert_eq!(vishnu(&whole[..2]), None);
        // At 1° Ta he is in D-9's Capricorn, so Saturn, and Venus has no part.
        let at = [(Jupiter, dms(1, 1, 0, 0.0)), (Saturn, mid(Taurus))];
        assert_eq!(present(vishnu(&at)), [Jupiter, Saturn]);
    }

    #[test]
    fn gouri_is_the_10th_lords_navamsa_dispositor_exalted_in_the_10th_with_the_lagna_lord() {
        use Body::*;
        use Rasi::*;
        let gouri = |at: &[(Body, f64)]| with_degrees(Libra, at, YogaName::Gouri);
        // The Moon rules Libra's 10th; at 28° Ar she is in D-9's Sagittarius, so Jupiter, exalted in Cn beside Venus.
        let whole = [
            (Moon, dms(0, 28, 0, 0.0)),
            (Jupiter, mid(Cancer)),
            (Venus, mid(Cancer)),
        ];
        assert_eq!(present(gouri(&whole)), [Moon, Jupiter, Venus]);
        // Her navamsa elsewhere, Jupiter out of Cancer, and Venus away.
        assert_eq!(gouri(&[(Moon, mid(Taurus)), whole[1], whole[2]]), None);
        assert_eq!(gouri(&[whole[0], (Jupiter, mid(Leo)), whole[2]]), None);
        assert_eq!(gouri(&[whole[0], whole[1], (Venus, mid(Leo))]), None);
        // Aries: Saturn at 15° Cn is in D-9's Scorpio, so Mars exalted in the 10th, but he is the lagna lord and joins no one.
        let at = [(Saturn, mid(Cancer)), (Mars, mid(Capricorn))];
        assert_eq!(with_degrees(Aries, &at, YogaName::Gouri), None);
    }

    #[test]
    fn chandikaa_is_a_fixed_lagna_aspected_by_the_6th_lord_with_the_sun_beside_the_dispositors() {
        use Body::*;
        use Rasi::*;
        let chandikaa = |at: &[(Body, f64)]| with_degrees(Leo, at, YogaName::Chandikaa);
        // Leo: Saturn rules the 6th, and his 10th from Sc is on it. In D-9's Scorpio he gives Mars, and Mars, the 9th lord, in D-9's
        // Taurus gives Venus.
        let whole = [
            (Saturn, mid(Scorpio)),
            (Mars, mid(Taurus)),
            (Sun, mid(Taurus)),
            (Venus, mid(Taurus)),
        ];
        assert_eq!(present(chandikaa(&whole)), [Sun, Mars, Venus, Saturn]);
        // Saturn in Ta aspects Cn, Sc and Aq, and both dispositors are then Venus.
        let at = [(Saturn, mid(Taurus)), whole[1], whole[2], whole[3]];
        assert_eq!(chandikaa(&at), None);
        // Venus or the Sun away.
        assert_eq!(
            chandikaa(&[whole[0], whole[1], whole[2], (Venus, mid(Aries))]),
            None
        );
        assert_eq!(
            chandikaa(&[whole[0], whole[1], (Sun, mid(Aries)), whole[3]]),
            None
        );
        // Aries is movable, though Mercury in Li aspects it and the pile holds the Sun and D-9 Aquarius's lords.
        let at = [(Mercury, mid(Libra))];
        assert_eq!(with_degrees(Aries, &at, YogaName::Chandikaa), None);
    }

    #[test]
    fn the_sun_as_one_of_chandikaas_dispositors_leaves_the_other_but_not_as_both() {
        use Body::*;
        use Rasi::*;
        let chandikaa = |at: &[(Body, f64)]| with_degrees(Leo, at, YogaName::Chandikaa);
        // Mars at 15° Ar is in D-9's Leo, so the Sun, who is beside Mars, Saturn's dispositor.
        let at = [
            (Saturn, mid(Scorpio)),
            (Mars, mid(Aries)),
            (Sun, mid(Aries)),
        ];
        assert_eq!(present(chandikaa(&at)), [Sun, Mars, Saturn]);
        // Saturn at 5° Sc is in D-9's Leo as well, so the Sun would join only himself.
        let at = [
            (Saturn, dms(7, 5, 0, 0.0)),
            (Mars, mid(Aries)),
            (Sun, mid(Aries)),
        ];
        assert_eq!(chandikaa(&at), None);
    }

    #[test]
    fn bhaarathi_is_a_navamsa_dispositor_of_the_2nd_5th_or_11th_lord_exalted_with_the_9th_lord() {
        use Body::*;
        use Rasi::*;
        let bhaarathi = |at: &[(Body, f64)]| with_degrees(Aries, at, YogaName::Bhaarathi);
        // At 15° Cn each lord is in D-9's Scorpio, so Mars, exalted in Cp beside Jupiter, the 9th lord.
        let exalted_pair = [(Mars, mid(Capricorn)), (Jupiter, mid(Capricorn))];
        for (lord, expected) in [
            (Venus, [Mars, Jupiter, Venus]),
            (Sun, [Sun, Mars, Jupiter]),
            (Saturn, [Mars, Jupiter, Saturn]),
        ] {
            let at = [(lord, mid(Cancer)), exalted_pair[0], exalted_pair[1]];
            assert_eq!(present(bhaarathi(&at)), expected, "{lord:?}");
        }
        // Mercury rules the 3rd and 6th, no house of the rule.
        let at = [(Mercury, mid(Cancer)), exalted_pair[0], exalted_pair[1]];
        assert_eq!(bhaarathi(&at), None);
        let lord = (Saturn, mid(Cancer));
        assert_eq!(
            bhaarathi(&[lord, (Mars, mid(Aries)), exalted_pair[1]]),
            None
        );
        assert_eq!(
            bhaarathi(&[lord, exalted_pair[0], (Jupiter, mid(Aries))]),
            None
        );
        // At 28° Ar Saturn is in D-9's Sagittarius, so Jupiter exalted, but he is the 9th lord and joins no one.
        let at = [(Saturn, dms(0, 28, 0, 0.0)), (Jupiter, mid(Cancer))];
        assert_eq!(bhaarathi(&at), None);
    }

    #[test]
    fn vidyut_is_the_11th_lord_exalted_with_venus_in_a_kendra_from_the_lagna_lord_to_judge_at_any_degree()
     {
        use Body::*;
        use Rasi::*;
        let vidyut = |saturn: f64, more: &[(Body, f64)]| {
            let mut at = vec![(Saturn, saturn), (Venus, mid(Libra)), (Mars, mid(Aries))];
            at.extend_from_slice(more);
            with_degrees(Aries, &at, YogaName::Vidyut)
        };
        // At his deep degree and far from it alike.
        for degree in [dms(6, 20, 0, 0.0), dms(6, 1, 0, 0.0), dms(6, 29, 0, 0.0)] {
            assert_eq!(
                formed(vidyut(degree, &[])),
                (
                    None,
                    vec![ToJudge::DeepExaltation(Saturn)],
                    vec![Mars, Venus, Saturn]
                )
            );
        }
        // Out of his uchcha rasi, even with Venus beside him.
        assert_eq!(vidyut(mid(Capricorn), &[(Venus, mid(Capricorn))]), None);
        let saturn = dms(6, 20, 0, 0.0);
        // Venus away, and the lagna lord in Ta, from whom Li is the 6th.
        assert_eq!(vidyut(saturn, &[(Venus, mid(Scorpio))]), None);
        assert_eq!(vidyut(saturn, &[(Mars, mid(Taurus))]), None);
        // Cancer: Venus rules the 11th and joins no one.
        let at = [(Venus, mid(Pisces)), (Moon, mid(Pisces))];
        assert_eq!(with_degrees(Cancer, &at, YogaName::Vidyut), None);
        // Taurus: Venus is the lagna lord too, leaving Jupiter in a kendra from her, as with Bheri (ADR 0014).
        let at = [(Jupiter, mid(Cancer)), (Venus, mid(Cancer))];
        let row = with_degrees(Taurus, &at, YogaName::Vidyut);
        assert_eq!(
            formed(row),
            (
                None,
                vec![ToJudge::DeepExaltation(Jupiter)],
                vec![Jupiter, Venus]
            )
        );
    }

    #[test]
    fn jaya_is_the_10th_lord_exalted_and_the_6th_lord_debilitated_to_judge_at_any_degree() {
        use Body::*;
        use Rasi::*;
        let jaya = |saturn: f64, mercury: Rasi| {
            let at = [(Saturn, saturn), (Mercury, mid(mercury))];
            with_degrees(Aries, &at, YogaName::Jaya)
        };
        for degree in [dms(6, 20, 0, 0.0), dms(6, 1, 0, 0.0)] {
            assert_eq!(
                formed(jaya(degree, Pisces)),
                (
                    None,
                    vec![ToJudge::DeepExaltation(Saturn)],
                    vec![Mercury, Saturn]
                )
            );
        }
        assert_eq!(jaya(mid(Capricorn), Pisces), None);
        assert_eq!(jaya(dms(6, 20, 0, 0.0), Virgo), None);
    }

    #[test]
    fn the_rasi_only_six_are_never_in_a_vargas_list() {
        use Body::*;
        use Rasi::*;
        // Every place below is the same rasi in D-9, so each rule would hold there too but for its being rasi-only.
        let first = |r: Rasi| dms(r as u8, 1, 0, 0.0);
        let last = |r: Rasi| dms(r as u8, 28, 0, 0.0);
        let cases = [
            (
                YogaName::Vishnu,
                vec![
                    (Lagna, first(Aries)),
                    (Jupiter, mid(Taurus)),
                    (Saturn, mid(Taurus)),
                    (Venus, mid(Taurus)),
                ],
            ),
            (
                YogaName::Gouri,
                vec![
                    (Lagna, first(Libra)),
                    (Moon, last(Sagittarius)),
                    (Jupiter, first(Cancer)),
                    (Venus, first(Cancer)),
                ],
            ),
            (
                YogaName::Chandikaa,
                vec![
                    (Lagna, mid(Leo)),
                    (Saturn, mid(Scorpio)),
                    (Mars, mid(Taurus)),
                    (Sun, mid(Taurus)),
                    (Venus, mid(Taurus)),
                ],
            ),
            (
                YogaName::Bhaarathi,
                vec![
                    (Lagna, first(Aries)),
                    (Saturn, mid(Scorpio)),
                    (Mars, first(Capricorn)),
                    (Jupiter, first(Capricorn)),
                ],
            ),
            (
                YogaName::Vidyut,
                vec![
                    (Lagna, first(Aries)),
                    (Mars, first(Aries)),
                    (Saturn, first(Libra)),
                    (Venus, first(Libra)),
                ],
            ),
            (
                YogaName::Jaya,
                vec![
                    (Lagna, first(Aries)),
                    (Saturn, first(Libra)),
                    (Mercury, last(Pisces)),
                ],
            ),
        ];
        for (name, at) in cases {
            let chart = scene(Gemini, &at);
            assert!(
                yoga_in(&chart, Varga::RasiChart, name).is_some(),
                "{name:?}"
            );
            for d in VARGAS.into_iter().filter(|&d| d != Varga::RasiChart) {
                assert_eq!(yoga_in(&chart, d, name), None, "{name:?} in {d:?}");
            }
        }
    }

    #[test]
    fn vishnu_to_vimala_come_in_the_books_order() {
        use Body::*;
        use Rasi::*;
        use YogaName as N;
        let names_in = |at: &[(Body, Rasi)]| -> Vec<YogaName> {
            let mut at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
            at.push((Lagna, mid(Aries)));
            yogas(Varga::RasiChart, &scene(Gemini, &at))
                .into_iter()
                .map(|y| y.name)
                .filter(|y| {
                    [
                        N::Trilochana,
                        N::Lakshmi,
                        N::Amsaavatara,
                        N::Vidyut,
                        N::Jaya,
                        N::Harsha,
                        N::Sarala,
                        N::Vimala,
                    ]
                    .contains(y)
                })
                .collect()
        };
        let at = [
            (Sun, Aries),
            (Moon, Leo),
            (Mars, Aries),
            (Mercury, Pisces),
            (Jupiter, Cancer),
            (Venus, Libra),
            (Saturn, Libra),
        ];
        assert_eq!(
            names_in(&at),
            [
                N::Trilochana,
                N::Lakshmi,
                N::Amsaavatara,
                N::Vidyut,
                N::Jaya
            ]
        );
        let at = [
            (Sun, Pisces),
            (Moon, Pisces),
            (Mars, Scorpio),
            (Mercury, Virgo),
            (Jupiter, Pisces),
        ];
        assert_eq!(
            names_in(&at),
            [N::Trilochana, N::Harsha, N::Sarala, N::Vimala]
        );
    }

    #[test]
    fn yogas_come_in_the_books_order() {
        use Body::*;
        use Rasi::*;
        // Mars in the Sun's 12th and Jupiter in his 2nd; Mercury in the Moon's 2nd, and nothing in her 12th.
        let at = [
            (Lagna, Aquarius),
            (Sun, Scorpio),
            (Mercury, Scorpio),
            (Moon, Libra),
            (Mars, Libra),
            (Jupiter, Sagittarius),
            (Saturn, Aquarius),
            (Venus, Taurus),
        ];
        let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
        let names: Vec<YogaName> = yogas(Varga::RasiChart, &scene(Gemini, &at))
            .into_iter()
            .map(|y| y.name)
            .collect();
        assert_eq!(
            names,
            [
                YogaName::Ubhayachara,
                YogaName::BudhaAaditya,
                YogaName::Sunaphaa,
                YogaName::ChandraMangala,
                YogaName::Sasa,
                YogaName::Maalavya,
                YogaName::Paasa,
                YogaName::Asubha,
                YogaName::Mridanga,
                YogaName::Kalpadruma,
                YogaName::Brahma,
                YogaName::Lakshmi,
            ]
        );
    }

    #[test]
    fn naabhasa_yogas_come_in_the_books_order() {
        use YogaName::*;
        // Dala before Aakriti, and Vajra between Hala and Kamala: Maalaa beside Sarpa, then Vajra and Kamala.
        assert_eq!(
            naabhasa_names([1, 4, 7, 10, 7, 1, 10], 4),
            [Rajju, Maalaa, Sarpa, Vajra, Kamala]
        );
    }
}

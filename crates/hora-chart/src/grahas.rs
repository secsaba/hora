use crate::*;

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

/// A graha's sex; Mercury and Saturn are female as printed (ch. 3 §3.2.7).
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

/// The §3.2 lists of ch. 3; each is `None` for the lagna and for a graha the book leaves out, never filled from elsewhere.
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
        // Time strength is the seven planets', though §3.2.2 makes the nodes natural malefics.
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
            // "First 12º of Le" in §3.3 (3) is a typo for Aries.
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

impl VargaChakra<'_> {
    /// `graha`'s natural nature in `d`'s chart, only ever `Saumya` or `Kroora`: the Moon's by paksha, Mercury's by his company
    /// there with the nodes as malefics, and none on a tie, which §3.2.2 has no rule for, or for a lagna (ch. 3 §3.2.2, ADR 0014).
    pub fn natural_nature(&self, graha: Body) -> Option<Disposition> {
        use Disposition::*;
        match graha.disposition()? {
            ByPaksha => {
                let longitude = |body| self.chart.entry(body).longitude;
                // The paksha is the day's, the same in every varga.
                Some(match tithi(longitude(Body::Sun), longitude(Body::Moon)).1 {
                    Paksha::Sukla => Saumya,
                    Paksha::Krishna => Kroora,
                })
            }
            ByCompany => {
                let company = self.grahas_in(self.rasi_of(graha));
                let count = |nature| {
                    company
                        .iter()
                        .filter(|&&g| g != graha && self.natural_nature(g) == Some(nature))
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
}

#[cfg(test)]
mod tests;

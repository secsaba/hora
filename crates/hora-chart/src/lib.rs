//! The chart prepared from a Sky, in the book's terms (ch. 1 *Basic Concepts*); no web code.

pub use hora_swe::arcseconds;
use hora_swe::{KARANA, NAKSHATRA, NewMoon, Position, Sky, TITHI, VedicDay, arcseconds_from_aries};

mod aspects;
mod grahas;
mod panchanga;
mod rasis;
mod special_lagnas;
mod strength;
mod upagrahas;
mod vargas;
pub use aspects::*;
pub use grahas::*;
pub use panchanga::*;
pub use rasis::*;
pub use special_lagnas::*;
pub use strength::*;
pub use upagrahas::*;
pub use vargas::*;

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

/// The lagna and the grahas, each with where it is, and the special lagnas (glossary: Chart).
// shortcut: upagrahas are not in the Chart yet, add them when a technique reads them.
#[derive(Debug, Clone)]
pub struct Chart {
    // The lagna, then the grahas in the book's order, so `Body as usize` indexes it.
    entries: [Entry; 10],
    special_lagnas: Vec<(SpecialLagna, f64)>,
}

impl Chart {
    /// A chart of `entries`, the lagna then the nine grahas in the book's order, and `special_lagnas` in
    /// [`special_lagnas`]'s order. Panics on entries out of that order, a programmer's error; Ketu is not checked to be
    /// opposite Rahu, so tests may place him freely.
    pub fn new(entries: [Entry; 10], special_lagnas: Vec<(SpecialLagna, f64)>) -> Chart {
        let order = std::iter::once(Body::Lagna).chain(GRAHAS);
        assert!(
            entries.iter().map(|e| e.body).eq(order),
            "a chart's entries are the lagna, then the grahas in the book's order"
        );
        Chart {
            entries,
            special_lagnas,
        }
    }

    /// Every entry in display order: the lagna, then the grahas in the book's order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn lagna(&self) -> &Entry {
        self.entry(Body::Lagna)
    }

    /// The nine grahas in the book's order.
    pub fn grahas(&self) -> &[Entry] {
        &self.entries[1..]
    }

    pub fn entry(&self, body: Body) -> &Entry {
        &self.entries[body as usize]
    }

    /// The special lagnas with their longitudes, in [`special_lagnas`]'s order.
    pub fn special_lagnas(&self) -> &[(SpecialLagna, f64)] {
        &self.special_lagnas
    }

    /// `which`'s longitude, if the chart has it: HL, GL and BL need a Vedic day.
    pub fn special_lagna(&self, which: SpecialLagna) -> Option<f64> {
        self.special_lagnas
            .iter()
            .find(|s| s.0 == which)
            .map(|s| s.1)
    }
}

/// The chart of `sky`: the lagna, then the nine grahas in the book's order, and its special lagnas.
pub fn chart(sky: &Sky) -> Chart {
    let g = &sky.grahas;
    let entry = |body, longitude, retrograde| Entry {
        body,
        longitude,
        retrograde,
    };
    // The same rule for all nine, so the mean nodes always carry R (glossary: Retrograde).
    let graha = |body, p: Position| entry(body, p.longitude, p.speed < 0.0);
    let entries = [
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
    ];
    Chart::new(entries, special_lagnas(sky))
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
    let (index, degrees) = hora_swe::rasi_index(reduce(longitude));
    (RASIS[index], degrees)
}

/// The bhava, 1 to 12, that `rasi` is counted from the rasi holding the reference point; Pisces is followed by Aries (ch. 1 §1.3.3).
pub fn bhava(reference: Rasi, rasi: Rasi) -> u8 {
    (rasi as u8 + 12 - reference as u8) % 12 + 1
}

impl Chart {
    /// The varga chakra `d` of this chart; the rasi chart is D-1 (glossary: Varga chakra).
    pub fn in_varga(&self, d: Varga) -> VargaChakra<'_> {
        VargaChakra { d, chart: self }
    }

    /// `body`'s compound relationship with the lord of the rasi it occupies; none in its own rasi, or for Rahu, Ketu
    /// and the lagna (ch. 3 §3.4.3). Separate from its dignity: Saturn in Aries is neecha whatever he is to Mars.
    pub fn lord_relationship(&self, body: Body) -> Option<Compound> {
        self.in_varga(Varga::RasiChart).with_rasi_lord(body)
    }
}

/// A varga chakra of a chart: each placement worked out when asked, nothing stored, since every varga is a cheap pure
/// function of the longitude (ADR 0009).
#[derive(Debug, Clone, Copy)]
pub struct VargaChakra<'a> {
    d: Varga,
    chart: &'a Chart,
}

impl VargaChakra<'_> {
    /// The rasi holding `body`.
    pub fn rasi_of(&self, body: Body) -> Rasi {
        varga(self.d, self.chart.entry(body).longitude)
    }

    /// Paaka lagna: the rasi the lagna's lord occupies, both read in `d`'s own chart, since it "applies to all
    /// divisional charts" (ch. 7 §7.3.5, ADR 0003).
    pub fn paaka_lagna(&self) -> Rasi {
        // shortcut: Scorpio takes Mars and Aquarius Saturn, not the stronger co-lord; revisit with ch. 15's decision (ADR 0003).
        self.rasi_of(self.rasi_of(Body::Lagna).lord())
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

impl VargaChakra<'_> {
    /// The bhava arudhas of `d`'s chart, from AL (the 1st bhava) to UL (the 12th), counted from that chart's lagna, each bhava's
    /// lord being its stronger lord (ch. 9 §9.2, ADR 0012).
    pub fn bhava_arudhas(&self) -> [Rasi; 12] {
        let lagna = self.rasi_of(Body::Lagna);
        std::array::from_fn(|i| {
            let house = kth(lagna, i + 1);
            arudha_pada(house, self.rasi_of(self.stronger_lord(house)))
        })
    }

    /// The graha arudhas of `d`'s chart, the Sun to Ketu, each counted through the rasi its graha owns, or the stronger of its two
    /// (ch. 9 §9.5, ADR 0012).
    pub fn graha_arudhas(&self) -> [(Body, Rasi); 9] {
        GRAHAS.map(|graha| {
            // Every graha has dignities and owns a rasi (Table 6).
            let rasis = graha.dignities().unwrap().own.iter().copied();
            let owned = rasis.reduce(|a, b| self.stronger_rasi(a, b)).unwrap();
            (graha, arudha_pada(self.rasi_of(graha), owned))
        })
    }
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

impl Chart {
    /// The chara karaka of the Sun to Saturn and Rahu, in the book's graha order, ranked by advancement in their rasis
    /// in whole arcseconds, Rahu's measured back from his rasi's end (ch. 8 §8.2, ADR 0011). From the rasi chart only;
    /// Ketu and the lagna have none.
    pub fn chara_karakas(&self) -> [(Body, CharaKaraka); 8] {
        // GRAHAS ends with Ketu, who holds none.
        let grahas: [Body; 8] = std::array::from_fn(|i| GRAHAS[i]);
        let advanced = grahas.map(|g| advancement(self, g));
        // Ranking by how many are strictly ahead: grahas tied share a karakatwa and the next is left empty (ADR 0011).
        std::array::from_fn(|i| {
            let ahead = advanced.iter().filter(|&&a| a > advanced[i]).count();
            (grahas[i], CHARA_KARAKAS[ahead])
        })
    }

    /// The seven sthira karakas in §8.3's order, from the rasi chart: the father the stronger of the Sun and Venus, the mother
    /// of the Moon and Mars, by §15.5.1's rules 1–4 and 5b (ch. 8 §8.3, pp. 81–82; ch. 15 §15.5.1, pp. 202–203; ADR 0018).
    pub fn sthira_karakas(&self) -> [SthiraKaraka; 7] {
        use Body::*;
        use Kin::*;
        let rasi_chart = self.in_varga(Varga::RasiChart);
        let stronger = |kin, a, b| {
            let (graha, rule) = rasi_chart.stronger_planet_by(a, b);
            SthiraKaraka { kin, graha, rule }
        };
        let fixed = |kin, graha| SthiraKaraka {
            kin,
            graha,
            rule: None,
        };
        [
            stronger(Father, Sun, Venus),
            stronger(Mother, Moon, Mars),
            fixed(YoungerSiblings, Mars),
            fixed(MaternalRelatives, Mercury),
            fixed(Husband, Jupiter),
            fixed(Wife, Venus),
            fixed(ElderSiblings, Saturn),
        ]
    }
}

/// The relatives of a sthira karaka, one per line of §8.3 (ch. 8 §8.3, pp. 81–82).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kin {
    Father,
    Mother,
    /// Younger siblings and the spouses of siblings: Mars.
    YoungerSiblings,
    /// Maternal uncles and aunts: Mercury.
    MaternalRelatives,
    /// The husband in a woman's chart, sons, paternal grandparents and other paternal relatives: Jupiter.
    Husband,
    /// The wife in a man's chart, parents-in-law and maternal grandparents: Venus.
    Wife,
    /// Elder siblings: Saturn.
    ElderSiblings,
}

/// A sthira karaka (ch. 8 §8.3, ADR 0018).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SthiraKaraka {
    pub kin: Kin,
    pub graha: Body,
    /// For the father and mother, the rule that made `graha` the stronger, None on a tie through rule 5b, which takes the Sun or
    /// the Moon; None for the other five.
    pub rule: Option<StrongerRule>,
}

/// What bhavas are counted from (ADR 0003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferencePoint {
    Body(Body),
    Special(SpecialLagna),
    PaakaLagna,
    AtmaKaraka,
    /// The arudha pada of that bhava.
    Arudha(Bhava),
    Relative(Relative),
}

/// A bhava, from 1 (the lagna's) to 12.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bhava(u8);

impl Bhava {
    /// `None` outside 1 to 12.
    pub const fn new(n: u8) -> Option<Self> {
        if 1 <= n && n <= 12 {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// A relative whose lagna is the rasi of a house's lord (ch. 13 §13.4.2, ADR 0016).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relative {
    Father,
    Mother,
    Spouse,
    YoungerSibling(Nth),
    ElderSibling(Nth),
    Child(Nth),
}

/// Which sibling or child (§13.4.2 reads up to the 3rd).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nth {
    First,
    Second,
    Third,
}

impl VargaChakra<'_> {
    /// `point`'s rasi in `d`'s chart, each point read in that chart's own terms: "the 10th house from AL in D-10" (ch. 9 §9.3,
    /// ADR 0003). `None` for HL and GL without a Vedic day (ch. 5).
    pub fn reference(&self, point: ReferencePoint) -> Option<Rasi> {
        Some(match point {
            ReferencePoint::Body(body) => self.rasi_of(body),
            ReferencePoint::Special(lagna) => varga(self.d, self.chart.special_lagna(lagna)?),
            ReferencePoint::PaakaLagna => self.paaka_lagna(),
            // shortcut: a tie for AK (ch. 8 §8.2) counts from the first in graha order, upgrade when a later chapter settles it.
            ReferencePoint::AtmaKaraka => {
                let karakas = self.chart.chara_karakas();
                // The most advanced graha has none ahead, so AK is always held.
                let ak = karakas.iter().find(|k| k.1 == CharaKaraka::Atma).unwrap();
                self.rasi_of(ak.0)
            }
            ReferencePoint::Arudha(Bhava(n)) => self.bhava_arudhas()[usize::from(n - 1)],
            ReferencePoint::Relative(who) => {
                let lagna = self.rasi_of(Body::Lagna);
                let (house, reversible) = match who {
                    Relative::Father => (9, false),
                    Relative::Mother => (4, false),
                    Relative::Spouse => (7, false),
                    Relative::YoungerSibling(n) => (3 + 2 * n as usize, true),
                    Relative::ElderSibling(n) => (11 - 2 * n as usize, true),
                    Relative::Child(n) => (5 + 2 * n as usize, true),
                };
                // Siblings and children count backward from an even lagna (§13.4.2); 14 − k is the kth counted backward.
                let house = if reversible && lagna.parity() == Parity::Sama {
                    14 - house
                } else {
                    house
                };
                self.rasi_of(self.stronger_lord(kth(lagna, house)))
            }
        })
    }
}

impl VargaChakra<'_> {
    /// A rasi's lords as a yoga counts them: its lord, or either co-lord of Scorpio or Aquarius, but only the other if one is in
    /// the rasi, and either if both are (Parasara as the book gives him on p. 218, ADR 0014).
    pub fn lords_in_yoga(&self, rasi: Rasi) -> Vec<Body> {
        let outside: Vec<Body> = lords(rasi).filter(|&g| self.rasi_of(g) != rasi).collect();
        if outside.is_empty() {
            lords(rasi).collect()
        } else {
            outside
        }
    }

    /// The lords a §11.6 rule tries, in turn: [`VargaChakra::lords_in_yoga`]'s, then a seven-planet co-lord at home, since "Saturn also owns
    /// 5th" (p. 218, ADR 0014). A node at home is not tried: the book says that only of Saturn.
    fn lords_to_try(&self, rasi: Rasi) -> Vec<Body> {
        let mut lords_tried = self.lords_in_yoga(rasi);
        for lord in lords(rasi) {
            if PLANETS.contains(&lord) && !lords_tried.contains(&lord) {
                lords_tried.push(lord);
            }
        }
        lords_tried
    }
}
/// A planet's functional nature from a lagna (ch. 13 §13.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionalNature {
    Benefic,
    Neutral,
    Malefic,
}

/// Table 30 (ch. 13 §13.2, p. 167) as printed, in rasi order: the yogakaraka, then the functional benefics, neutrals and malefics.
/// Not derived from §13.2's rules, which five cells break, as Parasara does (ADR 0016).
const TABLE_30: [(Option<Body>, [&[Body]; 3]); 12] = {
    use Body::*;
    [
        (
            None,
            [&[Sun, Mars, Jupiter], &[], &[Mercury, Venus, Saturn]],
        ),
        (
            Some(Saturn),
            [&[Sun, Mercury, Saturn], &[Mars], &[Moon, Jupiter, Venus]],
        ),
        (
            None,
            [&[Venus], &[Moon, Mercury, Saturn], &[Sun, Mars, Jupiter]],
        ),
        (
            Some(Mars),
            [&[Moon, Mars, Jupiter], &[Sun, Saturn], &[Mercury, Venus]],
        ),
        (
            Some(Mars),
            [&[Sun, Mars, Jupiter], &[Moon], &[Mercury, Venus, Saturn]],
        ),
        (
            None,
            [&[Mercury, Venus], &[Sun, Saturn], &[Moon, Mars, Jupiter]],
        ),
        (
            Some(Saturn),
            [&[Mercury, Venus, Saturn], &[], &[Sun, Mars, Jupiter]],
        ),
        (
            None,
            [&[Moon, Jupiter], &[Sun, Mars], &[Mercury, Venus, Saturn]],
        ),
        (
            None,
            [&[Sun, Mars], &[Moon, Mercury, Jupiter], &[Venus, Saturn]],
        ),
        (
            Some(Venus),
            [&[Venus, Mercury, Saturn], &[Sun], &[Mars, Jupiter]],
        ),
        (
            Some(Venus),
            [&[Venus, Saturn], &[Sun, Mercury], &[Moon, Mars, Jupiter]],
        ),
        (
            None,
            [&[Moon, Mars], &[Jupiter], &[Sun, Mercury, Venus, Saturn]],
        ),
    ]
};

impl VargaChakra<'_> {
    /// `graha`'s functional nature by Table 30, from `from`'s rasi in `d`'s chart ("yogakaraka from Moon", p. 221; "a functional
    /// benefic in dasamsa", p. 219). The Moon the table leaves out is a malefic in Sukla paksha, neutral in Krishna (§13.2,
    /// pp. 166–167). `None` for the nodes, a lagna, or a reference point with no rasi (ADR 0016).
    pub fn functional_nature(&self, graha: Body, from: ReferencePoint) -> Option<FunctionalNature> {
        use FunctionalNature::*;
        let (_, columns) = TABLE_30[self.reference(from)? as usize];
        let column = columns.iter().position(|c| c.contains(&graha));
        match column {
            Some(i) => Some([Benefic, Neutral, Malefic][i]),
            None if graha == Body::Moon => match self.natural_nature(Body::Moon)? {
                Disposition::Saumya => Some(Malefic),
                _ => Some(Neutral),
            },
            None => None,
        }
    }

    /// Table 30's yogakaraka from `from`'s rasi in `d`'s chart, if it has one (ch. 13 §13.2, p. 167, ADR 0016).
    pub fn yogakaraka(&self, from: ReferencePoint) -> Option<Body> {
        TABLE_30[self.reference(from)? as usize].0
    }
}

/// What a yogada gives: money, by HL, or power and authority, by GL (ch. 13 §13.2, pp. 167–168).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gives {
    Money,
    Power,
}

/// How a yogada reaches a rasi: owns it as its seven-planet lord, is in it, or has rasi drishti on it (ADR 0016).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Link {
    Owns,
    Conjoins,
    Aspects,
}

/// A planet giving yoga in money or power (ch. 13 §13.2, ADR 0016).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yogada {
    pub graha: Body,
    pub gives: Gives,
    /// Every link it has to the lagna, then to HL or GL, each in [`Link`]'s order.
    pub links: [Vec<Link>; 2],
}

impl VargaChakra<'_> {
    /// The yogadas of `d`'s chart from its own lagna, HL and GL, whatever the reference point: each of the seven planets that owns,
    /// conjoins or has rasi drishti on both the lagna and HL (money), or the lagna and GL (power), money first, then in graha
    /// order. "Aspects" is rasi drishti, as every worked example reads it (pp. 230, 495, 496); none without a Vedic day (ADR 0016).
    pub fn yogadas(&self) -> Vec<Yogada> {
        let lagna = self.rasi_of(Body::Lagna);
        let mut found = Vec::new();
        for (gives, point) in [
            (Gives::Money, SpecialLagna::HoraLagna),
            (Gives::Power, SpecialLagna::GhatiLagna),
        ] {
            let Some(point) = self.reference(ReferencePoint::Special(point)) else {
                continue;
            };
            for graha in PLANETS {
                let at = self.rasi_of(graha);
                let links = [lagna, point].map(|r| {
                    [
                        (Link::Owns, r.lord() == graha),
                        (Link::Conjoins, at == r),
                        (Link::Aspects, rasi_drishti(at, r)),
                    ]
                    .into_iter()
                    .filter_map(|(link, holds)| holds.then_some(link))
                    .collect::<Vec<_>>()
                });
                if links.iter().all(|l| !l.is_empty()) {
                    found.push(Yogada {
                        graha,
                        gives,
                        links,
                    });
                }
            }
        }
        found
    }
}

/// `rasi`'s baadhaka sthaana: the 11th from a chara rasi, the 9th from a sthira, the 7th from a dwiswabhava (ch. 13 §13.3,
/// Table 31, pp. 168–169). Its occupants are [`VargaChakra::grahas_in`] it.
pub fn baadhaka_sthaana(rasi: Rasi) -> Rasi {
    let k = match rasi.mobility() {
        Mobility::Chara => 11,
        Mobility::Sthira => 9,
        Mobility::Dwiswabhava => 7,
    };
    kth(rasi, k)
}

/// `rasi`'s baadhakas, its baadhaka sthaana's lords: both co-lords for Aquarius and Scorpio, as Table 31 prints them (ch. 13
/// §13.3, pp. 168–169, ADR 0016).
pub fn baadhakas(rasi: Rasi) -> Vec<Body> {
    lords(baadhaka_sthaana(rasi)).collect()
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

/// Every entry of ch. 11 in the book's order, its numbered ones by class and number, and ch. 13's Tapaswi. The Aakriti yoga `Vajra` and the Sankhya
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
    // ch. 13 §13.5
    Tapaswi,
    // §11.7.1
    BasicRaaja,
    DharmaKarmadhipati,
    VipareetaRaaja,
    /// §11.7.3 (1) to (17); (18) only says when others work better, so is not computed (ADR 0014).
    Raaja(u8),
    /// §11.8 (1) to (15).
    RaajaSambandha(u8),
    /// §11.9's basic rule: the Moon, Mercury, Jupiter or Venus exalted in the 2nd.
    BasicDhana,
    /// §11.9 (1) to (12), one per lagna from Aries.
    Dhana(u8),
    /// §11.10 (1) to (9) and (11) to (13); (10)'s "benefic houses" are undefined, so it is not computed (ADR 0014).
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
            | Vimala | Tapaswi => C::OtherPopular,
            BasicRaaja | DharmaKarmadhipati | VipareetaRaaja | Raaja(_) => C::Raaja,
            RaajaSambandha(_) => C::RaajaSambandha,
            BasicDhana | Dhana(_) => C::Dhana,
            Daridra(_) => C::Daridra,
        }
    }
}

/// A condition of a yoga left to the reader, having no measure in the book or bearing only on its results (ADR 0014).
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
    /// All three of a Tapaswi yoga are in one rasi, where the book asks for the third to aspect the two (ch. 13 §13.5, ADR 0016).
    AllInOneRasi,
}

/// A yoga present in a chart, or to judge if `to_judge` holds anything; `name.class()` gives its class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yoga {
    pub name: YogaName,
    /// The grahas forming it: both of a conjunction, only those placed from the Sun or Moon, all seven of a shape or count,
    /// only those of the stated nature in the kendras for Maalaa, Sarpa, Vajra and Yava, none for Kemadruma, the four in the
    /// book's order, repeats included, for Kalpadruma, and otherwise every graha a rule names, in graha order.
    pub grahas: Vec<YogaGraha>,
    pub to_judge: Vec<ToJudge>,
    /// Which of the book's forms held, 1 or 2, for a yoga with an "alternately" form or Brahma's second; a present form before a
    /// to-judge one, then the book's order. Raaja (1)'s and a Dhana row's are 1 for the first half or combination, 2 for the
    /// second, 3 for both; Raaja Sambandha (11)'s likewise for A9 in the lagna and AK in the 9th.
    pub form: Option<u8>,
    /// §11.7.2's measures, for a basic Raaja, Dharma-Karmadhipati or conjoined Vipareeta pair in the rasi chart.
    pub magnitude: Option<Magnitude>,
    /// §11.7.2 (1)'s functional malefics afflicting such a pair, in any chart; empty for every other row (ADR 0016).
    pub afflictions: Vec<Affliction>,
    /// What the book says strengthens the yoga, never a condition; only the Tapaswi yoga has any (ADR 0016).
    pub notes: Vec<Note>,
}

/// A remark the book adds to a yoga present, not a condition of it (ch. 13 §13.5, ADR 0016).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    /// This AK, from the rasi chart, conjoins or aspects by graha drishti a graha of the yoga other than himself: "more fruitful".
    AtmaKaraka(Body),
    /// This graha of the yoga is in the 8th from the lagna: "particularly powerful".
    InTheEighth(Body),
}

/// A functional malefic from the lagna conjoining or aspecting, by graha drishti, a graha of a Raaja pair (ch. 11 §11.7.2 (1),
/// ch. 13 Table 30, ADR 0016). Not rated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Affliction {
    pub graha: Body,
    pub afflicts: Body,
    /// As [`Magnitude::closeness`] measures it, in the rasi chart only.
    pub closeness: Option<u32>,
}

/// A graha of a yoga row with the roles the row names it by (ch. 11, ADR 0014).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YogaGraha {
    pub graha: Body,
    /// Its bhava from `d`'s lagna, never the reference point.
    pub bhava: u8,
    /// The bhavas the row names it lord of, from that lagna, in order: every one [`VargaChakra::lords_in_yoga`] counts, or for
    /// §11.6 [`VargaChakra::lords_to_try`], but for Raaja (1) only the 1st and 5th, and none under form 1 (§11.7.3).
    pub named_lord_of: Vec<u8>,
    /// AK or PK, for Raaja (1) under form 1 or 3 only (§11.7.3); from the rasi chart in every varga.
    pub karaka: Option<CharaKaraka>,
}

/// How exact a Raaja yoga's pair is and how good their amsas, with no rating (ch. 11 §11.7.2, ADR 0014).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Magnitude {
    /// The difference of the two grahas' degrees within their rasis in whole arcseconds, the nodes' as they stand, for any
    /// association.
    pub closeness: u32,
    /// Each graha of the pair, in the row's order, with its Dasa Varga count and amsa (ch. 6 §6.6).
    pub amsas: [(Body, u8, Option<Amsa>); 2],
}

impl VargaChakra<'_> {
    /// Whether `g` aspects rasi `r` with graha drishti from his own rasi in `d`'s chart (ch. 10 §10.2).
    fn aspects(&self, g: Body, r: Rasi) -> bool {
        graha_drishti(g, self.rasi_of(g)).contains(&r)
    }

    /// Whether `g` occupies rasi `r` or aspects it with graha drishti in `d`'s chart.
    fn joins_or_aspects(&self, g: Body, r: Rasi) -> bool {
        self.rasi_of(g) == r || self.aspects(g, r)
    }

    /// Whether two distinct grahas are associated in `d`'s chart: conjoined, in mutual graha drishti, or each in a rasi the
    /// other owns (parivartana) (ch. 11 §11.7.1). The nodes have no graha drishti (ADR 0013).
    pub fn associated(&self, a: Body, b: Body) -> bool {
        let (ra, rb) = (self.rasi_of(a), self.rasi_of(b));
        a != b
            && (ra == rb
                || self.aspects(a, rb) && self.aspects(b, ra)
                || owns(b, ra) && owns(a, rb))
    }
}

/// The Sankhya yoga by how many rasis the seven planets occupy (§11.5.4); [`VargaChakra::yogas`] adds the proviso, so never lists `Gola`.
pub fn sankhya_yoga(planets: [Rasi; 7]) -> YogaName {
    use YogaName::*;
    let occupied = RASIS.iter().filter(|r| planets.contains(r)).count();
    [Gola, Yuga, Soola, Kedaara, Paasa, Daama, Veenaa][occupied - 1]
}

impl VargaChakra<'_> {
    /// The yogas of `d`'s chart, counted from that chart's lagna, in the book's order; absent ones are not listed (ADR 0014).
    /// Raaja (6) and (8) need the chart's HL and GL.
    pub fn yogas(&self) -> Vec<Yoga> {
        use Body::*;
        use YogaName as N;

        let yoga = |name, grahas: Vec<Body>| Yoga {
            name,
            grahas: self.yoga_grahas(name, &grahas),
            to_judge: Vec::new(),
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        // Mars to Saturn: the Sun's and Moon's 2nd and 12th leave out the other light and the nodes (ADR 0014).
        let others_in = |rasi| -> Vec<Body> {
            PLANETS[2..]
                .iter()
                .copied()
                .filter(|&g| self.rasi_of(g) == rasi)
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
        let sun = self.rasi_of(Sun);
        found.extend(flanking(sun, [N::Vesi, N::Vosi, N::Ubhayachara]));
        if self.rasi_of(Mercury) == sun {
            found.push(yoga(N::BudhaAaditya, vec![Sun, Mercury]));
        }

        // §11.3.1–11.3.6.
        let moon = self.rasi_of(Moon);
        let lagna = self.rasi_of(Lagna);
        let kendra = |rasi| matches!(bhava(lagna, rasi), 1 | 4 | 7 | 10);
        found.extend(flanking(moon, [N::Sunaphaa, N::Anaphaa, N::Duradhara]));
        let beside_moon = [1, 2, 12]
            .iter()
            .any(|&k| !others_in(kth(moon, k)).is_empty());
        let in_kendra = PLANETS
            .iter()
            .any(|&g| g != Moon && kendra(self.rasi_of(g)));
        if !beside_moon && !in_kendra {
            found.push(yoga(N::Kemadruma, Vec::new()));
        }
        if self.rasi_of(Mars) == moon {
            found.push(yoga(N::ChandraMangala, vec![Moon, Mars]));
        }
        // Two or more natural benefics in the places, in any arrangement; a kroora Mercury is not one, and the Moon is counted from (ADR 0014).
        let places = [6, 7, 8].map(|k| kth(moon, k));
        found.extend(self.with_mercury_each_way(N::Adhi, |mercury| {
            let held: Vec<Body> = [Mercury, Jupiter, Venus]
                .into_iter()
                .filter(|&g| g != Mercury || mercury == Disposition::Saumya)
                .filter(|&g| places.contains(&self.rasi_of(g)))
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
            let rasi = self.rasi_of(graha);
            if kendra(rasi) && (lords(rasi).any(|g| g == graha) || exalted(graha, rasi)) {
                found.push(yoga(name, vec![graha]));
            }
        }

        // §11.5: over the seven planets, as "all the planets" means (ADR 0014).
        let planets = PLANETS.map(|g| self.rasi_of(g));
        let houses = planets.map(|r| bhava(lagna, r));
        let within = |places: &[u8]| houses.iter().all(|h| places.contains(h));
        let forms = |places: &[u8]| within(places) && places.iter().all(|p| houses.contains(p));
        let seven_from = |k: u8| -> [u8; 7] { std::array::from_fn(|i| (k - 1 + i as u8) % 12 + 1) };
        let first_naabhasa = found.len();
        // Among the grahas only a tied Mercury has no nature.
        let of_nature =
            |g: Body, nature, mercury_as| self.natural_nature(g).unwrap_or(mercury_as) == nature;

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
            let row = self.with_mercury_each_way(name, |mercury_as| {
                let grahas: Vec<Body> = GRAHAS
                    .into_iter()
                    .filter(|&g| kendra(self.rasi_of(g)) && of_nature(g, forming, mercury_as))
                    .collect();
                let held = [1, 4, 7, 10]
                    .iter()
                    .filter(|&&k| grahas.iter().any(|&g| bhava(lagna, self.rasi_of(g)) == k))
                    .count();
                (held >= 3).then_some(grahas)
            });
            found.extend(row.map(|mut row| {
                if GRAHAS
                    .iter()
                    .any(|&g| kendra(self.rasi_of(g)) && self.natural_nature(g) == Some(opposite))
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
            found.extend(self.with_mercury_each_way(name, |mercury_as| {
                let grahas: Vec<Body> = GRAHAS
                    .into_iter()
                    .filter(|&g| match bhava(lagna, self.rasi_of(g)) {
                        1 | 7 => of_nature(g, in_1st_7th, mercury_as),
                        4 | 10 => of_nature(g, in_4th_10th, mercury_as),
                        _ => false,
                    })
                    .collect();
                [1, 4, 7, 10]
                    .iter()
                    .all(|&k| grahas.iter().any(|&g| bhava(lagna, self.rasi_of(g)) == k))
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

        found.extend(self.other_popular_yogas());
        found.extend(self.raaja_yogas());
        found.extend(self.dhana_yogas());
        found.extend(self.daridra_yogas());

        found
    }

    /// §11.6, Subha to Vimala, then ch. 13 §13.5's Tapaswi. "Planets" are the seven, "benefics" and "malefics" by [`VargaChakra::natural_nature`] (ADR 0014).
    fn other_popular_yogas(&self) -> Vec<Yoga> {
        let (d, chart) = (self.d, self.chart);
        use Body::*;
        use Disposition::{Kroora, Saumya};
        use YogaName as N;

        let lagna = self.rasi_of(Lagna);
        let kendra = |rasi| matches!(bhava(lagna, rasi), 1 | 4 | 7 | 10);
        let trikona = |rasi| matches!(bhava(lagna, rasi), 1 | 5 | 9);
        let yoga = |name, grahas: Vec<Body>| Yoga {
            name,
            grahas: self.yoga_grahas(name, &grahas),
            to_judge: Vec::new(),
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        // Among the grahas only a tied Mercury has no nature.
        let of_nature =
            |g, nature, mercury_as| self.natural_nature(g).unwrap_or(mercury_as) == nature;
        let of_nature_in = |rasi, nature, mercury_as| -> Vec<Body> {
            GRAHAS
                .into_iter()
                .filter(|&g| self.rasi_of(g) == rasi && of_nature(g, nature, mercury_as))
                .collect()
        };
        // "Only" benefics: a tied Mercury always has a malefic beside him, so failing him here is right either way (§3.2.2).
        let only_benefics = |rasi: Rasi| {
            self.grahas_in(rasi)
                .iter()
                .all(|&g| self.natural_nature(g) == Some(Saumya))
        };
        let conjoins_or_graha_drishti_on = |g, other| self.joins_or_aspects(g, self.rasi_of(other));
        let with_rasi_lord = |g| self.with_rasi_lord(g);
        // A graha is never in a kendra from himself: these are relations between two (ADR 0014).
        let kendra_from =
            |a, b| a != b && matches!(bhava(self.rasi_of(a), self.rasi_of(b)), 1 | 4 | 7 | 10);
        let own_or_uchcha = |g| own_or_exalted(g, self.rasi_of(g));
        let row = |name, form, grahas: &[Body], to_judge| Yoga {
            name,
            grahas: self.yoga_grahas(name, &in_graha_order(grahas)),
            to_judge,
            form,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        let strong = |g| vec![ToJudge::Strong(g)];
        // A graha in his own rasi is his own dispositor, whatever the co-lord rule says: "Saturn himself" of Aq in chart 9's navamsa.
        let dispositors = |v, g| {
            let rasi = chart.in_varga(v).rasi_of(g);
            if owns(g, rasi) {
                vec![g]
            } else {
                chart.in_varga(v).lords_to_try(rasi)
            }
        };
        let in_house = |g, k| self.rasi_of(g) == kth(lagna, k);
        let friendly = |g| {
            own_or_uchcha(g)
                || matches!(
                    with_rasi_lord(g),
                    Some(Compound::Mitra | Compound::Adhimitra)
                )
        };
        // A graha never joins himself: these are relations between two (ADR 0014).
        let joins = |a, b| a != b && self.rasi_of(a) == self.rasi_of(b);
        let mut found = Vec::new();

        for (name, nature) in [(N::Subha, Saumya), (N::Asubha, Kroora)] {
            found.extend(self.with_mercury_each_way(name, |mercury_as| {
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
        let jupiter = self.rasi_of(Jupiter);
        let in_enemys_house = matches!(
            with_rasi_lord(Jupiter),
            Some(Compound::Satru | Compound::Adhisatru)
        );
        if matches!(bhava(self.rasi_of(Moon), jupiter), 1 | 4 | 7 | 10)
            && !in_enemys_house
            && !debilitated(Jupiter, jupiter)
        {
            let row = self.with_mercury_each_way(N::GajaKesari, |mercury_as| {
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
                if d == Varga::RasiChart && self.rasi_of(Sun) == jupiter {
                    row.to_judge.push(ToJudge::Combust(Jupiter));
                }
                row
            }));
        }

        if matches!(bhava(jupiter, self.rasi_of(Mars)), 1 | 7) {
            found.push(yoga(N::GuruMangala, vec![Mars, Jupiter]));
        }

        let amala: Vec<Body> = [lagna, self.rasi_of(Moon)]
            .map(|from| kth(from, 10))
            .into_iter()
            .filter(|&rasi| only_benefics(rasi))
            .flat_map(|rasi| self.grahas_in(rasi))
            .collect();
        if !amala.is_empty() {
            found.push(yoga(N::Amala, in_graha_order(&amala)));
        }

        // (2) leaves the 7th free to be vacant, so a kendra may be, but one must be occupied (ADR 0014).
        let parvata = [1, 4, 7, 10, 8].map(|k| kth(lagna, k));
        let in_kendras = [1, 4, 7, 10]
            .iter()
            .any(|&k| !self.grahas_in(kth(lagna, k)).is_empty());
        if in_kendras && parvata.iter().all(|&rasi| only_benefics(rasi)) {
            let grahas: Vec<Body> = parvata
                .into_iter()
                .flat_map(|rasi| self.grahas_in(rasi))
                .collect();
            found.push(yoga(N::Parvata, in_graha_order(&grahas)));
        }

        // "Some say 9th lord" is not computed (ADR 0014).
        let mut rows = Vec::new();
        for [fourth, first] in self.house_lords([4, 1]) {
            if kendra_from(fourth, Jupiter) {
                let grahas = [fourth, Jupiter, first];
                rows.push(row(N::Kaahala, Some(1), &grahas, strong(first)));
            }
        }
        for [fourth, tenth] in self.house_lords([4, 10]) {
            if own_or_uchcha(fourth) && self.rasi_of(tenth) == self.rasi_of(fourth) {
                rows.push(row(N::Kaahala, Some(2), &[fourth, tenth], Vec::new()));
            }
        }
        found.extend(the_best_of(rows));

        found.extend(self.with_mercury_each_way(N::Chaamara, |mercury_as| {
            let mut grahas = Vec::new();
            for first in self.lords_to_try(lagna) {
                let place = self.rasi_of(first);
                if exalted(first, place) && kendra(place) && self.aspects(Jupiter, place) {
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
        for [first, fifth, sixth] in self.house_lords([1, 5, 6]) {
            if kendra_from(fifth, sixth) {
                let grahas = [first, fifth, sixth];
                rows.push(row(N::Sankha, Some(1), &grahas, strong(first)));
            }
        }
        for [first, tenth, ninth] in self.house_lords([1, 10, 9]) {
            if first != tenth
                && self.rasi_of(first) == self.rasi_of(tenth)
                && self.rasi_of(first).mobility() == Mobility::Chara
            {
                let grahas = [first, tenth, ninth];
                rows.push(row(N::Sankha, Some(2), &grahas, strong(ninth)));
            }
        }
        found.extend(the_best_of(rows));

        let planets_in = |rasi| -> Vec<Body> {
            PLANETS
                .iter()
                .copied()
                .filter(|&g| self.rasi_of(g) == rasi)
                .collect()
        };
        let bheris_houses = [1, 2, 7, 12].map(|k| planets_in(kth(lagna, k)));
        let mut rows = Vec::new();
        for [ninth] in self.house_lords([9]) {
            if bheris_houses.iter().all(|planets| !planets.is_empty()) {
                let grahas = [&bheris_houses.concat()[..], &[ninth]].concat();
                rows.push(row(N::Bheri, Some(1), &grahas, strong(ninth)));
            }
        }
        for [ninth, first] in self.house_lords([9, 1]) {
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
            .filter(|&g| own_or_uchcha(g) && (kendra(self.rasi_of(g)) || trikona(self.rasi_of(g))))
            .collect();
        if !dignified.is_empty() {
            // The lagna lord alone dignified is one graha, and no yoga (ADR 0014).
            let rows = self
                .lords_to_try(lagna)
                .into_iter()
                .filter(|first| dignified != [*first])
                .map(|first| {
                    let grahas = [&dignified[..], &[first]].concat();
                    row(N::Mridanga, None, &grahas, strong(first))
                });
            found.extend(the_best_of(rows.collect()));
        }

        // Applied strictly, as footnote 33 says: so only for a Sagittarius lagna.
        let rows = self
            .house_lords([7, 10, 9])
            .into_iter()
            .filter(|&[seventh, tenth, ninth]| {
                self.rasi_of(seventh) == kth(lagna, 10)
                    && exalted(seventh, self.rasi_of(seventh))
                    && self.rasi_of(tenth) == self.rasi_of(ninth)
            })
            .map(|[seventh, tenth, ninth]| {
                row(N::Sreenaatha, None, &[seventh, tenth, ninth], Vec::new())
            });
        found.extend(the_best_of(rows.collect()));

        // The book glosses its chaturasras as the 4th and 8th.
        found.extend(self.with_mercury_each_way(N::Matsya, |mercury_as| {
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

        found.extend(self.with_mercury_each_way(N::Koorma, |mercury_as| {
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

        let rows = self
            .house_lords([2, 9, 1])
            .into_iter()
            .filter(|&[second, ninth, first]| {
                self.rasi_of(second) == kth(lagna, 9)
                    && self.rasi_of(ninth) == kth(lagna, 2)
                    && (kendra(self.rasi_of(first)) || trikona(self.rasi_of(first)))
            })
            .map(|lords| row(N::Khadga, None, &lords, Vec::new()));
        found.extend(the_best_of(rows.collect()));

        if lagna.mobility() == Mobility::Sthira
            && kendra(self.rasi_of(Venus))
            && trikona(self.rasi_of(Moon))
            && self.rasi_of(Saturn) == kth(lagna, 10)
        {
            found.extend(self.with_mercury_each_way(N::Kusuma, |mercury_as| {
                let mut company = of_nature_in(self.rasi_of(Moon), Saumya, mercury_as);
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
            let placed = |g| {
                kendra(self.rasi_of(g)) || trikona(self.rasi_of(g)) || exalted(g, self.rasi_of(g))
            };
            let mut rows = Vec::new();
            for first in self.lords_to_try(lagna) {
                for second in dispositors(d, first) {
                    for third in dispositors(d, second) {
                        for fourth in dispositors(Varga::Navamsa, second) {
                            let four = [first, second, third, fourth];
                            // One graha all four times is no yoga (ADR 0014).
                            if four.iter().all(|&g| placed(g)) && four.iter().any(|&g| g != first) {
                                rows.push(yoga(N::Kalpadruma, four.to_vec()));
                            }
                        }
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        // A kroora Mercury is a malefic here; "Adhi Yoga from lagna" does not add the 6th (ADR 0014).
        found.extend(self.with_mercury_each_way(N::Lagnaadhi, |mercury_as| {
            let places = [kth(lagna, 7), kth(lagna, 8)];
            let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
                .into_iter()
                .filter(|&g| of_nature(g, Saumya, mercury_as) && places.contains(&self.rasi_of(g)))
                .collect();
            let afflicted = benefics.iter().any(|&b| {
                GRAHAS
                    .into_iter()
                    .any(|m| of_nature(m, Kroora, mercury_as) && conjoins_or_graha_drishti_on(m, b))
            });
            (benefics.len() >= 2 && !afflicted).then_some(benefics)
        }));

        // Sets of places: every benefic but the graha counted from, and so at least one, Jupiter or Venus (ADR 0014).
        let trimurthi = |name, house, places: [usize; 3]| -> Vec<Yoga> {
            self.lords_to_try(kth(lagna, house))
                .into_iter()
                .filter_map(|lord| {
                    self.with_mercury_each_way(name, |mercury_as| {
                        let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
                            .into_iter()
                            .filter(|&g| g != lord && of_nature(g, Saumya, mercury_as))
                            .collect();
                        benefics
                            .iter()
                            .all(|&g| {
                                places
                                    .iter()
                                    .any(|&k| kth(self.rasi_of(lord), k) == self.rasi_of(g))
                            })
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
        for [ninth, eleventh, first, tenth] in self.house_lords([9, 11, 1, 10]) {
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

        // Rasi-only: the 9th lord's navamsa picks a graha placed in the rasi chart (ADR 0014).
        if d == Varga::RasiChart {
            let mut rows = Vec::new();
            for [ninth, tenth] in self.house_lords([9, 10]) {
                for lord in dispositors(Varga::Navamsa, ninth) {
                    let grahas = [ninth, tenth, lord];
                    // The dispositor must bring in another graha (ADR 0014).
                    if grahas.iter().all(|&g| in_house(g, 2)) && lord != ninth {
                        rows.push(row(N::Vishnu, None, &grahas, Vec::new()));
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        let rows = self
            .house_lords([5, 9, 10])
            .into_iter()
            .filter(|&[fifth, ninth, tenth]| {
                in_house(fifth, 9) && in_house(ninth, 10) && in_house(tenth, 5)
            })
            .map(|lords| row(N::Siva, None, &lords, Vec::new()));
        found.extend(the_best_of(rows.collect()));

        // The same rasi counts as a trine, as it does as a kendra, and a trine's trines are its own (ADR 0014).
        let trine_from = |a, b| matches!(bhava(self.rasi_of(a), self.rasi_of(b)), 1 | 5 | 9);
        if trine_from(Sun, Moon) && trine_from(Sun, Mars) {
            found.push(yoga(N::Trilochana, vec![Sun, Moon, Mars]));
        }

        if d == Varga::RasiChart {
            let mut rows = Vec::new();
            for [tenth, first] in self.house_lords([10, 1]) {
                for lord in dispositors(Varga::Navamsa, tenth) {
                    if in_house(lord, 10) && exalted(lord, self.rasi_of(lord)) && joins(first, lord)
                    {
                        rows.push(row(N::Gouri, None, &[tenth, lord, first], Vec::new()));
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        if d == Varga::RasiChart && lagna.mobility() == Mobility::Sthira {
            let mut rows = Vec::new();
            for [sixth, ninth] in self.house_lords([6, 9]) {
                for a in dispositors(Varga::Navamsa, sixth) {
                    for b in dispositors(Varga::Navamsa, ninth) {
                        // The Sun as one of them leaves the other, as Bheri's lagna lord does, but joins no one as both (ADR 0014).
                        if self.aspects(sixth, lagna)
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
        let rows = self
            .house_lords([9, 1])
            .into_iter()
            .filter(|&[ninth, _]| own_or_uchcha(ninth) && kendra(self.rasi_of(ninth)))
            .map(|[ninth, first]| row(N::Lakshmi, None, &[ninth, first], strong(first)));
        found.extend(the_best_of(rows.collect()));

        let in_moons_trines: Vec<Body> = [Mercury, Jupiter]
            .into_iter()
            .filter(|&g| trine_from(Moon, g))
            .collect();
        if kendra(self.rasi_of(Mercury))
            && self.rasi_of(Sun) == Rasi::Leo
            && !in_moons_trines.is_empty()
            && in_house(Mars, 11)
        {
            let rows = self
                .house_lords([10])
                .into_iter()
                .filter(|&[tenth]| in_house(tenth, 5))
                .map(|[tenth]| {
                    let grahas =
                        [&[tenth, Mercury, Sun, Moon, Mars], &in_moons_trines[..]].concat();
                    row(N::Saarada, None, &grahas, strong(Sun))
                });
            found.extend(the_best_of(rows.collect()));
        }

        if d == Varga::RasiChart {
            let mut rows = Vec::new();
            for k in [2, 5, 11] {
                for [lord, ninth] in self.house_lords([k, 9]) {
                    for disposing in dispositors(Varga::Navamsa, lord) {
                        if exalted(disposing, self.rasi_of(disposing)) && joins(disposing, ninth) {
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
            .all(|&g| matches!(bhava(lagna, self.rasi_of(g)), 1 | 2 | 4 | 5 | 7 | 9 | 10))
            && friendly(Jupiter)
        {
            found.push(yoga(N::Saraswathi, saraswathi.to_vec()));
        }

        let amsaavatara = [Jupiter, Venus, Saturn];
        if amsaavatara.iter().all(|&g| kendra(self.rasi_of(g)))
            && exalted(Saturn, self.rasi_of(Saturn))
        {
            found.push(yoga(N::Amsaavatara, amsaavatara.to_vec()));
        }

        // Footnote 35: each lord in the other's house.
        let exchange = |a, i, b, j| in_house(a, i) && in_house(b, j);
        if lagna.mobility() == Mobility::Sthira {
            let rows = self
                .house_lords([2, 10, 1, 11])
                .into_iter()
                .filter(|&[second, tenth, first, eleventh]| {
                    exchange(second, 10, tenth, 2) && exchange(first, 11, eleventh, 1)
                })
                .map(|lords| row(N::Devendra, None, &lords, Vec::new()));
            found.extend(the_best_of(rows.collect()));
        }

        let rows = self
            .house_lords([5, 11])
            .into_iter()
            .filter(|&[fifth, eleventh]| exchange(fifth, 11, eleventh, 5) && in_house(Moon, 5))
            .map(|[fifth, eleventh]| row(N::Indra, None, &[fifth, eleventh, Moon], Vec::new()));
        found.extend(the_best_of(rows.collect()));

        let rows = self
            .house_lords([10])
            .into_iter()
            .filter(|&[tenth]| in_house(Sun, 10) && in_house(tenth, 3) && joins(tenth, Saturn))
            .map(|[tenth]| row(N::Ravi, None, &[Sun, tenth, Saturn], Vec::new()));
        found.extend(the_best_of(rows.collect()));

        if bhava(self.rasi_of(Sun), self.rasi_of(Moon)) == 12
            && bhava(self.rasi_of(Sun), self.rasi_of(Mercury)) == 2
            && matches!(bhava(self.rasi_of(Moon), jupiter), 5 | 9)
        {
            found.push(yoga(N::Bhaaskara, vec![Sun, Moon, Mercury, Jupiter]));
        }

        let fifths = [lagna, self.rasi_of(Moon), self.rasi_of(Sun)].map(|from| kth(from, 5));
        if PLANETS.iter().all(|&g| fifths.contains(&self.rasi_of(g))) {
            found.push(yoga(N::Kulavardhana, PLANETS.to_vec()));
        }

        // A tied Mercury has a benefic and a malefic beside him, so his reading decides neither the yoga nor the note (ADR 0014).
        let upachaya = |g| matches!(bhava(lagna, self.rasi_of(g)), 3 | 6 | 10 | 11);
        let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
            .into_iter()
            .filter(|&g| of_nature(g, Saumya, Saumya))
            .collect();
        if benefics.iter().all(|&g| upachaya(g)) {
            let malefics = GRAHAS
                .into_iter()
                .filter(|&g| upachaya(g) && self.natural_nature(g) == Some(Kroora));
            let to_judge = benefics
                .iter()
                .map(|&g| ToJudge::Strong(g))
                .chain(malefics.map(ToJudge::MaleficInUpachaya))
                .collect();
            found.push(row(N::Vasumati, None, &benefics, to_judge));
        }

        if exalted(Sun, self.rasi_of(Sun)) && in_house(Moon, 9) {
            let rows = self
                .house_lords([10, 1])
                .into_iter()
                .filter(|&[tenth, first]| {
                    matches!(bhava(kth(lagna, 7), self.rasi_of(tenth)), 1 | 5 | 9)
                        && first != Jupiter
                        && conjoins_or_graha_drishti_on(Jupiter, first)
                })
                .map(|[tenth, first]| {
                    let grahas = [tenth, first, Jupiter, Sun, Moon];
                    row(N::Gandharva, None, &grahas, strong(Sun))
                });
            found.extend(the_best_of(rows.collect()));
        }

        // The whole sign: §3.3's degrees are of results, as Exercise 41 and §6.6's example show (ADR 0014).
        if jupiter == Rasi::Sagittarius {
            let rows = self
                .house_lords([2, 1])
                .into_iter()
                .filter(|&[second, first]| {
                    joins(second, Jupiter) && exalted(first, self.rasi_of(first))
                })
                .map(|[second, first]| {
                    row(N::Go, None, &[second, Jupiter, first], strong(Jupiter))
                });
            found.extend(the_best_of(rows.collect()));
        }

        if d == Varga::RasiChart {
            let rows = self
                .house_lords([11, 1])
                .into_iter()
                .filter(|&[eleventh, first]| {
                    exalted(eleventh, self.rasi_of(eleventh))
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

        let rows = self
            .house_lords([4, 10, 1])
            .into_iter()
            .filter(|&[fourth, tenth, first]| {
                exchange(fourth, 10, tenth, 4) && exalted(first, self.rasi_of(first))
            })
            .map(|lords| row(N::Chapa, None, &lords, Vec::new()));
        found.extend(the_best_of(rows.collect()));

        // The book numbers its four conditions (1), (2), (2), (4).
        let lagna_planets = planets_in(lagna);
        if !lagna_planets.is_empty() {
            let mut rows = Vec::new();
            for first in self.lords_to_try(lagna) {
                for lord in dispositors(d, Moon) {
                    if joins(first, Moon)
                        && (kendra(self.rasi_of(lord))
                            || with_rasi_lord(lord) == Some(Compound::Adhimitra))
                        && self.aspects(lord, lagna)
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
            let rows = self
                .house_lords([9])
                .into_iter()
                .filter(|&[ninth]| bhava(self.rasi_of(ninth), jupiter) == 9)
                .map(|[ninth]| {
                    let grahas = [&[ninth, Jupiter, Saturn], &beyond_jupiter[..]].concat();
                    row(N::Makuta, None, &grahas, Vec::new())
                });
            found.extend(the_best_of(rows.collect()));
        }

        if d == Varga::RasiChart {
            let rows = self
                .house_lords([10, 6])
                .into_iter()
                .filter(|&[tenth, sixth]| {
                    exalted(tenth, self.rasi_of(tenth)) && debilitated(sixth, self.rasi_of(sixth))
                })
                .map(|[tenth, sixth]| {
                    let deep = vec![ToJudge::DeepExaltation(tenth)];
                    row(N::Jaya, None, &[tenth, sixth], deep)
                });
            found.extend(the_best_of(rows.collect()));
        }

        for (name, k) in [(N::Harsha, 6), (N::Sarala, 8), (N::Vimala, 12)] {
            let rows = self
                .lords_to_try(kth(lagna, k))
                .into_iter()
                .filter(|&lord| in_house(lord, k))
                .map(|lord| row(name, None, &[lord], Vec::new()));
            found.extend(the_best_of(rows.collect()));
        }

        // Ch. 13 §13.5, Example 44 (p. 174): two of the three in one rasi and the third aspecting them by graha drishti, so never
        // Ketu (ADR 0013); all three together is to judge, as the book only says "aspects" (ADR 0016).
        let tapaswi = [Venus, Saturn, Ketu];
        let together = |a, b| self.rasi_of(a) == self.rasi_of(b);
        let to_judge = if together(Venus, Saturn) && together(Saturn, Ketu) {
            Some(vec![ToJudge::AllInOneRasi])
        } else {
            [(Venus, Saturn, Ketu), (Saturn, Venus, Ketu)]
                .into_iter()
                .any(|(third, a, b)| together(a, b) && conjoins_or_graha_drishti_on(third, a))
                .then(Vec::new)
        };
        if let Some(to_judge) = to_judge {
            // A karakatwa shared on a tie is each holder's (ADR 0011).
            let mut notes: Vec<Note> = chart
                .chara_karakas()
                .into_iter()
                .filter(|&(ak, k)| {
                    k == CharaKaraka::Atma
                        && tapaswi
                            .iter()
                            .any(|&g| g != ak && conjoins_or_graha_drishti_on(ak, g))
                })
                .map(|(ak, _)| Note::AtmaKaraka(ak))
                .collect();
            notes.extend(
                tapaswi
                    .into_iter()
                    .filter(|&g| in_house(g, 8))
                    .map(Note::InTheEighth),
            );
            found.push(Yoga {
                notes,
                ..row(N::Tapaswi, None, &tapaswi, to_judge)
            });
        }

        found
    }

    /// [`Chart::lord_relationship`] in `d`'s chart: `g`'s compound relationship with its rasi's seven-planet lord (§3.4.3).
    fn with_rasi_lord(&self, g: Body) -> Option<Compound> {
        self.compound_with(g, self.rasi_of(g).lord())
    }

    /// `g`'s compound relationship with `other` in `d`'s chart; none unless both are the seven planets, and differ (§3.4.3).
    fn compound_with(&self, g: Body, other: Body) -> Option<Compound> {
        Some(compound(
            natural(g, other)?,
            tatkaala(self.rasi_of(g), self.rasi_of(other)),
        ))
    }
}

/// §11.7.2's closeness of `a` and `b`: the difference of their degrees within their rasis in the rasi chart (ADR 0009, ADR 0014).
fn closeness(chart: &Chart, a: Body, b: Body) -> u32 {
    let degrees = |g| arcseconds(rasi(chart.entry(g).longitude).1);
    degrees(a).abs_diff(degrees(b))
}

/// §11.7.2's magnitude of `pair` from the rasi chart's longitudes (ADR 0009, ADR 0014).
fn magnitude(chart: &Chart, pair: [Body; 2]) -> Magnitude {
    let amsa = |g| {
        let (count, amsa) = amsabala(g, chart.entry(g).longitude, VargaGroup::DasaVarga);
        (g, count, amsa)
    };
    Magnitude {
        closeness: closeness(chart, pair[0], pair[1]),
        amsas: pair.map(amsa),
    }
}

impl VargaChakra<'_> {
    /// §11.7.2 (1)'s afflictions of a Raaja `pair` in `d`'s chart: each functional malefic from its lagna, not of the pair, in the
    /// rasi of one of them or aspecting it by graha drishti, per graha afflicted, in graha order (ADR 0016).
    fn afflictions(&self, pair: [Body; 2]) -> Vec<Affliction> {
        let lagna = ReferencePoint::Body(Body::Lagna);
        let rasi_chart = self.d == Varga::RasiChart;
        PLANETS
            .into_iter()
            .filter(|&g| {
                !pair.contains(&g)
                    && self.functional_nature(g, lagna) == Some(FunctionalNature::Malefic)
            })
            .flat_map(|g| pair.map(|afflicts| (g, afflicts)))
            .filter(|&(g, afflicts)| self.joins_or_aspects(g, self.rasi_of(afflicts)))
            .map(|(graha, afflicts)| Affliction {
                graha,
                afflicts,
                closeness: rasi_chart.then(|| closeness(self.chart, graha, afflicts)),
            })
            .collect()
    }
}

impl VargaChakra<'_> {
    /// §11.7 and §11.8, Raaja and Raaja Sambandha yogas, from `d`'s lagna. "Planets" are the seven, "benefics" and "malefics" by
    /// [`VargaChakra::natural_nature`], an aspect is graha drishti and a lord is [`VargaChakra::lords_in_yoga`]'s, not §11.6's [`VargaChakra::lords_to_try`] (ADR 0014).
    fn raaja_yogas(&self) -> Vec<Yoga> {
        let (d, chart) = (self.d, self.chart);
        use Body::*;
        use Disposition::{Kroora, Saumya};
        use YogaName as N;

        let lagna = self.rasi_of(Lagna);
        let house = |k| kth(lagna, k);
        let lords_of = |k| self.lords_in_yoga(house(k));
        let rasi_chart = d == Varga::RasiChart;
        let row = |name, grahas: &[Body], to_judge| Yoga {
            name,
            grahas: self.yoga_grahas(name, &in_graha_order(grahas)),
            to_judge,
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        // `pair` comes in graha order, the row's.
        let paired = |name, pair: [Body; 2], to_judge| Yoga {
            magnitude: rasi_chart.then(|| magnitude(chart, pair)),
            afflictions: self.afflictions(pair),
            notes: Vec::new(),
            ..row(name, &pair, to_judge)
        };
        // A graha never joins himself: these are relations between two (ADR 0014).
        let joins = |a, b| a != b && self.rasi_of(a) == self.rasi_of(b);
        let aspects = |a: Body, b: Body| self.aspects(a, self.rasi_of(b));
        let own_or_uchcha = |g| own_or_exalted(g, self.rasi_of(g));
        let first_of = |xs: &[Body], ys: &[Body], holds: &dyn Fn(Body, Body) -> bool| {
            xs.iter()
                .flat_map(|&a| ys.iter().map(move |&b| [a, b]))
                .find(|&[a, b]| holds(a, b))
        };
        // Among the grahas only a tied Mercury has no nature.
        let of_nature =
            |g, nature, mercury_as| self.natural_nature(g).unwrap_or(mercury_as) == nature;
        let mut found = Vec::new();

        // §11.7.1: one row per pair of grahas, the first co-lord that holds standing for its houses (ADR 0014).
        let mut pairs: Vec<([Body; 2], bool)> = Vec::new();
        for k in [1, 4, 7, 10] {
            for t in [1, 5, 9] {
                // Two lords of one house are no pair: the lagna's, or a yogakaraka and his co-lord (ADR 0014).
                let co_lords = |a, b| {
                    [k, t]
                        .iter()
                        .any(|&h| owns(a, house(h)) && owns(b, house(h)))
                };
                let found_pair = first_of(&lords_of(k), &lords_of(t), &|a, b| {
                    self.associated(a, b) && !co_lords(a, b)
                });
                if let Some(pair) = found_pair {
                    let dharma_karma = (k, t) == (10, 9);
                    let pair = in_graha_order(&pair);
                    match pairs.iter_mut().find(|(p, _)| *p == pair[..]) {
                        Some((_, dk)) => *dk |= dharma_karma,
                        None => pairs.push(([pair[0], pair[1]], dharma_karma)),
                    }
                }
            }
        }
        for dharma_karma in [false, true] {
            for &(pair, _) in pairs.iter().filter(|(_, dk)| *dk == dharma_karma) {
                let name = if dharma_karma {
                    N::DharmaKarmadhipati
                } else {
                    N::BasicRaaja
                };
                found.push(paired(name, pair, Vec::new()));
            }
        }

        // A dusthana lord in a dusthana is present, two conjoined elsewhere leave their strength to judge (p. 494, ADR 0014).
        let dusthana = |r| matches!(bhava(lagna, r), 6 | 8 | 12);
        let dusthana_lords = in_graha_order(&[6, 8, 12].map(lords_of).concat());
        for &g in dusthana_lords
            .iter()
            .filter(|&&g| dusthana(self.rasi_of(g)))
        {
            found.push(row(N::VipareetaRaaja, &[g], Vec::new()));
        }
        for (i, &a) in dusthana_lords.iter().enumerate() {
            for &b in &dusthana_lords[i + 1..] {
                if self.rasi_of(a) == self.rasi_of(b) && !dusthana(self.rasi_of(a)) {
                    let strong = vec![ToJudge::Strong(a), ToJudge::Strong(b)];
                    found.push(paired(N::VipareetaRaaja, [a, b], strong));
                }
            }
        }

        // §11.7.3, from the rasi chart's karakas; a karakatwa shared on a tie is each holder's, an empty one fails (ADR 0011).
        let karakas = chart.chara_karakas();
        let karaka = |k| -> Vec<Body> {
            karakas
                .iter()
                .filter(|&&(_, c)| c == k)
                .map(|&(g, _)| g)
                .collect()
        };
        let (ak, pk) = (karaka(CharaKaraka::Atma), karaka(CharaKaraka::Putra));
        let (first, fifth, ninth) = (lords_of(1), lords_of(5), lords_of(9));

        // (1): either half by itself is a Raaja yoga (chart 3, p. 265; ADR 0014).
        let karakas_joined = first_of(&ak, &pk, &joins);
        let lords_joined = first_of(&first, &fifth, &joins);
        let form = form_of(karakas_joined.is_some(), lords_joined.is_some());
        if form.is_some() {
            let grahas = [karakas_joined, lords_joined]
                .into_iter()
                .flatten()
                .flatten();
            let mut row = Yoga {
                form,
                ..row(N::Raaja(1), &grahas.collect::<Vec<_>>(), Vec::new())
            };
            // §11.7.3 (1): form 1 is (a) AK with PK, 2 is (b) the lagna and 5th lords, 3 both.
            for g in &mut row.grahas {
                g.karaka = karakas
                    .iter()
                    .find(|k| k.0 == g.graha)
                    .map(|k| k.1)
                    .filter(|&k| {
                        form != Some(2) && matches!(k, CharaKaraka::Atma | CharaKaraka::Putra)
                    });
                g.named_lord_of
                    .retain(|&n| form != Some(1) && matches!(n, 1 | 5));
            }
            found.push(row);
        }

        let aspected_by = |g, nature, mercury_as| {
            GRAHAS
                .into_iter()
                .any(|b| of_nature(b, nature, mercury_as) && aspects(b, g))
        };
        // (2): (d)'s "those planets" are (c)'s AK and PK, and "amsa" is the navamsa (ADR 0014).
        found.extend(self.with_mercury_each_way(N::Raaja(2), |mercury_as| {
            let fit = |g| {
                matches!(bhava(lagna, self.rasi_of(g)), 1 | 5)
                    && (own_or_uchcha(g)
                        || owns(g, chart.in_varga(Varga::Navamsa).rasi_of(g))
                        || aspected_by(g, Saumya, mercury_as))
            };
            let exchanged = first_of(&first, &fifth, &|l, f| {
                self.rasi_of(l) == house(5) && self.rasi_of(f) == lagna
            })?;
            let karakas = first_of(&ak, &pk, &|a, p| fit(a) && fit(p))?;
            Some(in_graha_order(&[exchanged, karakas].concat()))
        }));

        // (3): each in one of the houses and aspected by a benefic.
        found.extend(self.with_mercury_each_way(N::Raaja(3), |mercury_as| {
            let fit = |g| {
                matches!(bhava(lagna, self.rasi_of(g)), 1 | 5 | 7)
                    && aspected_by(g, Saumya, mercury_as)
            };
            first_of(&ninth, &ak, &|n, a| fit(n) && fit(a)).map(|pair| in_graha_order(&pair))
        }));

        // (4), (5): each named house from each holds one, others beside it or not (ADR 0014).
        let of_nature_in = |r, nature, mercury_as| -> Vec<Body> {
            GRAHAS
                .into_iter()
                .filter(|&g| self.rasi_of(g) == r && of_nature(g, nature, mercury_as))
                .collect()
        };
        found.extend(self.with_mercury_each_way(N::Raaja(4), |mercury_as| {
            let fit = |g: Body| {
                [2, 4, 5]
                    .iter()
                    .all(|&k| !of_nature_in(kth(self.rasi_of(g), k), Saumya, mercury_as).is_empty())
            };
            let [l, a] = first_of(&first, &ak, &|l, a| fit(l) && fit(a))?;
            let benefics = [l, a].into_iter().flat_map(|g| {
                [2, 4, 5].map(|k| of_nature_in(kth(self.rasi_of(g), k), Saumya, mercury_as))
            });
            Some(in_graha_order(
                &[vec![l, a], benefics.flatten().collect()].concat(),
            ))
        }));
        found.extend(self.with_mercury_each_way(N::Raaja(5), |mercury_as| {
            // Saturn's 3rd aspect never counts for himself: these are relations between two (ADR 0014).
            let malefics_on = |from: Body, k| -> Vec<Body> {
                let r = kth(self.rasi_of(from), k);
                GRAHAS
                    .into_iter()
                    .filter(|&g| {
                        g != from && of_nature(g, Kroora, mercury_as) && self.joins_or_aspects(g, r)
                    })
                    .collect()
            };
            let fit = |g: Body| [3, 6].iter().all(|&k| !malefics_on(g, k).is_empty());
            let [l, a] = first_of(&first, &ak, &|l, a| fit(l) && fit(a))?;
            let malefics = [l, a]
                .into_iter()
                .flat_map(|g| [3, 6].map(|k| malefics_on(g, k)));
            Some(in_graha_order(
                &[vec![l, a], malefics.flatten().collect()].concat(),
            ))
        }));

        // (6), (8): hora's own HL and GL, placed in `d` (ADR 0008). (6)'s "owned" and looser form are not computed (ADR 0014).
        let special = |which| {
            chart
                .special_lagna(which)
                .map(|longitude| varga(d, longitude))
        };
        if let (Some(hl), Some(gl)) = (
            special(SpecialLagna::HoraLagna),
            special(SpecialLagna::GhatiLagna),
        ) {
            let three = [lagna, hl, gl];
            let on_all: Vec<Body> = PLANETS
                .into_iter()
                .filter(|&g| three.iter().all(|&r| self.joins_or_aspects(g, r)))
                .collect();
            if !on_all.is_empty() {
                found.push(row(N::Raaja(6), &on_all, Vec::new()));
            }
            let dignified_in = |r| -> Vec<Body> {
                PLANETS
                    .into_iter()
                    .filter(|&g| self.rasi_of(g) == r && own_or_uchcha(g))
                    .collect()
            };
            if three.iter().all(|&r| !dignified_in(r).is_empty()) {
                let grahas: Vec<Body> = three.iter().flat_map(|&r| dignified_in(r)).collect();
                found.push(row(N::Raaja(8), &grahas, Vec::new()));
            }
        }

        // (9): rasi-only, since it reads other vargas. (7) cannot hold in the book's D-2, only Cn and Le, so is not computed (ADR 0014).
        if rasi_chart {
            use Varga::*;
            let dignified_on_lagna = |v| -> Vec<Body> {
                let r = chart.in_varga(v).rasi_of(Lagna);
                PLANETS
                    .into_iter()
                    .filter(|&g| {
                        let at = chart.in_varga(v).rasi_of(g);
                        at == r && own_or_exalted(g, at)
                    })
                    .collect()
            };
            let held = [RasiChart, Navamsa, Drekkana].map(dignified_on_lagna);
            if held.iter().all(|h| !h.is_empty()) {
                found.push(row(N::Raaja(9), &held.concat(), Vec::new()));
            }
        }

        // (10), (11) (b): a lagna lord in own or uchcha rasi aspecting the lagna, so not in it.
        let strong_first = first
            .iter()
            .copied()
            .find(|&g| own_or_uchcha(g) && self.aspects(g, lagna));
        if let Some(l) = strong_first {
            // (10) (a): with (b) at most two can be there, so "one or two" is any.
            let low: Vec<Body> = PLANETS
                .into_iter()
                .filter(|&g| {
                    matches!(bhava(lagna, self.rasi_of(g)), 3 | 6 | 8)
                        && debilitated(g, self.rasi_of(g))
                })
                .collect();
            if !low.is_empty() {
                found.push(row(N::Raaja(10), &[low, vec![l]].concat(), Vec::new()));
            }
            // (11) (a): every dusthana has a lord so placed; combustion follows ADR 0014.
            let weak = |g: Body| -> Option<Vec<ToJudge>> {
                let inimical = matches!(
                    self.with_rasi_lord(g),
                    Some(Compound::Satru | Compound::Adhisatru)
                );
                if debilitated(g, self.rasi_of(g)) || inimical {
                    Some(Vec::new())
                } else if rasi_chart
                    && PLANETS[1..].contains(&g)
                    && self.rasi_of(g) == self.rasi_of(Sun)
                {
                    Some(vec![ToJudge::Combust(g)])
                } else {
                    None
                }
            };
            let mut grahas = vec![l];
            let mut to_judge = Vec::new();
            let all_weak = [6, 8, 12].iter().all(|&k| {
                let mut ways: Vec<(Body, Vec<ToJudge>)> = lords_of(k)
                    .into_iter()
                    .filter_map(|g| Some((g, weak(g)?)))
                    .collect();
                ways.sort_by_key(|(_, j)| j.len());
                ways.first().is_some_and(|(g, j)| {
                    grahas.push(*g);
                    to_judge.extend_from_slice(j);
                    true
                })
            });
            if all_weak {
                found.push(row(N::Raaja(11), &grahas, to_judge));
            }
        }

        // (12): conjoined or in mutual graha drishti.
        let together = |a, b| joins(a, b) || a != b && aspects(a, b) && aspects(b, a);
        if let Some(pair) = first_of(&fifth, &ninth, &together) {
            found.push(row(N::Raaja(12), &pair, Vec::new()));
        }

        // (13): no graha drishti reaches both of two opposite rasis, so each is aspected by either lord (ADR 0014).
        let (fourth, tenth) = (lords_of(4), lords_of(10));
        let exchanged = first_of(&fourth, &tenth, &|f, t| {
            self.rasi_of(f) == house(10) && self.rasi_of(t) == house(4)
        });
        if let Some([f, t]) = exchanged {
            let aspectors = |g| -> Vec<Body> {
                [&fifth[..], &ninth[..]]
                    .concat()
                    .into_iter()
                    .filter(|&a| aspects(a, g))
                    .collect()
            };
            let (on_f, on_t) = (aspectors(f), aspectors(t));
            if !on_f.is_empty() && !on_t.is_empty() {
                found.push(row(
                    N::Raaja(13),
                    &[vec![f, t], on_f, on_t].concat(),
                    Vec::new(),
                ));
            }
        }

        // (14).
        let joiner = [&first[..], &ninth[..]].concat();
        let in_kendra = fifth
            .iter()
            .copied()
            .filter(|&g| matches!(bhava(lagna, self.rasi_of(g)), 1 | 4 | 10))
            .collect::<Vec<_>>();
        if let Some(pair) = first_of(&in_kendra, &joiner, &joins) {
            found.push(row(N::Raaja(14), &pair, Vec::new()));
        }

        // (15): vargottama by the rasi chart and navamsa in any chart read (§11.7.3 note 40); "strong" is to judge (ADR 0014).
        let vargottama = chart.in_varga(Varga::RasiChart).rasi_of(Moon)
            == chart.in_varga(Varga::Navamsa).rasi_of(Moon);
        let on_moon: Vec<Body> = PLANETS
            .into_iter()
            .filter(|&g| g != Moon && aspects(g, Moon))
            .collect();
        if vargottama && on_moon.len() >= 4 {
            found.push(row(
                N::Raaja(15),
                &[vec![Moon], on_moon].concat(),
                vec![ToJudge::Strong(Moon)],
            ));
        }

        // (16): the whole moolatrikona rasi, as Go's (ADR 0014).
        let high: Vec<Body> = PLANETS
            .into_iter()
            .filter(|&g| exalted(g, self.rasi_of(g)) || moolatrikona(g, self.rasi_of(g)))
            .collect();
        if high.len() >= 4 {
            found.push(row(N::Raaja(16), &high, Vec::new()));
        }

        // (17): every benefic in a kendra, a malefic in each of the 3rd, 6th and 11th (ADR 0014).
        found.extend(self.with_mercury_each_way(N::Raaja(17), |mercury_as| {
            let benefics: Vec<Body> = GRAHAS
                .into_iter()
                .filter(|&g| of_nature(g, Saumya, mercury_as))
                .collect();
            let in_kendras = benefics
                .iter()
                .all(|&g| matches!(bhava(lagna, self.rasi_of(g)), 1 | 4 | 7 | 10));
            let malefics = [3, 6, 11].map(|k| of_nature_in(house(k), Kroora, mercury_as));
            (in_kendras && malefics.iter().all(|m| !m.is_empty()))
                .then(|| in_graha_order(&[benefics, malefics.concat()].concat()))
        }));

        // §11.8, from the same karakas.
        let amk = karaka(CharaKaraka::Amatya);
        let eleventh = lords_of(11);
        let on = |a: Body, b: Body| a != b && self.joins_or_aspects(a, self.rasi_of(b));
        let dispositors = |g| self.dispositors_in_yoga(g);
        let sambandha = |n, grahas: &[Body]| row(N::RaajaSambandha(n), grahas, Vec::new());
        let with_benefics = |g: Body, mercury_as| -> Vec<Body> {
            of_nature_in(self.rasi_of(g), Saumya, mercury_as)
                .into_iter()
                .filter(|&b| b != g)
                .collect()
        };

        // (1).
        let amk_side: Vec<Body> = amk
            .iter()
            .flat_map(|&a| [vec![a], dispositors(a)].concat())
            .collect();
        if let Some(pair) = first_of(&tenth, &amk_side, &|t, a| on(a, t)) {
            found.push(sambandha(1, &pair));
        }

        // (2): the 11th lord's own aspect is the one asked for, so he is no malefic on the 11th (ADR 0014).
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(2), |mercury_as| {
                let l = eleventh
                    .iter()
                    .copied()
                    .find(|&l| self.aspects(l, house(11)))?;
                let clear = GRAHAS
                    .into_iter()
                    .filter(|&g| of_nature(g, Kroora, mercury_as))
                    .all(|g| {
                        [10, 11]
                            .iter()
                            .all(|&k| g == l && k == 11 || !self.joins_or_aspects(g, house(k)))
                    });
                clear.then(|| vec![l])
            }),
        );

        // (3)–(6).
        if let Some(pair) = first_of(&ak, &amk, &joins) {
            found.push(sambandha(3, &pair));
        }
        if let Some(&a) = amk.iter().find(|&&a| own_or_uchcha(a)) {
            found.push(row(N::RaajaSambandha(4), &[a], vec![ToJudge::Strong(a)]));
        }
        if let Some(&a) = amk
            .iter()
            .find(|&&a| matches!(bhava(lagna, self.rasi_of(a)), 1 | 5 | 9))
        {
            found.push(sambandha(5, &[a]));
        }
        let kendra_or_trine = |from, r| matches!(bhava(from, r), 1 | 4 | 5 | 7 | 9 | 10);
        if let Some(pair) = first_of(&ak, &amk, &|a, m| {
            kendra_or_trine(self.rasi_of(a), self.rasi_of(m))
        }) {
            found.push(sambandha(6, &pair));
        }

        // (7): each of the 3rd and 6th from each holds a malefic, as (4)'s houses (ADR 0014).
        let arudhas = self.bhava_arudhas();
        let al = arudhas[0];
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(7), |mercury_as| {
                ak.iter().find_map(|&a| {
                    let held: Vec<Vec<Body>> = [lagna, al, self.rasi_of(a)]
                        .iter()
                        .flat_map(|&r| [3, 6].map(|k| of_nature_in(kth(r, k), Kroora, mercury_as)))
                        .collect();
                    held.iter()
                        .all(|h| !h.is_empty())
                        .then(|| in_graha_order(&[vec![a], held.concat()].concat()))
                })
            }),
        );

        // (8).
        let ak_placed = |a: Body| own_or_uchcha(a) && kendra_or_trine(lagna, self.rasi_of(a));
        let placed: Vec<Body> = ak.iter().copied().filter(|&a| ak_placed(a)).collect();
        if let Some(pair) = first_of(&placed, &ninth, &on) {
            found.push(sambandha(8, &pair));
        }

        // (9), (10): AK joined by a benefic other than himself.
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(9), |mercury_as| {
                ak.iter().find_map(|&a| {
                    let benefics = with_benefics(a, mercury_as);
                    (dispositors(Moon).contains(&a)
                        && self.rasi_of(a) == lagna
                        && !benefics.is_empty())
                    .then(|| in_graha_order(&[vec![Moon, a], benefics].concat()))
                })
            }),
        );
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(10), |mercury_as| {
                ak.iter().find_map(|&a| {
                    let benefics = with_benefics(a, mercury_as);
                    (matches!(bhava(lagna, self.rasi_of(a)), 5 | 7 | 9 | 10)
                        && !benefics.is_empty())
                    .then(|| in_graha_order(&[vec![a], benefics].concat()))
                })
            }),
        );

        // (11): A9 is a rasi, so the form says which half held.
        let a9_in_lagna = arudhas[8] == lagna;
        let ak_in_9th: Vec<Body> = ak
            .iter()
            .copied()
            .filter(|&a| self.rasi_of(a) == house(9))
            .take(1)
            .collect();
        let form = form_of(a9_in_lagna, !ak_in_9th.is_empty());
        if form.is_some() {
            found.push(Yoga {
                form,
                ..sambandha(11, &ak_in_9th)
            });
        }

        // (12): no malefic's graha drishti on the 11th lord.
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(12), |mercury_as| {
                let unaspected = |l: Body| {
                    !GRAHAS
                        .into_iter()
                        .any(|g| g != l && of_nature(g, Kroora, mercury_as) && aspects(g, l))
                };
                let l = eleventh
                    .iter()
                    .copied()
                    .find(|&l| self.rasi_of(l) == house(11) && unaspected(l))?;
                ak.iter().find_map(|&a| {
                    let benefics = with_benefics(a, mercury_as);
                    (!benefics.is_empty()).then(|| in_graha_order(&[vec![l, a], benefics].concat()))
                })
            }),
        );

        // (13)–(15).
        if let Some(pair) = first_of(&first, &tenth, &|l, t| {
            self.rasi_of(l) == house(10) && self.rasi_of(t) == lagna
        }) {
            found.push(sambandha(13, &pair));
        }
        let fourth_from = |a: Body| kth(self.rasi_of(a), 4);
        if let Some(&a) = ak.iter().find(|&&a| {
            self.rasi_of(Moon) == fourth_from(a) && self.rasi_of(Venus) == fourth_from(a)
        }) {
            found.push(sambandha(14, &[a, Moon, Venus]));
        }
        let first_or_ak = [&first[..], &ak[..]].concat();
        if let Some(pair) = first_of(&first_or_ak, &fifth, &|x, f| {
            joins(x, f) && kendra_or_trine(lagna, self.rasi_of(f))
        }) {
            found.push(sambandha(15, &pair));
        }

        found
    }
}

/// A graha of a Dhana combination and those it names beside him (§11.9).
type Combination = (Body, &'static [Body]);

impl VargaChakra<'_> {
    /// §11.9, Dhana yogas, from `d`'s lagna. "Conjoined or aspected" is by graha drishti, and "by X and Y" is by each (ADR 0014).
    fn dhana_yogas(&self) -> Vec<Yoga> {
        use Body::*;
        let lagna = self.rasi_of(Lagna);
        let in_house = |g, k| self.rasi_of(g) == kth(lagna, k);
        let row = |name, form, grahas: &[Body]| Yoga {
            name,
            grahas: self.yoga_grahas(name, &in_graha_order(grahas)),
            to_judge: Vec::new(),
            form,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        let mut found = Vec::new();

        let exalted_2nd: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
            .into_iter()
            .filter(|&g| in_house(g, 2) && exalted(g, self.rasi_of(g)))
            .collect();
        if !exalted_2nd.is_empty() {
            found.push(row(YogaName::BasicDhana, None, &exalted_2nd));
        }

        // Per lagna from Aries: the 5th's graha and the 11th's, then the graha in the lagna and those joining or aspecting him.
        let (first, second): (Option<Combination>, Combination) = match lagna {
            Rasi::Aries => (
                Some((Sun, &[Saturn, Moon, Jupiter])),
                (Mars, &[Mercury, Venus, Saturn]),
            ),
            Rasi::Taurus => (
                Some((Mercury, &[Moon, Mars, Jupiter])),
                (Venus, &[Mercury, Saturn]),
            ),
            Rasi::Gemini => (Some((Venus, &[Mars])), (Mercury, &[Jupiter, Saturn])),
            Rasi::Cancer => (Some((Mars, &[Venus])), (Moon, &[Mercury, Jupiter])),
            Rasi::Leo => (Some((Jupiter, &[Mercury])), (Sun, &[Mars, Jupiter])),
            Rasi::Virgo => (Some((Saturn, &[Sun, Moon])), (Mercury, &[Jupiter, Saturn])),
            Rasi::Libra => (Some((Saturn, &[Sun, Moon])), (Venus, &[Mercury, Saturn])),
            Rasi::Scorpio => (
                Some((Jupiter, &[Mercury])),
                (Mars, &[Mercury, Venus, Saturn]),
            ),
            Rasi::Sagittarius => (Some((Mars, &[Venus])), (Jupiter, &[Mars, Mercury])),
            Rasi::Capricorn => (Some((Venus, &[Mars])), (Saturn, &[Mars, Jupiter])),
            Rasi::Aquarius => (
                Some((Mercury, &[Moon, Mars, Jupiter])),
                (Saturn, &[Mars, Jupiter]),
            ),
            // The first, the Moon in both the 5th and 11th, is impossible as printed, so is not computed (ADR 0014).
            Rasi::Pisces => (None, (Jupiter, &[Mars, Mercury])),
        };
        let first = first
            .filter(|&(fifth, eleventh)| {
                in_house(fifth, 5) && eleventh.iter().all(|&g| in_house(g, 11))
            })
            .map(|(fifth, eleventh)| [&[fifth], eleventh].concat());
        let (lord, by) = second;
        let second = (in_house(lord, 1) && by.iter().all(|&g| self.joins_or_aspects(g, lagna)))
            .then(|| [&[lord], by].concat());
        let form = form_of(first.is_some(), second.is_some());
        if form.is_some() {
            let grahas = [first, second]
                .into_iter()
                .flatten()
                .flatten()
                .collect::<Vec<_>>();
            found.push(row(YogaName::Dhana(lagna as u8 + 1), form, &grahas));
        }

        found
    }

    /// §11.10, Daridra yogas, from `d`'s lagna: marakas are §14.2's, read each way for a tied Mercury, and "conjoined or aspected"
    /// relates two grahas, by graha drishti (ADR 0014). (10) is not computed.
    fn daridra_yogas(&self) -> Vec<Yoga> {
        let (d, chart) = (self.d, self.chart);
        use Body::*;
        use Disposition::{Kroora, Saumya};
        use YogaName as N;

        let lagna = self.rasi_of(Lagna);
        let house = |k| kth(lagna, k);
        let lords_of = |k| self.lords_in_yoga(house(k));
        let marakas_as = |mercury_as| -> Vec<Body> {
            self.maraka_reasons_as(mercury_as)
                .into_iter()
                .map(|m| m.graha)
                .collect()
        };
        let (saumya_marakas, kroora_marakas) = (marakas_as(Saumya), marakas_as(Kroora));
        let marakas_for = |mercury_as| -> &[Body] {
            if mercury_as == Kroora {
                &kroora_marakas
            } else {
                &saumya_marakas
            }
        };
        let dusthana = |r| matches!(bhava(lagna, r), 6 | 8 | 12);
        let aspects = |a: Body, b: Body| self.aspects(a, self.rasi_of(b));
        let on = |a: Body, b: Body| a != b && self.joins_or_aspects(a, self.rasi_of(b));
        let on_from = |g: Body, from: &[Body]| -> Vec<Body> {
            from.iter().copied().filter(|&x| on(x, g)).collect()
        };
        let malefics = |mercury_as| -> Vec<Body> {
            GRAHAS
                .into_iter()
                .filter(|&g| self.natural_nature(g).unwrap_or(mercury_as) == Kroora)
                .collect()
        };
        let dispositors = |v, g| chart.in_varga(v).dispositors_in_yoga(g);
        let row = |name, grahas: &[Body]| Yoga {
            name,
            grahas: self.yoga_grahas(name, &in_graha_order(grahas)),
            to_judge: Vec::new(),
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        let first = lords_of(1);
        let mut found = Vec::new();

        // (1), (2): each of the two conjoined or aspected by a maraka other than himself (ADR 0014).
        for (n, k) in [(1, 12), (2, 6)] {
            let other = lords_of(k);
            found.extend(self.with_mercury_each_way(N::Daridra(n), |mercury_as| {
                let marakas = marakas_for(mercury_as);
                first.iter().find_map(|&l| {
                    other.iter().find_map(|&o| {
                        let (on_l, on_o) = (on_from(l, marakas), on_from(o, marakas));
                        (self.rasi_of(l) == house(k)
                            && self.rasi_of(o) == lagna
                            && !on_l.is_empty()
                            && !on_o.is_empty())
                        .then(|| in_graha_order(&[vec![l, o], on_l, on_o].concat()))
                    })
                })
            }));
        }

        // (3).
        found.extend(self.with_mercury_each_way(N::Daridra(3), |mercury_as| {
            let ketu = self.rasi_of(Ketu);
            let moon: &[Body] = if ketu == self.rasi_of(Moon) {
                &[Moon]
            } else {
                &[]
            };
            if ketu != lagna && moon.is_empty() {
                return None;
            }
            let marakas = marakas_for(mercury_as);
            first.iter().find_map(|&l| {
                let on_l = on_from(l, marakas);
                (self.rasi_of(l) == house(8) && !on_l.is_empty())
                    .then(|| in_graha_order(&[vec![Ketu, l], moon.to_vec(), on_l].concat()))
            })
        }));

        // (4): an enemy's rasi as Raaja (11)'s, the compound relationship with its seven-planet lord (ADR 0014).
        let second = lords_of(2);
        let low = second.iter().copied().find(|&s| {
            debilitated(s, self.rasi_of(s))
                || matches!(
                    self.with_rasi_lord(s),
                    Some(Compound::Satru | Compound::Adhisatru)
                )
        });
        if let Some(s) = low {
            found.extend(self.with_mercury_each_way(N::Daridra(4), |mercury_as| {
                let malefics = malefics(mercury_as);
                first.iter().find_map(|&l| {
                    let with: Vec<Body> = malefics
                        .iter()
                        .copied()
                        .filter(|&m| m != l && self.rasi_of(m) == self.rasi_of(l))
                        .collect();
                    (dusthana(self.rasi_of(l)) && !with.is_empty())
                        .then(|| in_graha_order(&[vec![l, s], with].concat()))
                })
            }));
        }

        // (5): aspects only, each lord by a maraka.
        let (fifth, ninth) = (lords_of(5), lords_of(9));
        found.extend(self.with_mercury_each_way(N::Daridra(5), |mercury_as| {
            let marakas = marakas_for(mercury_as);
            let aspecting = |g| -> Vec<Body> {
                marakas
                    .iter()
                    .copied()
                    .filter(|&m| m != g && aspects(m, g))
                    .collect()
            };
            fifth.iter().find_map(|&f| {
                ninth.iter().find_map(|&n| {
                    let (on_f, on_n) = (aspecting(f), aspecting(n));
                    (self.rasi_of(f) == house(6)
                        && self.rasi_of(n) == house(12)
                        && !on_f.is_empty()
                        && !on_n.is_empty())
                    .then(|| in_graha_order(&[vec![f, n], on_f, on_n].concat()))
                })
            })
        }));

        // (6): a maraka other than the malefic it conjoins or aspects (ADR 0014).
        let ninth_or_tenth_in = [&ninth[..], &lords_of(10)[..]]
            .concat()
            .iter()
            .any(|&g| self.rasi_of(g) == lagna);
        if !ninth_or_tenth_in {
            found.extend(self.with_mercury_each_way(N::Daridra(6), |mercury_as| {
                let marakas = marakas_for(mercury_as);
                let in_lagna: Vec<Body> = malefics(mercury_as)
                    .into_iter()
                    .filter(|&g| self.rasi_of(g) == lagna)
                    .collect();
                let on_them: Vec<Body> = marakas
                    .iter()
                    .copied()
                    .filter(|&m| in_lagna.iter().any(|&g| on(m, g)))
                    .collect();
                (!on_them.is_empty()).then(|| in_graha_order(&[in_lagna, on_them].concat()))
            }));
        }

        // (7): each dusthana lord's dispositor in some dusthana, with a malefic on him (ADR 0014).
        found.extend(self.with_mercury_each_way(N::Daridra(7), |mercury_as| {
            let malefics = malefics(mercury_as);
            let mut grahas = Vec::new();
            for k in [6, 8, 12] {
                let afflicted = lords_of(k).into_iter().find_map(|l| {
                    dispositors(d, l).into_iter().find_map(|g| {
                        let on_g = on_from(g, &malefics);
                        (dusthana(self.rasi_of(g)) && !on_g.is_empty())
                            .then(|| [vec![g], on_g].concat())
                    })
                })?;
                grahas.extend(afflicted);
            }
            Some(in_graha_order(&grahas))
        }));

        // (8), (9): rasi-only, since they read the navamsa (ADR 0014).
        if d == Varga::RasiChart {
            found.extend(self.with_mercury_each_way(N::Daridra(8), |mercury_as| {
                let marakas = marakas_for(mercury_as);
                dispositors(Varga::Navamsa, Moon).into_iter().find_map(|g| {
                    let with: Vec<Body> = marakas
                        .iter()
                        .copied()
                        .filter(|&m| m != g && self.rasi_of(m) == self.rasi_of(g))
                        .collect();
                    (!with.is_empty() || matches!(bhava(lagna, self.rasi_of(g)), 2 | 7))
                        .then(|| in_graha_order(&[vec![Moon, g], with].concat()))
                })
            }));
            let navamsa_first = chart
                .in_varga(Varga::Navamsa)
                .lords_in_yoga(chart.in_varga(Varga::Navamsa).rasi_of(Lagna));
            found.extend(self.with_mercury_each_way(N::Daridra(9), |mercury_as| {
                let marakas = marakas_for(mercury_as);
                first.iter().find_map(|&l| {
                    navamsa_first.iter().find_map(|&n| {
                        let (on_l, on_n) = (on_from(l, marakas), on_from(n, marakas));
                        (!on_l.is_empty() && !on_n.is_empty())
                            .then(|| in_graha_order(&[vec![l, n], on_l, on_n].concat()))
                    })
                })
            }));
        }

        // (11): planets joined by a dusthana lord and untouched by the 1st, 5th and 9th lords (ADR 0014).
        let dusthana_lords = [lords_of(6), lords_of(8), lords_of(12)].concat();
        let trine_lords = [first.clone(), fifth, ninth].concat();
        let mut losing = Vec::new();
        for g in PLANETS {
            let joined: Vec<Body> = dusthana_lords
                .iter()
                .copied()
                .filter(|&l| l != g && self.rasi_of(l) == self.rasi_of(g))
                .collect();
            if !joined.is_empty() && on_from(g, &trine_lords).is_empty() {
                losing.push(g);
                losing.extend(joined);
            }
        }
        if !losing.is_empty() {
            found.push(row(N::Daridra(11), &losing));
        }

        // (12): Mercury's only aspect is his 7th; with it the yoga is absent, and the knowledge base gives its wealth instead.
        if self.rasi_of(Mars) == house(2)
            && self.rasi_of(Saturn) == house(2)
            && !self.aspects(Mercury, house(2))
        {
            found.push(row(N::Daridra(12), &[Mars, Saturn]));
        }
        // (13): likewise without Saturn's aspect.
        if self.rasi_of(Sun) == house(2) && aspects(Saturn, Sun) {
            found.push(row(N::Daridra(13), &[Sun, Saturn]));
        }

        found
    }

    /// The maraka grahas of `d`'s chart, in graha order: the 2nd and 7th lords, and the malefics conjoining or aspecting with graha
    /// drishti those houses or lords, "powerfully" not weighed (ch. 14 §14.2, ADR 0014). A tied Mercury is listed only as a lord;
    /// [`VargaChakra::yogas`] reads him each way.
    pub fn marakas(&self) -> Vec<Body> {
        self.maraka_reasons().into_iter().map(|m| m.graha).collect()
    }

    /// The maraka sthanas of `d`'s chart: the 2nd and 7th rasis from its lagna (ch. 14 §14.2, p. 181).
    pub fn maraka_sthanas(&self) -> [Rasi; 2] {
        let lagna = self.rasi_of(Body::Lagna);
        [kth(lagna, 2), kth(lagna, 7)]
    }

    /// [`VargaChakra::marakas`], each with every reason it qualifies (ch. 14 §14.2, ADR 0014).
    pub fn maraka_reasons(&self) -> Vec<Maraka> {
        let mercury_as = self
            .natural_nature(Body::Mercury)
            .unwrap_or(Disposition::Saumya);
        self.maraka_reasons_as(mercury_as)
    }

    /// [`VargaChakra::maraka_reasons`] with a tied Mercury read as `mercury_as`.
    fn maraka_reasons_as(&self, mercury_as: Disposition) -> Vec<Maraka> {
        let [second, seventh] = self.maraka_sthanas();
        let houses = [(2, second), (7, seventh)];
        let on = |g: Body, r| self.joins_or_aspects(g, r);
        let reasons = |g: Body| {
            let malefic = self.natural_nature(g).unwrap_or(mercury_as) == Disposition::Kroora;
            let mut reasons = Vec::new();
            for (house, r) in houses {
                let lords = self.lords_in_yoga(r);
                if lords.contains(&g) {
                    reasons.push(MarakaReason::Owns(house));
                }
                if !malefic {
                    continue;
                }
                if self.rasi_of(g) == r {
                    reasons.push(MarakaReason::Occupies(house));
                } else if on(g, r) {
                    reasons.push(MarakaReason::Aspects(house));
                }
                for lord in lords.into_iter().filter(|&l| l != g) {
                    if self.rasi_of(g) == self.rasi_of(lord) {
                        reasons.push(MarakaReason::ConjoinsLord(house, lord));
                    } else if on(g, self.rasi_of(lord)) {
                        reasons.push(MarakaReason::AspectsLord(house, lord));
                    }
                }
            }
            reasons
        };
        GRAHAS
            .into_iter()
            .map(|graha| Maraka {
                graha,
                reasons: reasons(graha),
            })
            .filter(|m| !m.reasons.is_empty())
            .collect()
    }

    /// Each way of choosing a lord for `houses` from `d`'s lagna, a co-lord giving two (ADR 0014).
    fn house_lords<const N: usize>(&self, houses: [usize; N]) -> Vec<[Body; N]> {
        let lagna = self.rasi_of(Body::Lagna);
        let mut ways = vec![[Body::Lagna; N]];
        for (i, k) in houses.into_iter().enumerate() {
            let of_house = self.lords_to_try(kth(lagna, k));
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
}

/// Why a graha is a maraka, each naming the house, 2 or 7, it concerns (ch. 14 §14.2, p. 181).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarakaReason {
    /// A lord of the house.
    Owns(u8),
    /// A malefic in the house.
    Occupies(u8),
    /// A malefic aspecting the house by graha drishti.
    Aspects(u8),
    /// A malefic conjoining the house's lord.
    ConjoinsLord(u8, Body),
    /// A malefic aspecting the house's lord by graha drishti.
    AspectsLord(u8, Body),
}

/// A maraka graha and every reason it qualifies, the 2nd house's before the 7th's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Maraka {
    pub graha: Body,
    pub reasons: Vec<MarakaReason>,
}

/// The 8th house from `rasi` for Rudra by Table 32 (ch. 14 §14.3, p. 182) as printed, built by footnote 50's motions, which are
/// not derived here (ADR 0017).
pub fn table_32(rasi: Rasi) -> Rasi {
    use Rasi::*;
    [
        Scorpio,
        Gemini,
        Capricorn,
        Sagittarius,
        Cancer,
        Aquarius,
        Taurus,
        Sagittarius,
        Cancer,
        Gemini,
        Capricorn,
        Leo,
    ][rasi as usize]
}

/// The rule of §14.3's list that decided Rudra, in its order (ch. 14 §14.3, p. 182).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RudraRule {
    /// Conjoins more of the nine grahas.
    Conjoins,
    /// Exalted or in its own rasi.
    ExaltedOrOwn,
    /// Joins exalted planets.
    JoinsExalted,
    /// Aspected by more grahas by rasi drishti.
    Aspected,
    /// More advanced in its rasi.
    Advanced,
}

/// Rudra and the Trishoola rasis (ch. 14 §14.3, ADR 0017).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rudra {
    /// The 8th houses from the lagna and from the 7th.
    pub eighths: [Rasi; 2],
    /// Whether they were counted normally, Saturn being in the lagna or the 7th (ch. 23, p. 276), not by Table 32.
    pub counted_normally: bool,
    /// The stronger of the two 8th lords.
    pub rudra: Body,
    /// The weaker, Rudra too when afflicted.
    pub weaker: Body,
    /// Debilitated or in a natural enemy's rasi, and conjoined or aspected by rasi drishti by Mars, Saturn, Rahu or Ketu.
    pub weaker_afflicted: bool,
    /// The rule that decided; none on a tie through all five.
    pub decided_by: Option<RudraRule>,
    /// The trines from the afflicted weaker's rasi if he is, else from Rudra's, from Aries.
    pub trishoola: [Rasi; 3],
}

/// An exception §14.3 applied to Maheswara (ch. 14 §14.3, p. 183).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaheswaraException {
    /// (1): the lord in his own or exaltation rasi, so the stronger of the 8th and 12th lords from him.
    OwnOrExalted,
    /// (2): Rahu or Ketu with AK or in the 8th from him, so the 6th.
    Node,
    /// (3): Rahu taken as Mercury, Ketu as Jupiter.
    NodeReplaced,
}

/// Maheswara and the exceptions applied to find him, in the order applied: (2), (1), (3) (ch. 14 §14.3, p. 183).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Maheswara {
    pub graha: Body,
    pub exceptions: Vec<MaheswaraException>,
}

impl VargaChakra<'_> {
    /// Rudra in `d`'s chart, the book reading it in the rasi chart, counted from the lagna: the stronger by §14.3's list of the lords
    /// ([`VargaChakra::stronger_lord`]) of Table 32's 8th from the lagna and from the 7th, the weaker listed too when debilitated or
    /// in a natural enemy's rasi and conjoined or aspected by rasi drishti by Mars, Saturn, Rahu or Ketu (ch. 14 §14.3, ADR 0017).
    pub fn rudra(&self) -> Rudra {
        use Body::*;
        let (eighths, counted_normally) = self.rudra_eighths();
        let [a, b] = eighths.map(|r| self.stronger_lord(r));
        let others_in = |g: Body| {
            let at = self.rasi_of(g);
            GRAHAS
                .into_iter()
                .filter(move |&o| o != g && self.rasi_of(o) == at)
        };
        let strength = |g: Body| {
            let at = self.rasi_of(g);
            [
                others_in(g).count(),
                usize::from(own_or_exalted(g, at)),
                usize::from(others_in(g).any(|o| exalted(o, at))),
                GRAHAS
                    .into_iter()
                    .filter(|&o| o != g && rasi_drishti(self.rasi_of(o), at))
                    .count(),
                advancement(self.chart, g) as usize,
            ]
        };
        let rules = [
            RudraRule::Conjoins,
            RudraRule::ExaltedOrOwn,
            RudraRule::JoinsExalted,
            RudraRule::Aspected,
            RudraRule::Advanced,
        ];
        let (sa, sb) = (strength(a), strength(b));
        let decided = (0..5).find(|&i| sa[i] != sb[i]);
        // shortcut: the book has no rule for a tie through all five, so the lagna's 8th lord; upgrade if a later chapter gives one.
        let (rudra, weaker) = match decided {
            Some(i) if sb[i] > sa[i] => (b, a),
            _ => (a, b),
        };
        let at = self.rasi_of(weaker);
        let weak = debilitated(weaker, at) || natural(weaker, at.lord()) == Some(Natural::Satru);
        // A malefic never afflicts himself.
        let afflicted_by = [Mars, Saturn, Rahu, Ketu]
            .into_iter()
            .any(|m| m != weaker && (self.rasi_of(m) == at || rasi_drishti(self.rasi_of(m), at)));
        let weaker_afflicted = weak && afflicted_by;
        let from = self.rasi_of(if weaker_afflicted { weaker } else { rudra });
        let mut trishoola = [1, 5, 9].map(|k| kth(from, k));
        trishoola.sort_by_key(|&r| r as u8);
        Rudra {
            eighths,
            counted_normally,
            rudra,
            weaker,
            weaker_afflicted,
            decided_by: decided.map(|i| rules[i]),
            trishoola,
        }
    }

    /// Table 32's 8th houses from the lagna and the 7th, or both counted normally with Saturn in either (ch. 14 §14.3, p. 182;
    /// ch. 23, p. 276), and whether they were.
    fn rudra_eighths(&self) -> ([Rasi; 2], bool) {
        let lagna = self.rasi_of(Body::Lagna);
        let seventh = kth(lagna, 7);
        let counted_normally = [lagna, seventh].contains(&self.rasi_of(Body::Saturn));
        let eighth = |r| {
            if counted_normally {
                kth(r, 8)
            } else {
                table_32(r)
            }
        };
        ([eighth(lagna), eighth(seventh)], counted_normally)
    }

    /// The method of three pairs in `d`'s chart, the book reading it in the rasi chart, counted from the lagna: the lagna's lord
    /// and the lord of the 8th by [`VargaChakra::rudra`]'s rule, the Moon and Saturn, the lagna and HL, each by Table 33 (ch. 14
    /// §14.4, pp. 183–184, ADR 0017). Without HL there is no pair 3, result or paramaayush.
    pub fn three_pairs(&self) -> ThreePairs {
        use Body::*;
        let pair = |rasis: [Rasi; 2]| Pair {
            rasis,
            longevity: table_33(rasis[0].mobility(), rasis[1].mobility()),
        };
        let lagna = self.rasi_of(Lagna);
        let ([eighth, _], _) = self.rudra_eighths();
        let lords = [self.stronger_lord(lagna), self.stronger_lord(eighth)];
        let pairs = [
            pair(lords.map(|l| self.rasi_of(l))),
            pair([self.rasi_of(Moon), self.rasi_of(Saturn)]),
        ];
        let hl_pair = self
            .reference(ReferencePoint::Special(SpecialLagna::HoraLagna))
            .map(|hl| pair([lagna, hl]));
        let (mut result, mut paramaayush) = (None, None);
        if let Some(third) = hl_pair {
            let [p1, p2, p3] = [pairs[0].longevity, pairs[1].longevity, third.longevity];
            // The agreeing category and the odd one out, itself when all three agree.
            let agreed = [(p1, p2, p3), (p1, p3, p2), (p2, p3, p1)]
                .into_iter()
                .find(|(a, b, _)| a == b)
                .map(|(two, _, odd)| (two, odd));
            if let Some((two, other)) = agreed {
                result = Some((two, PairsRule::Agreement));
                paramaayush = Some(table_34(two, other));
            } else if [lagna, kth(lagna, 7)].contains(&self.rasi_of(Moon)) {
                result = Some((p2, PairsRule::MoonAndSaturn));
            } else {
                result = Some((p3, PairsRule::LagnaAndHl));
            }
        }
        ThreePairs {
            lords,
            pairs,
            hl_pair,
            result,
            paramaayush,
        }
    }

    /// The eighth lord method in `d`'s chart, the book reading it in the rasi chart: the stronger of the lagna and the 7th by
    /// [`VargaChakra::stronger_rasi`], and where its 8th lord, counted normally, is from it (ch. 14 §14.5, p. 185, ADR 0017).
    pub fn eighth_lord_method(&self) -> EighthLordMethod {
        let lagna = self.rasi_of(Body::Lagna);
        let reference = self.stronger_rasi(lagna, kth(lagna, 7));
        let lord = self.stronger_lord(kth(reference, 8));
        let house = bhava(reference, self.rasi_of(lord));
        // Kendras are the 1st, 4th, 7th and 10th, panapharas the next, apoklimas the next again.
        let longevity =
            [Longevity::Long, Longevity::Middle, Longevity::Short][(house as usize - 1) % 3];
        EighthLordMethod {
            reference,
            lord,
            house,
            longevity,
        }
    }

    /// Maheswara in `d`'s chart, the book reading it in the rasi chart: the lord ([`VargaChakra::stronger_lord`]) of the 8th from
    /// the chara AK counted normally, with §14.3's exceptions applied as (2), (1), (3) (ch. 14 §14.3, p. 183, ADR 0017).
    pub fn maheswara(&self) -> Maheswara {
        use Body::*;
        let mut exceptions = Vec::new();
        // shortcut: a tie for AK (ch. 8 §8.2) takes the first in graha order, as `reference` does; upgrade when a later chapter settles it.
        let karakas = self.chart.chara_karakas();
        let ak = karakas.iter().find(|k| k.1 == CharaKaraka::Atma).unwrap().0;
        let at = self.rasi_of(ak);
        // A node as AK does not join himself: (2) says "joins AK".
        let node_on = |r| {
            [Rahu, Ketu]
                .iter()
                .any(|&n| n != ak && self.rasi_of(n) == r)
        };
        let house = if node_on(at) || node_on(kth(at, 8)) {
            exceptions.push(MaheswaraException::Node);
            6
        } else {
            8
        };
        let mut graha = self.stronger_lord(kth(at, house));
        let his = self.rasi_of(graha);
        if own_or_exalted(graha, his) {
            exceptions.push(MaheswaraException::OwnOrExalted);
            // The 8th and 12th, or after (2) the 6th and 2nd, the 8th and 12th counted back.
            let (first, second) = if house == 8 { (8, 12) } else { (6, 2) };
            graha = self.stronger_planet(
                self.stronger_lord(kth(his, first)),
                self.stronger_lord(kth(his, second)),
            );
        }
        if let Some(instead) = match graha {
            Rahu => Some(Mercury),
            Ketu => Some(Jupiter),
            _ => None,
        } {
            exceptions.push(MaheswaraException::NodeReplaced);
            graha = instead;
        }
        Maheswara { graha, exceptions }
    }
}

/// A longevity category (ch. 14 §14.4, p. 184), from short to long: [`Longevity::years`] and [`table_34`] rely on this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Longevity {
    Short,
    Middle,
    Long,
}

impl Longevity {
    /// Its range in years: short 0–36, middle 36–72, long 72–108 (ch. 14 §14.4, p. 184).
    pub fn years(self) -> (u8, u8) {
        let from = 36 * self as u8;
        (from, from + 36)
    }
}

/// The longevity category of a pair whose rasis have mobilities `a` and `b`, by Table 33 (ch. 14 §14.4, pp. 183–184).
pub fn table_33(a: Mobility, b: Mobility) -> Longevity {
    use Mobility::*;
    match (a, b) {
        (Sthira, Dwiswabhava) | (Dwiswabhava, Sthira) | (Chara, Chara) => Longevity::Long,
        (Chara, Sthira) | (Sthira, Chara) | (Dwiswabhava, Dwiswabhava) => Longevity::Middle,
        (Chara, Dwiswabhava) | (Dwiswabhava, Chara) | (Sthira, Sthira) => Longevity::Short,
    }
}

/// A pair of the method of three pairs: its two rasis, and their category by Table 33 (ch. 14 §14.4, p. 183).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pair {
    pub rasis: [Rasi; 2],
    pub longevity: Longevity,
}

/// The rule that combined the three pairs (ch. 14 §14.4, p. 184).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairsRule {
    /// Two or three pairs agree.
    Agreement,
    /// All three differ, so the lagna and HL pair.
    LagnaAndHl,
    /// All three differ with the Moon in the lagna or the 7th, so the Moon and Saturn pair.
    MoonAndSaturn,
}

/// The method of three pairs (ch. 14 §14.4, ADR 0017).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThreePairs {
    /// Pair 1's grahas: the lagna's lord and the 8th lord.
    pub lords: [Body; 2],
    /// Pair 1, then pair 2, the Moon and Saturn.
    pub pairs: [Pair; 2],
    /// Pair 3, the lagna and HL; none without a Vedic day.
    pub hl_pair: Option<Pair>,
    /// The combined category and the rule that combined it; none without pair 3.
    pub result: Option<(Longevity, PairsRule)>,
    /// By Table 34; none when all three differ or without pair 3.
    pub paramaayush: Option<u8>,
}

/// The eighth lord method (ch. 14 §14.5, ADR 0017).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EighthLordMethod {
    /// The stronger of the lagna and the 7th.
    pub reference: Rasi,
    /// The 8th lord from it, counted normally.
    pub lord: Body,
    /// The house he is in from it.
    pub house: u8,
    /// Long life in a kendra, middle in a panaphara, short in an apoklima.
    pub longevity: Longevity,
}

/// The paramaayush in years of `two` agreeing pairs against the `third`, by Table 34 (ch. 14 §14.4, p. 184).
pub fn table_34(two: Longevity, third: Longevity) -> u8 {
    [[32, 64, 96], [36, 72, 108], [40, 80, 120]][third as usize][two as usize]
}

/// A two-halved yoga's form: 1 for the first half, 2 for the second, 3 for both (see [`Yoga::form`]).
fn form_of(first: bool, second: bool) -> Option<u8> {
    match (first, second) {
        (true, false) => Some(1),
        (false, true) => Some(2),
        (true, true) => Some(3),
        (false, false) => None,
    }
}

impl VargaChakra<'_> {
    /// `g`'s dispositors in `v`'s chart for §11.7–11.10: himself in his own rasi (p. 128), else [`VargaChakra::lords_in_yoga`]'s (ADR 0014).
    fn dispositors_in_yoga(&self, g: Body) -> Vec<Body> {
        let rasi = self.rasi_of(g);
        if owns(g, rasi) {
            vec![g]
        } else {
            self.lords_in_yoga(rasi)
        }
    }
}

/// The row of a yoga with several forms or lords that hold: the least left to judge, then the first (ADR 0014).
fn the_best_of(rows: Vec<Yoga>) -> Option<Yoga> {
    rows.into_iter().min_by_key(|row| row.to_judge.len())
}

impl VargaChakra<'_> {
    /// `grahas` with their bhavas and every lordship, from `d`'s lagna, by the lords `name`'s rule tries.
    fn yoga_grahas(&self, name: YogaName, grahas: &[Body]) -> Vec<YogaGraha> {
        let lagna = self.rasi_of(Body::Lagna);
        // §11.6 also counts a co-lord at home, so its row names the lordship that formed it (p. 218, ADR 0014).
        let lords = RASIS.map(|rasi| {
            if name.class() == YogaClass::OtherPopular {
                self.lords_to_try(rasi)
            } else {
                self.lords_in_yoga(rasi)
            }
        });
        grahas
            .iter()
            .map(|&graha| YogaGraha {
                graha,
                bhava: bhava(lagna, self.rasi_of(graha)),
                // RASIS from Aries, so sorted after counting from the lagna.
                named_lord_of: {
                    let mut lord_of: Vec<u8> = RASIS
                        .into_iter()
                        .zip(&lords)
                        .filter(|(_, lords)| lords.contains(&graha))
                        .map(|(rasi, _)| bhava(lagna, rasi))
                        .collect();
                    lord_of.sort();
                    lord_of
                },
                karaka: None,
            })
            .collect()
    }
}

/// `grahas` once each, in the book's order.
fn in_graha_order(grahas: &[Body]) -> Vec<Body> {
    GRAHAS.into_iter().filter(|g| grahas.contains(g)).collect()
}

impl VargaChakra<'_> {
    /// The yoga `rule` gives its grahas for, Mercury being `Saumya` or `Kroora`: as he is in `d`'s chart, or each way on a tie, when
    /// agreement is the result and a difference is to judge (ch. 3 §3.2.2, ADR 0014). On agreement the saumya reading's grahas stand.
    fn with_mercury_each_way(
        &self,
        name: YogaName,
        rule: impl Fn(Disposition) -> Option<Vec<Body>>,
    ) -> Option<Yoga> {
        use Disposition::{Kroora, Saumya};
        let (grahas, to_judge) = match self.natural_nature(Body::Mercury) {
            Some(nature) => (rule(nature)?, Vec::new()),
            None => match (rule(Saumya), rule(Kroora)) {
                (Some(grahas), Some(_)) => (grahas, Vec::new()),
                (Some(grahas), None) | (None, Some(grahas)) => {
                    (grahas, vec![ToJudge::MercurysNature])
                }
                (None, None) => return None,
            },
        };
        Some(Yoga {
            name,
            grahas: self.yoga_grahas(name, &grahas),
            to_judge,
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        })
    }
}

/// The eight references of ashtakavarga, in the order of Tables 19–26's columns: the seven planets, then the lagna. The same
/// eight have an ashtakavarga; Rahu and Ketu take no part (ch. 12 §12.2, ADR 0015).
pub const AV_REFERENCES: [Body; 8] = [
    Body::Sun,
    Body::Moon,
    Body::Mars,
    Body::Mercury,
    Body::Jupiter,
    Body::Venus,
    Body::Saturn,
    Body::Lagna,
];

/// Parasara's benefic houses (rekhas) of each of [`AV_REFERENCES`], counted from each of them in the same order: ch. 12 Table 19
/// (p. 146), Tables 20–21 (p. 147), 22–23 (p. 148), 24–25 (p. 149) and 26 (p. 150), as printed (ADR 0015).
const REKHAS: [[&[u8]; 8]; 8] = [
    // Table 19: Sun.
    [
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 6, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 5, 6, 9, 10, 11, 12],
        &[5, 6, 9, 11],
        &[6, 7, 12],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 4, 6, 10, 11, 12],
    ],
    // Table 20: Moon.
    [
        &[3, 6, 7, 8, 10, 11],
        &[1, 3, 6, 7, 9, 10, 11],
        &[2, 3, 5, 6, 10, 11],
        &[1, 3, 4, 5, 7, 8, 10, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 4, 5, 7, 9, 10, 11],
        &[3, 5, 6, 11],
        &[3, 6, 10, 11],
    ],
    // Table 21: Mars.
    [
        &[3, 5, 6, 10, 11],
        &[3, 6, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 5, 6, 11],
        &[6, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 4, 7, 8, 9, 10, 11],
        &[1, 3, 6, 10, 11],
    ],
    // Table 22: Mercury.
    [
        &[5, 6, 9, 11, 12],
        &[2, 4, 6, 8, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 3, 5, 6, 9, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 6, 8, 10, 11],
    ],
    // Table 23: Jupiter.
    [
        &[1, 2, 3, 4, 7, 8, 9, 10, 11],
        &[2, 5, 7, 9, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[1, 2, 4, 5, 6, 9, 10, 11],
        &[1, 2, 3, 4, 7, 8, 10, 11],
        &[2, 5, 6, 9, 10, 11],
        &[3, 5, 6, 12],
        &[1, 2, 4, 5, 6, 7, 9, 10, 11],
    ],
    // Table 24: Venus.
    [
        &[8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11, 12],
        &[3, 4, 6, 9, 11, 12],
        &[3, 5, 6, 9, 11],
        &[5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[3, 4, 5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 11],
    ],
    // Table 25: Saturn.
    [
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 6, 11],
        &[3, 5, 6, 10, 11, 12],
        &[6, 8, 9, 10, 11, 12],
        &[5, 6, 11, 12],
        &[6, 11, 12],
        &[3, 5, 6, 11],
        &[1, 3, 4, 6, 10, 11],
    ],
    // Table 26: the lagna.
    [
        &[3, 4, 6, 10, 11, 12],
        &[3, 6, 10, 11, 12],
        &[1, 3, 6, 10, 11],
        &[1, 2, 4, 6, 8, 10, 11],
        &[1, 2, 4, 5, 6, 7, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9],
        &[1, 3, 4, 6, 10, 11],
        &[3, 6, 10, 11],
    ],
];

/// A planet's BAV count in a rasi, read as §12.3 reads it (ADR 0015).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BavBand {
    /// 5 to 8 rekhas.
    Favourable,
    /// 4 rekhas.
    Neutral,
    /// 0 to 3 rekhas.
    Unfavourable,
}

/// The band of `count` rekhas in `of`'s BAV; the lagna's has none, as §12.3 speaks of a planet (ADR 0015).
pub fn bav_band(of: Body, count: u8) -> Option<BavBand> {
    PLANETS.contains(&of).then_some(match count {
        5.. => BavBand::Favourable,
        4 => BavBand::Neutral,
        _ => BavBand::Unfavourable,
    })
}

/// An SAV count in a rasi, read as §12.4 reads it: its "30 or more" comes first, so 30 is strong, not average (ADR 0015).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavBand {
    /// 30 or more rekhas.
    Strong,
    /// 25 to 29 rekhas.
    Average,
    /// Under 25 rekhas.
    Weak,
}

/// The band of `count` rekhas in an SAV (ch. 12 §12.4).
pub fn sav_band(count: u8) -> SavBand {
    match count {
        30.. => SavBand::Strong,
        25.. => SavBand::Average,
        _ => SavBand::Weak,
    }
}

impl VargaChakra<'_> {
    /// `of`'s PAV, `of` being one of [`AV_REFERENCES`]: for each reference in that order, whether it gives `of` a rekha in each rasi
    /// from Aries, houses counted as whole rasis from the reference's rasi in `d`'s chart (ch. 12 §12.6, §12.8; ADR 0010).
    /// Panics on Rahu or Ketu, a programmer's error.
    pub fn pav(&self, of: Body) -> [[bool; 12]; 8] {
        let of = AV_REFERENCES
            .iter()
            .position(|&b| b == of)
            .expect("only the seven planets and the lagna have an ashtakavarga");
        std::array::from_fn(|from| {
            let at = self.rasi_of(AV_REFERENCES[from]);
            RASIS.map(|rasi| REKHAS[of][from].contains(&bhava(at, rasi)))
        })
    }

    /// `of`'s BAV: its PAV's rekhas in each rasi from Aries, 0 to 8 (ch. 12 §12.3).
    pub fn bav(&self, of: Body) -> [u8; 12] {
        let pav = self.pav(of);
        std::array::from_fn(|rasi| pav.iter().filter(|from| from[rasi]).count() as u8)
    }

    /// The SAV: the seven planets' BAVs summed in each rasi from Aries, the lagna's not included (ch. 12 §12.4).
    pub fn sav(&self) -> [u8; 12] {
        let bavs = PLANETS.map(|planet| self.bav(planet));
        std::array::from_fn(|rasi| bavs.iter().map(|bav| bav[rasi]).sum())
    }

    /// `of`'s SoAV, `of` being one of the seven planets: its BAV after trikona and then ekaadhipatya sodhana, a rasi occupied when
    /// any of [`AV_REFERENCES`] is in it, the lagna included and Rahu and Ketu not (ch. 12 §12.7, ADR 0015). Panics on the
    /// lagna, Rahu or Ketu, a programmer's error: §12.7 reduces only a planet's BAV.
    pub fn soav(&self, of: Body) -> [u8; 12] {
        assert!(PLANETS.contains(&of), "only the seven planets have an SoAV");
        let at = AV_REFERENCES.map(|b| self.rasi_of(b));
        let occupied = RASIS.map(|rasi| at.contains(&rasi));
        ekaadhipatya_sodhana(trikona_sodhana(self.bav(of)), occupied)
    }

    /// `of`'s rasi and graha pindas from its [`soav`](Self::soav) (ch. 12 §12.7.3); panics as that does.
    pub fn pindas(&self, of: Body) -> Pindas {
        pindas(self.soav(of), PLANETS.map(|p| self.rasi_of(p)))
    }
}

/// The pindas of an SoAV from Aries, `at` holding the rasis of [`PLANETS`] (ch. 12 §12.7.3).
fn pindas(soav: [u8; 12], at: [Rasi; 7]) -> Pindas {
    Pindas {
        rasi: soav
            .iter()
            .zip(RASI_MULTIPLIERS)
            .map(|(&n, m)| u32::from(n) * m)
            .sum(),
        graha: at
            .iter()
            .zip(GRAHA_MULTIPLIERS)
            .map(|(&r, m)| u32::from(soav[r as usize]) * m)
            .sum(),
    }
}

/// Table 28's multipliers (p. 159) from Aries, with Virgo 5, not the printed 6: an erratum, as Exercise 22's rasi pindas (p. 165)
/// all match only with 5 (ADR 0015).
pub const RASI_MULTIPLIERS: [u32; 12] = [7, 10, 8, 4, 10, 5, 7, 8, 9, 5, 11, 12];

/// Table 29's multipliers (p. 160), in the order of [`PLANETS`].
pub const GRAHA_MULTIPLIERS: [u32; 7] = [5, 5, 8, 5, 10, 7, 5];

/// A planet's pindas from its SoAV (ch. 12 §12.7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pindas {
    /// Σ of the SoAV in each rasi × Table 28's multiplier.
    pub rasi: u32,
    /// Σ over the seven planets of the SoAV in that planet's rasi × Table 29's multiplier.
    pub graha: u32,
}

impl Pindas {
    /// Sodhya pinda: rasi pinda + graha pinda.
    pub fn sodhya(&self) -> u32 {
        self.rasi + self.graha
    }
}

/// Trikona sodhana of a BAV from Aries: each set of mutually trinal rasis loses its lowest value, which also covers rules (1)
/// and (2); not footnote 44's "some authors" (ch. 12 §12.7.1, ADR 0015).
pub fn trikona_sodhana(bav: [u8; 12]) -> [u8; 12] {
    let low: [u8; 4] = std::array::from_fn(|r| bav[r].min(bav[r + 4]).min(bav[r + 8]));
    std::array::from_fn(|r| bav[r] - low[r % 4])
}

/// Ekaadhipatya sodhana of `values` from Aries, `occupied` saying which rasis hold a reference: rules (1)–(4) on each co-owned
/// pair, an empty rasi equal to the occupied one becoming 0 under rule (3) (ch. 12 §12.7.2, ADR 0015).
fn ekaadhipatya_sodhana(values: [u8; 12], occupied: [bool; 12]) -> [u8; 12] {
    use Rasi::*;
    let mut out = values;
    for (a, b) in [
        (Aries, Scorpio),
        (Taurus, Libra),
        (Gemini, Virgo),
        (Sagittarius, Pisces),
        (Capricorn, Aquarius),
    ] {
        let (a, b) = (a as usize, b as usize);
        let (va, vb) = (values[a], values[b]);
        if va == 0 || vb == 0 {
            continue;
        }
        let empty = |e: u8, o: u8| if e <= o { 0 } else { o };
        match (occupied[a], occupied[b]) {
            (true, true) => {}
            (true, false) => out[b] = empty(vb, va),
            (false, true) => out[a] = empty(va, vb),
            (false, false) if va == vb => [out[a], out[b]] = [0, 0],
            (false, false) => [out[a], out[b]] = [va.min(vb); 2],
        }
    }
    out
}

/// Table 35 (ch. 15 §15.4.1, p. 190), with the results each gives: a quarter, half, full, some, none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeAvastha {
    Saisava,
    Kumaara,
    Yuva,
    Vriddha,
    Mrita,
}

/// The age avastha at `longitude`: 6° steps from the rasi's start in an odd rasi, from its end in an even one; the nodes' too,
/// by their degree as it stands (ch. 15 §15.4.1, Table 35, p. 190).
pub fn age_avastha(longitude: f64) -> AgeAvastha {
    use AgeAvastha::*;
    let (r, degrees) = rasi(longitude);
    // Whole arcseconds, as `dignity` reads a printed boundary.
    let step = (arcseconds(degrees) / (6 * 3600)) as usize;
    let step = match r.parity() {
        Parity::Vishama => step,
        Parity::Sama => 4 - step,
    };
    [Saisava, Kumaara, Yuva, Vriddha, Mrita][step]
}

/// Ch. 15 §15.4.2 (pp. 190–191): full, medium and negligible results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alertness {
    Jaagrita,
    Swapna,
    Sushupta,
}

/// Ch. 15 §15.4.3 (pp. 191–192): the nine, then the six more, Mudita once for either of its two forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoodAvastha {
    Deepta,
    Swastha,
    Mudita,
    Saanta,
    Deena,
    Duhkhita,
    Vikala,
    Khala,
    /// "Joined closely by Sun": the closeness is to judge, since hora has no combustion orbs (ADR 0018).
    Kopita,
    Lajjita,
    Garvita,
    Kshudhita,
    Trishita,
    Kshobhita,
}

/// A graha's age, alertness and mood avasthas in the rasi chart (ch. 15 §15.4.1–15.4.3, ADR 0018).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Avasthas {
    pub graha: Body,
    pub age: AgeAvastha,
    /// None for a node neither at home, exalted nor debilitated: it has no relationship to read (ADR 0018).
    pub alertness: Option<Alertness>,
    /// Every mood avastha that holds, in [`MoodAvastha`]'s order.
    pub moods: Vec<MoodAvastha>,
}

impl Chart {
    /// The nine grahas' avasthas in the book's order, in the rasi chart whatever the reference point (ADR 0018).
    pub fn avasthas(&self) -> Vec<Avasthas> {
        let d1 = self.in_varga(Varga::RasiChart);
        GRAHAS
            .into_iter()
            .map(|graha| Avasthas {
                graha,
                age: age_avastha(self.entry(graha).longitude),
                alertness: d1.alertness(graha),
                moods: d1.moods(graha),
            })
            .collect()
    }
}

impl VargaChakra<'_> {
    /// §15.4.2 by whole rasi: at home or exalted first, then debilitated, then the compound relationship with the rasi's
    /// seven-planet lord (ADR 0018).
    fn alertness(&self, graha: Body) -> Option<Alertness> {
        use Compound::*;
        let at = self.rasi_of(graha);
        if own_or_exalted(graha, at) {
            return Some(Alertness::Jaagrita);
        }
        if debilitated(graha, at) {
            return Some(Alertness::Sushupta);
        }
        Some(match self.with_rasi_lord(graha)? {
            Adhimitra | Mitra | Sama => Alertness::Swapna,
            Satru | Adhisatru => Alertness::Sushupta,
        })
    }
}

impl VargaChakra<'_> {
    /// §15.4.3 by whole rasi: friend, neutral and enemy by the compound relationship, malefic and benefic by `natural_nature`,
    /// aspects by rasi drishti, and no graha joining himself (ADR 0018). A node has no relationship, so no rung, Kshudhita or
    /// Kshobhita, and its Trishita needs no enemy (ADR 0018).
    fn moods(&self, graha: Body) -> Vec<MoodAvastha> {
        use Compound::*;
        use MoodAvastha::*;
        let at = self.rasi_of(graha);
        let others = || GRAHAS.into_iter().filter(move |&g| g != graha);
        let joined_by = |g: Body| g != graha && self.rasi_of(g) == at;
        let malefic = |g| self.natural_nature(g) == Some(Disposition::Kroora);
        let aspected_by = |g: Body| rasi_drishti(self.rasi_of(g), at);
        let benefic = |g| self.natural_nature(g) == Some(Disposition::Saumya);
        let node = !PLANETS.contains(&graha);
        let friend = |g| matches!(self.compound_with(graha, g), Some(Adhimitra | Mitra));
        let enemy = |g| matches!(self.compound_with(graha, g), Some(Satru | Adhisatru));
        let lord = self.with_rasi_lord(graha);
        let lord_is = |rungs: &[Compound]| lord.is_some_and(|c| rungs.contains(&c));
        let in_enemys_rasi = lord_is(&[Satru, Adhisatru]);
        let mudita_by_company = lord_is(&[Adhimitra, Mitra])
            && others().any(|g| friend(g) && (joined_by(g) || aspected_by(g)))
            && joined_by(Body::Jupiter);
        let fifth = kth(self.rasi_of(Body::Lagna), 5);
        let shaming = [Body::Sun, Body::Mars, Body::Saturn, Body::Rahu, Body::Ketu];
        let watery = at.element() == Element::Jala;
        [
            (Deepta, exalted(graha, at)),
            (Swastha, owns(graha, at)),
            (Mudita, lord_is(&[Adhimitra]) || mudita_by_company),
            (Saanta, lord_is(&[Mitra])),
            (Deena, lord_is(&[Sama])),
            (Duhkhita, in_enemys_rasi),
            (Vikala, others().any(|g| joined_by(g) && malefic(g))),
            (Khala, malefic(at.lord())),
            (Kopita, joined_by(Body::Sun)),
            (Lajjita, at == fifth && shaming.into_iter().any(joined_by)),
            (Garvita, exalted(graha, at) || moolatrikona(graha, at)),
            (
                Kshudhita,
                in_enemys_rasi
                    || others().any(|g| enemy(g) && (joined_by(g) || aspected_by(g)))
                    || joined_by(Body::Saturn) && !node,
            ),
            (
                Trishita,
                watery
                    && (node || others().any(|g| enemy(g) && aspected_by(g)))
                    && !others().any(|g| benefic(g) && aspected_by(g)),
            ),
            (
                Kshobhita,
                !node
                    && joined_by(Body::Sun)
                    && others().any(|g| aspected_by(g) && (malefic(g) || enemy(g))),
            ),
        ]
        .into_iter()
        .filter_map(|(state, holds)| holds.then_some(state))
        .collect()
    }
}

/// Table 36 (ch. 15 §15.4.4, pp. 192–193), in index order, Sayana 1 to Nidraa 12.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SayanaadiAvastha {
    Sayana,
    Upavesana,
    Netrapaani,
    Prakaasana,
    Gamana,
    Aagamana,
    Sabhaa,
    Aagama,
    Bhojana,
    Nriyalipsaa,
    Kautuka,
    Nidraa,
}

const SAYANAADI_AVASTHAS: [SayanaadiAvastha; 12] = {
    use SayanaadiAvastha::*;
    [
        Sayana,
        Upavesana,
        Netrapaani,
        Prakaasana,
        Gamana,
        Aagamana,
        Sabhaa,
        Aagama,
        Bhojana,
        Nriyalipsaa,
        Kautuka,
        Nidraa,
    ]
};

/// How fully a graha gives its Sayanaadi avastha's results: full, medium, very little (ch. 15 §15.4.4, p. 193).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStrength {
    Cheshta,
    Drishti,
    Vicheshta,
}

/// A graha's Sayanaadi avastha in the rasi chart (ch. 15 §15.4.4, ADR 0018).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sayanaadi {
    pub graha: Body,
    pub avastha: SayanaadiAvastha,
    /// None without the name's sound.
    pub strength: Option<ActivityStrength>,
}

/// The ghati running at `birth` (UT Julian day): whole 24-minute ghatis since the Vedic day's sunrise, plus 1, so a night birth
/// counts from the sunrise before it (ch. 15 §15.4.4, footnote 52, p. 192).
pub fn ghati(day: &VedicDay, birth: f64) -> u32 {
    // Whole seconds, so a birth printed on a ghati's boundary is not a hair short of it.
    let seconds = ((birth - day.sunrise) * 86400.0).round() as u32;
    seconds / (24 * 60) + 1
}

/// A of the Sayanaadi formula: the 3°20′ part of its rasi `longitude` is in, 1–9, counted from the rasi's start for the nodes too
/// (ch. 15 §15.4.4, footnote 51, p. 192, ADR 0018).
fn sayanaadi_part(longitude: f64) -> u32 {
    arcseconds(rasi(longitude).1) / (NAKSHATRA / 4) + 1
}

/// The nine grahas' Sayanaadi avasthas in the book's order, in the rasi chart, with their strength given the name's sound
/// `sound` (1–5, Table 37, validated by the caller); None without a Vedic day, whose sunrise G counts from (ch. 15 §15.4.4,
/// pp. 192–193, ADR 0018).
pub fn sayanaadi_avasthas(sky: &Sky, sound: Option<u8>) -> Option<Vec<Sayanaadi>> {
    use Body::*;
    let g = ghati(sky.vedic_day.as_ref()?, sky.jd);
    let chart = chart(sky);
    let nakshatra_number = |b| nakshatra(chart.entry(b).longitude).0 as u32 + 1;
    let l = rasi(chart.entry(Lagna).longitude).0 as u32 + 1;
    let m = nakshatra_number(Moon);
    Some(
        GRAHAS
            .into_iter()
            .map(|graha| {
                // P: Sun 1 to Saturn 7, Rahu 8, Ketu 9 (ADR 0018).
                let p = graha as u32;
                let a = sayanaadi_part(chart.entry(graha).longitude);
                // A remainder of 0 is Nidraa, 12.
                let index = (nakshatra_number(graha) * p * a + m + g + l + 11) % 12 + 1;
                let adjustment = match graha {
                    Sun | Jupiter => 5,
                    Moon | Mars => 2,
                    Mercury | Venus | Saturn => 3,
                    Rahu | Ketu => 4,
                    Lagna => unreachable!("GRAHAS holds no lagna"),
                };
                Sayanaadi {
                    graha,
                    avastha: SAYANAADI_AVASTHAS[index as usize - 1],
                    strength: sound.map(|s| {
                        use ActivityStrength::*;
                        [Vicheshta, Drishti, Cheshta]
                            [((index * index + u32::from(s)) % 12 + adjustment) as usize % 3]
                    }),
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests;

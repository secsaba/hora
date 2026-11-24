//! The chart prepared from a Sky, in the book's terms (ch. 1 *Basic Concepts*); no web code.

pub use hora_swe::arcseconds;
use hora_swe::{KARANA, NAKSHATRA, NewMoon, Position, Sky, TITHI, VedicDay, arcseconds_from_aries};

mod arudhas;
mod aspects;
mod grahas;
mod interpretation;
mod karakas;
mod panchanga;
mod rasis;
mod special_lagnas;
mod strength;
mod upagrahas;
mod vargas;
mod yogas;
pub use aspects::*;
pub use grahas::*;
pub use interpretation::*;
pub use karakas::*;
pub use panchanga::*;
pub use rasis::*;
pub use special_lagnas::*;
pub use strength::*;
pub use upagrahas::*;
pub use vargas::*;
pub use yogas::*;

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

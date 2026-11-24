//! The chart prepared from a Sky, in the book's terms (ch. 1 *Basic Concepts*); no web code.

pub use hora_swe::arcseconds;
use hora_swe::{KARANA, NAKSHATRA, NewMoon, Position, Sky, TITHI, VedicDay, arcseconds_from_aries};

mod arudhas;
mod ashtakavarga;
mod aspects;
mod grahas;
mod interpretation;
mod karakas;
mod longevity;
mod panchanga;
mod rasis;
mod special_lagnas;
mod strength;
mod upagrahas;
mod vargas;
mod yogas;
pub use ashtakavarga::*;
pub use aspects::*;
pub use grahas::*;
pub use interpretation::*;
pub use karakas::*;
pub use longevity::*;
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

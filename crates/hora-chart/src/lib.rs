//! The chart prepared from a Sky, in the book's terms (ch. 1 *Basic Concepts*); no web code.

pub use hora_swe::arcseconds;
use hora_swe::{KARANA, NAKSHATRA, NewMoon, Position, Sky, TITHI, VedicDay, arcseconds_from_aries};

mod arudhas;
mod ashtakavarga;
mod aspects;
mod avasthas;
mod grahas;
mod interpretation;
mod karakas;
mod longevity;
mod panchaanga;
mod rasis;
mod special_lagnas;
mod strength;
mod upagrahas;
mod vargas;
mod yogas;
pub use ashtakavarga::*;
pub use aspects::*;
pub use avasthas::*;
pub use grahas::*;
pub use interpretation::*;
pub use karakas::*;
pub use longevity::*;
pub use panchaanga::*;
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

#[cfg(test)]
mod tests;

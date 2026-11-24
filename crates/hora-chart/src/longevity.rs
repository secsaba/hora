use crate::*;

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
    pub(crate) fn maraka_reasons_as(&self, mercury_as: Disposition) -> Vec<Maraka> {
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

#[cfg(test)]
mod tests;

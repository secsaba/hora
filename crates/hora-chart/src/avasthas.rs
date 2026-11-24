use crate::*;

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

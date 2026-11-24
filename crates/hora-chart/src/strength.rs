use crate::*;

/// `rasi`'s owners in Table 6, so Mars and Ketu for Scorpio, Saturn and Rahu for Aquarius (ch. 15 §15.5.1).
pub(crate) fn lords(rasi: Rasi) -> impl Iterator<Item = Body> {
    GRAHAS
        .into_iter()
        .filter(move |g| g.dignities().is_some_and(|d| d.own.contains(&rasi)))
}

impl VargaChakra<'_> {
    /// How many of Jupiter, Mercury and `lords`, other than `graha`, occupy or aspect `rasi` in `d`'s chart. These are roles,
    /// not grahas, so Mercury as a lord counts twice (ch. 15 §15.5.1 and §15.5.2, rule 2; ADR 0012).
    fn roles_on(
        &self,
        rasi: Rasi,
        lords: impl Iterator<Item = Body>,
        graha: Option<Body>,
    ) -> usize {
        [Body::Jupiter, Body::Mercury]
            .into_iter()
            .chain(lords)
            .filter(|&g| Some(g) != graha)
            .map(|g| self.rasi_of(g))
            .filter(|&at| at == rasi || rasi_drishti(at, rasi))
            .count()
    }
}

/// Exalted by whole rasi, since a varga has no degrees (ADR 0012).
pub(crate) fn exalted(graha: Body, at: Rasi) -> bool {
    graha.dignities().is_some_and(|d| d.uchcha.0 == at)
}

/// In its moolatrikona rasi by whole rasi, as [`exalted`] is (ADR 0014).
pub(crate) fn moolatrikona(graha: Body, at: Rasi) -> bool {
    graha.dignities().is_some_and(|d| d.moolatrikona.0 == at)
}

/// One of `rasi`'s [`lords`].
pub(crate) fn owns(graha: Body, rasi: Rasi) -> bool {
    lords(rasi).any(|l| l == graha)
}

/// In its own or exaltation rasi by whole rasi, as [`exalted`] is.
pub(crate) fn own_or_exalted(graha: Body, at: Rasi) -> bool {
    exalted(graha, at) || owns(graha, at)
}

/// Debilitated by whole rasi, as [`exalted`] is.
pub(crate) fn debilitated(graha: Body, at: Rasi) -> bool {
    graha.dignities().is_some_and(|d| d.neecha.0 == at)
}

/// `graha`'s advancement in its rasi in the rasi chart, whichever chart is being read, in whole arcseconds; the nodes'
/// measured back from the rasi's end (ch. 15 §15.5.1 rule 5b, ch. 8 §8.2, ADR 0011, ADR 0012).
pub(crate) fn advancement(chart: &Chart, graha: Body) -> u32 {
    let at = arcseconds(rasi(chart.entry(graha).longitude).1);
    if matches!(graha, Body::Rahu | Body::Ketu) {
        30 * 3600 - at
    } else {
        at
    }
}

/// The rule of §15.5.1 that found the stronger of two grahas (ch. 15 §15.5.1, pp. 202–203). Rule 5a, for dasa lengths, waits for
/// the dasas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrongerRule {
    /// The basic rule for co-lords: one is in the rasi, so the other is its lord. Never in [`STRONGER_RULES`].
    Basic,
    /// (1) joined by more grahas.
    Joined,
    /// (2) conjoined or aspected by more of Jupiter, Mercury and its dispositor.
    JupiterMercuryDispositor,
    /// (3) exalted.
    Exalted,
    /// (4) in a rasi stronger by mobility: dual over fixed over movable.
    Mobility,
    /// (5b) more advanced in its rasi.
    Advancement,
}

/// In the order of `stronger_planet_by`'s strength components.
const STRONGER_RULES: [StrongerRule; 5] = {
    use StrongerRule::*;
    [
        Joined,
        JupiterMercuryDispositor,
        Exalted,
        Mobility,
        Advancement,
    ]
};

impl VargaChakra<'_> {
    /// `rasi`'s lord in `d`'s chart: its only one, or the stronger of Scorpio's or Aquarius's co-lords by §15.5.1's basic rule,
    /// then rules 1, 2, 3, 4 and 5b (ch. 15 §15.5.1, ADR 0012). Rule 5a, for dasa lengths, waits for the dasas.
    pub fn stronger_lord(&self, rasi: Rasi) -> Body {
        self.stronger_lord_by(rasi).0
    }

    /// [`VargaChakra::stronger_lord`] with the rule that decided: None for a rasi with one lord, or a tie through rule 5b.
    pub fn stronger_lord_by(&self, rasi: Rasi) -> (Body, Option<StrongerRule>) {
        let lord = rasi.lord();
        let Some(co_lord) = lords(rasi).find(|&g| g != lord) else {
            return (lord, None);
        };
        match (self.rasi_of(lord) == rasi, self.rasi_of(co_lord) == rasi) {
            (true, false) => (co_lord, Some(StrongerRule::Basic)),
            (false, true) => (lord, Some(StrongerRule::Basic)),
            _ => self.stronger_planet_by(lord, co_lord),
        }
    }

    /// The stronger of two grahas in `d`'s chart by §15.5.1's rules 1, 2, 3, 4 and 5b, without its basic rule for co-lords
    /// (ch. 15 §15.5.1, ADR 0012).
    pub(crate) fn stronger_planet(&self, a: Body, b: Body) -> Body {
        self.stronger_planet_by(a, b).0
    }

    /// [`VargaChakra::stronger_planet`] with the rule that decided, None on a tie through rule 5b.
    pub(crate) fn stronger_planet_by(&self, a: Body, b: Body) -> (Body, Option<StrongerRule>) {
        // Compared in order, so a rule is reached only on a tie in those before it.
        let strength = |graha: Body| {
            let at = self.rasi_of(graha);
            [
                GRAHAS
                    .into_iter()
                    .filter(|&g| g != graha && self.rasi_of(g) == at)
                    .count() as u32,
                // No graha joins itself, as Jupiter, Mercury or its own dispositor (ADR 0012).
                self.roles_on(at, lords(at), Some(graha)) as u32,
                exalted(graha, at).into(),
                // Mobility runs from movable to dual, weakest to strongest.
                at.mobility() as u32,
                advancement(self.chart, graha),
            ]
        };
        let (sa, sb) = (strength(a), strength(b));
        // shortcut: rustc 1.99.1 and 1.100.0 at opt-level 3 miscompile zip().position() here, giving 5b for every rule; return to it once a release fixes that.
        let rule = (0..sa.len()).find(|&i| sa[i] != sb[i]);
        // shortcut: the book has no rule for a tie after 5b, so `a`, the seven-planet lord for co-lords; upgrade if a later chapter gives one.
        let graha = if sb > sa { b } else { a };
        (graha, rule.map(|i| STRONGER_RULES[i]))
    }

    /// The stronger of two rasis in `d`'s chart, by §15.5.2's rules 1 to 6 in order, which also run for a graha's own two rasis
    /// (ch. 15 §15.5.2, ADR 0012). Not for ayur dasas, where §15.5.2's warning drops the lord's aspect: chart 44's Aq beats Le (p. 285), but this gives Le.
    pub fn stronger_rasi(&self, a: Rasi, b: Rasi) -> Rasi {
        // Compared in order, so a rule is reached only on a tie in those before it.
        let strength = |r: Rasi| {
            let occupants = || GRAHAS.into_iter().filter(|&g| self.rasi_of(g) == r);
            let stronger = self.stronger_lord(r);
            (
                occupants().count(),
                // Both co-lords, not the stronger lord alone (Exercise 26(5), ADR 0012).
                self.roles_on(r, lords(r), None),
                occupants().any(|g| exalted(g, r)),
                self.rasi_of(stronger).parity() != r.parity(),
                // Mobility runs from movable to dual, weakest to strongest.
                r.mobility() as u8,
                advancement(self.chart, stronger),
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
}

#[cfg(test)]
mod tests;

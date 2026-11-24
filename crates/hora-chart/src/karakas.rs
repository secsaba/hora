use crate::*;

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

#[cfg(test)]
mod tests;

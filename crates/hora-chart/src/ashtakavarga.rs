use crate::*;

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

#[cfg(test)]
mod tests;

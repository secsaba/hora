use crate::*;

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

#[cfg(test)]
mod tests;

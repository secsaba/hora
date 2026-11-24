use crate::*;

/// Whether rasi `a` aspects rasi `b`, which is then also so the other way: movable and fixed rasis aspect each other
/// except the adjacent ones, dual rasis the other dual rasis (ch. 10 §10.3).
pub fn rasi_drishti(a: Rasi, b: Rasi) -> bool {
    use Mobility::*;
    match (a.mobility(), b.mobility()) {
        (Chara, Sthira) | (Sthira, Chara) => !matches!(bhava(a, b), 2 | 12),
        (Dwiswabhava, Dwiswabhava) => a != b,
        _ => false,
    }
}

/// The rasis `graha` in `at` aspects with graha drishti, in count order from `at` (ch. 10 §10.2). The nodes and a lagna have none,
/// as in Exercise 14 (ADR 0013).
pub fn graha_drishti(graha: Body, at: Rasi) -> Vec<Rasi> {
    let houses: &[usize] = match graha {
        Body::Sun | Body::Moon | Body::Mercury | Body::Venus => &[7],
        Body::Mars => &[4, 7, 8],
        Body::Jupiter => &[5, 7, 9],
        Body::Saturn => &[3, 7, 10],
        Body::Rahu | Body::Ketu | Body::Lagna => &[],
    };
    houses.iter().map(|&k| kth(at, k)).collect()
}

/// The rasis `at` aspects with rasi drishti, in count order from `at` (ch. 10 §10.3).
pub fn rasi_drishtis(at: Rasi) -> Vec<Rasi> {
    (2..=12)
        .map(|k| kth(at, k))
        .filter(|&r| rasi_drishti(at, r))
        .collect()
}

impl VargaChakra<'_> {
    /// The nine grahas in `rasi` in `d`'s chart, in the book's order; a lagna is never aspected as a graha (ADR 0013).
    pub fn grahas_in(&self, rasi: Rasi) -> Vec<Body> {
        GRAHAS
            .into_iter()
            .filter(|&g| self.rasi_of(g) == rasi)
            .collect()
    }
}

/// One argala on a rasi and what obstructs it. An empty `virodhargala` is the "unobstructed" the book speaks of; there is
/// no verdict (ADR 0013).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argala {
    /// The count the argala comes from: 2, 4, 5 or 11.
    pub from: u8,
    /// The count its obstruction comes from: 12, 10, 9 or 3.
    pub obstructed_from: u8,
    pub argala: Vec<Body>,
    pub virodhargala: Vec<Body>,
}

impl VargaChakra<'_> {
    /// Whether `rasi`'s argala counts run anti-zodiacally in `d`'s chart: it holds Ketu there (ch. 10 §10.6's note).
    pub fn argala_reversed(&self, rasi: Rasi) -> bool {
        self.rasi_of(Body::Ketu) == rasi
    }

    /// The argalas on `rasi` in `d`'s chart, in the order 2nd/12th, 4th/10th, 5th/9th, 11th/3rd (ch. 10 §10.5–10.6), counted as
    /// [`VargaChakra::argala_reversed`] says. Several malefics in the 3rd stay virodhargala (ADR 0013).
    pub fn argala(&self, rasi: Rasi) -> [Argala; 4] {
        let reversed = self.argala_reversed(rasi);
        let held = |count: u8| {
            // The k-th counted in reverse is the (14 − k)th counted forward.
            let count = if reversed { 14 - count } else { count };
            self.grahas_in(kth(rasi, count.into()))
        };
        [(2, 12), (4, 10), (5, 9), (11, 3)].map(|(from, obstructed_from)| Argala {
            from,
            obstructed_from,
            argala: held(from),
            virodhargala: held(obstructed_from),
        })
    }
}

#[cfg(test)]
mod tests;

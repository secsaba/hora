use crate::*;

/// The twenty varga chakras of ch. 6 §6.2, in its order; named in the glossary's spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Varga {
    RasiChart = 1,
    Hora = 2,
    Drekkana = 3,
    Chaturthamsa = 4,
    Panchamsa = 5,
    Shashthamsa = 6,
    Saptamsa = 7,
    Ashtamsa = 8,
    Navamsa = 9,
    Dasamsa = 10,
    Rudramsa = 11,
    Dwadasamsa = 12,
    Shodasamsa = 16,
    Vimsamsa = 20,
    Chaturvimsamsa = 24,
    Nakshatramsa = 27,
    Trimsamsa = 30,
    Khavedamsa = 40,
    Akshavedamsa = 45,
    Shashtyamsa = 60,
}

/// The twenty varga chakras in the book's order, D-1 to D-60 (ch. 6 §6.2).
pub const VARGAS: [Varga; 20] = [
    Varga::RasiChart,
    Varga::Hora,
    Varga::Drekkana,
    Varga::Chaturthamsa,
    Varga::Panchamsa,
    Varga::Shashthamsa,
    Varga::Saptamsa,
    Varga::Ashtamsa,
    Varga::Navamsa,
    Varga::Dasamsa,
    Varga::Rudramsa,
    Varga::Dwadasamsa,
    Varga::Shodasamsa,
    Varga::Vimsamsa,
    Varga::Chaturvimsamsa,
    Varga::Nakshatramsa,
    Varga::Trimsamsa,
    Varga::Khavedamsa,
    Varga::Akshavedamsa,
    Varga::Shashtyamsa,
];

impl Varga {
    /// The n of D-n: the parts each rasi is divided into (ch. 6 §6.1).
    pub fn parts(self) -> u8 {
        self as u8
    }
}

/// The kth rasi from `from`, counted zodiacally: the 13th is the 1st again (§6.2.13).
pub(crate) fn kth(from: Rasi, k: usize) -> Rasi {
    RASIS[(from as usize + k - 1) % 12]
}

/// The rasi `longitude` falls in in `varga`, by the part of its rasi it is in (ch. 6 §6.2, ADR 0009).
pub fn varga(varga: Varga, longitude: f64) -> Rasi {
    use Rasi::*;
    use Varga::*;
    let (r, degrees) = rasi(longitude);
    // Whole arcseconds, as for dignities: 9°59′59.9″ is still the 1st drekkana, 10°00′00″ the 2nd.
    let at = arcseconds(degrees);
    let part = (at * u32::from(varga.parts()) / (30 * 3600)) as usize;
    let odd = r.parity() == Parity::Vishama;
    let by_parity = |odd_rasi, even_rasi| if odd { odd_rasi } else { even_rasi };
    let by_mobility = |[chara, sthira, dwiswabhava]: [Rasi; 3]| match r.mobility() {
        Mobility::Chara => chara,
        Mobility::Sthira => sthira,
        Mobility::Dwiswabhava => dwiswabhava,
    };
    let by_element = |[agni, bhoo, vaayu, jala]: [Rasi; 4]| match r.element() {
        Element::Agni => agni,
        Element::Bhoo => bhoo,
        Element::Vaayu => vaayu,
        // Aakaasa is no rasi's element of its own (ch. 2 §2.2.5).
        Element::Jala | Element::Aakaasa => jala,
    };
    let (start, step) = match varga {
        RasiChart | Dwadasamsa | Shashtyamsa => (r, 1),
        // The Sun's hora is Leo, the Moon's Cancer (Example 27, ADR 0003).
        Hora => return if odd { [Leo, Cancer] } else { [Cancer, Leo] }[part],
        // The 1st, 5th and 9th from the rasi.
        Drekkana => (r, 4),
        // The 1st, 4th, 7th and 10th from the rasi.
        Chaturthamsa => (r, 3),
        Panchamsa => {
            return if odd {
                [Aries, Aquarius, Sagittarius, Gemini, Libra]
            } else {
                [Taurus, Virgo, Pisces, Capricorn, Scorpio]
            }[part];
        }
        Shashthamsa | Khavedamsa => (by_parity(Aries, Libra), 1),
        Saptamsa => (by_parity(r, kth(r, 7)), 1),
        Ashtamsa | Vimsamsa => (by_mobility([Aries, Sagittarius, Leo]), 1),
        Navamsa => (by_element([Aries, Capricorn, Libra, Cancer]), 1),
        Dasamsa => (by_parity(r, kth(r, 9)), 1),
        // As many rasis back from Aries as the rasi is from it (§6.2.11, footnote 11).
        Rudramsa => (RASIS[(12 - r as usize) % 12], 1),
        Shodasamsa | Akshavedamsa => (by_mobility([Aries, Leo, Sagittarius]), 1),
        Chaturvimsamsa => (by_parity(Leo, Cancer), 1),
        Nakshatramsa => (by_element([Aries, Cancer, Libra, Capricorn]), 1),
        // Unequal parts: each rasi up to the degree mark it is below (§6.2.17).
        Trimsamsa => {
            let marks = if odd {
                [
                    (5, Aries),
                    (10, Aquarius),
                    (18, Sagittarius),
                    (25, Gemini),
                    (30, Libra),
                ]
            } else {
                [
                    (5, Taurus),
                    (12, Virgo),
                    (20, Pisces),
                    (25, Capricorn),
                    (30, Scorpio),
                ]
            };
            // The last mark is 30°, beyond every `at`.
            return marks
                .into_iter()
                .find(|&(mark, _)| at < mark * 3600)
                .unwrap()
                .1;
        }
    };
    kth(start, part * step + 1)
}

/// The four groups of varga chakras amsabala is counted over (ch. 6 §6.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VargaGroup {
    Shadvarga,
    SaptaVarga,
    DasaVarga,
    ShodasaVarga,
}

/// The varga groups in the book's order (ch. 6 §6.6.1–6.6.4).
pub const VARGA_GROUPS: [VargaGroup; 4] = [
    VargaGroup::Shadvarga,
    VargaGroup::SaptaVarga,
    VargaGroup::DasaVarga,
    VargaGroup::ShodasaVarga,
];

/// The amsas of §6.6, named for a graha's count of good varga chakras in a group, in the book's spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Amsa {
    Kimsukaamsa,
    Vyanjanaamsa,
    Chaamaraamsa,
    Chatraamsa,
    Kundalaamsa,
    Mukutaamsa,
    Paarijaataamsa,
    Uttamaamsa,
    Gopuraamsa,
    Simhaasanaamsa,
    Paaraavataamsa,
    Devalokaamsa,
    Brahmalokamsa,
    Airaavataamsa,
    Sreedhaamaamsa,
    Bhedakaamsa,
    Kusumaamsa,
    Nagapurushaamsa,
    Kandukaamsa,
    Keralaamsa,
    Kalpavrikshaamsa,
    Chandanavanaamsa,
    Poornachandraamsa,
    Uchchaisravaamsa,
    Dhanvantaryamsa,
    Sooryakaantaamsa,
    Vidrumaamsa,
    Indraasanaamsa,
    Golokaamsa,
    SreeVallabhaamsa,
}

impl VargaGroup {
    /// The group's varga chakras (§6.6.1–6.6.4).
    pub fn vargas(self) -> &'static [Varga] {
        use Varga::*;
        match self {
            VargaGroup::Shadvarga => &[RasiChart, Hora, Drekkana, Navamsa, Dwadasamsa, Trimsamsa],
            VargaGroup::SaptaVarga => &[
                RasiChart, Hora, Drekkana, Saptamsa, Navamsa, Dwadasamsa, Trimsamsa,
            ],
            VargaGroup::DasaVarga => &[
                RasiChart,
                Hora,
                Drekkana,
                Saptamsa,
                Navamsa,
                Dasamsa,
                Dwadasamsa,
                Shodasamsa,
                Trimsamsa,
                Shashtyamsa,
            ],
            VargaGroup::ShodasaVarga => &[
                RasiChart,
                Hora,
                Drekkana,
                Chaturthamsa,
                Saptamsa,
                Navamsa,
                Dasamsa,
                Dwadasamsa,
                Shodasamsa,
                Vimsamsa,
                Chaturvimsamsa,
                Nakshatramsa,
                Trimsamsa,
                Khavedamsa,
                Akshavedamsa,
                Shashtyamsa,
            ],
        }
    }

    /// The group's amsas, for a count of 2 up to all its vargas (§6.6.1–6.6.4).
    pub fn amsas(self) -> &'static [Amsa] {
        use Amsa::*;
        const SAPTA: [Amsa; 6] = [
            Kimsukaamsa,
            Vyanjanaamsa,
            Chaamaraamsa,
            Chatraamsa,
            Kundalaamsa,
            Mukutaamsa,
        ];
        match self {
            // Shadvarga's six vargas stop one short of sapta varga's mukutaamsa.
            VargaGroup::Shadvarga => &SAPTA[..5],
            VargaGroup::SaptaVarga => &SAPTA,
            VargaGroup::DasaVarga => &[
                Paarijaataamsa,
                Uttamaamsa,
                Gopuraamsa,
                Simhaasanaamsa,
                Paaraavataamsa,
                Devalokaamsa,
                Brahmalokamsa,
                Airaavataamsa,
                Sreedhaamaamsa,
            ],
            VargaGroup::ShodasaVarga => &[
                Bhedakaamsa,
                Kusumaamsa,
                Nagapurushaamsa,
                Kandukaamsa,
                Keralaamsa,
                Kalpavrikshaamsa,
                Chandanavanaamsa,
                Poornachandraamsa,
                Uchchaisravaamsa,
                Dhanvantaryamsa,
                Sooryakaantaamsa,
                Vidrumaamsa,
                Indraasanaamsa,
                Golokaamsa,
                SreeVallabhaamsa,
            ],
        }
    }

    /// The amsa named for `count` good vargas in the group; none below 2.
    pub fn amsa(self, count: u8) -> Option<Amsa> {
        self.amsas()
            .get(usize::from(count.checked_sub(2)?))
            .copied()
    }
}

/// How many of `group`'s vargas put `graha` in its uchcha, moolatrikona or own rasi, and the amsa named for it
/// (ch. 6 §6.6). By whole rasi: Example 27 counts Jupiter at 29°49′ Sg, past §3.3's moolatrikona degrees. None for
/// the lagna.
pub fn amsabala(graha: Body, longitude: f64, group: VargaGroup) -> (u8, Option<Amsa>) {
    let Some(d) = graha.dignities() else {
        return (0, None);
    };
    let count = group
        .vargas()
        .iter()
        .map(|&v| varga(v, longitude))
        .filter(|r| *r == d.uchcha.0 || *r == d.moolatrikona.0 || d.own.contains(r))
        .count() as u8;
    (count, group.amsa(count))
}

#[cfg(test)]
mod tests;

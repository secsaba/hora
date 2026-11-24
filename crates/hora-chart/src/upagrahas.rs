use crate::*;

/// Ch. 4 §4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Upagraha {
    Dhuma,
    Vyatipaata,
    Parivesha,
    Indrachaapa,
    Upaketu,
    Kaala,
    Mrityu,
    Arthaprahaara,
    Yamaghantaka,
    Gulika,
    Maandi,
}

/// The eleven upagrahas in the book's order: five Sun-based, then six time-based (ch. 4 §4.2–4.3).
pub const UPAGRAHAS: [Upagraha; 11] = [
    Upagraha::Dhuma,
    Upagraha::Vyatipaata,
    Upagraha::Parivesha,
    Upagraha::Indrachaapa,
    Upagraha::Upaketu,
    Upagraha::Kaala,
    Upagraha::Mrityu,
    Upagraha::Arthaprahaara,
    Upagraha::Yamaghantaka,
    Upagraha::Gulika,
    Upagraha::Maandi,
];

/// What the book says of an upagraha's nature, no more: only Kaala and Mrityu are called malefic among the six (ch. 4 §4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpagrahaNature {
    VeryMalefic,
    MaleficLikeSun,
    MaleficLikeMars,
    LikeMercury,
    LikeJupiter,
    LikeSaturn,
}

impl Upagraha {
    /// Ch. 4 §4.2–4.3.
    pub fn nature(self) -> UpagrahaNature {
        use Upagraha::*;
        match self {
            Dhuma | Vyatipaata | Parivesha | Indrachaapa | Upaketu => UpagrahaNature::VeryMalefic,
            Kaala => UpagrahaNature::MaleficLikeSun,
            Mrityu => UpagrahaNature::MaleficLikeMars,
            Arthaprahaara => UpagrahaNature::LikeMercury,
            Yamaghantaka => UpagrahaNature::LikeJupiter,
            Gulika | Maandi => UpagrahaNature::LikeSaturn,
        }
    }

    /// Whether it is one of the five found from the Sun's longitude, not from a part of the day or night (ch. 4 §4.2).
    pub fn is_sun_based(self) -> bool {
        use Upagraha::*;
        matches!(self, Dhuma | Vyatipaata | Parivesha | Indrachaapa | Upaketu)
    }
}

/// The five Sun-based upagrahas from the Sun's `sun` longitude, each reduced to [0°, 360°) (ch. 4 §4.2, Table 9, footnote 8).
pub fn sun_based(sun: f64) -> [(Upagraha, f64); 5] {
    let dhuma = reduce(sun + 133.0 + 20.0 / 60.0);
    let vyatipaata = reduce(360.0 - dhuma);
    let parivesha = reduce(vyatipaata + 180.0);
    let indrachaapa = reduce(360.0 - parivesha);
    let upaketu = reduce(indrachaapa + 16.0 + 40.0 / 60.0);
    use Upagraha::*;
    [
        (Dhuma, dhuma),
        (Vyatipaata, vyatipaata),
        (Parivesha, parivesha),
        (Indrachaapa, indrachaapa),
        (Upaketu, upaketu),
    ]
}

/// The graha ruling each eighth of `vaara`'s day, or its `night`, `None` for the lord-less part: by day the vaara's graha
/// first, by night the 5th from it, then on in weekday order with a lord-less part after Saturn's (ch. 4 §4.3, Table 10).
pub fn part_rulers(vaara: Vaara, night: bool) -> [Option<Body>; 8] {
    use Body::*;
    const ORDER: [Option<Body>; 8] = [
        Some(Sun),
        Some(Moon),
        Some(Mars),
        Some(Mercury),
        Some(Jupiter),
        Some(Venus),
        Some(Saturn),
        None,
    ];
    // The 5th graha is counted among the seven, without the lord-less part.
    let first = (vaara as usize + if night { 4 } else { 0 }) % 7;
    std::array::from_fn(|k| ORDER[(first + k) % 8])
}

/// The upagrahas of `sky` with their longitudes, in the book's order: the five Sun-based for every chart, then the six
/// time-based where the Sky has a Vedic day, Gulika at the start of Saturn's part and the rest at a middle (ch. 4, ADR 0003).
pub fn upagrahas(sky: &Sky) -> Vec<(Upagraha, f64)> {
    let mut upagrahas = sun_based(sky.grahas.sun.longitude).to_vec();
    if let (Some(day), Some(lagnas)) = (&sky.vedic_day, &sky.part_lagnas) {
        let rulers = part_rulers(vaara(day), day.is_night(sky.jd));
        // Each graha rules one of the eight parts.
        let lagna = |graha, middle: bool| {
            lagnas[rulers.iter().position(|&r| r == Some(graha)).unwrap()][usize::from(middle)]
        };
        use Upagraha::*;
        upagrahas.extend([
            (Kaala, lagna(Body::Sun, true)),
            (Mrityu, lagna(Body::Mars, true)),
            (Arthaprahaara, lagna(Body::Mercury, true)),
            (Yamaghantaka, lagna(Body::Jupiter, true)),
            (Gulika, lagna(Body::Saturn, false)),
            (Maandi, lagna(Body::Saturn, true)),
        ]);
    }
    upagrahas
}

#[cfg(test)]
mod tests;

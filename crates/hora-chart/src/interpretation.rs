use crate::*;

/// A planet's functional nature from a lagna (ch. 13 §13.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionalNature {
    Benefic,
    Neutral,
    Malefic,
}

/// Table 30 (ch. 13 §13.2, p. 167) as printed, in rasi order: the yogakaraka, then the functional benefics, neutrals and malefics.
/// Not derived from §13.2's rules, which five cells break, as Parasara does (ADR 0016).
const TABLE_30: [(Option<Body>, [&[Body]; 3]); 12] = {
    use Body::*;
    [
        (
            None,
            [&[Sun, Mars, Jupiter], &[], &[Mercury, Venus, Saturn]],
        ),
        (
            Some(Saturn),
            [&[Sun, Mercury, Saturn], &[Mars], &[Moon, Jupiter, Venus]],
        ),
        (
            None,
            [&[Venus], &[Moon, Mercury, Saturn], &[Sun, Mars, Jupiter]],
        ),
        (
            Some(Mars),
            [&[Moon, Mars, Jupiter], &[Sun, Saturn], &[Mercury, Venus]],
        ),
        (
            Some(Mars),
            [&[Sun, Mars, Jupiter], &[Moon], &[Mercury, Venus, Saturn]],
        ),
        (
            None,
            [&[Mercury, Venus], &[Sun, Saturn], &[Moon, Mars, Jupiter]],
        ),
        (
            Some(Saturn),
            [&[Mercury, Venus, Saturn], &[], &[Sun, Mars, Jupiter]],
        ),
        (
            None,
            [&[Moon, Jupiter], &[Sun, Mars], &[Mercury, Venus, Saturn]],
        ),
        (
            None,
            [&[Sun, Mars], &[Moon, Mercury, Jupiter], &[Venus, Saturn]],
        ),
        (
            Some(Venus),
            [&[Venus, Mercury, Saturn], &[Sun], &[Mars, Jupiter]],
        ),
        (
            Some(Venus),
            [&[Venus, Saturn], &[Sun, Mercury], &[Moon, Mars, Jupiter]],
        ),
        (
            None,
            [&[Moon, Mars], &[Jupiter], &[Sun, Mercury, Venus, Saturn]],
        ),
    ]
};

impl VargaChakra<'_> {
    /// `graha`'s functional nature by Table 30, from `from`'s rasi in `d`'s chart ("yogakaraka from Moon", p. 221; "a functional
    /// benefic in dasamsa", p. 219). The Moon the table leaves out is a malefic in Sukla paksha, neutral in Krishna (§13.2,
    /// pp. 166–167). `None` for the nodes, a lagna, or a reference point with no rasi (ADR 0016).
    pub fn functional_nature(&self, graha: Body, from: ReferencePoint) -> Option<FunctionalNature> {
        use FunctionalNature::*;
        let (_, columns) = TABLE_30[self.reference(from)? as usize];
        let column = columns.iter().position(|c| c.contains(&graha));
        match column {
            Some(i) => Some([Benefic, Neutral, Malefic][i]),
            None if graha == Body::Moon => match self.natural_nature(Body::Moon)? {
                Disposition::Saumya => Some(Malefic),
                _ => Some(Neutral),
            },
            None => None,
        }
    }

    /// Table 30's yogakaraka from `from`'s rasi in `d`'s chart, if it has one (ch. 13 §13.2, p. 167, ADR 0016).
    pub fn yogakaraka(&self, from: ReferencePoint) -> Option<Body> {
        TABLE_30[self.reference(from)? as usize].0
    }
}

/// What a yogada gives: money, by HL, or power and authority, by GL (ch. 13 §13.2, pp. 167–168).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gives {
    Money,
    Power,
}

/// How a yogada reaches a rasi: owns it as its seven-planet lord, is in it, or has rasi drishti on it (ADR 0016).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Link {
    Owns,
    Conjoins,
    Aspects,
}

/// A planet giving yoga in money or power (ch. 13 §13.2, ADR 0016).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yogada {
    pub graha: Body,
    pub gives: Gives,
    /// Every link it has to the lagna, then to HL or GL, each in [`Link`]'s order.
    pub links: [Vec<Link>; 2],
}

impl VargaChakra<'_> {
    /// The yogadas of `d`'s chart from its own lagna, HL and GL, whatever the reference point: each of the seven planets that owns,
    /// conjoins or has rasi drishti on both the lagna and HL (money), or the lagna and GL (power), money first, then in graha
    /// order. "Aspects" is rasi drishti, as every worked example reads it (pp. 230, 495, 496); none without a Vedic day (ADR 0016).
    pub fn yogadas(&self) -> Vec<Yogada> {
        let lagna = self.rasi_of(Body::Lagna);
        let mut found = Vec::new();
        for (gives, point) in [
            (Gives::Money, SpecialLagna::HoraLagna),
            (Gives::Power, SpecialLagna::GhatiLagna),
        ] {
            let Some(point) = self.reference(ReferencePoint::Special(point)) else {
                continue;
            };
            for graha in PLANETS {
                let at = self.rasi_of(graha);
                let links = [lagna, point].map(|r| {
                    [
                        (Link::Owns, r.lord() == graha),
                        (Link::Conjoins, at == r),
                        (Link::Aspects, rasi_drishti(at, r)),
                    ]
                    .into_iter()
                    .filter_map(|(link, holds)| holds.then_some(link))
                    .collect::<Vec<_>>()
                });
                if links.iter().all(|l| !l.is_empty()) {
                    found.push(Yogada {
                        graha,
                        gives,
                        links,
                    });
                }
            }
        }
        found
    }
}

/// `rasi`'s baadhaka sthaana: the 11th from a chara rasi, the 9th from a sthira, the 7th from a dwiswabhava (ch. 13 §13.3,
/// Table 31, pp. 168–169). Its occupants are [`VargaChakra::grahas_in`] it.
pub fn baadhaka_sthaana(rasi: Rasi) -> Rasi {
    let k = match rasi.mobility() {
        Mobility::Chara => 11,
        Mobility::Sthira => 9,
        Mobility::Dwiswabhava => 7,
    };
    kth(rasi, k)
}

/// `rasi`'s baadhakas, its baadhaka sthaana's lords: both co-lords for Aquarius and Scorpio, as Table 31 prints them (ch. 13
/// §13.3, pp. 168–169, ADR 0016).
pub fn baadhakas(rasi: Rasi) -> Vec<Body> {
    lords(baadhaka_sthaana(rasi)).collect()
}

#[cfg(test)]
mod tests;

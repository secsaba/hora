use crate::*;

mod dhana;
mod popular;
mod raaja;

impl VargaChakra<'_> {
    /// A rasi's lords as a yoga counts them: its lord, or either co-lord of Scorpio or Aquarius, but only the other if one is in
    /// the rasi, and either if both are (Parasara as the book gives him on p. 218, ADR 0014).
    pub fn lords_in_yoga(&self, rasi: Rasi) -> Vec<Body> {
        let outside: Vec<Body> = lords(rasi).filter(|&g| self.rasi_of(g) != rasi).collect();
        if outside.is_empty() {
            lords(rasi).collect()
        } else {
            outside
        }
    }

    /// The lords a §11.6 rule tries, in turn: [`VargaChakra::lords_in_yoga`]'s, then a seven-planet co-lord at home, since "Saturn also owns
    /// 5th" (p. 218, ADR 0014). A node at home is not tried: the book says that only of Saturn.
    pub(crate) fn lords_to_try(&self, rasi: Rasi) -> Vec<Body> {
        let mut lords_tried = self.lords_in_yoga(rasi);
        for lord in lords(rasi) {
            if PLANETS.contains(&lord) && !lords_tried.contains(&lord) {
                lords_tried.push(lord);
            }
        }
        lords_tried
    }
}
/// The classes ch. 11 §11.1 divides the yogas into. `Ravi` is the class of solar yogas, not the single [`YogaName::Ravi`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YogaClass {
    Ravi,
    Chandra,
    Mahaapurusha,
    Naabhasa,
    OtherPopular,
    Raaja,
    RaajaSambandha,
    Dhana,
    Daridra,
}

/// Every entry of ch. 11 in the book's order, its numbered ones by class and number, and ch. 13's Tapaswi. The Aakriti yoga `Vajra` and the Sankhya
/// yoga `Soola` are not the Sun-Moon yoga's values, and `Chaapa` is not `Chapa`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YogaName {
    // §11.2
    Vesi,
    Vosi,
    Ubhayachara,
    BudhaAaditya,
    // §11.3
    Sunaphaa,
    Anaphaa,
    Duradhara,
    Kemadruma,
    ChandraMangala,
    Adhi,
    // §11.4
    Ruchaka,
    Bhadra,
    Sasa,
    Maalavya,
    Hamsa,
    // §11.5.1
    Rajju,
    Musala,
    Nala,
    // §11.5.2
    Maalaa,
    Sarpa,
    // §11.5.3
    Gadaa,
    Sakata,
    Vihanga,
    Sringaataka,
    Hala,
    Vajra,
    Yava,
    Kamala,
    Vaapi,
    Yoopa,
    Sara,
    Sakti,
    Danda,
    Naukaa,
    Koota,
    Chatra,
    Chaapa,
    Ardhachandra,
    Chakra,
    Samudra,
    // §11.5.4
    Veenaa,
    Daama,
    Paasa,
    Kedaara,
    Soola,
    Yuga,
    Gola,
    // §11.6
    Subha,
    Asubha,
    GajaKesari,
    GuruMangala,
    Amala,
    Parvata,
    Kaahala,
    Chaamara,
    Sankha,
    Bheri,
    Mridanga,
    Sreenaatha,
    Matsya,
    Koorma,
    Khadga,
    Kusuma,
    Kalaanidhi,
    Kalpadruma,
    Lagnaadhi,
    Hari,
    Hara,
    Brahma,
    Vishnu,
    Siva,
    Trilochana,
    Gouri,
    Chandikaa,
    Lakshmi,
    Saarada,
    Bhaarathi,
    Saraswathi,
    Amsaavatara,
    Devendra,
    Indra,
    Ravi,
    Bhaaskara,
    Kulavardhana,
    Vasumati,
    Gandharva,
    Go,
    Vidyut,
    Chapa,
    Pushkala,
    Makuta,
    Jaya,
    Harsha,
    Sarala,
    Vimala,
    // ch. 13 §13.5
    Tapaswi,
    // §11.7.1
    BasicRaaja,
    DharmaKarmadhipati,
    VipareetaRaaja,
    /// §11.7.3 (1) to (17); (18) only says when others work better, so is not computed (ADR 0014).
    Raaja(u8),
    /// §11.8 (1) to (15).
    RaajaSambandha(u8),
    /// §11.9's basic rule: the Moon, Mercury, Jupiter or Venus exalted in the 2nd.
    BasicDhana,
    /// §11.9 (1) to (12), one per lagna from Aries.
    Dhana(u8),
    /// §11.10 (1) to (9) and (11) to (13); (10)'s "benefic houses" are undefined, so it is not computed (ADR 0014).
    Daridra(u8),
}

impl YogaName {
    /// The class ch. 11 lists it under.
    pub fn class(self) -> YogaClass {
        use YogaClass as C;
        use YogaName::*;
        match self {
            Vesi | Vosi | Ubhayachara | BudhaAaditya => C::Ravi,
            Sunaphaa | Anaphaa | Duradhara | Kemadruma | ChandraMangala | Adhi => C::Chandra,
            Ruchaka | Bhadra | Sasa | Maalavya | Hamsa => C::Mahaapurusha,
            Rajju | Musala | Nala | Maalaa | Sarpa | Gadaa | Sakata | Vihanga | Sringaataka
            | Hala | Vajra | Yava | Kamala | Vaapi | Yoopa | Sara | Sakti | Danda | Naukaa
            | Koota | Chatra | Chaapa | Ardhachandra | Chakra | Samudra | Veenaa | Daama
            | Paasa | Kedaara | Soola | Yuga | Gola => C::Naabhasa,
            Subha | Asubha | GajaKesari | GuruMangala | Amala | Parvata | Kaahala | Chaamara
            | Sankha | Bheri | Mridanga | Sreenaatha | Matsya | Koorma | Khadga | Kusuma
            | Kalaanidhi | Kalpadruma | Lagnaadhi | Hari | Hara | Brahma | Vishnu | Siva
            | Trilochana | Gouri | Chandikaa | Lakshmi | Saarada | Bhaarathi | Saraswathi
            | Amsaavatara | Devendra | Indra | Ravi | Bhaaskara | Kulavardhana | Vasumati
            | Gandharva | Go | Vidyut | Chapa | Pushkala | Makuta | Jaya | Harsha | Sarala
            | Vimala | Tapaswi => C::OtherPopular,
            BasicRaaja | DharmaKarmadhipati | VipareetaRaaja | Raaja(_) => C::Raaja,
            RaajaSambandha(_) => C::RaajaSambandha,
            BasicDhana | Dhana(_) => C::Dhana,
            Daridra(_) => C::Daridra,
        }
    }
}

/// A condition of a yoga left to the reader, having no measure in the book or bearing only on its results (ADR 0014).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToJudge {
    /// Mercury's company ties and the yoga holds only with one of his natures.
    MercurysNature,
    /// A graha of the opposite nature is in a kendra of a Dala yoga, which "may not operate well" (§11.5.2).
    MayNotOperateWell,
    /// A rule's "strong": no threshold anywhere in the book.
    Strong(Body),
    /// A stated "not combust": this graha shares the Sun's rasi in the rasi chart, and the book gives no number.
    Combust(Body),
    /// A stated "deep exaltation": this graha is in his uchcha rasi, and how near the degree must be has no number (ADR 0014).
    DeepExaltation(Body),
    /// Vasumati's "full results" want no malefic in the upachayas (§11.6), and this one is there.
    MaleficInUpachaya(Body),
    /// All three of a Tapaswi yoga are in one rasi, where the book asks for the third to aspect the two (ch. 13 §13.5, ADR 0016).
    AllInOneRasi,
}

/// A yoga present in a chart, or to judge if `to_judge` holds anything; `name.class()` gives its class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yoga {
    pub name: YogaName,
    /// The grahas forming it: both of a conjunction, only those placed from the Sun or Moon, all seven of a shape or count,
    /// only those of the stated nature in the kendras for Maalaa, Sarpa, Vajra and Yava, none for Kemadruma, the four in the
    /// book's order, repeats included, for Kalpadruma, and otherwise every graha a rule names, in graha order.
    pub grahas: Vec<YogaGraha>,
    pub to_judge: Vec<ToJudge>,
    /// Which of the book's forms held, 1 or 2, for a yoga with an "alternately" form or Brahma's second; a present form before a
    /// to-judge one, then the book's order. Raaja (1)'s and a Dhana row's are 1 for the first half or combination, 2 for the
    /// second, 3 for both; Raaja Sambandha (11)'s likewise for A9 in the lagna and AK in the 9th.
    pub form: Option<u8>,
    /// §11.7.2's measures, for a basic Raaja, Dharma-Karmadhipati or conjoined Vipareeta pair in the rasi chart.
    pub magnitude: Option<Magnitude>,
    /// §11.7.2 (1)'s functional malefics afflicting such a pair, in any chart; empty for every other row (ADR 0016).
    pub afflictions: Vec<Affliction>,
    /// What the book says strengthens the yoga, never a condition; only the Tapaswi yoga has any (ADR 0016).
    pub notes: Vec<Note>,
}

/// A remark the book adds to a yoga present, not a condition of it (ch. 13 §13.5, ADR 0016).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    /// This AK, from the rasi chart, conjoins or aspects by graha drishti a graha of the yoga other than himself: "more fruitful".
    AtmaKaraka(Body),
    /// This graha of the yoga is in the 8th from the lagna: "particularly powerful".
    InTheEighth(Body),
}

/// A functional malefic from the lagna conjoining or aspecting, by graha drishti, a graha of a Raaja pair (ch. 11 §11.7.2 (1),
/// ch. 13 Table 30, ADR 0016). Not rated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Affliction {
    pub graha: Body,
    pub afflicts: Body,
    /// As [`Magnitude::closeness`] measures it, in the rasi chart only.
    pub closeness: Option<u32>,
}

/// A graha of a yoga row with the roles the row names it by (ch. 11, ADR 0014).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YogaGraha {
    pub graha: Body,
    /// Its bhava from `d`'s lagna, never the reference point.
    pub bhava: u8,
    /// The bhavas the row names it lord of, from that lagna, in order: every one [`VargaChakra::lords_in_yoga`] counts, or for
    /// §11.6 [`VargaChakra::lords_to_try`], but for Raaja (1) only the 1st and 5th, and none under form 1 (§11.7.3).
    pub named_lord_of: Vec<u8>,
    /// AK or PK, for Raaja (1) under form 1 or 3 only (§11.7.3); from the rasi chart in every varga.
    pub karaka: Option<CharaKaraka>,
}

/// How exact a Raaja yoga's pair is and how good their amsas, with no rating (ch. 11 §11.7.2, ADR 0014).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Magnitude {
    /// The difference of the two grahas' degrees within their rasis in whole arcseconds, the nodes' as they stand, for any
    /// association.
    pub closeness: u32,
    /// Each graha of the pair, in the row's order, with its Dasa Varga count and amsa (ch. 6 §6.6).
    pub amsas: [(Body, u8, Option<Amsa>); 2],
}

impl VargaChakra<'_> {
    /// Whether `g` aspects rasi `r` with graha drishti from his own rasi in `d`'s chart (ch. 10 §10.2).
    fn aspects(&self, g: Body, r: Rasi) -> bool {
        graha_drishti(g, self.rasi_of(g)).contains(&r)
    }

    /// Whether `g` occupies rasi `r` or aspects it with graha drishti in `d`'s chart.
    pub(crate) fn joins_or_aspects(&self, g: Body, r: Rasi) -> bool {
        self.rasi_of(g) == r || self.aspects(g, r)
    }

    /// Whether two distinct grahas are associated in `d`'s chart: conjoined, in mutual graha drishti, or each in a rasi the
    /// other owns (parivartana) (ch. 11 §11.7.1). The nodes have no graha drishti (ADR 0013).
    pub fn associated(&self, a: Body, b: Body) -> bool {
        let (ra, rb) = (self.rasi_of(a), self.rasi_of(b));
        a != b
            && (ra == rb
                || self.aspects(a, rb) && self.aspects(b, ra)
                || owns(b, ra) && owns(a, rb))
    }
}

/// The Sankhya yoga by how many rasis the seven planets occupy (§11.5.4); [`VargaChakra::yogas`] adds the proviso, so never lists `Gola`.
pub fn sankhya_yoga(planets: [Rasi; 7]) -> YogaName {
    use YogaName::*;
    let occupied = RASIS.iter().filter(|r| planets.contains(r)).count();
    [Gola, Yuga, Soola, Kedaara, Paasa, Daama, Veenaa][occupied - 1]
}

impl VargaChakra<'_> {
    /// The yogas of `d`'s chart, counted from that chart's lagna, in the book's order; absent ones are not listed (ADR 0014).
    /// Raaja (6) and (8) need the chart's HL and GL.
    pub fn yogas(&self) -> Vec<Yoga> {
        use Body::*;
        use YogaName as N;

        let yoga = |name, grahas: Vec<Body>| Yoga {
            name,
            grahas: self.yoga_grahas(name, &grahas),
            to_judge: Vec::new(),
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        // Mars to Saturn: the Sun's and Moon's 2nd and 12th leave out the other light and the nodes (ADR 0014).
        let others_in = |rasi| -> Vec<Body> {
            PLANETS[2..]
                .iter()
                .copied()
                .filter(|&g| self.rasi_of(g) == rasi)
                .collect()
        };
        // Both sides are the third yoga alone, not the two single ones as well.
        let flanking = |from, [second, twelfth, both]: [YogaName; 3]| {
            let (s, t) = (others_in(kth(from, 2)), others_in(kth(from, 12)));
            match (s.is_empty(), t.is_empty()) {
                (true, true) => None,
                (false, true) => Some(yoga(second, s)),
                (true, false) => Some(yoga(twelfth, t)),
                (false, false) => Some(yoga(both, [s, t].concat())),
            }
        };

        let mut found = Vec::new();

        // §11.2.1–11.2.4. Budha-Aaditya's combustion is a note on its power (ADR 0014).
        let sun = self.rasi_of(Sun);
        found.extend(flanking(sun, [N::Vesi, N::Vosi, N::Ubhayachara]));
        if self.rasi_of(Mercury) == sun {
            found.push(yoga(N::BudhaAaditya, vec![Sun, Mercury]));
        }

        // §11.3.1–11.3.6.
        let moon = self.rasi_of(Moon);
        let lagna = self.rasi_of(Lagna);
        let kendra = |rasi| matches!(bhava(lagna, rasi), 1 | 4 | 7 | 10);
        found.extend(flanking(moon, [N::Sunaphaa, N::Anaphaa, N::Duradhara]));
        let beside_moon = [1, 2, 12]
            .iter()
            .any(|&k| !others_in(kth(moon, k)).is_empty());
        let in_kendra = PLANETS
            .iter()
            .any(|&g| g != Moon && kendra(self.rasi_of(g)));
        if !beside_moon && !in_kendra {
            found.push(yoga(N::Kemadruma, Vec::new()));
        }
        if self.rasi_of(Mars) == moon {
            found.push(yoga(N::ChandraMangala, vec![Moon, Mars]));
        }
        // Two or more natural benefics in the places, in any arrangement; a kroora Mercury is not one, and the Moon is counted from (ADR 0014).
        let places = [6, 7, 8].map(|k| kth(moon, k));
        found.extend(self.with_mercury_each_way(N::Adhi, |mercury| {
            let held: Vec<Body> = [Mercury, Jupiter, Venus]
                .into_iter()
                .filter(|&g| g != Mercury || mercury == Disposition::Saumya)
                .filter(|&g| places.contains(&self.rasi_of(g)))
                .collect();
            (held.len() >= 2).then_some(held)
        }));

        // §11.4.1–11.4.5, by whole rasi. Hamsa's "Ruchaka" is a misprint (ADR 0014).
        for (name, graha) in [
            (N::Ruchaka, Mars),
            (N::Bhadra, Mercury),
            (N::Sasa, Saturn),
            (N::Maalavya, Venus),
            (N::Hamsa, Jupiter),
        ] {
            let rasi = self.rasi_of(graha);
            if kendra(rasi) && (lords(rasi).any(|g| g == graha) || exalted(graha, rasi)) {
                found.push(yoga(name, vec![graha]));
            }
        }

        // §11.5: over the seven planets, as "all the planets" means (ADR 0014).
        let planets = PLANETS.map(|g| self.rasi_of(g));
        let houses = planets.map(|r| bhava(lagna, r));
        let within = |places: &[u8]| houses.iter().all(|h| places.contains(h));
        let forms = |places: &[u8]| within(places) && places.iter().all(|p| houses.contains(p));
        let seven_from = |k: u8| -> [u8; 7] { std::array::from_fn(|i| (k - 1 + i as u8) % 12 + 1) };
        let first_naabhasa = found.len();
        // Among the grahas only a tied Mercury has no nature.
        let of_nature =
            |g: Body, nature, mercury_as| self.natural_nature(g).unwrap_or(mercury_as) == nature;

        // §11.5.1.
        for (name, mobility) in [
            (N::Rajju, Mobility::Chara),
            (N::Musala, Mobility::Sthira),
            (N::Nala, Mobility::Dwiswabhava),
        ] {
            if planets.iter().all(|r| r.mobility() == mobility) {
                found.push(yoga(name, PLANETS.to_vec()));
            }
        }

        // §11.5.2. A tied Mercury always has both natures beside him, so his reading never decides a kendra or the note.
        for (name, forming, opposite) in [
            (N::Maalaa, Disposition::Saumya, Disposition::Kroora),
            (N::Sarpa, Disposition::Kroora, Disposition::Saumya),
        ] {
            let row = self.with_mercury_each_way(name, |mercury_as| {
                let grahas: Vec<Body> = GRAHAS
                    .into_iter()
                    .filter(|&g| kendra(self.rasi_of(g)) && of_nature(g, forming, mercury_as))
                    .collect();
                let held = [1, 4, 7, 10]
                    .iter()
                    .filter(|&&k| grahas.iter().any(|&g| bhava(lagna, self.rasi_of(g)) == k))
                    .count();
                (held >= 3).then_some(grahas)
            });
            found.extend(row.map(|mut row| {
                if GRAHAS
                    .iter()
                    .any(|&g| kendra(self.rasi_of(g)) && self.natural_nature(g) == Some(opposite))
                {
                    row.to_judge.push(ToJudge::MayNotOperateWell);
                }
                row
            }));
        }

        // §11.5.3: every place occupied, but for Kamala and Vaapi (ADR 0014).
        for (name, formed) in [
            (
                N::Gadaa,
                [[1, 4], [4, 7], [7, 10], [10, 1]].iter().any(|p| forms(p)),
            ),
            (N::Sakata, forms(&[1, 7])),
            (N::Vihanga, forms(&[4, 10])),
            (N::Sringaataka, forms(&[1, 5, 9])),
            (
                N::Hala,
                [[2, 6, 10], [3, 7, 11], [4, 8, 12]]
                    .iter()
                    .any(|p| forms(p)),
            ),
        ] {
            if formed {
                found.push(yoga(name, PLANETS.to_vec()));
            }
        }
        // Each named house holds at least one of its nature, others beside it or not (ADR 0014).
        for (name, in_1st_7th, in_4th_10th) in [
            (N::Vajra, Disposition::Saumya, Disposition::Kroora),
            (N::Yava, Disposition::Kroora, Disposition::Saumya),
        ] {
            found.extend(self.with_mercury_each_way(name, |mercury_as| {
                let grahas: Vec<Body> = GRAHAS
                    .into_iter()
                    .filter(|&g| match bhava(lagna, self.rasi_of(g)) {
                        1 | 7 => of_nature(g, in_1st_7th, mercury_as),
                        4 | 10 => of_nature(g, in_4th_10th, mercury_as),
                        _ => false,
                    })
                    .collect();
                [1, 4, 7, 10]
                    .iter()
                    .all(|&k| grahas.iter().any(|&g| bhava(lagna, self.rasi_of(g)) == k))
                    .then_some(grahas)
            }));
        }
        for (name, formed) in [
            (N::Kamala, within(&[1, 4, 7, 10])),
            (N::Vaapi, within(&[2, 5, 8, 11]) || within(&[3, 6, 9, 12])),
            (N::Yoopa, forms(&[1, 2, 3, 4])),
            (N::Sara, forms(&[4, 5, 6, 7])),
            (N::Sakti, forms(&[7, 8, 9, 10])),
            (N::Danda, forms(&[10, 11, 12, 1])),
            (N::Naukaa, forms(&seven_from(1))),
            (N::Koota, forms(&seven_from(4))),
            (N::Chatra, forms(&seven_from(7))),
            (N::Chaapa, forms(&seven_from(10))),
            (
                N::Ardhachandra,
                [2, 5, 8, 11, 3, 6, 9, 12]
                    .iter()
                    .any(|&k| forms(&seven_from(k))),
            ),
            (N::Chakra, forms(&[1, 3, 5, 7, 9, 11])),
            (N::Samudra, forms(&[2, 4, 6, 8, 10, 12])),
        ] {
            if formed {
                found.push(yoga(name, PLANETS.to_vec()));
            }
        }

        // §11.5.4: a to-judge row is applicable too, so it also stops these (ADR 0014).
        if found.len() == first_naabhasa {
            found.push(yoga(sankhya_yoga(planets), PLANETS.to_vec()));
        }

        found.extend(self.other_popular_yogas());
        found.extend(self.raaja_yogas());
        found.extend(self.dhana_yogas());
        found.extend(self.daridra_yogas());

        found
    }
}
impl VargaChakra<'_> {
    /// [`Chart::lord_relationship`] in `d`'s chart: `g`'s compound relationship with its rasi's seven-planet lord (§3.4.3).
    pub(crate) fn with_rasi_lord(&self, g: Body) -> Option<Compound> {
        self.compound_with(g, self.rasi_of(g).lord())
    }

    /// `g`'s compound relationship with `other` in `d`'s chart; none unless both are the seven planets, and differ (§3.4.3).
    pub(crate) fn compound_with(&self, g: Body, other: Body) -> Option<Compound> {
        Some(compound(
            natural(g, other)?,
            tatkaala(self.rasi_of(g), self.rasi_of(other)),
        ))
    }
}

/// §11.7.2's closeness of `a` and `b`: the difference of their degrees within their rasis in the rasi chart (ADR 0009, ADR 0014).
fn closeness(chart: &Chart, a: Body, b: Body) -> u32 {
    let degrees = |g| arcseconds(rasi(chart.entry(g).longitude).1);
    degrees(a).abs_diff(degrees(b))
}

/// §11.7.2's magnitude of `pair` from the rasi chart's longitudes (ADR 0009, ADR 0014).
fn magnitude(chart: &Chart, pair: [Body; 2]) -> Magnitude {
    let amsa = |g| {
        let (count, amsa) = amsabala(g, chart.entry(g).longitude, VargaGroup::DasaVarga);
        (g, count, amsa)
    };
    Magnitude {
        closeness: closeness(chart, pair[0], pair[1]),
        amsas: pair.map(amsa),
    }
}

impl VargaChakra<'_> {
    /// §11.7.2 (1)'s afflictions of a Raaja `pair` in `d`'s chart: each functional malefic from its lagna, not of the pair, in the
    /// rasi of one of them or aspecting it by graha drishti, per graha afflicted, in graha order (ADR 0016).
    fn afflictions(&self, pair: [Body; 2]) -> Vec<Affliction> {
        let lagna = ReferencePoint::Body(Body::Lagna);
        let rasi_chart = self.d == Varga::RasiChart;
        PLANETS
            .into_iter()
            .filter(|&g| {
                !pair.contains(&g)
                    && self.functional_nature(g, lagna) == Some(FunctionalNature::Malefic)
            })
            .flat_map(|g| pair.map(|afflicts| (g, afflicts)))
            .filter(|&(g, afflicts)| self.joins_or_aspects(g, self.rasi_of(afflicts)))
            .map(|(graha, afflicts)| Affliction {
                graha,
                afflicts,
                closeness: rasi_chart.then(|| closeness(self.chart, graha, afflicts)),
            })
            .collect()
    }
}

impl VargaChakra<'_> {
    /// Each way of choosing a lord for `houses` from `d`'s lagna, a co-lord giving two (ADR 0014).
    fn house_lords<const N: usize>(&self, houses: [usize; N]) -> Vec<[Body; N]> {
        let lagna = self.rasi_of(Body::Lagna);
        let mut ways = vec![[Body::Lagna; N]];
        for (i, k) in houses.into_iter().enumerate() {
            let of_house = self.lords_to_try(kth(lagna, k));
            ways = ways
                .iter()
                .flat_map(|way| {
                    of_house.iter().map(move |&lord| {
                        let mut way = *way;
                        way[i] = lord;
                        way
                    })
                })
                .collect();
        }
        ways
    }
}

/// A two-halved yoga's form: 1 for the first half, 2 for the second, 3 for both (see [`Yoga::form`]).
fn form_of(first: bool, second: bool) -> Option<u8> {
    match (first, second) {
        (true, false) => Some(1),
        (false, true) => Some(2),
        (true, true) => Some(3),
        (false, false) => None,
    }
}

impl VargaChakra<'_> {
    /// `g`'s dispositors in `v`'s chart for §11.7–11.10: himself in his own rasi (p. 128), else [`VargaChakra::lords_in_yoga`]'s (ADR 0014).
    fn dispositors_in_yoga(&self, g: Body) -> Vec<Body> {
        let rasi = self.rasi_of(g);
        if owns(g, rasi) {
            vec![g]
        } else {
            self.lords_in_yoga(rasi)
        }
    }
}

/// The row of a yoga with several forms or lords that hold: the least left to judge, then the first (ADR 0014).
fn the_best_of(rows: Vec<Yoga>) -> Option<Yoga> {
    rows.into_iter().min_by_key(|row| row.to_judge.len())
}

impl VargaChakra<'_> {
    /// `grahas` with their bhavas and every lordship, from `d`'s lagna, by the lords `name`'s rule tries.
    fn yoga_grahas(&self, name: YogaName, grahas: &[Body]) -> Vec<YogaGraha> {
        let lagna = self.rasi_of(Body::Lagna);
        // §11.6 also counts a co-lord at home, so its row names the lordship that formed it (p. 218, ADR 0014).
        let lords = RASIS.map(|rasi| {
            if name.class() == YogaClass::OtherPopular {
                self.lords_to_try(rasi)
            } else {
                self.lords_in_yoga(rasi)
            }
        });
        grahas
            .iter()
            .map(|&graha| YogaGraha {
                graha,
                bhava: bhava(lagna, self.rasi_of(graha)),
                // RASIS from Aries, so sorted after counting from the lagna.
                named_lord_of: {
                    let mut lord_of: Vec<u8> = RASIS
                        .into_iter()
                        .zip(&lords)
                        .filter(|(_, lords)| lords.contains(&graha))
                        .map(|(rasi, _)| bhava(lagna, rasi))
                        .collect();
                    lord_of.sort();
                    lord_of
                },
                karaka: None,
            })
            .collect()
    }
}

/// `grahas` once each, in the book's order.
fn in_graha_order(grahas: &[Body]) -> Vec<Body> {
    GRAHAS.into_iter().filter(|g| grahas.contains(g)).collect()
}

impl VargaChakra<'_> {
    /// The yoga `rule` gives its grahas for, Mercury being `Saumya` or `Kroora`: as he is in `d`'s chart, or each way on a tie, when
    /// agreement is the result and a difference is to judge (ch. 3 §3.2.2, ADR 0014). On agreement the saumya reading's grahas stand.
    fn with_mercury_each_way(
        &self,
        name: YogaName,
        rule: impl Fn(Disposition) -> Option<Vec<Body>>,
    ) -> Option<Yoga> {
        use Disposition::{Kroora, Saumya};
        let (grahas, to_judge) = match self.natural_nature(Body::Mercury) {
            Some(nature) => (rule(nature)?, Vec::new()),
            None => match (rule(Saumya), rule(Kroora)) {
                (Some(grahas), Some(_)) => (grahas, Vec::new()),
                (Some(grahas), None) | (None, Some(grahas)) => {
                    (grahas, vec![ToJudge::MercurysNature])
                }
                (None, None) => return None,
            },
        };
        Some(Yoga {
            name,
            grahas: self.yoga_grahas(name, &grahas),
            to_judge,
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests;

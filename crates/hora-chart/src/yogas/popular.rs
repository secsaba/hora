use super::*;

impl VargaChakra<'_> {
    /// §11.6, Subha to Vimala, then ch. 13 §13.5's Tapaswi. "Planets" are the seven, "benefics" and "malefics" by [`VargaChakra::natural_nature`] (ADR 0014).
    pub(crate) fn other_popular_yogas(&self) -> Vec<Yoga> {
        let (d, chart) = (self.d, self.chart);
        use Body::*;
        use Disposition::{Kroora, Saumya};
        use YogaName as N;

        let lagna = self.rasi_of(Lagna);
        let kendra = |rasi| matches!(bhava(lagna, rasi), 1 | 4 | 7 | 10);
        let trikona = |rasi| matches!(bhava(lagna, rasi), 1 | 5 | 9);
        let yoga = |name, grahas: Vec<Body>| Yoga {
            name,
            grahas: self.yoga_grahas(name, &grahas),
            to_judge: Vec::new(),
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        // Among the grahas only a tied Mercury has no nature.
        let of_nature =
            |g, nature, mercury_as| self.natural_nature(g).unwrap_or(mercury_as) == nature;
        let of_nature_in = |rasi, nature, mercury_as| -> Vec<Body> {
            GRAHAS
                .into_iter()
                .filter(|&g| self.rasi_of(g) == rasi && of_nature(g, nature, mercury_as))
                .collect()
        };
        // "Only" benefics: a tied Mercury always has a malefic beside him, so failing him here is right either way (§3.2.2).
        let only_benefics = |rasi: Rasi| {
            self.grahas_in(rasi)
                .iter()
                .all(|&g| self.natural_nature(g) == Some(Saumya))
        };
        let conjoins_or_graha_drishti_on = |g, other| self.joins_or_aspects(g, self.rasi_of(other));
        let with_rasi_lord = |g| self.with_rasi_lord(g);
        // A graha is never in a kendra from himself: these are relations between two (ADR 0014).
        let kendra_from =
            |a, b| a != b && matches!(bhava(self.rasi_of(a), self.rasi_of(b)), 1 | 4 | 7 | 10);
        let own_or_uchcha = |g| own_or_exalted(g, self.rasi_of(g));
        let row = |name, form, grahas: &[Body], to_judge| Yoga {
            name,
            grahas: self.yoga_grahas(name, &in_graha_order(grahas)),
            to_judge,
            form,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        let strong = |g| vec![ToJudge::Strong(g)];
        // A graha in his own rasi is his own dispositor, whatever the co-lord rule says: "Saturn himself" of Aq in chart 9's navamsa.
        let dispositors = |v, g| {
            let rasi = chart.in_varga(v).rasi_of(g);
            if owns(g, rasi) {
                vec![g]
            } else {
                chart.in_varga(v).lords_to_try(rasi)
            }
        };
        let in_house = |g, k| self.rasi_of(g) == kth(lagna, k);
        let friendly = |g| {
            own_or_uchcha(g)
                || matches!(
                    with_rasi_lord(g),
                    Some(Compound::Mitra | Compound::Adhimitra)
                )
        };
        // A graha never joins himself: these are relations between two (ADR 0014).
        let joins = |a, b| a != b && self.rasi_of(a) == self.rasi_of(b);
        let mut found = Vec::new();

        for (name, nature) in [(N::Subha, Saumya), (N::Asubha, Kroora)] {
            found.extend(self.with_mercury_each_way(name, |mercury_as| {
                let [first, second, twelfth] =
                    [1, 2, 12].map(|k| of_nature_in(kth(lagna, k), nature, mercury_as));
                let kartari = if second.is_empty() || twelfth.is_empty() {
                    Vec::new()
                } else {
                    [second, twelfth].concat()
                };
                let grahas = in_graha_order(&[first, kartari].concat());
                (!grahas.is_empty()).then_some(grahas)
            }));
        }

        // "Some authors'" form from the lagna is not computed (ADR 0014).
        let jupiter = self.rasi_of(Jupiter);
        let in_enemys_house = matches!(
            with_rasi_lord(Jupiter),
            Some(Compound::Satru | Compound::Adhisatru)
        );
        if matches!(bhava(self.rasi_of(Moon), jupiter), 1 | 4 | 7 | 10)
            && !in_enemys_house
            && !debilitated(Jupiter, jupiter)
        {
            let row = self.with_mercury_each_way(N::GajaKesari, |mercury_as| {
                let benefics: Vec<Body> = [Moon, Mercury, Venus]
                    .into_iter()
                    .filter(|&g| {
                        of_nature(g, Saumya, mercury_as) && conjoins_or_graha_drishti_on(g, Jupiter)
                    })
                    .collect();
                (!benefics.is_empty())
                    .then(|| in_graha_order(&[&[Moon, Jupiter], &benefics[..]].concat()))
            });
            found.extend(row.map(|mut row| {
                if d == Varga::RasiChart && self.rasi_of(Sun) == jupiter {
                    row.to_judge.push(ToJudge::Combust(Jupiter));
                }
                row
            }));
        }

        if matches!(bhava(jupiter, self.rasi_of(Mars)), 1 | 7) {
            found.push(yoga(N::GuruMangala, vec![Mars, Jupiter]));
        }

        let amala: Vec<Body> = [lagna, self.rasi_of(Moon)]
            .map(|from| kth(from, 10))
            .into_iter()
            .filter(|&rasi| only_benefics(rasi))
            .flat_map(|rasi| self.grahas_in(rasi))
            .collect();
        if !amala.is_empty() {
            found.push(yoga(N::Amala, in_graha_order(&amala)));
        }

        // (2) leaves the 7th free to be vacant, so a kendra may be, but one must be occupied (ADR 0014).
        let parvata = [1, 4, 7, 10, 8].map(|k| kth(lagna, k));
        let in_kendras = [1, 4, 7, 10]
            .iter()
            .any(|&k| !self.grahas_in(kth(lagna, k)).is_empty());
        if in_kendras && parvata.iter().all(|&rasi| only_benefics(rasi)) {
            let grahas: Vec<Body> = parvata
                .into_iter()
                .flat_map(|rasi| self.grahas_in(rasi))
                .collect();
            found.push(yoga(N::Parvata, in_graha_order(&grahas)));
        }

        // "Some say 9th lord" is not computed (ADR 0014).
        let mut rows = Vec::new();
        for [fourth, first] in self.house_lords([4, 1]) {
            if kendra_from(fourth, Jupiter) {
                let grahas = [fourth, Jupiter, first];
                rows.push(row(N::Kaahala, Some(1), &grahas, strong(first)));
            }
        }
        for [fourth, tenth] in self.house_lords([4, 10]) {
            if own_or_uchcha(fourth) && self.rasi_of(tenth) == self.rasi_of(fourth) {
                rows.push(row(N::Kaahala, Some(2), &[fourth, tenth], Vec::new()));
            }
        }
        found.extend(the_best_of(rows));

        found.extend(self.with_mercury_each_way(N::Chaamara, |mercury_as| {
            let mut grahas = Vec::new();
            for first in self.lords_to_try(lagna) {
                let place = self.rasi_of(first);
                if exalted(first, place) && kendra(place) && self.aspects(Jupiter, place) {
                    grahas.extend([first, Jupiter]);
                }
            }
            for k in [7, 9, 10] {
                let benefics = of_nature_in(kth(lagna, k), Saumya, mercury_as);
                if benefics.len() >= 2 {
                    grahas.extend(benefics);
                }
            }
            (!grahas.is_empty()).then(|| in_graha_order(&grahas))
        }));

        let mut rows = Vec::new();
        for [first, fifth, sixth] in self.house_lords([1, 5, 6]) {
            if kendra_from(fifth, sixth) {
                let grahas = [first, fifth, sixth];
                rows.push(row(N::Sankha, Some(1), &grahas, strong(first)));
            }
        }
        for [first, tenth, ninth] in self.house_lords([1, 10, 9]) {
            if first != tenth
                && self.rasi_of(first) == self.rasi_of(tenth)
                && self.rasi_of(first).mobility() == Mobility::Chara
            {
                let grahas = [first, tenth, ninth];
                rows.push(row(N::Sankha, Some(2), &grahas, strong(ninth)));
            }
        }
        found.extend(the_best_of(rows));

        let planets_in = |rasi| -> Vec<Body> {
            PLANETS
                .iter()
                .copied()
                .filter(|&g| self.rasi_of(g) == rasi)
                .collect()
        };
        let bheris_houses = [1, 2, 7, 12].map(|k| planets_in(kth(lagna, k)));
        let mut rows = Vec::new();
        for [ninth] in self.house_lords([9]) {
            if bheris_houses.iter().all(|planets| !planets.is_empty()) {
                let grahas = [&bheris_houses.concat()[..], &[ninth]].concat();
                rows.push(row(N::Bheri, Some(1), &grahas, strong(ninth)));
            }
        }
        for [ninth, first] in self.house_lords([9, 1]) {
            // The lagna's lord may be Jupiter or Venus himself, leaving the one pair.
            if kendra_from(Jupiter, Venus)
                && [Jupiter, Venus]
                    .iter()
                    .all(|&g| g == first || kendra_from(g, first))
            {
                let grahas = [Jupiter, Venus, first, ninth];
                rows.push(row(N::Bheri, Some(2), &grahas, strong(ninth)));
            }
        }
        found.extend(the_best_of(rows));

        // At least one, as "benefics occupy" and "some planets" read (ADR 0014).
        let dignified: Vec<Body> = PLANETS
            .iter()
            .copied()
            .filter(|&g| own_or_uchcha(g) && (kendra(self.rasi_of(g)) || trikona(self.rasi_of(g))))
            .collect();
        if !dignified.is_empty() {
            // The lagna lord alone dignified is one graha, and no yoga (ADR 0014).
            let rows = self
                .lords_to_try(lagna)
                .into_iter()
                .filter(|first| dignified != [*first])
                .map(|first| {
                    let grahas = [&dignified[..], &[first]].concat();
                    row(N::Mridanga, None, &grahas, strong(first))
                });
            found.extend(the_best_of(rows.collect()));
        }

        // Applied strictly, as footnote 33 says: so only for a Sagittarius lagna.
        let rows = self
            .house_lords([7, 10, 9])
            .into_iter()
            .filter(|&[seventh, tenth, ninth]| {
                self.rasi_of(seventh) == kth(lagna, 10)
                    && exalted(seventh, self.rasi_of(seventh))
                    && self.rasi_of(tenth) == self.rasi_of(ninth)
            })
            .map(|[seventh, tenth, ninth]| {
                row(N::Sreenaatha, None, &[seventh, tenth, ninth], Vec::new())
            });
        found.extend(the_best_of(rows.collect()));

        // The book glosses its chaturasras as the 4th and 8th.
        found.extend(self.with_mercury_each_way(N::Matsya, |mercury_as| {
            let houses = [
                of_nature_in(kth(lagna, 1), Saumya, mercury_as),
                of_nature_in(kth(lagna, 9), Saumya, mercury_as),
                planets_in(kth(lagna, 5)),
                of_nature_in(kth(lagna, 4), Kroora, mercury_as),
                of_nature_in(kth(lagna, 8), Kroora, mercury_as),
            ];
            houses
                .iter()
                .all(|held| !held.is_empty())
                .then(|| in_graha_order(&houses.concat()))
        }));

        found.extend(self.with_mercury_each_way(N::Koorma, |mercury_as| {
            let benefics = [5, 6, 7].map(|k| {
                let mut held = of_nature_in(kth(lagna, k), Saumya, mercury_as);
                held.retain(|&g| friendly(g));
                held
            });
            let malefics = [1, 3, 11].map(|k| {
                let mut held = of_nature_in(kth(lagna, k), Kroora, mercury_as);
                held.retain(|&g| own_or_uchcha(g));
                held
            });
            benefics
                .iter()
                .chain(&malefics)
                .all(|held| !held.is_empty())
                .then(|| in_graha_order(&[benefics.concat(), malefics.concat()].concat()))
        }));

        let rows = self
            .house_lords([2, 9, 1])
            .into_iter()
            .filter(|&[second, ninth, first]| {
                self.rasi_of(second) == kth(lagna, 9)
                    && self.rasi_of(ninth) == kth(lagna, 2)
                    && (kendra(self.rasi_of(first)) || trikona(self.rasi_of(first)))
            })
            .map(|lords| row(N::Khadga, None, &lords, Vec::new()));
        found.extend(the_best_of(rows.collect()));

        if lagna.mobility() == Mobility::Sthira
            && kendra(self.rasi_of(Venus))
            && trikona(self.rasi_of(Moon))
            && self.rasi_of(Saturn) == kth(lagna, 10)
        {
            found.extend(self.with_mercury_each_way(N::Kusuma, |mercury_as| {
                let mut company = of_nature_in(self.rasi_of(Moon), Saumya, mercury_as);
                company.retain(|&g| g != Moon);
                (!company.is_empty())
                    .then(|| in_graha_order(&[&[Moon, Venus, Saturn], &company[..]].concat()))
            }));
        }

        if matches!(bhava(lagna, jupiter), 2 | 5)
            && conjoins_or_graha_drishti_on(Mercury, Jupiter)
            && conjoins_or_graha_drishti_on(Venus, Jupiter)
        {
            found.push(yoga(N::Kalaanidhi, vec![Mercury, Jupiter, Venus]));
        }

        // Rasi-only: the 4th graha is taken in the navamsa and placed in the rasi chart (ADR 0014). pp. 128–129.
        if d == Varga::RasiChart {
            let placed = |g| {
                kendra(self.rasi_of(g)) || trikona(self.rasi_of(g)) || exalted(g, self.rasi_of(g))
            };
            let mut rows = Vec::new();
            for first in self.lords_to_try(lagna) {
                for second in dispositors(d, first) {
                    for third in dispositors(d, second) {
                        for fourth in dispositors(Varga::Navamsa, second) {
                            let four = [first, second, third, fourth];
                            // One graha all four times is no yoga (ADR 0014).
                            if four.iter().all(|&g| placed(g)) && four.iter().any(|&g| g != first) {
                                rows.push(yoga(N::Kalpadruma, four.to_vec()));
                            }
                        }
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        // A kroora Mercury is a malefic here; "Adhi Yoga from lagna" does not add the 6th (ADR 0014).
        found.extend(self.with_mercury_each_way(N::Lagnaadhi, |mercury_as| {
            let places = [kth(lagna, 7), kth(lagna, 8)];
            let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
                .into_iter()
                .filter(|&g| of_nature(g, Saumya, mercury_as) && places.contains(&self.rasi_of(g)))
                .collect();
            let afflicted = benefics.iter().any(|&b| {
                GRAHAS
                    .into_iter()
                    .any(|m| of_nature(m, Kroora, mercury_as) && conjoins_or_graha_drishti_on(m, b))
            });
            (benefics.len() >= 2 && !afflicted).then_some(benefics)
        }));

        // Sets of places: every benefic but the graha counted from, and so at least one, Jupiter or Venus (ADR 0014).
        let trimurthi = |name, house, places: [usize; 3]| -> Vec<Yoga> {
            self.lords_to_try(kth(lagna, house))
                .into_iter()
                .filter_map(|lord| {
                    self.with_mercury_each_way(name, |mercury_as| {
                        let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
                            .into_iter()
                            .filter(|&g| g != lord && of_nature(g, Saumya, mercury_as))
                            .collect();
                        benefics
                            .iter()
                            .all(|&g| {
                                places
                                    .iter()
                                    .any(|&k| kth(self.rasi_of(lord), k) == self.rasi_of(g))
                            })
                            .then(|| in_graha_order(&[&benefics[..], &[lord]].concat()))
                    })
                })
                .collect()
        };
        found.extend(the_best_of(trimurthi(N::Hari, 2, [2, 12, 8])));
        found.extend(the_best_of(trimurthi(N::Hara, 7, [4, 9, 8])));

        let mut rows = trimurthi(N::Brahma, 1, [4, 10, 11]);
        for row in &mut rows {
            row.form = Some(1);
        }
        // The NOTE's variation, p. 130.
        for [ninth, eleventh, first, tenth] in self.house_lords([9, 11, 1, 10]) {
            let mercurys: Vec<Body> = [first, tenth]
                .into_iter()
                .filter(|&l| kendra_from(Mercury, l))
                .collect();
            if kendra_from(Jupiter, ninth) && kendra_from(Venus, eleventh) && !mercurys.is_empty() {
                let grahas = [&[Jupiter, ninth, Venus, eleventh, Mercury], &mercurys[..]].concat();
                rows.push(row(N::Brahma, Some(2), &grahas, Vec::new()));
            }
        }
        found.extend(the_best_of(rows));

        // Rasi-only: the 9th lord's navamsa picks a graha placed in the rasi chart (ADR 0014).
        if d == Varga::RasiChart {
            let mut rows = Vec::new();
            for [ninth, tenth] in self.house_lords([9, 10]) {
                for lord in dispositors(Varga::Navamsa, ninth) {
                    let grahas = [ninth, tenth, lord];
                    // The dispositor must bring in another graha (ADR 0014).
                    if grahas.iter().all(|&g| in_house(g, 2)) && lord != ninth {
                        rows.push(row(N::Vishnu, None, &grahas, Vec::new()));
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        let rows = self
            .house_lords([5, 9, 10])
            .into_iter()
            .filter(|&[fifth, ninth, tenth]| {
                in_house(fifth, 9) && in_house(ninth, 10) && in_house(tenth, 5)
            })
            .map(|lords| row(N::Siva, None, &lords, Vec::new()));
        found.extend(the_best_of(rows.collect()));

        // The same rasi counts as a trine, as it does as a kendra, and a trine's trines are its own (ADR 0014).
        let trine_from = |a, b| matches!(bhava(self.rasi_of(a), self.rasi_of(b)), 1 | 5 | 9);
        if trine_from(Sun, Moon) && trine_from(Sun, Mars) {
            found.push(yoga(N::Trilochana, vec![Sun, Moon, Mars]));
        }

        if d == Varga::RasiChart {
            let mut rows = Vec::new();
            for [tenth, first] in self.house_lords([10, 1]) {
                for lord in dispositors(Varga::Navamsa, tenth) {
                    if in_house(lord, 10) && exalted(lord, self.rasi_of(lord)) && joins(first, lord)
                    {
                        rows.push(row(N::Gouri, None, &[tenth, lord, first], Vec::new()));
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        if d == Varga::RasiChart && lagna.mobility() == Mobility::Sthira {
            let mut rows = Vec::new();
            for [sixth, ninth] in self.house_lords([6, 9]) {
                for a in dispositors(Varga::Navamsa, sixth) {
                    for b in dispositors(Varga::Navamsa, ninth) {
                        // The Sun as one of them leaves the other, as Bheri's lagna lord does, but joins no one as both (ADR 0014).
                        if self.aspects(sixth, lagna)
                            && (a != Sun || b != Sun)
                            && [a, b].iter().all(|&g| g == Sun || joins(g, Sun))
                        {
                            let grahas = [sixth, ninth, Sun, a, b];
                            rows.push(row(N::Chandikaa, None, &grahas, Vec::new()));
                        }
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        // "Quadrant" holds of the own rasi and the uchcha alike.
        let rows = self
            .house_lords([9, 1])
            .into_iter()
            .filter(|&[ninth, _]| own_or_uchcha(ninth) && kendra(self.rasi_of(ninth)))
            .map(|[ninth, first]| row(N::Lakshmi, None, &[ninth, first], strong(first)));
        found.extend(the_best_of(rows.collect()));

        let in_moons_trines: Vec<Body> = [Mercury, Jupiter]
            .into_iter()
            .filter(|&g| trine_from(Moon, g))
            .collect();
        if kendra(self.rasi_of(Mercury))
            && self.rasi_of(Sun) == Rasi::Leo
            && !in_moons_trines.is_empty()
            && in_house(Mars, 11)
        {
            let rows = self
                .house_lords([10])
                .into_iter()
                .filter(|&[tenth]| in_house(tenth, 5))
                .map(|[tenth]| {
                    let grahas =
                        [&[tenth, Mercury, Sun, Moon, Mars], &in_moons_trines[..]].concat();
                    row(N::Saarada, None, &grahas, strong(Sun))
                });
            found.extend(the_best_of(rows.collect()));
        }

        if d == Varga::RasiChart {
            let mut rows = Vec::new();
            for k in [2, 5, 11] {
                for [lord, ninth] in self.house_lords([k, 9]) {
                    for disposing in dispositors(Varga::Navamsa, lord) {
                        if exalted(disposing, self.rasi_of(disposing)) && joins(disposing, ninth) {
                            let grahas = [lord, disposing, ninth];
                            rows.push(row(N::Bhaarathi, None, &grahas, Vec::new()));
                        }
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        let saraswathi = [Mercury, Jupiter, Venus];
        if saraswathi
            .iter()
            .all(|&g| matches!(bhava(lagna, self.rasi_of(g)), 1 | 2 | 4 | 5 | 7 | 9 | 10))
            && friendly(Jupiter)
        {
            found.push(yoga(N::Saraswathi, saraswathi.to_vec()));
        }

        let amsaavatara = [Jupiter, Venus, Saturn];
        if amsaavatara.iter().all(|&g| kendra(self.rasi_of(g)))
            && exalted(Saturn, self.rasi_of(Saturn))
        {
            found.push(yoga(N::Amsaavatara, amsaavatara.to_vec()));
        }

        // Footnote 35: each lord in the other's house.
        let exchange = |a, i, b, j| in_house(a, i) && in_house(b, j);
        if lagna.mobility() == Mobility::Sthira {
            let rows = self
                .house_lords([2, 10, 1, 11])
                .into_iter()
                .filter(|&[second, tenth, first, eleventh]| {
                    exchange(second, 10, tenth, 2) && exchange(first, 11, eleventh, 1)
                })
                .map(|lords| row(N::Devendra, None, &lords, Vec::new()));
            found.extend(the_best_of(rows.collect()));
        }

        let rows = self
            .house_lords([5, 11])
            .into_iter()
            .filter(|&[fifth, eleventh]| exchange(fifth, 11, eleventh, 5) && in_house(Moon, 5))
            .map(|[fifth, eleventh]| row(N::Indra, None, &[fifth, eleventh, Moon], Vec::new()));
        found.extend(the_best_of(rows.collect()));

        let rows = self
            .house_lords([10])
            .into_iter()
            .filter(|&[tenth]| in_house(Sun, 10) && in_house(tenth, 3) && joins(tenth, Saturn))
            .map(|[tenth]| row(N::Ravi, None, &[Sun, tenth, Saturn], Vec::new()));
        found.extend(the_best_of(rows.collect()));

        if bhava(self.rasi_of(Sun), self.rasi_of(Moon)) == 12
            && bhava(self.rasi_of(Sun), self.rasi_of(Mercury)) == 2
            && matches!(bhava(self.rasi_of(Moon), jupiter), 5 | 9)
        {
            found.push(yoga(N::Bhaaskara, vec![Sun, Moon, Mercury, Jupiter]));
        }

        let fifths = [lagna, self.rasi_of(Moon), self.rasi_of(Sun)].map(|from| kth(from, 5));
        if PLANETS.iter().all(|&g| fifths.contains(&self.rasi_of(g))) {
            found.push(yoga(N::Kulavardhana, PLANETS.to_vec()));
        }

        // A tied Mercury has a benefic and a malefic beside him, so his reading decides neither the yoga nor the note (ADR 0014).
        let upachaya = |g| matches!(bhava(lagna, self.rasi_of(g)), 3 | 6 | 10 | 11);
        let benefics: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
            .into_iter()
            .filter(|&g| of_nature(g, Saumya, Saumya))
            .collect();
        if benefics.iter().all(|&g| upachaya(g)) {
            let malefics = GRAHAS
                .into_iter()
                .filter(|&g| upachaya(g) && self.natural_nature(g) == Some(Kroora));
            let to_judge = benefics
                .iter()
                .map(|&g| ToJudge::Strong(g))
                .chain(malefics.map(ToJudge::MaleficInUpachaya))
                .collect();
            found.push(row(N::Vasumati, None, &benefics, to_judge));
        }

        if exalted(Sun, self.rasi_of(Sun)) && in_house(Moon, 9) {
            let rows = self
                .house_lords([10, 1])
                .into_iter()
                .filter(|&[tenth, first]| {
                    matches!(bhava(kth(lagna, 7), self.rasi_of(tenth)), 1 | 5 | 9)
                        && first != Jupiter
                        && conjoins_or_graha_drishti_on(Jupiter, first)
                })
                .map(|[tenth, first]| {
                    let grahas = [tenth, first, Jupiter, Sun, Moon];
                    row(N::Gandharva, None, &grahas, strong(Sun))
                });
            found.extend(the_best_of(rows.collect()));
        }

        // The whole sign: §3.3's degrees are of results, as Exercise 41 and §6.6's example show (ADR 0014).
        if jupiter == Rasi::Sagittarius {
            let rows = self
                .house_lords([2, 1])
                .into_iter()
                .filter(|&[second, first]| {
                    joins(second, Jupiter) && exalted(first, self.rasi_of(first))
                })
                .map(|[second, first]| {
                    row(N::Go, None, &[second, Jupiter, first], strong(Jupiter))
                });
            found.extend(the_best_of(rows.collect()));
        }

        if d == Varga::RasiChart {
            let rows = self
                .house_lords([11, 1])
                .into_iter()
                .filter(|&[eleventh, first]| {
                    exalted(eleventh, self.rasi_of(eleventh))
                    && joins(eleventh, Venus)
                    // A Venus who is the lagna lord leaves the 11th lord, as Bheri's does (ADR 0014).
                    && [eleventh, Venus]
                        .iter()
                        .all(|&g| g == first || kendra_from(first, g))
                })
                .map(|[eleventh, first]| {
                    let deep = vec![ToJudge::DeepExaltation(eleventh)];
                    row(N::Vidyut, None, &[eleventh, Venus, first], deep)
                });
            found.extend(the_best_of(rows.collect()));
        }

        let rows = self
            .house_lords([4, 10, 1])
            .into_iter()
            .filter(|&[fourth, tenth, first]| {
                exchange(fourth, 10, tenth, 4) && exalted(first, self.rasi_of(first))
            })
            .map(|lords| row(N::Chapa, None, &lords, Vec::new()));
        found.extend(the_best_of(rows.collect()));

        // The book numbers its four conditions (1), (2), (2), (4).
        let lagna_planets = planets_in(lagna);
        if !lagna_planets.is_empty() {
            let mut rows = Vec::new();
            for first in self.lords_to_try(lagna) {
                for lord in dispositors(d, Moon) {
                    if joins(first, Moon)
                        && (kendra(self.rasi_of(lord))
                            || with_rasi_lord(lord) == Some(Compound::Adhimitra))
                        && self.aspects(lord, lagna)
                    {
                        let grahas = [&[first, Moon, lord], &lagna_planets[..]].concat();
                        rows.push(row(N::Pushkala, None, &grahas, Vec::new()));
                    }
                }
            }
            found.extend(the_best_of(rows));
        }

        // A tied Mercury has a benefic beside him, so his reading never decides; a saumya reading lists him (ADR 0014).
        let beyond_jupiter = of_nature_in(kth(jupiter, 9), Saumya, Saumya);
        if !beyond_jupiter.is_empty() && in_house(Saturn, 10) {
            let rows = self
                .house_lords([9])
                .into_iter()
                .filter(|&[ninth]| bhava(self.rasi_of(ninth), jupiter) == 9)
                .map(|[ninth]| {
                    let grahas = [&[ninth, Jupiter, Saturn], &beyond_jupiter[..]].concat();
                    row(N::Makuta, None, &grahas, Vec::new())
                });
            found.extend(the_best_of(rows.collect()));
        }

        if d == Varga::RasiChart {
            let rows = self
                .house_lords([10, 6])
                .into_iter()
                .filter(|&[tenth, sixth]| {
                    exalted(tenth, self.rasi_of(tenth)) && debilitated(sixth, self.rasi_of(sixth))
                })
                .map(|[tenth, sixth]| {
                    let deep = vec![ToJudge::DeepExaltation(tenth)];
                    row(N::Jaya, None, &[tenth, sixth], deep)
                });
            found.extend(the_best_of(rows.collect()));
        }

        for (name, k) in [(N::Harsha, 6), (N::Sarala, 8), (N::Vimala, 12)] {
            let rows = self
                .lords_to_try(kth(lagna, k))
                .into_iter()
                .filter(|&lord| in_house(lord, k))
                .map(|lord| row(name, None, &[lord], Vec::new()));
            found.extend(the_best_of(rows.collect()));
        }

        // Ch. 13 §13.5, Example 44 (p. 174): two of the three in one rasi and the third aspecting them by graha drishti, so never
        // Ketu (ADR 0013); all three together is to judge, as the book only says "aspects" (ADR 0016).
        let tapaswi = [Venus, Saturn, Ketu];
        let together = |a, b| self.rasi_of(a) == self.rasi_of(b);
        let to_judge = if together(Venus, Saturn) && together(Saturn, Ketu) {
            Some(vec![ToJudge::AllInOneRasi])
        } else {
            [(Venus, Saturn, Ketu), (Saturn, Venus, Ketu)]
                .into_iter()
                .any(|(third, a, b)| together(a, b) && conjoins_or_graha_drishti_on(third, a))
                .then(Vec::new)
        };
        if let Some(to_judge) = to_judge {
            // A karakatwa shared on a tie is each holder's (ADR 0011).
            let mut notes: Vec<Note> = chart
                .chara_karakas()
                .into_iter()
                .filter(|&(ak, k)| {
                    k == CharaKaraka::Atma
                        && tapaswi
                            .iter()
                            .any(|&g| g != ak && conjoins_or_graha_drishti_on(ak, g))
                })
                .map(|(ak, _)| Note::AtmaKaraka(ak))
                .collect();
            notes.extend(
                tapaswi
                    .into_iter()
                    .filter(|&g| in_house(g, 8))
                    .map(Note::InTheEighth),
            );
            found.push(Yoga {
                notes,
                ..row(N::Tapaswi, None, &tapaswi, to_judge)
            });
        }

        found
    }
}

#[cfg(test)]
mod tests;

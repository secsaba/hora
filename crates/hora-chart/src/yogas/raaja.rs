use super::*;

impl VargaChakra<'_> {
    /// §11.7 and §11.8, Raaja and Raaja Sambandha yogas, from `d`'s lagna. "Planets" are the seven, "benefics" and "malefics" by
    /// [`VargaChakra::natural_nature`], an aspect is graha drishti and a lord is [`VargaChakra::lords_in_yoga`]'s, not §11.6's [`VargaChakra::lords_to_try`] (ADR 0014).
    pub(crate) fn raaja_yogas(&self) -> Vec<Yoga> {
        let (d, chart) = (self.d, self.chart);
        use Body::*;
        use Disposition::{Kroora, Saumya};
        use YogaName as N;

        let lagna = self.rasi_of(Lagna);
        let house = |k| kth(lagna, k);
        let lords_of = |k| self.lords_in_yoga(house(k));
        let rasi_chart = d == Varga::RasiChart;
        let row = |name, grahas: &[Body], to_judge| Yoga {
            name,
            grahas: self.yoga_grahas(name, &in_graha_order(grahas)),
            to_judge,
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        // `pair` comes in graha order, the row's.
        let paired = |name, pair: [Body; 2], to_judge| Yoga {
            magnitude: rasi_chart.then(|| magnitude(chart, pair)),
            afflictions: self.afflictions(pair),
            notes: Vec::new(),
            ..row(name, &pair, to_judge)
        };
        // A graha never joins himself: these are relations between two (ADR 0014).
        let joins = |a, b| a != b && self.rasi_of(a) == self.rasi_of(b);
        let aspects = |a: Body, b: Body| self.aspects(a, self.rasi_of(b));
        let own_or_uchcha = |g| own_or_exalted(g, self.rasi_of(g));
        let first_of = |xs: &[Body], ys: &[Body], holds: &dyn Fn(Body, Body) -> bool| {
            xs.iter()
                .flat_map(|&a| ys.iter().map(move |&b| [a, b]))
                .find(|&[a, b]| holds(a, b))
        };
        // Among the grahas only a tied Mercury has no nature.
        let of_nature =
            |g, nature, mercury_as| self.natural_nature(g).unwrap_or(mercury_as) == nature;
        let mut found = Vec::new();

        // §11.7.1: one row per pair of grahas, the first co-lord that holds standing for its houses (ADR 0014).
        let mut pairs: Vec<([Body; 2], bool)> = Vec::new();
        for k in [1, 4, 7, 10] {
            for t in [1, 5, 9] {
                // Two lords of one house are no pair: the lagna's, or a yogakaraka and his co-lord (ADR 0014).
                let co_lords = |a, b| {
                    [k, t]
                        .iter()
                        .any(|&h| owns(a, house(h)) && owns(b, house(h)))
                };
                let found_pair = first_of(&lords_of(k), &lords_of(t), &|a, b| {
                    self.associated(a, b) && !co_lords(a, b)
                });
                if let Some(pair) = found_pair {
                    let dharma_karma = (k, t) == (10, 9);
                    let pair = in_graha_order(&pair);
                    match pairs.iter_mut().find(|(p, _)| *p == pair[..]) {
                        Some((_, dk)) => *dk |= dharma_karma,
                        None => pairs.push(([pair[0], pair[1]], dharma_karma)),
                    }
                }
            }
        }
        for dharma_karma in [false, true] {
            for &(pair, _) in pairs.iter().filter(|(_, dk)| *dk == dharma_karma) {
                let name = if dharma_karma {
                    N::DharmaKarmadhipati
                } else {
                    N::BasicRaaja
                };
                found.push(paired(name, pair, Vec::new()));
            }
        }

        // A dusthana lord in a dusthana is present, two conjoined elsewhere leave their strength to judge (p. 494, ADR 0014).
        let dusthana = |r| matches!(bhava(lagna, r), 6 | 8 | 12);
        let dusthana_lords = in_graha_order(&[6, 8, 12].map(lords_of).concat());
        for &g in dusthana_lords
            .iter()
            .filter(|&&g| dusthana(self.rasi_of(g)))
        {
            found.push(row(N::VipareetaRaaja, &[g], Vec::new()));
        }
        for (i, &a) in dusthana_lords.iter().enumerate() {
            for &b in &dusthana_lords[i + 1..] {
                if self.rasi_of(a) == self.rasi_of(b) && !dusthana(self.rasi_of(a)) {
                    let strong = vec![ToJudge::Strong(a), ToJudge::Strong(b)];
                    found.push(paired(N::VipareetaRaaja, [a, b], strong));
                }
            }
        }

        // §11.7.3, from the rasi chart's karakas; a karakatwa shared on a tie is each holder's, an empty one fails (ADR 0011).
        let karakas = chart.chara_karakas();
        let karaka = |k| -> Vec<Body> {
            karakas
                .iter()
                .filter(|&&(_, c)| c == k)
                .map(|&(g, _)| g)
                .collect()
        };
        let (ak, pk) = (karaka(CharaKaraka::Atma), karaka(CharaKaraka::Putra));
        let (first, fifth, ninth) = (lords_of(1), lords_of(5), lords_of(9));

        // (1): either half by itself is a Raaja yoga (chart 3, p. 265; ADR 0014).
        let karakas_joined = first_of(&ak, &pk, &joins);
        let lords_joined = first_of(&first, &fifth, &joins);
        let form = form_of(karakas_joined.is_some(), lords_joined.is_some());
        if form.is_some() {
            let grahas = [karakas_joined, lords_joined]
                .into_iter()
                .flatten()
                .flatten();
            let mut row = Yoga {
                form,
                ..row(N::Raaja(1), &grahas.collect::<Vec<_>>(), Vec::new())
            };
            // §11.7.3 (1): form 1 is (a) AK with PK, 2 is (b) the lagna and 5th lords, 3 both.
            for g in &mut row.grahas {
                g.karaka = karakas
                    .iter()
                    .find(|k| k.0 == g.graha)
                    .map(|k| k.1)
                    .filter(|&k| {
                        form != Some(2) && matches!(k, CharaKaraka::Atma | CharaKaraka::Putra)
                    });
                g.named_lord_of
                    .retain(|&n| form != Some(1) && matches!(n, 1 | 5));
            }
            found.push(row);
        }

        let aspected_by = |g, nature, mercury_as| {
            GRAHAS
                .into_iter()
                .any(|b| of_nature(b, nature, mercury_as) && aspects(b, g))
        };
        // (2): (d)'s "those planets" are (c)'s AK and PK, and "amsa" is the navamsa (ADR 0014).
        found.extend(self.with_mercury_each_way(N::Raaja(2), |mercury_as| {
            let fit = |g| {
                matches!(bhava(lagna, self.rasi_of(g)), 1 | 5)
                    && (own_or_uchcha(g)
                        || owns(g, chart.in_varga(Varga::Navamsa).rasi_of(g))
                        || aspected_by(g, Saumya, mercury_as))
            };
            let exchanged = first_of(&first, &fifth, &|l, f| {
                self.rasi_of(l) == house(5) && self.rasi_of(f) == lagna
            })?;
            let karakas = first_of(&ak, &pk, &|a, p| fit(a) && fit(p))?;
            Some(in_graha_order(&[exchanged, karakas].concat()))
        }));

        // (3): each in one of the houses and aspected by a benefic.
        found.extend(self.with_mercury_each_way(N::Raaja(3), |mercury_as| {
            let fit = |g| {
                matches!(bhava(lagna, self.rasi_of(g)), 1 | 5 | 7)
                    && aspected_by(g, Saumya, mercury_as)
            };
            first_of(&ninth, &ak, &|n, a| fit(n) && fit(a)).map(|pair| in_graha_order(&pair))
        }));

        // (4), (5): each named house from each holds one, others beside it or not (ADR 0014).
        let of_nature_in = |r, nature, mercury_as| -> Vec<Body> {
            GRAHAS
                .into_iter()
                .filter(|&g| self.rasi_of(g) == r && of_nature(g, nature, mercury_as))
                .collect()
        };
        found.extend(self.with_mercury_each_way(N::Raaja(4), |mercury_as| {
            let fit = |g: Body| {
                [2, 4, 5]
                    .iter()
                    .all(|&k| !of_nature_in(kth(self.rasi_of(g), k), Saumya, mercury_as).is_empty())
            };
            let [l, a] = first_of(&first, &ak, &|l, a| fit(l) && fit(a))?;
            let benefics = [l, a].into_iter().flat_map(|g| {
                [2, 4, 5].map(|k| of_nature_in(kth(self.rasi_of(g), k), Saumya, mercury_as))
            });
            Some(in_graha_order(
                &[vec![l, a], benefics.flatten().collect()].concat(),
            ))
        }));
        found.extend(self.with_mercury_each_way(N::Raaja(5), |mercury_as| {
            // Saturn's 3rd aspect never counts for himself: these are relations between two (ADR 0014).
            let malefics_on = |from: Body, k| -> Vec<Body> {
                let r = kth(self.rasi_of(from), k);
                GRAHAS
                    .into_iter()
                    .filter(|&g| {
                        g != from && of_nature(g, Kroora, mercury_as) && self.joins_or_aspects(g, r)
                    })
                    .collect()
            };
            let fit = |g: Body| [3, 6].iter().all(|&k| !malefics_on(g, k).is_empty());
            let [l, a] = first_of(&first, &ak, &|l, a| fit(l) && fit(a))?;
            let malefics = [l, a]
                .into_iter()
                .flat_map(|g| [3, 6].map(|k| malefics_on(g, k)));
            Some(in_graha_order(
                &[vec![l, a], malefics.flatten().collect()].concat(),
            ))
        }));

        // (6), (8): hora's own HL and GL, placed in `d` (ADR 0008). (6)'s "owned" and looser form are not computed (ADR 0014).
        let special = |which| {
            chart
                .special_lagna(which)
                .map(|longitude| varga(d, longitude))
        };
        if let (Some(hl), Some(gl)) = (
            special(SpecialLagna::HoraLagna),
            special(SpecialLagna::GhatiLagna),
        ) {
            let three = [lagna, hl, gl];
            let on_all: Vec<Body> = PLANETS
                .into_iter()
                .filter(|&g| three.iter().all(|&r| self.joins_or_aspects(g, r)))
                .collect();
            if !on_all.is_empty() {
                found.push(row(N::Raaja(6), &on_all, Vec::new()));
            }
            let dignified_in = |r| -> Vec<Body> {
                PLANETS
                    .into_iter()
                    .filter(|&g| self.rasi_of(g) == r && own_or_uchcha(g))
                    .collect()
            };
            if three.iter().all(|&r| !dignified_in(r).is_empty()) {
                let grahas: Vec<Body> = three.iter().flat_map(|&r| dignified_in(r)).collect();
                found.push(row(N::Raaja(8), &grahas, Vec::new()));
            }
        }

        // (9): rasi-only, since it reads other vargas. (7) cannot hold in the book's D-2, only Cn and Le, so is not computed (ADR 0014).
        if rasi_chart {
            use Varga::*;
            let dignified_on_lagna = |v| -> Vec<Body> {
                let r = chart.in_varga(v).rasi_of(Lagna);
                PLANETS
                    .into_iter()
                    .filter(|&g| {
                        let at = chart.in_varga(v).rasi_of(g);
                        at == r && own_or_exalted(g, at)
                    })
                    .collect()
            };
            let held = [RasiChart, Navamsa, Drekkana].map(dignified_on_lagna);
            if held.iter().all(|h| !h.is_empty()) {
                found.push(row(N::Raaja(9), &held.concat(), Vec::new()));
            }
        }

        // (10), (11) (b): a lagna lord in own or uchcha rasi aspecting the lagna, so not in it.
        let strong_first = first
            .iter()
            .copied()
            .find(|&g| own_or_uchcha(g) && self.aspects(g, lagna));
        if let Some(l) = strong_first {
            // (10) (a): with (b) at most two can be there, so "one or two" is any.
            let low: Vec<Body> = PLANETS
                .into_iter()
                .filter(|&g| {
                    matches!(bhava(lagna, self.rasi_of(g)), 3 | 6 | 8)
                        && debilitated(g, self.rasi_of(g))
                })
                .collect();
            if !low.is_empty() {
                found.push(row(N::Raaja(10), &[low, vec![l]].concat(), Vec::new()));
            }
            // (11) (a): every dusthana has a lord so placed; combustion follows ADR 0014.
            let weak = |g: Body| -> Option<Vec<ToJudge>> {
                let inimical = matches!(
                    self.with_rasi_lord(g),
                    Some(Compound::Satru | Compound::Adhisatru)
                );
                if debilitated(g, self.rasi_of(g)) || inimical {
                    Some(Vec::new())
                } else if rasi_chart
                    && PLANETS[1..].contains(&g)
                    && self.rasi_of(g) == self.rasi_of(Sun)
                {
                    Some(vec![ToJudge::Combust(g)])
                } else {
                    None
                }
            };
            let mut grahas = vec![l];
            let mut to_judge = Vec::new();
            let all_weak = [6, 8, 12].iter().all(|&k| {
                let mut ways: Vec<(Body, Vec<ToJudge>)> = lords_of(k)
                    .into_iter()
                    .filter_map(|g| Some((g, weak(g)?)))
                    .collect();
                ways.sort_by_key(|(_, j)| j.len());
                ways.first().is_some_and(|(g, j)| {
                    grahas.push(*g);
                    to_judge.extend_from_slice(j);
                    true
                })
            });
            if all_weak {
                found.push(row(N::Raaja(11), &grahas, to_judge));
            }
        }

        // (12): conjoined or in mutual graha drishti.
        let together = |a, b| joins(a, b) || a != b && aspects(a, b) && aspects(b, a);
        if let Some(pair) = first_of(&fifth, &ninth, &together) {
            found.push(row(N::Raaja(12), &pair, Vec::new()));
        }

        // (13): no graha drishti reaches both of two opposite rasis, so each is aspected by either lord (ADR 0014).
        let (fourth, tenth) = (lords_of(4), lords_of(10));
        let exchanged = first_of(&fourth, &tenth, &|f, t| {
            self.rasi_of(f) == house(10) && self.rasi_of(t) == house(4)
        });
        if let Some([f, t]) = exchanged {
            let aspectors = |g| -> Vec<Body> {
                [&fifth[..], &ninth[..]]
                    .concat()
                    .into_iter()
                    .filter(|&a| aspects(a, g))
                    .collect()
            };
            let (on_f, on_t) = (aspectors(f), aspectors(t));
            if !on_f.is_empty() && !on_t.is_empty() {
                found.push(row(
                    N::Raaja(13),
                    &[vec![f, t], on_f, on_t].concat(),
                    Vec::new(),
                ));
            }
        }

        // (14).
        let joiner = [&first[..], &ninth[..]].concat();
        let in_kendra = fifth
            .iter()
            .copied()
            .filter(|&g| matches!(bhava(lagna, self.rasi_of(g)), 1 | 4 | 10))
            .collect::<Vec<_>>();
        if let Some(pair) = first_of(&in_kendra, &joiner, &joins) {
            found.push(row(N::Raaja(14), &pair, Vec::new()));
        }

        // (15): vargottama by the rasi chart and navamsa in any chart read (§11.7.3 note 40); "strong" is to judge (ADR 0014).
        let vargottama = chart.in_varga(Varga::RasiChart).rasi_of(Moon)
            == chart.in_varga(Varga::Navamsa).rasi_of(Moon);
        let on_moon: Vec<Body> = PLANETS
            .into_iter()
            .filter(|&g| g != Moon && aspects(g, Moon))
            .collect();
        if vargottama && on_moon.len() >= 4 {
            found.push(row(
                N::Raaja(15),
                &[vec![Moon], on_moon].concat(),
                vec![ToJudge::Strong(Moon)],
            ));
        }

        // (16): the whole moolatrikona rasi, as Go's (ADR 0014).
        let high: Vec<Body> = PLANETS
            .into_iter()
            .filter(|&g| exalted(g, self.rasi_of(g)) || moolatrikona(g, self.rasi_of(g)))
            .collect();
        if high.len() >= 4 {
            found.push(row(N::Raaja(16), &high, Vec::new()));
        }

        // (17): every benefic in a kendra, a malefic in each of the 3rd, 6th and 11th (ADR 0014).
        found.extend(self.with_mercury_each_way(N::Raaja(17), |mercury_as| {
            let benefics: Vec<Body> = GRAHAS
                .into_iter()
                .filter(|&g| of_nature(g, Saumya, mercury_as))
                .collect();
            let in_kendras = benefics
                .iter()
                .all(|&g| matches!(bhava(lagna, self.rasi_of(g)), 1 | 4 | 7 | 10));
            let malefics = [3, 6, 11].map(|k| of_nature_in(house(k), Kroora, mercury_as));
            (in_kendras && malefics.iter().all(|m| !m.is_empty()))
                .then(|| in_graha_order(&[benefics, malefics.concat()].concat()))
        }));

        // §11.8, from the same karakas.
        let amk = karaka(CharaKaraka::Amatya);
        let eleventh = lords_of(11);
        let on = |a: Body, b: Body| a != b && self.joins_or_aspects(a, self.rasi_of(b));
        let dispositors = |g| self.dispositors_in_yoga(g);
        let sambandha = |n, grahas: &[Body]| row(N::RaajaSambandha(n), grahas, Vec::new());
        let with_benefics = |g: Body, mercury_as| -> Vec<Body> {
            of_nature_in(self.rasi_of(g), Saumya, mercury_as)
                .into_iter()
                .filter(|&b| b != g)
                .collect()
        };

        // (1).
        let amk_side: Vec<Body> = amk
            .iter()
            .flat_map(|&a| [vec![a], dispositors(a)].concat())
            .collect();
        if let Some(pair) = first_of(&tenth, &amk_side, &|t, a| on(a, t)) {
            found.push(sambandha(1, &pair));
        }

        // (2): the 11th lord's own aspect is the one asked for, so he is no malefic on the 11th (ADR 0014).
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(2), |mercury_as| {
                let l = eleventh
                    .iter()
                    .copied()
                    .find(|&l| self.aspects(l, house(11)))?;
                let clear = GRAHAS
                    .into_iter()
                    .filter(|&g| of_nature(g, Kroora, mercury_as))
                    .all(|g| {
                        [10, 11]
                            .iter()
                            .all(|&k| g == l && k == 11 || !self.joins_or_aspects(g, house(k)))
                    });
                clear.then(|| vec![l])
            }),
        );

        // (3)–(6).
        if let Some(pair) = first_of(&ak, &amk, &joins) {
            found.push(sambandha(3, &pair));
        }
        if let Some(&a) = amk.iter().find(|&&a| own_or_uchcha(a)) {
            found.push(row(N::RaajaSambandha(4), &[a], vec![ToJudge::Strong(a)]));
        }
        if let Some(&a) = amk
            .iter()
            .find(|&&a| matches!(bhava(lagna, self.rasi_of(a)), 1 | 5 | 9))
        {
            found.push(sambandha(5, &[a]));
        }
        let kendra_or_trine = |from, r| matches!(bhava(from, r), 1 | 4 | 5 | 7 | 9 | 10);
        if let Some(pair) = first_of(&ak, &amk, &|a, m| {
            kendra_or_trine(self.rasi_of(a), self.rasi_of(m))
        }) {
            found.push(sambandha(6, &pair));
        }

        // (7): each of the 3rd and 6th from each holds a malefic, as (4)'s houses (ADR 0014).
        let arudhas = self.bhava_arudhas();
        let al = arudhas[0];
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(7), |mercury_as| {
                ak.iter().find_map(|&a| {
                    let held: Vec<Vec<Body>> = [lagna, al, self.rasi_of(a)]
                        .iter()
                        .flat_map(|&r| [3, 6].map(|k| of_nature_in(kth(r, k), Kroora, mercury_as)))
                        .collect();
                    held.iter()
                        .all(|h| !h.is_empty())
                        .then(|| in_graha_order(&[vec![a], held.concat()].concat()))
                })
            }),
        );

        // (8).
        let ak_placed = |a: Body| own_or_uchcha(a) && kendra_or_trine(lagna, self.rasi_of(a));
        let placed: Vec<Body> = ak.iter().copied().filter(|&a| ak_placed(a)).collect();
        if let Some(pair) = first_of(&placed, &ninth, &on) {
            found.push(sambandha(8, &pair));
        }

        // (9), (10): AK joined by a benefic other than himself.
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(9), |mercury_as| {
                ak.iter().find_map(|&a| {
                    let benefics = with_benefics(a, mercury_as);
                    (dispositors(Moon).contains(&a)
                        && self.rasi_of(a) == lagna
                        && !benefics.is_empty())
                    .then(|| in_graha_order(&[vec![Moon, a], benefics].concat()))
                })
            }),
        );
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(10), |mercury_as| {
                ak.iter().find_map(|&a| {
                    let benefics = with_benefics(a, mercury_as);
                    (matches!(bhava(lagna, self.rasi_of(a)), 5 | 7 | 9 | 10)
                        && !benefics.is_empty())
                    .then(|| in_graha_order(&[vec![a], benefics].concat()))
                })
            }),
        );

        // (11): A9 is a rasi, so the form says which half held.
        let a9_in_lagna = arudhas[8] == lagna;
        let ak_in_9th: Vec<Body> = ak
            .iter()
            .copied()
            .filter(|&a| self.rasi_of(a) == house(9))
            .take(1)
            .collect();
        let form = form_of(a9_in_lagna, !ak_in_9th.is_empty());
        if form.is_some() {
            found.push(Yoga {
                form,
                ..sambandha(11, &ak_in_9th)
            });
        }

        // (12): no malefic's graha drishti on the 11th lord.
        found.extend(
            self.with_mercury_each_way(N::RaajaSambandha(12), |mercury_as| {
                let unaspected = |l: Body| {
                    !GRAHAS
                        .into_iter()
                        .any(|g| g != l && of_nature(g, Kroora, mercury_as) && aspects(g, l))
                };
                let l = eleventh
                    .iter()
                    .copied()
                    .find(|&l| self.rasi_of(l) == house(11) && unaspected(l))?;
                ak.iter().find_map(|&a| {
                    let benefics = with_benefics(a, mercury_as);
                    (!benefics.is_empty()).then(|| in_graha_order(&[vec![l, a], benefics].concat()))
                })
            }),
        );

        // (13)–(15).
        if let Some(pair) = first_of(&first, &tenth, &|l, t| {
            self.rasi_of(l) == house(10) && self.rasi_of(t) == lagna
        }) {
            found.push(sambandha(13, &pair));
        }
        let fourth_from = |a: Body| kth(self.rasi_of(a), 4);
        if let Some(&a) = ak.iter().find(|&&a| {
            self.rasi_of(Moon) == fourth_from(a) && self.rasi_of(Venus) == fourth_from(a)
        }) {
            found.push(sambandha(14, &[a, Moon, Venus]));
        }
        let first_or_ak = [&first[..], &ak[..]].concat();
        if let Some(pair) = first_of(&first_or_ak, &fifth, &|x, f| {
            joins(x, f) && kendra_or_trine(lagna, self.rasi_of(f))
        }) {
            found.push(sambandha(15, &pair));
        }

        found
    }
}

#[cfg(test)]
mod tests;

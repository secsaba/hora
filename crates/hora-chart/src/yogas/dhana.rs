use super::*;

/// A graha of a Dhana combination and those it names beside him (§11.9).
type Combination = (Body, &'static [Body]);

impl VargaChakra<'_> {
    /// §11.9, Dhana yogas, from `d`'s lagna. "Conjoined or aspected" is by graha drishti, and "by X and Y" is by each (ADR 0014).
    pub(crate) fn dhana_yogas(&self) -> Vec<Yoga> {
        use Body::*;
        let lagna = self.rasi_of(Lagna);
        let in_house = |g, k| self.rasi_of(g) == kth(lagna, k);
        let row = |name, form, grahas: &[Body]| Yoga {
            name,
            grahas: self.yoga_grahas(name, &in_graha_order(grahas)),
            to_judge: Vec::new(),
            form,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        let mut found = Vec::new();

        let exalted_2nd: Vec<Body> = [Moon, Mercury, Jupiter, Venus]
            .into_iter()
            .filter(|&g| in_house(g, 2) && exalted(g, self.rasi_of(g)))
            .collect();
        if !exalted_2nd.is_empty() {
            found.push(row(YogaName::BasicDhana, None, &exalted_2nd));
        }

        // Per lagna from Aries: the 5th's graha and the 11th's, then the graha in the lagna and those joining or aspecting him.
        let (first, second): (Option<Combination>, Combination) = match lagna {
            Rasi::Aries => (
                Some((Sun, &[Saturn, Moon, Jupiter])),
                (Mars, &[Mercury, Venus, Saturn]),
            ),
            Rasi::Taurus => (
                Some((Mercury, &[Moon, Mars, Jupiter])),
                (Venus, &[Mercury, Saturn]),
            ),
            Rasi::Gemini => (Some((Venus, &[Mars])), (Mercury, &[Jupiter, Saturn])),
            Rasi::Cancer => (Some((Mars, &[Venus])), (Moon, &[Mercury, Jupiter])),
            Rasi::Leo => (Some((Jupiter, &[Mercury])), (Sun, &[Mars, Jupiter])),
            Rasi::Virgo => (Some((Saturn, &[Sun, Moon])), (Mercury, &[Jupiter, Saturn])),
            Rasi::Libra => (Some((Saturn, &[Sun, Moon])), (Venus, &[Mercury, Saturn])),
            Rasi::Scorpio => (
                Some((Jupiter, &[Mercury])),
                (Mars, &[Mercury, Venus, Saturn]),
            ),
            Rasi::Sagittarius => (Some((Mars, &[Venus])), (Jupiter, &[Mars, Mercury])),
            Rasi::Capricorn => (Some((Venus, &[Mars])), (Saturn, &[Mars, Jupiter])),
            Rasi::Aquarius => (
                Some((Mercury, &[Moon, Mars, Jupiter])),
                (Saturn, &[Mars, Jupiter]),
            ),
            // The first, the Moon in both the 5th and 11th, is impossible as printed, so is not computed (ADR 0014).
            Rasi::Pisces => (None, (Jupiter, &[Mars, Mercury])),
        };
        let first = first
            .filter(|&(fifth, eleventh)| {
                in_house(fifth, 5) && eleventh.iter().all(|&g| in_house(g, 11))
            })
            .map(|(fifth, eleventh)| [&[fifth], eleventh].concat());
        let (lord, by) = second;
        let second = (in_house(lord, 1) && by.iter().all(|&g| self.joins_or_aspects(g, lagna)))
            .then(|| [&[lord], by].concat());
        let form = form_of(first.is_some(), second.is_some());
        if form.is_some() {
            let grahas = [first, second]
                .into_iter()
                .flatten()
                .flatten()
                .collect::<Vec<_>>();
            found.push(row(YogaName::Dhana(lagna as u8 + 1), form, &grahas));
        }

        found
    }

    /// §11.10, Daridra yogas, from `d`'s lagna: marakas are §14.2's, read each way for a tied Mercury, and "conjoined or aspected"
    /// relates two grahas, by graha drishti (ADR 0014). (10) is not computed.
    pub(crate) fn daridra_yogas(&self) -> Vec<Yoga> {
        let (d, chart) = (self.d, self.chart);
        use Body::*;
        use Disposition::{Kroora, Saumya};
        use YogaName as N;

        let lagna = self.rasi_of(Lagna);
        let house = |k| kth(lagna, k);
        let lords_of = |k| self.lords_in_yoga(house(k));
        let marakas_as = |mercury_as| -> Vec<Body> {
            self.maraka_reasons_as(mercury_as)
                .into_iter()
                .map(|m| m.graha)
                .collect()
        };
        let (saumya_marakas, kroora_marakas) = (marakas_as(Saumya), marakas_as(Kroora));
        let marakas_for = |mercury_as| -> &[Body] {
            if mercury_as == Kroora {
                &kroora_marakas
            } else {
                &saumya_marakas
            }
        };
        let dusthana = |r| matches!(bhava(lagna, r), 6 | 8 | 12);
        let aspects = |a: Body, b: Body| self.aspects(a, self.rasi_of(b));
        let on = |a: Body, b: Body| a != b && self.joins_or_aspects(a, self.rasi_of(b));
        let on_from = |g: Body, from: &[Body]| -> Vec<Body> {
            from.iter().copied().filter(|&x| on(x, g)).collect()
        };
        let malefics = |mercury_as| -> Vec<Body> {
            GRAHAS
                .into_iter()
                .filter(|&g| self.natural_nature(g).unwrap_or(mercury_as) == Kroora)
                .collect()
        };
        let dispositors = |v, g| chart.in_varga(v).dispositors_in_yoga(g);
        let row = |name, grahas: &[Body]| Yoga {
            name,
            grahas: self.yoga_grahas(name, &in_graha_order(grahas)),
            to_judge: Vec::new(),
            form: None,
            magnitude: None,
            afflictions: Vec::new(),
            notes: Vec::new(),
        };
        let first = lords_of(1);
        let mut found = Vec::new();

        // (1), (2): each of the two conjoined or aspected by a maraka other than himself (ADR 0014).
        for (n, k) in [(1, 12), (2, 6)] {
            let other = lords_of(k);
            found.extend(self.with_mercury_each_way(N::Daridra(n), |mercury_as| {
                let marakas = marakas_for(mercury_as);
                first.iter().find_map(|&l| {
                    other.iter().find_map(|&o| {
                        let (on_l, on_o) = (on_from(l, marakas), on_from(o, marakas));
                        (self.rasi_of(l) == house(k)
                            && self.rasi_of(o) == lagna
                            && !on_l.is_empty()
                            && !on_o.is_empty())
                        .then(|| in_graha_order(&[vec![l, o], on_l, on_o].concat()))
                    })
                })
            }));
        }

        // (3).
        found.extend(self.with_mercury_each_way(N::Daridra(3), |mercury_as| {
            let ketu = self.rasi_of(Ketu);
            let moon: &[Body] = if ketu == self.rasi_of(Moon) {
                &[Moon]
            } else {
                &[]
            };
            if ketu != lagna && moon.is_empty() {
                return None;
            }
            let marakas = marakas_for(mercury_as);
            first.iter().find_map(|&l| {
                let on_l = on_from(l, marakas);
                (self.rasi_of(l) == house(8) && !on_l.is_empty())
                    .then(|| in_graha_order(&[vec![Ketu, l], moon.to_vec(), on_l].concat()))
            })
        }));

        // (4): an enemy's rasi as Raaja (11)'s, the compound relationship with its seven-planet lord (ADR 0014).
        let second = lords_of(2);
        let low = second.iter().copied().find(|&s| {
            debilitated(s, self.rasi_of(s))
                || matches!(
                    self.with_rasi_lord(s),
                    Some(Compound::Satru | Compound::Adhisatru)
                )
        });
        if let Some(s) = low {
            found.extend(self.with_mercury_each_way(N::Daridra(4), |mercury_as| {
                let malefics = malefics(mercury_as);
                first.iter().find_map(|&l| {
                    let with: Vec<Body> = malefics
                        .iter()
                        .copied()
                        .filter(|&m| m != l && self.rasi_of(m) == self.rasi_of(l))
                        .collect();
                    (dusthana(self.rasi_of(l)) && !with.is_empty())
                        .then(|| in_graha_order(&[vec![l, s], with].concat()))
                })
            }));
        }

        // (5): aspects only, each lord by a maraka.
        let (fifth, ninth) = (lords_of(5), lords_of(9));
        found.extend(self.with_mercury_each_way(N::Daridra(5), |mercury_as| {
            let marakas = marakas_for(mercury_as);
            let aspecting = |g| -> Vec<Body> {
                marakas
                    .iter()
                    .copied()
                    .filter(|&m| m != g && aspects(m, g))
                    .collect()
            };
            fifth.iter().find_map(|&f| {
                ninth.iter().find_map(|&n| {
                    let (on_f, on_n) = (aspecting(f), aspecting(n));
                    (self.rasi_of(f) == house(6)
                        && self.rasi_of(n) == house(12)
                        && !on_f.is_empty()
                        && !on_n.is_empty())
                    .then(|| in_graha_order(&[vec![f, n], on_f, on_n].concat()))
                })
            })
        }));

        // (6): a maraka other than the malefic it conjoins or aspects (ADR 0014).
        let ninth_or_tenth_in = [&ninth[..], &lords_of(10)[..]]
            .concat()
            .iter()
            .any(|&g| self.rasi_of(g) == lagna);
        if !ninth_or_tenth_in {
            found.extend(self.with_mercury_each_way(N::Daridra(6), |mercury_as| {
                let marakas = marakas_for(mercury_as);
                let in_lagna: Vec<Body> = malefics(mercury_as)
                    .into_iter()
                    .filter(|&g| self.rasi_of(g) == lagna)
                    .collect();
                let on_them: Vec<Body> = marakas
                    .iter()
                    .copied()
                    .filter(|&m| in_lagna.iter().any(|&g| on(m, g)))
                    .collect();
                (!on_them.is_empty()).then(|| in_graha_order(&[in_lagna, on_them].concat()))
            }));
        }

        // (7): each dusthana lord's dispositor in some dusthana, with a malefic on him (ADR 0014).
        found.extend(self.with_mercury_each_way(N::Daridra(7), |mercury_as| {
            let malefics = malefics(mercury_as);
            let mut grahas = Vec::new();
            for k in [6, 8, 12] {
                let afflicted = lords_of(k).into_iter().find_map(|l| {
                    dispositors(d, l).into_iter().find_map(|g| {
                        let on_g = on_from(g, &malefics);
                        (dusthana(self.rasi_of(g)) && !on_g.is_empty())
                            .then(|| [vec![g], on_g].concat())
                    })
                })?;
                grahas.extend(afflicted);
            }
            Some(in_graha_order(&grahas))
        }));

        // (8), (9): rasi-only, since they read the navamsa (ADR 0014).
        if d == Varga::RasiChart {
            found.extend(self.with_mercury_each_way(N::Daridra(8), |mercury_as| {
                let marakas = marakas_for(mercury_as);
                dispositors(Varga::Navamsa, Moon).into_iter().find_map(|g| {
                    let with: Vec<Body> = marakas
                        .iter()
                        .copied()
                        .filter(|&m| m != g && self.rasi_of(m) == self.rasi_of(g))
                        .collect();
                    (!with.is_empty() || matches!(bhava(lagna, self.rasi_of(g)), 2 | 7))
                        .then(|| in_graha_order(&[vec![Moon, g], with].concat()))
                })
            }));
            let navamsa_first = chart
                .in_varga(Varga::Navamsa)
                .lords_in_yoga(chart.in_varga(Varga::Navamsa).rasi_of(Lagna));
            found.extend(self.with_mercury_each_way(N::Daridra(9), |mercury_as| {
                let marakas = marakas_for(mercury_as);
                first.iter().find_map(|&l| {
                    navamsa_first.iter().find_map(|&n| {
                        let (on_l, on_n) = (on_from(l, marakas), on_from(n, marakas));
                        (!on_l.is_empty() && !on_n.is_empty())
                            .then(|| in_graha_order(&[vec![l, n], on_l, on_n].concat()))
                    })
                })
            }));
        }

        // (11): planets joined by a dusthana lord and untouched by the 1st, 5th and 9th lords (ADR 0014).
        let dusthana_lords = [lords_of(6), lords_of(8), lords_of(12)].concat();
        let trine_lords = [first.clone(), fifth, ninth].concat();
        let mut losing = Vec::new();
        for g in PLANETS {
            let joined: Vec<Body> = dusthana_lords
                .iter()
                .copied()
                .filter(|&l| l != g && self.rasi_of(l) == self.rasi_of(g))
                .collect();
            if !joined.is_empty() && on_from(g, &trine_lords).is_empty() {
                losing.push(g);
                losing.extend(joined);
            }
        }
        if !losing.is_empty() {
            found.push(row(N::Daridra(11), &losing));
        }

        // (12): Mercury's only aspect is his 7th; with it the yoga is absent, and the knowledge base gives its wealth instead.
        if self.rasi_of(Mars) == house(2)
            && self.rasi_of(Saturn) == house(2)
            && !self.aspects(Mercury, house(2))
        {
            found.push(row(N::Daridra(12), &[Mars, Saturn]));
        }
        // (13): likewise without Saturn's aspect.
        if self.rasi_of(Sun) == house(2) && aspects(Saturn, Sun) {
            found.push(row(N::Daridra(13), &[Sun, Saturn]));
        }

        found
    }
}

#[cfg(test)]
mod tests;

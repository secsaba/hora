use super::*;

/// One row of the chart table, as message ids and text.
pub(crate) struct Row {
    /// Message ids; `name` is one of a graha, the lagna or an upagraha.
    pub(crate) name: String,
    pub(crate) rasi: &'static str,
    /// 1 to 12, counted from the reference point's rasi.
    pub(crate) bhava: u8,
    /// Degree, arc-minute and arc-second within the rasi.
    pub(crate) position: String,
    pub(crate) retrograde: bool,
    pub(crate) nakshatra: &'static str,
    /// 1 to 27, from Aswini.
    pub(crate) nakshatra_number: u8,
    pub(crate) pada: u8,
    pub(crate) lord: &'static str,
    /// `dignity-…`, none outside the graha's dignity rasis.
    pub(crate) dignity: Option<String>,
    pub(crate) rasi_lord: &'static str,
    /// `relationship-…`, the graha's compound relationship with `rasi_lord`.
    pub(crate) relationship: Option<String>,
    /// The graha's chara karaka (ch. 8 §8.2); none for Ketu and every point that is no graha.
    pub(crate) karaka: Option<CharaKaraka>,
    /// In the rasi chart and the varga shown, from the reference point: `functional-…` and whether it is the yogakaraka
    /// (ch. 13 §13.2, Table 30); none for the nodes and every point that is no graha.
    pub(crate) functional: [Option<(String, bool)>; 2],
}

/// The row of the point `name` at `longitude`, its bhava counted from `reference_rasi`; no R, dignity or relationship.
pub(crate) fn point_row(name: String, longitude: f64, reference_rasi: Rasi) -> Row {
    let (rasi, degrees) = hora_chart::rasi(longitude);
    let (nakshatra, pada) = hora_chart::nakshatra(longitude);
    Row {
        name,
        rasi: rasi_id(rasi),
        bhava: hora_chart::bhava(reference_rasi, rasi),
        position: dms(degrees),
        retrograde: false,
        nakshatra: nakshatra_id(nakshatra),
        nakshatra_number: nakshatra as u8 + 1,
        pada,
        lord: body_id(hora_chart::vimsottari_lord(nakshatra)),
        dignity: None,
        rasi_lord: body_id(rasi.lord()),
        relationship: None,
        karaka: None,
        functional: [None, None],
    }
}

/// The table rows of `chart`, in its order, with `karakas`, bhavas counted from `reference_rasi` and functional nature from
/// `reference` in the rasi chart and `varga`.
pub(crate) fn rows(
    chart: &hora_chart::Chart,
    karakas: &[(Body, CharaKaraka)],
    reference_rasi: Rasi,
    reference: ReferencePoint,
    varga: Varga,
) -> Vec<Row> {
    chart
        .entries()
        .iter()
        .map(|entry| Row {
            functional: [Varga::RasiChart, varga].map(|d| {
                let v = chart.in_varga(d);
                let nature = v.functional_nature(entry.body, reference)?;
                let yogakaraka = v.yogakaraka(reference) == Some(entry.body);
                Some((kb::message_id("functional", nature), yogakaraka))
            }),
            retrograde: entry.retrograde,
            dignity: hora_chart::dignity(entry.body, entry.longitude)
                .map(|d| kb::message_id("dignity", d)),
            relationship: chart
                .lord_relationship(entry.body)
                .map(|r| kb::message_id("relationship", r)),
            karaka: karakas.iter().find(|k| k.0 == entry.body).map(|k| k.1),
            ..point_row(body_id(entry.body).into(), entry.longitude, reference_rasi)
        })
        .collect()
}

/// A row of an arudha table: an arudha pada's rasi and bhava in the rasi chart and in the varga chakra shown.
pub(crate) struct ArudhaRow {
    /// `bhava-arudha-…`, its message id, or `arudha-graha-…`, a graha's id after `arudha-`; its table row's and label's `data-body`.
    pub(crate) id: String,
    pub(crate) rasi: &'static str,
    /// 1 to 12, counted from the reference point's rasi in the rasi chart.
    pub(crate) bhava: u8,
    pub(crate) varga_rasi: &'static str,
    /// 1 to 12, counted from the reference point's rasi in the varga.
    pub(crate) varga_bhava: u8,
}

/// A row of an aspects table: what one graha aspects in one chart.
pub(crate) struct AspectRow {
    pub(crate) graha: &'static str,
    pub(crate) graha_drishti: Aspected,
    pub(crate) rasi_drishti: Aspected,
}

/// What a graha aspects by one drishti.
pub(crate) struct Aspected {
    /// (message id, bhava from that chart's reference point), in count order from the graha.
    pub(crate) rasis: Vec<(&'static str, u8)>,
    /// The grahas in those rasis, in the same order, as Exercises 14 and 15 list them.
    pub(crate) grahas: Vec<&'static str>,
}

/// What each graha aspects in `d`'s chart, counting bhavas from `reference` (ch. 10 §10.2–10.3).
pub(crate) fn aspect_rows(d: Varga, chart: &hora_chart::Chart, reference: Rasi) -> Vec<AspectRow> {
    let aspected = |rasis: Vec<Rasi>| Aspected {
        grahas: rasis
            .iter()
            .flat_map(|&rasi| chart.in_varga(d).grahas_in(rasi))
            .map(body_id)
            .collect(),
        rasis: rasis
            .iter()
            .map(|&rasi| (rasi_id(rasi), hora_chart::bhava(reference, rasi)))
            .collect(),
    };
    chart
        .grahas()
        .iter()
        .map(|e| {
            let at = hora_chart::varga(d, e.longitude);
            AspectRow {
                graha: body_id(e.body),
                graha_drishti: aspected(hora_chart::graha_drishti(e.body, at)),
                rasi_drishti: aspected(hora_chart::rasi_drishtis(at)),
            }
        })
        .collect()
}

/// A row of an argala table: one rasi in one chart.
pub(crate) struct ArgalaRow {
    /// 1 to 12, from that chart's reference point.
    pub(crate) bhava: u8,
    pub(crate) rasi: &'static str,
    /// The rasi holds Ketu, so its counts run in reverse (ch. 10 §10.6's note).
    pub(crate) reversed: bool,
    /// (argala, virodhargala) as graha message ids: 2nd/12th, 4th/10th, 5th/9th, 11th/3rd.
    pub(crate) pairs: [(Vec<&'static str>, Vec<&'static str>); 4],
}

/// The argalas on every rasi in `d`'s chart, the bhava `reference` is in first (ch. 10 §10.5–10.6, ADR 0013).
pub(crate) fn argala_rows(d: Varga, chart: &hora_chart::Chart, reference: Rasi) -> Vec<ArgalaRow> {
    let ids = |grahas: Vec<Body>| grahas.into_iter().map(body_id).collect();
    let mut rasis = hora_chart::RASIS;
    rasis.sort_by_key(|&rasi| hora_chart::bhava(reference, rasi));
    rasis
        .map(|rasi| ArgalaRow {
            bhava: hora_chart::bhava(reference, rasi),
            rasi: rasi_id(rasi),
            reversed: chart.in_varga(d).argala_reversed(rasi),
            pairs: chart
                .in_varga(d)
                .argala(rasi)
                .map(|argala| (ids(argala.argala), ids(argala.virodhargala))),
        })
        .into()
}

/// A row of a baadhakas table: one bhava in one chart (ch. 13 §13.3).
pub(crate) struct BaadhakaRow {
    /// 1 to 12, from that chart's reference point.
    pub(crate) bhava: u8,
    pub(crate) rasi: &'static str,
    /// The bhava arudhas in it, as `bhava-arudha-…` ids (p. 170's "baadhaka from A3").
    pub(crate) arudhas: Vec<String>,
    /// Its baadhaka sthaana's rasi and bhava.
    pub(crate) sthaana: (&'static str, u8),
    pub(crate) baadhakas: Vec<&'static str>,
    /// The grahas in the baadhaka sthaana (p. 168's "planets in Sg").
    pub(crate) grahas: Vec<&'static str>,
}

/// Every bhava's baadhaka sthaana and baadhakas in `d`'s chart, the bhava `reference` is in first (ch. 13 §13.3, Table 31).
pub(crate) fn baadhaka_rows(
    d: Varga,
    chart: &hora_chart::Chart,
    reference: Rasi,
) -> Vec<BaadhakaRow> {
    let v = chart.in_varga(d);
    let arudhas = v.bhava_arudhas();
    let mut rasis = hora_chart::RASIS;
    rasis.sort_by_key(|&rasi| hora_chart::bhava(reference, rasi));
    rasis
        .map(|rasi| {
            let sthaana = hora_chart::baadhaka_sthaana(rasi);
            BaadhakaRow {
                bhava: hora_chart::bhava(reference, rasi),
                rasi: rasi_id(rasi),
                arudhas: (0..12)
                    .filter(|&n| arudhas[n] == rasi)
                    .map(arudha_id)
                    .collect(),
                sthaana: (rasi_id(sthaana), hora_chart::bhava(reference, sthaana)),
                baadhakas: hora_chart::baadhakas(rasi)
                    .into_iter()
                    .map(body_id)
                    .collect(),
                grahas: v.grahas_in(sthaana).into_iter().map(body_id).collect(),
            }
        })
        .into()
}

/// A row of a yogadas table: one yogada in one chart (ch. 13 §13.2).
pub(crate) struct YogadaRow {
    pub(crate) graha: &'static str,
    /// `yogada-gives-…`: money (HL) or power (GL).
    pub(crate) gives: String,
    /// Its links to the lagna, then to HL or GL, in words.
    pub(crate) links: [Vec<String>; 2],
}

/// The yogadas of `d`'s chart, from its own lagna, HL and GL; none without a Vedic day (ch. 13 §13.2, ADR 0016).
pub(crate) fn yogada_rows(d: Varga, chart: &hora_chart::Chart) -> Vec<YogadaRow> {
    chart
        .in_varga(d)
        .yogadas()
        .into_iter()
        .map(|y| YogadaRow {
            graha: body_id(y.graha),
            gives: kb::message_id("yogada-gives", y.gives),
            links: y.links.map(|links| {
                links
                    .into_iter()
                    .map(|l| kb::message_id("yogada-link", l))
                    .collect()
            }),
        })
        .collect()
}

/// The maraka grahas of `d`'s chart, counted from its lagna, each with its reasons in words (ch. 14 §14.2, ADR 0017).
pub(crate) fn maraka_rows(
    lang: Lang,
    d: Varga,
    chart: &hora_chart::Chart,
) -> Vec<(&'static str, Vec<String>)> {
    use hora_chart::MarakaReason as R;
    let graha = |g| LOCALES.lookup(&lang.id(), body_id(g));
    chart
        .in_varga(d)
        .maraka_reasons()
        .into_iter()
        .map(|m| {
            let reasons = m.reasons.into_iter().map(|reason| {
                let (id, house, lord) = match reason {
                    R::Owns(h) => ("maraka-owns", h, None),
                    R::Occupies(h) => ("maraka-occupies", h, None),
                    R::Aspects(h) => ("maraka-aspects", h, None),
                    R::ConjoinsLord(h, l) => ("maraka-conjoins-lord", h, Some(l)),
                    R::AspectsLord(h, l) => ("maraka-aspects-lord", h, Some(l)),
                };
                let lord = lord.map_or(String::new(), graha);
                message(lang, id, [("house", house.to_string()), ("lord", lord)])
            });
            (body_id(m.graha), reasons.collect())
        })
        .collect()
}

/// A row of the three pairs table (ch. 14 §14.4).
pub(crate) struct PairRow {
    /// Which pair, in words.
    pub(crate) label: String,
    /// Its two rasis, each with its `mobility-…` id.
    pub(crate) rasis: [(&'static str, String); 2],
    /// Its `longevity-…` id.
    pub(crate) longevity: String,
}

/// What the Longevity topic shows of the rasi chart, counted from the lagna whatever the reference point (ch. 14, ADR 0017).
pub(crate) struct LongevityView {
    /// Rudra's candidates, the 8th from the lagna and from the 7th: (that rasi, its lord).
    pub(crate) candidates: [(&'static str, &'static str); 2],
    /// Whether both 8ths were counted normally, Saturn being in the lagna or the 7th (p. 276), not by Table 32.
    pub(crate) counted_normally: bool,
    pub(crate) rudra: &'static str,
    /// The `rudra-rule-…` id of the rule that decided, `rudra-rule-tie` if none did.
    pub(crate) decided_by: String,
    pub(crate) weaker: &'static str,
    /// Whether the weaker is afflicted, and so Rudra too.
    pub(crate) weaker_afflicted: bool,
    pub(crate) trishoola: [&'static str; 3],
    pub(crate) maheswara: &'static str,
    /// The `maheswara-exception-…` ids of the exceptions applied, in the order applied.
    pub(crate) maheswara_exceptions: Vec<String>,
    /// The three pairs; the third only with a Vedic day.
    pub(crate) pairs: Vec<PairRow>,
    /// The combined category in words with its years, and the `pairs-rule-…` id; none without the third pair.
    pub(crate) result: Option<(String, String)>,
    /// The paramaayush in words, or why there is none.
    pub(crate) paramaayush: String,
    /// The eighth lord method's reference rasi, 8th lord, his house from it, and the category in words with its years.
    pub(crate) eighth_lord: (&'static str, &'static str, u8, String),
}

impl LongevityView {
    pub(crate) fn new(lang: Lang, chart: &hora_chart::Chart) -> Self {
        let t = |id: &str| LOCALES.lookup(&lang.id(), id);
        let category = |l: hora_chart::Longevity| {
            let (from, to) = l.years();
            let years = message(
                lang,
                "longevity-years",
                [("from", from.to_string()), ("to", to.to_string())],
            );
            format!("{} ({years})", t(&kb::message_id("longevity", l)))
        };
        let d1 = chart.in_varga(Varga::RasiChart);
        let rudra = d1.rudra();
        let maheswara = d1.maheswara();
        let three = d1.three_pairs();
        let eighth = d1.eighth_lord_method();
        let row = |label: String, pair: hora_chart::Pair| PairRow {
            label,
            rasis: pair
                .rasis
                .map(|r| (rasi_id(r), kb::message_id("mobility", r.mobility()))),
            longevity: kb::message_id("longevity", pair.longevity),
        };
        let [lagna_lord, eighth_lord] = three.lords.map(|g| t(body_id(g)));
        let mut pairs = vec![
            row(
                message(
                    lang,
                    "pair-lords",
                    [("lagna_lord", lagna_lord), ("eighth_lord", eighth_lord)],
                ),
                three.pairs[0],
            ),
            row(t("pair-moon-saturn"), three.pairs[1]),
        ];
        pairs.extend(three.hl_pair.map(|p| row(t("pair-lagna-hl"), p)));
        LongevityView {
            candidates: rudra
                .eighths
                .map(|r| (rasi_id(r), body_id(d1.stronger_lord(r)))),
            counted_normally: rudra.counted_normally,
            rudra: body_id(rudra.rudra),
            decided_by: rudra.decided_by.map_or("rudra-rule-tie".to_owned(), |r| {
                kb::message_id("rudra-rule", r)
            }),
            weaker: body_id(rudra.weaker),
            weaker_afflicted: rudra.weaker_afflicted,
            trishoola: rudra.trishoola.map(rasi_id),
            maheswara: body_id(maheswara.graha),
            maheswara_exceptions: maheswara
                .exceptions
                .into_iter()
                .map(|e| kb::message_id("maheswara-exception", e))
                .collect(),
            pairs,
            result: three
                .result
                .map(|(l, rule)| (category(l), kb::message_id("pairs-rule", rule))),
            paramaayush: three.paramaayush.map_or(t("paramaayush-none"), |years| {
                message(lang, "paramaayush-years", [("years", years.to_string())])
            }),
            eighth_lord: (
                rasi_id(eighth.reference),
                body_id(eighth.lord),
                eighth.house,
                category(eighth.longevity),
            ),
        }
    }
}

/// A graha's row of the avasthas table, each state by its message id (ch. 15 §15.4).
pub(crate) struct AvasthaRow {
    pub(crate) graha: &'static str,
    /// `age-avastha-…`; its share of results is `…-share`.
    pub(crate) age: String,
    /// None for a node with no relationship to read (ADR 0018).
    pub(crate) alertness: Option<String>,
    pub(crate) moods: Vec<String>,
    /// The Sayanaadi avastha and its strength: None without a Vedic day; the strength None without the name sound.
    pub(crate) sayanaadi: Option<(String, Option<String>)>,
}

/// What the Strength topic shows of the rasi chart, counted from the lagna whatever the reference point (ch. 15, ADR 0018).
pub(crate) struct StrengthView {
    /// The Sun to Ketu.
    pub(crate) avasthas: Vec<AvasthaRow>,
    /// Scorpio's and Aquarius's stronger lord with the `stronger-rule-…` id that decided (ch. 15 §15.5.1).
    pub(crate) stronger_lords: [(&'static str, &'static str, String); 2],
    /// The seven in §8.3's order: `kin-…`, the graha, and for the father and mother the `stronger-rule-…` id that decided.
    pub(crate) sthira_karakas: Vec<(String, &'static str, Option<String>)>,
}

impl StrengthView {
    pub(crate) fn new(chart: &hora_chart::Chart, sky: &Sky, sound: Option<u8>) -> Self {
        let sayanaadi = hora_chart::sayanaadi_avasthas(sky, sound);
        let d1 = chart.in_varga(Varga::RasiChart);
        let rule = |r: Option<hora_chart::StrongerRule>| {
            r.map_or("stronger-rule-tie".to_owned(), |r| {
                kb::message_id("stronger-rule", r)
            })
        };
        StrengthView {
            avasthas: chart
                .avasthas()
                .into_iter()
                .map(|a| AvasthaRow {
                    graha: body_id(a.graha),
                    age: kb::message_id("age-avastha", a.age),
                    alertness: a.alertness.map(|l| kb::message_id("alertness", l)),
                    moods: a
                        .moods
                        .into_iter()
                        .map(|m| kb::message_id("mood-avastha", m))
                        .collect(),
                    sayanaadi: sayanaadi
                        .iter()
                        .flatten()
                        .find(|s| s.graha == a.graha)
                        .map(|s| {
                            (
                                kb::message_id("sayanaadi-avastha", s.avastha),
                                s.strength.map(|s| kb::message_id("activity-strength", s)),
                            )
                        }),
                })
                .collect(),
            stronger_lords: [Rasi::Scorpio, Rasi::Aquarius].map(|r| {
                let (graha, decided) = d1.stronger_lord_by(r);
                (rasi_id(r), body_id(graha), rule(decided))
            }),
            sthira_karakas: chart
                .sthira_karakas()
                .into_iter()
                .map(|k| {
                    let decided =
                        matches!(k.kin, hora_chart::Kin::Father | hora_chart::Kin::Mother)
                            .then(|| rule(k.rule));
                    (kb::message_id("kin", k.kin), body_id(k.graha), decided)
                })
                .collect(),
        }
    }
}

/// A yoga class's rows in a yoga table, in the book's order (ch. 11).
pub(crate) struct YogaGroup {
    /// The class's message id.
    pub(crate) class: String,
    /// The class's knowledge base page.
    pub(crate) page: &'static str,
    pub(crate) rows: Vec<YogaRow>,
}

/// A row of a yoga table: one present or to-judge yoga in one chart.
pub(crate) struct YogaRow {
    /// The name as the book spells it, with its form where it has one.
    pub(crate) name: String,
    /// (graha message id, its role in the yoga and the bhava it is in), counted from that chart's lagna: the role is its
    /// karaka or lordship where the row names one (§11.7.3 (1)), else every lordship as a yoga counts them.
    pub(crate) grahas: Vec<(&'static str, String)>,
    /// Each condition left to the reader, in words.
    pub(crate) to_judge: Vec<String>,
    /// A Raaja pair's afflictions (§11.7.2 (1)), then the book's notes (ch. 13 §13.5), in words.
    pub(crate) notes: Vec<String>,
    /// A Raaja pair's closeness, and each graha's (message id, Dasa Varga count, amsa message id) (§11.7.2).
    pub(crate) magnitude: Option<(String, Vec<AmsaCell>)>,
}

/// A graha's message id, its Dasa Varga count, and its amsa's message id from 2 (ch. 6 §6.6).
type AmsaCell = (&'static str, u8, Option<String>);

/// §11.7.2's closeness in arcseconds as degrees and minutes.
fn closeness(arcseconds: u32) -> String {
    // Rounded, unlike positions: chart 6's pair, over 16′30″ apart, is the book's "17'" (p. 230), as hora-chart's golden test reads it.
    let minutes = (arcseconds + 30) / 60;
    format!("{}°{:02}′", minutes / 60, minutes % 60)
}

/// The yogas of `d`'s chart grouped by class, each graha's bhava counted from that chart's lagna, never the reference point
/// (ch. 11, ADR 0014).
pub(crate) fn yoga_groups(lang: Lang, d: Varga, chart: &hora_chart::Chart) -> Vec<YogaGroup> {
    use hora_chart::{Note, ToJudge, YogaName as N};
    let mut groups: Vec<YogaGroup> = Vec::new();
    for yoga in chart.in_varga(d).yogas() {
        let class = yoga.name.class();
        let name = match yoga.name {
            N::Raaja(n) => message(lang, "yoga-raaja-n", [("number", n.to_string())]),
            N::RaajaSambandha(n) => {
                message(lang, "yoga-raaja-sambandha-n", [("number", n.to_string())])
            }
            N::Dhana(n) => message(lang, "yoga-dhana-n", [("number", n.to_string())]),
            N::Daridra(n) => message(lang, "yoga-daridra-n", [("number", n.to_string())]),
            name => LOCALES.lookup(&lang.id(), &kb::message_id("yoga", name)),
        };
        let name = match (yoga.name, yoga.form) {
            // Its grahas say which half held, by their roles below.
            (N::Raaja(1), _) => name,
            (_, Some(3)) => message(lang, "yoga-forms-both", [("name", name)]),
            (_, Some(form)) => message(
                lang,
                "yoga-form",
                [("name", name), ("form", form.to_string())],
            ),
            (_, None) => name,
        };
        let graha = |g| LOCALES.lookup(&lang.id(), body_id(g));
        let row = YogaRow {
            name,
            grahas: yoga
                .grahas
                .iter()
                .map(|g| {
                    let karaka = g.karaka.map(|k| {
                        LOCALES.lookup(
                            &lang.id(),
                            &format!("{}-short", kb::message_id("chara-karaka", k)),
                        )
                    });
                    let lords = g
                        .named_lord_of
                        .iter()
                        .map(|n| message(lang, "yoga-lord", [("bhava", n.to_string())]));
                    let roles: Vec<String> = karaka.into_iter().chain(lords).collect();
                    let bhava = g.bhava.to_string();
                    let place = if roles.is_empty() {
                        message(lang, "yoga-graha-in", [("bhava", bhava)])
                    } else {
                        message(
                            lang,
                            "yoga-graha-roles-in",
                            [("roles", roles.join(", ")), ("bhava", bhava)],
                        )
                    };
                    (body_id(g.graha), place)
                })
                .collect(),
            to_judge: yoga
                .to_judge
                .iter()
                .map(|condition| match *condition {
                    ToJudge::MercurysNature => {
                        LOCALES.lookup(&lang.id(), "yoga-to-judge-mercurys-nature")
                    }
                    ToJudge::MayNotOperateWell => {
                        LOCALES.lookup(&lang.id(), "yoga-to-judge-may-not-operate-well")
                    }
                    ToJudge::Strong(g) => {
                        message(lang, "yoga-to-judge-strong", [("graha", graha(g))])
                    }
                    ToJudge::Combust(g) => {
                        message(lang, "yoga-to-judge-combust", [("graha", graha(g))])
                    }
                    ToJudge::DeepExaltation(g) => {
                        message(lang, "yoga-to-judge-deep-exaltation", [("graha", graha(g))])
                    }
                    ToJudge::MaleficInUpachaya(g) => message(
                        lang,
                        "yoga-to-judge-malefic-in-upachaya",
                        [("graha", graha(g))],
                    ),
                    ToJudge::AllInOneRasi => {
                        LOCALES.lookup(&lang.id(), "yoga-to-judge-all-in-one-rasi")
                    }
                })
                .collect(),
            notes: yoga
                .afflictions
                .iter()
                .map(|a| {
                    let (graha, afflicts) = (graha(a.graha), graha(a.afflicts));
                    match a.closeness {
                        Some(c) => message(
                            lang,
                            "yoga-affliction-closeness",
                            [
                                ("graha", graha),
                                ("afflicts", afflicts),
                                ("closeness", closeness(c)),
                            ],
                        ),
                        None => message(
                            lang,
                            "yoga-affliction",
                            [("graha", graha), ("afflicts", afflicts)],
                        ),
                    }
                })
                .chain(yoga.notes.iter().map(|note| match *note {
                    Note::AtmaKaraka(g) => {
                        message(lang, "yoga-note-atma-karaka", [("graha", graha(g))])
                    }
                    Note::InTheEighth(g) => {
                        message(lang, "yoga-note-in-the-eighth", [("graha", graha(g))])
                    }
                }))
                .collect(),
            magnitude: yoga.magnitude.map(|m| {
                (
                    closeness(m.closeness),
                    m.amsas
                        .iter()
                        .map(|&(g, count, amsa)| {
                            (body_id(g), count, amsa.map(|a| kb::message_id("amsa", a)))
                        })
                        .collect(),
                )
            }),
        };
        let class_id = kb::message_id("yoga-class", class);
        match groups.last_mut() {
            Some(group) if group.class == class_id => group.rows.push(row),
            _ => groups.push(YogaGroup {
                class: class_id,
                page: kb::yoga_page(class),
                rows: vec![row],
            }),
        }
    }
    groups
}

/// A count in an ashtakavarga table and the message ids of its marks, each also with `-mark` its visible sign.
type AvCell = (u8, Vec<String>);

/// A PAV's row: a reference's id and whether it gives a rekha in each rasi from Aries (ch. 12 §12.6).
type PavRow = (&'static str, [bool; 12]);

/// One chart's ashtakavarga, counted from its own lagna and positions, never the reference point (ch. 12 §12.5, ADR 0015).
pub(crate) struct Ashtakavarga {
    /// Each rasi's bhava from Aries, counted from the chart's lagna.
    pub(crate) bhavas: [u8; 12],
    /// (row id, cells from Aries): the seven planets' BAVs, then the SAV, then the lagna's BAV (ch. 12 §12.3–12.4).
    pub(crate) bav: Vec<(&'static str, [AvCell; 12])>,
    /// (summary, rows) for each PAV, of the seven planets and the lagna (ch. 12 §12.6).
    pub(crate) pav: Vec<(String, Vec<PavRow>)>,
    /// (graha id, SoAV from Aries) of the seven planets (ch. 12 §12.7).
    pub(crate) soav: Vec<(&'static str, [u8; 12])>,
    /// (graha id, pindas) of the seven planets (ch. 12 §12.7.3).
    pub(crate) pindas: Vec<(&'static str, hora_chart::Pindas)>,
}

impl Ashtakavarga {
    pub(crate) fn new(lang: Lang, d: Varga, chart: &hora_chart::Chart) -> Self {
        use hora_chart::{AV_REFERENCES, PLANETS, RASIS};
        let v = chart.in_varga(d);
        let lagna = v.rasi_of(Body::Lagna);
        let bav_row = |planet: Body| {
            let (bav, at) = (v.bav(planet), v.rasi_of(planet));
            let cells = RASIS.map(|rasi| {
                let count = bav[rasi as usize];
                let band = match hora_chart::bav_band(planet, count) {
                    Some(hora_chart::BavBand::Neutral) | None => None,
                    Some(band) => Some(kb::message_id("av-bav", band)),
                };
                let natal = (rasi == at && planet != Body::Lagna).then(|| "av-natal".to_owned());
                (count, band.into_iter().chain(natal).collect())
            });
            (body_id(planet), cells)
        };
        let sav = v.sav().map(|count| {
            let band = match hora_chart::sav_band(count) {
                hora_chart::SavBand::Average => None,
                band => Some(kb::message_id("av-sav", band)),
            };
            (count, band.into_iter().collect())
        });
        Ashtakavarga {
            bhavas: RASIS.map(|rasi| hora_chart::bhava(lagna, rasi)),
            bav: PLANETS
                .map(bav_row)
                .into_iter()
                .chain([("av-sav", sav), ("av-lagna-bav", bav_row(Body::Lagna).1)])
                .collect(),
            pav: AV_REFERENCES
                .map(|of| {
                    let rows = AV_REFERENCES
                        .map(body_id)
                        .into_iter()
                        .zip(v.pav(of))
                        .collect();
                    let body = LOCALES.lookup(&lang.id(), body_id(of));
                    (message(lang, "av-pav-of", [("body", body)]), rows)
                })
                .into(),
            soav: PLANETS.map(|p| (body_id(p), v.soav(p))).into(),
            pindas: PLANETS.map(|p| (body_id(p), v.pindas(p))).into(),
        }
    }
}

/// The message id of the bhava arudha of the `n`th bhava from 0: AL (0) to UL (11), with `-short` its abbreviation (ch. 9 §9.2).
pub(crate) fn arudha_id(n: usize) -> String {
    format!("bhava-arudha-{}", n + 1)
}

/// A graha's count of good vargas in a varga group, and its amsa's message id from 2 (ch. 6 §6.6).
pub(crate) type AmsabalaCell = (u8, Option<String>);

/// A varga's row of the Vargas table and its link in the varga chakra's link row.
pub(crate) struct VargaRow {
    /// The n of D-n.
    pub(crate) parts: u8,
    /// E.g. "Navamsa (D-9)".
    pub(crate) name: String,
    /// The chart shown with this varga chakra.
    pub(crate) href: String,
    /// The rasi message id of each of the table's columns.
    pub(crate) rasis: Vec<&'static str>,
}

/// A varga's name with its D-n, as the glossary writes it on first use.
pub(crate) fn varga_name(lang: Lang, varga: Varga) -> String {
    let name = LOCALES.lookup(&lang.id(), &kb::message_id("varga", varga));
    message(
        lang,
        "varga-full",
        [("name", name), ("parts", varga.parts().to_string())],
    )
}

/// The page at `uri` with the field `name` set to `value`, at `#fragment`: its query with `name` replaced, as sent.
pub(crate) fn query_href(uri: &Uri, name: &str, value: &str, fragment: &str) -> String {
    let query: String = uri
        .query()
        .unwrap_or("")
        .split('&')
        .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(name))
        .map(|pair| format!("{pair}&"))
        .collect();
    let fragment = if fragment.is_empty() {
        String::new()
    } else {
        format!("#{fragment}")
    };
    format!("{}?{query}{name}={value}{fragment}", uri.path())
}

/// The upagraha table's rows of `sky`, each with its section on the upagraha page, bhavas counted from `reference_rasi` (ch. 4).
pub(crate) fn upagraha_rows(sky: &Sky, reference_rasi: Rasi) -> Vec<(&'static str, Row)> {
    hora_chart::upagrahas(sky)
        .into_iter()
        .map(|(upagraha, longitude)| {
            let group = if upagraha.is_sun_based() {
                "sun-based"
            } else {
                "time-based"
            };
            let name = kb::message_id("upagraha", upagraha);
            (group, point_row(name, longitude, reference_rasi))
        })
        .collect()
}

#[cfg(test)]
mod tests;

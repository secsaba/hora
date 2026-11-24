use super::*;
use crate::web::tests::*;

/// Saturn in Aries is neecha, sama with Mars (ch. 3 §3.3–3.4); the graha-relationships page works its example on this chart.
#[tokio::test]
async fn the_chart_page_shows_each_grahas_dignity_and_rasi_lord_as_the_kb_example_says() {
    let view = view(Lang::En, &format!("{HORAS_CHART}&style=south")).await;
    for (id, dignity, rasi_lord, relationship) in [
        (
            "graha-saturn",
            Some("dignity-neecha"),
            "graha-mars",
            "relationship-sama",
        ),
        (
            "graha-jupiter",
            None,
            "graha-mars",
            "relationship-adhimitra",
        ),
        ("graha-mercury", None, "graha-jupiter", "relationship-satru"),
        ("graha-venus", None, "graha-mars", "relationship-mitra"),
    ] {
        let row = graha(&view, id);
        assert_eq!(
            (
                row.dignity.as_deref(),
                row.rasi_lord,
                row.relationship.as_deref()
            ),
            (dignity, rasi_lord, Some(relationship)),
            "{id}"
        );
    }
    // The lagna, in Scorpio, and Rahu, in Cancer, have no dignity and only their rasi's lord.
    for (point, rasi_lord) in [("lagna", "graha-mars"), ("graha-rahu", "graha-moon")] {
        let row = graha(&view, point);
        assert_eq!(
            (
                row.dignity.as_deref(),
                row.rasi_lord,
                row.relationship.as_deref()
            ),
            (None, rasi_lord, None),
            "{point}"
        );
    }
    // Each links to its knowledge base section.
    for (lang, saturn) in [("en", "Saturn"), ("hu", "Szaturnusz")] {
        let page = body(&format!("/{lang}/chart?{HORAS_CHART}&style=south")).await;
        let kb = format!("/{lang}/kb");
        let cells = [
            format!(">{saturn}<"),
            format!(r#"<td><a href="{kb}/dignity#neecha">neecha</a></td>"#),
            format!(
                r#"<td><a href="{kb}/graha-characteristics#mars">Mars</a> (<a href="{kb}/graha-relationships#compound">sama</a>)</td>"#
            ),
        ];
        let cells = cells.each_ref().map(String::as_str);
        assert!(row(table(&page), &cells).is_some(), "{lang}: {page}");
    }
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/graha-relationships.md")),
        ("hu", include_str!("../../../kb/hu/graha-relationships.md")),
    ] {
        let link = format!("(/{lang}/chart?{HORAS_CHART}&style=south)");
        assert!(source.contains(&link), "{lang}");
        for term in ["neecha", "sama", "adhimitra", "satru"] {
            assert!(source.contains(term), "{lang}: {term}");
        }
    }
}

/// Ch. 4: eleven upagrahas with a Vedic day, only the five Sun-based without one.
#[tokio::test]
async fn the_chart_page_shows_each_upagrahas_rasi_nakshatra_and_lords() {
    let tromso = with(&TROMSO);
    for (lang, query, upagrahas) in [
        ("en", HORAS_CHART, 11),
        ("hu", HORAS_CHART, 11),
        ("en", tromso.as_str(), 5),
    ] {
        let page = body(&format!("/{lang}/chart?{query}&style=south&topic=lagnas")).await;
        let section = section(&page, "upagrahas");
        let kb = format!("/{lang}/kb");
        let rows: Vec<_> = section.split("<tr>").skip(2).collect();
        assert_eq!(rows.len(), upagrahas, "{section}");
        // Only a chart without a Vedic day says why six are missing.
        assert_eq!(section.contains("<p>"), upagrahas == 5, "{section}");
        for (i, row) in rows.iter().enumerate() {
            let group = if hora_chart::UPAGRAHAS[i].is_sun_based() {
                "sun-based"
            } else {
                "time-based"
            };
            for (href, count) in [
                (format!("{kb}/upagraha#{group}"), 1),
                (format!("{kb}/rasi-characteristics#"), 1),
                (format!("{kb}/bhava"), 1),
                (format!("{kb}/nakshatra"), 1),
                // The Vimsottari lord and the rasi lord.
                (format!("{kb}/graha-characteristics#"), 2),
            ] {
                let href = format!(r#"<a href="{href}"#);
                assert_eq!(row.matches(&href).count(), count, "{href}: {row}");
            }
        }
    }
}

/// Ch. 5: HL, GL, SL and BL with a Vedic day, only SL without one.
#[tokio::test]
async fn the_chart_page_shows_each_special_lagnas_rasi_nakshatra_and_lords() {
    let tromso = with(&TROMSO);
    for (lang, query, names) in [
        (
            "en",
            HORAS_CHART,
            &["hora-lagna", "ghati-lagna", "sree-lagna", "bhaava-lagna"][..],
        ),
        (
            "hu",
            HORAS_CHART,
            &["hora-lagna", "ghati-lagna", "sree-lagna", "bhaava-lagna"],
        ),
        ("en", tromso.as_str(), &["sree-lagna"]),
    ] {
        let page = body(&format!("/{lang}/chart?{query}&style=south&topic=lagnas")).await;
        let section = section(&page, "special-lagnas");
        let kb = format!("/{lang}/kb");
        let rows: Vec<_> = section.split("<tr").skip(2).collect();
        assert_eq!(rows.len(), names.len(), "{section}");
        assert_eq!(section.contains("<p>"), names.len() == 1, "{section}");
        for (row, name) in rows.iter().zip(names) {
            for (href, count) in [
                (format!("{kb}/special-lagna#{name}\""), 1),
                (format!("{kb}/rasi-characteristics#"), 1),
                (format!("{kb}/bhava"), 1),
                (format!("{kb}/nakshatra"), 1),
                // The Vimsottari lord and the rasi lord.
                (format!("{kb}/graha-characteristics#"), 2),
            ] {
                let href = format!(r#"<a href="{href}"#);
                assert_eq!(row.matches(&href).count(), count, "{href}: {row}");
            }
        }
    }
}

/// The upagraha page works its example on hora's chart: a Friday night, Saturn's part the 5th, the birth in the Sun's 7th.
#[tokio::test]
async fn the_upagraha_pages_example_is_horas_chart() {
    let view = view(Lang::En, &format!("{HORAS_CHART}&style=south")).await;
    let upagraha =
        |view: &ChartView, name| row_named(view.upagrahas.iter().map(|(_, row)| row), name);
    // Bhavas from the lagna, in Scorpio.
    let examples = [
        ("Dhuma", "Aries", 6, "29°35′59″"),
        ("Indrachaapa", "Libra", 12, "29°35′59″"),
        ("Vyatipaata", "Pisces", 5, "0°24′"),
        ("Parivesha", "Virgo", 11, "0°24′"),
        ("Upaketu", "Scorpio", 1, "16°15′59″"),
        ("Gulika", "Virgo", 11, "13°41′"),
        ("Maandi", "Virgo", 11, "24°20′"),
        ("Kaala", "Scorpio", 1, "6°55′"),
    ];
    for (name, rasi, bhava, position) in examples {
        let row = upagraha(&view, name);
        assert_eq!([&*row[1], &*row[2]], [rasi, &*bhava.to_string()], "{name}");
        assert!(row[3].starts_with(position), "{name}: {row:?}");
    }
    // From the Moon, in Libra, Upaketu in Scorpio is in the 2nd.
    let from_moon = HORAS_CHART.replace("reference=lagna", "reference=moon");
    let moon_view = self::view(Lang::En, &format!("{from_moon}&style=south")).await;
    assert_eq!(upagraha(&moon_view, "Upaketu")[1..3], ["Scorpio", "2"]);
    let sun = row_named(&view.grahas, "Sun");
    assert_eq!(sun[1], "Sagittarius");
    assert!(sun[3].starts_with("16°15′59″"), "{sun:?}");
    let day = view.vedic_day.as_ref().expect("a Vedic day");
    assert!(
        day.iter()
            .any(|(_, time)| time == "15:59:58 (December 31, 1999)")
    );
    assert!(day.iter().any(|(_, time)| time.ends_with("08:05:46")));
    assert_eq!(
        limb(Lang::En, &view, "vaara")
            .map(|(value, _)| value)
            .as_deref(),
        Some("Friday")
    );
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/upagraha.md")),
        ("hu", include_str!("../../../kb/hu/upagraha.md")),
    ] {
        let link = format!("(/{lang}/chart?{HORAS_CHART}&style=south)");
        assert!(source.contains(&link), "{lang}");
        for (_, _, _, position) in examples {
            assert!(source.contains(position), "{lang}: {position}");
        }
        for text in [
            "256°15′59″",
            "389°35′59″",
            "330°24′",
            "15:59:58",
            "08:05:46",
        ] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// An aspects table's rows as the page writes them in English: the graha, then each drishti's rasis and grahas.
fn aspect_texts(rows: &[AspectRow]) -> Vec<[String; 3]> {
    let aspected = |a: &Aspected| match a.rasis.as_slice() {
        [] => "—".to_owned(),
        rasis => {
            let rasis: Vec<_> = rasis
                .iter()
                .map(|(rasi, bhava)| format!("{} ({bhava})", short(rasi)))
                .collect();
            format!("{} / {}", rasis.join(", "), shorts(&a.grahas))
        }
    };
    rows.iter()
        .map(|r| {
            [
                en(r.graha),
                aspected(&r.graha_drishti),
                aspected(&r.rasi_drishti),
            ]
        })
        .collect()
}

/// An argala table's rows as the page writes them in English: bhava, rasi (`*` when reversed), then each pair.
fn argala_texts(rows: &[ArgalaRow]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|r| {
            let mark = if r.reversed { " *" } else { "" };
            let mut cells = vec![r.bhava.to_string(), format!("{}{mark}", en(r.rasi))];
            for (argala, virodhargala) in &r.pairs {
                cells.extend([shorts(argala), shorts(virodhargala)]);
            }
            cells
        })
        .collect()
}

/// A yoga table's rows across its classes as the page writes them in `lang`: name, grahas, conditions to judge, notes,
/// closeness and amsas, "—" for each one missing.
fn yoga_texts(lang: Lang, groups: &[YogaGroup]) -> Vec<Vec<String>> {
    let or_dash = |cells: Vec<String>, by: &str| match cells.is_empty() {
        true => "—".to_owned(),
        false => cells.join(by),
    };
    groups
        .iter()
        .flat_map(|group| &group.rows)
        .map(|row| {
            let grahas = row
                .grahas
                .iter()
                .map(|(id, place)| format!("{} ({place})", t(lang, id)))
                .collect();
            let (closeness, amsas) = row.magnitude.as_ref().map_or_else(
                || ("—".to_owned(), "—".to_owned()),
                |(closeness, amsas)| {
                    let amsas = amsas
                        .iter()
                        .map(|(id, count, amsa)| {
                            let amsa = amsa.as_ref().map_or("—".to_owned(), |a| t(lang, a));
                            format!("{}: {count} {amsa}", t(lang, id))
                        })
                        .collect();
                    (closeness.clone(), or_dash(amsas, " / "))
                },
            );
            vec![
                row.name.clone(),
                or_dash(grahas, ", "),
                or_dash(row.to_judge.clone(), " / "),
                or_dash(row.notes.clone(), " / "),
                closeness,
                amsas,
            ]
        })
        .collect()
}

/// Chart 3 (ch. 9 *Arudha Padas*, p. 97), at the seconds its golden test is cast with.
const CHART_3: &str = "date=1926-12-25&time=05%3A12%3A30&utc=stated&offset=%2B05%3A30\
        &latitude=26.233333333333334&longitude=78.16666666666667";

/// The `<tr` of the body or arudha `id` in `html`.
fn body_row<'a>(html: &'a str, id: &str) -> &'a str {
    let at = format!(r#" data-body="{id}">"#);
    html.split("<tr").find(|r| r.starts_with(&at)).expect(id)
}

#[tokio::test]
async fn the_arudha_tables_show_chart_3s_arudha_padas() {
    // Ch. 20, p. 265: "Cp contains AL and A5"; Example 32, p. 96: Jupiter's and Rahu's arudha padas in Libra. Lagna in Scorpio.
    let view = view(Lang::En, CHART_3).await;
    for id in ["bhava-arudha-1", "bhava-arudha-5"] {
        let row = arudha(&view, id);
        assert_eq!((row.rasi, row.bhava), ("rasi-capricorn", 3), "{id}");
    }
    assert_eq!(view.bhava_arudhas.len(), 12);
    for id in ["arudha-graha-jupiter", "arudha-graha-rahu"] {
        let row = arudha(&view, id);
        assert_eq!((row.rasi, row.bhava), ("rasi-libra", 12), "{id}");
    }
    assert_eq!(view.graha_arudhas.len(), 9);
    let page = body(&format!("/en/chart?{CHART_3}&topic=arudhas")).await;
    let arudha_padas = section(&page, "arudha-padas");
    assert!(
        arudha_padas.contains(r#"<a href="/en/kb/arudha-padas">Arudha padas</a></h2>"#),
        "{arudha_padas}"
    );
    for (name, short) in [
        ("Arudha lagna", "AL"),
        ("Arudha pada of the 5th bhava", "A5"),
        ("Darapada", "A7"),
        ("Upapada lagna", "UL"),
    ] {
        assert!(
            arudha_padas.contains(&format!(r#"<abbr title="{name}">{short}</abbr>"#)),
            "{arudha_padas}"
        );
    }
    let grahas = section(&page, "graha-arudhas");
    assert!(
        grahas.contains(r#"<a href="/en/kb/arudha-padas">Graha arudhas</a></h2>"#),
        "{grahas}"
    );
    assert!(
        grahas.contains(
            r##"<th scope="row"><a href="/en/kb/graha-characteristics#jupiter">Jupiter</a></th>"##
        ),
        "{grahas}"
    );
    // Hover lights the row, which needs its own rule.
    for id in ["bhava-arudha-1", "arudha-graha-rahu"] {
        assert!(page.contains(&format!(r#"<tr data-body="{id}">"#)), "{id}");
        assert!(
            page.contains(&format!("[data-body={id}] {{ --lit: 1; }}")),
            "{id}"
        );
    }
    let hu = body(&format!("/hu/chart?{CHART_3}&topic=arudhas")).await;
    assert!(section(&hu, "arudha-padas").contains("Arudha padák"));
    assert!(
        section(&hu, "arudha-padas")
            .contains(r#"<abbr title="Az 5. bhava arudha padája">A5</abbr>"#)
    );
    assert!(section(&hu, "graha-arudhas").contains("Graha arudhák"));
}

#[tokio::test]
async fn the_arudha_tables_varga_columns_hold_exercises_12_and_13() {
    // Chart 2: chart 1's birth at UTC−5, in D-16 (ch. 9 §9.8, p. 99); its D-16 lagna counts the varga's Bhava with the lagna reference.
    let query = with(&[("offset", "-05%3A00"), ("time", "17%3A55%3A51")]);
    let query = format!("{query}&varga=d16");
    let view = view(Lang::En, &query).await;
    let sky = sky_of(&query).await;
    let lagna = hora_chart::varga(Varga::Shodasamsa, hora_chart::chart(&sky).lagna().longitude);
    let d16 = "Shodasamsa (D-16)";
    assert_eq!(view.varga_name, d16);
    let page = body(&format!("/en/chart?{query}&topic=arudhas")).await;
    for id in ["arudha-padas", "graha-arudhas"] {
        let header = format!(r#"<th scope="colgroup" colspan="2">{d16}</th>"#);
        assert!(section(&page, id).contains(&header), "{id}");
    }
    use Rasi::*;
    for (n, rasi) in [
        Aquarius,
        Aries,
        Sagittarius,
        Scorpio,
        Sagittarius,
        Aquarius,
        Pisces,
        Virgo,
        Taurus,
        Sagittarius,
        Sagittarius,
        Aquarius,
    ]
    .into_iter()
    .enumerate()
    {
        let row = arudha(&view, &arudha_id(n));
        let shown = (rasi_id(rasi), hora_chart::bhava(lagna, rasi));
        assert_eq!((row.varga_rasi, row.varga_bhava), shown, "{}", row.id);
    }
    // The page writes each row's cells as the chakra's rasi and bhava, then the varga's.
    let al = arudha(&view, &arudha_id(0));
    let cell = |rasi: &str, bhava: u8| {
        let slug = rasi.trim_start_matches("rasi-");
        format!(
            r##"<td><a href="/en/kb/rasi-characteristics#{slug}">{}</a></td><td><a href="/en/kb/bhava">{bhava}</a></td>"##,
            en(rasi)
        )
    };
    let cells = cell(al.rasi, al.bhava) + &cell(al.varga_rasi, al.varga_bhava);
    assert!(section(&page, "arudha-padas").contains(&cells), "{cells}");
    use hora_chart::Body::*;
    for (graha, rasi) in [
        (Sun, Scorpio),
        (Moon, Sagittarius),
        (Mars, Gemini),
        (Mercury, Pisces),
        (Jupiter, Scorpio),
        (Venus, Aquarius),
        (Saturn, Sagittarius),
        (Rahu, Virgo),
        (Ketu, Pisces),
    ] {
        let row = arudha(&view, &format!("arudha-{}", body_id(graha)));
        let shown = (rasi_id(rasi), hora_chart::bhava(lagna, rasi));
        assert_eq!((row.varga_rasi, row.varga_bhava), shown, "{}", row.id);
    }
}

/// Chart 5 (ch. 10 *Aspects and Argalas*, p. 101), at the seconds its golden test is cast with.
fn chart_5() -> String {
    with(&[
        ("date", "2000-04-19"),
        ("time", "19%3A21%3A57"),
        ("latitude", "42.666666666666664"),
        ("longitude", "-71.13333333333334"),
    ])
}

/// Chart 6 (ch. 10 *Aspects and Argalas*, p. 107) as the book prints it.
fn chart_6() -> String {
    with(&[
        ("date", "1921-06-28"),
        ("time", "12%3A49"),
        ("offset", "%2B05%3A17"),
        ("latitude", "18.433333333333334"),
        ("longitude", "79.15"),
    ])
}

/// A section's table for the rasi chart and, in its `<details>`, for the varga shown.
fn both_tables(section: &str) -> (&str, &str) {
    section.split_once("<details>").expect("a details")
}

#[tokio::test]
async fn the_aspects_tables_hold_exercises_14_and_15_in_chart_5s_navamsa() {
    let query = format!("{}&varga=d9", chart_5());
    let page = body(&format!("/en/chart?{query}&topic=aspects")).await;
    let aspects = section(&page, "aspects");
    assert!(
        aspects.contains(r#"<a href="/en/kb/aspects-and-argalas">Aspects</a></h2>"#),
        "{aspects}"
    );
    assert!(
        aspects.contains("<details>\n<summary>Navamsa (D-9)</summary>"),
        "{aspects}"
    );
    let view = view(Lang::En, &query).await;
    assert_eq!(view.rasi.aspects.len(), 9);
    let rows = aspect_texts(&view.varga.aspects);
    // Ch. 10 §10.8, pp. 109–110: the rasis and houses in count order, the planets as the book lists them.
    assert_eq!(
        rows[2],
        [
            "Mars",
            "Aq (4), Ta (7), Ge (8) / Ke, Su",
            "Cp (3), Ar (6), Cn (9) / Ve, Mo",
        ]
    );
    assert_eq!(
        rows[4],
        [
            "Jupiter",
            "Cp (3), Pi (5), Ta (7) / Ve, Su",
            "Sg (2), Pi (5), Ge (8) / Me",
        ]
    );
    // Exercise 14 leaves out the nodes (ADR 0013); Exercise 15 lists them.
    assert_eq!(rows[7], ["Rahu", "—", "Li (12), Cp (3), Ar (6) / Ve, Mo"]);
    assert_eq!(rows[8], ["Ketu", "—", "Ar (6), Cn (9), Li (12) / Mo"]);
    // Moon's graha drishti falls on Libra, which holds no graha.
    assert_eq!(rows[1][1], "Li (12) / —");
    assert!(!aspects.contains("Lagna"), "{aspects}");
    // Hover lights the chakra and the grahas table through the rules their rows already have.
    assert!(
        aspects.contains(r#"<tr data-body="graha-mars">"#),
        "{aspects}"
    );
    assert!(aspects.contains(r#"<th scope="col">Graha drishti</th>"#));
    assert!(aspects.contains(r#"<th scope="col">Rasi drishti</th>"#));
    let hu = body(&format!("/hu/chart?{query}&topic=aspects")).await;
    assert!(section(&hu, "aspects").contains("Aspektusok"));
}

#[tokio::test]
async fn the_aspects_tables_bhavas_follow_the_reference_point_and_nothing_else_does() {
    let at = |reference: &'static str| async move {
        let view = view(Lang::En, &format!("{}&reference={reference}", chart_6())).await;
        aspect_texts(&view.rasi.aspects)
    };
    let (lagna, saturn) = (at("lagna").await, at("saturn").await);
    // Saturn in Leo: the Sun in Gemini aspects Sagittarius, the 5th from him.
    assert!(saturn[0][1].starts_with("Sg (5)"), "{:?}", saturn[0]);
    assert_ne!(lagna, saturn);
    let without_bhavas = |rows: Vec<[String; 3]>| {
        rows.concat()
            .join("|")
            .replace(|c: char| c.is_ascii_digit(), "")
    };
    assert_eq!(without_bhavas(lagna), without_bhavas(saturn));
}

#[tokio::test]
async fn the_argala_tables_hold_exercise_16_in_chart_5s_navamsa() {
    let query = format!("{}&varga=d9", chart_5());
    let page = body(&format!("/en/chart?{query}&topic=aspects")).await;
    let argalas = section(&page, "argalas");
    assert!(
        argalas.contains(r#"<a href="/en/kb/aspects-and-argalas">Argalas</a></h2>"#),
        "{argalas}"
    );
    assert!(
        argalas.contains("<details>\n<summary>Navamsa (D-9)</summary>"),
        "{argalas}"
    );
    let (rasi_chart, navamsa) = both_tables(argalas);
    let view = view(Lang::En, &query).await;
    assert_eq!(view.rasi.argalas.len(), 12);
    let rows = argala_texts(&view.varga.argalas);
    assert_eq!(rows.len(), 12);
    // Ch. 10 §10.8, pp. 110–111, each argala beside its obstruction: 2nd/12th, 4th/10th, 5th/9th, 11th/3rd.
    assert_eq!(
        rows[10],
        [
            "11", "Virgo", "—", "Ra", "Me", "—", "Ve", "Su", "—", "Ma, Sa"
        ]
    );
    // The book's asterisk: Ketu is in Aquarius, so its counts run in reverse.
    assert_eq!(
        rows[3],
        [
            "4",
            "Aquarius *",
            "Ve",
            "—",
            "Ma, Sa",
            "Su",
            "—",
            "—",
            "Mo",
            "Me"
        ]
    );
    assert_eq!(rows.iter().filter(|r| r[1].ends_with('*')).count(), 1);
    assert!(navamsa.contains("Counted in reverse: this rasi holds Ketu."));
    assert!(!argalas.contains("Lagna"), "{argalas}");
    // Real headers, one group per pair, for screen readers.
    for pair in ["2nd / 12th", "4th / 10th", "5th / 9th", "11th / 3rd"] {
        let header = format!(r#"<th scope="colgroup" colspan="2">{pair}</th>"#);
        assert!(rasi_chart.contains(&header), "{header}: {rasi_chart}");
    }
    assert_eq!(rasi_chart.matches(r#"<colgroup span="2">"#).count(), 5);
    assert_eq!(
        rasi_chart.matches(r#"<th scope="col">Argala</th>"#).count(),
        4
    );
    assert_eq!(
        rasi_chart
            .matches(r#"<th scope="col">Virodhargala</th>"#)
            .count(),
        4
    );
    // §10.6's several malefics are not computed (ADR 0013).
    assert!(
        argalas.contains(r##"<a href="/en/kb/aspects-and-argalas#virodhargala">"##),
        "{argalas}"
    );
    let hu = body(&format!("/hu/chart?{query}&topic=aspects")).await;
    let hu = section(&hu, "argalas");
    assert!(hu.contains("Argalák"), "{hu}");
    assert!(hu.contains(r#"<th scope="colgroup" colspan="2">11. / 3.</th>"#));
    assert!(hu.contains("Visszafelé számolva"), "{hu}");
}

#[tokio::test]
async fn the_varga_tables_count_from_the_reference_point_in_the_varga() {
    // Chart 5's D-9 (p. 101): the Moon in Aries, the Sun in Taurus aspecting Scorpio, the 8th from Aries.
    let view = view(Lang::En, &format!("{}&varga=d9&reference=moon", chart_5())).await;
    let argalas = argala_texts(&view.varga.argalas);
    assert_eq!(argalas[0][..2], ["1", "Aries"]);
    let aspects = aspect_texts(&view.varga.aspects);
    assert!(aspects[0][1].starts_with("Sc (8)"), "{:?}", aspects[0]);
}

#[tokio::test]
async fn example_35_counting_from_saturn_makes_leo_the_first_argala_row() {
    // Ch. 10 §10.7, pp. 107–108: "Mercury, Mars and Sun have an argala on Saturn, as they are in the 11th".
    let view = view(Lang::En, &format!("{}&reference=saturn", chart_6())).await;
    let rows = argala_texts(&view.rasi.argalas);
    assert_eq!(rows[0][..2], ["1", "Leo"]);
    assert_eq!(rows[0][8], "Su, Ma, Me");
}

/// The aspects page works its example on hora's chart, counted from Saturn.
#[tokio::test]
async fn the_aspects_pages_example_is_horas_chart() {
    let from_saturn = HORAS_CHART.replace("reference=lagna", "reference=saturn");
    let view_from_saturn = view(Lang::En, &format!("{from_saturn}&style=south")).await;
    let aspects = aspect_texts(&view_from_saturn.rasi.aspects);
    assert_eq!(
        aspects[6],
        [
            "Saturn",
            "Ge (3), Li (7), Cp (10) / Mo, Ke",
            "Le (5), Sc (8), Aq (11) / Ve, Ma",
        ]
    );
    assert_eq!(
        aspects[4],
        [
            "Jupiter",
            "Le (5), Li (7), Sg (9) / Mo, Su, Me",
            "Le (5), Sc (8), Aq (11) / Ve, Ma",
        ]
    );
    assert_eq!(aspects[1][1], "Ar (1) / Ju, Sa");
    // The Moon alone aspects Saturn by graha drishti; Mars and Venus by rasi drishti.
    for (i, row) in aspects.iter().enumerate() {
        assert_eq!(row[1].contains("Ar (1)"), i == 1, "{row:?}");
        assert_eq!(
            row[2].contains("Ar (1)"),
            row[0] == "Mars" || row[0] == "Venus",
            "{row:?}"
        );
    }
    let rows = argala_texts(&view_from_saturn.rasi.argalas);
    assert_eq!(
        rows[0],
        ["1", "Aries", "—", "—", "Ra", "Ke", "—", "Su, Me", "Ma", "—"]
    );
    // Ketu's rasi, counted in reverse.
    assert_eq!(
        rows[9],
        [
            "10",
            "Capricorn *",
            "Su, Me",
            "Ma",
            "Mo",
            "Ju, Sa",
            "—",
            "—",
            "—",
            "Ve"
        ]
    );
    // From the lagna, in Scorpio, Saturn's rasi is the 6th row, with the same grahas.
    let from_lagna = view(Lang::En, &format!("{HORAS_CHART}&style=south")).await;
    let rows = argala_texts(&from_lagna.rasi.argalas);
    assert_eq!(
        rows[5],
        ["6", "Aries", "—", "—", "Ra", "Ke", "—", "Su, Me", "Ma", "—"]
    );
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/aspects-and-argalas.md")),
        ("hu", include_str!("../../../kb/hu/aspects-and-argalas.md")),
    ] {
        let link = format!("(/{lang}/chart?{from_saturn}&style=south)");
        assert!(source.contains(&link), "{lang}");
    }
}

#[test]
fn the_arudha_padas_page_sends_rasi_drishti_to_the_aspects_page() {
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/arudha-padas.md")),
        ("hu", include_str!("../../../kb/hu/arudha-padas.md")),
    ] {
        let link = format!("(/{lang}/kb/aspects-and-argalas#rasi-drishti)");
        assert_eq!(source.matches(&link).count(), 2, "{lang}");
    }
}

#[tokio::test]
async fn each_bhava_arudha_is_drawn_once_in_each_chakra_in_its_tables_rasi() {
    for style in ["south", "north", "east"] {
        let query = format!("{CHART_3}&style={style}&varga=d10");
        let page = body(&format!("/en/chart?{query}")).await;
        let view = view(Lang::En, &query).await;
        let chart = hora_chart::chart(&sky_of(CHART_3).await);
        for (chakra, d) in [
            (drawn(&page), Varga::RasiChart),
            (section(&page, "varga"), Varga::Dasamsa),
        ] {
            let arudhas = chart.in_varga(d).bhava_arudhas();
            for (n, rasi) in arudhas.into_iter().enumerate() {
                let id = arudha_id(n);
                let opening = format!(
                    r#"data-body="{id}" role="img" aria-label="{} in {}""#,
                    en(&id),
                    en(rasi_id(rasi))
                );
                assert_eq!(
                    chakra.matches(&opening).count(),
                    1,
                    "{style} {d:?} {opening}: {chakra}"
                );
                assert_eq!(
                    chakra.matches(&format!(r#"data-body="{id}""#)).count(),
                    1,
                    "{style} {d:?} {id}"
                );
                // The row's rasi in the rasi chart, then in the varga.
                let row = arudha(&view, &id);
                let shown = if d == Varga::RasiChart {
                    row.rasi
                } else {
                    row.varga_rasi
                };
                assert_eq!(shown, rasi_id(rasi), "{id} {d:?}");
            }
            // Graha arudhas are not drawn.
            assert!(!chakra.contains("arudha-graha"), "{chakra}");
        }
        for chakra in [drawn(&page), section(&page, "varga")] {
            assert!(
                chakra.contains(r#"<tspan class="arudha-lagna">AL</tspan></text>"#),
                "{chakra}"
            );
            assert!(
                chakra.contains(r#"<tspan class="arudha">A7</tspan></text>"#),
                "{chakra}"
            );
        }
        assert!(page.contains("are the bhava arudhas"), "{style}");
    }
}

#[tokio::test]
async fn bhavas_are_counted_from_the_arudha_and_upapada_lagnas_each_chart_its_own() {
    let chart = hora_chart::chart(&sky_of(CHART_3).await);
    for (reference, name, n) in [
        ("arudha-lagna", "Arudha lagna", 0),
        ("upapada-lagna", "Upapada lagna", 11),
    ] {
        let query = format!("{CHART_3}&reference={reference}&style=north&varga=d10");
        let page = body(&format!("/en/chart?{query}")).await;
        let view = view(Lang::En, &query).await;
        let rasi = chart.in_varga(Varga::RasiChart).bhava_arudhas()[n];
        let d10 = chart.in_varga(Varga::Dasamsa).bhava_arudhas()[n];
        assert!(
            page.contains(&format!(r#"value="{reference}" selected>{name}<"#)),
            "{reference}"
        );
        // The row of the reference point itself is 1st in both charts.
        let row = arudha(&view, &arudha_id(n));
        assert_eq!(
            [(row.rasi, row.bhava), (row.varga_rasi, row.varga_bhava)],
            [(rasi_id(rasi), 1), (rasi_id(d10), 1)],
            "{reference}"
        );
        let top = |rasi: Rasi| {
            format!(
                r#"y="222" class="mark" aria-hidden="true">{}<"#,
                rasi as u8 + 1
            )
        };
        assert!(drawn(&page).contains(&top(rasi)), "{reference}");
        assert!(section(&page, "varga").contains(&top(d10)), "{reference}");
        // Every other table counts from it too: the lagna's row, Scorpio from Capricorn is 11th.
        let lagna = hora_chart::bhava(rasi, Rasi::Scorpio);
        assert_eq!(graha(&view, "lagna").bhava, lagna, "{reference}");
    }
    // The varga chakra's 1st bhava is D-10's AL: Virgo, from a count (p. 303).
    let page = body(&format!(
        "/en/chart?{CHART_3}&reference=arudha-lagna&style=north&varga=d10"
    ))
    .await;
    assert!(section(&page, "varga").contains(r#"y="222" class="mark" aria-hidden="true">6<"#));
    // AL in Capricorn: the rasi chakra's 1st bhava is 10.
    assert!(drawn(&page).contains(r#"y="222" class="mark" aria-hidden="true">10<"#));
}

#[tokio::test]
async fn the_grahas_table_shows_chart_3s_chara_karakas() {
    // Chart 3: AK Mercury and PK Saturn (ch. 8 §8.2, p. 265).
    let view = view(Lang::En, CHART_3).await;
    for (id, karaka) in [
        ("graha-mercury", Some(CharaKaraka::Atma)),
        ("graha-saturn", Some(CharaKaraka::Putra)),
        ("lagna", None),
        ("graha-ketu", None),
    ] {
        assert_eq!(graha(&view, id).karaka, karaka, "{id}");
    }
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{CHART_3}")).await;
        let cell = format!(
            r#"</th><td><a href="/{lang}/kb/karakas#atma-karaka"><abbr title="Atma karaka">AK</abbr></a></td>"#
        );
        assert!(body_row(&page, "graha-mercury").contains(&cell), "{lang}");
        assert!(
            body_row(&page, "lagna").contains("</th><td>—</td>"),
            "{lang}"
        );
    }
}

#[tokio::test]
async fn the_grahas_table_sorts_by_graha_or_by_chara_karaka() {
    let by_graha = [
        "lagna",
        "graha-sun",
        "graha-moon",
        "graha-mars",
        "graha-mercury",
        "graha-jupiter",
        "graha-venus",
        "graha-saturn",
        "graha-rahu",
        "graha-ketu",
    ];
    // Chart 3's karakas from its golden positions: Me, Ve, Ra, Mo, Ma, Sa, Su, Ju (ch. 8 §8.2).
    let by_karaka = [
        "lagna",
        "graha-mercury",
        "graha-venus",
        "graha-rahu",
        "graha-moon",
        "graha-mars",
        "graha-saturn",
        "graha-sun",
        "graha-jupiter",
        "graha-ketu",
    ];
    for (sort, order, active) in [
        ("", by_graha, 0),
        ("&sort=graha", by_graha, 0),
        ("&sort=karaka", by_karaka, 1),
    ] {
        let query = format!("{CHART_3}{sort}");
        let view = view(Lang::En, &query).await;
        let names: Vec<_> = view.grahas.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(names, order, "{sort}");
        let page = body(&format!("/en/chart?{query}")).await;
        let start = page.find(r#"<table id="grahas">"#).unwrap();
        let head = &page[start..start + page[start..].find("</thead>").unwrap()];
        let headers: Vec<_> = head.split("<th ").skip(1).collect();
        let sorted: Vec<_> = headers
            .iter()
            .map(|h| h.contains(r#"aria-sort="ascending""#))
            .collect();
        assert_eq!(sorted.iter().filter(|s| **s).count(), 1, "{sort}");
        let (graha, karaka) = (headers[0], headers[1]);
        assert!([graha, karaka][active].contains("aria-sort"), "{sort}");
        // Askama escapes `&` in attributes.
        let base = query.replace("&sort=graha", "").replace("&sort=karaka", "");
        for (header, value) in [(graha, "graha"), (karaka, "karaka")] {
            let href = format!(r#"href="/en/chart?{base}&sort={value}#grahas""#);
            let href = href.replace('&', "&#38;");
            assert!(header.contains(&href), "{sort}: {header}");
        }
        let hidden = sort.trim_start_matches("&sort=");
        assert!(page.contains(&format!(
            r#"<input type="hidden" name="sort" value="{hidden}">"#
        )));
    }
}

/// Chart 9 (ch. 11 *Yogas*, p. 128), at the date, time and seconds its golden test is cast with.
fn chart_9() -> String {
    with(&[
        ("date", "1630-02-19"),
        ("time", "18%3A25%3A32"),
        ("calendar", "julian"),
        ("utc", "local-mean"),
        ("offset", ""),
        ("latitude", "18.533333333333335"),
        ("longitude", "73.88333333333334"),
    ])
}

/// Chart 10 (ch. 11 *Yogas*, p. 137), entered as the Julian date the book also prints.
fn chart_10() -> String {
    with(&[
        ("date", "1542-11-24"),
        ("time", "03%3A39"),
        ("calendar", "julian"),
        ("utc", "local-mean"),
        ("offset", ""),
        ("latitude", "25.316666666666666"),
        ("longitude", "69.78333333333333"),
    ])
}

/// Chart 18 (ch. 16 *Vimsottari Dasa*, p. 217) as the book prints it.
fn chart_18() -> String {
    with(&[
        ("date", "1972-06-01"),
        ("time", "04%3A16"),
        ("offset", "%2B05%3A30"),
        ("latitude", "16.25"),
        ("longitude", "81.2"),
    ])
}

/// Chart 90 (ch. 36, p. 493), at the seconds its golden test is cast with.
fn chart_90() -> String {
    with(&[
        ("date", "1968-01-25"),
        ("time", "11%3A14%3A45"),
        ("offset", "%2B05%3A30"),
        ("latitude", "17.483333333333334"),
        ("longitude", "78.5"),
    ])
}

/// The row of a yoga table whose first cell is `name`.
fn yoga_row(rows: &[Vec<String>], name: &str) -> Vec<String> {
    let row = rows.iter().find(|row| row[0] == name);
    row.unwrap_or_else(|| panic!("no {name} in {rows:?}"))
        .clone()
}

#[tokio::test]
async fn chart_10_lists_its_mahaapurusha_yogas_and_the_raaja_yoga_of_venus_and_saturn() {
    let page = body(&format!("/en/chart?{}&topic=yogas", chart_10())).await;
    let yogas = section(&page, "yogas");
    assert!(
        yogas.contains(r#"<a href="/en/kb/yogas">Yogas</a></h2>"#),
        "{yogas}"
    );
    assert!(
            yogas.contains(r#"<th scope="rowgroup" colspan="6"><a href="/en/kb/mahaapurusha-yogas">Mahaapurusha yogas</a></th>"#),
            "{yogas}"
        );
    assert!(
        yogas.contains(r##"<a href="/en/kb/yogas#checked">"##),
        "{yogas}"
    );
    let rows = yoga_texts(Lang::En, &view(Lang::En, &chart_10()).await.rasi.yogas);
    // Ch. 11 §11.7.2, p. 137: "Mahapurusha yogas (Maalavya and Sasa)" and "another Mahapurusha Yoga – Ruchaka Yoga".
    for (name, graha) in [
        ("Maalavya yoga", "Venus"),
        ("Sasa yoga", "Saturn"),
        ("Ruchaka yoga", "Mars"),
    ] {
        let row = yoga_row(&rows, name);
        assert!(row[1].starts_with(&format!("{graha} (")), "{row:?}");
        assert_eq!(row[2..], ["—", "—", "—", "—"], "{row:?}");
    }
    // P. 137: Venus and Saturn "less than a degree apart"; their printed amsas are not golden (ADR 0014).
    let row = rows.iter().find(|row| {
        row[0] == "Raaja yoga" && row[1] == "Venus (1L, 8L, in 1), Saturn (4L, 5L, in 1)"
    });
    let row = row.unwrap_or_else(|| panic!("{rows:?}"));
    assert!(row[4].starts_with("0°"), "{row:?}");
    // P. 137: Jupiter, a functional malefic, in their rasi "22° away" from Venus (ADR 0016).
    assert!(
        row[3].contains("Venus afflicted by Jupiter, a functional malefic, 22°28′ apart"),
        "{row:?}"
    );
    let amsas: Vec<_> = row[5].split(" / ").collect();
    assert!(
        amsas.len() == 2 && amsas[0].starts_with("Venus: ") && amsas[1].starts_with("Saturn: "),
        "{row:?}"
    );
}

#[tokio::test]
async fn chart_6s_vipareeta_raaja_yoga_of_mars_and_the_sun_is_17_apart() {
    let rows = yoga_texts(Lang::En, &view(Lang::En, &chart_6()).await.rasi.yogas);
    // Ch. 17, p. 230: Mars and the Sun "within a 17' from each other".
    let row = rows
        .iter()
        .find(|row| row[0] == "Vipareeta Raaja yoga" && row[1].starts_with("Sun ("));
    assert_eq!(row.map(|row| row[4].as_str()), Some("0°17′"), "{rows:?}");
}

#[tokio::test]
async fn chart_18s_saptamsa_lists_the_raaja_yoga_of_venus_and_rahu() {
    let query = format!("{}&varga=d7", chart_18());
    let page = body(&format!("/en/chart?{query}&topic=yogas")).await;
    let yogas = section(&page, "yogas");
    assert!(
        yogas.contains("<details>\n<summary>Saptamsa (D-7)</summary>"),
        "{yogas}"
    );
    // A varga's table has no magnitude columns (ADR 0014).
    assert!(
        !yogas
            .split_once("<details>")
            .unwrap()
            .1
            .contains("Closeness"),
        "{yogas}"
    );
    // Ch. 16, p. 218: Rahu "acts as the 5th lord", "with lagna lord Venus. His conjunction with Venus – conjunction of 1st and
    // 5th lords – consists of a raja yoga", which is also §11.7.3 (1)'s second half.
    let rows = yoga_texts(Lang::En, &view(Lang::En, &query).await.varga.yogas);
    for (name, grahas) in [
        ("Raaja yoga", "Venus (1L, 8L, in 11), Rahu (5L, in 11)"),
        (
            "Raaja yoga §11.7.3 (1)",
            "Venus (1L, in 11), Rahu (5L, in 11)",
        ),
    ] {
        let row = rows.iter().find(|row| row[0] == name && row[1] == grahas);
        assert!(row.is_some_and(|row| row[2] == "—"), "{name}: {rows:?}");
    }
}

#[tokio::test]
async fn chart_3s_raaja_yoga_1_names_its_ak_and_pk() {
    // Ch. 20, p. 265: "a powerful raja yoga between AK (Mercury) and PK (Saturn)", in the lagna.
    for (lang, grahas) in [
        (Lang::En, "Mercury (AK, in 1), Saturn (PK, in 1)"),
        (
            Lang::Hu,
            "Merkúr (AK, 1. bhavában), Szaturnusz (PK, 1. bhavában)",
        ),
    ] {
        let rows = yoga_texts(lang, &view(lang, CHART_3).await.rasi.yogas);
        assert_eq!(yoga_row(&rows, "Raaja yoga §11.7.3 (1)")[1], grahas);
    }
}

#[tokio::test]
async fn chart_90s_dasamsa_has_the_vipareeta_raaja_yoga_of_the_moon_and_jupiter_to_judge() {
    let query = format!("{}&varga=d10", chart_90());
    // Real-life Example 3, p. 494: the 8th and 12th lords conjoined in a kendra, effective only "in strength".
    let rows = yoga_texts(Lang::En, &view(Lang::En, &query).await.varga.yogas);
    let row = rows
        .iter()
        .find(|row| row[0] == "Vipareeta Raaja yoga" && row[1].starts_with("Moon ("));
    let row = row.unwrap_or_else(|| panic!("{rows:?}"));
    assert!(row[1].contains("Jupiter ("), "{row:?}");
    assert_eq!(row[2], "Whether Moon is strong / Whether Jupiter is strong");
    let page = body(&format!("/en/chart?{query}&topic=yogas")).await;
    assert!(
        page.contains(r#"<a href="/en/kb/raaja-yogas">Whether Moon is strong</a>"#),
        "{page}"
    );
    let rows = yoga_texts(Lang::Hu, &view(Lang::Hu, &query).await.varga.yogas);
    assert!(
        rows.iter()
            .any(|row| row[2] == "Erős-e a Hold / Erős-e a Jupiter"),
        "{rows:?}"
    );
}

#[tokio::test]
async fn the_rasi_only_yogas_are_never_in_the_vargas_table() {
    let view = view(Lang::En, &format!("{}&varga=d9", chart_9())).await;
    let names = |groups| -> Vec<String> {
        yoga_texts(Lang::En, groups)
            .into_iter()
            .map(|row| row[0].clone())
            .collect()
    };
    let (rasi_chart, navamsa) = (names(&view.rasi.yogas), names(&view.varga.yogas));
    // Ch. 11 §11.6, pp. 128–129: Kalpadruma mixes the rasi chart and the navamsa (ADR 0014).
    assert!(
        rasi_chart.iter().any(|name| name == "Kalpadruma yoga"),
        "{rasi_chart:?}"
    );
    for name in [
        "Kalpadruma",
        "Vishnu",
        "Gouri",
        "Chandikaa",
        "Bhaarathi",
        "Vidyut",
        "Jaya",
    ] {
        let yoga = format!("{name} yoga");
        assert!(!navamsa.iter().any(|n| n.starts_with(&yoga)), "{name}");
    }
    for numbered in [
        "Raaja yoga §11.7.3 (9)",
        "Daridra yoga §11.10 (8)",
        "Daridra yoga §11.10 (9)",
    ] {
        assert!(!navamsa.iter().any(|n| n.contains(numbered)), "{numbered}");
    }
}

#[tokio::test]
async fn the_yoga_tables_count_from_the_lagna_whatever_the_reference() {
    for chart in [chart_10(), chart_90()] {
        let yogas = |view: ChartView| {
            [view.rasi.yogas, view.varga.yogas].map(|groups| yoga_texts(Lang::En, &groups))
        };
        let from_lagna = yogas(view(Lang::En, &format!("{chart}&varga=d10")).await);
        let from_moon = yogas(view(Lang::En, &format!("{chart}&varga=d10&reference=moon")).await);
        assert_eq!(from_lagna, from_moon);
    }
}

/// The row of `id` ("graha-mercury", "av-sav", ...) in an ashtakavarga's BAV table.
fn av_row<'a>(av: &'a Ashtakavarga, id: &str) -> &'a [AvCell; 12] {
    &av.bav.iter().find(|row| row.0 == id).expect(id).1
}

#[tokio::test]
async fn chart_6s_ashtakavarga_holds_mercurys_bav_and_pinda_and_the_sav() {
    use Rasi::*;
    let view = view(Lang::En, &chart_6()).await;
    let av = &view.rasi.ashtakavarga;
    // Ch. 12 Exercise 19, p. 163 (Mercury in Ge), Exercise 20's SAV and Example 43's sodhya pinda, p. 160.
    let mercury = av_row(av, "graha-mercury");
    assert_eq!(
        [mercury[Aries as usize].0, mercury[Gemini as usize].0],
        [7, 7]
    );
    let natal = "av-natal".to_owned();
    assert!(mercury[Gemini as usize].1.contains(&natal));
    assert!(!mercury[Aries as usize].1.contains(&natal));
    let sav = &av_row(av, "av-sav")[Aries as usize];
    assert_eq!(
        (sav.0, sav.1.as_slice()),
        (34, ["av-sav-strong".to_owned()].as_slice())
    );
    let pinda = av.pindas.iter().find(|p| p.0 == "graha-mercury").unwrap();
    assert_eq!(pinda.1.sodhya(), 152);
    // The marker is text, with words for a screen reader (ADR 0015).
    let page = body(&format!("/en/chart?{}&topic=ashtakavarga", chart_6())).await;
    let section = section(&page, "ashtakavarga");
    assert!(
        section.contains(
            r#"<h2 id="ashtakavarga"><a href="/en/kb/ashtakavarga">Ashtakavarga</a></h2>"#
        ),
        "{section}"
    );
    let row = section
            .split("<tr>")
            .find(|row| row.contains(r#"<th scope="row"><a href="/en/kb/graha-characteristics#mercury">Mercury</a></th><td>"#))
            .unwrap();
    assert!(
            row.contains(r#"<td>7 <span aria-hidden="true">+</span><span class="sr">favourable</span> <span aria-hidden="true">•</span><span class="sr">its rasi</span></td>"#),
            "{row}"
        );
}

#[tokio::test]
async fn chart_3s_dasamsa_has_a_strong_35_in_scorpio() {
    // Ch. 12 Example 39, p. 154: the D-10's SAV.
    let page = body(&format!("/en/chart?{CHART_3}&varga=d10&topic=ashtakavarga")).await;
    let section = section(&page, "ashtakavarga");
    let (_, varga) = section
        .split_once("<details>\n<summary>Dasamsa (D-10)</summary>")
        .expect(section);
    let sav = varga
        .split("<tr>")
        .find(|row| row.starts_with(r#"<th scope="row">SAV</th>"#))
        .unwrap();
    let cells: Vec<_> = sav.split("<td>").skip(1).collect();
    assert_eq!(
        cells[Rasi::Scorpio as usize],
        r#"35 <span aria-hidden="true">+</span><span class="sr">strong</span></td>"#
    );
}

#[tokio::test]
async fn the_ashtakavarga_tables_count_from_the_lagna_whatever_the_reference() {
    let tables = |query: String| async move {
        section(
            &body(&format!("/en/chart?{query}&topic=ashtakavarga")).await,
            "ashtakavarga",
        )
        .to_owned()
    };
    let from_lagna = tables(format!("{CHART_3}&varga=d10")).await;
    assert!(from_lagna.contains("Dasamsa (D-10)"));
    for reference in ["moon", "arudha-lagna"] {
        let from = tables(format!("{CHART_3}&varga=d10&reference={reference}")).await;
        assert_eq!(from, from_lagna, "{reference}");
    }
}

#[test]
fn every_ashtakavarga_band_has_a_name_and_each_marked_one_a_mark() {
    use hora_chart::{BavBand, SavBand};
    let en = keys(include_str!("../../../locales/en/main.ftl"));
    let names = [BavBand::Favourable, BavBand::Neutral, BavBand::Unfavourable]
        .map(|b| kb::message_id("av-bav", b))
        .into_iter()
        .chain(
            [SavBand::Strong, SavBand::Average, SavBand::Weak].map(|b| kb::message_id("av-sav", b)),
        );
    let marks = [
        "av-bav-favourable",
        "av-bav-unfavourable",
        "av-sav-strong",
        "av-sav-weak",
        "av-natal",
    ]
    .map(|id| format!("{id}-mark"));
    let missing: Vec<_> = names
        .chain(marks)
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    // both_languages_have_the_same_messages holds Hungarian to the same ids.
    assert_eq!(missing, [""; 0]);
}

/// Every yoga with a name of its own, without the numbered entries of §11.7.3–11.10.
fn named_yogas() -> Vec<hora_chart::YogaName> {
    use hora_chart::YogaName::*;
    vec![
        Vesi,
        Vosi,
        Ubhayachara,
        BudhaAaditya,
        Sunaphaa,
        Anaphaa,
        Duradhara,
        Kemadruma,
        ChandraMangala,
        Adhi,
        Ruchaka,
        Bhadra,
        Sasa,
        Maalavya,
        Hamsa,
        Rajju,
        Musala,
        Nala,
        Maalaa,
        Sarpa,
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
        Veenaa,
        Daama,
        Paasa,
        Kedaara,
        Soola,
        Yuga,
        Gola,
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
        Tapaswi,
        BasicRaaja,
        DharmaKarmadhipati,
        VipareetaRaaja,
        BasicDhana,
    ]
}

#[test]
fn every_yoga_class_and_condition_has_a_name() {
    use hora_chart::{YogaClass, YogaName};
    let en = keys(include_str!("../../../locales/en/main.ftl"));
    let names = named_yogas();
    let classes = [
        YogaClass::Ravi,
        YogaClass::Chandra,
        YogaClass::Mahaapurusha,
        YogaClass::Naabhasa,
        YogaClass::OtherPopular,
        YogaClass::Raaja,
        YogaClass::RaajaSambandha,
        YogaClass::Dhana,
        YogaClass::Daridra,
    ];
    let numbered =
        ["raaja", "raaja-sambandha", "dhana", "daridra"].map(|class| format!("yoga-{class}-n"));
    let conditions = [
        "mercurys-nature",
        "may-not-operate-well",
        "strong",
        "combust",
        "deep-exaltation",
        "malefic-in-upachaya",
        "all-in-one-rasi",
    ]
    .map(|c| format!("yoga-to-judge-{c}"));
    let missing: Vec<_> = names
        .into_iter()
        .map(|n: YogaName| kb::message_id("yoga", n))
        .chain(classes.map(|c| kb::message_id("yoga-class", c)))
        .chain(numbered)
        .chain(conditions)
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}

/// Every ch. 11 entry under its own heading, and ch. 13's Tapaswi: hora's 100 named yogas (the basic Dhana among them) and every numbered entry, Raaja (18) and Daridra (10) included though not computed.
#[tokio::test]
async fn every_yoga_has_a_section_on_its_class_page_in_both_languages() {
    use hora_chart::YogaName as N;
    let named = named_yogas().into_iter().map(|n| {
        let id = kb::message_id("yoga", n);
        (kb::yoga_page(n.class()), id["yoga-".len()..].to_owned())
    });
    let numbered = [
        (N::Raaja(1), "raaja", 18),
        (N::RaajaSambandha(1), "raaja-sambandha", 15),
        (N::Dhana(1), "dhana", 12),
        (N::Daridra(1), "daridra", 13),
    ]
    .into_iter()
    .flat_map(|(n, prefix, last)| {
        (1..=last).map(move |i| (kb::yoga_page(n.class()), format!("{prefix}-{i}")))
    });
    let anchors: Vec<_> = named.chain(numbered).collect();
    assert_eq!(anchors.len(), 100 + 18 + 15 + 12 + 13);
    for lang in ["en", "hu"] {
        let overview = body(&format!("/{lang}/kb/yogas")).await;
        assert!(overview.contains(r#"id="checked""#), "{lang}");
        let mut pages = std::collections::HashMap::new();
        for (page, _) in &anchors {
            if !pages.contains_key(page) {
                pages.insert(*page, body(&format!("/{lang}/kb/{page}")).await);
            }
        }
        let missing: Vec<_> = anchors
            .iter()
            .filter(|(page, id)| !pages[page].contains(&format!(r#"id="{id}""#)))
            .collect();
        assert!(missing.is_empty(), "{lang}: {missing:?}");
    }
}

/// Chart 13 (ch. 13 *Interpreting Charts*, p. 173) as the book prints it.
fn chart_13() -> String {
    with(&[
        ("date", "1894-05-20"),
        ("time", "13%3A22"),
        ("offset", "%2B05%3A30"),
        ("latitude", "11.95"),
        ("longitude", "79.53333333333333"),
    ])
}

/// Chart 14 (ch. 13, p. 175), at the seconds of its footnote 37.
fn chart_14() -> String {
    with(&[
        ("date", "1944-08-20"),
        ("time", "07%3A11%3A40"),
        ("offset", "%2B05%3A30"),
        ("latitude", "18.966666666666665"),
        ("longitude", "72.81666666666666"),
    ])
}

#[tokio::test]
async fn a_relatives_lagna_counts_the_vargas_bhavas_from_it() {
    // Ch. 13 §13.4.2, p. 176: in chart 14's D-3 the younger brother's lagna is Li, where the 3rd lord Saturn is (ADR 0016).
    let query = format!("{}&varga=d3&reference=younger-sibling-1", chart_14());
    let view = view(Lang::En, &query).await;
    let first = &view.varga.argalas[0];
    assert_eq!((first.bhava, first.rasi), (1, "rasi-libra"));
    for (lang, label) in [
        ("en", "1st younger sibling (D-3)"),
        ("hu", "1. fiatalabb testvér (D-3)"),
    ] {
        let page = body(&format!("/{lang}/chart?{query}")).await;
        let option = format!(r#"<option value="younger-sibling-1" selected>{label}</option>"#);
        assert!(page.contains(&option), "{lang}");
    }
}

#[tokio::test]
async fn chart_13_has_the_tapaswi_yoga_with_its_two_notes() {
    // Ch. 13 §13.5, Example 44, p. 174: Saturn and Ketu in Vi, Venus aspecting from Pi, the 8th; "AK is also involved".
    for (lang, name, notes) in [
        (
            Lang::En,
            "Tapaswi yoga",
            "AK Saturn joins or aspects another of the three: more fruitful / Venus in the 8th: particularly powerful",
        ),
        (
            Lang::Hu,
            "Tapaswi yoga",
            "Az AK (Szaturnusz) együtt áll a másik kettő egyikével, vagy aspektálja azt: gyümölcsözőbb / A Vénusz a 8. bhavában: különösen erős",
        ),
    ] {
        let rows = yoga_texts(lang, &view(lang, &chart_13()).await.rasi.yogas);
        let row = yoga_row(&rows, name);
        assert_eq!(row[2..4], ["—", notes], "{row:?}");
    }
    let page = body(&format!("/en/chart?{}&topic=yogas", chart_13())).await;
    let note =
        r#"<a href="/en/kb/other-popular-yogas">Venus in the 8th: particularly powerful</a>"#;
    assert!(section(&page, "yogas").contains(note), "{page}");
}

#[tokio::test]
async fn chart_14s_raaja_yoga_lists_its_afflicting_functional_malefics() {
    // Ch. 11 §11.7.2, pp. 136–137: Mercury and Venus, Leo functional malefics, with the Sun and Jupiter, and Saturn's
    // "very close aspect" on Jupiter (ADR 0016).
    let rows = yoga_texts(Lang::En, &view(Lang::En, &chart_14()).await.rasi.yogas);
    let row = rows.iter().find(|row| {
        row[0] == "Raaja yoga" && row[1].starts_with("Sun (") && row[1].contains("Jupiter (")
    });
    let row = row.unwrap_or_else(|| panic!("{rows:?}"));
    for graha in ["Mercury", "Venus"] {
        assert!(
            row[3].contains(&format!("Sun afflicted by {graha}, a functional malefic, ")),
            "{row:?}"
        );
    }
    assert!(
        row[3].contains("Jupiter afflicted by Saturn, a functional malefic, 2°01′ apart"),
        "{row:?}"
    );
    // A varga's afflictions have no closeness.
    let rows = yoga_texts(Lang::En, &view(Lang::En, &chart_14()).await.varga.yogas);
    assert!(
        rows.iter().all(|row| !row[3].contains(" apart")),
        "{rows:?}"
    );
}

#[tokio::test]
async fn chart_90s_venus_is_a_yogada_for_money_and_power() {
    // Ch. 36, p. 496: lagna Pi, HL Ta, GL Sg, Venus in Sg "aspects lagna (by rasi aspect)… Venus is a yogada" (ADR 0016).
    let view = view(Lang::En, &chart_90()).await;
    let venus: Vec<_> = view
        .rasi
        .yogadas
        .iter()
        .filter(|y| y.graha == "graha-venus")
        .map(|y| {
            let links = y
                .links
                .each_ref()
                .map(|l| l.iter().map(|id| en(id)).collect::<Vec<_>>().join(", "));
            (en(&y.gives), links)
        })
        .collect();
    assert_eq!(
        venus,
        [
            (
                "Money (HL)".to_owned(),
                ["rasi drishti".to_owned(), "owns".to_owned()]
            ),
            (
                "Power (GL)".to_owned(),
                ["rasi drishti".to_owned(), "in it".to_owned()]
            ),
        ]
    );
    let page = body(&format!("/hu/chart?{}&topic=interpreting", chart_90())).await;
    let yogadas = section(&page, "yogadas");
    assert!(yogadas.contains("<td>Pénz (HL)</td>"), "{yogadas}");
}

#[tokio::test]
async fn without_a_vedic_day_the_yogadas_say_so() {
    let page = body(&format!("/en/chart?{}&topic=interpreting", with(&TROMSO))).await;
    let yogadas = section(&page, "yogadas");
    assert!(!yogadas.contains("<table"), "{yogadas}");
    assert!(yogadas.contains(&en("yogadas-no-vedic-day")), "{yogadas}");
}

#[tokio::test]
async fn the_grahas_table_shows_functional_nature_from_the_reference_point() {
    // Ch. 13 §13.2, Table 30 (p. 167): from Le, Mars the yogakaraka, Venus a malefic; from the Moon's Cn too, Mars the
    // yogakaraka, Jupiter a benefic. Nodes and the lagna have none (ADR 0016).
    let nature = |view: &ChartView, id: &str| -> [String; 2] {
        graha(view, id).functional.clone().map(|cell| {
            cell.map_or("—".to_owned(), |(id, yogakaraka)| {
                let name = en(&id);
                if yogakaraka {
                    format!("{name}, {}", en("yogakaraka"))
                } else {
                    name
                }
            })
        })
    };
    let view = view(Lang::En, &chart_14()).await;
    assert_eq!(nature(&view, "graha-mars")[0], "Benefic, Yogakaraka");
    assert_eq!(nature(&view, "graha-venus")[0], "Malefic");
    for id in ["lagna", "graha-rahu", "graha-ketu"] {
        assert_eq!(nature(&view, id), ["—", "—"], "{id}");
    }
    let page = body(&format!("/en/chart?{}", chart_14())).await;
    assert!(
            page.contains(r#"<th scope="colgroup" colspan="2"><a href="/en/kb/functional-nature">Functional nature</a></th>"#),
            "{page}"
        );
}

#[tokio::test]
async fn every_kb_link_on_every_topic_reaches_a_listed_page() {
    for lang in ["en", "hu"] {
        let mut pages = std::collections::HashMap::new();
        for topic in &TOPICS {
            let page = body(&format!("/{lang}/chart?{}&topic={}", chart_14(), topic.key)).await;
            let prefix = format!(r#"href="/{lang}/kb/"#);
            let mut links = page.split(prefix.as_str()).skip(1).peekable();
            assert!(links.peek().is_some(), "{lang} {}", topic.key);
            for link in links {
                let link = &link[..link.find('"').unwrap()];
                let (slug, anchor) = link.split_once('#').unwrap_or((link, ""));
                assert!(
                    slug.is_empty() || kb::PAGES.iter().any(|p| p.slug == slug),
                    "{lang} {}: {slug}",
                    topic.key
                );
                if !anchor.is_empty() {
                    if !pages.contains_key(slug) {
                        pages.insert(slug.to_owned(), body(&format!("/{lang}/kb/{slug}")).await);
                    }
                    assert!(
                        pages[slug].contains(&format!(r#"id="{anchor}""#)),
                        "{lang} {}: {slug}#{anchor}",
                        topic.key
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn the_baadhakas_table_counts_from_the_reference_point() {
    // Ch. 13 §13.3, Table 31 (pp. 168–169): chart 1's Vi lagna, a dwiswabhava, has its baadhaka sthaana in the 7th, Pi,
    // and Jupiter its baadhaka.
    let view = view(Lang::En, CHART_1).await;
    assert_eq!(view.rasi.baadhakas.len(), 12);
    let first = &view.rasi.baadhakas[0];
    assert_eq!(
        (
            first.bhava,
            first.rasi,
            first.sthaana,
            first.baadhakas.as_slice()
        ),
        (1, "rasi-virgo", ("rasi-pisces", 7), &["graha-jupiter"][..])
    );
    assert!(
        first
            .arudhas
            .iter()
            .all(|id| id.starts_with("bhava-arudha-"))
    );
    let arudhas: usize = view.rasi.baadhakas.iter().map(|r| r.arudhas.len()).sum();
    assert_eq!(arudhas, 12);
    let page = body(&format!("/en/chart?{CHART_1}&topic=interpreting")).await;
    let baadhakas = section(&page, "baadhakas");
    assert!(
        baadhakas.contains(r#"<a href="/en/kb/baadhaka-and-yogada">Baadhakas</a></h2>"#),
        "{baadhakas}"
    );
    assert!(
        baadhakas.contains("<summary>Navamsa (D-9)</summary>"),
        "{baadhakas}"
    );
}

#[test]
fn every_chapter_13_value_has_a_message() {
    use hora_chart::{FunctionalNature, Gives, Link};
    let en = keys(include_str!("../../../locales/en/main.ftl"));
    let missing: Vec<_> = [
        FunctionalNature::Benefic,
        FunctionalNature::Neutral,
        FunctionalNature::Malefic,
    ]
    .map(|n| kb::message_id("functional", n))
    .into_iter()
    .chain([Gives::Money, Gives::Power].map(|g| kb::message_id("yogada-gives", g)))
    .chain([Link::Owns, Link::Conjoins, Link::Aspects].map(|l| kb::message_id("yogada-link", l)))
    .chain(REFERENCES.iter().map(|r| r.2.to_owned()))
    .chain(
        [
            "yogakaraka",
            "functional-nature",
            "yoga-notes",
            "yoga-affliction",
            "yoga-affliction-closeness",
            "yoga-note-atma-karaka",
            "yoga-note-in-the-eighth",
            "topic-interpreting-charts",
            "baadhakas",
            "baadhaka-sthaana",
            "baadhaka-arudhas",
            "baadhaka-grahas",
            "yogadas",
            "yogada-gives",
            "yogada-hl-or-gl",
            "yogadas-none",
            "yogadas-no-vedic-day",
        ]
        .map(str::to_owned),
    )
    .filter(|id| !en.contains(id.as_str()))
    .collect();
    assert_eq!(missing, [""; 0]);
}

/// Chart 8 (ch. 10, p. 109; Exercise 23, p. 186) as the book prints it.
fn chart_8() -> String {
    with(&[
        ("date", "1946-12-02"),
        ("time", "06%3A45"),
        ("offset", "%2B01%3A00"),
        ("latitude", "38.1"),
        ("longitude", "15.65"),
    ])
}

#[tokio::test]
async fn chart_8_shows_its_marakas_rudra_and_longevity() {
    // Exercise 23's answer (ch. 14 §14.7, p. 186): Jupiter, Venus and Rahu the main marakas; Rudra Mercury, Trishoola Ge, Li, Aq;
    // Maheswara Jupiter by exception (2); middle life; the eighth lord method's short life from Sc, Mercury in its 12th. Jupiter as
    // the afflicted weaker and the paramaayush of 64 are hora's readings (ADR 0017).
    let view = view(Lang::En, &chart_8()).await;
    let marakas: Vec<_> = view.rasi.marakas.iter().map(|m| m.0).collect();
    for graha in ["graha-jupiter", "graha-venus", "graha-rahu"] {
        assert!(marakas.contains(&graha), "{marakas:?}");
    }
    assert_eq!(
        view.rasi.maraka_sthanas,
        [("rasi-sagittarius", 2), ("rasi-taurus", 7)]
    );
    let rahu = view.rasi.marakas.iter().find(|m| m.0 == "graha-rahu");
    assert!(rahu.unwrap().1.contains(&"in 7".to_owned()), "{rahu:?}");
    let l = &view.longevity;
    assert_eq!(
        l.candidates,
        [
            ("rasi-sagittarius", "graha-jupiter"),
            ("rasi-gemini", "graha-mercury")
        ]
    );
    assert_eq!(
        (l.rudra, l.decided_by.as_str(), l.weaker, l.weaker_afflicted),
        (
            "graha-mercury",
            "rudra-rule-advanced",
            "graha-jupiter",
            true
        )
    );
    assert_eq!(l.trishoola, ["rasi-gemini", "rasi-libra", "rasi-aquarius"]);
    assert_eq!(
        (l.maheswara, l.maheswara_exceptions.as_slice()),
        (
            "graha-jupiter",
            &["maheswara-exception-node".to_owned()][..]
        )
    );
    let longevity: Vec<_> = l.pairs.iter().map(|p| p.longevity.as_str()).collect();
    assert_eq!(
        longevity,
        ["longevity-middle", "longevity-middle", "longevity-short"]
    );
    assert_eq!(
        l.result,
        Some((
            "Middle life (36–72 years)".to_owned(),
            "pairs-rule-agreement".to_owned()
        ))
    );
    assert_eq!(l.paramaayush, "64 years");
    assert_eq!(
        l.eighth_lord,
        (
            "rasi-scorpio",
            "graha-mercury",
            12,
            "Short life (0–36 years)".to_owned()
        )
    );
    for (lang, middle, caution) in [
        ("en", "Middle life (36–72 years)", "not infallible"),
        ("hu", "Közepes élet (36–72 év)", "nem tévedhetetlenek"),
    ] {
        let page = body(&format!("/{lang}/chart?{}&topic=longevity", chart_8())).await;
        let pairs = section(&page, "three-pairs");
        assert!(pairs.contains(&format!("<td>{middle}</td>")), "{pairs}");
        assert!(
            pairs.contains(&format!(r#"<a href="/{lang}/kb/longevity#caution">"#))
                && pairs.contains(caution),
            "{pairs}"
        );
        let rudra = section(&page, "rudra");
        assert!(
            rudra.contains(&format!(
                r#"<h2 id="rudra"><a href="/{lang}/kb/marakas-and-rudra#rudra">Rudra</a></h2>"#
            )),
            "{rudra}"
        );
    }
}

#[tokio::test]
async fn a_varga_shows_marakas_and_no_rudra_or_longevity() {
    // Ch. 14 §14.2 reads marakas in any chart, D-11 and D-30 by name; Rudra and both methods in the rasi chart only (ADR 0017).
    let query = format!("{}&varga=d11&topic=longevity", chart_8());
    let view = view(Lang::En, &query).await;
    assert!(!view.varga.marakas.is_empty());
    let page = body(&format!("/en/chart?{query}")).await;
    let marakas = section(&page, "marakas");
    assert!(
        marakas.contains("<summary>Rudramsa (D-11)</summary>"),
        "{marakas}"
    );
    assert_eq!(marakas.matches("<table>").count(), 2, "{marakas}");
    for id in ["rudra", "three-pairs", "eighth-lord"] {
        let shown = section(&page, id);
        assert!(
            !shown.contains("<details") && !shown.contains("D-11"),
            "{shown}"
        );
        assert!(shown.contains("<caption>Rasi chart</caption>"), "{shown}");
    }
}

#[tokio::test]
async fn without_a_vedic_day_the_three_pairs_say_so() {
    let view = view(Lang::En, &with(&TROMSO)).await;
    assert_eq!(view.longevity.pairs.len(), 2);
    assert!(view.longevity.result.is_none());
    let page = body(&format!("/en/chart?{}&topic=longevity", with(&TROMSO))).await;
    let pairs = section(&page, "three-pairs");
    assert!(pairs.contains(&en("pairs-no-vedic-day")), "{pairs}");
    assert!(!pairs.contains(&en("paramaayush")), "{pairs}");
}

/// Example 49 (ch. 15 §15.4.4, pp. 199–200), chart 27's birth, with the name sound `sound`.
fn example_49(sound: &str) -> String {
    with(&[
        ("date", "1970-04-04"),
        ("time", "17%3A50"),
        ("offset", "%2B05%3A30"),
        ("latitude", "16.25"),
        ("longitude", "81.2"),
        ("sound", sound),
    ])
}

/// The Sayanaadi cell pair of `graha`'s avasthas row: (avastha id, strength id).
fn sayanaadi<'a>(view: &'a ChartView, graha: &str) -> &'a Option<(String, Option<String>)> {
    let row = view.strength.avasthas.iter().find(|r| r.graha == graha);
    &row.expect(graha).sayanaadi
}

#[tokio::test]
async fn example_49_shows_the_books_sayanaadi_avasthas_and_strengths() {
    // Example 49's answer, sound 1 (p. 200): Mercury Gamana and cheshta, Jupiter Nriyalipsaa and drishti, Venus Sabhaa and cheshta.
    let view = view(Lang::En, &example_49("1")).await;
    for (graha, avastha, strength) in [
        ("graha-mercury", "gamana", "cheshta"),
        ("graha-jupiter", "nriyalipsaa", "drishti"),
        ("graha-venus", "sabhaa", "cheshta"),
    ] {
        assert_eq!(
            sayanaadi(&view, graha),
            &Some((
                format!("sayanaadi-avastha-{avastha}"),
                Some(format!("activity-strength-{strength}"))
            )),
            "{graha}"
        );
    }
    // The state names are the same in both languages (GLOSSARY.md).
    for lang in ["en", "hu"] {
        let query = format!("{}&topic=strength", example_49("1"));
        let page = body(&format!("/{lang}/chart?{query}")).await;
        let avasthas = section(&page, "avasthas");
        assert!(
            avasthas.contains("<td>Gamana</td><td>cheshta</td>"),
            "{avasthas}"
        );
        assert!(avasthas.contains(r#"<th scope="col">"#), "{avasthas}");
        assert!(avasthas.contains(r#"<th scope="row">"#), "{avasthas}");
        let results = format!(r#"href="/{lang}/kb/avasthas""#);
        assert!(avasthas.contains(&results), "{avasthas}");
        let lords = section(&page, "stronger-lords");
        assert_eq!(lords.matches("<tr>").count(), 1 + 2, "{lords}");
        assert_eq!(lords.matches(r#"<th scope="col">"#).count(), 2, "{lords}");
        let karakas = section(&page, "sthira-karakas");
        assert_eq!(karakas.matches("<tr>").count(), 1 + 7, "{karakas}");
        // The reference point does not move ch. 15's tables (ADR 0018).
        let from_moon = body(&format!("/{lang}/chart?{query}&reference=moon")).await;
        for id in ["avasthas", "stronger-lords", "sthira-karakas"] {
            assert_eq!(section(&from_moon, id), section(&page, id), "{lang} {id}");
        }
    }
}

#[tokio::test]
async fn without_a_sound_the_sayanaadi_avasthas_show_and_their_strength_says_why_not() {
    let view = view(Lang::En, &example_49("")).await;
    assert_eq!(
        sayanaadi(&view, "graha-mercury"),
        &Some(("sayanaadi-avastha-gamana".to_owned(), None))
    );
    for lang in [Lang::En, Lang::Hu] {
        let query = format!("{}&topic=strength", example_49(""));
        let page = body(&format!("/{}/chart?{query}", lang.code())).await;
        let avasthas = section(&page, "avasthas");
        let needs = t(lang, "activity-strength-needs-sound");
        assert!(
            avasthas.contains(&format!("<td>Gamana</td><td>{needs}</td>")),
            "{avasthas}"
        );
    }
}

#[tokio::test]
async fn without_a_vedic_day_the_sayanaadi_avasthas_say_so() {
    let query = format!("{}&sound=1&topic=strength", with(&TROMSO));
    let view = view(Lang::En, &query).await;
    assert!(view.strength.avasthas.iter().all(|r| r.sayanaadi.is_none()));
    let page = body(&format!("/en/chart?{query}")).await;
    let avasthas = section(&page, "avasthas");
    assert!(
        avasthas.contains(&en("sayanaadi-no-vedic-day")),
        "{avasthas}"
    );
}

#[tokio::test]
async fn a_node_reads_a_dash_where_a_state_does_not_apply() {
    // Example 49's Ketu is in Leo, neither his own rasi, exaltation nor debilitation (ADR 0018).
    let view = view(Lang::En, &example_49("1")).await;
    let ketu = view
        .strength
        .avasthas
        .iter()
        .find(|r| r.graha == "graha-ketu");
    assert_eq!(ketu.unwrap().alertness, None);
    let page = body(&format!("/en/chart?{}&topic=strength", example_49("1"))).await;
    let avasthas = section(&page, "avasthas");
    let row = &avasthas[avasthas.find(r#"data-body="graha-ketu""#).unwrap()..];
    let row = &row[..row.find("</tr>").unwrap()];
    assert!(row.contains("<td>—</td>"), "{row}");
}

#[tokio::test]
async fn a_kopita_graha_says_its_mood_is_to_judge() {
    // §15.4.3: Kopita is being joined by the Sun, and how closely is left to judge (ADR 0018). Chart 1's Sun joins
    // Mercury and Venus in Pisces.
    let view = view(Lang::En, CHART_1).await;
    let kopita: Vec<_> = view
        .strength
        .avasthas
        .iter()
        .filter(|r| r.moods.iter().any(|m| m == "mood-avastha-kopita"))
        .map(|r| r.graha)
        .collect();
    assert_eq!(kopita, ["graha-mercury", "graha-venus"]);
    for lang in [Lang::En, Lang::Hu] {
        let page = body(&format!("/{}/chart?{CHART_1}&topic=strength", lang.code())).await;
        let avasthas = section(&page, "avasthas");
        let note = t(lang, "kopita-to-judge");
        for row in view.strength.avasthas.iter() {
            let cells = &avasthas[avasthas
                .find(&format!(r#"data-body="{}""#, row.graha))
                .unwrap()..];
            let cells = &cells[..cells.find("</tr>").unwrap()];
            assert_eq!(
                cells.contains(&note),
                kopita.contains(&row.graha),
                "{cells}"
            );
        }
    }
}

#[test]
fn every_chapter_15_value_has_a_message() {
    use hora_chart::{
        ActivityStrength as S, AgeAvastha as A, Alertness as L, Kin as K, MoodAvastha as M,
        SayanaadiAvastha as Y, StrongerRule as R,
    };
    let en = keys(include_str!("../../../locales/en/main.ftl"));
    let hu = keys(include_str!("../../../locales/hu/main.ftl"));
    let ages = [A::Saisava, A::Kumaara, A::Yuva, A::Vriddha, A::Mrita]
        .map(|a| kb::message_id("age-avastha", a));
    let ids: Vec<String> = ages
        .iter()
        .flat_map(|id| [id.clone(), format!("{id}-share")])
        .chain([L::Jaagrita, L::Swapna, L::Sushupta].map(|a| kb::message_id("alertness", a)))
        .chain(
            [
                M::Deepta,
                M::Swastha,
                M::Mudita,
                M::Saanta,
                M::Deena,
                M::Duhkhita,
                M::Vikala,
                M::Khala,
                M::Kopita,
                M::Lajjita,
                M::Garvita,
                M::Kshudhita,
                M::Trishita,
                M::Kshobhita,
            ]
            .map(|m| kb::message_id("mood-avastha", m)),
        )
        .chain(
            [
                Y::Sayana,
                Y::Upavesana,
                Y::Netrapaani,
                Y::Prakaasana,
                Y::Gamana,
                Y::Aagamana,
                Y::Sabhaa,
                Y::Aagama,
                Y::Bhojana,
                Y::Nriyalipsaa,
                Y::Kautuka,
                Y::Nidraa,
            ]
            .map(|y| kb::message_id("sayanaadi-avastha", y)),
        )
        .chain(
            [S::Cheshta, S::Drishti, S::Vicheshta].map(|s| kb::message_id("activity-strength", s)),
        )
        .chain(
            [
                R::Basic,
                R::Joined,
                R::JupiterMercuryDispositor,
                R::Exalted,
                R::Mobility,
                R::Advancement,
            ]
            .map(|r| kb::message_id("stronger-rule", r)),
        )
        .chain(
            [
                K::Father,
                K::Mother,
                K::YoungerSiblings,
                K::MaternalRelatives,
                K::Husband,
                K::Wife,
                K::ElderSiblings,
            ]
            .map(|k| kb::message_id("kin", k)),
        )
        .collect();
    let missing: Vec<_> = ids
        .iter()
        .filter(|id| !en.contains(id.as_str()) || !hu.contains(id.as_str()))
        .collect();
    assert!(missing.is_empty(), "{missing:?}");
}

#[test]
fn every_chapter_14_value_has_a_message() {
    use hora_chart::{Longevity, MaheswaraException as M, PairsRule, RudraRule as R};
    let en = keys(include_str!("../../../locales/en/main.ftl"));
    let missing: Vec<_> = [Longevity::Short, Longevity::Middle, Longevity::Long]
        .map(|l| kb::message_id("longevity", l))
        .into_iter()
        .chain(
            [
                R::Conjoins,
                R::ExaltedOrOwn,
                R::JoinsExalted,
                R::Aspected,
                R::Advanced,
            ]
            .map(|r| kb::message_id("rudra-rule", r)),
        )
        .chain(
            [M::OwnOrExalted, M::Node, M::NodeReplaced]
                .map(|e| kb::message_id("maheswara-exception", e)),
        )
        .chain(
            [
                PairsRule::Agreement,
                PairsRule::LagnaAndHl,
                PairsRule::MoonAndSaturn,
            ]
            .map(|r| kb::message_id("pairs-rule", r)),
        )
        .chain(
            [
                "topic-longevity",
                "marakas",
                "maraka",
                "maraka-reasons",
                "maraka-sthana",
                "maraka-owns",
                "maraka-occupies",
                "maraka-aspects",
                "maraka-conjoins-lord",
                "maraka-aspects-lord",
                "rudra",
                "rudra-from-lagna",
                "rudra-from-seventh",
                "rudra-counted-by-table",
                "rudra-counted-normally",
                "rudra-decided-by",
                "rudra-rule-tie",
                "rudra-weaker",
                "rudra-weaker-afflicted",
                "rudra-weaker-unafflicted",
                "trishoola-rasis",
                "maheswara",
                "three-pairs",
                "longevity-caution",
                "pair",
                "pair-rasis",
                "pair-lords",
                "pair-moon-saturn",
                "pair-lagna-hl",
                "longevity",
                "longevity-years",
                "pairs-result",
                "pairs-rule",
                "paramaayush",
                "paramaayush-years",
                "paramaayush-none",
                "pairs-no-vedic-day",
                "eighth-lord-method",
                "eighth-lord-reference",
                "eighth-lord",
                "eighth-lord-house",
            ]
            .map(str::to_owned),
        )
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}

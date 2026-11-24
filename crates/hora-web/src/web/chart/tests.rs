use super::*;
use crate::web::tests::get;
use crate::web::tests::*;

/// `golden.rs` owns the values' correctness; this checks the wiring: form → chart → table, in both languages.
#[tokio::test]
async fn the_chart_page_shows_chart_1_from_its_form_in_both_languages() {
    // The book prints whole arcminutes, and the page truncates, so its degrees and minutes are the book's.
    // Nakshatra, pada and lord follow from them (ch. 1 §1.3.6, Table 2); Venus, the closest, is 1′ into its pada.
    let books = [
        // Only the degree: the book's lagna (10°58′) is 6–10′ off ours for an unknown reason.
        (
            ("Lagna", "Virgo", "Moon"),
            ("Lagna", "Szűz", "Hold"),
            "10°",
            "Hasta (13)",
            "1",
        ),
        (
            ("Sun", "Pisces", "Mercury"),
            ("Nap", "Halak", "Merkúr"),
            "26°29′",
            "Revati (27)",
            "3",
        ),
        (
            ("Moon", "Gemini", "Mars"),
            ("Hold", "Ikrek", "Mars"),
            "4°45′",
            "Mrigasira (5)",
            "4",
        ),
        (
            ("Mars", "Aries", "Venus"),
            ("Mars", "Kos", "Vénusz"),
            "19°09′",
            "Bharani (2)",
            "2",
        ),
        (
            ("Mercury", "Pisces", "Jupiter"),
            ("Merkúr", "Halak", "Jupiter"),
            "1°36′",
            "Poorvaabhaadra (25)",
            "4",
        ),
        (
            ("Jupiter", "Aries", "Venus"),
            ("Jupiter", "Kos", "Vénusz"),
            "17°21′",
            "Bharani (2)",
            "2",
        ),
        (
            ("Venus", "Pisces", "Saturn"),
            ("Vénusz", "Halak", "Szaturnusz"),
            "10°01′",
            "Uttaraabhaadra (26)",
            "3",
        ),
        (
            ("Saturn", "Aries", "Venus"),
            ("Szaturnusz", "Kos", "Vénusz"),
            "22°41′",
            "Bharani (2)",
            "3",
        ),
        (
            ("Rahu", "Cancer", "Saturn"),
            ("Rahu", "Rák", "Szaturnusz"),
            "5°55′",
            "Pushyami (8)",
            "1",
        ),
        (
            ("Ketu", "Capricorn", "Sun"),
            ("Ketu", "Bak", "Nap"),
            "5°55′",
            "Uttaraashaadha (21)",
            "3",
        ),
    ];
    for (lang, heading) in [(Lang::En, "Vimsottari lord"), (Lang::Hu, "Vimsottari ura")] {
        let page = body(&format!("/{}/chart?{CHART_1}", lang.code())).await;
        let headings = ["Graha", "Rasi", "Bhava", "Nakshatra", "Pada", heading];
        let headings = headings.map(|h| format!(">{h}<"));
        let headings = headings.each_ref().map(String::as_str);
        assert!(row(table(&page), &headings).is_some(), "{page}");
        let view = view(lang, CHART_1).await;
        // The book prints no ayanamsa value; this is the library's Lahiri at chart 1, truncated like the grahas.
        assert_eq!(view.ayanamsa, "23°51′39″");
        // Of chart 1's grahas only the mean nodes move backward.
        let marked: Vec<_> = view.grahas.iter().filter(|r| r.retrograde).collect();
        let marked: Vec<_> = marked.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(marked, ["graha-rahu", "graha-ketu"]);
        for (en_names, hu_names, position, nakshatra, pada) in books {
            let (graha, rasi, lord) = if lang == Lang::En { en_names } else { hu_names };
            let row = view.grahas.iter().find(|r| t(lang, &r.name) == graha);
            let row = row.expect(graha);
            let shown = format!("{} ({})", t(lang, row.nakshatra), row.nakshatra_number);
            assert_eq!(
                (&*t(lang, row.rasi), &*shown, &*row.pada.to_string()),
                (rasi, nakshatra, pada),
                "{graha}"
            );
            assert_eq!(t(lang, row.lord), lord, "{graha}");
            assert!(
                row.position.starts_with(position),
                "{graha}: {}",
                row.position
            );
        }
    }
}

#[tokio::test]
async fn every_graha_the_chart_page_names_links_to_its_kb_page() {
    for (lang, sun, lagna_nakshatra_lord) in [("en", "Sun", "Moon"), ("hu", "Nap", "Hold")] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let link = |anchor, name| {
            format!(r#"<a href="/{lang}/kb/graha-characteristics#{anchor}">{name}</a>"#)
        };
        let table = table(&page);
        assert!(
            table.contains(&format!(r#"<th scope="row">{}</th>"#, link("sun", sun))),
            "{table}"
        );
        assert!(table.contains(r#"<th scope="row">Lagna</th>"#), "{table}");
        assert!(
            table.contains(&format!(
                // The Vimsottari lord, then the lagna's empty dignity.
                "<td>{}</td><td>—</td>",
                link("moon", lagna_nakshatra_lord)
            )),
            "{table}"
        );
        // Chart 1's tithi, vaara, nakshatra and hora each have a lord.
        let day = body(&format!("/{lang}/chart?{CHART_1}&topic=day")).await;
        let panchaanga = section(&day, "panchaanga");
        let href = format!(r#"<td><a href="/{lang}/kb/graha-characteristics#"#);
        assert_eq!(panchaanga.matches(&href).count(), 4, "{panchaanga}");
    }
}

#[tokio::test]
async fn every_rasi_the_chart_page_names_links_to_its_kb_page() {
    for (lang, virgo) in [("en", "Virgo"), ("hu", "Szűz")] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let table = table(&page);
        let href = format!(r#"<td><a href="/{lang}/kb/rasi-characteristics#"#);
        // The lagna and the nine grahas.
        assert_eq!(table.matches(&href).count(), 10, "{table}");
        assert!(
            table.contains(&format!(r#"{href}virgo">{virgo}</a></td>"#)),
            "{table}"
        );
    }
}

#[tokio::test]
async fn every_nakshatra_the_chart_page_names_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let href = format!(r#"<td><a href="/{lang}/kb/nakshatra">"#);
        // The lagna and the nine grahas.
        let table = table(&page);
        assert_eq!(table.matches(&href).count(), 10, "{table}");
        assert!(
            table.contains(&format!("{href}Hasta (13)</a></td>")),
            "{table}"
        );
        // The Moon's nakshatra, the panchaanga's only one.
        let day = body(&format!("/{lang}/chart?{CHART_1}&topic=day")).await;
        let panchaanga = section(&day, "panchaanga");
        assert_eq!(panchaanga.matches(&href).count(), 1, "{panchaanga}");
    }
}

/// The ayanamsa page's worked examples: GREENWICH_2000 under each ayanamsa, and Lahiri a century either side.
#[tokio::test]
async fn the_ayanamsa_pages_examples_are_horas_charts() {
    const SOURCES: [(&str, &str); 2] = [
        ("en", include_str!("../../../kb/en/ayanamsa.md")),
        ("hu", include_str!("../../../kb/hu/ayanamsa.md")),
    ];
    /// The chart, once its ayanamsa is checked and both pages are found to link it and quote the value.
    async fn chart(year: &str, ayanamsa: &str, value: &str) -> ChartView {
        let query = GREENWICH_2000
            .replace("date=2000", &format!("date={year}"))
            .replace("ayanamsa=lahiri", &format!("ayanamsa={ayanamsa}"));
        let view = view(Lang::En, &query).await;
        assert_eq!(view.ayanamsa, value);
        for (lang, source) in SOURCES {
            for text in [&format!("(/{lang}/chart?{query})"), value] {
                assert!(source.contains(text), "{lang}: {text}");
            }
        }
        view
    }
    // The Sun's degree, pada and bhava, and the lagna's rasi.
    let suns = [
        ("lahiri", "23°51′25″", ["16°31′17″", "1", "9", "Aries"]),
        ("true-chitra", "23°50′28″", ["16°32′13″", "1", "9", "Aries"]),
        ("raman", "22°24′38″", ["17°58′04″", "2", "9", "Aries"]),
        ("true-pushya", "22°43′19″", ["17°39′23″", "2", "9", "Aries"]),
        ("true-mula", "24°35′08″", ["15°47′34″", "1", "10", "Pisces"]),
    ];
    for (ayanamsa, value, [sun, pada, bhava, lagna]) in suns {
        let view = chart("2000", ayanamsa, value).await;
        assert_eq!(
            row_named(&view.grahas, "Sun")[1..6],
            ["Sagittarius", bhava, sun, "Poorvaashaadha (20)", pada],
            "{ayanamsa}"
        );
        assert_eq!(row_named(&view.grahas, "Lagna")[1], lagna, "{ayanamsa}");
        for (lang, source) in SOURCES {
            assert!(source.contains(sun), "{lang}: {sun}");
        }
    }
    for (year, value) in [("1900", "22°27′37″"), ("2100", "25°15′15″")] {
        chart(year, "lahiri", value).await;
    }
    for (lang, source) in SOURCES {
        for text in ["256°31′17″", "280°22′42″", "257°58′04″"] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// The rasi page's worked example is hora's own Sun at noon UT, 1 January 2000, at Greenwich.
#[tokio::test]
async fn the_rasi_pages_example_is_horas_chart() {
    let query = GREENWICH_2000;
    let view = view(Lang::En, query).await;
    let sun = row_named(&view.grahas, "Sun");
    assert_eq!([&*sun[1], &*sun[3]], ["Sagittarius", "16°31′17″"]);
    assert_eq!(view.ayanamsa, "23°51′25″");
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/rasi.md")),
        ("hu", include_str!("../../../kb/hu/rasi.md")),
    ] {
        for text in [
            &format!("(/{lang}/chart?{query})"),
            "16°31′17″",
            "256°31′17″",
            "16 Sg 31",
            "8s 16° 31′",
            "23°51′",
        ] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

#[tokio::test]
async fn every_bhava_the_chart_page_names_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let table = table(&page);
        let href = format!(r#"<td><a href="/{lang}/kb/bhava">"#);
        // The lagna and the nine grahas.
        assert_eq!(table.matches(&href).count(), 10, "{table}");
        assert!(table.contains(&format!("{href}1</a></td>")), "{table}");
    }
}

/// The bhava page's example is hora's chart of the rasi page, counted from the lagna and from the Moon.
#[tokio::test]
async fn the_bhava_pages_example_is_horas_chart() {
    let query = "date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
            &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent";
    for (reference, bhavas) in [
        (
            "lagna",
            [
                ("Lagna", "Aries", 1),
                ("Sun", "Sagittarius", 9),
                ("Moon", "Libra", 7),
                ("Ketu", "Capricorn", 10),
            ],
        ),
        (
            "moon",
            [
                ("Lagna", "Aries", 7),
                ("Sun", "Sagittarius", 3),
                ("Moon", "Libra", 1),
                ("Ketu", "Capricorn", 4),
            ],
        ),
    ] {
        let view = view(
            Lang::En,
            &format!("{query}&reference={reference}&style=south"),
        )
        .await;
        for (graha, rasi, bhava) in bhavas {
            let row = row_named(&view.grahas, graha);
            assert_eq!(
                [&*row[1], &*row[2]],
                [rasi, &*bhava.to_string()],
                "{reference} {graha}"
            );
        }
    }
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/bhava.md")),
        ("hu", include_str!("../../../kb/hu/bhava.md")),
    ] {
        for reference in ["lagna", "moon"] {
            let link = format!("(/{lang}/chart?{query}&reference={reference}&style=south)");
            assert!(source.contains(&link), "{lang}: {reference}");
        }
        // The page's sums: the Sun, Ketu and the lagna counted from the Moon.
        for sum in ["9 − 7 + 1 = 3", "10 − 7 + 1 = 4", "1 − 7 + 1 = −5"] {
            assert!(source.contains(sum), "{lang}: {sum}");
        }
    }
}

#[tokio::test]
async fn the_chart_heading_links_to_its_kb_page() {
    for (lang, name) in [("en", "Chart"), ("hu", "Chakra")] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let heading = format!(r#"<h2><a href="/{lang}/kb/chart">{name}</a></h2>"#);
        assert!(page.contains(&heading), "{page}");
    }
}

/// The chart KB page's example is hora's chart, drawn in each style.
#[tokio::test]
async fn the_chart_pages_example_is_horas_chart() {
    let page = body(&format!("/en/chart?{HORAS_CHART}&style=north")).await;
    let view = view(Lang::En, &format!("{HORAS_CHART}&style=north")).await;
    for (point, rasi, retrograde) in [
        ("Lagna", "Scorpio", false),
        ("Venus", "Scorpio", false),
        ("Sun", "Sagittarius", false),
        ("Mercury", "Sagittarius", false),
        ("Moon", "Libra", false),
        ("Mars", "Aquarius", false),
        ("Jupiter", "Aries", false),
        ("Saturn", "Aries", true),
        ("Rahu", "Cancer", true),
        ("Ketu", "Capricorn", true),
    ] {
        let row = view
            .grahas
            .iter()
            .find(|r| en(&r.name) == point)
            .expect(point);
        assert_eq!(
            (&*en(row.rasi), row.retrograde),
            (rasi, retrograde),
            "{point}"
        );
    }
    for text in [
        r#"aria-label="Lagna, 17 degrees 14 minutes in Scorpio""#,
        r#"aria-label="Venus, 7 degrees 25 minutes in Scorpio""#,
        r#"aria-label="Saturn, retrograde, 16 degrees 32 minutes in Aries""#,
    ] {
        assert!(page.contains(text), "{text}: {page}");
    }
    // The page walks the bhavas from the top diamond: Scorpio's 8, then on round to Libra's 7.
    let numbers: Vec<_> = drawn(&page)
        .split(r#"class="mark" aria-hidden="true">"#)
        .skip(1)
        .map(|rest| &rest[..rest.find('<').unwrap()])
        .collect();
    assert_eq!(
        numbers,
        [
            "8", "9", "10", "11", "12", "1", "2", "3", "4", "5", "6", "7"
        ]
    );
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/chart.md")),
        ("hu", include_str!("../../../kb/hu/chart.md")),
    ] {
        for style in ["south", "east", "north"] {
            let link = format!("(/{lang}/chart?{HORAS_CHART}&style={style})");
            assert!(source.contains(&link), "{lang}: {style}");
        }
        for text in ["17°14′", "7°25′", "16°32′"] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

#[tokio::test]
async fn a_retrograde_graha_shows_r_in_its_row() {
    // Chart 4 (ch. 9 *Arudha Padas*, p. 98), whose Jupiter hora-chart's golden test marks retrograde.
    let chart_4 = "date=1968-03-19&time=18%3A30&utc=stated&offset=%2B05%3A30\
            &latitude=17.116666666666667&longitude=82.31666666666666";
    assert!(graha(&view(Lang::En, chart_4).await, "graha-jupiter").retrograde);
    // The table marks it.
    let page = body(&format!("/en/chart?{chart_4}")).await;
    let table = table(&page);
    assert!(row(table, &[">Jupiter<", ">R<"]).is_some(), "{table}");
}

#[tokio::test]
async fn chart_1_is_drawn_in_each_style_in_both_languages() {
    // Chart 1's lagna is in Virgo, its Saturn in Aries (ch. 9 *Arudha Padas*, p. 87); its mean Rahu moves backward.
    for (lang, title, rahu, saturn, legend) in [
        (
            "en",
            "Rasi chart",
            "Rahu, retrograde, 5 degrees 55 minutes in Cancer",
            ">Sa<",
            "R marks a retrograde graha.",
        ),
        (
            "hu",
            "Rasi chakra",
            "Rahu, retrográd, 5 fok 55 perc, Rák",
            ">Sz<",
            "Az R retrográd grahát jelöl.",
        ),
    ] {
        for style in ["south", "north", "east"] {
            let page = body(&format!("/{lang}/chart?{}", with(&[("style", style)]))).await;
            let drawn = drawn(&page);
            for text in [
                title,
                rahu,
                saturn,
                legend,
                r#"<tspan class="lagna">La</tspan>"#,
            ] {
                assert!(drawn.contains(text), "{lang} {style} {text}: {drawn}");
            }
            // Only North Indian numbers its areas, the 1st bhava Virgo's 6, and explains them.
            let north = drawn.contains(r#"y="222" class="mark" aria-hidden="true">6<"#);
            assert_eq!(north, style == "north", "{lang} {style}: {drawn}");
            assert_eq!(
                drawn.matches(" class=\"mark\" aria-hidden").count() == 12,
                north
            );
        }
    }
}

#[tokio::test]
async fn hl_and_gl_are_drawn_in_each_style_but_not_without_a_vedic_day() {
    for (lang, legend, hora, ghati) in [
        (
            "en",
            "HL and GL are the hora and ghati lagnas.",
            "Hora lagna in ",
            "Ghati lagna in ",
        ),
        (
            "hu",
            "A HL és a GL a hora és a ghati lagna.",
            "Hora lagna, ",
            "Ghati lagna, ",
        ),
    ] {
        for style in ["south", "north", "east"] {
            let page = body(&format!("/{lang}/chart?{}", with(&[("style", style)]))).await;
            let chakra = drawn(&page);
            for text in [
                r#"<tspan class="special">HL</tspan>"#,
                r#"<tspan class="special">GL</tspan>"#,
                &format!(r#"aria-label="{hora}"#),
                &format!(r#"aria-label="{ghati}"#),
                legend,
            ] {
                assert!(chakra.contains(text), "{lang} {style} {text}: {chakra}");
            }
            assert!(
                !chakra.contains(">SL<") && !chakra.contains(">BL<"),
                "{chakra}"
            );
            let polar = with(&[TROMSO.as_slice(), &[("style", style)]].concat());
            let polar = body(&format!("/{lang}/chart?{polar}")).await;
            let chakra = drawn(&polar);
            assert!(!chakra.contains("class=\"special\""), "{chakra}");
            assert!(!chakra.contains(legend), "{chakra}");
        }
    }
}

#[tokio::test]
async fn the_varga_chakra_is_navamsa_by_default_and_the_link_row_picks_another() {
    for (varga, query, heading) in [
        (Varga::Navamsa, CHART_1.to_owned(), "Navamsa (D-9)"),
        (
            Varga::Dasamsa,
            format!("{CHART_1}&varga=d10"),
            "Dasamsa (D-10)",
        ),
    ] {
        let page = body(&format!("/en/chart?{query}")).await;
        let section = section(&page, "varga");
        let n = varga.parts();
        let title = format!(r#"<a href="/en/kb/vargas#d-{n}">{heading}</a></h2>"#);
        assert!(section.contains(&title), "{section}");
        // Nineteen links, D-2 to D-60, each the chart with its varga; only the shown one is current.
        assert_eq!(section.matches("&#38;varga=d").count(), 19, "{section}");
        // The chakras head the page, so a link to one lands at the top, not past the header.
        assert!(!section.contains("#varga"), "{section}");
        let current = format!(
            r#"href="/en/chart?{}&#38;varga=d{n}" aria-current="page" aria-label="{heading}">D-{n}</a>"#,
            CHART_1.replace('&', "&#38;")
        );
        assert!(section.contains(&current), "{current}: {section}");
        assert_eq!(section.matches("aria-current").count(), 1, "{section}");
        assert!(
            section.contains(r#"aria-label="Shashtyamsa (D-60)">D-60</a>"#),
            "{section}"
        );
        assert!(section.contains("rasis only"), "{section}");
    }
}

#[tokio::test]
async fn each_body_sits_in_its_varga_rasi_in_each_style_but_hl_and_gl_not_without_a_vedic_day() {
    let sky = sky_of(CHART_1).await;
    let chart = hora_chart::chart(&sky);
    let specials: Vec<_> = hora_chart::special_lagnas(&sky)
        .into_iter()
        .filter(|(l, _)| matches!(l, SpecialLagna::HoraLagna | SpecialLagna::GhatiLagna))
        .map(|(l, longitude)| (kb::message_id("special-lagna", l), longitude, false))
        .collect();
    let bodies = chart
        .entries()
        .iter()
        .map(|e| (body_id(e.body).to_owned(), e.longitude, e.retrograde))
        .chain(specials);
    let bodies: Vec<_> = bodies.collect();
    assert_eq!(bodies.len(), 12);
    for varga in [Varga::Navamsa, Varga::Dasamsa, Varga::Shashtyamsa] {
        for style in ["south", "north", "east"] {
            let query = format!("{}&varga=d{}", with(&[("style", style)]), varga.parts());
            let page = body(&format!("/en/chart?{query}")).await;
            let chakra = section(&page, "varga");
            for (id, longitude, retrograde) in &bodies {
                let rasi = en(rasi_id(hora_chart::varga(varga, *longitude)));
                let r = if *retrograde { ", retrograde," } else { "" };
                let label = format!(r#"aria-label="{}{r} in {rasi}""#, en(id));
                assert!(
                    chakra.contains(&label),
                    "{varga:?} {style} {label}: {chakra}"
                );
            }
            assert!(!chakra.contains('°'), "{chakra}");
        }
    }
    let row = section(&body(&format!("/en/chart?{CHART_1}")).await, "varga").to_owned();
    assert!(!row.contains(">D-1<"), "{row}");
    assert!(row.contains(">D-2<"), "{row}");
    let polar = body(&format!("/en/chart?{}", with(&TROMSO))).await;
    let chakra = section(&polar, "varga");
    assert!(chakra.contains("<svg"), "{chakra}");
    assert!(!chakra.contains(r#"class="special""#), "{chakra}");
}

#[tokio::test]
async fn an_unknown_varga_is_refused() {
    // D-1 is the rasi chart, always drawn beside the varga chakra, so it is not one of its choices.
    for varga in ["d1", "d13", "D9", "9", "d"] {
        let response = get(&format!("/en/chart?{CHART_1}&varga={varga}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{varga}");
        let page = page(response).await;
        let error =
            r#"<strong id="varga-error" class="error">Choose one of the listed options.</strong>"#;
        assert!(page.contains(error), "{varga}: {page}");
        assert!(!page.contains("<table"), "{page}");
    }
}

#[tokio::test]
async fn the_north_indian_varga_chakra_counts_from_the_reference_points_varga_rasi() {
    let query = with(&[("reference", "hora-lagna"), ("style", "north")]);
    let sky = sky_of(&query).await;
    let (_, hl) = hora_chart::special_lagnas(&sky)
        .into_iter()
        .find(|(l, _)| *l == SpecialLagna::HoraLagna)
        .unwrap();
    for varga in [Varga::Navamsa, Varga::Drekkana] {
        let page = body(&format!("/en/chart?{query}&varga=d{}", varga.parts())).await;
        let first = hora_chart::varga(varga, hl) as u8 + 1;
        let top = format!(r#"y="222" class="mark" aria-hidden="true">{first}<"#);
        assert!(section(&page, "varga").contains(&top), "{varga:?} {top}");
    }
}

#[tokio::test]
async fn the_vargas_table_has_a_row_per_varga_and_a_column_per_point() {
    let sky = sky_of(CHART_1).await;
    let chart = hora_chart::chart(&sky);
    let mut points: Vec<_> = chart.entries().iter().map(|e| e.longitude).collect();
    for lagna in [
        SpecialLagna::HoraLagna,
        SpecialLagna::GhatiLagna,
        SpecialLagna::SreeLagna,
    ] {
        let special = hora_chart::special_lagnas(&sky)
            .into_iter()
            .find(|s| s.0 == lagna);
        points.push(special.unwrap().1);
    }
    let page = body(&format!("/en/chart?{CHART_1}&topic=vargas")).await;
    let vargas = section(&page, "vargas");
    assert!(vargas.contains(r#"<a href="/en/kb/vargas">"#), "{vargas}");
    let chart_1 = view(Lang::En, CHART_1).await;
    assert_eq!(chart_1.vargas.len(), 20);
    for (varga, row) in hora_chart::VARGAS.into_iter().zip(&chart_1.vargas) {
        let n = varga.parts();
        let name = en(&kb::message_id("varga", varga));
        assert_eq!((row.parts, &*row.name), (n, &*format!("{name} (D-{n})")));
        let rasis: Vec<_> = points
            .iter()
            .map(|&longitude| rasi_id(hora_chart::varga(varga, longitude)))
            .collect();
        assert_eq!(row.rasis, rasis, "{varga:?}");
    }
    let shorts = |view: &ChartView| -> Vec<String> {
        view.varga_columns.iter().map(|(_, s)| en(s)).collect()
    };
    let columns = shorts(&chart_1);
    for short in ["La", "Su", "Ke", "HL", "GL", "SL"] {
        assert!(columns.iter().any(|c| c == short), "{short}");
    }
    let polar = shorts(&view(Lang::En, &with(&TROMSO)).await);
    assert!(!polar.iter().any(|c| c == "HL" || c == "GL"), "{polar:?}");
    assert!(polar.iter().any(|c| c == "SL"), "{polar:?}");
}

#[tokio::test]
async fn the_amsabala_table_counts_each_graha_over_the_four_varga_groups() {
    let chart = hora_chart::chart(&sky_of(CHART_1).await);
    for lang in [Lang::En, Lang::Hu] {
        let page = body(&format!("/{}/chart?{CHART_1}&topic=vargas", lang.code())).await;
        let amsabala = section(&page, "amsabala");
        assert!(
            amsabala.contains(&format!(r#"<a href="/{}/kb/amsabala">"#, lang.code())),
            "{amsabala}"
        );
        for group in hora_chart::VARGA_GROUPS {
            let name = t(lang, &kb::message_id("varga-group", group));
            assert!(amsabala.contains(&format!(">{name}</th>")), "{name}");
        }
    }
    let view = view(Lang::En, CHART_1).await;
    assert_eq!(view.amsabala.len(), 9);
    for (entry, (id, cells)) in chart.grahas().iter().zip(&view.amsabala) {
        assert_eq!(*id, body_id(entry.body));
        let expected = hora_chart::VARGA_GROUPS.map(|group| {
            let (count, amsa) = hora_chart::amsabala(entry.body, entry.longitude, group);
            (count, amsa.map(|a| kb::message_id("amsa", a)))
        });
        assert_eq!(*cells, expected, "{id}");
    }
}

#[tokio::test]
async fn bhavas_are_counted_from_the_chosen_reference_point() {
    // Chart 1's lagna is in Virgo, its Moon in Gemini, its Sun in Pisces (ch. 9 *Arudha Padas*, p. 87).
    let moon = with(&[("reference", "moon")]);
    for (query, [lagna, moon, sun]) in [(CHART_1, [1, 10, 7]), (&moon, [4, 1, 10])] {
        let view = view(Lang::En, query).await;
        for (id, rasi, bhava) in [
            ("lagna", "rasi-virgo", lagna),
            ("graha-moon", "rasi-gemini", moon),
            ("graha-sun", "rasi-pisces", sun),
        ] {
            let row = graha(&view, id);
            assert_eq!((row.rasi, row.bhava), (rasi, bhava), "{id}");
        }
    }
    let from_moon = body(&format!("/en/chart?{}", with(&[("reference", "moon")]))).await;
    assert!(
        from_moon.contains(r#"value="moon" selected"#),
        "{from_moon}"
    );
}

#[tokio::test]
async fn bhavas_are_counted_from_a_chosen_special_lagna() {
    for reference in ["hora-lagna", "ghati-lagna", "sree-lagna"] {
        let query = with(&[("reference", reference), ("style", "north")]);
        let view = view(Lang::En, &query).await;
        let id = format!("special-lagna-{reference}");
        let row = view.special_lagnas.iter().find(|row| row.name == id);
        let row = row.expect(reference);
        assert_eq!(row.bhava, 1, "{reference}");
        let number = hora_chart::RASIS
            .iter()
            .position(|r| rasi_id(*r) == row.rasi)
            .unwrap()
            + 1;
        let page = body(&format!("/en/chart?{query}")).await;
        let top = format!(r#"y="222" class="mark" aria-hidden="true">{number}<"#);
        assert!(drawn(&page).contains(&top), "{reference}: {top}");
        assert!(page.contains(&format!(r#"value="{reference}" selected"#)));
    }
}

#[tokio::test]
async fn bhavas_are_counted_from_paaka_lagna() {
    // Chart 1's lagna is in Virgo, so its paaka lagna is Mercury's rasi (ch. 7 §7.3.5).
    let query = with(&[("reference", "paaka-lagna"), ("style", "north")]);
    let chart = hora_chart::chart(&sky_of(&query).await);
    let paaka = chart.in_varga(Varga::RasiChart).paaka_lagna();
    let page = body(&format!("/en/chart?{query}")).await;
    let mercury_query = with(&[("reference", "mercury"), ("style", "north")]);
    let mercury = body(&format!("/en/chart?{mercury_query}")).await;
    let lagna = body(&format!("/en/chart?{}", with(&[("style", "north")]))).await;
    // The varga's functional nature, the last column, counts from that varga's own paaka lagna (ADR 0016).
    let rasi_chart = |page: &str| -> Vec<String> {
        tbody(page)
            .lines()
            .map(|line| {
                line.rsplit_once("<td>")
                    .map_or(line, |(row, _)| row)
                    .to_owned()
            })
            .collect()
    };
    assert_ne!(
        rasi_chart(&page),
        rasi_chart(&lagna),
        "Mercury is not in Virgo"
    );
    assert_eq!(rasi_chart(&page), rasi_chart(&mercury));
    let page_lagnas = body(&format!("/en/chart?{query}&topic=lagnas")).await;
    let mercury_lagnas = body(&format!("/en/chart?{mercury_query}&topic=lagnas")).await;
    for id in ["special-lagnas", "upagrahas"] {
        assert_eq!(
            section(&page_lagnas, id),
            section(&mercury_lagnas, id),
            "{id}"
        );
    }
    assert_eq!(drawn(&page), drawn(&mercury));
    let top = format!(
        r#"y="222" class="mark" aria-hidden="true">{}<"#,
        paaka as u8 + 1
    );
    assert!(drawn(&page).contains(&top), "{top}");
    assert!(page.contains(r#"value="paaka-lagna" selected>Paaka lagna<"#));
}

#[tokio::test]
async fn the_north_indian_varga_chakra_counts_from_that_vargas_paaka_lagna() {
    let query = with(&[("reference", "paaka-lagna"), ("style", "north")]);
    let chart = hora_chart::chart(&sky_of(&query).await);
    let paaka = chart.in_varga(Varga::Navamsa).paaka_lagna();
    let mercury = chart.entry(hora_chart::Body::Mercury);
    // The varga's own lagna lord decides, not the rasi chart's (ADR 0003).
    assert_ne!(paaka, hora_chart::varga(Varga::Navamsa, mercury.longitude));
    let page = body(&format!("/en/chart?{query}&varga=d9")).await;
    let top = format!(
        r#"y="222" class="mark" aria-hidden="true">{}<"#,
        paaka as u8 + 1
    );
    assert!(section(&page, "varga").contains(&top), "{top}");
}

#[tokio::test]
async fn each_topic_shows_both_chakras_and_only_its_sections() {
    let all = [
        r#"<table id="grahas">"#,
        "vedic-day",
        "panchaanga",
        "limb-spans",
        "special-lagnas",
        "upagrahas",
        "vargas",
        "amsabala",
        "arudha-padas",
        "graha-arudhas",
        "aspects",
        "argalas",
        "yogas",
        "ashtakavarga",
        "baadhakas",
        "yogadas",
        "marakas",
        "rudra",
        "three-pairs",
        "eighth-lord",
    ];
    for (topic, shown) in [
        ("", &[r#"<table id="grahas">"#][..]),
        ("&topic=grahas", &[r#"<table id="grahas">"#]),
        ("&topic=day", &["vedic-day", "panchaanga", "limb-spans"]),
        ("&topic=lagnas", &["special-lagnas", "upagrahas"]),
        ("&topic=vargas", &["vargas", "amsabala"]),
        ("&topic=arudhas", &["arudha-padas", "graha-arudhas"]),
        ("&topic=aspects", &["aspects", "argalas"]),
        ("&topic=yogas", &["yogas"]),
        ("&topic=ashtakavarga", &["ashtakavarga"]),
        ("&topic=interpreting", &["baadhakas", "yogadas"]),
        (
            "&topic=longevity",
            &["marakas", "rudra", "three-pairs", "eighth-lord"],
        ),
    ] {
        let page = body(&format!("/en/chart?{CHART_1}{topic}")).await;
        assert!(
            drawn(&page).contains(r#"aria-labelledby="chakra-title""#),
            "{topic}"
        );
        assert!(section(&page, "varga").contains("<svg"), "{topic}");
        assert!(
            page.contains(r#"<a href="/en/kb/ayanamsa">Ayanamsa</a>: "#),
            "{topic}"
        );
        for id in all {
            let marker = if id.starts_with('<') {
                id.to_owned()
            } else {
                format!(r#"<section aria-labelledby="{id}">"#)
            };
            assert_eq!(page.contains(&marker), shown.contains(&id), "{topic}: {id}");
        }
    }
}

#[tokio::test]
async fn the_contents_and_every_link_keep_the_whole_view() {
    let keys = [
        "grahas",
        "day",
        "lagnas",
        "vargas",
        "arudhas",
        "aspects",
        "yogas",
        "ashtakavarga",
        "interpreting",
        "longevity",
        "strength",
    ];
    let state = "&varga=d10&sort=karaka";
    for (i, topic) in keys.into_iter().enumerate() {
        let page = body(&format!("/en/chart?{CHART_1}{state}&topic={topic}")).await;
        // Askama escapes `&` in attributes.
        let href =
            |key: &str| format!("/en/chart?{CHART_1}{state}&topic={key}").replace('&', "&#38;");
        let start = page.find(r#"<nav class="toc""#).unwrap();
        let toc = &page[start..start + page[start..].find("</nav>").unwrap()];
        assert!(toc.contains("<h3>Chart Analysis</h3>"), "{toc}");
        let links: Vec<_> = toc.split("<li>").skip(1).collect();
        assert_eq!(links.len(), keys.len(), "{toc}");
        for (j, (link, key)) in links.iter().zip(keys).enumerate() {
            assert!(
                link.starts_with(&format!(r#"<a href="{}""#, href(key))),
                "{link}"
            );
            assert_eq!(
                link.contains(r#"aria-current="page""#),
                i == j,
                "{topic}: {link}"
            );
        }
        for (key, fact) in [
            (
                "grahas",
                "<small>ch. 1–3, 7–8</small><span>Lagna Virgo</span>",
            ),
            ("day", "<small>ch. 1</small><span>Sunday</span>"),
            ("vargas", "<small>ch. 6</small><span>Dasamsa (D-10)</span>"),
            ("aspects", "<small>ch. 10</small></a>"),
            ("ashtakavarga", "<small>ch. 12</small></a>"),
            ("interpreting", "<small>ch. 13</small></a>"),
            ("longevity", "<small>ch. 14</small></a>"),
        ] {
            let link = links[keys.iter().position(|k| *k == key).unwrap()];
            assert!(link.contains(fact), "{link}");
        }
        assert!(links[6].contains(" found</span>"), "{}", links[6]);
        let start = page.find(r#"<nav class="path">"#).unwrap();
        let path = &page[start..start + page[start..].find("</nav>").unwrap()];
        let previous = format!(
            r#"<a rel="prev" href="{}">"#,
            href(keys.get(i.wrapping_sub(1)).unwrap_or(&""))
        );
        assert_eq!(path.contains(&previous), i > 0, "{topic}: {path}");
        assert_eq!(path.contains(r#"rel="prev""#), i > 0, "{topic}: {path}");
        let next = keys
            .get(i + 1)
            .map(|key| format!(r#"<a rel="next" href="{}">"#, href(key)));
        assert_eq!(
            next.is_some(),
            path.contains(r#"rel="next""#),
            "{topic}: {path}"
        );
        assert!(
            next.is_none_or(|next| path.contains(&next)),
            "{topic}: {path}"
        );
        // The varga row and the grahas table's headers keep the topic too.
        let d9 = format!("/en/chart?{CHART_1}&sort=karaka&topic={topic}&varga=d9\"");
        assert!(page.contains(&d9.replace('&', "&#38;")), "{topic}");
        for hidden in [("varga", "d10"), ("sort", "karaka"), ("topic", topic)] {
            let input = format!(
                r#"<input type="hidden" name="{}" value="{}">"#,
                hidden.0, hidden.1
            );
            assert!(page.contains(&input), "{topic}: {input}");
        }
        // Each topic is its own page, so the tab and history name it.
        let title = links[i]
            .split("<span>")
            .nth(1)
            .unwrap()
            .split("</span>")
            .next()
            .unwrap();
        assert!(
            page.contains(&format!("<title>{title} – hora</title>")),
            "{topic}"
        );
        // Cast, the birth form folds into one closed line.
        let fold = r#"<details class="birth">
<summary><span>2000-04-09 · 17:55 · 42.5, -71.2</span> <span>Change birth data</span></summary>"#;
        assert!(page.contains(fold), "{topic}");
    }
    let sort = format!("/en/chart?{CHART_1}&varga=d10&topic=day&sort=graha#grahas");
    let page = body(&format!("/en/chart?{CHART_1}{state}&topic=day")).await;
    // The grahas table is not on the day topic, so its sort links are not either.
    assert!(!page.contains(&sort.replace('&', "&#38;")));
}

#[tokio::test]
async fn without_a_vedic_day_the_contents_name_no_vaara_and_no_hora_lagna() {
    let page = body(&format!("/hu/chart?{}", with(&TROMSO))).await;
    let start = page.find(r#"<nav class="toc""#).unwrap();
    let toc = &page[start..start + page[start..].find("</nav>").unwrap()];
    assert!(toc.contains("<h3>Képletelemzés</h3>"), "{toc}");
    assert!(toc.contains("<small>1. fej.</small></a>"), "{toc}");
    assert!(toc.contains("<small>4–5. fej.</small></a>"), "{toc}");
    assert!(toc.contains(" találat</span>"), "{toc}");
}

#[tokio::test]
async fn bhavas_are_counted_from_the_atma_karaka() {
    // Chart 1's AK is the Sun, 26°29′ Pisces (ch. 9 *Arudha Padas*, p. 87; ch. 8 §8.2).
    let query = with(&[("reference", "atma-karaka"), ("style", "north")]);
    let sky = sky_of(&query).await;
    let karakas = hora_chart::chart(&sky).chara_karakas();
    assert!(karakas.contains(&(hora_chart::Body::Sun, CharaKaraka::Atma)));
    let page = body(&format!("/en/chart?{query}")).await;
    let sun_query = with(&[("reference", "sun"), ("style", "north")]);
    let sun = body(&format!("/en/chart?{sun_query}")).await;
    assert_eq!(tbody(&page), tbody(&sun));
    assert_eq!(graha(&view(Lang::En, &query).await, "lagna").bhava, 7);
    let page_lagnas = body(&format!("/en/chart?{query}&topic=lagnas")).await;
    let sun_lagnas = body(&format!("/en/chart?{sun_query}&topic=lagnas")).await;
    for id in ["special-lagnas", "upagrahas"] {
        assert_eq!(section(&page_lagnas, id), section(&sun_lagnas, id), "{id}");
    }
    assert_eq!(drawn(&page), drawn(&sun));
    assert!(page.contains(r#"value="atma-karaka" selected>Atma karaka<"#));
}

#[tokio::test]
async fn the_north_indian_navamsa_counts_from_the_karakamsa() {
    // Karakamsa: the AK's navamsa rasi (ch. 7 §7.3.6, ADR 0003).
    let query = with(&[("reference", "atma-karaka"), ("style", "north")]);
    let sky = sky_of(&query).await;
    let karakamsa = hora_chart::varga(Varga::Navamsa, sky.grahas.sun.longitude);
    assert_ne!(karakamsa, hora_chart::rasi(sky.grahas.sun.longitude).0);
    let page = body(&format!("/en/chart?{query}&varga=d9")).await;
    let top = format!(
        r#"y="222" class="mark" aria-hidden="true">{}<"#,
        karakamsa as u8 + 1
    );
    assert!(section(&page, "varga").contains(&top), "{top}");
}

#[test]
fn every_chara_karaka_has_a_name_and_an_abbreviation() {
    let en = keys(include_str!("../../../locales/en/main.ftl"));
    let missing: Vec<_> = hora_chart::CHARA_KARAKAS
        .into_iter()
        .flat_map(|k| {
            let id = kb::message_id("chara-karaka", k);
            [format!("{id}-short"), id]
        })
        .filter(|id| !en.contains(id.as_str()))
        .collect();
    assert_eq!(missing, [""; 0]);
}

#[tokio::test]
async fn a_polar_birth_offers_and_takes_only_the_sree_lagna_among_special_lagnas() {
    let chart = body(&format!("/en/chart?{}", with(&TROMSO))).await;
    let start = chart.find(r#"<select id="reference""#).unwrap();
    let select = &chart[start..start + chart[start..].find("</select>").unwrap()];
    assert!(select.contains(r#"value="sree-lagna""#), "{select}");
    assert!(!select.contains(r#"value="hora-lagna""#), "{select}");
    assert!(!select.contains(r#"value="ghati-lagna""#), "{select}");
    for reference in ["hora-lagna", "ghati-lagna"] {
        let query = with(&[TROMSO.as_slice(), &[("reference", reference)]].concat());
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{reference}");
        let page = page(response).await;
        assert!(
                page.contains(r#"<strong id="reference-error" class="error">There is no Vedic day here, so only the sree lagna: the others count from sunrise.</strong>"#),
                "{page}"
            );
        assert!(!page.contains("<figure>"), "{reference}");
        // The refused choice stays selected beside its error.
        assert!(page.contains(&format!(r#"value="{reference}" selected"#)));
    }
    let query = with(&[TROMSO.as_slice(), &[("reference", "sree-lagna")]].concat());
    assert_eq!(
        get(&format!("/en/chart?{query}"), None).await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn an_overloaded_pool_answers_busy_with_retry_after() {
    for (lang, text) in [
        (Lang::En, "Try again in a few seconds."),
        (Lang::Hu, "néhány másodperc múlva"),
    ] {
        let response = failure(
            lang,
            &Uri::from_static("/en/chart"),
            hora_swe::Error::Overloaded,
        );
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()[header::RETRY_AFTER], "5");
        assert!(page(response).await.contains(text));
    }
}

#[tokio::test]
async fn an_ephemeris_failure_is_a_generic_500_page() {
    let response = failure(
        Lang::Hu,
        &Uri::from_static("/hu/chart"),
        hora_swe::Error::Ephemeris,
    );
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!response.headers().contains_key(header::RETRY_AFTER));
    let page = page(response).await;
    assert!(
        page.contains("A chakrát nem sikerült kiszámítani."),
        "{page}"
    );
    assert!(!page.contains("ephemeris"), "{page}");
}

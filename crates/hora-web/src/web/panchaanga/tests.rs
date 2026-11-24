use super::*;
use crate::web::tests::*;

#[tokio::test]
async fn the_panchaangas_tithi_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{GREENWICH_2000}&topic=day")).await;
        let panchaanga = section(&page, "panchaanga");
        let href = format!(r#"<a href="/{lang}/kb/tithi">"#);
        assert_eq!(panchaanga.matches(&href).count(), 1, "{panchaanga}");
        assert!(
            panchaanga.contains(&format!("<td>{href}Krishna Ekadasi (26)</a></td>")),
            "{panchaanga}"
        );
    }
}

#[tokio::test]
async fn the_panchaangas_lunar_month_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{GREENWICH_2000}&topic=day")).await;
        let panchaanga = section(&page, "panchaanga");
        let href = format!(r#"<a href="/{lang}/kb/lunar-month">"#);
        assert_eq!(panchaanga.matches(&href).count(), 1, "{panchaanga}");
        assert!(
            panchaanga.contains(&format!("<td>{href}Maargasira</a></td>")),
            "{panchaanga}"
        );
    }
}

#[tokio::test]
async fn the_panchaangas_sun_moon_yoga_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{GREENWICH_2000}&topic=day")).await;
        let panchaanga = section(&page, "panchaanga");
        let href = format!(r#"<a href="/{lang}/kb/sun-moon-yoga">"#);
        assert_eq!(panchaanga.matches(&href).count(), 1, "{panchaanga}");
        assert!(
            panchaanga.contains(&format!("<td>{href}Dhriti (8)</a></td>")),
            "{panchaanga}"
        );
    }
}

#[tokio::test]
async fn the_panchaangas_karana_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{GREENWICH_2000}&topic=day")).await;
        let panchaanga = section(&page, "panchaanga");
        let href = format!(r#"<a href="/{lang}/kb/karana">"#);
        assert_eq!(panchaanga.matches(&href).count(), 1, "{panchaanga}");
        assert!(
            panchaanga.contains(&format!("<td>{href}Bava</a></td>")),
            "{panchaanga}"
        );
    }
}

#[tokio::test]
async fn the_panchaangas_vaara_and_hora_link_to_their_kb_page() {
    for (lang, saturday) in [("en", "Saturday"), ("hu", "szombat")] {
        let page = body(&format!("/{lang}/chart?{GREENWICH_2000}&topic=day")).await;
        let panchaanga = section(&page, "panchaanga");
        let href = format!(r#"<a href="/{lang}/kb/vaara-and-hora">"#);
        assert_eq!(panchaanga.matches(&href).count(), 2, "{panchaanga}");
        for value in [saturday, "4"] {
            assert!(
                panchaanga.contains(&format!("<td>{href}{value}</a></td>")),
                "{panchaanga}"
            );
        }
    }
}

/// The vaara and hora page's worked examples: GREENWICH_2000, and the same place six hours earlier, before sunrise.
#[tokio::test]
async fn the_vaara_and_hora_pages_examples_are_horas_charts() {
    let before_sunrise = GREENWICH_2000.replace("time=12%3A00", "time=06%3A00");
    for (query, sunrise, next_sunrise, vaara, vaara_lord, hora, hora_lord) in [
        (
            GREENWICH_2000,
            "08:05:47",
            "08:05:41",
            "Saturday",
            "Saturn",
            "4",
            "Sun",
        ),
        (
            &before_sunrise,
            "08:05:49",
            "08:05:46",
            "Friday",
            "Venus",
            "22",
            "Venus",
        ),
    ] {
        let view = view(Lang::En, query).await;
        assert!(vedic_day(&view, "vedic-day-sunrise").starts_with(sunrise));
        assert!(vedic_day(&view, "vedic-day-next-sunrise").starts_with(next_sunrise));
        for (name, value, lord) in [("vaara", vaara, vaara_lord), ("hora", hora, hora_lord)] {
            let shown = limb(Lang::En, &view, name);
            assert_eq!(shown, Some((value.into(), lord.into())), "{name}");
        }
        for (lang, source) in [
            ("en", include_str!("../../../kb/en/vaara-and-hora.md")),
            ("hu", include_str!("../../../kb/hu/vaara-and-hora.md")),
        ] {
            for text in [&format!("(/{lang}/chart?{query})"), sunrise, next_sunrise] {
                assert!(source.contains(text), "{lang}: {text}");
            }
        }
    }
}

#[tokio::test]
async fn the_vimsottari_lord_heading_links_to_the_dasa_kb_page() {
    for (lang, name) in [("en", "Vimsottari lord"), ("hu", "Vimsottari ura")] {
        let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
        let heading =
            format!(r#"<th scope="col" rowspan="2"><a href="/{lang}/kb/dasa">{name}</a></th>"#);
        assert!(table(&page).contains(&heading), "{page}");
    }
}

#[tokio::test]
async fn the_panchaanga_heading_links_to_its_kb_page() {
    for lang in ["en", "hu"] {
        let page = body(&format!("/{lang}/chart?{GREENWICH_2000}&topic=day")).await;
        let heading =
            format!(r#"<h2 id="panchaanga"><a href="/{lang}/kb/panchaanga">Panchaanga</a></h2>"#);
        assert!(section(&page, "panchaanga").contains(&heading), "{page}");
    }
}

/// The panchaanga page's worked example: GREENWICH_2000's five limbs.
#[tokio::test]
async fn the_panchaanga_pages_example_is_horas_chart() {
    let query = GREENWICH_2000;
    let view = view(Lang::En, query).await;
    for (name, value, lord) in [
        ("tithi", "Krishna Ekadasi (26)", "Mars"),
        ("vaara", "Saturday", "Saturn"),
        ("nakshatra", "Swaati (15)", "Rahu"),
        ("sun-moon-yoga", "Dhriti (8)", ""),
        ("karana", "Bava", ""),
    ] {
        let shown = limb(Lang::En, &view, name);
        assert_eq!(shown, Some((value.into(), lord.into())), "{name}");
    }
    for (name, start, end) in [
        ("tithi", "05:34:43", "07:50:31 (January 2, 2000)"),
        ("nakshatra", "10:37:39 (December 31, 1999)", "13:03:25"),
        ("sun-moon-yoga", "07:06:45", "07:43:37 (January 2, 2000)"),
        ("karana", "05:34:43", "18:40:01"),
    ] {
        assert_eq!(span(&view, name), (start, end), "{name}");
    }
    assert!(vedic_day(&view, "vedic-day-sunrise").starts_with("08:05:47"));
    assert!(vedic_day(&view, "vedic-day-next-sunrise").starts_with("08:05:41"));
    for (lang, source, limbs) in [
        (
            "en",
            include_str!("../../../kb/en/panchaanga.md"),
            [
                "- Tithi: Krishna Ekadasi (26), lord Mars,",
                "- Vaara: Saturday, lord Saturn,",
                "- Nakshatra: Swaati (15), lord Rahu,",
                "- Sun-Moon yoga: Dhriti (8),",
                "- Karana: Bava,",
            ],
        ),
        (
            "hu",
            include_str!("../../../kb/hu/panchaanga.md"),
            [
                "- Tithi: Krishna Ekadasi (26), ura a Mars,",
                "- Vaara: szombat, ura a Szaturnusz,",
                "- Nakshatra: Swaati (15), ura a Rahu,",
                "- Nap–Hold yoga: Dhriti (8),",
                "- Karana: Bava,",
            ],
        ),
    ] {
        for text in limbs.iter().chain(&[
            &format!("(/{lang}/chart?{query})")[..],
            "05:34:43",
            "07:50:31",
            "10:37:39",
            "13:03:25",
            "18:40:01",
            "07:06:45",
            "07:43:37",
            "08:05:47",
            "08:05:41",
        ]) {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// The karana page's worked example: GREENWICH_2000's tithi and karana.
#[tokio::test]
async fn the_karana_pages_example_is_horas_chart() {
    let query = GREENWICH_2000;
    let view = view(Lang::En, query).await;
    let value = |name| limb(Lang::En, &view, name).map(|(value, _)| value);
    assert_eq!(value("tithi").as_deref(), Some("Krishna Ekadasi (26)"));
    assert_eq!(value("karana").as_deref(), Some("Bava"));
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/karana.md")),
        ("hu", include_str!("../../../kb/hu/karana.md")),
    ] {
        for text in [
            &format!("(/{lang}/chart?{query})"),
            "302°56′57″",
            "2°56′57″",
            "3°03′03″",
            "Krishna Ekadasi",
            "Bava",
        ] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// The Sun-Moon yoga page's worked example: GREENWICH_2000's Sun and Moon.
#[tokio::test]
async fn the_sun_moon_yoga_pages_example_is_horas_chart() {
    let query = GREENWICH_2000;
    let view = view(Lang::En, query).await;
    for (graha, rasi, position) in [
        ("Sun", "Sagittarius", "16°31′17″"),
        ("Moon", "Libra", "19°28′14″"),
    ] {
        let row = row_named(&view.grahas, graha);
        assert_eq!([&*row[1], &*row[3]], [rasi, position], "{graha}");
    }
    assert_eq!(
        limb(Lang::En, &view, "sun-moon-yoga"),
        Some(("Dhriti (8)".into(), String::new()))
    );
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/sun-moon-yoga.md")),
        ("hu", include_str!("../../../kb/hu/sun-moon-yoga.md")),
    ] {
        for text in [
            &format!("(/{lang}/chart?{query})"),
            "256°31′17″",
            "199°28′14″",
            "455°59′31″",
            "95°59′31″",
            "2°39′31″",
            "10°40′29″",
            "Dhriti",
        ] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// The lunar month page's worked examples: GREENWICH_2000's month and its conjunction, and the conjunctions of 2023's two Sraavanas.
#[tokio::test]
async fn the_lunar_month_pages_examples_are_horas_charts() {
    const SOURCES: [(&str, &str); 2] = [
        ("en", include_str!("../../../kb/en/lunar-month.md")),
        ("hu", include_str!("../../../kb/hu/lunar-month.md")),
    ];
    /// The chart at `date` and `time` UT, once both pages are found to link it.
    async fn chart(date: &str, time: &str) -> ChartView {
        let query = GREENWICH_2000
            .replace("2000-01-01", date)
            .replace("12%3A00", time);
        for (lang, source) in SOURCES {
            let link = format!("(/{lang}/chart?{query})");
            assert!(source.contains(&link), "{lang}: {link}");
        }
        view(Lang::En, &query).await
    }
    // GREENWICH_2000, then the second after each conjunction: the Sun and the Moon together, Sukla Pratipat just started.
    let charts = [
        ("2000-01-01", "12%3A00", "Maargasira", None),
        (
            "1999-12-07",
            "22%3A32%3A23",
            "Maargasira",
            Some(("Scorpio", "21°31′26″", "21°31′26″", "22:32:22")),
        ),
        (
            "2023-07-17",
            "18%3A32%3A33",
            "Adhika Sraavana",
            Some(("Cancer", "0°45′29″", "0°45′29″", "18:32:32")),
        ),
        (
            "2023-08-16",
            "09%3A38%3A55",
            "Nija Sraavana",
            Some(("Cancer", "29°06′30″", "29°06′31″", "09:38:54")),
        ),
    ];
    for (date, time, month, conjunction) in charts {
        let view = chart(date, time).await;
        let value = |name| limb(Lang::En, &view, name).map(|(value, _)| value);
        assert_eq!(value("lunar-month").as_deref(), Some(month), "{date}");
        for (lang, source) in SOURCES {
            assert!(source.contains(month), "{lang}: {month}");
        }
        let Some((rasi, sun, moon, conjunction)) = conjunction else {
            continue;
        };
        assert_eq!(
            value("tithi").as_deref(),
            Some("Sukla Pratipat (1)"),
            "{date}"
        );
        // The chart is cast at the tithi's start: the conjunction's second, rounded up.
        assert_eq!(span(&view, "tithi").0, time.replace("%3A", ":"), "{date}");
        for (lang, source) in SOURCES {
            assert!(source.contains(conjunction), "{lang}: {conjunction}");
        }
        for (graha, value) in [("Sun", sun), ("Moon", moon)] {
            let row = row_named(&view.grahas, graha);
            assert_eq!([&*row[1], &*row[3]], [rasi, value], "{date} {graha}");
            for (lang, source) in SOURCES {
                assert!(source.contains(value), "{lang}: {value}");
            }
        }
    }
}

/// The tithi page's worked example: GREENWICH_2000's Sun and Moon.
#[tokio::test]
async fn the_tithi_pages_example_is_horas_chart() {
    let query = GREENWICH_2000;
    let view = view(Lang::En, query).await;
    for (graha, rasi, position) in [
        ("Sun", "Sagittarius", "16°31′17″"),
        ("Moon", "Libra", "19°28′14″"),
    ] {
        let row = row_named(&view.grahas, graha);
        assert_eq!([&*row[1], &*row[3]], [rasi, position], "{graha}");
    }
    assert_eq!(
        limb(Lang::En, &view, "tithi"),
        Some(("Krishna Ekadasi (26)".into(), "Mars".into()))
    );
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/tithi.md")),
        ("hu", include_str!("../../../kb/hu/tithi.md")),
    ] {
        for text in [
            &format!("(/{lang}/chart?{query})"),
            "256°31′17″",
            "199°28′14″",
            "302°56′57″",
            "2°56′57″",
            "Krishna Ekadasi",
        ] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// The nakshatra page's worked example is the rasi page's Sun.
#[tokio::test]
async fn the_nakshatra_pages_example_is_horas_chart() {
    let query = GREENWICH_2000;
    let sun = row_named(&view(Lang::En, query).await.grahas, "Sun");
    assert_eq!(
        [&*sun[3], &*sun[4], &*sun[5], &*sun[6]],
        ["16°31′17″", "Poorvaashaadha (20)", "1", "Venus"]
    );
    for (lang, source) in [
        ("en", include_str!("../../../kb/en/nakshatra.md")),
        ("hu", include_str!("../../../kb/hu/nakshatra.md")),
    ] {
        for text in [
            &format!("(/{lang}/chart?{query})"),
            "16°31′17″",
            "256°31′17″",
            "3°11′17″",
            "Poorvaashaadha",
            "13 Sg 20",
        ] {
            assert!(source.contains(text), "{lang}: {text}");
        }
    }
}

/// The panchaanga's limb named `name` in `lang`, as `limb` gives it.
fn limb_named(lang: Lang, view: &ChartView, name: &str) -> Option<(String, String)> {
    let id = view
        .panchaanga
        .iter()
        .find(|limb| t(lang, limb.name) == name)?
        .name;
    limb(lang, view, id)
}

/// The (start, end) of the limb named `name` in `lang`.
fn span_named<'a>(lang: Lang, view: &'a ChartView, name: &str) -> (&'a str, &'a str) {
    let id = view
        .limb_spans
        .iter()
        .find(|s| t(lang, s.name) == name)
        .expect(name)
        .name;
    span(view, id)
}

#[tokio::test]
async fn chart_1_shows_its_vedic_day_in_its_own_clock() {
    // Sunrise rounded up, the rest truncated; Jagannatha Hora's 19:20:09 sunset is 1–2 s later as its Sun has no aberration.
    for (lang, rows) in [
        ("en", ["Sunrise", "Sunset", "Next sunrise"]),
        ("hu", ["Napkelte", "Napnyugta", "Következő napkelte"]),
    ] {
        let page = body(&format!("/{lang}/chart?{CHART_1}&topic=day")).await;
        let day = section(&page, "vedic-day");
        assert!(
            day.contains(if lang == "en" {
                ">Vedic day</h2>"
            } else {
                ">Védikus nap</h2>"
            }),
            "{day}"
        );
        assert!(!day.contains("<p>"), "{day}");
        let (lang, next) = if lang == "en" {
            (Lang::En, "06:11:27 (April 10, 2000)")
        } else {
            (Lang::Hu, "06:11:27 (2000. április 10.)")
        };
        let view = view(lang, CHART_1).await;
        let day = view.vedic_day.expect("a Vedic day");
        let shown: Vec<_> = day
            .iter()
            .map(|(id, time)| (t(lang, id), &**time))
            .collect();
        let names = rows.map(str::to_owned);
        let expected: Vec<_> = names
            .into_iter()
            .zip(["06:13:08", "19:20:07", next])
            .collect();
        assert_eq!(shown, expected);
    }
}

#[tokio::test]
async fn a_birth_before_sunrise_sees_the_previous_date_on_its_sunrise_and_sunset() {
    // Chart 3 (ch. 9 *Arudha Padas*, p. 97): 1926-12-25 05:12 at UTC+5:30, 78°E10′, 26°N14′; sunrise is 07:01.
    let chart_3 = "date=1926-12-25&time=05%3A12&utc=stated&offset=%2B05%3A30\
            &latitude=26.233333333333334&longitude=78.16666666666667";
    for (lang, date) in [
        (Lang::En, " (December 24, 1926)"),
        (Lang::Hu, " (1926. december 24.)"),
    ] {
        let view = view(lang, chart_3).await;
        let day = view.vedic_day.expect("a Vedic day");
        for (id, time) in &day {
            // Only the next sunrise is on the birth's date, so it alone has no date.
            if *id == "vedic-day-next-sunrise" {
                assert!(!time.contains('('), "{id}: {time}");
            } else {
                assert!(time.ends_with(date), "{id}: {time}");
            }
        }
    }
}

#[tokio::test]
async fn a_polar_night_birth_shows_its_chart_and_why_it_has_no_vedic_day() {
    let tromso = with(&TROMSO);
    let view = view(Lang::En, &tromso).await;
    assert_eq!(view.grahas[0].name, "lagna");
    assert!(view.vedic_day.is_none());
    let en = body(&format!("/en/chart?{tromso}&topic=day")).await;
    let day = section(&en, "vedic-day");
    assert!(
        day.contains("There is no Vedic day at this place on this date"),
        "{day}"
    );
    assert!(!day.contains("<table"), "{day}");
    let hu = body(&format!("/hu/chart?{tromso}&topic=day")).await;
    assert!(
        section(&hu, "vedic-day").contains("Ezen a helyen ezen a napon nincs védikus nap"),
        "{hu}"
    );
}

#[tokio::test]
async fn chart_1_shows_its_panchaanga_in_both_languages() {
    // Sun 26°29′ Pi, Moon 4°45′ Ge (p. 87): Moon − Sun 68°16′, sum 61°14′, so 6th tithi, 5th nakshatra, 5th yoga, 12th half.
    // 2000-04-09 is a Sunday; the birth is 11 h 42 min after a 23 h 58 min Vedic day's sunrise, in the 12th hora.
    // The month started at the new Moon of 2000-04-04 in Pisces: Chaitra.
    for (lang, heading, rows) in [
        (
            "en",
            ">Panchaanga</a></h2>",
            [
                ["Tithi", "Sukla Shashti (6)", "Venus"],
                ["Vaara", "Sunday", "Sun"],
                ["Nakshatra", "Mrigasira (5)", "Mars"],
                ["Sun-Moon yoga", "Sobhana (5)", ""],
                ["Karana", "Taitula", ""],
                ["Hora", "12", "Saturn"],
                ["Lunar month", "Chaitra", ""],
            ],
        ),
        (
            "hu",
            ">Panchaanga</a></h2>",
            [
                ["Tithi", "Sukla Shashti (6)", "Vénusz"],
                ["Vaara", "vasárnap", "Nap"],
                ["Nakshatra", "Mrigasira (5)", "Mars"],
                ["Nap–Hold yoga", "Sobhana (5)", ""],
                ["Karana", "Taitula", ""],
                ["Hora", "12", "Szaturnusz"],
                ["Holdhónap", "Chaitra", ""],
            ],
        ),
    ] {
        let page = body(&format!("/{lang}/chart?{CHART_1}&topic=day")).await;
        let panchaanga = section(&page, "panchaanga");
        assert!(panchaanga.contains(heading), "{panchaanga}");
        assert!(!panchaanga.contains("<p>"), "{panchaanga}");
        let lang = if lang == "en" { Lang::En } else { Lang::Hu };
        let view = view(lang, CHART_1).await;
        for [name, value, lord] in rows {
            let shown = limb_named(lang, &view, name);
            assert_eq!(shown, Some((value.into(), lord.into())), "{name}");
        }
    }
}

#[tokio::test]
async fn chart_1_shows_when_its_limbs_started_and_end_in_its_own_clock() {
    // Not from the book, which prints no limb times: Mrigasira ends as the Moon's remaining 1°55′ at 14.2°/day says.
    for (lang, heading, yoga, [april_8, april_10]) in [
        (
            "en",
            ">Start and end of the limbs</h2>",
            "Sun-Moon yoga",
            [" (April 8, 2000)", " (April 10, 2000)"],
        ),
        (
            "hu",
            ">Az elemek kezdete és vége</h2>",
            "Nap–Hold yoga",
            [" (2000. április 8.)", " (2000. április 10.)"],
        ),
    ] {
        let page = body(&format!("/{lang}/chart?{CHART_1}&topic=day")).await;
        let spans = section(&page, "limb-spans");
        assert!(spans.contains(heading), "{spans}");
        let lang = if lang == "en" { Lang::En } else { Lang::Hu };
        let view = view(lang, CHART_1).await;
        let times = [
            ("03:00:52", format!("00:41:32{april_10}")),
            (&*format!("22:44:19{april_8}"), "21:08:41".into()),
            ("05:28:29", format!("02:28:13{april_10}")),
            ("13:50:12", format!("00:41:32{april_10}")),
        ];
        for (name, (start, end)) in ["Tithi", "Nakshatra", yoga, "Karana"]
            .into_iter()
            .zip(times)
        {
            assert_eq!(span_named(lang, &view, name), (start, &*end), "{name}");
        }
    }
}

#[tokio::test]
async fn a_birth_at_either_shown_end_of_a_limb_has_chart_1s_limb() {
    // Starts are shown rounded up and ends truncated, so both shown seconds are inside.
    for (name, value, moments) in [
        (
            "Tithi",
            "Sukla Shashti (6)",
            [("2000-04-09", "03:00:52"), ("2000-04-10", "00:41:32")],
        ),
        (
            "Nakshatra",
            "Mrigasira (5)",
            [("2000-04-08", "22:44:19"), ("2000-04-09", "21:08:41")],
        ),
        (
            "Sun-Moon yoga",
            "Sobhana (5)",
            [("2000-04-09", "05:28:29"), ("2000-04-10", "02:28:13")],
        ),
        (
            "Karana",
            "Taitula",
            [("2000-04-09", "13:50:12"), ("2000-04-10", "00:41:32")],
        ),
    ] {
        for (date, time) in moments {
            let query = with(&[("date", date), ("time", &time.replace(':', "%3A"))]);
            let view = view(Lang::En, &query).await;
            let shown = limb_named(Lang::En, &view, name).map(|(value, _)| value);
            assert_eq!(shown.as_deref(), Some(value), "{date} {time}");
        }
    }
}

#[tokio::test]
async fn a_birth_in_1999s_first_jyeshtha_is_in_adhika_jyeshtha() {
    // ch. 1 §1.3.8.2: new Moons in Taurus on 1999-05-15 and 06-14.
    let query =
        "date=1999-05-20&time=12%3A00&utc=stated&offset=%2B05%3A30&latitude=28.6&longitude=77.2";
    for (lang, name) in [(Lang::En, "Lunar month"), (Lang::Hu, "Holdhónap")] {
        let view = view(lang, query).await;
        let shown = limb_named(lang, &view, name).map(|(value, _)| value);
        assert_eq!(shown.as_deref(), Some("Adhika Jyeshtha"), "{lang:?}");
    }
}

#[tokio::test]
async fn a_birth_before_sunrise_has_the_previous_days_vaara() {
    // Chart 3 (ch. 9 *Arudha Padas*, p. 97): born on Saturday 1926-12-25 at 05:12, before the 07:01 sunrise.
    let chart_3 = "date=1926-12-25&time=05%3A12&utc=stated&offset=%2B05%3A30\
            &latitude=26.233333333333334&longitude=78.16666666666667";
    for (lang, vaara, lord) in [
        (Lang::En, "Friday", "Venus"),
        (Lang::Hu, "péntek", "Vénusz"),
    ] {
        let view = view(lang, chart_3).await;
        let shown = limb(lang, &view, "vaara");
        assert_eq!(shown, Some((vaara.into(), lord.into())), "{lang:?}");
    }
}

#[tokio::test]
async fn a_birth_at_chart_1s_shown_sunrise_has_its_sunday() {
    let view = view(Lang::En, &with(&[("time", "06%3A13%3A08")])).await;
    let shown = limb(Lang::En, &view, "vaara").map(|(value, _)| value);
    assert_eq!(shown.as_deref(), Some("Sunday"));
}

#[tokio::test]
async fn a_polar_night_birth_has_a_tithi_but_no_vaara_or_hora() {
    let tromso = with(&TROMSO);
    let view = view(Lang::En, &tromso).await;
    for (name, shown) in [
        ("tithi", true),
        ("karana", true),
        ("vaara", false),
        ("hora", false),
    ] {
        assert_eq!(limb(Lang::En, &view, name).is_some(), shown, "{name}");
    }
    for (lang, none) in [
        ("en", "There is no Vedic day here, so no vaara or hora."),
        ("hu", "Itt nincs védikus nap, így vaara és hora sincs."),
    ] {
        let page = body(&format!("/{lang}/chart?{tromso}&topic=day")).await;
        let panchaanga = section(&page, "panchaanga");
        assert!(
            panchaanga.contains(&format!("<p>{none}</p>")),
            "{panchaanga}"
        );
    }
}

#[tokio::test]
async fn a_birth_in_tokyo_stated_in_ut_warns_that_the_vaara_may_be_a_day_off() {
    // Wednesday 2000-01-05 08:00 JST: sunrise was 21:51 UT on Tuesday.
    let tokyo_in_ut = with(&[
        ("date", "2000-01-04"),
        ("time", "23%3A00"),
        ("offset", "%2B00%3A00"),
        ("latitude", "35.68"),
        ("longitude", "139.69"),
    ]);
    for (lang, warning) in [
        ("en", "The stated UTC offset is far from this place"),
        (
            "hu",
            "A megadott UTC-eltérés messze van e hely helyi idejétől",
        ),
    ] {
        let page = body(&format!("/{lang}/chart?{tokyo_in_ut}&topic=day")).await;
        let panchaanga = section(&page, "panchaanga");
        assert!(panchaanga.contains(warning), "{panchaanga}");
    }
}

#[tokio::test]
async fn kiritimati_at_plus_14_has_its_own_dates_vaara_and_no_warning() {
    // 157°W, but on the Asian side of the date line since 1995.
    let kiritimati = with(&[
        ("date", "2000-01-05"),
        ("time", "12%3A00"),
        ("offset", "%2B14%3A00"),
        ("latitude", "1.87"),
        ("longitude", "-157.4"),
    ]);
    let view = view(Lang::En, &kiritimati).await;
    let shown = limb(Lang::En, &view, "vaara").map(|(value, _)| value);
    assert_eq!(shown.as_deref(), Some("Wednesday"));
    assert!(!view.far_offset);
}

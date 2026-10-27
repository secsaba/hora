//! The knowledge base (ADR 0004): one Markdown page per concept per language, compiled in, served at `/{lang}/kb/{slug}`.

use askama::Template;
use axum::extract::Path;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use fluent_templates::Loader;
use fluent_templates::fluent_bundle::FluentValue;
use pulldown_cmark::{Event, Options, Parser, Tag};

use super::{
    LOCALES, Lang, Localized, Message, body_id, karana_id, lunar_month_id, nakshatra_id, paksha_id,
    rasi_id, render, sun_moon_yoga_id, switch_href, tithi_id,
};

/// A page: its slug (the glossary term in the book's spelling), where the book teaches it, and its Markdown in each language.
pub struct Page {
    pub slug: &'static str,
    chapter: &'static str,
    section: &'static str,
    en: &'static str,
    hu: &'static str,
}

impl Page {
    fn source(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => self.en,
            Lang::Hu => self.hu,
        }
    }

    /// The text of the page's leading `# ` heading.
    pub fn title(&self, lang: Lang) -> &'static str {
        let first = self.source(lang).lines().next().unwrap_or_default();
        first.strip_prefix("# ").unwrap_or(first)
    }
}

/// A listed page whose file is missing in either language fails the build.
macro_rules! pages {
    ($(($slug:literal, $chapter:literal, $section:literal)),* $(,)?) => {
        &[$(Page {
            slug: $slug,
            chapter: $chapter,
            section: $section,
            en: include_str!(concat!("../../kb/en/", $slug, ".md")),
            hu: include_str!(concat!("../../kb/hu/", $slug, ".md")),
        }),*]
    };
}

/// The learning path: every page, in the book's order.
pub const PAGES: &[Page] = pages![
    ("jyotish", "1", "1.1–1.2"),
    ("graha", "1", "1.3.1"),
    ("rasi", "1", "1.3.2"),
    ("bhava", "1", "1.3.3"),
    ("chart", "1", "1.3.4"),
    ("varga-chakra", "1", "1.3.5"),
    ("nakshatra", "1", "1.3.6"),
    ("tithi", "1", "1.3.8.1"),
    ("lunar-month", "1", "1.3.8.2"),
    ("sun-moon-yoga", "1", "1.3.9"),
    ("karana", "1", "1.3.10"),
    ("ayanamsa", "1", "1.3.13"),
];

/// `source` as HTML: `{{table:…}}` placeholders filled from the calculations' data, raw HTML escaped, links only to this site or https.
fn to_html(source: &str, lang: Lang) -> Result<String, String> {
    let mut markdown = String::new();
    let mut rest = source;
    while let Some(start) = rest.find("{{") {
        markdown.push_str(&rest[..start]);
        let end = rest[start..].find("}}").ok_or("a {{ without its }}")? + start;
        let name = &rest[start + 2..end];
        markdown.push_str(&table(name, lang).ok_or_else(|| format!("unknown placeholder {name}"))?);
        rest = &rest[end + 2..];
    }
    markdown.push_str(rest);
    let events = Parser::new_ext(&markdown, Options::ENABLE_TABLES)
        .map(|event| match event {
            // The files are trusted, but the renderer must never pass HTML through.
            Event::Html(html) | Event::InlineHtml(html) => Ok(Event::Text(html)),
            Event::Start(Tag::Link { ref dest_url, .. } | Tag::Image { ref dest_url, .. })
                if !(dest_url.starts_with('/') && !dest_url.starts_with("//")
                    || dest_url.starts_with('#')
                    || dest_url.starts_with("https://")) =>
            {
                Err(format!("a link to {dest_url}"))
            }
            event => Ok(event),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    Ok(html)
}

/// A placeholder's Markdown table, names translated as on the chart page; `None` for an unknown one.
fn table(name: &str, lang: Lang) -> Option<String> {
    let t = |id| LOCALES.lookup(&lang.id(), id);
    match name {
        "table:rasis" => {
            let header = [
                "rasi",
                "rasi-sanskrit-name",
                "rasi-symbol",
                "rasi-start",
                "rasi-end",
            ]
            .map(t);
            let mut table = format!("\n\n| {} |\n|---|---|---|---|---|\n", header.join(" | "));
            for (i, rasi) in hora_chart::RASIS.into_iter().enumerate() {
                let id = rasi_id(rasi);
                let [sanskrit, short] = ["sanskrit", "short"]
                    .map(|part| LOCALES.lookup(&lang.id(), &format!("{id}-{part}")));
                let start = i * hora_chart::RASI_SPAN as usize;
                // As the book's Table 1 prints them: a rasi ends a second short of the next one's start.
                let end = start + hora_chart::RASI_SPAN as usize - 1;
                table += &format!(
                    "| {} | {sanskrit} | {short} | {start}°00′00″ | {end}°59′59″ |\n",
                    t(id)
                );
            }
            Some(table)
        }
        "table:nakshatras" => {
            let header = [
                "nakshatra",
                "nakshatra-common",
                "nakshatra-start",
                "nakshatra-end",
                "vimsottari-lord",
                "nakshatra-deity",
            ]
            .map(t);
            let mut table = format!(
                "\n\n| {} |\n|---|---|---|---|---|---|\n",
                header.join(" | ")
            );
            // As Table 2 prints them, "13 Ar 20": an end at a rasi's edge is 30° into the rasi before it.
            let rasi_span = hora_chart::RASI_SPAN as usize * 60;
            let short = |arcminutes: usize, end: bool| {
                let rasi = (arcminutes - usize::from(end)) / rasi_span;
                let inside = arcminutes - rasi * rasi_span;
                let rasi = LOCALES.lookup(
                    &lang.id(),
                    &format!("{}-short", rasi_id(hora_chart::RASIS[rasi])),
                );
                format!("{:02} {rasi} {:02}", inside / 60, inside % 60)
            };
            for (i, nakshatra) in hora_chart::NAKSHATRAS.into_iter().enumerate() {
                let id = nakshatra_id(nakshatra);
                let [common, deity] = ["common", "deity"]
                    .map(|part| LOCALES.lookup(&lang.id(), &format!("{id}-{part}")));
                // 13°20′ is 800′, 4/9 of a rasi.
                table += &format!(
                    "| {} | {common} | {} | {} | {} | {deity} |\n",
                    t(id),
                    short(i * 800, false),
                    short((i + 1) * 800, true),
                    t(body_id(hora_chart::vimsottari_lord(nakshatra))),
                );
            }
            Some(table)
        }
        "table:tithis" => {
            let header = ["paksha-sukla", "paksha-krishna", "tithi", "lord"].map(t);
            let mut table = format!("\n\n| {} |\n|---|---|---|---|\n", header.join(" | "));
            // As Table 3 prints them: Paurnami only in Sukla paksha, Amavasya only in Krishna.
            for (i, tithi) in hora_chart::TITHIS.into_iter().enumerate() {
                let [sukla, krishna] = match tithi {
                    hora_chart::Tithi::Paurnami => ["15".into(), "—".into()],
                    hora_chart::Tithi::Amavasya => ["—".into(), "30".into()],
                    _ => [i + 1, i + 16].map(|index| index.to_string()),
                };
                table += &format!(
                    "| {sukla} | {krishna} | {} | {} |\n",
                    t(tithi_id(tithi)),
                    t(body_id(hora_chart::tithi_lord(tithi))),
                );
            }
            Some(table)
        }
        "table:lunar-months" => {
            let header = [
                "lunar-month-conjunction",
                "lunar-month",
                "lunar-month-full-moon",
                "lunar-month-when",
            ]
            .map(t);
            let mut table = format!("\n\n| {} |\n|---|---|---|---|\n", header.join(" | "));
            let gregorian = |month: usize| {
                let args = [("month".into(), FluentValue::from(month))].into();
                LOCALES.lookup_with_args(&lang.id(), "gregorian-month-short", &args)
            };
            for (i, month) in hora_chart::LUNAR_MONTHS.into_iter().enumerate() {
                let nakshatras: Vec<_> = hora_chart::full_moon_nakshatras(month)
                    .iter()
                    .map(|&nakshatra| t(nakshatra_id(nakshatra)))
                    .collect();
                // Pisces starts Chaitra, which Table 4 places in March/April.
                table += &format!(
                    "| {} | {} | {} | {}/{} |\n",
                    t(rasi_id(hora_chart::RASIS[(i + 11) % 12])),
                    t(lunar_month_id(month)),
                    nakshatras.join("/"),
                    gregorian((i + 2) % 12 + 1),
                    gregorian((i + 3) % 12 + 1),
                );
            }
            Some(table)
        }
        "table:sun-moon-yogas" => {
            let header = [
                "sun-moon-yoga-number",
                "sun-moon-yoga",
                "sun-moon-yoga-meaning",
            ]
            .map(t);
            let mut table = format!("\n\n| {} |\n|---|---|---|\n", header.join(" | "));
            for (i, yoga) in hora_chart::SUN_MOON_YOGAS.into_iter().enumerate() {
                let id = sun_moon_yoga_id(yoga);
                let meaning = LOCALES.lookup(&lang.id(), &format!("{id}-meaning"));
                table += &format!("| {} | {} | {meaning} |\n", i + 1, t(id));
            }
            Some(table)
        }
        "table:karanas" => {
            let header = ["tithi", "karana-first-half", "karana-second-half"].map(t);
            let mut table = format!("\n\n| {} |\n|---|---|---|\n", header.join(" | "));
            // Each tithi's halves from the Moon 3° and 9° into it, ahead of a Sun at 0°.
            for start in (0..360).step_by(12) {
                let (index, paksha, tithi) = hora_chart::tithi(0.0, f64::from(start));
                let [first, second] = [3, 9]
                    .map(|into| t(karana_id(hora_chart::karana(0.0, f64::from(start + into)))));
                table += &format!(
                    "| {} {} ({index}) | {first} | {second} |\n",
                    t(paksha_id(paksha)),
                    t(tithi_id(tithi)),
                );
            }
            Some(table)
        }
        "table:grahas" => {
            let mut table = format!("\n\n| {} | {} |\n|---|---|\n", t("graha"), t("short-label"));
            for graha in hora_chart::GRAHAS {
                let id = body_id(graha);
                let short = LOCALES.lookup(&lang.id(), &format!("{id}-short"));
                table += &format!("| {} | {short} |\n", t(id));
            }
            Some(table)
        }
        _ => None,
    }
}

trait BookReference: Localized {
    /// "chapter 1, § 1.3.2", in the page's language.
    fn chapter_section(&self, page: &Page) -> String {
        let args = [("chapter", page.chapter), ("section", page.section)]
            .map(|(name, value)| (name.into(), FluentValue::from(value)))
            .into();
        LOCALES.lookup_with_args(&self.lang().id(), "kb-chapter-section", &args)
    }
}

#[derive(Template)]
#[template(path = "kb_index.html")]
struct Index {
    lang: Lang,
    switch_href: String,
}

impl Localized for Index {
    fn lang(&self) -> Lang {
        self.lang
    }
}

impl BookReference for Index {}

#[derive(Template)]
#[template(path = "kb_page.html")]
struct KbPage {
    lang: Lang,
    switch_href: String,
    page: &'static Page,
    body: String,
}

impl Localized for KbPage {
    fn lang(&self) -> Lang {
        self.lang
    }
}

impl BookReference for KbPage {}

pub async fn index(lang: Lang, uri: Uri) -> Response {
    let page = Index {
        lang,
        switch_href: switch_href(lang, &uri),
    };
    render(StatusCode::OK, &uri, &page)
}

pub async fn page(lang: Lang, uri: Uri, Path(slug): Path<String>) -> Response {
    let switch_href = switch_href(lang, &uri);
    let Some(page) = PAGES.iter().find(|page| page.slug == slug) else {
        let page = Message {
            lang,
            switch_href,
            heading: "kb-not-found-heading",
            text: "kb-not-found-text",
        };
        return render(StatusCode::NOT_FOUND, &uri, &page);
    };
    let body = match to_html(page.source(lang), lang) {
        Ok(body) => body,
        // The tests render every page, so this is a bad deploy.
        Err(e) => {
            tracing::error!("rendering {uri}: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let page = KbPage {
        lang,
        switch_href,
        page,
        body,
    };
    render(StatusCode::OK, &uri, &page)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    const LANGS: [Lang; 2] = [Lang::En, Lang::Hu];

    #[test]
    fn every_page_is_in_both_languages_and_on_the_learning_path() {
        let slugs = |lang: &str| -> BTreeSet<String> {
            let dir = format!("{}/kb/{lang}", env!("CARGO_MANIFEST_DIR"));
            std::fs::read_dir(&dir)
                .expect(&dir)
                .map(|entry| {
                    let name = entry.unwrap().file_name().into_string().unwrap();
                    name.strip_suffix(".md").expect(&name).to_owned()
                })
                .collect()
        };
        let path: BTreeSet<_> = PAGES.iter().map(|page| page.slug.to_owned()).collect();
        assert_eq!(path.len(), PAGES.len(), "a page listed twice");
        assert_eq!(slugs("en"), path, "kb/en");
        assert_eq!(slugs("hu"), path, "kb/hu");
    }

    #[test]
    fn every_page_renders_with_one_leading_h1() {
        for page in PAGES {
            for lang in LANGS {
                let html = to_html(page.source(lang), lang)
                    .unwrap_or_else(|e| panic!("{}: {e}", page.slug));
                assert!(html.starts_with("<h1>"), "{}: {html}", page.slug);
                assert_eq!(html.matches("<h1>").count(), 1, "{}", page.slug);
                assert!(!page.title(lang).is_empty() && !page.title(lang).starts_with('#'));
            }
        }
    }

    #[test]
    fn kb_links_stay_in_the_pages_language_and_reach_a_listed_page() {
        for page in PAGES {
            for lang in LANGS {
                let html = to_html(page.source(lang), lang).unwrap();
                for link in html.split(r#"href="/"#).skip(1) {
                    let link = &link[..link.find('"').unwrap()];
                    let Some(slug) = link.split_once("/kb/").map(|(_, slug)| slug) else {
                        continue;
                    };
                    assert!(
                        link.starts_with(&format!("{}/", lang.code())),
                        "{}: {link}",
                        page.slug
                    );
                    assert!(
                        slug.is_empty() || PAGES.iter().any(|p| p.slug == slug),
                        "{}: {link}",
                        page.slug
                    );
                }
            }
        }
        // Without a link to check, the loop above passes vacuously.
        let graha = PAGES.iter().find(|p| p.slug == "graha").unwrap();
        assert!(
            to_html(graha.hu, Lang::Hu)
                .unwrap()
                .contains(r#"href="/hu/kb/rasi""#)
        );
    }

    #[test]
    fn an_unknown_or_unclosed_placeholder_fails() {
        for source in ["{{table:bogus}}", "{{tabel:rasis}}", "{{table:rasis"] {
            assert!(to_html(source, Lang::En).is_err(), "{source}");
        }
    }

    #[test]
    fn the_rasi_table_has_header_cells_and_the_charts_names() {
        let html = to_html("Before.\n{{table:rasis}}", Lang::Hu).unwrap();
        assert!(
            html.contains(
                "<th>Rasi</th><th>Szanszkrit név</th><th>Jel</th><th>Kezdet</th><th>Vég</th>"
            ),
            "{html}"
        );
        // ch. 1 §1.3.2, Table 1: the first spelling where the book gives two.
        assert!(
            html.contains(
                "<td>Kos</td><td>Mesha</td><td>Ar</td><td>0°00′00″</td><td>29°59′59″</td>"
            ),
            "{html}"
        );
        assert!(
            html.contains("<td>Bika</td><td>Vrishabha</td><td>Ta</td>"),
            "{html}"
        );
        assert!(
            html.contains(
                "<td>Halak</td><td>Meena</td><td>Pi</td><td>330°00′00″</td><td>359°59′59″</td>"
            ),
            "{html}"
        );
        assert_eq!(html.matches("<tr>").count(), 13, "{html}");
    }

    #[test]
    fn the_nakshatra_table_has_table_2s_rows_with_the_common_spellings() {
        let html = to_html("Before.\n{{table:nakshatras}}", Lang::Hu).unwrap();
        assert!(
            html.contains(
                "<th>Nakshatra</th><th>Szokásos írásmód</th><th>Kezdet</th><th>Vég</th><th>Vimsottari ura</th><th>Uralkodó istenség</th>"
            ),
            "{html}"
        );
        // ch. 1 §1.3.6, Table 2.
        for row in [
            "<td>Aswini</td><td>Ashvini</td><td>00 Ar 00</td><td>13 Ar 20</td><td>Ketu</td><td>Aswini Kumara</td>",
            "<td>Mrigasira</td><td>Mrigashira</td><td>23 Ta 20</td><td>06 Ge 40</td><td>Mars</td><td>Hold</td>",
            "<td>Aasresha</td><td>Ashlesha</td><td>16 Cn 40</td><td>30 Cn 00</td><td>Merkúr</td><td>Rahu</td>",
            "<td>Makha</td><td>Magha</td><td>00 Le 00</td><td>13 Le 20</td><td>Ketu</td><td>Nap</td>",
            "<td>Satabhishak</td><td>Shatabhisha</td><td>06 Aq 40</td><td>20 Aq 00</td><td>Rahu</td><td>Varuna</td>",
            "<td>Revati</td><td>Revati</td><td>16 Pi 40</td><td>30 Pi 00</td><td>Merkúr</td><td>Pooshan</td>",
        ] {
            assert!(html.contains(row), "{row}: {html}");
        }
        assert_eq!(html.matches("<tr>").count(), 28, "{html}");
    }

    #[test]
    fn the_tithi_table_has_table_3s_rows() {
        let html = to_html("Before.\n{{table:tithis}}", Lang::Hu).unwrap();
        assert!(
            html.contains("<th>Sukla</th><th>Krishna</th><th>Tithi</th><th>Ura</th>"),
            "{html}"
        );
        // ch. 1 §1.3.8.1, Table 3.
        for row in [
            "<td>1</td><td>16</td><td>Pratipat</td><td>Nap</td>",
            "<td>8</td><td>23</td><td>Ashtami</td><td>Rahu</td>",
            "<td>14</td><td>29</td><td>Chaturdasi</td><td>Vénusz</td>",
            "<td>15</td><td>—</td><td>Paurnami</td><td>Szaturnusz</td>",
            "<td>—</td><td>30</td><td>Amavasya</td><td>Rahu</td>",
        ] {
            assert!(html.contains(row), "{row}: {html}");
        }
        assert_eq!(html.matches("<tr>").count(), 17, "{html}");
    }

    #[test]
    fn the_lunar_month_table_has_table_4s_rows() {
        let html = to_html("Before.\n{{table:lunar-months}}", Lang::Hu).unwrap();
        assert!(
            html.contains(
                "<th>Az együttállás rasija</th><th>Holdhónap</th><th>A telihold valószínű nakshatrája</th><th>Nagyjából mikor</th>"
            ),
            "{html}"
        );
        // ch. 1 §1.3.8.2, Table 4, nakshatras spelled as in Table 2.
        for row in [
            "<td>Halak</td><td>Chaitra</td><td>Chitra</td><td>márc./ápr.</td>",
            "<td>Ikrek</td><td>Aashaadha</td><td>Poorvaashaadha/Uttaraashaadha</td><td>jún./júl.</td>",
            "<td>Rák</td><td>Sraavana</td><td>Sravanam</td><td>júl./aug.</td>",
            "<td>Nyilas</td><td>Pushya</td><td>Pushyami</td><td>dec./jan.</td>",
            "<td>Vízöntő</td><td>Phaalguna</td><td>Poorva Phalguni/Uttara Phalguni</td><td>febr./márc.</td>",
        ] {
            assert!(html.contains(row), "{row}: {html}");
        }
        assert_eq!(html.matches("<tr>").count(), 13, "{html}");
    }

    #[test]
    fn the_karana_table_has_both_halves_of_every_tithi() {
        let html = to_html("Before.\n{{table:karanas}}", Lang::Hu).unwrap();
        assert!(
            html.contains("<th>Tithi</th><th>Első fele</th><th>Második fele</th>"),
            "{html}"
        );
        // ch. 1 §1.3.10: Kimstughna first, then Bava to Vishti, Sakuna to Naga last.
        for row in [
            "<td>Sukla Pratipat (1)</td><td>Kimstughna</td><td>Bava</td>",
            "<td>Sukla Dwitiya (2)</td><td>Balava</td><td>Kaulava</td>",
            "<td>Krishna Ekadasi (26)</td><td>Bava</td><td>Balava</td>",
            "<td>Krishna Chaturdasi (29)</td><td>Vishti</td><td>Sakuna</td>",
            "<td>Krishna Amavasya (30)</td><td>Chatushpada</td><td>Naga</td>",
        ] {
            assert!(html.contains(row), "{row}: {html}");
        }
        assert_eq!(html.matches("<tr>").count(), 31, "{html}");
    }

    #[test]
    fn the_sun_moon_yoga_table_has_table_5s_rows() {
        let html = to_html("Before.\n{{table:sun-moon-yogas}}", Lang::Hu).unwrap();
        assert!(
            html.contains("<th>Sorszám</th><th>Nap–Hold yoga</th><th>Jelentés</th>"),
            "{html}"
        );
        // ch. 1 §1.3.9, Table 5.
        for row in [
            "<td>1</td><td>Vishkambha</td><td>Ajtóretesz, tartóoszlop</td>",
            "<td>10</td><td>Ganda</td><td>Veszély</td>",
            "<td>27</td><td>Vaidhriti</td><td>Istenek egy osztálya</td>",
        ] {
            assert!(html.contains(row), "{row}: {html}");
        }
        assert_eq!(html.matches("<tr>").count(), 28, "{html}");
    }

    #[test]
    fn the_graha_table_has_the_nine_grahas_with_the_charts_labels() {
        let html = to_html("Before.\n{{table:grahas}}", Lang::Hu).unwrap();
        assert!(html.contains("<th>Graha</th><th>Jelölés</th>"), "{html}");
        assert!(html.contains("<td>Nap</td><td>Na</td>"), "{html}");
        assert!(html.contains("<td>Szaturnusz</td><td>Sz</td>"), "{html}");
        assert!(html.contains("<td>Ketu</td><td>Ke</td>"), "{html}");
        assert_eq!(html.matches("<tr>").count(), 10, "{html}");
    }

    #[test]
    fn raw_html_is_escaped_and_unsafe_links_fail() {
        let html = to_html(
            "<script>alert(1)</script>\n\nA <b onclick=x>b</b>.",
            Lang::En,
        )
        .unwrap();
        assert!(!html.contains("<script") && !html.contains("<b"), "{html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
        for link in [
            "javascript:alert(1)",
            "//evil.example",
            "http://example.com",
        ] {
            assert!(
                to_html(&format!("[x]({link})"), Lang::En).is_err(),
                "{link}"
            );
            assert!(
                to_html(&format!("![x]({link})"), Lang::En).is_err(),
                "{link}"
            );
        }
        for link in ["/en/kb/rasi", "#top", "https://www.vedicastrologer.org/"] {
            assert!(to_html(&format!("[x]({link})"), Lang::En).is_ok(), "{link}");
        }
    }
}

//! The knowledge base (ADR 0004): one Markdown page per concept per language, compiled in, served at `/{lang}/kb/{slug}`.

use askama::Template;
use axum::extract::Path;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use fluent_templates::Loader;
use fluent_templates::fluent_bundle::FluentValue;
use pulldown_cmark::{Event, Options, Parser, Tag};

use super::{LOCALES, Lang, Localized, Message, body_id, rasi_id, render, switch_href};

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
                "sanskrit-name",
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

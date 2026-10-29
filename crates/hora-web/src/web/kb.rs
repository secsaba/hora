//! The knowledge base (ADR 0004): one Markdown page per concept per language, compiled in, served at `/{lang}/kb/{slug}`.

use std::fmt::Debug;

use askama::Template;
use axum::extract::Path;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use fluent_templates::Loader;
use fluent_templates::fluent_bundle::FluentValue;
use pulldown_cmark::{Event, Options, Parser, Tag};

use super::{
    LOCALES, Lang, Localized, Message, body_id, karana_id, lunar_month_id, nakshatra_id, paksha_id,
    rasi_id, render, sun_moon_yoga_id, switch_href, tithi_id, vaara_id,
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
    ("vaara-and-hora", "1", "1.3.11"),
    ("panchaanga", "1", "1.3.12"),
    ("ayanamsa", "1", "1.3.13"),
    ("dasa", "1", "1.4"),
    ("rasi-characteristics", "2", "2.2–2.3"),
    ("graha-characteristics", "3", "3.1–3.2"),
];

/// The learning path cut into the book's chapters.
pub fn chapters() -> impl Iterator<Item = &'static [Page]> {
    PAGES.chunk_by(|a, b| a.chapter == b.chapter)
}

/// The ch. 2 §2.2 characteristics in the book's order: each one's name (its glossary term where it has one) and a rasi's value as a message id.
const CHARACTERISTICS: [(&str, Value); 12] = [
    ("limb", |r| message_id("limb", r.limb())),
    ("vishama-rasi", |r| message_id("parity", r.parity())),
    ("vishamapada-rasi", |r| {
        message_id("footedness", r.footedness())
    }),
    ("chara-sthira-dwiswabhava", |r| {
        message_id("mobility", r.mobility())
    }),
    ("element", |r| message_id("element", r.element())),
    ("pitta-vaata-kapha", |r| message_id("nature", r.nature())),
    ("guna", |r| message_id("quality", r.quality())),
    ("direction", |r| message_id("direction", r.direction())),
    ("colour", |r| message_id("colour", r.colour())),
    ("divaa-rasi", |r| message_id("strong-by", r.strong_by())),
    ("seershodaya-rasi", |r| message_id("rising", r.rising())),
    ("varna", |r| message_id("class", r.class())),
];

/// A rasi's value of one characteristic, as a message id.
type Value = fn(hora_chart::Rasi) -> String;

/// The .ftl ids of the §2.2 values are named after hora-chart's variants: `Limb::BelowNavel` is `limb-below-navel`.
fn message_id(prefix: &str, value: impl Debug) -> String {
    let mut id = prefix.to_owned();
    for c in format!("{value:?}").chars() {
        if c.is_uppercase() {
            id.push('-');
        }
        id.push(c.to_ascii_lowercase());
    }
    id
}

/// The ch. 3 §3.2 characteristics in the book's order, by anchor; `None` where the book gives a graha no value.
const GRAHA_CHARACTERISTICS: [(&str, GrahaValue); 20] = [
    ("avatara", |g| g.avatara().map(|v| message_id("avatara", v))),
    ("saumya-graha", |g| {
        g.disposition().map(|v| message_id("disposition", v))
    }),
    ("governance", |g| {
        g.governance().map(|v| message_id("governance", v))
    }),
    ("colour", |g| g.colour().map(|v| message_id("colour", v))),
    ("cabinet", |g| g.cabinet().map(|v| message_id("cabinet", v))),
    ("deity", |g| g.deity().map(|v| message_id("deity", v))),
    ("sex", |g| g.sex().map(|v| message_id("sex", v))),
    ("element", |g| g.element().map(|v| message_id("element", v))),
    ("varna", |g| g.class().map(|v| message_id("class", v))),
    ("guna", |g| g.quality().map(|v| message_id("quality", v))),
    ("abode", |g| g.abode().map(|v| message_id("abode", v))),
    ("sapta-dhaatu", |g| {
        g.sapta_dhaatu().map(|v| message_id("sapta-dhaatu", v))
    }),
    ("time-period", |g| {
        g.period().map(|v| message_id("period", v))
    }),
    ("taste", |g| g.taste().map(|v| message_id("taste", v))),
    ("digbala", |g| {
        g.digbala().map(|v| message_id("direction", v))
    }),
    ("day-and-night", |g| {
        g.strong_by().map(|v| message_id("strong-by", v))
    }),
    ("paksha", |g| {
        g.strong_paksha().map(|v| message_id("paksha", v))
    }),
    ("ayana", |g| {
        g.strong_ayana().map(|v| message_id("ayana", v))
    }),
    ("ritu", |g| g.ritu().map(|v| message_id("ritu", v))),
    ("dhatu-moola-jeeva", |g| {
        g.realm().map(|v| message_id("realm", v))
    }),
];

/// A graha's value of one characteristic, as a message id.
type GrahaValue = fn(hora_chart::Body) -> Option<String>;

/// A rasi's section on the rasi-characteristics page: `aries` to `pisces`.
fn rasi_anchor(rasi: hora_chart::Rasi) -> &'static str {
    &rasi_id(rasi)["rasi-".len()..]
}

/// A graha's section on the graha-characteristics page: `sun` to `ketu`.
fn graha_anchor(graha: hora_chart::Body) -> &'static str {
    body_id(graha).trim_start_matches("graha-")
}

/// A characteristic's table as the book lists most of them: each value with the rasis or grahas that have it, "—" (none given) last.
fn grouped(heading: &str, column: &str, values: impl Iterator<Item = (String, String)>) -> String {
    let mut rows: Vec<(String, Vec<String>)> = Vec::new();
    for (link, value) in values {
        match rows.iter_mut().find(|(row, _)| *row == value) {
            Some((_, links)) => links.push(link),
            None => rows.push((value, vec![link])),
        }
    }
    rows.sort_by_key(|(value, _)| value == "—");
    let mut table = format!("\n\n| {heading} | {column} |\n|---|---|\n");
    for (value, links) in rows {
        table += &format!("| {value} | {} |\n", links.join(", "));
    }
    table
}

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
    let events = Parser::new_ext(
        &markdown,
        Options::ENABLE_TABLES | Options::ENABLE_HEADING_ATTRIBUTES,
    )
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

/// A placeholder's Markdown, a table or a rasi's characteristics line, names translated as on the chart page; `None` for an unknown one.
fn table(name: &str, lang: Lang) -> Option<String> {
    let t = |id: &str| LOCALES.lookup(&lang.id(), id);
    let heading = |characteristic| t(&format!("rasi-characteristic-{characteristic}"));
    let graha_heading = |characteristic| t(&format!("graha-characteristic-{characteristic}"));
    let graha_value = |value: GrahaValue, graha| value(graha).map_or("—".into(), |id| t(&id));
    if let Some(anchor) = name.strip_prefix("characteristics:") {
        if let Some(rasi) = hora_chart::RASIS
            .into_iter()
            .find(|&rasi| rasi_anchor(rasi) == anchor)
        {
            let values: Vec<_> = CHARACTERISTICS
                .iter()
                .map(|(characteristic, value)| {
                    format!("*{}:* {}", heading(characteristic), t(&value(rasi)))
                })
                .collect();
            return Some(values.join("; "));
        }
        let graha = hora_chart::GRAHAS
            .into_iter()
            .find(|&graha| graha_anchor(graha) == anchor)?;
        let values: Vec<_> = GRAHA_CHARACTERISTICS
            .iter()
            .map(|&(characteristic, value)| {
                format!(
                    "*{}:* {}",
                    graha_heading(characteristic),
                    graha_value(value, graha)
                )
            })
            .collect();
        return Some(values.join("; "));
    }
    if let Some(&(characteristic, value)) = name
        .strip_prefix("table:graha-")
        .and_then(|name| GRAHA_CHARACTERISTICS.iter().find(|(c, _)| *c == name))
    {
        let values = hora_chart::GRAHAS.into_iter().map(|graha| {
            let link = format!("[{}](#{})", t(body_id(graha)), graha_anchor(graha));
            (link, graha_value(value, graha))
        });
        return Some(grouped(
            &graha_heading(characteristic),
            &t("graha-characteristic-grahas"),
            values,
        ));
    }
    if let Some((characteristic, value)) = name
        .strip_prefix("table:")
        .and_then(|name| CHARACTERISTICS.iter().find(|(c, _)| *c == name))
    {
        let values = hora_chart::RASIS.into_iter().map(|rasi| {
            let link = format!("[{}](#{})", t(rasi_id(rasi)), rasi_anchor(rasi));
            (link, t(&value(rasi)))
        });
        return Some(grouped(
            &heading(characteristic),
            &t("rasi-characteristic-rasis"),
            values,
        ));
    }
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
        "table:hora-lords" => {
            // A Vedic day of each weekday, Sunday first: Julian day 2451546 is a Sunday.
            let days: Vec<_> = (0..7)
                .map(|weekday| hora_swe::VedicDay {
                    sunrise: 0.0,
                    sunset: 0.5,
                    next_sunrise: 1.0,
                    clock_sunrise: 2451546.0 + f64::from(weekday),
                    clock_sunset: 0.0,
                    clock_next_sunrise: 0.0,
                })
                .collect();
            let vaaras: Vec<_> = days
                .iter()
                .map(|day| t(vaara_id(hora_chart::vaara(day))))
                .collect();
            let mut table = format!(
                "\n\n| {} | {} |\n|---|---|---|---|---|---|---|---|\n",
                t("hora"),
                vaaras.join(" | ")
            );
            // The lords repeat every seven horas, so one row holds the 1st, 8th, 15th and 22nd.
            for first in 1..=7u8 {
                let horas: Vec<_> = (first..=24).step_by(7).map(|h| h.to_string()).collect();
                let lords: Vec<_> = days
                    .iter()
                    .map(|day| {
                        t(body_id(
                            hora_chart::hora(day, (f64::from(first) - 0.5) / 24.0).1,
                        ))
                    })
                    .collect();
                table += &format!("| {} | {} |\n", horas.join(", "), lords.join(" | "));
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

    /// "Chapter 1: Basic concepts", the book's chapter title in the page's language.
    fn chapter_title(&self, page: &Page) -> String {
        LOCALES.lookup(&self.lang().id(), &format!("kb-chapter-{}", page.chapter))
    }

    /// "§ 1.3.2", for a page listed under its chapter.
    fn section(&self, page: &Page) -> String {
        let args = [("section".into(), FluentValue::from(page.section))].into();
        LOCALES.lookup_with_args(&self.lang().id(), "kb-section", &args)
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
    previous: Option<&'static Page>,
    next: Option<&'static Page>,
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
    let Some(i) = PAGES.iter().position(|page| page.slug == slug) else {
        let page = Message {
            lang,
            switch_href,
            heading: "kb-not-found-heading",
            text: "kb-not-found-text",
        };
        return render(StatusCode::NOT_FOUND, &uri, &page);
    };
    let page = &PAGES[i];
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
        previous: i.checked_sub(1).map(|i| &PAGES[i]),
        next: PAGES.get(i + 1),
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
    fn every_chapter_has_a_title_in_both_languages() {
        for pages in chapters() {
            let id = format!("kb-chapter-{}", pages[0].chapter);
            for lang in LANGS {
                assert!(LOCALES.try_lookup(&lang.id(), &id).is_some(), "{id}");
            }
        }
        // chunk_by would list a chapter twice if its pages were not together on the path.
        let listed: Vec<_> = chapters().map(|pages| pages[0].chapter).collect();
        assert_eq!(
            listed.len(),
            listed.iter().collect::<BTreeSet<_>>().len(),
            "{listed:?}"
        );
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
                    let Some(slug) = link
                        .split_once("/kb/")
                        .map(|(_, slug)| slug.split('#').next().unwrap())
                    else {
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
        for (from, to) in [
            ("rasi", "rasi-characteristics"),
            ("graha", "graha-characteristics"),
        ] {
            let page = PAGES.iter().find(|p| p.slug == from).unwrap();
            for lang in LANGS {
                let html = to_html(page.source(lang), lang).unwrap();
                let href = format!(r#"href="/{}/kb/{to}""#, lang.code());
                assert!(html.contains(&href), "{from}: {html}");
            }
        }
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
    fn the_hora_lord_table_has_every_vaaras_horas() {
        let html = to_html("Before.\n{{table:hora-lords}}", Lang::En).unwrap();
        assert!(
            html.contains("<th>Hora</th><th>Sunday</th><th>Monday</th>"),
            "{html}"
        );
        // ch. 1 §1.3.11: the vaara's lord first, then on in the order Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon.
        for row in [
            "<td>1, 8, 15, 22</td><td>Sun</td><td>Moon</td><td>Mars</td><td>Mercury</td><td>Jupiter</td><td>Venus</td><td>Saturn</td>",
            "<td>2, 9, 16, 23</td><td>Venus</td><td>Saturn</td><td>Sun</td><td>Moon</td><td>Mars</td><td>Mercury</td><td>Jupiter</td>",
            "<td>4, 11, 18</td><td>Moon</td><td>Mars</td><td>Mercury</td><td>Jupiter</td><td>Venus</td><td>Saturn</td><td>Sun</td>",
            "<td>7, 14, 21</td><td>Mars</td><td>Mercury</td><td>Jupiter</td><td>Venus</td><td>Saturn</td><td>Sun</td><td>Moon</td>",
        ] {
            assert!(html.contains(row), "{row}: {html}");
        }
        assert_eq!(html.matches("<tr>").count(), 8, "{html}");
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
    fn a_characteristic_table_groups_the_rasis_by_value_and_links_their_sections() {
        let html = to_html("Before.\n{{table:element}}", Lang::Hu).unwrap();
        assert!(html.contains("<th>Elem</th><th>Rasik</th>"), "{html}");
        // ch. 2 §2.2.5.
        assert!(
            html.contains(
                r##"<td>agni (tűz)</td><td><a href="#aries">Kos</a>, <a href="#leo">Oroszlán</a>, <a href="#sagittarius">Nyilas</a></td>"##
            ),
            "{html}"
        );
        assert_eq!(html.matches("<tr>").count(), 5, "{html}");
        // ch. 2 §2.2.9: Taurus and Leo share white.
        let html = to_html("Before.\n{{table:colour}}", Lang::En).unwrap();
        assert!(
            html.contains(
                r##"<td>white</td><td><a href="#taurus">Taurus</a>, <a href="#leo">Leo</a></td>"##
            ),
            "{html}"
        );
        assert_eq!(html.matches("<tr>").count(), 11, "{html}");
    }

    #[test]
    fn a_rasis_characteristics_line_has_all_twelve() {
        let html = to_html("{{characteristics:pisces}}", Lang::En).unwrap();
        // ch. 2 §2.2.1–2.2.12.
        for value in [
            "feet",
            "sama (even)",
            "samapada (even-footed)",
            "dwiswabhava (dual)",
            "jala (water)",
            "kapha (phlegmatic)",
            "sattwa (purity)",
            "north",
            "cream, or the colour of fish",
            "divaa (day)",
            "both head and feet",
            "Brahmana (scholar)",
        ] {
            assert!(html.contains(value), "{value}: {html}");
        }
        assert!(to_html("{{characteristics:pluto}}", Lang::En).is_err());
    }

    #[test]
    fn the_rasi_characteristics_page_has_every_rasis_section_and_every_name() {
        let page = PAGES
            .iter()
            .find(|p| p.slug == "rasi-characteristics")
            .unwrap();
        for lang in LANGS {
            let html = to_html(page.source(lang), lang).unwrap();
            for rasi in hora_chart::RASIS {
                let anchor = rasi_anchor(rasi);
                assert!(html.contains(&format!(r#" id="{anchor}""#)), "{anchor}");
            }
            // fluent-templates renders an id missing from every .ftl file as this text.
            assert!(!html.contains("Unknown localization key"), "{html}");
        }
    }

    #[test]
    fn a_graha_characteristic_table_groups_the_grahas_and_puts_none_given_last() {
        let html = to_html("Before.\n{{table:graha-abode}}", Lang::En).unwrap();
        assert!(html.contains("<th>Abode</th><th>Grahas</th>"), "{html}");
        // ch. 3 §3.2.11 gives Mars and the nodes no abode.
        assert!(
            html.contains(r##"<td>temple</td><td><a href="#sun">Sun</a></td>"##),
            "{html}"
        );
        assert!(
            html.ends_with(
                r##"<td>—</td><td><a href="#mars">Mars</a>, <a href="#rahu">Rahu</a>, <a href="#ketu">Ketu</a></td></tr>
</tbody></table>
"##
            ),
            "{html}"
        );
        assert_eq!(html.matches("<tr>").count(), 8, "{html}");
        // ch. 3 §3.2.4: the Moon and Jupiter share tawny.
        let html = to_html("Before.\n{{table:graha-colour}}", Lang::Hu).unwrap();
        assert!(
            html.contains(
                r##"<td>sárgásbarna</td><td><a href="#moon">Hold</a>, <a href="#jupiter">Jupiter</a></td>"##
            ),
            "{html}"
        );
    }

    #[test]
    fn a_grahas_characteristics_line_has_all_twenty() {
        let html = to_html("{{characteristics:mercury}}", Lang::En).unwrap();
        // ch. 3 §3.2.1–3.2.17.
        for value in [
            "Buddha",
            "saumya alone or with more natural benefics",
            "speech",
            "grass green",
            "prince",
            "Maha Vishnu",
            "female",
            "bhoo (earth)",
            "Vaisya (trader)",
            "rajas (energy)",
            "sports ground",
            "skin",
            "ritu (season of two months)",
            "mixed",
            "east",
            "always",
            "hemanta (season of dew)",
            "jeeva (living beings)",
        ] {
            assert!(html.contains(value), "{value}: {html}");
        }
        // Mercury's paksha and ayana strength turn with his nature.
        assert_eq!(html.matches("—").count(), 2, "{html}");
        assert_eq!(html.matches("<em>").count(), 20, "{html}");
        assert!(to_html("{{characteristics:lagna}}", Lang::En).is_err());
    }

    #[test]
    fn the_graha_characteristics_page_has_every_grahas_section_and_every_name() {
        let page = PAGES
            .iter()
            .find(|p| p.slug == "graha-characteristics")
            .unwrap();
        for lang in LANGS {
            let html = to_html(page.source(lang), lang).unwrap();
            for anchor in hora_chart::GRAHAS
                .map(graha_anchor)
                .iter()
                .chain(GRAHA_CHARACTERISTICS.map(|(c, _)| c).iter())
            {
                assert!(html.contains(&format!(r#" id="{anchor}""#)), "{anchor}");
            }
            assert!(!html.contains("Unknown localization key"), "{html}");
        }
    }

    #[test]
    fn every_page_has_the_same_anchors_in_both_languages() {
        let anchors = |html: String| -> BTreeSet<String> {
            html.split(r#" id=""#)
                .skip(1)
                .map(|id| id[..id.find('"').unwrap()].to_owned())
                .collect()
        };
        for page in PAGES {
            let [en, hu] = LANGS.map(|lang| anchors(to_html(page.source(lang), lang).unwrap()));
            assert_eq!(en, hu, "{}", page.slug);
        }
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

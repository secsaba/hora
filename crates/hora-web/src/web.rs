//! Routes and pages; every page is served in English under `/en/` and in Hungarian under `/hu/`.

use std::collections::{BTreeMap, HashMap};

use askama::Template;
use axum::Router;
use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use fluent_templates::{LanguageIdentifier, Loader, langid, static_loader};
use hora_swe::{Ayanamsa, Calendar, NodeType, Request, Rule, Sky, UtcOffset};

use crate::chart::{self, Body, Rasi};

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        // No Unicode isolation marks around placeables, so rendered text is plain.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/en/", get(|uri| home(Lang::En, uri)))
        .route("/hu/", get(|uri| home(Lang::Hu, uri)))
        .route(
            "/en/chart",
            get(|uri, query| chart_page(Lang::En, uri, query)),
        )
        .route(
            "/hu/chart",
            get(|uri, query| chart_page(Lang::Hu, uri, query)),
        )
        .route("/en", get(|| async { Redirect::permanent("/en/") }))
        .route("/hu", get(|| async { Redirect::permanent("/hu/") }))
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Lang {
    En,
    Hu,
}

impl Lang {
    fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Hu => "hu",
        }
    }

    fn other(self) -> Lang {
        match self {
            Lang::En => Lang::Hu,
            Lang::Hu => Lang::En,
        }
    }

    fn id(self) -> LanguageIdentifier {
        match self {
            Lang::En => langid!("en"),
            Lang::Hu => langid!("hu"),
        }
    }

    /// The `Accept-Language` choice with the highest q (the first on a tie); English when neither is acceptable.
    fn from_accept_language(value: &str) -> Lang {
        let mut best = (Lang::En, 0.0);
        for item in value.split(',') {
            let mut parts = item.split(';').map(str::trim);
            let tag = parts.next().unwrap_or_default();
            let lang = match tag.split('-').next().unwrap_or_default() {
                t if t.eq_ignore_ascii_case("en") => Lang::En,
                t if t.eq_ignore_ascii_case("hu") => Lang::Hu,
                _ => continue,
            };
            // Parameter names are case-insensitive (RFC 9110 §5.6.6).
            let q = match parts.find_map(|p| {
                p.get(..2)
                    .filter(|k| k.eq_ignore_ascii_case("q="))
                    .map(|_| &p[2..])
            }) {
                None => 1.0,
                Some(q) => match q.parse::<f32>() {
                    Ok(q) if (0.0..=1.0).contains(&q) => q,
                    _ => continue,
                },
            };
            if q > best.1 {
                best = (lang, q);
            }
        }
        best.0
    }
}

async fn root(headers: HeaderMap) -> impl IntoResponse {
    let accept = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let lang = Lang::from_accept_language(accept);
    (
        [(header::VARY, "accept-language")],
        Redirect::to(&format!("/{}/", lang.code())),
    )
}

/// What `base.html` asks of every page.
trait Localized {
    fn lang(&self) -> Lang;

    fn t(&self, id: &str) -> String {
        LOCALES.lookup(&self.lang().id(), id)
    }

    /// AGPL §13: every page links to the source.
    fn repository(&self) -> &'static str {
        env!("CARGO_PKG_REPOSITORY")
    }
}

/// The form, and the chart once one is calculated.
#[derive(Template)]
#[template(path = "home.html")]
struct Home {
    lang: Lang,
    switch_href: String,
    /// The submitted fields, echoed back so a bad one can be corrected.
    form: HashMap<String, String>,
    /// Field name → message id.
    errors: BTreeMap<&'static str, &'static str>,
    chart: Vec<Row>,
}

impl Localized for Home {
    fn lang(&self) -> Lang {
        self.lang
    }
}

impl Home {
    fn value(&self, name: &str) -> &str {
        self.form.get(name).map_or("", String::as_str)
    }

    /// Whether an option is the submitted one, or the first when none was.
    fn selected(&self, name: &str, option: &str, first: bool) -> bool {
        match field(&self.form, name) {
            "" => first,
            value => value == option,
        }
    }

    fn error(&self, name: &str) -> Option<String> {
        self.errors.get(name).map(|id| self.t(id))
    }
}

/// A page with one message: busy, or a failure.
#[derive(Template)]
#[template(path = "message.html")]
struct Message {
    lang: Lang,
    switch_href: String,
    heading: &'static str,
    text: &'static str,
}

impl Localized for Message {
    fn lang(&self) -> Lang {
        self.lang
    }
}

async fn home(lang: Lang, uri: Uri) -> Response {
    let page = Home {
        lang,
        switch_href: switch_href(lang, &uri),
        form: HashMap::new(),
        errors: BTreeMap::new(),
        chart: Vec::new(),
    };
    render(StatusCode::OK, &uri, &page)
}

async fn chart_page(lang: Lang, uri: Uri, Query(form): Query<HashMap<String, String>>) -> Response {
    let (status, errors, chart) = match parse(&form) {
        Err(errors) => (StatusCode::BAD_REQUEST, errors, Vec::new()),
        Ok(request) => match hora_swe::calculate(request).await {
            Ok(sky) => (StatusCode::OK, BTreeMap::new(), rows(&sky)),
            Err(hora_swe::Error::InvalidInput(rules)) => (
                StatusCode::BAD_REQUEST,
                rules.into_iter().map(field_error).collect(),
                Vec::new(),
            ),
            Err(error) => return failure(lang, &uri, error),
        },
    };
    let page = Home {
        lang,
        switch_href: switch_href(lang, &uri),
        form,
        errors,
        chart,
    };
    render(status, &uri, &page)
}

/// The busy page for `Overloaded`, else a generic one; the detail goes only to the log.
fn failure(lang: Lang, uri: &Uri, error: hora_swe::Error) -> Response {
    let (status, heading, text) = match error {
        hora_swe::Error::Overloaded => {
            (StatusCode::SERVICE_UNAVAILABLE, "busy-heading", "busy-text")
        }
        error => {
            tracing::error!("calculating {uri}: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failure-heading",
                "failure-text",
            )
        }
    };
    let page = Message {
        lang,
        switch_href: switch_href(lang, uri),
        heading,
        text,
    };
    let mut response = render(status, uri, &page);
    if status == StatusCode::SERVICE_UNAVAILABLE {
        // A queued job waits at most 2 s (ADR 0001), so a few seconds is enough for the queue to drain.
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, header::HeaderValue::from_static("5"));
    }
    response
}

fn render(status: StatusCode, uri: &Uri, page: &impl Template) -> Response {
    match page.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(e) => {
            tracing::error!("rendering {uri}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// A plain choice field's options in form order: the value sent, the choice, the label's message id; the first is the default.
type Choices<T> = [(&'static str, T, &'static str)];

const CALENDARS: &Choices<Calendar> = &[
    ("gregorian", Calendar::Gregorian, "calendar-gregorian"),
    ("julian", Calendar::Julian, "calendar-julian"),
];

const AYANAMSAS: &Choices<Ayanamsa> = &[
    ("true-chitra", Ayanamsa::TrueChitra, "ayanamsa-true-chitra"),
    ("raman", Ayanamsa::Raman, "ayanamsa-raman"),
    ("true-pushya", Ayanamsa::TruePushya, "ayanamsa-true-pushya"),
    ("true-mula", Ayanamsa::TrueMula, "ayanamsa-true-mula"),
];

const NODES: &Choices<NodeType> = &[
    ("mean", NodeType::Mean, "node-mean"),
    ("true", NodeType::True, "node-true"),
];

/// The choice a submitted value stands for; empty means the first.
fn choice<T: Copy>(choices: &Choices<T>, value: &str) -> Option<T> {
    match value {
        "" => choices.first(),
        value => choices.iter().find(|(v, ..)| *v == value),
    }
    .map(|&(_, choice, _)| choice)
}

fn field<'a>(form: &'a HashMap<String, String>, name: &str) -> &'a str {
    form.get(name).map_or("", |v| v.trim())
}

/// The field and message id of a rule `hora_swe` reports broken.
fn field_error(rule: Rule) -> (&'static str, &'static str) {
    match rule {
        Rule::Latitude => ("latitude", "error-latitude"),
        Rule::Longitude => ("longitude", "error-longitude"),
        Rule::Altitude => ("altitude", "error-altitude"),
        Rule::BirthTime => ("time", "error-time"),
        Rule::UtcOffset => ("offset", "error-offset"),
        // `error-date` states the files' window.
        Rule::ImpossibleDate | Rule::OutsideFiles => ("date", "error-date"),
        Rule::BeforeGregorian => ("date", "error-before-gregorian"),
    }
}

/// The form as a request, or each unreadable field's message id; whether the values are in range is `hora_swe`'s to say.
fn parse(form: &HashMap<String, String>) -> Result<Request, BTreeMap<&'static str, &'static str>> {
    let value = |name: &str| field(form, name);
    let calendar = choice(CALENDARS, value("calendar"));
    let utc_offset = match value("utc") {
        "" | "stated" => offset(value("offset"))
            .map(UtcOffset::Stated)
            .ok_or(("offset", "error-offset")),
        "local-mean" => Ok(UtcOffset::LocalMean),
        "local-apparent" => Ok(UtcOffset::LocalApparent),
        _ => Err(("utc", "error-choice")),
    };
    let latitude = decimal(value("latitude"));
    let longitude = decimal(value("longitude"));
    let altitude = match value("altitude") {
        "" => Some(0.0),
        value => decimal(value),
    };
    let ayanamsa = choice(AYANAMSAS, value("ayanamsa"));
    let node = choice(NODES, value("node"));
    let time = time(value("time"));
    let date = date(value("date"));
    let errors = [
        calendar.is_none().then_some(("calendar", "error-choice")),
        utc_offset.err(),
        latitude.is_none().then_some(("latitude", "error-latitude")),
        longitude
            .is_none()
            .then_some(("longitude", "error-longitude")),
        altitude.is_none().then_some(("altitude", "error-altitude")),
        ayanamsa.is_none().then_some(("ayanamsa", "error-choice")),
        node.is_none().then_some(("node", "error-choice")),
        time.is_none().then_some(("time", "error-time")),
        date.is_none().then_some(("date", "error-date")),
    ]
    .into_iter()
    .flatten()
    .collect();
    let (
        Some([year, month, day]),
        Some([hour, minute, second]),
        Some(calendar),
        Ok(utc_offset),
        Some(latitude),
        Some(longitude),
        Some(altitude),
        Some(ayanamsa),
        Some(node),
    ) = (
        date, time, calendar, utc_offset, latitude, longitude, altitude, ayanamsa, node,
    )
    else {
        return Err(errors);
    };
    Ok(Request {
        year,
        month,
        day,
        hour,
        minute,
        second,
        calendar,
        utc_offset,
        latitude,
        longitude,
        altitude,
        ayanamsa,
        node,
        // shortcut: the sunrise definition is not asked for, until #34 shows the Vedic day.
        sunrise_definition: Default::default(),
    })
}

/// Up to four digits, nothing else (no sign).
fn whole(value: &str) -> Option<i32> {
    let digits = !value.is_empty() && value.len() <= 4 && value.bytes().all(|b| b.is_ascii_digit());
    digits.then(|| value.parse().ok()).flatten()
}

/// `YYYY-MM-DD`, read but not judged.
fn date(value: &str) -> Option<[i32; 3]> {
    let parts = value.split('-').map(whole).collect::<Option<Vec<_>>>()?;
    parts.try_into().ok()
}

/// `HH:MM` or `HH:MM:SS`, read but not judged; browsers send the first when the seconds are zero.
fn time(value: &str) -> Option<[i32; 3]> {
    let parts = value.split(':').map(whole).collect::<Option<Vec<_>>>()?;
    match parts[..] {
        [hour, minute] => Some([hour, minute, 0]),
        [hour, minute, second] => Some([hour, minute, second]),
        _ => None,
    }
}

/// `±HH:MM[:SS]` in seconds; the range is `hora_swe`'s to judge.
fn offset(value: &str) -> Option<i32> {
    // The form's own label shows U+2212, so a copied minus must work too.
    let (sign, rest) = match value.strip_prefix(['-', '−']) {
        Some(rest) => (-1, rest),
        None => (1, value.strip_prefix('+').unwrap_or(value)),
    };
    let [hour, minute, second] = time(rest)?;
    // Part of the format: +05:60 is a typo, not +06:00.
    (minute < 60 && second < 60).then_some(sign * (hour * 3600 + minute * 60 + second))
}

/// A number of degrees or metres; NaN and the infinities parse here and fail the caller's range check.
fn decimal(value: &str) -> Option<f64> {
    // Hungarian writes a decimal comma, and phones offer one.
    value.replace(',', ".").parse().ok()
}

/// One row of the chart table, as message ids and text.
struct Row {
    /// Message ids.
    name: &'static str,
    rasi: &'static str,
    /// Degree, arc-minute and arc-second within the rasi.
    position: String,
    retrograde: bool,
}

/// The table rows of the chart of `sky`, in the chart's order.
fn rows(sky: &Sky) -> Vec<Row> {
    chart::chart(sky)
        .iter()
        .map(|entry| {
            let (rasi, degrees) = chart::rasi(entry.longitude);
            Row {
                name: body_id(entry.body),
                rasi: rasi_id(rasi),
                position: dms(degrees),
                retrograde: entry.retrograde,
            }
        })
        .collect()
}

fn body_id(body: Body) -> &'static str {
    match body {
        Body::Lagna => "lagna",
        Body::Sun => "graha-sun",
        Body::Moon => "graha-moon",
        Body::Mars => "graha-mars",
        Body::Mercury => "graha-mercury",
        Body::Jupiter => "graha-jupiter",
        Body::Venus => "graha-venus",
        Body::Saturn => "graha-saturn",
        Body::Rahu => "graha-rahu",
        Body::Ketu => "graha-ketu",
    }
}

fn rasi_id(rasi: Rasi) -> &'static str {
    match rasi {
        Rasi::Aries => "rasi-aries",
        Rasi::Taurus => "rasi-taurus",
        Rasi::Gemini => "rasi-gemini",
        Rasi::Cancer => "rasi-cancer",
        Rasi::Leo => "rasi-leo",
        Rasi::Virgo => "rasi-virgo",
        Rasi::Libra => "rasi-libra",
        Rasi::Scorpio => "rasi-scorpio",
        Rasi::Sagittarius => "rasi-sagittarius",
        Rasi::Capricorn => "rasi-capricorn",
        Rasi::Aquarius => "rasi-aquarius",
        Rasi::Pisces => "rasi-pisces",
    }
}

/// Degrees in [0, 30) as degree, arc-minute and arc-second, truncated: 29°59′59.9″ stays 29°59′59″.
fn dms(degrees: f64) -> String {
    // Just under 30, the product can round up to 30 × 3600.
    let seconds = ((degrees * 3600.0) as u32).min(30 * 3600 - 1);
    format!(
        "{}°{:02}′{:02}″",
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60
    )
}

/// The same path and query string in the other language; `uri` is under `/{lang}/`, as routed.
fn switch_href(lang: Lang, uri: &Uri) -> String {
    let rest = &uri.path()[1 + lang.code().len()..];
    match uri.query() {
        Some(query) => format!("/{}{rest}?{query}", lang.other().code()),
        None => format!("/{}{rest}", lang.other().code()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;

    use super::*;

    async fn get(uri: &str, accept_language: Option<&str>) -> axum::response::Response {
        let mut request = Request::get(uri);
        if let Some(value) = accept_language {
            request = request.header(header::ACCEPT_LANGUAGE, value);
        }
        app()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn root_redirect(accept_language: Option<&str>) -> String {
        let response = get("/", accept_language).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::VARY], "accept-language");
        response.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[tokio::test]
    async fn root_redirects_by_accept_language() {
        assert_eq!(root_redirect(Some("hu")).await, "/hu/");
        assert_eq!(root_redirect(None).await, "/en/");
        assert_eq!(root_redirect(Some("hu-HU,hu;q=0.9,en;q=0.8")).await, "/hu/");
        assert_eq!(root_redirect(Some("en-GB,hu;q=0.5")).await, "/en/");
        assert_eq!(root_redirect(Some("de, HU;q=0.3")).await, "/hu/");
        assert_eq!(root_redirect(Some("hu;q=0, en;q=0.1")).await, "/en/");
        assert_eq!(root_redirect(Some("en;q=0.2, hu;q=0.8")).await, "/hu/");
        assert_eq!(root_redirect(Some("hungarian, ;;q=x")).await, "/en/");
        assert_eq!(root_redirect(Some("hu;Q=0.1, en")).await, "/en/");
    }

    async fn page(response: axum::response::Response) -> String {
        let page = String::from_utf8(
            to_bytes(response.into_body(), 1 << 20)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        // fluent-templates renders an id missing from every .ftl file as this text.
        assert!(!page.contains("Unknown localization key"), "{page}");
        page
    }

    async fn body(uri: &str) -> String {
        let response = get(uri, None).await;
        assert_eq!(response.status(), StatusCode::OK);
        page(response).await
    }

    #[tokio::test]
    async fn each_language_serves_the_home_page_in_it() {
        let en = body("/en/").await;
        assert!(en.contains(r#"<html lang="en">"#), "{en}");
        assert!(en.contains("Source code"), "{en}");
        let hu = body("/hu/").await;
        assert!(hu.contains(r#"<html lang="hu">"#), "{hu}");
        assert!(hu.contains("Forráskód"), "{hu}");
        assert!(
            hu.contains(r#"href="https://github.com/secsaba/hora""#),
            "{hu}"
        );
    }

    #[tokio::test]
    async fn the_language_link_keeps_the_path_and_query_string() {
        assert!(
            body("/en/?a=1&b=%C3%A9")
                .await
                .contains(r#"href="/hu/?a=1&#38;b=%C3%A9""#)
        );
        assert!(body("/hu/").await.contains(r#"href="/en/""#));
    }

    #[tokio::test]
    async fn a_language_without_its_slash_redirects_to_it() {
        for lang in ["en", "hu"] {
            let response = get(&format!("/{lang}"), None).await;
            assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
            assert_eq!(response.headers()[header::LOCATION], format!("/{lang}/"));
        }
    }

    #[tokio::test]
    async fn other_languages_are_not_found() {
        assert_eq!(get("/de/", None).await.status(), StatusCode::NOT_FOUND);
    }

    /// Golden chart 1 (ch. 9 *Arudha Padas*, p. 87) as the book prints it: 2000-04-09 17:55 at UTC−4:00, 71°W12′, 42°N30′.
    const CHART_1: &str = "date=2000-04-09&time=17%3A55&calendar=gregorian&utc=stated&offset=-04%3A00\
        &latitude=42.5&longitude=-71.2&altitude=&ayanamsa=true-chitra&node=mean";

    /// The chart table, without the form that echoes the request.
    fn table(page: &str) -> &str {
        let start = page.find("<table").expect("a table");
        &page[start..page.find("</table>").expect("a table end")]
    }

    /// One `<tr>` holding all of `cells` in order, or `None`.
    fn row<'a>(table: &'a str, cells: &[&str]) -> Option<&'a str> {
        table.split("<tr>").find(|row| {
            let mut rest = *row;
            cells.iter().all(|cell| match rest.find(cell) {
                Some(i) => {
                    rest = &rest[i + cell.len()..];
                    true
                }
                None => false,
            })
        })
    }

    #[tokio::test]
    async fn golden_chart_1_shows_the_books_rasis_and_degrees() {
        // The book's values truncated to the arc-second: lagna 10°49′44.53″ is 10°49′44″, not 10°49′45″.
        let books = [
            (("Lagna", "Virgo"), ("Lagna", "Szűz"), "10°49′44″"),
            (("Sun", "Pisces"), ("Nap", "Halak"), "26°30′31″"),
            (("Moon", "Gemini"), ("Hold", "Ikrek"), "4°45′58″"),
            (("Mars", "Aries"), ("Mars", "Kos"), "19°10′12″"),
            (("Mercury", "Pisces"), ("Merkúr", "Halak"), "1°37′02″"),
            (("Jupiter", "Aries"), ("Jupiter", "Kos"), "17°22′39″"),
            (("Venus", "Pisces"), ("Vénusz", "Halak"), "10°02′30″"),
            (("Saturn", "Aries"), ("Szaturnusz", "Kos"), "22°42′46″"),
            (("Rahu", "Cancer"), ("Rahu", "Rák"), "5°56′06″"),
            (("Ketu", "Capricorn"), ("Ketu", "Bak"), "5°56′06″"),
        ];
        let en = body(&format!("/en/chart?{CHART_1}")).await;
        let hu = body(&format!("/hu/chart?{CHART_1}")).await;
        for (page, headings) in [(&en, ["Graha", "Rasi"]), (&hu, ["Graha", "Rasi"])] {
            let table = table(page);
            assert!(row(table, &headings).is_some(), "{table}");
            // No graha of chart 1 is retrograde, and Rahu's backward motion is never marked.
            assert!(!table.contains(">R<"), "{table}");
        }
        for ((en_graha, en_rasi), (hu_graha, hu_rasi), position) in books {
            for (page, graha, rasi) in [(&en, en_graha, en_rasi), (&hu, hu_graha, hu_rasi)] {
                let cells = [
                    format!(">{graha}<"),
                    format!(">{rasi}<"),
                    format!(">{position}"),
                ];
                let cells = cells.each_ref().map(String::as_str);
                assert!(
                    row(table(page), &cells).is_some(),
                    "{graha}: {}",
                    table(page)
                );
            }
        }
    }

    /// Chart 1's query with each `(field, value)` set (values URL-encoded).
    fn with(fields: &[(&str, &str)]) -> String {
        for (field, _) in fields {
            // A field chart 1 lacks would leave the test on chart 1 itself.
            assert!(
                CHART_1
                    .split('&')
                    .any(|p| p.starts_with(&format!("{field}="))),
                "{field}"
            );
        }
        CHART_1
            .split('&')
            .map(|pair| {
                let name = pair.split_once('=').map_or(pair, |(name, _)| name);
                match fields.iter().find(|(field, _)| *field == name) {
                    Some((field, value)) => format!("{field}={value}"),
                    None => pair.to_owned(),
                }
            })
            .collect::<Vec<_>>()
            .join("&")
    }

    #[tokio::test]
    async fn golden_chart_4_marks_jupiter_retrograde() {
        // Chart 4 (ch. 9 *Arudha Padas*, p. 98): 1968-03-19 18:30 at UTC+5:30, 82°E19′, 17°N07′.
        let chart_4 = "date=1968-03-19&time=18%3A30&utc=stated&offset=%2B05%3A30\
            &latitude=17.116666666666667&longitude=82.31666666666666";
        let page = body(&format!("/en/chart?{chart_4}")).await;
        let table = table(&page);
        // Jupiter, about 150° from the Sun, is the chart's only retrograde graha.
        assert!(row(table, &[">Jupiter<", ">R<"]).is_some(), "{table}");
        assert_eq!(table.matches(">R<").count(), 1, "{table}");
    }

    /// Chart 1 with `field` set to `value` (URL-encoded) gives 400 with `message` next to that field, keeping `value`.
    async fn assert_field_error(field: &str, value: &str, message: &str) {
        let query = with(&[(field, value)]);
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "{field}={value}"
        );
        let page = page(response).await;
        let error = format!(r#"<strong id="{field}-error" class="error">{message}</strong>"#);
        assert!(page.contains(&error), "{field}={value}: {page}");
        assert!(page.contains(r#"aria-invalid="true""#), "{page}");
        assert!(!page.contains("<table"), "{page}");
        // Every other field keeps its value.
        assert!(
            page.contains(r#"value="42.5""#) || field == "latitude",
            "{page}"
        );
    }

    #[tokio::test]
    async fn a_bad_date_is_refused_next_to_its_field() {
        for date in [
            "2000-02-30",
            "2000-4-31",
            "9-04-09",
            "2000-04",
            "2000-04-09x",
            "",
        ] {
            assert_field_error("date", date, DATE_ERROR).await;
        }
    }

    const DATE_ERROR: &str = "Enter a real date as YYYY-MM-DD, from 1200-01-04 to 2399-12-28 (in the Julian calendar, to 2399-12-12).";

    #[tokio::test]
    async fn a_julian_date_past_the_window_is_refused() {
        // Julian 2399-12-20 is Gregorian 2400-01-05, past the files' end.
        let query = with(&[("calendar", "julian"), ("date", "2399-12-20")]);
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let page = page(response).await;
        let error = format!(r#"<strong id="date-error" class="error">{DATE_ERROR}</strong>"#);
        assert!(page.contains(&error), "{page}");
    }

    #[tokio::test]
    async fn every_out_of_range_field_is_refused_at_once() {
        let query = with(&[
            ("latitude", "95"),
            ("offset", "%2B15%3A00"),
            ("time", "24%3A00"),
        ]);
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let page = page(response).await;
        for field in ["latitude", "offset", "time"] {
            assert!(
                page.contains(&format!(r#"id="{field}-error""#)),
                "{field}: {page}"
            );
        }
        assert!(!page.contains(r#"id="date-error""#), "{page}");
    }

    #[tokio::test]
    async fn an_unknown_calendar_does_not_judge_the_date() {
        let query = with(&[("calendar", "bogus"), ("date", "1500-03-01")]);
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let page = page(response).await;
        assert!(page.contains(r#"id="calendar-error""#), "{page}");
        assert!(!page.contains(r#"id="date-error""#), "{page}");
    }

    #[tokio::test]
    async fn the_dates_the_messages_name_are_charts() {
        let first_gregorian = with(&[("date", "1582-10-15")]);
        body(&format!("/en/chart?{first_gregorian}")).await;
        // The ends `error-date` names, at the clock time and offset that push them furthest out.
        for (calendar, date, time, offset) in [
            ("julian", "1200-01-04", "00%3A00", "%2B14%3A00"),
            ("gregorian", "2399-12-28", "23%3A59%3A59", "-12%3A00"),
            ("julian", "2399-12-12", "23%3A59%3A59", "-12%3A00"),
        ] {
            let query = with(&[
                ("calendar", calendar),
                ("date", date),
                ("time", time),
                ("offset", offset),
            ]);
            let response = get(&format!("/en/chart?{query}"), None).await;
            assert_eq!(response.status(), StatusCode::OK, "{query}");
        }
    }

    #[tokio::test]
    async fn every_option_in_the_form_is_a_chart() {
        let home = body("/en/").await;
        for name in ["calendar", "utc", "ayanamsa", "node"] {
            let start = home.find(&format!(r#"<select id="{name}""#)).expect(name);
            let select = &home[start..start + home[start..].find("</select>").expect(name)];
            let values: Vec<_> = select
                .split(r#"<option value=""#)
                .skip(1)
                .filter_map(|option| option.split_once('"').map(|(value, _)| value))
                .collect();
            assert!(!values.is_empty(), "{select}");
            for value in values {
                let response = get(&format!("/en/chart?{}", with(&[(name, value)])), None).await;
                assert_eq!(response.status(), StatusCode::OK, "{name}={value}");
            }
        }
    }

    #[tokio::test]
    async fn a_gregorian_date_before_1582_10_15_is_refused() {
        let message =
            "The Gregorian calendar starts on 1582-10-15; choose Julian for an earlier date.";
        assert_field_error("date", "1582-10-14", message).await;
        // The same day in the Julian calendar is a chart.
        let julian = with(&[("date", "1582-10-04"), ("calendar", "julian")]);
        body(&format!("/en/chart?{julian}")).await;
    }

    #[tokio::test]
    async fn a_latitude_out_of_range_is_refused() {
        for latitude in ["90", "-90", "91", "north", ""] {
            assert_field_error(
                "latitude",
                latitude,
                "Enter the latitude in degrees, between −90 and 90.",
            )
            .await;
        }
        assert_field_error(
            "longitude",
            "180.5",
            "Enter the longitude in degrees, from −180 to 180.",
        )
        .await;
        assert_field_error(
            "altitude",
            "9001",
            "Enter the altitude in metres, from −500 to 9000, or leave it empty.",
        )
        .await;
    }

    #[tokio::test]
    async fn nan_and_infinity_are_refused() {
        for value in ["NaN", "nan", "inf", "-infinity"] {
            assert_field_error(
                "latitude",
                value,
                "Enter the latitude in degrees, between −90 and 90.",
            )
            .await;
            assert_field_error(
                "longitude",
                value,
                "Enter the longitude in degrees, from −180 to 180.",
            )
            .await;
            assert_field_error(
                "altitude",
                value,
                "Enter the altitude in metres, from −500 to 9000, or leave it empty.",
            )
            .await;
        }
    }

    #[tokio::test]
    async fn a_bad_or_out_of_range_offset_is_refused() {
        let message = "Enter the offset as +HH:MM or −HH:MM, from −12:00 to +14:00.";
        for offset in [
            "-12%3A00%3A01",
            "%E2%88%9212%3A00%3A01",
            "%2B14%3A01",
            "%2B5",
            "%2B05%3A60",
            "05%3A30%3A00%3A00",
            "--4%3A00",
            "",
        ] {
            assert_field_error("offset", offset, message).await;
        }
        assert_field_error("time", "24%3A00", "Enter the time as HH:MM or HH:MM:SS.").await;
    }

    #[test]
    fn each_choice_starts_on_the_glossarys_default() {
        assert_eq!(CALENDARS[0].1, Calendar::default());
        assert_eq!(AYANAMSAS[0].1, Ayanamsa::default());
        assert_eq!(NODES[0].1, NodeType::default());
    }

    #[tokio::test]
    async fn an_unknown_choice_is_refused() {
        for field in ["calendar", "utc", "ayanamsa", "node"] {
            assert_field_error(field, "fagan", "Choose one of the listed options.").await;
        }
    }

    #[tokio::test]
    async fn errors_are_shown_in_hungarian_too() {
        let response = get(&format!("/hu/chart?{}", with(&[("latitude", "95")])), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(
            page(response)
                .await
                .contains("A szélességet fokban adja meg, −90 és 90 között.")
        );
    }

    #[tokio::test]
    async fn a_decimal_comma_is_read_as_a_point() {
        let comma = with(&[("latitude", "42%2C5")]);
        assert_eq!(
            table(&body(&format!("/en/chart?{comma}")).await),
            table(&body(&format!("/en/chart?{CHART_1}")).await)
        );
    }

    #[tokio::test]
    async fn local_mean_time_is_the_stated_offset_of_its_longitude() {
        // 71°12′ W is 71.2 × 4 min = 4 h 44 min 48 s behind UT.
        let stated = with(&[("offset", "-04%3A44%3A48")]);
        let mean = with(&[("utc", "local-mean"), ("offset", "")]);
        assert_eq!(
            table(&body(&format!("/en/chart?{mean}")).await),
            table(&body(&format!("/en/chart?{stated}")).await)
        );
        let minus = with(&[("offset", "%E2%88%9204%3A00")]);
        assert_eq!(
            table(&body(&format!("/en/chart?{minus}")).await),
            table(&body(&format!("/en/chart?{CHART_1}")).await)
        );
        // The comparison would pass vacuously if the offset did not move the chart.
        assert_ne!(
            table(&body(&format!("/en/chart?{mean}")).await),
            table(&body(&format!("/en/chart?{CHART_1}")).await)
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

    /// The Fluent message (and term) ids: lines that start an entry at column 0.
    fn keys(ftl: &str) -> BTreeSet<&str> {
        ftl.lines()
            .filter(|line| !line.starts_with([' ', '#', '.']))
            .filter_map(|line| line.split_once('=').map(|(id, _)| id.trim()))
            .collect()
    }

    #[test]
    fn degrees_are_truncated_and_never_shown_as_30() {
        assert_eq!(dms(29.0 + 59.0 / 60.0 + 59.9 / 3600.0), "29°59′59″");
        assert_eq!(dms(30.0 - 1e-15), "29°59′59″");
        assert_eq!(dms(0.0), "0°00′00″");
    }

    #[test]
    fn both_languages_have_the_same_messages() {
        let en = keys(include_str!("../locales/en/main.ftl"));
        let hu = keys(include_str!("../locales/hu/main.ftl"));
        assert!(!en.is_empty());
        assert_eq!(
            en.difference(&hu).collect::<Vec<_>>(),
            [&""; 0],
            "missing in hu"
        );
        assert_eq!(
            hu.difference(&en).collect::<Vec<_>>(),
            [&""; 0],
            "missing in en"
        );
    }
}

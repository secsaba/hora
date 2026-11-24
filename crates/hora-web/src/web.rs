//! Routes and pages; every page is served in English under `/en/` and in Hungarian under `/hu/`.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use askama::Template;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader, langid, static_loader};
use hora_swe::{Ayanamsa, Calendar, NodeType, Request, Rule, Sky, SunriseDefinition, UtcOffset};
use jiff::tz::{AmbiguousOffset, TimeZone};

use hora_chart::{
    Bhava, Body, CharaKaraka, Karana, LunarMonth, Nakshatra, Nth, Paksha, Rasi, ReferencePoint,
    Relative, SpecialLagna, SunMoonYoga, Tithi, TwinMonth, Vaara, Varga,
};

mod book_charts;
mod chakra;
mod kb;
mod place;

pub use place::Places;

mod chart;
mod form;
mod panchaanga;
mod tables;
use chart::*;
use form::*;
use panchaanga::*;
use tables::*;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        // No Unicode isolation marks around placeables, so rendered text is plain.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[derive(Clone)]
struct Site {
    places: Arc<Places>,
    /// The place whose chart of now an empty request shows.
    default_place: Option<u32>,
    /// The wall clock, outside tests.
    now: fn() -> jiff::Timestamp,
}

/// `id` if it is in `places`, so a wrong `HORA_DEFAULT_PLACE` stops startup.
pub fn default_place(id: Option<u32>, places: &Places) -> anyhow::Result<Option<u32>> {
    match id {
        Some(id) if places.get(id).is_none() => {
            anyhow::bail!("HORA_DEFAULT_PLACE {id} is not a place in HORA_PLACES")
        }
        id => Ok(id),
    }
}

pub fn app(
    places: Arc<Places>,
    default_place: Option<u32>,
    now: fn() -> jiff::Timestamp,
) -> Router {
    Router::new()
        .route("/", get(root))
        .route(
            "/hora.js",
            get(|headers: HeaderMap| async move {
                let script = include_bytes!("../assets/hora.js");
                asset(&headers, script, "text/javascript; charset=utf-8")
            }),
        )
        .route(
            "/favicon.svg",
            get(|headers: HeaderMap| async move {
                asset(
                    &headers,
                    include_bytes!("../assets/favicon.svg"),
                    "image/svg+xml",
                )
            }),
        )
        .route(
            "/apple-touch-icon.png",
            get(|headers: HeaderMap| async move {
                asset(
                    &headers,
                    include_bytes!("../assets/apple-touch-icon.png"),
                    "image/png",
                )
            }),
        )
        .route(
            "/robots.txt",
            get(|headers: HeaderMap| async move {
                asset(
                    &headers,
                    include_bytes!("../assets/robots.txt"),
                    "text/plain; charset=utf-8",
                )
            }),
        )
        .route("/en/", get(|site, uri| home(Lang::En, site, uri)))
        .route("/hu/", get(|site, uri| home(Lang::Hu, site, uri)))
        .route(
            "/en/chart",
            get(|site, uri, query| chart_page(Lang::En, site, uri, query)),
        )
        .route(
            "/hu/chart",
            get(|site, uri, query| chart_page(Lang::Hu, site, uri, query)),
        )
        .route("/en/privacy", get(|uri| privacy(Lang::En, uri)))
        .route("/hu/privacy", get(|uri| privacy(Lang::Hu, uri)))
        .route("/en/disclaimer", get(|uri| disclaimer(Lang::En, uri)))
        .route("/hu/disclaimer", get(|uri| disclaimer(Lang::Hu, uri)))
        .route("/en/kb/", get(|uri| kb::index(Lang::En, uri)))
        .route("/hu/kb/", get(|uri| kb::index(Lang::Hu, uri)))
        .route(
            "/en/kb/{slug}",
            get(|uri, slug| kb::page(Lang::En, uri, slug)),
        )
        .route(
            "/hu/kb/{slug}",
            get(|uri, slug| kb::page(Lang::Hu, uri, slug)),
        )
        .route("/en/kb", get(|| async { Redirect::permanent("/en/kb/") }))
        .route("/hu/kb", get(|| async { Redirect::permanent("/hu/kb/") }))
        .route("/en", get(|| async { Redirect::permanent("/en/") }))
        .route("/hu", get(|| async { Redirect::permanent("/hu/") }))
        .with_state(Site {
            places,
            default_place,
            now,
        })
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

    /// The running release; the deploy tags its source `v{version}`.
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn version_label(&self) -> String {
        message(
            self.lang(),
            "version",
            [("version", self.version().to_owned())],
        )
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
    chart: Option<ChartView>,
    /// A place search's results: geonameid and label.
    places: Vec<(u32, String)>,
    /// When the birth time fell in a clock change of the place's zone: the offsets to choose from, as the form writes them.
    offsets: Vec<String>,
    /// With a chart whose offset came from the place's zone: the sentence naming both.
    zone_used: Option<String>,
    /// With `zone_used`: the date is before 1970, where tzdb's offsets are often wrong outside the zone's main city.
    before_1970: bool,
}

impl Localized for Home {
    fn lang(&self) -> Lang {
        self.lang
    }
}

impl Home {
    /// The page without a form or a chart.
    fn empty(lang: Lang, uri: &Uri) -> Self {
        Home {
            lang,
            switch_href: switch_href(lang, uri),
            form: HashMap::new(),
            errors: BTreeMap::new(),
            chart: None,
            places: Vec::new(),
            offsets: Vec::new(),
            zone_used: None,
            before_1970: false,
        }
    }

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

    /// The reference points the shown chart has: without a Vedic day, no HL or GL (ch. 5).
    fn references(&self) -> Vec<(&'static str, ReferencePoint, &'static str)> {
        REFERENCES
            .iter()
            .filter(|(_, reference, _)| {
                self.chart
                    .as_ref()
                    .is_none_or(|view| view.vedic_day.is_some())
                    || !matches!(
                        reference,
                        ReferencePoint::Special(SpecialLagna::HoraLagna | SpecialLagna::GhatiLagna)
                    )
            })
            .copied()
            .collect()
    }

    /// A chara karaka's message id: its name, and with `-short` its abbreviation.
    fn karaka_id(&self, karaka: &CharaKaraka) -> String {
        kb::message_id("chara-karaka", karaka)
    }

    /// The rasis' message ids from Aries, the ashtakavarga tables' columns (ch. 12).
    fn rasis(&self) -> [&'static str; 12] {
        hora_chart::RASIS.map(rasi_id)
    }

    /// The Amsabala table's columns, as message ids (ch. 6 §6.6).
    fn varga_groups(&self) -> [String; 4] {
        hora_chart::VARGA_GROUPS.map(|group| kb::message_id("varga-group", group))
    }

    fn error(&self, name: &str) -> Option<String> {
        self.errors.get(name).map(|id| self.t(id))
    }

    /// A panchaanga limb's knowledge base page, if it has one: the vaara and hora share one, the others are named for the limb.
    fn kb_slug(&self, name: &'static str) -> Option<&'static str> {
        let slug = match name {
            "vaara" | "hora" => "vaara-and-hora",
            name => name,
        };
        kb::PAGES
            .iter()
            .any(|page| page.slug == slug)
            .then_some(slug)
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

/// What hora keeps and who sees a request.
#[derive(Template)]
#[template(path = "privacy.html")]
struct Privacy {
    lang: Lang,
    switch_href: String,
}

impl Localized for Privacy {
    fn lang(&self) -> Lang {
        self.lang
    }
}

async fn privacy(lang: Lang, uri: Uri) -> Response {
    let page = Privacy {
        lang,
        switch_href: switch_href(lang, &uri),
    };
    render(StatusCode::OK, &uri, &page)
}

/// What hora's chakras and readings are not.
#[derive(Template)]
#[template(path = "disclaimer.html")]
struct Disclaimer {
    lang: Lang,
    switch_href: String,
}

impl Localized for Disclaimer {
    fn lang(&self) -> Lang {
        self.lang
    }
}

async fn disclaimer(lang: Lang, uri: Uri) -> Response {
    let page = Disclaimer {
        lang,
        switch_href: switch_href(lang, &uri),
    };
    render(StatusCode::OK, &uri, &page)
}

/// A file compiled into the binary, revalidated by its ETag.
fn asset(headers: &HeaderMap, content: &'static [u8], content_type: &'static str) -> Response {
    use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
    let hash = BuildHasherDefault::<DefaultHasher>::default().hash_one(content);
    let etag = format!("\"{hash:x}\"");
    let cache = [
        (header::ETAG, etag.clone()),
        // No hash in the name, so a browser revalidates instead of keeping an old file.
        (header::CACHE_CONTROL, "no-cache".to_owned()),
    ];
    // Weak comparison (RFC 9110 §13.1.2): a proxy that compresses the file weakens its tag.
    if headers
        .get(header::IF_NONE_MATCH)
        .and_then(|tags| tags.to_str().ok())
        .is_some_and(|tags| {
            tags.trim() == "*"
                || tags
                    .split(',')
                    .any(|tag| tag.trim().trim_start_matches("W/") == etag)
        })
    {
        return (StatusCode::NOT_MODIFIED, cache).into_response();
    }
    (cache, [(header::CONTENT_TYPE, content_type)], content).into_response()
}

async fn home(lang: Lang, State(site): State<Site>, uri: Uri) -> Response {
    moment(lang, &site, &uri)
        .unwrap_or_else(|| render(StatusCode::OK, &uri, &Home::empty(lang, &uri)))
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

const fn body_id(body: Body) -> &'static str {
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

fn nakshatra_id(nakshatra: Nakshatra) -> &'static str {
    match nakshatra {
        Nakshatra::Aswini => "nakshatra-aswini",
        Nakshatra::Bharani => "nakshatra-bharani",
        Nakshatra::Krittika => "nakshatra-krittika",
        Nakshatra::Rohini => "nakshatra-rohini",
        Nakshatra::Mrigasira => "nakshatra-mrigasira",
        Nakshatra::Aardra => "nakshatra-aardra",
        Nakshatra::Punarvasu => "nakshatra-punarvasu",
        Nakshatra::Pushyami => "nakshatra-pushyami",
        Nakshatra::Aasresha => "nakshatra-aasresha",
        Nakshatra::Makha => "nakshatra-makha",
        Nakshatra::PoorvaPhalguni => "nakshatra-poorva-phalguni",
        Nakshatra::UttaraPhalguni => "nakshatra-uttara-phalguni",
        Nakshatra::Hasta => "nakshatra-hasta",
        Nakshatra::Chitra => "nakshatra-chitra",
        Nakshatra::Swaati => "nakshatra-swaati",
        Nakshatra::Visaakha => "nakshatra-visaakha",
        Nakshatra::Anooraadha => "nakshatra-anooraadha",
        Nakshatra::Jyeshtha => "nakshatra-jyeshtha",
        Nakshatra::Moola => "nakshatra-moola",
        Nakshatra::Poorvaashaadha => "nakshatra-poorvaashaadha",
        Nakshatra::Uttaraashaadha => "nakshatra-uttaraashaadha",
        Nakshatra::Sravanam => "nakshatra-sravanam",
        Nakshatra::Dhanishtha => "nakshatra-dhanishtha",
        Nakshatra::Satabhishak => "nakshatra-satabhishak",
        Nakshatra::Poorvaabhaadra => "nakshatra-poorvaabhaadra",
        Nakshatra::Uttaraabhaadra => "nakshatra-uttaraabhaadra",
        Nakshatra::Revati => "nakshatra-revati",
    }
}

fn paksha_id(paksha: Paksha) -> &'static str {
    match paksha {
        Paksha::Sukla => "paksha-sukla",
        Paksha::Krishna => "paksha-krishna",
    }
}

fn tithi_id(tithi: Tithi) -> &'static str {
    match tithi {
        Tithi::Pratipat => "tithi-pratipat",
        Tithi::Dwitiya => "tithi-dwitiya",
        Tithi::Tritiya => "tithi-tritiya",
        Tithi::Chaturthi => "tithi-chaturthi",
        Tithi::Panchami => "tithi-panchami",
        Tithi::Shashti => "tithi-shashti",
        Tithi::Saptami => "tithi-saptami",
        Tithi::Ashtami => "tithi-ashtami",
        Tithi::Navami => "tithi-navami",
        Tithi::Dasami => "tithi-dasami",
        Tithi::Ekadasi => "tithi-ekadasi",
        Tithi::Dwadasi => "tithi-dwadasi",
        Tithi::Trayodasi => "tithi-trayodasi",
        Tithi::Chaturdasi => "tithi-chaturdasi",
        Tithi::Paurnami => "tithi-paurnami",
        Tithi::Amavasya => "tithi-amavasya",
    }
}

fn sun_moon_yoga_id(yoga: SunMoonYoga) -> &'static str {
    match yoga {
        SunMoonYoga::Vishkambha => "sun-moon-yoga-vishkambha",
        SunMoonYoga::Preeti => "sun-moon-yoga-preeti",
        SunMoonYoga::Aayushmaan => "sun-moon-yoga-aayushmaan",
        SunMoonYoga::Saubhaagya => "sun-moon-yoga-saubhaagya",
        SunMoonYoga::Sobhana => "sun-moon-yoga-sobhana",
        SunMoonYoga::Atiganda => "sun-moon-yoga-atiganda",
        SunMoonYoga::Sukarman => "sun-moon-yoga-sukarman",
        SunMoonYoga::Dhriti => "sun-moon-yoga-dhriti",
        SunMoonYoga::Shoola => "sun-moon-yoga-shoola",
        SunMoonYoga::Ganda => "sun-moon-yoga-ganda",
        SunMoonYoga::Vriddhi => "sun-moon-yoga-vriddhi",
        SunMoonYoga::Dhruva => "sun-moon-yoga-dhruva",
        SunMoonYoga::Vyaaghaata => "sun-moon-yoga-vyaaghaata",
        SunMoonYoga::Harshana => "sun-moon-yoga-harshana",
        SunMoonYoga::Vajra => "sun-moon-yoga-vajra",
        SunMoonYoga::Siddhi => "sun-moon-yoga-siddhi",
        SunMoonYoga::Vyatipaata => "sun-moon-yoga-vyatipaata",
        SunMoonYoga::Variyan => "sun-moon-yoga-variyan",
        SunMoonYoga::Parigha => "sun-moon-yoga-parigha",
        SunMoonYoga::Shiva => "sun-moon-yoga-shiva",
        SunMoonYoga::Siddha => "sun-moon-yoga-siddha",
        SunMoonYoga::Saadhya => "sun-moon-yoga-saadhya",
        SunMoonYoga::Subha => "sun-moon-yoga-subha",
        SunMoonYoga::Sukla => "sun-moon-yoga-sukla",
        SunMoonYoga::Brahma => "sun-moon-yoga-brahma",
        SunMoonYoga::Indra => "sun-moon-yoga-indra",
        SunMoonYoga::Vaidhriti => "sun-moon-yoga-vaidhriti",
    }
}

fn lunar_month_id(month: LunarMonth) -> &'static str {
    match month {
        LunarMonth::Chaitra => "lunar-month-chaitra",
        LunarMonth::Vaisaakha => "lunar-month-vaisaakha",
        LunarMonth::Jyeshtha => "lunar-month-jyeshtha",
        LunarMonth::Aashaadha => "lunar-month-aashaadha",
        LunarMonth::Sraavana => "lunar-month-sraavana",
        LunarMonth::Bhaadrapada => "lunar-month-bhaadrapada",
        LunarMonth::Aaswayuja => "lunar-month-aaswayuja",
        LunarMonth::Kaarteeka => "lunar-month-kaarteeka",
        LunarMonth::Maargasira => "lunar-month-maargasira",
        LunarMonth::Pushya => "lunar-month-pushya",
        LunarMonth::Maagha => "lunar-month-maagha",
        LunarMonth::Phaalguna => "lunar-month-phaalguna",
    }
}

fn twin_month_id(twin: TwinMonth) -> &'static str {
    match twin {
        TwinMonth::Adhika => "lunar-month-adhika",
        TwinMonth::Nija => "lunar-month-nija",
    }
}

fn karana_id(karana: Karana) -> &'static str {
    match karana {
        Karana::Bava => "karana-bava",
        Karana::Balava => "karana-balava",
        Karana::Kaulava => "karana-kaulava",
        Karana::Taitula => "karana-taitula",
        Karana::Garija => "karana-garija",
        Karana::Vanija => "karana-vanija",
        Karana::Vishti => "karana-vishti",
        Karana::Sakuna => "karana-sakuna",
        Karana::Chatushpada => "karana-chatushpada",
        Karana::Naga => "karana-naga",
        Karana::Kimstughna => "karana-kimstughna",
    }
}

fn vaara_id(vaara: Vaara) -> &'static str {
    match vaara {
        Vaara::Sunday => "vaara-sunday",
        Vaara::Monday => "vaara-monday",
        Vaara::Tuesday => "vaara-tuesday",
        Vaara::Wednesday => "vaara-wednesday",
        Vaara::Thursday => "vaara-thursday",
        Vaara::Friday => "vaara-friday",
        Vaara::Saturday => "vaara-saturday",
    }
}

/// Degrees in [0, 30) as degree, arc-minute and arc-second, truncated: 29°59′59.9″ stays 29°59′59″.
fn dms(degrees: f64) -> String {
    let seconds = hora_chart::arcseconds(degrees);
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
mod tests;

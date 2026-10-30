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
    Body, Karana, LunarMonth, Nakshatra, Paksha, Rasi, SunMoonYoga, Tithi, TwinMonth, Vaara,
};

mod chakra;
mod kb;
mod place;

pub use place::Places;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        // No Unicode isolation marks around placeables, so rendered text is plain.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

pub fn app(places: Arc<Places>) -> Router {
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
        .route("/en/", get(|uri| home(Lang::En, uri)))
        .route("/hu/", get(|uri| home(Lang::Hu, uri)))
        .route(
            "/en/chart",
            get(|places, uri, query| chart_page(Lang::En, places, uri, query)),
        )
        .route(
            "/hu/chart",
            get(|places, uri, query| chart_page(Lang::Hu, places, uri, query)),
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
        .with_state(places)
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
    chart: Vec<Row>,
    /// With a chart: its upagrahas' rows, each with its section on the upagraha page.
    upagrahas: Vec<(&'static str, Row)>,
    /// With a chart: the ayanamsa its positions are measured with; empty without one.
    ayanamsa: String,
    /// With a chart: (message id, clock time) for its Vedic day's sunrise, sunset and next sunrise; `None` where there is none (polar latitudes).
    vedic_day: Option<[(&'static str, String); 3]>,
    /// With a chart: its panchaanga at the birth; vaara and hora only with a Vedic day.
    panchaanga: Vec<Limb>,
    /// With a chart: when its tithi, nakshatra, Sun-Moon yoga and karana started and end.
    limb_spans: Vec<LimbSpan>,
    /// With a chart: it drawn in the chosen style.
    drawing: chakra::Drawing,
    /// With a chart: whether its stated offset is far from the place's local time, so its vaara and hora may be a day off.
    far_offset: bool,
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
            chart: Vec::new(),
            upagrahas: Vec::new(),
            ayanamsa: String::new(),
            vedic_day: None,
            panchaanga: Vec::new(),
            limb_spans: Vec::new(),
            drawing: chakra::Drawing::default(),
            far_offset: false,
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

async fn home(lang: Lang, uri: Uri) -> Response {
    render(StatusCode::OK, &uri, &Home::empty(lang, &uri))
}

async fn chart_page(
    lang: Lang,
    State(places): State<Arc<Places>>,
    uri: Uri,
    Query(mut form): Query<HashMap<String, String>>,
) -> Response {
    let mut page = Home::empty(lang, &uri);
    if let Some(pick) = form.remove("pick") {
        match pick.parse().ok().and_then(|id| places.get(id)) {
            Some(place) => {
                for (name, value) in [
                    ("place", place.name().to_owned()),
                    ("place-id", place.id.to_string()),
                    ("latitude", place.latitude.to_string()),
                    ("longitude", place.longitude.to_string()),
                    ("utc", "zone".to_owned()),
                ] {
                    form.insert(name.to_owned(), value);
                }
            }
            None => {
                page.errors.insert("place", "error-choice");
            }
        }
    }
    if let Some(offset) = form.remove("offset-pick") {
        form.insert("utc".to_owned(), "stated".to_owned());
        form.insert("offset".to_owned(), offset);
    }
    // A place counts as chosen while the field still holds one of its names; any other text is a search.
    let text = field(&form, "place");
    let place = field(&form, "place-id")
        .parse()
        .ok()
        .and_then(|id| places.get(id))
        .filter(|place| place.names().any(|name| name == text));
    if !page.errors.is_empty() || (place.is_none() && !text.is_empty()) {
        let status = if !page.errors.is_empty() {
            StatusCode::BAD_REQUEST
        } else if !place_query(text) {
            page.errors.insert("place", "error-place");
            StatusCode::BAD_REQUEST
        } else {
            page.places = places
                .search(text)
                .into_iter()
                .map(|(place, name)| (place.id, place_label(lang, &places, place, name)))
                .collect();
            if page.places.is_empty() {
                page.errors.insert("place", "error-place-none");
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::OK
            }
        };
        page.form = form;
        return render(status, &uri, &page);
    }
    // Typed coordinates replace the place's, and with them its zone.
    let at_place = |name, value| decimal(field(&form, name)) == Some(value);
    let place = place.filter(|place| {
        at_place("latitude", place.latitude) && at_place("longitude", place.longitude)
    });
    let zone = place.map(|place| places.zone(place));
    let status = match parse(&form, zone.map(|(_, tz)| tz)) {
        Err(refusal) => {
            page.errors = refusal.errors;
            page.offsets = refusal.offsets.into_iter().map(offset_text).collect();
            // A refused invalid request is never queued; a valid one costs a chart, but only on this refused path.
            if let Some(probe) = refusal.probe
                && let Err(hora_swe::Error::InvalidInput(rules)) = hora_swe::calculate(probe).await
            {
                for (field, message) in rules.into_iter().map(field_error) {
                    page.errors.entry(field).or_insert(message);
                }
            }
            StatusCode::BAD_REQUEST
        }
        Ok((request, reference, style)) => match hora_swe::calculate(request.clone()).await {
            Ok(sky) => {
                let chart = hora_chart::chart(&sky);
                let reference_rasi = hora_chart::rasi_of(&chart, reference);
                page.chart = rows(&chart, reference_rasi);
                page.upagrahas = upagraha_rows(&sky, reference_rasi);
                page.ayanamsa = dms(sky.ayanamsa);
                page.vedic_day = sky.vedic_day.map(|day| vedic_day_rows(lang, &request, day));
                page.panchaanga = panchaanga(lang, &sky);
                page.limb_spans = limb_spans(lang, &request, &sky.limbs);
                page.drawing = chakra::drawing(lang, style, &chart, reference);
                page.far_offset = far_offset(&request);
                if let (Some(place), Some((zone, _)), "zone", UtcOffset::Stated(seconds)) =
                    (place, zone, field(&form, "utc"), request.utc_offset)
                {
                    page.zone_used = Some(message(
                        lang,
                        "zone-used",
                        [
                            ("offset", offset_text(seconds)),
                            ("zone", zone.to_owned()),
                            ("place", format!("{}, {}", place.name(), place.country)),
                        ],
                    ));
                    // The local date in the Gregorian calendar, which tzdb's 1970 is in.
                    let local = sky.jd + f64::from(seconds) / 86400.0;
                    page.before_1970 = hora_swe::civil_date(local, Calendar::Gregorian)[0] < 1970;
                }
                StatusCode::OK
            }
            Err(hora_swe::Error::InvalidInput(rules)) => {
                page.errors = rules.into_iter().map(field_error).collect();
                StatusCode::BAD_REQUEST
            }
            Err(error) => return failure(lang, &uri, error),
        },
    };
    page.form = form;
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
    ("lahiri", Ayanamsa::Lahiri, "ayanamsa-lahiri"),
    ("true-chitra", Ayanamsa::TrueChitra, "ayanamsa-true-chitra"),
    ("raman", Ayanamsa::Raman, "ayanamsa-raman"),
    ("true-pushya", Ayanamsa::TruePushya, "ayanamsa-true-pushya"),
    ("true-mula", Ayanamsa::TrueMula, "ayanamsa-true-mula"),
];

const NODES: &Choices<NodeType> = &[
    ("mean", NodeType::Mean, "node-mean"),
    ("true", NodeType::True, "node-true"),
];

const SUNRISES: &Choices<SunriseDefinition> = &[
    (
        "upper-limb-apparent",
        SunriseDefinition::UpperLimbApparent,
        "sunrise-upper-limb-apparent",
    ),
    (
        "centre-true",
        SunriseDefinition::CentreTrue,
        "sunrise-centre-true",
    ),
    (
        "upper-limb-true",
        SunriseDefinition::UpperLimbTrue,
        "sunrise-upper-limb-true",
    ),
    (
        "centre-apparent",
        SunriseDefinition::CentreApparent,
        "sunrise-centre-apparent",
    ),
];

/// The lagna, then the grahas in the chart's order; special lagnas join in ch. 5.
const REFERENCES: &Choices<Body> = &[
    ("lagna", Body::Lagna, body_id(Body::Lagna)),
    ("sun", Body::Sun, body_id(Body::Sun)),
    ("moon", Body::Moon, body_id(Body::Moon)),
    ("mars", Body::Mars, body_id(Body::Mars)),
    ("mercury", Body::Mercury, body_id(Body::Mercury)),
    ("jupiter", Body::Jupiter, body_id(Body::Jupiter)),
    ("venus", Body::Venus, body_id(Body::Venus)),
    ("saturn", Body::Saturn, body_id(Body::Saturn)),
    ("rahu", Body::Rahu, body_id(Body::Rahu)),
    ("ketu", Body::Ketu, body_id(Body::Ketu)),
];

/// The three of ch. 1 §1.3.4; a display choice only (ADR 0003).
const STYLES: &Choices<chakra::Style> = &[
    ("south", chakra::Style::South, "style-south"),
    ("north", chakra::Style::North, "style-north"),
    ("east", chakra::Style::East, "style-east"),
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

/// Why a form gives no request.
struct Refusal {
    /// Field name → message id.
    errors: BTreeMap<&'static str, &'static str>,
    /// The offsets to choose from when the birth time fell in a clock change of the place's zone.
    offsets: Vec<i32>,
    /// When the zone refused the date or time: the request with a stand-in offset, for `hora_swe` to name the other fields' broken rules too.
    probe: Option<Request>,
}

impl From<BTreeMap<&'static str, &'static str>> for Refusal {
    fn from(errors: BTreeMap<&'static str, &'static str>) -> Self {
        Refusal {
            errors,
            offsets: Vec::new(),
            probe: None,
        }
    }
}

impl Refusal {
    fn at(field: &'static str, message: &'static str) -> Self {
        BTreeMap::from([(field, message)]).into()
    }
}

/// The form as a request, the bhavas' reference point and the chart style, or each unreadable field's message id; whether the values are in range is `hora_swe`'s to say.
/// With `utc=zone`, the offset is `zone`'s at the birth time (ADR 0005).
fn parse(
    form: &HashMap<String, String>,
    zone: Option<&TimeZone>,
) -> Result<(Request, Body, chakra::Style), Refusal> {
    let value = |name: &str| field(form, name);
    let calendar = choice(CALENDARS, value("calendar"));
    // Err inside: the offset comes from that zone once the date and time are read.
    let utc_offset = match value("utc") {
        "" | "stated" => offset(value("offset"))
            .map(|seconds| Ok(UtcOffset::Stated(seconds)))
            .ok_or(("offset", "error-offset")),
        "local-mean" => Ok(Ok(UtcOffset::LocalMean)),
        "local-apparent" => Ok(Ok(UtcOffset::LocalApparent)),
        "zone" => zone.map(Err).ok_or(("utc", "error-utc-zone")),
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
    let sunrise_definition = choice(SUNRISES, value("sunrise"));
    let reference = choice(REFERENCES, value("reference"));
    let style = choice(STYLES, value("style"));
    let time = time(value("time"));
    let date = date(value("date"));
    let errors: BTreeMap<_, _> = [
        calendar.is_none().then_some(("calendar", "error-choice")),
        utc_offset.err(),
        latitude.is_none().then_some(("latitude", "error-latitude")),
        longitude
            .is_none()
            .then_some(("longitude", "error-longitude")),
        altitude.is_none().then_some(("altitude", "error-altitude")),
        ayanamsa.is_none().then_some(("ayanamsa", "error-choice")),
        node.is_none().then_some(("node", "error-choice")),
        sunrise_definition
            .is_none()
            .then_some(("sunrise", "error-choice")),
        reference.is_none().then_some(("reference", "error-choice")),
        style.is_none().then_some(("style", "error-choice")),
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
        Some(sunrise_definition),
        Some(reference),
        Some(style),
    ) = (
        date,
        time,
        calendar,
        utc_offset,
        latitude,
        longitude,
        altitude,
        ayanamsa,
        node,
        sunrise_definition,
        reference,
        style,
    )
    else {
        return Err(errors.into());
    };
    let (utc_offset, refusal) = match utc_offset {
        Ok(utc_offset) => (utc_offset, None),
        Err(tz) => match zone_offset(tz, calendar, [year, month, day], [hour, minute, second]) {
            Ok(seconds) => (UtcOffset::Stated(seconds), None),
            Err(refusal) => {
                let stand_in = refusal.offsets.first().copied().unwrap_or(0);
                (UtcOffset::Stated(stand_in), Some(refusal))
            }
        },
    };
    let request = Request {
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
        sunrise_definition,
    };
    match refusal {
        None => Ok((request, reference, style)),
        Some(refusal) => Err(Refusal {
            probe: Some(request),
            ..refusal
        }),
    }
}

/// `tz`'s offset in seconds at a local date and time; jiff counts in the Gregorian calendar, so a Julian date is converted first.
fn zone_offset(
    tz: &TimeZone,
    calendar: Calendar,
    [year, month, day]: [i32; 3],
    [hour, minute, second]: [i32; 3],
) -> Result<i32, Refusal> {
    let hours = f64::from(hour) + f64::from(minute) / 60.0 + f64::from(second) / 3600.0;
    let jd = hora_swe::julian_day(year, month, day, hours, calendar).map_err(|error| {
        let rules = match error {
            hora_swe::Error::InvalidInput(rules) => rules,
            _ => vec![Rule::ImpossibleDate],
        };
        Refusal::from(
            rules
                .into_iter()
                .map(field_error)
                .collect::<BTreeMap<_, _>>(),
        )
    })?;
    let [year, month, day] = hora_swe::civil_date(jd, Calendar::Gregorian);
    // Out of jiff's range too, so a value too big for its type is refused like any other.
    let small = |n: i32| i8::try_from(n).unwrap_or(-1);
    let date = jiff::civil::Date::new(
        i16::try_from(year).unwrap_or(i16::MIN),
        small(month),
        small(day),
    )
    .map_err(|_| Refusal::at("date", "error-date"))?;
    let time = jiff::civil::Time::new(small(hour), small(minute), small(second), 0)
        .map_err(|_| Refusal::at("time", "error-time"))?;
    // hora never guesses across a clock change: the user states which offset was meant.
    let (message, before, after) = match tz.to_ambiguous_timestamp(date.to_datetime(time)).offset()
    {
        AmbiguousOffset::Unambiguous { offset } => return Ok(offset.seconds()),
        AmbiguousOffset::Gap { before, after } => ("error-time-gap", before, after),
        AmbiguousOffset::Fold { before, after } => ("error-time-fold", before, after),
    };
    Err(Refusal {
        offsets: vec![before.seconds(), after.seconds()],
        ..Refusal::at("time", message)
    })
}

/// Seconds as the form writes an offset: `±HH:MM`, with `:SS` only when there are any.
fn offset_text(seconds: i32) -> String {
    let sign = if seconds < 0 { '−' } else { '+' };
    let s = seconds.unsigned_abs();
    let (hour, minute, second) = (s / 3600, s / 60 % 60, s % 60);
    match second {
        0 => format!("{sign}{hour:02}:{minute:02}"),
        _ => format!("{sign}{hour:02}:{minute:02}:{second:02}"),
    }
}

/// 2 to 100 letters, digits, spaces and the marks place names use; nothing else reaches the search.
fn place_query(value: &str) -> bool {
    (2..=100).contains(&value.chars().count())
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || " -'’.".contains(c))
}

/// A search result as listed: the name matched (with the primary name when it differs), country, zone and coordinates, which tell same-named places apart.
fn place_label(lang: Lang, places: &Places, place: &place::Place, matched: &str) -> String {
    let name = match place.name() {
        primary if primary == matched => matched.to_owned(),
        primary => format!("{matched} ({primary})"),
    };
    message(
        lang,
        "place-result",
        [
            ("name", name),
            ("country", place.country.to_string()),
            ("zone", places.zone(place).0.to_owned()),
            ("latitude", format!("{:.2}", place.latitude)),
            ("longitude", format!("{:.2}", place.longitude)),
        ],
    )
}

/// A message with text arguments, in `lang`.
fn message<const N: usize>(lang: Lang, id: &str, args: [(&'static str, String); N]) -> String {
    let args = args
        .map(|(name, value)| (name.into(), FluentValue::from(value)))
        .into();
    LOCALES.lookup_with_args(&lang.id(), id, &args)
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
    /// Message ids; `name` is one of a graha, the lagna or an upagraha.
    name: String,
    rasi: &'static str,
    /// 1 to 12, counted from the reference point's rasi.
    bhava: u8,
    /// Degree, arc-minute and arc-second within the rasi.
    position: String,
    retrograde: bool,
    nakshatra: &'static str,
    /// 1 to 27, from Aswini.
    nakshatra_number: u8,
    pada: u8,
    lord: &'static str,
    /// `dignity-…`, none outside the graha's dignity rasis.
    dignity: Option<String>,
    rasi_lord: &'static str,
    /// `relationship-…`, the graha's compound relationship with `rasi_lord`.
    relationship: Option<String>,
}

/// The row of the point `name` at `longitude`, its bhava counted from `reference_rasi`; no R, dignity or relationship.
fn point_row(name: String, longitude: f64, reference_rasi: Rasi) -> Row {
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
    }
}

/// The table rows of `chart`, in its order, with bhavas counted from `reference_rasi`.
fn rows(chart: &[hora_chart::Entry; 10], reference_rasi: Rasi) -> Vec<Row> {
    chart
        .iter()
        .map(|entry| Row {
            retrograde: entry.retrograde,
            dignity: hora_chart::dignity(entry.body, entry.longitude)
                .map(|d| kb::message_id("dignity", d)),
            relationship: hora_chart::lord_relationship(chart, entry.body)
                .map(|r| kb::message_id("relationship", r)),
            ..point_row(body_id(entry.body).into(), entry.longitude, reference_rasi)
        })
        .collect()
}

/// The upagraha table's rows of `sky`, each with its section on the upagraha page, bhavas counted from `reference_rasi` (ch. 4).
fn upagraha_rows(sky: &Sky, reference_rasi: Rasi) -> Vec<(&'static str, Row)> {
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

/// A clock Julian day's time, dated when it falls on another civil date than the birth.
fn clock_time(lang: Lang, birth: &Request, jd: f64) -> String {
    let [year, month, day] = hora_swe::civil_date(jd, birth.calendar);
    if [year, month, day] == [birth.year, birth.month, birth.day] {
        return hms(jd);
    }
    let args = [("year", year), ("month", month), ("day", day)]
        .map(|(name, n)| (name.into(), FluentValue::from(n)))
        .into();
    let date = LOCALES.lookup_with_args(&lang.id(), "vedic-day-date", &args);
    format!("{} ({date})", hms(jd))
}

fn vedic_day_rows(
    lang: Lang,
    birth: &Request,
    day: hora_swe::VedicDay,
) -> [(&'static str, String); 3] {
    let time = |jd| clock_time(lang, birth, jd);
    [
        ("vedic-day-sunrise", time(first_second(day.clock_sunrise))),
        ("vedic-day-sunset", time(day.clock_sunset)),
        ("vedic-day-next-sunrise", time(day.clock_next_sunrise)),
    ]
}

/// One limb's message id, and when it started and ends in the birth's clock.
struct LimbSpan {
    name: &'static str,
    start: String,
    end: String,
}

/// When the birth's tithi, nakshatra, Sun-Moon yoga and karana started and end (ch. 1 §1.3.12).
fn limb_spans(lang: Lang, birth: &Request, limbs: &hora_swe::Limbs) -> Vec<LimbSpan> {
    ["tithi", "nakshatra", "sun-moon-yoga", "karana"]
        .into_iter()
        .zip(limbs.spans())
        .map(|(name, span)| LimbSpan {
            name,
            start: clock_time(lang, birth, first_second(span.clock_start)),
            end: clock_time(lang, birth, span.clock_end),
        })
        .collect()
}

/// One limb of the panchaanga: message ids for its name and lord, its value in the page's language.
struct Limb {
    name: &'static str,
    value: String,
    lord: Option<&'static str>,
}

/// The panchaanga at the birth (ch. 1 §1.3.12), the hora and the lunar month, each limb only when its inputs exist.
fn panchaanga(lang: Lang, sky: &Sky) -> Vec<Limb> {
    let t = |id| LOCALES.lookup(&lang.id(), id);
    let (sun, moon) = (sky.grahas.sun.longitude, sky.grahas.moon.longitude);
    let (index, paksha, tithi) = hora_chart::tithi(sun, moon);
    let (nakshatra, _) = hora_chart::nakshatra(moon);
    let yoga = hora_chart::sun_moon_yoga(sun, moon);
    let mut limbs = vec![
        Limb {
            name: "tithi",
            value: format!("{} {} ({index})", t(paksha_id(paksha)), t(tithi_id(tithi))),
            lord: Some(body_id(hora_chart::tithi_lord(tithi))),
        },
        Limb {
            name: "nakshatra",
            value: format!("{} ({})", t(nakshatra_id(nakshatra)), nakshatra as u8 + 1),
            lord: Some(body_id(hora_chart::vimsottari_lord(nakshatra))),
        },
        Limb {
            name: "sun-moon-yoga",
            value: format!("{} ({})", t(sun_moon_yoga_id(yoga)), yoga as u8 + 1),
            lord: None,
        },
        Limb {
            name: "karana",
            value: t(karana_id(hora_chart::karana(sun, moon))),
            lord: None,
        },
    ];
    if let Some(day) = sky.vedic_day {
        let vaara = hora_chart::vaara(&day);
        let (hora, lord) = hora_chart::hora(&day, sky.jd);
        limbs.insert(
            1,
            Limb {
                name: "vaara",
                value: t(vaara_id(vaara)),
                lord: Some(body_id(hora_chart::vaara_lord(vaara))),
            },
        );
        limbs.push(Limb {
            name: "hora",
            value: hora.to_string(),
            lord: Some(body_id(lord)),
        });
    }
    if let Some(new_moons) = &sky.new_moons {
        let (month, twin) = hora_chart::lunar_month(new_moons);
        let month = t(lunar_month_id(month));
        limbs.push(Limb {
            name: "lunar-month",
            value: match twin {
                Some(twin) => format!("{} {month}", t(twin_month_id(twin))),
                None => month,
            },
            lord: None,
        });
    }
    limbs
}

/// Whether a stated offset is over 4 h from the longitude's mean time, around the clock so Kiribati's +14 at 157°W is near.
fn far_offset(request: &Request) -> bool {
    // Real zones keep within about 3 h of their longitude (western China on Beijing time is the widest); more is likely another zone's clock, such as UT.
    let UtcOffset::Stated(seconds) = request.utc_offset else {
        return false;
    };
    let gap = (f64::from(seconds) / 3600.0 - request.longitude / 15.0).rem_euclid(24.0);
    gap.min(24.0 - gap) > 4.0
}

/// A start rounded up for `hms`, which truncates, so a birth at the second shown is inside what it starts.
fn first_second(jd: f64) -> f64 {
    jd + 0.999 / 86400.0
}

/// A clock Julian day's time of day, truncated to the second like `dms`.
fn hms(jd: f64) -> String {
    // A Julian day starts at noon; just under midnight, the product can round up to 86400.
    let seconds = (((jd + 0.5).rem_euclid(1.0) * 86400.0) as u32).min(86399);
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60
    )
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
        app(places())
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    /// The dev shell's places file, loaded once for every test.
    fn places() -> Arc<Places> {
        static PLACES: std::sync::LazyLock<Arc<Places>> = std::sync::LazyLock::new(|| {
            let var = |name| {
                std::env::var_os(name).unwrap_or_else(|| panic!("{name}, set by the dev shell"))
            };
            let tzdb = jiff::tz::TimeZoneDatabase::from_dir(var("TZDIR")).unwrap();
            Arc::new(Places::load(var("HORA_PLACES").as_ref(), &tzdb).unwrap())
        });
        PLACES.clone()
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
    async fn no_field_offers_a_keypad_without_a_minus_sign() {
        // iOS numeric and decimal keypads have no "-", which dates and south/west coordinates need.
        let home = body("/en/").await;
        assert!(!home.contains("inputmode"), "{home}");
    }

    #[tokio::test]
    async fn the_footer_links_to_the_privacy_page_in_its_language() {
        for (lang, heading) in [("en", "Privacy"), ("hu", "Adatvédelem")] {
            let home = body(&format!("/{lang}/")).await;
            assert!(
                home.contains(&format!(r#"href="/{lang}/privacy""#)),
                "{home}"
            );
            let privacy = body(&format!("/{lang}/privacy")).await;
            assert!(
                privacy.contains(&format!("<h1>{heading}</h1>")),
                "{privacy}"
            );
            assert!(privacy.contains("Cloudflare"), "{privacy}");
        }
    }

    #[tokio::test]
    async fn the_footer_links_to_the_disclaimer_in_its_language() {
        for (lang, heading, text) in [
            ("en", "Disclaimer", "not medical, legal"),
            ("hu", "Felelősségkizárás", "nem orvosi, jogi"),
        ] {
            let home = body(&format!("/{lang}/")).await;
            assert!(
                home.contains(&format!(r#"href="/{lang}/disclaimer""#)),
                "{home}"
            );
            let disclaimer = body(&format!("/{lang}/disclaimer")).await;
            assert!(
                disclaimer.contains(&format!("<h1>{heading}</h1>")),
                "{disclaimer}"
            );
            assert!(disclaimer.contains(text), "{disclaimer}");
        }
    }

    #[tokio::test]
    async fn the_footer_links_the_version_to_its_tagged_source() {
        let version = env!("CARGO_PKG_VERSION");
        for (lang, label) in [("en", "Version"), ("hu", "Verzió")] {
            let home = body(&format!("/{lang}/")).await;
            assert!(
                home.contains(&format!(
                    r#"<a href="{}/tree/v{version}">{label} {version}</a>"#,
                    env!("CARGO_PKG_REPOSITORY")
                )),
                "{home}"
            );
        }
    }

    #[tokio::test]
    async fn every_page_names_the_icons_and_they_are_served() {
        let home = body("/en/").await;
        for link in [
            r#"<link rel="icon" href="/favicon.svg" type="image/svg+xml">"#,
            r#"<link rel="apple-touch-icon" href="/apple-touch-icon.png">"#,
            r#"<meta name="apple-mobile-web-app-title" content="hora">"#,
        ] {
            assert!(home.contains(link), "{home}");
        }
        for (uri, content_type) in [
            ("/favicon.svg", "image/svg+xml"),
            ("/apple-touch-icon.png", "image/png"),
        ] {
            let response = get(uri, None).await;
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            assert_eq!(response.headers()[header::CONTENT_TYPE], content_type);
            assert!(response.headers().contains_key(header::ETAG), "{uri}");
        }
    }

    #[tokio::test]
    async fn robots_txt_keeps_crawlers_off_the_chart_pages() {
        let response = get("/robots.txt", None).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/plain; charset=utf-8"
        );
        let robots = body("/robots.txt").await;
        for lang in ["en", "hu"] {
            assert!(
                robots.contains(&format!("Disallow: /{lang}/chart\n")),
                "{robots}"
            );
        }
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
        for path in ["/en", "/hu", "/en/kb", "/hu/kb"] {
            let response = get(path, None).await;
            assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
            assert_eq!(response.headers()[header::LOCATION], format!("{path}/"));
        }
    }

    #[tokio::test]
    async fn every_page_links_to_the_knowledge_base() {
        for (uri, link) in [
            ("/en/", r#"<a href="/en/kb/">Knowledge base</a>"#),
            ("/hu/", r#"<a href="/hu/kb/">Tudástár</a>"#),
            ("/hu/kb/rasi", r#"<a href="/hu/kb/">Tudástár</a>"#),
        ] {
            assert!(body(uri).await.contains(link), "{uri}");
        }
    }

    #[tokio::test]
    async fn the_kb_index_lists_the_learning_path_with_book_references() {
        let en = body("/en/kb/").await;
        assert!(en.contains("<h1>Knowledge base</h1>"), "{en}");
        let first = r#"<li><a href="/en/kb/jyotish">Jyotish</a> (§ 1.1–1.2)</li>"#;
        let graha = r#"<li><a href="/en/kb/graha">Graha</a> (§ 1.3.1)</li>"#;
        let rasi = r#"<li><a href="/en/kb/rasi">Rasi</a> (§ 1.3.2)</li>"#;
        let bhava = r#"<li><a href="/en/kb/bhava">Bhava</a> (§ 1.3.3)</li>"#;
        let chart = r#"<li><a href="/en/kb/chart">Chart</a> (§ 1.3.4)</li>"#;
        assert!(
            en.contains(first) && en.find(first) < en.find(graha),
            "{en}"
        );
        assert!(en.find(graha) < en.find(rasi), "{en}");
        assert!(en.find(rasi) < en.find(bhava), "{en}");
        assert!(en.find(bhava) < en.find(chart), "{en}");
        let varga = r#"<li><a href="/en/kb/varga-chakra">Varga chakra</a> (§ 1.3.5)</li>"#;
        assert!(en.find(chart) < en.find(varga), "{en}");
        let nakshatra = r#"<li><a href="/en/kb/nakshatra">Nakshatra</a> (§ 1.3.6)</li>"#;
        assert!(en.find(varga) < en.find(nakshatra), "{en}");
        let tithi = r#"<li><a href="/en/kb/tithi">Tithi</a> (§ 1.3.8.1)</li>"#;
        assert!(en.find(nakshatra) < en.find(tithi), "{en}");
        let ayanamsa = r#"<li><a href="/en/kb/ayanamsa">Ayanamsa</a> (§ 1.3.13)</li>"#;
        let month = r#"<li><a href="/en/kb/lunar-month">Lunar month</a> (§ 1.3.8.2)</li>"#;
        assert!(en.find(tithi) < en.find(month), "{en}");
        let yoga = r#"<li><a href="/en/kb/sun-moon-yoga">Sun-Moon yoga</a> (§ 1.3.9)</li>"#;
        assert!(en.find(month) < en.find(yoga), "{en}");
        let karana = r#"<li><a href="/en/kb/karana">Karana</a> (§ 1.3.10)</li>"#;
        assert!(en.find(yoga) < en.find(karana), "{en}");
        let vaara_and_hora =
            r#"<li><a href="/en/kb/vaara-and-hora">Vaara and hora</a> (§ 1.3.11)</li>"#;
        assert!(en.find(karana) < en.find(vaara_and_hora), "{en}");
        let panchaanga = r#"<li><a href="/en/kb/panchaanga">Panchaanga</a> (§ 1.3.12)</li>"#;
        assert!(en.find(vaara_and_hora) < en.find(panchaanga), "{en}");
        assert!(en.find(panchaanga) < en.find(ayanamsa), "{en}");
        let dasa = r#"<li><a href="/en/kb/dasa">Dasa</a> (§ 1.4)</li>"#;
        assert!(en.find(ayanamsa) < en.find(dasa), "{en}");
        let characteristics = r#"<li><a href="/en/kb/rasi-characteristics">Rasi characteristics</a> (§ 2.2–2.3)</li>"#;
        assert!(en.find(dasa) < en.find(characteristics), "{en}");
        let [basic, rasis] = [
            r#"<details>
<summary><h2>Chapter 1: Basic concepts</h2></summary>
<ol id="chapter-1">"#,
            r#"<details>
<summary><h2>Chapter 2: Rasis</h2></summary>
<ol id="chapter-2">"#,
        ];
        assert!(en.find(basic) < en.find(first), "{en}");
        assert!(en.find(dasa) < en.find(rasis), "{en}");
        assert!(en.find(rasis) < en.find(characteristics), "{en}");
        let grahas = r#"<details>
<summary><h2>Chapter 3: Grahas</h2></summary>
<ol id="chapter-3">"#;
        let graha_characteristics = r#"<li><a href="/en/kb/graha-characteristics">Graha characteristics</a> (§ 3.1–3.2)</li>"#;
        assert!(en.find(characteristics) < en.find(grahas), "{en}");
        assert!(en.find(grahas) < en.find(graha_characteristics), "{en}");
        let hu = body("/hu/kb/").await;
        let first = r#"<li><a href="/hu/kb/jyotish">Jyotish</a> (§ 1.1–1.2)</li>"#;
        let graha = r#"<li><a href="/hu/kb/graha">Graha</a> (§ 1.3.1)</li>"#;
        let rasi = r#"<li><a href="/hu/kb/rasi">Rasi</a> (§ 1.3.2)</li>"#;
        let bhava = r#"<li><a href="/hu/kb/bhava">Bhava</a> (§ 1.3.3)</li>"#;
        let chart = r#"<li><a href="/hu/kb/chart">Chakra</a> (§ 1.3.4)</li>"#;
        assert!(
            hu.contains(first) && hu.find(first) < hu.find(graha),
            "{hu}"
        );
        assert!(hu.find(graha) < hu.find(rasi), "{hu}");
        assert!(hu.find(rasi) < hu.find(bhava), "{hu}");
        assert!(hu.find(bhava) < hu.find(chart), "{hu}");
        let varga = r#"<li><a href="/hu/kb/varga-chakra">Varga chakra</a> (§ 1.3.5)</li>"#;
        assert!(hu.find(chart) < hu.find(varga), "{hu}");
        let nakshatra = r#"<li><a href="/hu/kb/nakshatra">Nakshatra</a> (§ 1.3.6)</li>"#;
        assert!(hu.find(varga) < hu.find(nakshatra), "{hu}");
        let tithi = r#"<li><a href="/hu/kb/tithi">Tithi</a> (§ 1.3.8.1)</li>"#;
        assert!(hu.find(nakshatra) < hu.find(tithi), "{hu}");
        let ayanamsa = r#"<li><a href="/hu/kb/ayanamsa">Ayanamsa</a> (§ 1.3.13)</li>"#;
        let month = r#"<li><a href="/hu/kb/lunar-month">Holdhónap</a> (§ 1.3.8.2)</li>"#;
        assert!(hu.find(tithi) < hu.find(month), "{hu}");
        let yoga = r#"<li><a href="/hu/kb/sun-moon-yoga">Nap–Hold yoga</a> (§ 1.3.9)</li>"#;
        assert!(hu.find(month) < hu.find(yoga), "{hu}");
        let karana = r#"<li><a href="/hu/kb/karana">Karana</a> (§ 1.3.10)</li>"#;
        assert!(hu.find(yoga) < hu.find(karana), "{hu}");
        let vaara_and_hora =
            r#"<li><a href="/hu/kb/vaara-and-hora">Vaara és hora</a> (§ 1.3.11)</li>"#;
        assert!(hu.find(karana) < hu.find(vaara_and_hora), "{hu}");
        let panchaanga = r#"<li><a href="/hu/kb/panchaanga">Panchaanga</a> (§ 1.3.12)</li>"#;
        assert!(hu.find(vaara_and_hora) < hu.find(panchaanga), "{hu}");
        assert!(hu.find(panchaanga) < hu.find(ayanamsa), "{hu}");
        let dasa = r#"<li><a href="/hu/kb/dasa">Dasa</a> (§ 1.4)</li>"#;
        assert!(hu.find(ayanamsa) < hu.find(dasa), "{hu}");
        let rasis = r#"<details>
<summary><h2>2. fejezet: Rasik</h2></summary>
<ol id="chapter-2">"#;
        assert!(hu.find(dasa) < hu.find(rasis), "{hu}");
    }

    #[tokio::test]
    async fn a_kb_page_names_its_chapter_and_links_its_neighbours_on_the_path() {
        let en = body("/en/kb/rasi").await;
        for text in [
            r#"<a href="/en/kb/#chapter-1">Chapter 1: Basic concepts</a>"#,
            r#"<a href="/en/kb/graha" rel="prev"><span>Previous</span> Graha</a>"#,
            r#"<a href="/en/kb/bhava" rel="next"><span>Next</span> Bhava</a>"#,
        ] {
            assert!(en.contains(text), "{text}: {en}");
        }
        let hu = body("/hu/kb/rasi-characteristics").await;
        assert!(
            hu.contains(r#"<a href="/hu/kb/#chapter-2">2. fejezet: Rasik</a>"#),
            "{hu}"
        );
        assert!(
            hu.contains(r#"<a href="/hu/kb/dasa" rel="prev"><span>Előző</span> Dasa</a>"#),
            "{hu}"
        );
        assert!(
            hu.contains(r#"<a href="/hu/kb/graha-characteristics" rel="next"><span>Következő</span> A grahák jellemzői</a>"#),
            "{hu}"
        );
        // The path's ends link one way only.
        let last = kb::PAGES.last().unwrap().slug;
        assert!(
            !body(&format!("/hu/kb/{last}"))
                .await
                .contains(r#"rel="next""#),
            "{last}"
        );
        assert!(!body("/en/kb/jyotish").await.contains(r#"rel="prev""#));
    }

    #[tokio::test]
    async fn a_kb_page_shows_its_table_book_reference_and_link_in_its_language() {
        let book = r#"<a href="https://www.vedicastrologer.org/articles/astro_books.htm">"#;
        let en = body("/en/kb/rasi").await;
        for text in [
            "<title>Rasi – hora</title>",
            "<td>Aries</td>",
            "Book reference:",
            "chapter 1, § 1.3.2",
            book,
        ] {
            assert!(en.contains(text), "{text}: {en}");
        }
        assert!(en.contains(r#"href="/hu/kb/rasi""#), "{en}");
        let hu = body("/hu/kb/rasi").await;
        for text in [
            "<td>Kos</td>",
            "Hivatkozás a könyvre:",
            "1. fejezet, § 1.3.2",
            book,
        ] {
            assert!(hu.contains(text), "{text}: {hu}");
        }
    }

    #[tokio::test]
    async fn an_unknown_kb_page_is_not_found_in_its_language() {
        for (uri, text) in [
            (
                "/en/kb/rashi",
                "There is no knowledge base page at this address.",
            ),
            ("/hu/kb/rashi", "Ezen a címen nincs tudástár-oldal."),
            (
                "/en/kb/..%2Fmain",
                "There is no knowledge base page at this address.",
            ),
        ] {
            let response = get(uri, None).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{uri}");
            assert!(page(response).await.contains(text), "{uri}");
        }
    }

    #[tokio::test]
    async fn other_languages_are_not_found() {
        assert_eq!(get("/de/", None).await.status(), StatusCode::NOT_FOUND);
    }

    /// Chart 1's birth data (ch. 9 *Arudha Padas*, p. 87) as the book prints it: 2000-04-09 17:55 at UTC−4:00, 71°W12′, 42°N30′.
    const CHART_1: &str = "date=2000-04-09&time=17%3A55&calendar=gregorian&utc=stated&offset=-04%3A00\
        &latitude=42.5&longitude=-71.2&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south";

    /// The chart table, without the form that echoes the request.
    fn table(page: &str) -> &str {
        let start = page.find("<table").expect("a table");
        &page[start..page.find("</table>").expect("a table end")]
    }

    /// One `<tr>` holding all of `cells` in order, or `None`.
    fn row<'a>(table: &'a str, cells: &[&str]) -> Option<&'a str> {
        table.split("<tr").find(|row| {
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
        let en = body(&format!("/en/chart?{CHART_1}")).await;
        let hu = body(&format!("/hu/chart?{CHART_1}")).await;
        // The book prints no ayanamsa value; this is the library's Lahiri at chart 1, truncated like the grahas.
        for page in [&en, &hu] {
            assert!(
                page.contains(r#"/kb/ayanamsa">Ayanamsa</a>: 23°51′39″</p>"#),
                "{page}"
            );
        }
        for (page, headings) in [
            (
                &en,
                [
                    "Graha",
                    "Rasi",
                    "Bhava",
                    "Nakshatra",
                    "Pada",
                    "Vimsottari lord",
                ],
            ),
            (
                &hu,
                [
                    "Graha",
                    "Rasi",
                    "Bhava",
                    "Nakshatra",
                    "Pada",
                    "Vimsottari ura",
                ],
            ),
        ] {
            let table = table(page);
            assert!(row(table, &headings).is_some(), "{table}");
            // Of chart 1's grahas only the mean nodes move backward; Mars has one name in both languages.
            for (graha, marked) in [("Mars", false), ("Rahu", true), ("Ketu", true)] {
                let marks = row(table, &[&format!(">{graha}<"), ">R<"]).is_some();
                assert_eq!(marks, marked, "{graha}: {table}");
            }
        }
        for (
            (en_graha, en_rasi, en_lord),
            (hu_graha, hu_rasi, hu_lord),
            position,
            nakshatra,
            pada,
        ) in books
        {
            for (page, graha, rasi, lord) in [
                (&en, en_graha, en_rasi, en_lord),
                (&hu, hu_graha, hu_rasi, hu_lord),
            ] {
                let cells = [
                    format!(">{graha}<"),
                    format!(">{rasi}<"),
                    format!(">{position}"),
                    format!(">{nakshatra}<"),
                    format!(">{pada}<"),
                    format!(">{lord}<"),
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
            let panchaanga = section(&page, "panchaanga");
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
            let panchaanga = section(&page, "panchaanga");
            assert_eq!(panchaanga.matches(&href).count(), 1, "{panchaanga}");
        }
    }

    /// The ayanamsa page's worked examples: GREENWICH_2000 under each ayanamsa, and Lahiri a century either side.
    #[tokio::test]
    async fn the_ayanamsa_pages_examples_are_horas_charts() {
        const SOURCES: [(&str, &str); 2] = [
            ("en", include_str!("../kb/en/ayanamsa.md")),
            ("hu", include_str!("../kb/hu/ayanamsa.md")),
        ];
        /// The chart's page, once its ayanamsa is checked and both pages are found to link it and quote the value.
        async fn chart(year: &str, ayanamsa: &str, value: &str) -> String {
            let query = GREENWICH_2000
                .replace("date=2000", &format!("date={year}"))
                .replace("ayanamsa=lahiri", &format!("ayanamsa={ayanamsa}"));
            let page = body(&format!("/en/chart?{query}")).await;
            assert!(page.contains(&format!("Ayanamsa</a>: {value}")), "{page}");
            for (lang, source) in SOURCES {
                for text in [&format!("(/{lang}/chart?{query})"), value] {
                    assert!(source.contains(text), "{lang}: {text}");
                }
            }
            page
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
            let page = chart("2000", ayanamsa, value).await;
            let table = table(&page);
            let cells = [
                ">Sun<",
                ">Sagittarius<",
                &format!(">{bhava}<"),
                &format!("<td>{sun}</td>"),
                ">Poorvaashaadha (20)<",
                &format!("<td>{pada}</td>"),
            ];
            assert!(row(table, &cells).is_some(), "{ayanamsa}: {table}");
            assert!(
                row(table, &[">Lagna<", &format!(">{lagna}<")]).is_some(),
                "{ayanamsa}: {table}"
            );
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

    #[tokio::test]
    async fn the_panchaangas_tithi_links_to_its_kb_page() {
        for lang in ["en", "hu"] {
            let page = body(&format!("/{lang}/chart?{GREENWICH_2000}")).await;
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
            let page = body(&format!("/{lang}/chart?{GREENWICH_2000}")).await;
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
            let page = body(&format!("/{lang}/chart?{GREENWICH_2000}")).await;
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
            let page = body(&format!("/{lang}/chart?{GREENWICH_2000}")).await;
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
            let page = body(&format!("/{lang}/chart?{GREENWICH_2000}")).await;
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
            let page = body(&format!("/en/chart?{query}")).await;
            let day = section(&page, "vedic-day");
            assert!(row(day, &[">Sunrise<", sunrise]).is_some(), "{day}");
            assert!(
                row(day, &[">Next sunrise<", next_sunrise]).is_some(),
                "{day}"
            );
            let panchaanga = section(&page, "panchaanga");
            for cells in [[">Vaara<", vaara, vaara_lord], [">Hora<", hora, hora_lord]] {
                assert!(row(panchaanga, &cells).is_some(), "{panchaanga}");
            }
            for (lang, source) in [
                ("en", include_str!("../kb/en/vaara-and-hora.md")),
                ("hu", include_str!("../kb/hu/vaara-and-hora.md")),
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
            let heading = format!(r#"<th scope="col"><a href="/{lang}/kb/dasa">{name}</a></th>"#);
            assert!(table(&page).contains(&heading), "{page}");
        }
    }

    #[tokio::test]
    async fn the_panchaanga_heading_links_to_its_kb_page() {
        for lang in ["en", "hu"] {
            let page = body(&format!("/{lang}/chart?{GREENWICH_2000}")).await;
            let heading = format!(
                r#"<h2 id="panchaanga"><a href="/{lang}/kb/panchaanga">Panchaanga</a></h2>"#
            );
            assert!(section(&page, "panchaanga").contains(&heading), "{page}");
        }
    }

    /// The panchaanga page's worked example: GREENWICH_2000's five limbs.
    #[tokio::test]
    async fn the_panchaanga_pages_example_is_horas_chart() {
        let query = GREENWICH_2000;
        let page = body(&format!("/en/chart?{query}")).await;
        let panchaanga = section(&page, "panchaanga");
        for cells in [
            [">Tithi<", ">Krishna Ekadasi (26)<", ">Mars<"],
            [">Vaara<", ">Saturday<", ">Saturn<"],
            [">Nakshatra<", ">Swaati (15)<", ">Rahu<"],
        ] {
            assert!(row(panchaanga, &cells).is_some(), "{panchaanga}");
        }
        for cells in [[">Sun-Moon yoga<", ">Dhriti (8)<"], [">Karana<", ">Bava<"]] {
            assert!(row(panchaanga, &cells).is_some(), "{panchaanga}");
        }
        let spans = section(&page, "limb-spans");
        for cells in [
            [">Tithi<", ">05:34:43<", ">07:50:31 (January 2, 2000)<"],
            [
                ">Nakshatra<",
                ">10:37:39 (December 31, 1999)<",
                ">13:03:25<",
            ],
            [
                ">Sun-Moon yoga<",
                ">07:06:45<",
                ">07:43:37 (January 2, 2000)<",
            ],
            [">Karana<", ">05:34:43<", ">18:40:01<"],
        ] {
            assert!(row(spans, &cells).is_some(), "{spans}");
        }
        let day = section(&page, "vedic-day");
        for cells in [[">Sunrise<", "08:05:47"], [">Next sunrise<", "08:05:41"]] {
            assert!(row(day, &cells).is_some(), "{day}");
        }
        for (lang, source, limbs) in [
            (
                "en",
                include_str!("../kb/en/panchaanga.md"),
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
                include_str!("../kb/hu/panchaanga.md"),
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
        let page = body(&format!("/en/chart?{query}")).await;
        let panchaanga = section(&page, "panchaanga");
        assert!(
            row(panchaanga, &[">Tithi<", ">Krishna Ekadasi (26)<"]).is_some(),
            "{panchaanga}"
        );
        assert!(
            row(panchaanga, &[">Karana<", ">Bava<"]).is_some(),
            "{panchaanga}"
        );
        for (lang, source) in [
            ("en", include_str!("../kb/en/karana.md")),
            ("hu", include_str!("../kb/hu/karana.md")),
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
        let page = body(&format!("/en/chart?{query}")).await;
        let table = table(&page);
        assert!(
            row(table, &[">Sun<", ">Sagittarius<", "<td>16°31′17″</td>"]).is_some(),
            "{table}"
        );
        assert!(
            row(table, &[">Moon<", ">Libra<", "<td>19°28′14″</td>"]).is_some(),
            "{table}"
        );
        let panchaanga = section(&page, "panchaanga");
        assert!(
            row(panchaanga, &[">Sun-Moon yoga<", ">Dhriti (8)<"]).is_some(),
            "{panchaanga}"
        );
        for (lang, source) in [
            ("en", include_str!("../kb/en/sun-moon-yoga.md")),
            ("hu", include_str!("../kb/hu/sun-moon-yoga.md")),
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
            ("en", include_str!("../kb/en/lunar-month.md")),
            ("hu", include_str!("../kb/hu/lunar-month.md")),
        ];
        /// The chart's page at `date` and `time` UT, once both pages are found to link it.
        async fn chart(date: &str, time: &str) -> String {
            let query = GREENWICH_2000
                .replace("2000-01-01", date)
                .replace("12%3A00", time);
            for (lang, source) in SOURCES {
                let link = format!("(/{lang}/chart?{query})");
                assert!(source.contains(&link), "{lang}: {link}");
            }
            body(&format!("/en/chart?{query}")).await
        }
        // GREENWICH_2000, then the second after each conjunction: the Sun and the Moon together, Sukla Pratipat just started.
        let charts = [
            ("2000-01-01", "12%3A00", "Maargasira", None),
            (
                "1999-12-07",
                "22%3A32%3A23",
                "Maargasira",
                Some((">Scorpio<", "21°31′26″", "21°31′26″", "22:32:22")),
            ),
            (
                "2023-07-17",
                "18%3A32%3A33",
                "Adhika Sraavana",
                Some((">Cancer<", "0°45′29″", "0°45′29″", "18:32:32")),
            ),
            (
                "2023-08-16",
                "09%3A38%3A55",
                "Nija Sraavana",
                Some((">Cancer<", "29°06′30″", "29°06′31″", "09:38:54")),
            ),
        ];
        for (date, time, month, conjunction) in charts {
            let page = chart(date, time).await;
            let panchaanga = section(&page, "panchaanga");
            assert!(
                panchaanga.contains(&format!(">{month}</a></td>")),
                "{date}: {panchaanga}"
            );
            for (lang, source) in SOURCES {
                assert!(source.contains(month), "{lang}: {month}");
            }
            let Some((rasi, sun, moon, conjunction)) = conjunction else {
                continue;
            };
            assert!(
                row(panchaanga, &[">Tithi<", ">Sukla Pratipat (1)<"]).is_some(),
                "{date}: {panchaanga}"
            );
            // The chart is cast at the tithi's start: the conjunction's second, rounded up.
            let start = format!("<td>{}</td>", time.replace("%3A", ":"));
            let spans = section(&page, "limb-spans");
            assert!(
                row(spans, &[">Tithi<", &start]).is_some(),
                "{date}: {spans}"
            );
            for (lang, source) in SOURCES {
                assert!(source.contains(conjunction), "{lang}: {conjunction}");
            }
            let table = table(&page);
            for (graha, value) in [(">Sun<", sun), (">Moon<", moon)] {
                let cell = format!("<td>{value}</td>");
                assert!(
                    row(table, &[graha, rasi, &cell]).is_some(),
                    "{date}: {table}"
                );
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
        let page = body(&format!("/en/chart?{query}")).await;
        let table = table(&page);
        assert!(
            row(table, &[">Sun<", ">Sagittarius<", "<td>16°31′17″</td>"]).is_some(),
            "{table}"
        );
        assert!(
            row(table, &[">Moon<", ">Libra<", "<td>19°28′14″</td>"]).is_some(),
            "{table}"
        );
        let panchaanga = section(&page, "panchaanga");
        assert!(
            row(panchaanga, &[">Tithi<", ">Krishna Ekadasi (26)<", ">Mars<"]).is_some(),
            "{panchaanga}"
        );
        for (lang, source) in [
            ("en", include_str!("../kb/en/tithi.md")),
            ("hu", include_str!("../kb/hu/tithi.md")),
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
        let page = body(&format!("/en/chart?{query}")).await;
        assert!(
            row(
                table(&page),
                &[
                    ">Sun<",
                    "<td>16°31′17″</td>",
                    ">Poorvaashaadha (20)<",
                    "<td>1</td>",
                    ">Venus<"
                ]
            )
            .is_some(),
            "{page}"
        );
        for (lang, source) in [
            ("en", include_str!("../kb/en/nakshatra.md")),
            ("hu", include_str!("../kb/hu/nakshatra.md")),
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

    /// The KB's worked examples: noon UT, 1 January 2000, at Greenwich.
    const GREENWICH_2000: &str = "date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
        &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south";

    /// The rasi page's worked example is hora's own Sun at noon UT, 1 January 2000, at Greenwich.
    #[tokio::test]
    async fn the_rasi_pages_example_is_horas_chart() {
        let query = GREENWICH_2000;
        let page = body(&format!("/en/chart?{query}")).await;
        assert!(
            row(
                table(&page),
                &[">Sun<", ">Sagittarius<", "<td>16°31′17″</td>"]
            )
            .is_some(),
            "{page}"
        );
        assert!(page.contains("Ayanamsa</a>: 23°51′25″"), "{page}");
        for (lang, source) in [
            ("en", include_str!("../kb/en/rasi.md")),
            ("hu", include_str!("../kb/hu/rasi.md")),
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
            let page = body(&format!(
                "/en/chart?{query}&reference={reference}&style=south"
            ))
            .await;
            for (graha, rasi, bhava) in bhavas {
                let cells = [
                    format!(">{graha}<"),
                    format!(">{rasi}</a></td><td><a href=\"/en/kb/bhava\">{bhava}<"),
                ];
                let cells = cells.each_ref().map(String::as_str);
                assert!(
                    row(table(&page), &cells).is_some(),
                    "{reference} {graha}: {page}"
                );
            }
        }
        for (lang, source) in [
            ("en", include_str!("../kb/en/bhava.md")),
            ("hu", include_str!("../kb/hu/bhava.md")),
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

    /// hora's own example chart, the chart KB page's, without a style.
    const HORAS_CHART: &str = "date=2000-01-01&time=06%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
        &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna";

    /// The chart KB page's example is hora's chart, drawn in each style.
    #[tokio::test]
    async fn the_chart_pages_example_is_horas_chart() {
        let page = body(&format!("/en/chart?{HORAS_CHART}&style=north")).await;
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
            let cells = [format!(">{point}<"), format!(">{rasi}<")];
            let cells = cells.each_ref().map(String::as_str);
            let row = row(table(&page), &cells).unwrap_or_else(|| panic!("{point}: {page}"));
            assert_eq!(row.contains("<abbr"), retrograde, "{row}");
        }
        for text in [
            r#"aria-label="Lagna, 17 degrees 14 minutes in Scorpio""#,
            r#"aria-label="Venus, 7 degrees 25 minutes in Scorpio""#,
            r#"aria-label="Saturn, retrograde, 16 degrees 32 minutes in Aries""#,
        ] {
            assert!(page.contains(text), "{text}: {page}");
        }
        // The page walks the bhavas from the top diamond: Scorpio's 8, then on round to Libra's 7.
        let numbers: Vec<_> = page
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
            ("en", include_str!("../kb/en/chart.md")),
            ("hu", include_str!("../kb/hu/chart.md")),
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

    /// Saturn in Aries is neecha, sama with Mars (ch. 3 §3.3–3.4); the graha-relationships page works its example on this chart.
    #[tokio::test]
    async fn the_chart_page_shows_each_grahas_dignity_and_rasi_lord_as_the_kb_example_says() {
        for (lang, moon, rows) in [
            (
                "en",
                "Moon",
                [
                    ("Saturn", "neecha", "neecha", "Mars", "sama"),
                    ("Jupiter", "", "—", "Mars", "adhimitra"),
                    ("Mercury", "", "—", "Jupiter", "satru"),
                    ("Venus", "", "—", "Mars", "mitra"),
                ],
            ),
            (
                "hu",
                "Hold",
                [
                    ("Szaturnusz", "neecha", "neecha", "Mars", "sama"),
                    ("Jupiter", "", "—", "Mars", "adhimitra"),
                    ("Merkúr", "", "—", "Jupiter", "satru"),
                    ("Vénusz", "", "—", "Mars", "mitra"),
                ],
            ),
        ] {
            let page = body(&format!("/{lang}/chart?{HORAS_CHART}&style=south")).await;
            let kb = format!("/{lang}/kb");
            let lord = |anchor: &str, name: &str| {
                format!(r#"<td><a href="{kb}/graha-characteristics#{anchor}">{name}</a>"#)
            };
            for (graha, anchor, dignity, lord_name, relationship) in rows {
                let dignity = if anchor.is_empty() {
                    format!("<td>{dignity}</td>")
                } else {
                    format!(r#"<td><a href="{kb}/dignity#{anchor}">{dignity}</a></td>"#)
                };
                let relationship = format!(
                    r#" (<a href="{kb}/graha-relationships#compound">{relationship}</a>)</td></tr>"#
                );
                let lord = lord(&lord_name.to_lowercase(), lord_name);
                let cells = [format!(">{graha}<"), dignity, lord, relationship];
                let cells = cells.each_ref().map(String::as_str);
                assert!(
                    row(table(&page), &cells).is_some(),
                    "{graha}: {}",
                    table(&page)
                );
            }
            // The lagna, in Scorpio, and Rahu, in Cancer, have no dignity and only their rasi's lord.
            for (point, lord) in [
                ("Lagna", lord("mars", "Mars")),
                ("Rahu", lord("moon", moon)),
            ] {
                let cells = [
                    format!(">{point}<"),
                    "<td>—</td>".into(),
                    format!("{lord}</td></tr>"),
                ];
                let cells = cells.each_ref().map(String::as_str);
                assert!(
                    row(table(&page), &cells).is_some(),
                    "{point}: {}",
                    table(&page)
                );
            }
        }
        for (lang, source) in [
            ("en", include_str!("../kb/en/graha-relationships.md")),
            ("hu", include_str!("../kb/hu/graha-relationships.md")),
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
        let tromso = with(&[
            ("date", "2000-12-21"),
            ("time", "12%3A00"),
            ("offset", "%2B01%3A00"),
            ("latitude", "69.65"),
            ("longitude", "18.95"),
        ]);
        for (lang, query, upagrahas) in [
            ("en", HORAS_CHART, 11),
            ("hu", HORAS_CHART, 11),
            ("en", tromso.as_str(), 5),
        ] {
            let page = body(&format!("/{lang}/chart?{query}&style=south")).await;
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

    /// The upagraha page works its example on hora's chart: a Friday night, Saturn's part the 5th, the birth in the Sun's 7th.
    #[tokio::test]
    async fn the_upagraha_pages_example_is_horas_chart() {
        let page = body(&format!("/en/chart?{HORAS_CHART}&style=south")).await;
        let upagrahas = section(&page, "upagrahas");
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
        for (upagraha, rasi, bhava, position) in examples {
            let cells = [
                format!(">{upagraha}<"),
                format!(">{rasi}<"),
                format!(">{bhava}<"),
                format!(">{position}"),
            ];
            let cells = cells.each_ref().map(String::as_str);
            assert!(row(upagrahas, &cells).is_some(), "{upagraha}: {upagrahas}");
        }
        // From the Moon, in Libra, Upaketu in Scorpio is in the 2nd.
        let from_moon = HORAS_CHART.replace("reference=lagna", "reference=moon");
        let moon_page = body(&format!("/en/chart?{from_moon}&style=south")).await;
        let cells = [">Upaketu<", ">Scorpio<", ">2<"];
        assert!(row(section(&moon_page, "upagrahas"), &cells).is_some());
        assert!(row(table(&page), &[">Sun<", ">Sagittarius<", ">16°15′59″"]).is_some());
        let day = section(&page, "vedic-day");
        for time in ["15:59:58 (December 31, 1999)", "08:05:46<"] {
            assert!(day.contains(time), "{day}");
        }
        assert!(row(section(&page, "panchaanga"), &[">Friday<"]).is_some());
        for (lang, source) in [
            ("en", include_str!("../kb/en/upagraha.md")),
            ("hu", include_str!("../kb/hu/upagraha.md")),
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
    async fn a_retrograde_graha_shows_r_in_its_row() {
        // Chart 4 (ch. 9 *Arudha Padas*, p. 98), whose Jupiter hora-chart's golden test marks retrograde.
        let chart_4 = "date=1968-03-19&time=18%3A30&utc=stated&offset=%2B05%3A30\
            &latitude=17.116666666666667&longitude=82.31666666666666";
        let page = body(&format!("/en/chart?{chart_4}")).await;
        let table = table(&page);
        assert!(row(table, &[">Jupiter<", ">R<"]).is_some(), "{table}");
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
        for name in ["calendar", "utc", "ayanamsa", "node", "sunrise"] {
            let start = home.find(&format!(r#"<select id="{name}""#)).expect(name);
            let select = &home[start..start + home[start..].find("</select>").expect(name)];
            let values: Vec<_> = select
                .split(r#"<option value=""#)
                .skip(1)
                .filter_map(|option| option.split_once('"').map(|(value, _)| value))
                .collect();
            assert!(!values.is_empty(), "{select}");
            for value in values {
                // A zone needs a chosen place at its coordinates: Boston, near chart 1's.
                let query = match value {
                    "zone" => format!(
                        "{}&place=Boston&place-id=4930956",
                        with(&[
                            (name, value),
                            ("latitude", "42.35843"),
                            ("longitude", "-71.05977")
                        ])
                    ),
                    _ => with(&[(name, value)]),
                };
                let response = get(&format!("/en/chart?{query}"), None).await;
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
    fn each_choice_starts_on_its_adr_0003_default() {
        assert_eq!(CALENDARS[0].1, Calendar::default());
        assert_eq!(AYANAMSAS[0].1, Ayanamsa::default());
        assert_eq!(NODES[0].1, NodeType::default());
        assert_eq!(SUNRISES[0].1, SunriseDefinition::default());
    }

    #[tokio::test]
    async fn a_choice_left_out_is_its_default() {
        let home = body("/en/").await;
        for value in [
            "gregorian",
            "stated",
            "lahiri",
            "mean",
            "upper-limb-apparent",
            "lagna",
            "south",
        ] {
            assert!(
                home.contains(&format!(r#"value="{value}" selected"#)),
                "{value}"
            );
        }
        let blank = with(&[
            ("calendar", ""),
            ("utc", ""),
            ("ayanamsa", ""),
            ("node", ""),
            ("sunrise", ""),
            ("reference", ""),
            ("style", ""),
        ]);
        let (blank, chart_1) = (
            body(&format!("/en/chart?{blank}")).await,
            body(&format!("/en/chart?{CHART_1}")).await,
        );
        assert_eq!(table(&blank), table(&chart_1));
        assert_eq!(drawn(&blank), drawn(&chart_1));
        assert!(
            drawn(&blank).contains("M0 0H480V480H0ZM120 0V480"),
            "South Indian"
        );
        assert_eq!(section(&blank, "vedic-day"), section(&chart_1, "vedic-day"));
    }

    /// The drawn chart and its legend.
    fn drawn(page: &str) -> &str {
        let start = page.find("<figure>").expect(page);
        &page[start..start + page[start..].find("</figure>").unwrap()]
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
    async fn an_unknown_choice_is_refused() {
        for field in [
            "calendar",
            "utc",
            "ayanamsa",
            "node",
            "sunrise",
            "reference",
            "style",
        ] {
            assert_field_error(field, "fagan", "Choose one of the listed options.").await;
        }
    }

    #[tokio::test]
    async fn bhavas_are_counted_from_the_chosen_reference_point() {
        // Chart 1's lagna is in Virgo, its Moon in Gemini, its Sun in Pisces (ch. 9 *Arudha Padas*, p. 87).
        let from_lagna = body(&format!("/en/chart?{CHART_1}")).await;
        let from_moon = body(&format!("/en/chart?{}", with(&[("reference", "moon")]))).await;
        for (page, [lagna, moon, sun]) in [(&from_lagna, [1, 10, 7]), (&from_moon, [4, 1, 10])] {
            let table = table(page);
            for (graha, rasi, bhava) in [
                ("Lagna", "Virgo", lagna),
                ("Moon", "Gemini", moon),
                ("Sun", "Pisces", sun),
            ] {
                // Rasi and bhava in one cell pattern, so the bhava can't match the pada further on.
                let cells = [
                    format!(">{graha}<"),
                    format!(">{rasi}</a></td><td><a href=\"/en/kb/bhava\">{bhava}<"),
                ];
                let cells = cells.each_ref().map(String::as_str);
                assert!(row(table, &cells).is_some(), "{graha}: {table}");
            }
        }
        assert!(
            from_moon.contains(r#"value="moon" selected"#),
            "{from_moon}"
        );
    }

    #[tokio::test]
    async fn a_refused_form_keeps_the_chosen_sunrise_definition() {
        let query = with(&[("latitude", "95"), ("sunrise", "centre-true")]);
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let page = page(response).await;
        assert!(page.contains(r#"value="centre-true" selected"#), "{page}");
    }

    /// The section headed `id`, which follows the chart table.
    fn section<'a>(page: &'a str, id: &str) -> &'a str {
        let start = page
            .find(&format!(r#"<section aria-labelledby="{id}">"#))
            .expect(id);
        &page[start..start + page[start..].find("</section>").expect("a section end")]
    }

    #[tokio::test]
    async fn chart_1_shows_its_vedic_day_in_its_own_clock() {
        // Sunrise rounded up, the rest truncated; Jagannatha Hora's 19:20:09 sunset is 1–2 s later as its Sun has no aberration.
        for (lang, rows) in [
            ("en", ["Sunrise", "Sunset", "Next sunrise"]),
            ("hu", ["Napkelte", "Napnyugta", "Következő napkelte"]),
        ] {
            let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
            let day = section(&page, "vedic-day");
            assert!(
                day.contains(if lang == "en" {
                    ">Vedic day</h2>"
                } else {
                    ">Védikus nap</h2>"
                }),
                "{day}"
            );
            let next = if lang == "en" {
                "06:11:27 (April 10, 2000)"
            } else {
                "06:11:27 (2000. április 10.)"
            };
            for (name, time) in rows.into_iter().zip(["06:13:08", "19:20:07", next]) {
                let cells = [format!(">{name}<"), format!(">{time}<")];
                let cells = cells.each_ref().map(String::as_str);
                assert!(row(day, &cells).is_some(), "{name}: {day}");
            }
            assert!(!day.contains("<p>"), "{day}");
        }
    }

    #[tokio::test]
    async fn a_birth_before_sunrise_sees_the_previous_date_on_its_sunrise_and_sunset() {
        // Chart 3 (ch. 9 *Arudha Padas*, p. 97): 1926-12-25 05:12 at UTC+5:30, 78°E10′, 26°N14′; sunrise is 07:01.
        let chart_3 = "date=1926-12-25&time=05%3A12&utc=stated&offset=%2B05%3A30\
            &latitude=26.233333333333334&longitude=78.16666666666667";
        for (lang, rows, date) in [
            (
                "en",
                ["Sunrise", "Sunset", "Next sunrise"],
                " (December 24, 1926)<",
            ),
            (
                "hu",
                ["Napkelte", "Napnyugta", "Következő napkelte"],
                " (1926. december 24.)<",
            ),
        ] {
            let page = body(&format!("/{lang}/chart?{chart_3}")).await;
            let day = section(&page, "vedic-day");
            for name in &rows[..2] {
                assert!(
                    row(day, &[&format!(">{name}<"), date]).is_some(),
                    "{name}: {day}"
                );
            }
            let next = row(day, &[&format!(">{}<", rows[2])]).expect("a next sunrise");
            assert!(!next.contains('('), "{next}");
        }
    }

    #[tokio::test]
    async fn a_polar_night_birth_shows_its_chart_and_why_it_has_no_vedic_day() {
        // Tromsø, 69°39′N 18°57′E, at the winter solstice.
        let tromso = with(&[
            ("date", "2000-12-21"),
            ("time", "12%3A00"),
            ("offset", "%2B01%3A00"),
            ("latitude", "69.65"),
            ("longitude", "18.95"),
        ]);
        let en = body(&format!("/en/chart?{tromso}")).await;
        assert!(row(table(&en), &[">Lagna<"]).is_some(), "{en}");
        let day = section(&en, "vedic-day");
        assert!(
            day.contains("There is no Vedic day at this place on this date"),
            "{day}"
        );
        assert!(!day.contains("<table"), "{day}");
        let hu = body(&format!("/hu/chart?{tromso}")).await;
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
            let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
            let panchaanga = section(&page, "panchaanga");
            assert!(panchaanga.contains(heading), "{panchaanga}");
            for [name, value, lord] in rows {
                let cells = [
                    format!(">{name}<"),
                    format!(">{value}<"),
                    format!(">{lord}<"),
                ];
                let cells = cells.each_ref().map(String::as_str);
                assert!(row(panchaanga, &cells).is_some(), "{name}: {panchaanga}");
            }
            assert!(!panchaanga.contains("<p>"), "{panchaanga}");
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
            let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
            let spans = section(&page, "limb-spans");
            assert!(spans.contains(heading), "{spans}");
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
                let cells = [
                    format!(">{name}<"),
                    format!(">{start}<"),
                    format!(">{end}<"),
                ];
                let cells = cells.each_ref().map(String::as_str);
                assert!(row(spans, &cells).is_some(), "{name}: {spans}");
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
                let page = body(&format!("/en/chart?{query}")).await;
                let panchaanga = section(&page, "panchaanga");
                let cells = [format!(">{name}<"), format!(">{value}<")];
                let cells = cells.each_ref().map(String::as_str);
                assert!(
                    row(panchaanga, &cells).is_some(),
                    "{date} {time}: {panchaanga}"
                );
            }
        }
    }

    #[tokio::test]
    async fn a_birth_in_1999s_first_jyeshtha_is_in_adhika_jyeshtha() {
        // ch. 1 §1.3.8.2: new Moons in Taurus on 1999-05-15 and 06-14.
        let query = "date=1999-05-20&time=12%3A00&utc=stated&offset=%2B05%3A30&latitude=28.6&longitude=77.2";
        for (lang, name) in [("en", "Lunar month"), ("hu", "Holdhónap")] {
            let page = body(&format!("/{lang}/chart?{query}")).await;
            let panchaanga = section(&page, "panchaanga");
            let cells = [format!(">{name}<"), ">Adhika Jyeshtha<".into()];
            let cells = cells.each_ref().map(String::as_str);
            assert!(row(panchaanga, &cells).is_some(), "{panchaanga}");
        }
    }

    #[tokio::test]
    async fn a_birth_before_sunrise_has_the_previous_days_vaara() {
        // Chart 3 (ch. 9 *Arudha Padas*, p. 97): born on Saturday 1926-12-25 at 05:12, before the 07:01 sunrise.
        let chart_3 = "date=1926-12-25&time=05%3A12&utc=stated&offset=%2B05%3A30\
            &latitude=26.233333333333334&longitude=78.16666666666667";
        for (lang, vaara, lord) in [("en", "Friday", "Venus"), ("hu", "péntek", "Vénusz")] {
            let page = body(&format!("/{lang}/chart?{chart_3}")).await;
            let panchaanga = section(&page, "panchaanga");
            assert!(
                row(
                    panchaanga,
                    &[">Vaara<", &format!(">{vaara}<"), &format!(">{lord}<")]
                )
                .is_some(),
                "{panchaanga}"
            );
        }
    }

    #[tokio::test]
    async fn a_birth_at_chart_1s_shown_sunrise_has_its_sunday() {
        let page = body(&format!("/en/chart?{}", with(&[("time", "06%3A13%3A08")]))).await;
        let panchaanga = section(&page, "panchaanga");
        assert!(
            row(panchaanga, &[">Vaara<", ">Sunday<"]).is_some(),
            "{panchaanga}"
        );
    }

    #[tokio::test]
    async fn a_polar_night_birth_has_a_tithi_but_no_vaara_or_hora() {
        let tromso = with(&[
            ("date", "2000-12-21"),
            ("time", "12%3A00"),
            ("offset", "%2B01%3A00"),
            ("latitude", "69.65"),
            ("longitude", "18.95"),
        ]);
        for (lang, none) in [
            ("en", "There is no Vedic day here, so no vaara or hora."),
            ("hu", "Itt nincs védikus nap, így vaara és hora sincs."),
        ] {
            let page = body(&format!("/{lang}/chart?{tromso}")).await;
            let panchaanga = section(&page, "panchaanga");
            assert!(row(panchaanga, &[">Tithi<"]).is_some(), "{panchaanga}");
            assert!(row(panchaanga, &[">Karana<"]).is_some(), "{panchaanga}");
            assert!(row(panchaanga, &[">Vaara<"]).is_none(), "{panchaanga}");
            assert!(row(panchaanga, &[">Hora<"]).is_none(), "{panchaanga}");
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
            let page = body(&format!("/{lang}/chart?{tokyo_in_ut}")).await;
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
        let page = body(&format!("/en/chart?{kiritimati}")).await;
        let panchaanga = section(&page, "panchaanga");
        assert!(
            row(panchaanga, &[">Vaara<", ">Wednesday<"]).is_some(),
            "{panchaanga}"
        );
        assert!(!panchaanga.contains("<p>"), "{panchaanga}");
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
    fn clock_times_are_truncated_and_never_shown_as_24() {
        // JD 2451644.5 is 2000-04-09 0h; it holds about 40 µs, so 1 ms before midnight is representable.
        let at = |seconds: f64| 2_451_644.5 + seconds / 86400.0;
        assert_eq!(hms(at(6.0 * 3600.0 + 13.0 * 60.0 + 7.9)), "06:13:07");
        assert_eq!(hms(at(86400.0 - 1e-3)), "23:59:59");
        assert_eq!(hms(at(0.0)), "00:00:00");
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

    #[test]
    fn every_rasi_nakshatra_and_panchaanga_value_has_a_message() {
        use hora_chart::{KARANAS, LUNAR_MONTHS, NAKSHATRAS, RASIS, SUN_MOON_YOGAS, TITHIS};
        let en = keys(include_str!("../locales/en/main.ftl"));
        let vaaras = [
            Vaara::Sunday,
            Vaara::Monday,
            Vaara::Tuesday,
            Vaara::Wednesday,
            Vaara::Thursday,
            Vaara::Friday,
            Vaara::Saturday,
        ];
        let ids = RASIS
            .map(rasi_id)
            .into_iter()
            .chain(NAKSHATRAS.map(nakshatra_id))
            .chain(TITHIS.map(tithi_id))
            .chain([Paksha::Sukla, Paksha::Krishna].map(paksha_id))
            .chain(SUN_MOON_YOGAS.map(sun_moon_yoga_id))
            .chain(KARANAS.map(karana_id))
            .chain(vaaras.map(vaara_id))
            .chain(LUNAR_MONTHS.map(lunar_month_id))
            .chain([TwinMonth::Adhika, TwinMonth::Nija].map(twin_month_id));
        let missing: Vec<_> = ids.filter(|id| !en.contains(id)).collect();
        assert_eq!(missing, [""; 0]);
    }

    #[test]
    fn every_special_lagna_has_a_name_and_an_abbreviation() {
        let en = keys(include_str!("../locales/en/main.ftl"));
        let missing: Vec<_> = hora_chart::SPECIAL_LAGNAS
            .into_iter()
            .flat_map(|l| {
                ["special-lagna", "special-lagna-abbreviation"].map(|p| kb::message_id(p, l))
            })
            .filter(|id| !en.contains(id.as_str()))
            .collect();
        assert_eq!(missing, [""; 0]);
    }

    /// A birth at Budapest (geonameid 3054643) with its zone, on `date` at `time`.
    fn in_budapest(date: &str, time: &str) -> String {
        format!(
            "/en/chart?date={date}&time={time}&utc=zone&place=Budapest&place-id=3054643\
            &latitude=47.49835&longitude=19.04045"
        )
    }

    #[tokio::test]
    async fn a_zone_gives_its_modern_and_historical_offsets_and_names_them() {
        let page = body(&in_budapest("2000-07-01", "12%3A00")).await;
        assert!(
            page.contains("UTC offset +02:00, from the time zone Europe/Budapest of Budapest, HU."),
            "{page}"
        );
        assert!(!page.contains("Before 1970"), "{page}");
        // tzdb's wartime India: +06:30 from 1942-09 to 1945-10.
        let page = body(
            "/en/chart?date=1943-06-01&time=12%3A00&utc=zone&place=Kolkata&place-id=1275004\
            &latitude=22.56263&longitude=88.36304",
        )
        .await;
        assert!(
            page.contains("UTC offset +06:30, from the time zone Asia/Kolkata of Kolkata, IN."),
            "{page}"
        );
        assert!(
            page.contains("Before 1970 the time zone database"),
            "{page}"
        );
    }

    #[tokio::test]
    async fn a_julian_date_takes_the_zones_offset_on_its_gregorian_day() {
        // Julian 2000-06-18 is Gregorian 2000-07-01, in summer time.
        let page = body(&format!(
            "{}&calendar=julian",
            in_budapest("2000-06-18", "12%3A00")
        ))
        .await;
        assert!(page.contains("UTC offset +02:00"), "{page}");
    }

    #[tokio::test]
    async fn a_time_in_a_clock_change_is_refused_with_both_offsets_to_choose_from() {
        for (date, message, offsets) in [
            (
                "2000-03-26",
                "were put forward at this time",
                ["+01:00", "+02:00"],
            ),
            (
                "2000-10-29",
                "were put back at this time",
                ["+02:00", "+01:00"],
            ),
        ] {
            let response = get(&in_budapest(date, "02%3A30"), None).await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{date}");
            let page = page(response).await;
            assert!(
                page.contains(r#"<strong id="time-error" class="error">"#),
                "{page}"
            );
            assert!(page.contains(message), "{page}");
            for offset in offsets {
                assert!(
                    page.contains(&format!(r#"name="offset-pick" value="{offset}""#)),
                    "{date} {offset}: {page}"
                );
            }
            assert!(!page.contains("<table"), "{page}");
        }
        // Picking one makes it a stated offset: no zone is named.
        let page = body(&format!(
            "{}&offset-pick=%2B02%3A00",
            in_budapest("2000-03-26", "02%3A30")
        ))
        .await;
        assert!(page.contains("<table"), "{page}");
        assert!(
            page.contains(r#"<option value="stated" selected>"#),
            "{page}"
        );
        assert!(page.contains(r#"value="+02:00""#), "{page}");
        assert!(!page.contains("from the time zone"), "{page}");
        // West of Greenwich the buttons carry U+2212, which the offset field reads back.
        let new_york = "/en/chart?date=2000-10-29&time=01%3A30&utc=zone&place=New%20York%20City\
            &place-id=5128581&latitude=40.71427&longitude=-74.00597";
        let response = get(new_york, None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let page = self::page(response).await;
        for offset in ["−04:00", "−05:00"] {
            assert!(
                page.contains(&format!(r#"name="offset-pick" value="{offset}""#)),
                "{offset}: {page}"
            );
        }
        let page = body(&format!("{new_york}&offset-pick=%E2%88%9205%3A00")).await;
        assert!(page.contains("<table"), "{page}");
        assert!(page.contains(r#"value="−05:00""#), "{page}");
    }

    #[test]
    fn an_offset_is_written_with_its_seconds_only_when_it_has_some() {
        // Budapest's local mean time until 1890, and New York's standard time.
        assert_eq!(offset_text(4580), "+01:16:20");
        assert_eq!(offset_text(-18000), "−05:00");
        assert_eq!(offset(&offset_text(-4580)), Some(-4580));
    }

    #[tokio::test]
    async fn a_place_name_lists_the_matching_places_in_both_languages() {
        let page = body("/en/chart?date=2000-07-01&time=12%3A00&place=Kolozsv%C3%A1r").await;
        assert!(
            page.contains(
                r#"<button type="submit" name="pick" value="681290" formnovalidate>Kolozsvár (Cluj-Napoca), RO (Europe/Bucharest; 46.77, 23.60)</button>"#
            ),
            "{page}"
        );
        assert!(page.contains("Choose the birth place"), "{page}");
        assert!(!page.contains("<table"), "{page}");
        // A village cities500 lacks, from the full Hungarian file.
        let page = body("/hu/chart?date=2000-07-01&time=12%3A00&place=sopronhorp%C3%A1cs").await;
        assert!(page.contains(r#"value="3045188""#), "{page}");
        assert!(page.contains("Válassza ki a születési helyet"), "{page}");
        // A name with digits and no spelled-out alternate.
        let page = body("/en/chart?date=2000-07-01&time=12%3A00&place=28%20May").await;
        assert!(page.contains(r#"value="12238134""#), "{page}");
    }

    #[tokio::test]
    async fn picking_a_place_fills_its_coordinates_and_zone() {
        let page =
            body("/en/chart?date=2000-07-01&time=12%3A00&utc=stated&place=Buda&pick=3054643").await;
        assert!(page.contains("<table"), "{page}");
        for value in ["Budapest", "3054643", "47.49835", "19.04045"] {
            assert!(
                page.contains(&format!(r#"value="{value}""#)),
                "{value}: {page}"
            );
        }
        assert!(page.contains(r#"<option value="zone" selected>"#), "{page}");
        assert!(
            page.contains("from the time zone Europe/Budapest"),
            "{page}"
        );
        // Edited, the place's text is a new search.
        let page =
            body("/en/chart?date=2000-07-01&time=12%3A00&utc=zone&place=Vienna&place-id=3054643")
                .await;
        assert!(page.contains(r#"name="pick""#), "{page}");
        assert!(!page.contains("<table"), "{page}");
    }

    #[tokio::test]
    async fn the_script_finds_what_it_swaps_and_is_cached_by_its_etag() {
        // hora.js swaps in the list by these ids; without a place-id typing only searches.
        let list = body("/en/chart?date=2000-07-01&time=12%3A00&place=Budap").await;
        assert!(
            list.contains(r#"<div id="places" aria-live="polite">"#),
            "{list}"
        );
        assert!(list.contains(r#"name="pick""#), "{list}");
        assert!(!list.contains("<table"), "{list}");
        let none =
            page(get("/en/chart?date=2000-07-01&time=12%3A00&place=Budapxx", None).await).await;
        assert!(none.contains(r#"id="place-error""#), "{none}");
        assert!(
            none.contains(r#"<script type="module" src="/hora.js"></script>"#),
            "{none}"
        );

        let response = get("/hora.js", None).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/javascript; charset=utf-8"
        );
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
        let etag = response.headers()[header::ETAG]
            .to_str()
            .unwrap()
            .to_owned();
        for tags in [etag.clone(), format!("W/{etag}"), format!("\"x\", {etag}")] {
            let revalidated = app(places())
                .oneshot(
                    Request::get("/hora.js")
                        .header(header::IF_NONE_MATCH, &tags)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(revalidated.status(), StatusCode::NOT_MODIFIED, "{tags}");
            let body = to_bytes(revalidated.into_body(), 1 << 20).await.unwrap();
            assert!(body.is_empty());
        }
    }

    #[tokio::test]
    async fn a_bad_or_unknown_place_or_a_zone_without_one_is_refused() {
        let long = "a".repeat(101);
        for (query, field, message) in [
            ("place=%3Cscript%3E", "place", "Enter 2 to 100 letters"),
            ("place=a", "place", "Enter 2 to 100 letters"),
            (
                &format!("place={long}") as &str,
                "place",
                "Enter 2 to 100 letters",
            ),
            ("place=Xyzzyqx", "place", "No place of that name"),
            (
                "place=Buda&pick=1",
                "place",
                "Choose one of the listed options.",
            ),
            (
                "utc=zone&latitude=47.5&longitude=19",
                "utc",
                "To use a time zone",
            ),
        ] {
            let response = get(
                &format!("/en/chart?date=2000-07-01&time=12%3A00&{query}"),
                None,
            )
            .await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{query}");
            let page = page(response).await;
            let error = format!(r#"<strong id="{field}-error" class="error">{message}"#);
            assert!(page.contains(&error), "{query}: {page}");
        }
    }

    #[tokio::test]
    async fn the_form_credits_geonames_in_both_languages() {
        for lang in ["en", "hu"] {
            let page = body(&format!("/{lang}/")).await;
            assert!(page.contains(r#"<a href="https://www.geonames.org/">GeoNames</a> (<a href="https://creativecommons.org/licenses/by/4.0/">CC BY 4.0</a>)"#), "{page}");
        }
    }

    #[tokio::test]
    async fn typed_coordinates_replace_the_chosen_places_zone() {
        let edited = in_budapest("2000-07-01", "12%3A00").replace("47.49835", "47.6");
        let response = get(&edited, None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let page = page(response).await;
        assert!(
            page.contains(r#"<strong id="utc-error" class="error">To use a time zone"#),
            "{page}"
        );
        // Stated, the offset needs no zone, and the kept place text is no new search.
        let page = body(&edited.replace("utc=zone", "utc=stated&offset=%2B02%3A00")).await;
        assert!(page.contains("<table"), "{page}");
        assert!(!page.contains("from the time zone"), "{page}");
    }

    #[tokio::test]
    async fn a_zone_refusal_comes_with_the_other_fields_errors() {
        for (date, time, field) in [
            ("2000-03-26", "02%3A30", "time"),
            ("2000-02-30", "12%3A00", "date"),
        ] {
            let query = format!("{}&altitude=99999", in_budapest(date, time));
            let response = get(&query, None).await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{date}");
            let page = page(response).await;
            for field in [field, "altitude"] {
                assert!(
                    page.contains(&format!(r#"<strong id="{field}-error" class="error">"#)),
                    "{date} {field}: {page}"
                );
            }
        }
    }

    #[tokio::test]
    async fn the_before_1970_warning_goes_by_the_gregorian_date() {
        let warned = |page: &str| page.contains("Before 1970 the time zone database");
        assert!(warned(&body(&in_budapest("1969-12-31", "12%3A00")).await));
        // Julian 1969-12-25 is Gregorian 1970-01-07.
        let page = body(&format!(
            "{}&calendar=julian",
            in_budapest("1969-12-25", "12%3A00")
        ))
        .await;
        assert!(
            page.contains("from the time zone Europe/Budapest"),
            "{page}"
        );
        assert!(!warned(&page), "{page}");
    }
}

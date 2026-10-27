//! Routes and pages; every page is served in English under `/en/` and in Hungarian under `/hu/`.

use std::collections::{BTreeMap, HashMap};

use askama::Template;
use axum::Router;
use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader, langid, static_loader};
use hora_swe::{Ayanamsa, Calendar, NodeType, Request, Rule, Sky, SunriseDefinition, UtcOffset};

use hora_chart::{
    Body, Karana, LunarMonth, Nakshatra, Paksha, Rasi, SunMoonYoga, Tithi, TwinMonth, Vaara,
};

mod chakra;
mod kb;

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
    /// With a chart: the ayanamsa its positions are measured with; empty without one.
    ayanamsa: String,
    /// With a chart: (message id, clock time) for its Vedic day's sunrise, sunset and next sunrise; `None` where there is none (polar latitudes).
    vedic_day: Option<[(&'static str, String); 3]>,
    /// With a chart: its panchaanga at the birth; vaara and hora only with a Vedic day.
    panchaanga: Vec<Limb>,
    /// With a chart: when its tithi, nakshatra, yoga and karana started and end.
    limb_spans: Vec<LimbSpan>,
    /// With a chart: it drawn in the chosen style.
    drawing: chakra::Drawing,
    /// With a chart: whether its stated offset is far from the place's local time, so its vaara and hora may be a day off.
    far_offset: bool,
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
            ayanamsa: String::new(),
            vedic_day: None,
            panchaanga: Vec::new(),
            limb_spans: Vec::new(),
            drawing: chakra::Drawing::default(),
            far_offset: false,
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
    render(StatusCode::OK, &uri, &Home::empty(lang, &uri))
}

async fn chart_page(lang: Lang, uri: Uri, Query(form): Query<HashMap<String, String>>) -> Response {
    let mut page = Home::empty(lang, &uri);
    let status = match parse(&form) {
        Err(errors) => {
            page.errors = errors;
            StatusCode::BAD_REQUEST
        }
        Ok((request, reference, style)) => match hora_swe::calculate(request.clone()).await {
            Ok(sky) => {
                page.chart = rows(&sky, reference);
                page.ayanamsa = dms(sky.ayanamsa);
                page.vedic_day = sky.vedic_day.map(|day| vedic_day_rows(lang, &request, day));
                page.panchaanga = panchaanga(lang, &sky);
                page.limb_spans = limb_spans(lang, &request, &sky.limbs);
                page.drawing = chakra::drawing(lang, style, &hora_chart::chart(&sky), reference);
                page.far_offset = far_offset(&request);
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

/// The form as a request, the bhavas' reference point and the chart style, or each unreadable field's message id; whether the values are in range is `hora_swe`'s to say.
fn parse(
    form: &HashMap<String, String>,
) -> Result<(Request, Body, chakra::Style), BTreeMap<&'static str, &'static str>> {
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
    let sunrise_definition = choice(SUNRISES, value("sunrise"));
    let reference = choice(REFERENCES, value("reference"));
    let style = choice(STYLES, value("style"));
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
        return Err(errors);
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
    Ok((request, reference, style))
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
}

/// The table rows of the chart of `sky`, in the chart's order, with bhavas counted from `reference`.
fn rows(sky: &Sky, reference: Body) -> Vec<Row> {
    let chart = hora_chart::chart(sky);
    let reference_rasi = hora_chart::rasi_of(&chart, reference);
    chart
        .iter()
        .map(|entry| {
            let (rasi, degrees) = hora_chart::rasi(entry.longitude);
            let (nakshatra, pada) = hora_chart::nakshatra(entry.longitude);
            Row {
                name: body_id(entry.body),
                rasi: rasi_id(rasi),
                bhava: hora_chart::bhava(reference_rasi, rasi),
                position: dms(degrees),
                retrograde: entry.retrograde,
                nakshatra: nakshatra_id(nakshatra),
                nakshatra_number: nakshatra as u8 + 1,
                pada,
                lord: body_id(hora_chart::vimsottari_lord(nakshatra)),
            }
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

/// When the birth's tithi, nakshatra, yoga and karana started and end (ch. 1 §1.3.12).
fn limb_spans(lang: Lang, birth: &Request, limbs: &hora_swe::Limbs) -> Vec<LimbSpan> {
    ["tithi", "nakshatra", "yoga", "karana"]
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
            name: "yoga",
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
        let first = r#"<li><a href="/en/kb/jyotish">Jyotish</a> (chapter 1, § 1.1–1.2)</li>"#;
        let graha = r#"<li><a href="/en/kb/graha">Graha</a> (chapter 1, § 1.3.1)</li>"#;
        let rasi = r#"<li><a href="/en/kb/rasi">Rasi</a> (chapter 1, § 1.3.2)</li>"#;
        assert!(
            en.contains(first) && en.find(first) < en.find(graha),
            "{en}"
        );
        assert!(en.find(graha) < en.find(rasi), "{en}");
        let hu = body("/hu/kb/").await;
        let first = r#"<li><a href="/hu/kb/jyotish">Jyotish</a> (1. fejezet, § 1.1–1.2)</li>"#;
        let graha = r#"<li><a href="/hu/kb/graha">Graha</a> (1. fejezet, § 1.3.1)</li>"#;
        let rasi = r#"<li><a href="/hu/kb/rasi">Rasi</a> (1. fejezet, § 1.3.2)</li>"#;
        assert!(
            hu.contains(first) && hu.find(first) < hu.find(graha),
            "{hu}"
        );
        assert!(hu.find(graha) < hu.find(rasi), "{hu}");
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
            assert!(page.contains("<p>Ayanamsa: 23°51′39″</p>"), "{page}");
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
            let link = |name| format!(r#"<a href="/{lang}/kb/graha">{name}</a>"#);
            let table = table(&page);
            assert!(
                table.contains(&format!(r#"<th scope="row">{}</th>"#, link(sun))),
                "{table}"
            );
            assert!(table.contains(r#"<th scope="row">Lagna</th>"#), "{table}");
            assert!(
                table.contains(&format!("<td>{}</td></tr>", link(lagna_nakshatra_lord))),
                "{table}"
            );
            // Chart 1's tithi, vaara, nakshatra and hora each have a lord.
            let panchaanga = section(&page, "panchaanga");
            let href = format!(r#"<td><a href="/{lang}/kb/graha">"#);
            assert_eq!(panchaanga.matches(&href).count(), 4, "{panchaanga}");
        }
    }

    #[tokio::test]
    async fn every_rasi_the_chart_page_names_links_to_its_kb_page() {
        for (lang, virgo) in [("en", "Virgo"), ("hu", "Szűz")] {
            let page = body(&format!("/{lang}/chart?{CHART_1}")).await;
            let table = table(&page);
            let href = format!(r#"<td><a href="/{lang}/kb/rasi">"#);
            // The lagna and the nine grahas.
            assert_eq!(table.matches(&href).count(), 10, "{table}");
            assert!(
                table.contains(&format!("{href}{virgo}</a></td>")),
                "{table}"
            );
        }
    }

    /// The rasi page's worked example is hora's own Sun at noon UT, 1 January 2000, at Greenwich.
    #[tokio::test]
    async fn the_rasi_pages_example_is_horas_chart() {
        let query = "date=2000-01-01&time=12%3A00&calendar=gregorian&utc=stated&offset=%2B00%3A00\
            &latitude=51.48&longitude=0&altitude=&ayanamsa=lahiri&node=mean&sunrise=upper-limb-apparent&reference=lagna&style=south";
        let page = body(&format!("/en/chart?{query}")).await;
        assert!(
            row(
                table(&page),
                &[">Sun<", ">Sagittarius<", "<td>16°31′17″</td>"]
            )
            .is_some(),
            "{page}"
        );
        assert!(page.contains("Ayanamsa: 23°51′25″"), "{page}");
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
                    format!(">{rasi}</a></td><td>{bhava}<"),
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
                ">Panchaanga</h2>",
                [
                    ["Tithi", "Sukla Shashti (6)", "Venus"],
                    ["Vaara", "Sunday", "Sun"],
                    ["Nakshatra", "Mrigasira (5)", "Mars"],
                    ["Yoga", "Sobhana (5)", ""],
                    ["Karana", "Taitula", ""],
                    ["Hora", "12", "Saturn"],
                    ["Lunar month", "Chaitra", ""],
                ],
            ),
            (
                "hu",
                ">Panchaanga</h2>",
                [
                    ["Tithi", "Sukla Shashti (6)", "Vénusz"],
                    ["Vaara", "vasárnap", "Nap"],
                    ["Nakshatra", "Mrigasira (5)", "Mars"],
                    ["Yoga", "Sobhana (5)", ""],
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
        for (lang, heading, [april_8, april_10]) in [
            (
                "en",
                ">Start and end of the limbs</h2>",
                [" (April 8, 2000)", " (April 10, 2000)"],
            ),
            (
                "hu",
                ">Az elemek kezdete és vége</h2>",
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
            // The limbs' names are the same in both languages.
            for (name, (start, end)) in ["Tithi", "Nakshatra", "Yoga", "Karana"]
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
                "Yoga",
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
}

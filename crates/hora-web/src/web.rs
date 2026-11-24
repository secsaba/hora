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

mod panchaanga;
use panchaanga::*;

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

/// The offset is stated, so the hour a clock change repeats is not refused as ambiguous.
fn moment(lang: Lang, site: &Site, uri: &Uri) -> Option<Response> {
    // Any parameter, even an unrelated one, keeps the page as it was.
    if uri.query().is_some_and(|query| !query.is_empty()) {
        return None;
    }
    let place = site.places.get(site.default_place?)?;
    let now = (site.now)().to_zoned(site.places.zone(place).1.clone());
    // offset_text's minus is U+2212, not allowed in a header; a bare + in a query is a space.
    let offset = offset_text(now.offset().seconds())
        .replace('+', "%2B")
        .replace('−', "-");
    let location = format!(
        "/{}/chart?date={}&time={:02}:{:02}&pick={}&offset-pick={offset}",
        lang.code(),
        now.date(),
        now.hour(),
        now.minute(),
        place.id,
    );
    Some(Redirect::to(&location).into_response())
}

/// What the chart page shows of a calculated chart.
struct ChartView {
    /// The grahas table's rows: the lagna and the grahas, in graha or chara karaka order.
    grahas: Vec<Row>,
    /// Its bhava arudhas' rows, AL to UL (ch. 9 §9.2).
    bhava_arudhas: Vec<ArudhaRow>,
    /// Its graha arudhas' rows, the Sun to Ketu (ch. 9 §9.5).
    graha_arudhas: Vec<ArudhaRow>,
    /// The rasi chart.
    rasi: ChakraView,
    /// The varga chakra shown.
    varga: ChakraView,
    /// The varga chakra shown, by name, heading the arudha tables' columns for it.
    varga_name: String,
    /// The n of the varga chakra shown (ADR 0003).
    parts: u8,
    /// Its upagrahas' rows, each with its section on the upagraha page.
    upagrahas: Vec<(&'static str, Row)>,
    /// Its special lagnas' rows, HL, GL, SL and BL, or SL alone without a Vedic day (ch. 5).
    special_lagnas: Vec<Row>,
    /// The ayanamsa its positions are measured with.
    ayanamsa: String,
    /// (message id, clock time) for its Vedic day's sunrise, sunset and next sunrise; `None` where there is none (polar latitudes).
    vedic_day: Option<[(&'static str, String); 3]>,
    /// Its panchaanga at the birth; vaara and hora only with a Vedic day.
    panchaanga: Vec<Limb>,
    /// When its tithi, nakshatra, Sun-Moon yoga and karana started and end.
    limb_spans: Vec<LimbSpan>,
    /// Whether the grahas table is in chara karaka order, else graha order.
    by_karaka: bool,
    /// The chart sorted by graha and by chara karaka, at the grahas table.
    sort_hrefs: [String; 2],
    /// The Vargas table's columns, (name, short) message ids: the lagna, the grahas, then HL and GL with a Vedic day, and SL.
    varga_columns: Vec<(String, String)>,
    /// Each varga's row of the Vargas table and link in the link row, D-1 to D-60.
    vargas: Vec<VargaRow>,
    /// Each graha's (message id, (count, amsa message id) in each varga group) (ch. 6 §6.6).
    amsabala: Vec<(&'static str, [AmsabalaCell; 4])>,
    /// Whether its stated offset is far from the place's local time, so its vaara and hora may be a day off.
    far_offset: bool,
    /// Rudra, Maheswara and both longevity methods, in the rasi chart only (ch. 14, ADR 0017).
    longevity: LongevityView,
    /// The avasthas, stronger lords and sthira karakas, in the rasi chart only (ch. 15, ch. 8 §8.3, ADR 0018).
    strength: StrengthView,
    /// The topic shown.
    topic: &'static Topic,
    /// The contents: every topic, in `TOPICS` order.
    contents: Vec<TopicLink>,
    /// The topics before and after the one shown, as (label message id, href).
    neighbours: [Option<(&'static str, String)>; 2],
}

/// A topic's entry in the contents.
struct TopicLink {
    label: &'static str,
    chapters: String,
    href: String,
    /// What it holds at a glance, or empty.
    fact: String,
    current: bool,
}

/// What one varga chakra shows, bhavas counted from the reference point's rasi in it.
struct ChakraView {
    /// What each graha aspects, the Sun to Ketu (ch. 10 §10.2–10.3).
    aspects: Vec<AspectRow>,
    /// The argalas on each rasi (ch. 10 §10.5–10.6).
    argalas: Vec<ArgalaRow>,
    /// The yogas by class, counted from its lagna (ch. 11).
    yogas: Vec<YogaGroup>,
    /// Its ashtakavarga, counted from its lagna (ch. 12).
    ashtakavarga: Ashtakavarga,
    /// Each bhava's baadhakas, from the bhava the reference point is in (ch. 13 §13.3).
    baadhakas: Vec<BaadhakaRow>,
    /// Its yogadas, from its lagna, HL and GL (ch. 13 §13.2).
    yogadas: Vec<YogadaRow>,
    /// Its maraka sthanas, the 2nd and 7th from its lagna, each with its bhava (ch. 14 §14.2).
    maraka_sthanas: [(&'static str, u8); 2],
    /// Its maraka grahas, each with its reasons in words (ch. 14 §14.2).
    marakas: Vec<(&'static str, Vec<String>)>,
    /// It drawn in the chosen style.
    drawing: chakra::Drawing,
}

impl ChakraView {
    /// `varga`'s chakra, or with `None` the rasi chart, drawn with each body's degrees.
    fn new(
        lang: Lang,
        chart: &hora_chart::Chart,
        varga: Option<Varga>,
        reference: Rasi,
        style: chakra::Style,
    ) -> Self {
        let d = varga.unwrap_or(Varga::RasiChart);
        ChakraView {
            aspects: aspect_rows(d, chart, reference),
            argalas: argala_rows(d, chart, reference),
            yogas: yoga_groups(lang, d, chart),
            ashtakavarga: Ashtakavarga::new(lang, d, chart),
            baadhakas: baadhaka_rows(d, chart, reference),
            yogadas: yogada_rows(d, chart),
            maraka_sthanas: {
                let [second, seventh] = chart.in_varga(d).maraka_sthanas();
                [(rasi_id(second), 2), (rasi_id(seventh), 7)]
            },
            marakas: maraka_rows(lang, d, chart),
            drawing: chakra::drawing(lang, style, chart, reference, varga),
        }
    }
}

/// How the form asks for a chart to be shown, apart from the birth data `hora_swe` calculates with.
#[derive(Clone, Copy)]
struct Shown {
    /// The name sound, 1–5 (Table 37), if given.
    sound: Option<u8>,
    /// The bhavas' reference point.
    reference: ReferencePoint,
    style: chakra::Style,
    /// The varga chakra shown beside the rasi chart.
    varga: Varga,
    /// Whether the grahas table is in chara karaka order.
    by_karaka: bool,
    /// The index in `TOPICS` of the topic shown.
    topic: usize,
}

impl ChartView {
    /// What the chart page shows of `sky`, calculated for `request` and reached at `uri`; or the message id of a reference
    /// point `sky` does not have.
    fn new(
        lang: Lang,
        uri: &Uri,
        request: &Request,
        sky: &Sky,
        Shown {
            sound,
            reference,
            style,
            varga,
            by_karaka,
            topic,
        }: Shown,
    ) -> Result<Self, &'static str> {
        let chart = hora_chart::chart(sky);
        let karakas = chart.chara_karakas();
        let rasi_arudhas = chart.in_varga(Varga::RasiChart).bhava_arudhas();
        let varga_arudhas = chart.in_varga(varga).bhava_arudhas();
        // North Indian counts from the reference point's rasi in the varga (ch. 6 §6.2).
        let (Some(reference_rasi), Some(varga_reference)) = (
            chart.in_varga(Varga::RasiChart).reference(reference),
            chart.in_varga(varga).reference(reference),
        ) else {
            // HL and GL need a Vedic day; the form offers them only where there is one.
            return Err("special-lagnas-no-vedic-day");
        };
        let mut grahas = rows(&chart, &karakas, reference_rasi, reference, varga);
        if by_karaka {
            // The lagna first, AK to DK (ties stay in graha order), then Ketu, who holds none.
            grahas.sort_by_key(|row| match (row.name.as_str(), row.karaka) {
                ("lagna", _) => 0,
                (_, Some(karaka)) => 1 + karaka as u8,
                (_, None) => 9,
            });
        }
        let arudha_row = |id, rasi, varga_rasi| ArudhaRow {
            id,
            rasi: rasi_id(rasi),
            bhava: hora_chart::bhava(reference_rasi, rasi),
            varga_rasi: rasi_id(varga_rasi),
            varga_bhava: hora_chart::bhava(varga_reference, varga_rasi),
        };
        // BL is in no chakra (ch. 5).
        let points: Vec<_> = chart
            .entries()
            .iter()
            .map(|e| {
                (
                    body_id(e.body).to_owned(),
                    format!("{}-short", body_id(e.body)),
                    e.longitude,
                )
            })
            .chain(
                chart
                    .special_lagnas()
                    .iter()
                    .filter(|s| s.0 != SpecialLagna::BhaavaLagna)
                    .map(|&(lagna, longitude)| {
                        let id = kb::message_id("special-lagna", lagna);
                        let short = kb::message_id("special-lagna-abbreviation", lagna);
                        (id, short, longitude)
                    }),
            )
            .collect();
        let mut view = ChartView {
            grahas,
            bhava_arudhas: (0..12)
                .map(|n| arudha_row(arudha_id(n), rasi_arudhas[n], varga_arudhas[n]))
                .collect(),
            graha_arudhas: chart
                .in_varga(Varga::RasiChart)
                .graha_arudhas()
                .into_iter()
                .zip(chart.in_varga(varga).graha_arudhas())
                .map(|((graha, rasi), (_, varga_rasi))| {
                    arudha_row(format!("arudha-{}", body_id(graha)), rasi, varga_rasi)
                })
                .collect(),
            rasi: ChakraView::new(lang, &chart, None, reference_rasi, style),
            varga: ChakraView::new(lang, &chart, Some(varga), varga_reference, style),
            varga_name: varga_name(lang, varga),
            parts: varga.parts(),
            upagrahas: upagraha_rows(sky, reference_rasi),
            special_lagnas: chart
                .special_lagnas()
                .iter()
                .map(|&(lagna, longitude)| {
                    point_row(
                        kb::message_id("special-lagna", lagna),
                        longitude,
                        reference_rasi,
                    )
                })
                .collect(),
            ayanamsa: dms(sky.ayanamsa),
            vedic_day: sky.vedic_day.map(|day| vedic_day_rows(lang, request, day)),
            panchaanga: panchaanga(lang, sky),
            limb_spans: limb_spans(lang, request, &sky.limbs),
            by_karaka,
            sort_hrefs: ["graha", "karaka"].map(|sort| query_href(uri, "sort", sort, "grahas")),
            vargas: hora_chart::VARGAS
                .map(|varga| VargaRow {
                    parts: varga.parts(),
                    name: varga_name(lang, varga),
                    href: query_href(uri, "varga", &format!("d{}", varga.parts()), ""),
                    rasis: points
                        .iter()
                        .map(|p| rasi_id(hora_chart::varga(varga, p.2)))
                        .collect(),
                })
                .into(),
            varga_columns: points
                .into_iter()
                .map(|(id, short, _)| (id, short))
                .collect(),
            amsabala: chart
                .grahas()
                .iter()
                .map(|e| {
                    let groups = hora_chart::VARGA_GROUPS.map(|group| {
                        let (count, amsa) = hora_chart::amsabala(e.body, e.longitude, group);
                        (count, amsa.map(|a| kb::message_id("amsa", a)))
                    });
                    (body_id(e.body), groups)
                })
                .collect(),
            far_offset: far_offset(request),
            longevity: LongevityView::new(lang, &chart),
            strength: StrengthView::new(&chart, sky, sound),
            topic: &TOPICS[topic],
            contents: Vec::new(),
            neighbours: [topic.checked_sub(1), Some(topic + 1)].map(|i| {
                let t = TOPICS.get(i?)?;
                Some((t.label, query_href(uri, "topic", t.key, "")))
            }),
        };
        view.contents = TOPICS
            .iter()
            .enumerate()
            .map(|(i, t)| TopicLink {
                label: t.label,
                chapters: message(
                    lang,
                    "topic-chapters",
                    [("chapters", t.chapters.to_owned())],
                ),
                href: query_href(uri, "topic", t.key, ""),
                fact: view.fact(lang, t.key),
                current: i == topic,
            })
            .collect();
        Ok(view)
    }

    /// The key fact of the topic `key`, empty where it has none or the chart lacks it (no Vedic day, no vaara or HL).
    fn fact(&self, lang: Lang, key: &str) -> String {
        let t = |id: &str| LOCALES.lookup(&lang.id(), id);
        let named = |short: &str, rasi: Option<&str>| {
            rasi.map_or(String::new(), |rasi| format!("{} {}", t(short), t(rasi)))
        };
        match key {
            "grahas" => named(
                "lagna",
                self.grahas
                    .iter()
                    .find(|r| r.name == "lagna")
                    .map(|r| r.rasi),
            ),
            "day" => self
                .panchaanga
                .iter()
                .find(|l| l.name == "vaara")
                .map_or(String::new(), |l| l.value.clone()),
            "lagnas" => named(
                "special-lagna-abbreviation-hora-lagna",
                self.special_lagnas
                    .iter()
                    .find(|r| r.name == "special-lagna-hora-lagna")
                    .map(|r| r.rasi),
            ),
            "vargas" => self.varga_name.clone(),
            "arudhas" => named(
                "bhava-arudha-1-short",
                self.bhava_arudhas.first().map(|r| r.rasi),
            ),
            "yogas" => {
                let count: usize = self.rasi.yogas.iter().map(|g| g.rows.len()).sum();
                message(lang, "yogas-found", [("count", count.to_string())])
            }
            _ => String::new(),
        }
    }
}

async fn chart_page(
    lang: Lang,
    State(site): State<Site>,
    uri: Uri,
    Query(mut form): Query<HashMap<String, String>>,
) -> Response {
    if let Some(redirect) = moment(lang, &site, &uri) {
        return redirect;
    }
    let places = site.places;
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
            if let Some(probe) = refusal.probe
                && let Err(rules) = hora_swe::validate(&probe)
            {
                for (field, message) in rules.into_iter().map(field_error) {
                    page.errors.entry(field).or_insert(message);
                }
            }
            StatusCode::BAD_REQUEST
        }
        Ok((request, shown)) => {
            match hora_swe::calculate(request.clone()).await {
                Ok(sky) => match ChartView::new(lang, &uri, &request, &sky, shown) {
                    Err(id) => {
                        page.errors.insert("reference", id);
                        StatusCode::BAD_REQUEST
                    }
                    Ok(view) => {
                        page.chart = Some(view);
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
                            page.before_1970 =
                                hora_swe::civil_date(local, Calendar::Gregorian)[0] < 1970;
                        }
                        StatusCode::OK
                    }
                },
                Err(hora_swe::Error::InvalidInput(rules)) => {
                    page.errors = rules.into_iter().map(field_error).collect();
                    StatusCode::BAD_REQUEST
                }
                Err(error) => return failure(lang, &uri, error),
            }
        }
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

/// The lagna, the grahas, the special lagnas but BL (ch. 5, footnote 10), paaka lagna (ch. 7 §7.3.5), the atma
/// karaka (ch. 7 §7.3.6, ch. 8 §8.2), the arudha and upapada lagnas (ch. 7 §7.3.4, ch. 9 §9.3–9.4), then the relatives'
/// lagnas, each labelled with the varga the book reads it in (ch. 13 §13.4.2, ADR 0016).
const REFERENCES: &Choices<ReferencePoint> = &[
    (
        "lagna",
        ReferencePoint::Body(Body::Lagna),
        body_id(Body::Lagna),
    ),
    ("sun", ReferencePoint::Body(Body::Sun), body_id(Body::Sun)),
    (
        "moon",
        ReferencePoint::Body(Body::Moon),
        body_id(Body::Moon),
    ),
    (
        "mars",
        ReferencePoint::Body(Body::Mars),
        body_id(Body::Mars),
    ),
    (
        "mercury",
        ReferencePoint::Body(Body::Mercury),
        body_id(Body::Mercury),
    ),
    (
        "jupiter",
        ReferencePoint::Body(Body::Jupiter),
        body_id(Body::Jupiter),
    ),
    (
        "venus",
        ReferencePoint::Body(Body::Venus),
        body_id(Body::Venus),
    ),
    (
        "saturn",
        ReferencePoint::Body(Body::Saturn),
        body_id(Body::Saturn),
    ),
    (
        "rahu",
        ReferencePoint::Body(Body::Rahu),
        body_id(Body::Rahu),
    ),
    (
        "ketu",
        ReferencePoint::Body(Body::Ketu),
        body_id(Body::Ketu),
    ),
    (
        "hora-lagna",
        ReferencePoint::Special(SpecialLagna::HoraLagna),
        "special-lagna-hora-lagna",
    ),
    (
        "ghati-lagna",
        ReferencePoint::Special(SpecialLagna::GhatiLagna),
        "special-lagna-ghati-lagna",
    ),
    (
        "sree-lagna",
        ReferencePoint::Special(SpecialLagna::SreeLagna),
        "special-lagna-sree-lagna",
    ),
    (
        "paaka-lagna",
        ReferencePoint::PaakaLagna,
        "reference-paaka-lagna",
    ),
    (
        "atma-karaka",
        ReferencePoint::AtmaKaraka,
        "chara-karaka-atma",
    ),
    (
        "arudha-lagna",
        ReferencePoint::Arudha(Bhava::new(1).unwrap()),
        "bhava-arudha-1",
    ),
    (
        "upapada-lagna",
        ReferencePoint::Arudha(Bhava::new(12).unwrap()),
        "bhava-arudha-12",
    ),
    (
        "father",
        ReferencePoint::Relative(Relative::Father),
        "reference-father",
    ),
    (
        "mother",
        ReferencePoint::Relative(Relative::Mother),
        "reference-mother",
    ),
    (
        "spouse",
        ReferencePoint::Relative(Relative::Spouse),
        "reference-spouse",
    ),
    (
        "younger-sibling-1",
        ReferencePoint::Relative(Relative::YoungerSibling(Nth::First)),
        "reference-younger-sibling-1",
    ),
    (
        "younger-sibling-2",
        ReferencePoint::Relative(Relative::YoungerSibling(Nth::Second)),
        "reference-younger-sibling-2",
    ),
    (
        "younger-sibling-3",
        ReferencePoint::Relative(Relative::YoungerSibling(Nth::Third)),
        "reference-younger-sibling-3",
    ),
    (
        "elder-sibling-1",
        ReferencePoint::Relative(Relative::ElderSibling(Nth::First)),
        "reference-elder-sibling-1",
    ),
    (
        "elder-sibling-2",
        ReferencePoint::Relative(Relative::ElderSibling(Nth::Second)),
        "reference-elder-sibling-2",
    ),
    (
        "elder-sibling-3",
        ReferencePoint::Relative(Relative::ElderSibling(Nth::Third)),
        "reference-elder-sibling-3",
    ),
    (
        "child-1",
        ReferencePoint::Relative(Relative::Child(Nth::First)),
        "reference-child-1",
    ),
    (
        "child-2",
        ReferencePoint::Relative(Relative::Child(Nth::Second)),
        "reference-child-2",
    ),
    (
        "child-3",
        ReferencePoint::Relative(Relative::Child(Nth::Third)),
        "reference-child-3",
    ),
];

/// The name sound's five groups of Table 37, as the book transliterates them, after "not given" (ch. 15 §15.4.4, p. 193, ADR 0018).
const SOUNDS: &Choices<Option<u8>> = &[
    ("", None, "sound-none"),
    ("1", Some(1), "sound-1"),
    ("2", Some(2), "sound-2"),
    ("3", Some(3), "sound-3"),
    ("4", Some(4), "sound-4"),
    ("5", Some(5), "sound-5"),
];

/// The grahas table's order: `true` is by chara karaka.
const SORTS: &Choices<bool> = &[("graha", false, "graha"), ("karaka", true, "chara-karaka")];

/// A topic of the chart page, shown under the chakras.
struct Topic {
    /// The `topic` sent.
    key: &'static str,
    /// Its label's message id.
    label: &'static str,
    /// The book's chapters it covers.
    chapters: &'static str,
}

/// The chart page's topics in the book's order; the first is shown when none is asked for.
const TOPICS: [Topic; 11] = {
    const fn topic(key: &'static str, label: &'static str, chapters: &'static str) -> Topic {
        Topic {
            key,
            label,
            chapters,
        }
    }
    [
        topic("grahas", "topic-grahas", "1–3, 7–8"),
        topic("day", "topic-day", "1"),
        topic("lagnas", "topic-lagnas", "4–5"),
        topic("vargas", "topic-vargas", "6"),
        topic("arudhas", "topic-arudhas", "9"),
        topic("aspects", "topic-aspects", "10"),
        topic("yogas", "topic-yogas", "11"),
        topic("ashtakavarga", "topic-ashtakavarga", "12"),
        topic("interpreting", "topic-interpreting-charts", "13"),
        topic("longevity", "topic-longevity", "14"),
        topic("strength", "topic-strength", "15"),
    ]
};

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
    /// When the zone refused the date or time: the request with a stand-in offset, for `hora_swe::validate` to name the other fields' broken rules.
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

/// The form as a request and how its chart is shown, or each unreadable field's message id; whether the values are in range is `hora_swe`'s to say.
/// With `utc=zone`, the offset is `zone`'s at the birth time (ADR 0005).
fn parse(
    form: &HashMap<String, String>,
    zone: Option<&TimeZone>,
) -> Result<(Request, Shown), Refusal> {
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
    let by_karaka = choice(SORTS, value("sort"));
    let sound = choice(SOUNDS, value("sound"));
    // Not a `Choices`: its default, navamsa (ADR 0003), is not its first.
    let varga = match value("varga") {
        "" => Some(Varga::Navamsa),
        // D-1 is the rasi chart, already drawn beside it.
        value => hora_chart::VARGAS
            .into_iter()
            .filter(|v| *v != Varga::RasiChart)
            .find(|v| format!("d{}", v.parts()) == value),
    };
    let topic = match value("topic") {
        "" => Some(0),
        value => TOPICS.iter().position(|t| t.key == value),
    };
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
        varga.is_none().then_some(("varga", "error-choice")),
        by_karaka.is_none().then_some(("sort", "error-choice")),
        sound.is_none().then_some(("sound", "error-choice")),
        topic.is_none().then_some(("topic", "error-choice")),
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
        Some(varga),
        Some(by_karaka),
        Some(sound),
        Some(topic),
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
        varga,
        by_karaka,
        sound,
        topic,
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
        None => Ok((
            request,
            Shown {
                sound,
                reference,
                style,
                varga,
                by_karaka,
                topic,
            },
        )),
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
    let jd = hora_swe::julian_day(year, month, day, hours, calendar)
        .map_err(|rule| Refusal::from(BTreeMap::from([field_error(rule)])))?;
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
    /// The graha's chara karaka (ch. 8 §8.2); none for Ketu and every point that is no graha.
    karaka: Option<CharaKaraka>,
    /// In the rasi chart and the varga shown, from the reference point: `functional-…` and whether it is the yogakaraka
    /// (ch. 13 §13.2, Table 30); none for the nodes and every point that is no graha.
    functional: [Option<(String, bool)>; 2],
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
        karaka: None,
        functional: [None, None],
    }
}

/// The table rows of `chart`, in its order, with `karakas`, bhavas counted from `reference_rasi` and functional nature from
/// `reference` in the rasi chart and `varga`.
fn rows(
    chart: &hora_chart::Chart,
    karakas: &[(Body, CharaKaraka)],
    reference_rasi: Rasi,
    reference: ReferencePoint,
    varga: Varga,
) -> Vec<Row> {
    chart
        .entries()
        .iter()
        .map(|entry| Row {
            functional: [Varga::RasiChart, varga].map(|d| {
                let v = chart.in_varga(d);
                let nature = v.functional_nature(entry.body, reference)?;
                let yogakaraka = v.yogakaraka(reference) == Some(entry.body);
                Some((kb::message_id("functional", nature), yogakaraka))
            }),
            retrograde: entry.retrograde,
            dignity: hora_chart::dignity(entry.body, entry.longitude)
                .map(|d| kb::message_id("dignity", d)),
            relationship: chart
                .lord_relationship(entry.body)
                .map(|r| kb::message_id("relationship", r)),
            karaka: karakas.iter().find(|k| k.0 == entry.body).map(|k| k.1),
            ..point_row(body_id(entry.body).into(), entry.longitude, reference_rasi)
        })
        .collect()
}

/// A row of an arudha table: an arudha pada's rasi and bhava in the rasi chart and in the varga chakra shown.
struct ArudhaRow {
    /// `bhava-arudha-…`, its message id, or `arudha-graha-…`, a graha's id after `arudha-`; its table row's and label's `data-body`.
    id: String,
    rasi: &'static str,
    /// 1 to 12, counted from the reference point's rasi in the rasi chart.
    bhava: u8,
    varga_rasi: &'static str,
    /// 1 to 12, counted from the reference point's rasi in the varga.
    varga_bhava: u8,
}

/// A row of an aspects table: what one graha aspects in one chart.
struct AspectRow {
    graha: &'static str,
    graha_drishti: Aspected,
    rasi_drishti: Aspected,
}

/// What a graha aspects by one drishti.
struct Aspected {
    /// (message id, bhava from that chart's reference point), in count order from the graha.
    rasis: Vec<(&'static str, u8)>,
    /// The grahas in those rasis, in the same order, as Exercises 14 and 15 list them.
    grahas: Vec<&'static str>,
}

/// What each graha aspects in `d`'s chart, counting bhavas from `reference` (ch. 10 §10.2–10.3).
fn aspect_rows(d: Varga, chart: &hora_chart::Chart, reference: Rasi) -> Vec<AspectRow> {
    let aspected = |rasis: Vec<Rasi>| Aspected {
        grahas: rasis
            .iter()
            .flat_map(|&rasi| chart.in_varga(d).grahas_in(rasi))
            .map(body_id)
            .collect(),
        rasis: rasis
            .iter()
            .map(|&rasi| (rasi_id(rasi), hora_chart::bhava(reference, rasi)))
            .collect(),
    };
    chart
        .grahas()
        .iter()
        .map(|e| {
            let at = hora_chart::varga(d, e.longitude);
            AspectRow {
                graha: body_id(e.body),
                graha_drishti: aspected(hora_chart::graha_drishti(e.body, at)),
                rasi_drishti: aspected(hora_chart::rasi_drishtis(at)),
            }
        })
        .collect()
}

/// A row of an argala table: one rasi in one chart.
struct ArgalaRow {
    /// 1 to 12, from that chart's reference point.
    bhava: u8,
    rasi: &'static str,
    /// The rasi holds Ketu, so its counts run in reverse (ch. 10 §10.6's note).
    reversed: bool,
    /// (argala, virodhargala) as graha message ids: 2nd/12th, 4th/10th, 5th/9th, 11th/3rd.
    pairs: [(Vec<&'static str>, Vec<&'static str>); 4],
}

/// The argalas on every rasi in `d`'s chart, the bhava `reference` is in first (ch. 10 §10.5–10.6, ADR 0013).
fn argala_rows(d: Varga, chart: &hora_chart::Chart, reference: Rasi) -> Vec<ArgalaRow> {
    let ids = |grahas: Vec<Body>| grahas.into_iter().map(body_id).collect();
    let mut rasis = hora_chart::RASIS;
    rasis.sort_by_key(|&rasi| hora_chart::bhava(reference, rasi));
    rasis
        .map(|rasi| ArgalaRow {
            bhava: hora_chart::bhava(reference, rasi),
            rasi: rasi_id(rasi),
            reversed: chart.in_varga(d).argala_reversed(rasi),
            pairs: chart
                .in_varga(d)
                .argala(rasi)
                .map(|argala| (ids(argala.argala), ids(argala.virodhargala))),
        })
        .into()
}

/// A row of a baadhakas table: one bhava in one chart (ch. 13 §13.3).
struct BaadhakaRow {
    /// 1 to 12, from that chart's reference point.
    bhava: u8,
    rasi: &'static str,
    /// The bhava arudhas in it, as `bhava-arudha-…` ids (p. 170's "baadhaka from A3").
    arudhas: Vec<String>,
    /// Its baadhaka sthaana's rasi and bhava.
    sthaana: (&'static str, u8),
    baadhakas: Vec<&'static str>,
    /// The grahas in the baadhaka sthaana (p. 168's "planets in Sg").
    grahas: Vec<&'static str>,
}

/// Every bhava's baadhaka sthaana and baadhakas in `d`'s chart, the bhava `reference` is in first (ch. 13 §13.3, Table 31).
fn baadhaka_rows(d: Varga, chart: &hora_chart::Chart, reference: Rasi) -> Vec<BaadhakaRow> {
    let v = chart.in_varga(d);
    let arudhas = v.bhava_arudhas();
    let mut rasis = hora_chart::RASIS;
    rasis.sort_by_key(|&rasi| hora_chart::bhava(reference, rasi));
    rasis
        .map(|rasi| {
            let sthaana = hora_chart::baadhaka_sthaana(rasi);
            BaadhakaRow {
                bhava: hora_chart::bhava(reference, rasi),
                rasi: rasi_id(rasi),
                arudhas: (0..12)
                    .filter(|&n| arudhas[n] == rasi)
                    .map(arudha_id)
                    .collect(),
                sthaana: (rasi_id(sthaana), hora_chart::bhava(reference, sthaana)),
                baadhakas: hora_chart::baadhakas(rasi)
                    .into_iter()
                    .map(body_id)
                    .collect(),
                grahas: v.grahas_in(sthaana).into_iter().map(body_id).collect(),
            }
        })
        .into()
}

/// A row of a yogadas table: one yogada in one chart (ch. 13 §13.2).
struct YogadaRow {
    graha: &'static str,
    /// `yogada-gives-…`: money (HL) or power (GL).
    gives: String,
    /// Its links to the lagna, then to HL or GL, in words.
    links: [Vec<String>; 2],
}

/// The yogadas of `d`'s chart, from its own lagna, HL and GL; none without a Vedic day (ch. 13 §13.2, ADR 0016).
fn yogada_rows(d: Varga, chart: &hora_chart::Chart) -> Vec<YogadaRow> {
    chart
        .in_varga(d)
        .yogadas()
        .into_iter()
        .map(|y| YogadaRow {
            graha: body_id(y.graha),
            gives: kb::message_id("yogada-gives", y.gives),
            links: y.links.map(|links| {
                links
                    .into_iter()
                    .map(|l| kb::message_id("yogada-link", l))
                    .collect()
            }),
        })
        .collect()
}

/// The maraka grahas of `d`'s chart, counted from its lagna, each with its reasons in words (ch. 14 §14.2, ADR 0017).
fn maraka_rows(
    lang: Lang,
    d: Varga,
    chart: &hora_chart::Chart,
) -> Vec<(&'static str, Vec<String>)> {
    use hora_chart::MarakaReason as R;
    let graha = |g| LOCALES.lookup(&lang.id(), body_id(g));
    chart
        .in_varga(d)
        .maraka_reasons()
        .into_iter()
        .map(|m| {
            let reasons = m.reasons.into_iter().map(|reason| {
                let (id, house, lord) = match reason {
                    R::Owns(h) => ("maraka-owns", h, None),
                    R::Occupies(h) => ("maraka-occupies", h, None),
                    R::Aspects(h) => ("maraka-aspects", h, None),
                    R::ConjoinsLord(h, l) => ("maraka-conjoins-lord", h, Some(l)),
                    R::AspectsLord(h, l) => ("maraka-aspects-lord", h, Some(l)),
                };
                let lord = lord.map_or(String::new(), graha);
                message(lang, id, [("house", house.to_string()), ("lord", lord)])
            });
            (body_id(m.graha), reasons.collect())
        })
        .collect()
}

/// A row of the three pairs table (ch. 14 §14.4).
struct PairRow {
    /// Which pair, in words.
    label: String,
    /// Its two rasis, each with its `mobility-…` id.
    rasis: [(&'static str, String); 2],
    /// Its `longevity-…` id.
    longevity: String,
}

/// What the Longevity topic shows of the rasi chart, counted from the lagna whatever the reference point (ch. 14, ADR 0017).
struct LongevityView {
    /// Rudra's candidates, the 8th from the lagna and from the 7th: (that rasi, its lord).
    candidates: [(&'static str, &'static str); 2],
    /// Whether both 8ths were counted normally, Saturn being in the lagna or the 7th (p. 276), not by Table 32.
    counted_normally: bool,
    rudra: &'static str,
    /// The `rudra-rule-…` id of the rule that decided, `rudra-rule-tie` if none did.
    decided_by: String,
    weaker: &'static str,
    /// Whether the weaker is afflicted, and so Rudra too.
    weaker_afflicted: bool,
    trishoola: [&'static str; 3],
    maheswara: &'static str,
    /// The `maheswara-exception-…` ids of the exceptions applied, in the order applied.
    maheswara_exceptions: Vec<String>,
    /// The three pairs; the third only with a Vedic day.
    pairs: Vec<PairRow>,
    /// The combined category in words with its years, and the `pairs-rule-…` id; none without the third pair.
    result: Option<(String, String)>,
    /// The paramaayush in words, or why there is none.
    paramaayush: String,
    /// The eighth lord method's reference rasi, 8th lord, his house from it, and the category in words with its years.
    eighth_lord: (&'static str, &'static str, u8, String),
}

impl LongevityView {
    fn new(lang: Lang, chart: &hora_chart::Chart) -> Self {
        let t = |id: &str| LOCALES.lookup(&lang.id(), id);
        let category = |l: hora_chart::Longevity| {
            let (from, to) = l.years();
            let years = message(
                lang,
                "longevity-years",
                [("from", from.to_string()), ("to", to.to_string())],
            );
            format!("{} ({years})", t(&kb::message_id("longevity", l)))
        };
        let d1 = chart.in_varga(Varga::RasiChart);
        let rudra = d1.rudra();
        let maheswara = d1.maheswara();
        let three = d1.three_pairs();
        let eighth = d1.eighth_lord_method();
        let row = |label: String, pair: hora_chart::Pair| PairRow {
            label,
            rasis: pair
                .rasis
                .map(|r| (rasi_id(r), kb::message_id("mobility", r.mobility()))),
            longevity: kb::message_id("longevity", pair.longevity),
        };
        let [lagna_lord, eighth_lord] = three.lords.map(|g| t(body_id(g)));
        let mut pairs = vec![
            row(
                message(
                    lang,
                    "pair-lords",
                    [("lagna_lord", lagna_lord), ("eighth_lord", eighth_lord)],
                ),
                three.pairs[0],
            ),
            row(t("pair-moon-saturn"), three.pairs[1]),
        ];
        pairs.extend(three.hl_pair.map(|p| row(t("pair-lagna-hl"), p)));
        LongevityView {
            candidates: rudra
                .eighths
                .map(|r| (rasi_id(r), body_id(d1.stronger_lord(r)))),
            counted_normally: rudra.counted_normally,
            rudra: body_id(rudra.rudra),
            decided_by: rudra.decided_by.map_or("rudra-rule-tie".to_owned(), |r| {
                kb::message_id("rudra-rule", r)
            }),
            weaker: body_id(rudra.weaker),
            weaker_afflicted: rudra.weaker_afflicted,
            trishoola: rudra.trishoola.map(rasi_id),
            maheswara: body_id(maheswara.graha),
            maheswara_exceptions: maheswara
                .exceptions
                .into_iter()
                .map(|e| kb::message_id("maheswara-exception", e))
                .collect(),
            pairs,
            result: three
                .result
                .map(|(l, rule)| (category(l), kb::message_id("pairs-rule", rule))),
            paramaayush: three.paramaayush.map_or(t("paramaayush-none"), |years| {
                message(lang, "paramaayush-years", [("years", years.to_string())])
            }),
            eighth_lord: (
                rasi_id(eighth.reference),
                body_id(eighth.lord),
                eighth.house,
                category(eighth.longevity),
            ),
        }
    }
}

/// A graha's row of the avasthas table, each state by its message id (ch. 15 §15.4).
struct AvasthaRow {
    graha: &'static str,
    /// `age-avastha-…`; its share of results is `…-share`.
    age: String,
    /// None for a node with no relationship to read (ADR 0018).
    alertness: Option<String>,
    moods: Vec<String>,
    /// The Sayanaadi avastha and its strength: None without a Vedic day; the strength None without the name sound.
    sayanaadi: Option<(String, Option<String>)>,
}

/// What the Strength topic shows of the rasi chart, counted from the lagna whatever the reference point (ch. 15, ADR 0018).
struct StrengthView {
    /// The Sun to Ketu.
    avasthas: Vec<AvasthaRow>,
    /// Scorpio's and Aquarius's stronger lord with the `stronger-rule-…` id that decided (ch. 15 §15.5.1).
    stronger_lords: [(&'static str, &'static str, String); 2],
    /// The seven in §8.3's order: `kin-…`, the graha, and for the father and mother the `stronger-rule-…` id that decided.
    sthira_karakas: Vec<(String, &'static str, Option<String>)>,
}

impl StrengthView {
    fn new(chart: &hora_chart::Chart, sky: &Sky, sound: Option<u8>) -> Self {
        let sayanaadi = hora_chart::sayanaadi_avasthas(sky, sound);
        let d1 = chart.in_varga(Varga::RasiChart);
        let rule = |r: Option<hora_chart::StrongerRule>| {
            r.map_or("stronger-rule-tie".to_owned(), |r| {
                kb::message_id("stronger-rule", r)
            })
        };
        StrengthView {
            avasthas: chart
                .avasthas()
                .into_iter()
                .map(|a| AvasthaRow {
                    graha: body_id(a.graha),
                    age: kb::message_id("age-avastha", a.age),
                    alertness: a.alertness.map(|l| kb::message_id("alertness", l)),
                    moods: a
                        .moods
                        .into_iter()
                        .map(|m| kb::message_id("mood-avastha", m))
                        .collect(),
                    sayanaadi: sayanaadi
                        .iter()
                        .flatten()
                        .find(|s| s.graha == a.graha)
                        .map(|s| {
                            (
                                kb::message_id("sayanaadi-avastha", s.avastha),
                                s.strength.map(|s| kb::message_id("activity-strength", s)),
                            )
                        }),
                })
                .collect(),
            stronger_lords: [Rasi::Scorpio, Rasi::Aquarius].map(|r| {
                let (graha, decided) = d1.stronger_lord_by(r);
                (rasi_id(r), body_id(graha), rule(decided))
            }),
            sthira_karakas: chart
                .sthira_karakas()
                .into_iter()
                .map(|k| {
                    let decided =
                        matches!(k.kin, hora_chart::Kin::Father | hora_chart::Kin::Mother)
                            .then(|| rule(k.rule));
                    (kb::message_id("kin", k.kin), body_id(k.graha), decided)
                })
                .collect(),
        }
    }
}

/// A yoga class's rows in a yoga table, in the book's order (ch. 11).
struct YogaGroup {
    /// The class's message id.
    class: String,
    /// The class's knowledge base page.
    page: &'static str,
    rows: Vec<YogaRow>,
}

/// A row of a yoga table: one present or to-judge yoga in one chart.
struct YogaRow {
    /// The name as the book spells it, with its form where it has one.
    name: String,
    /// (graha message id, its role in the yoga and the bhava it is in), counted from that chart's lagna: the role is its
    /// karaka or lordship where the row names one (§11.7.3 (1)), else every lordship as a yoga counts them.
    grahas: Vec<(&'static str, String)>,
    /// Each condition left to the reader, in words.
    to_judge: Vec<String>,
    /// A Raaja pair's afflictions (§11.7.2 (1)), then the book's notes (ch. 13 §13.5), in words.
    notes: Vec<String>,
    /// A Raaja pair's closeness, and each graha's (message id, Dasa Varga count, amsa message id) (§11.7.2).
    magnitude: Option<(String, Vec<AmsaCell>)>,
}

/// A graha's message id, its Dasa Varga count, and its amsa's message id from 2 (ch. 6 §6.6).
type AmsaCell = (&'static str, u8, Option<String>);

/// §11.7.2's closeness in arcseconds as degrees and minutes.
fn closeness(arcseconds: u32) -> String {
    // Rounded, unlike positions: chart 6's pair, over 16′30″ apart, is the book's "17'" (p. 230), as hora-chart's golden test reads it.
    let minutes = (arcseconds + 30) / 60;
    format!("{}°{:02}′", minutes / 60, minutes % 60)
}

/// The yogas of `d`'s chart grouped by class, each graha's bhava counted from that chart's lagna, never the reference point
/// (ch. 11, ADR 0014).
fn yoga_groups(lang: Lang, d: Varga, chart: &hora_chart::Chart) -> Vec<YogaGroup> {
    use hora_chart::{Note, ToJudge, YogaName as N};
    let mut groups: Vec<YogaGroup> = Vec::new();
    for yoga in chart.in_varga(d).yogas() {
        let class = yoga.name.class();
        let name = match yoga.name {
            N::Raaja(n) => message(lang, "yoga-raaja-n", [("number", n.to_string())]),
            N::RaajaSambandha(n) => {
                message(lang, "yoga-raaja-sambandha-n", [("number", n.to_string())])
            }
            N::Dhana(n) => message(lang, "yoga-dhana-n", [("number", n.to_string())]),
            N::Daridra(n) => message(lang, "yoga-daridra-n", [("number", n.to_string())]),
            name => LOCALES.lookup(&lang.id(), &kb::message_id("yoga", name)),
        };
        let name = match (yoga.name, yoga.form) {
            // Its grahas say which half held, by their roles below.
            (N::Raaja(1), _) => name,
            (_, Some(3)) => message(lang, "yoga-forms-both", [("name", name)]),
            (_, Some(form)) => message(
                lang,
                "yoga-form",
                [("name", name), ("form", form.to_string())],
            ),
            (_, None) => name,
        };
        let graha = |g| LOCALES.lookup(&lang.id(), body_id(g));
        let row = YogaRow {
            name,
            grahas: yoga
                .grahas
                .iter()
                .map(|g| {
                    let karaka = g.karaka.map(|k| {
                        LOCALES.lookup(
                            &lang.id(),
                            &format!("{}-short", kb::message_id("chara-karaka", k)),
                        )
                    });
                    let lords = g
                        .named_lord_of
                        .iter()
                        .map(|n| message(lang, "yoga-lord", [("bhava", n.to_string())]));
                    let roles: Vec<String> = karaka.into_iter().chain(lords).collect();
                    let bhava = g.bhava.to_string();
                    let place = if roles.is_empty() {
                        message(lang, "yoga-graha-in", [("bhava", bhava)])
                    } else {
                        message(
                            lang,
                            "yoga-graha-roles-in",
                            [("roles", roles.join(", ")), ("bhava", bhava)],
                        )
                    };
                    (body_id(g.graha), place)
                })
                .collect(),
            to_judge: yoga
                .to_judge
                .iter()
                .map(|condition| match *condition {
                    ToJudge::MercurysNature => {
                        LOCALES.lookup(&lang.id(), "yoga-to-judge-mercurys-nature")
                    }
                    ToJudge::MayNotOperateWell => {
                        LOCALES.lookup(&lang.id(), "yoga-to-judge-may-not-operate-well")
                    }
                    ToJudge::Strong(g) => {
                        message(lang, "yoga-to-judge-strong", [("graha", graha(g))])
                    }
                    ToJudge::Combust(g) => {
                        message(lang, "yoga-to-judge-combust", [("graha", graha(g))])
                    }
                    ToJudge::DeepExaltation(g) => {
                        message(lang, "yoga-to-judge-deep-exaltation", [("graha", graha(g))])
                    }
                    ToJudge::MaleficInUpachaya(g) => message(
                        lang,
                        "yoga-to-judge-malefic-in-upachaya",
                        [("graha", graha(g))],
                    ),
                    ToJudge::AllInOneRasi => {
                        LOCALES.lookup(&lang.id(), "yoga-to-judge-all-in-one-rasi")
                    }
                })
                .collect(),
            notes: yoga
                .afflictions
                .iter()
                .map(|a| {
                    let (graha, afflicts) = (graha(a.graha), graha(a.afflicts));
                    match a.closeness {
                        Some(c) => message(
                            lang,
                            "yoga-affliction-closeness",
                            [
                                ("graha", graha),
                                ("afflicts", afflicts),
                                ("closeness", closeness(c)),
                            ],
                        ),
                        None => message(
                            lang,
                            "yoga-affliction",
                            [("graha", graha), ("afflicts", afflicts)],
                        ),
                    }
                })
                .chain(yoga.notes.iter().map(|note| match *note {
                    Note::AtmaKaraka(g) => {
                        message(lang, "yoga-note-atma-karaka", [("graha", graha(g))])
                    }
                    Note::InTheEighth(g) => {
                        message(lang, "yoga-note-in-the-eighth", [("graha", graha(g))])
                    }
                }))
                .collect(),
            magnitude: yoga.magnitude.map(|m| {
                (
                    closeness(m.closeness),
                    m.amsas
                        .iter()
                        .map(|&(g, count, amsa)| {
                            (body_id(g), count, amsa.map(|a| kb::message_id("amsa", a)))
                        })
                        .collect(),
                )
            }),
        };
        let class_id = kb::message_id("yoga-class", class);
        match groups.last_mut() {
            Some(group) if group.class == class_id => group.rows.push(row),
            _ => groups.push(YogaGroup {
                class: class_id,
                page: kb::yoga_page(class),
                rows: vec![row],
            }),
        }
    }
    groups
}

/// A count in an ashtakavarga table and the message ids of its marks, each also with `-mark` its visible sign.
type AvCell = (u8, Vec<String>);

/// A PAV's row: a reference's id and whether it gives a rekha in each rasi from Aries (ch. 12 §12.6).
type PavRow = (&'static str, [bool; 12]);

/// One chart's ashtakavarga, counted from its own lagna and positions, never the reference point (ch. 12 §12.5, ADR 0015).
struct Ashtakavarga {
    /// Each rasi's bhava from Aries, counted from the chart's lagna.
    bhavas: [u8; 12],
    /// (row id, cells from Aries): the seven planets' BAVs, then the SAV, then the lagna's BAV (ch. 12 §12.3–12.4).
    bav: Vec<(&'static str, [AvCell; 12])>,
    /// (summary, rows) for each PAV, of the seven planets and the lagna (ch. 12 §12.6).
    pav: Vec<(String, Vec<PavRow>)>,
    /// (graha id, SoAV from Aries) of the seven planets (ch. 12 §12.7).
    soav: Vec<(&'static str, [u8; 12])>,
    /// (graha id, pindas) of the seven planets (ch. 12 §12.7.3).
    pindas: Vec<(&'static str, hora_chart::Pindas)>,
}

impl Ashtakavarga {
    fn new(lang: Lang, d: Varga, chart: &hora_chart::Chart) -> Self {
        use hora_chart::{AV_REFERENCES, PLANETS, RASIS};
        let v = chart.in_varga(d);
        let lagna = v.rasi_of(Body::Lagna);
        let bav_row = |planet: Body| {
            let (bav, at) = (v.bav(planet), v.rasi_of(planet));
            let cells = RASIS.map(|rasi| {
                let count = bav[rasi as usize];
                let band = match hora_chart::bav_band(planet, count) {
                    Some(hora_chart::BavBand::Neutral) | None => None,
                    Some(band) => Some(kb::message_id("av-bav", band)),
                };
                let natal = (rasi == at && planet != Body::Lagna).then(|| "av-natal".to_owned());
                (count, band.into_iter().chain(natal).collect())
            });
            (body_id(planet), cells)
        };
        let sav = v.sav().map(|count| {
            let band = match hora_chart::sav_band(count) {
                hora_chart::SavBand::Average => None,
                band => Some(kb::message_id("av-sav", band)),
            };
            (count, band.into_iter().collect())
        });
        Ashtakavarga {
            bhavas: RASIS.map(|rasi| hora_chart::bhava(lagna, rasi)),
            bav: PLANETS
                .map(bav_row)
                .into_iter()
                .chain([("av-sav", sav), ("av-lagna-bav", bav_row(Body::Lagna).1)])
                .collect(),
            pav: AV_REFERENCES
                .map(|of| {
                    let rows = AV_REFERENCES
                        .map(body_id)
                        .into_iter()
                        .zip(v.pav(of))
                        .collect();
                    let body = LOCALES.lookup(&lang.id(), body_id(of));
                    (message(lang, "av-pav-of", [("body", body)]), rows)
                })
                .into(),
            soav: PLANETS.map(|p| (body_id(p), v.soav(p))).into(),
            pindas: PLANETS.map(|p| (body_id(p), v.pindas(p))).into(),
        }
    }
}

/// The message id of the bhava arudha of the `n`th bhava from 0: AL (0) to UL (11), with `-short` its abbreviation (ch. 9 §9.2).
fn arudha_id(n: usize) -> String {
    format!("bhava-arudha-{}", n + 1)
}

/// A graha's count of good vargas in a varga group, and its amsa's message id from 2 (ch. 6 §6.6).
type AmsabalaCell = (u8, Option<String>);

/// A varga's row of the Vargas table and its link in the varga chakra's link row.
struct VargaRow {
    /// The n of D-n.
    parts: u8,
    /// E.g. "Navamsa (D-9)".
    name: String,
    /// The chart shown with this varga chakra.
    href: String,
    /// The rasi message id of each of the table's columns.
    rasis: Vec<&'static str>,
}

/// A varga's name with its D-n, as the glossary writes it on first use.
fn varga_name(lang: Lang, varga: Varga) -> String {
    let name = LOCALES.lookup(&lang.id(), &kb::message_id("varga", varga));
    message(
        lang,
        "varga-full",
        [("name", name), ("parts", varga.parts().to_string())],
    )
}

/// The page at `uri` with the field `name` set to `value`, at `#fragment`: its query with `name` replaced, as sent.
fn query_href(uri: &Uri, name: &str, value: &str, fragment: &str) -> String {
    let query: String = uri
        .query()
        .unwrap_or("")
        .split('&')
        .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(name))
        .map(|pair| format!("{pair}&"))
        .collect();
    let fragment = if fragment.is_empty() {
        String::new()
    } else {
        format!("#{fragment}")
    };
    format!("{}?{query}{name}={value}{fragment}", uri.path())
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

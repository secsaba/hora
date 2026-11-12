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
    Body, CharaKaraka, Karana, LunarMonth, Nakshatra, Paksha, Rasi, ReferencePoint, SpecialLagna,
    SunMoonYoga, Tithi, TwinMonth, Vaara, Varga,
};

mod book_charts;
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
    /// With a chart: its bhava arudhas' rows, AL to UL (ch. 9 §9.2).
    bhava_arudhas: Vec<ArudhaRow>,
    /// With a chart: its graha arudhas' rows, the Sun to Ketu (ch. 9 §9.5).
    graha_arudhas: Vec<ArudhaRow>,
    /// With a chart: what each graha aspects in the rasi chart, the Sun to Ketu (ch. 10 §10.2–10.3).
    aspects: Vec<AspectRow>,
    /// With a chart: what each graha aspects in the varga chakra shown.
    varga_aspects: Vec<AspectRow>,
    /// With a chart: the argalas on each rasi in the rasi chart, from the reference point's (ch. 10 §10.5–10.6).
    argalas: Vec<ArgalaRow>,
    /// With a chart: the argalas on each rasi in the varga chakra shown.
    varga_argalas: Vec<ArgalaRow>,
    /// With a chart: the yogas of the rasi chart by class, counted from its lagna (ch. 11).
    yogas: Vec<YogaGroup>,
    /// With a chart: the yogas of the varga chakra shown, counted from its lagna.
    varga_yogas: Vec<YogaGroup>,
    /// With a chart: the varga chakra shown, by name, heading the arudha tables' columns for it.
    varga_name: String,
    /// With a chart: its upagrahas' rows, each with its section on the upagraha page.
    upagrahas: Vec<(&'static str, Row)>,
    /// With a chart: its special lagnas' rows, HL, GL, SL and BL, or SL alone without a Vedic day (ch. 5).
    special_lagnas: Vec<Row>,
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
    /// With a chart: whether the grahas table is in chara karaka order, else graha order.
    by_karaka: bool,
    /// With a chart: the chart sorted by graha and by chara karaka, at the grahas table.
    sort_hrefs: [String; 2],
    /// With a chart: the n of the varga chakra shown (ADR 0003).
    varga: u8,
    /// With a chart: the varga chakra shown, drawn in the chosen style.
    varga_drawing: chakra::Drawing,
    /// With a chart: the Vargas table's columns, (name, short) message ids: the lagna, the grahas, then HL and GL with a Vedic day, and SL.
    varga_columns: Vec<(String, String)>,
    /// With a chart: each varga's row of the Vargas table and link in the link row, D-1 to D-60.
    vargas: Vec<VargaRow>,
    /// With a chart: each graha's (message id, (count, amsa message id) in each varga group) (ch. 6 §6.6).
    amsabala: Vec<(&'static str, [AmsabalaCell; 4])>,
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
            bhava_arudhas: Vec::new(),
            graha_arudhas: Vec::new(),
            aspects: Vec::new(),
            varga_aspects: Vec::new(),
            argalas: Vec::new(),
            varga_argalas: Vec::new(),
            yogas: Vec::new(),
            varga_yogas: Vec::new(),
            varga_name: String::new(),
            upagrahas: Vec::new(),
            special_lagnas: Vec::new(),
            ayanamsa: String::new(),
            vedic_day: None,
            panchaanga: Vec::new(),
            limb_spans: Vec::new(),
            drawing: chakra::Drawing::default(),
            by_karaka: false,
            sort_hrefs: Default::default(),
            varga: 0,
            varga_drawing: chakra::Drawing::default(),
            varga_columns: Vec::new(),
            vargas: Vec::new(),
            amsabala: Vec::new(),
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

    /// The reference points the shown chart has: without a Vedic day, no HL or GL (ch. 5).
    fn references(&self) -> Vec<(&'static str, ReferencePoint, &'static str)> {
        REFERENCES
            .iter()
            .filter(|(_, reference, _)| {
                self.chart.is_empty()
                    || self.vedic_day.is_some()
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
        Ok((request, reference, style, varga, by_karaka)) => {
            match hora_swe::calculate(request.clone()).await {
                Ok(sky) => 'chart: {
                    let chart = hora_chart::chart(&sky);
                    let karakas = chart.chara_karakas();
                    let rasi_arudhas = chart.in_varga(Varga::RasiChart).bhava_arudhas();
                    let varga_arudhas = chart.in_varga(varga).bhava_arudhas();
                    // North Indian counts from the reference point's rasi in the varga (ch. 6 §6.2).
                    let (Some(reference_rasi), Some(varga_reference)) = (
                        chart.in_varga(Varga::RasiChart).reference(reference),
                        chart.in_varga(varga).reference(reference),
                    ) else {
                        // HL and GL need a Vedic day; the form offers them only where there is one.
                        page.errors
                            .insert("reference", "special-lagnas-no-vedic-day");
                        break 'chart StatusCode::BAD_REQUEST;
                    };
                    page.chart = rows(&chart, &karakas, reference_rasi);
                    if by_karaka {
                        // The lagna first, AK to DK (ties stay in graha order), then Ketu, who holds none.
                        page.chart
                            .sort_by_key(|row| match (row.name.as_str(), row.karaka) {
                                ("lagna", _) => 0,
                                (_, Some(karaka)) => 1 + karaka as u8,
                                (_, None) => 9,
                            });
                    }
                    page.by_karaka = by_karaka;
                    page.sort_hrefs =
                        ["graha", "karaka"].map(|sort| query_href(&uri, "sort", sort, "grahas"));
                    let arudha_row = |id, rasi, varga_rasi| ArudhaRow {
                        id,
                        rasi: rasi_id(rasi),
                        bhava: hora_chart::bhava(reference_rasi, rasi),
                        varga_rasi: rasi_id(varga_rasi),
                        varga_bhava: hora_chart::bhava(varga_reference, varga_rasi),
                    };
                    page.bhava_arudhas = (0..12)
                        .map(|n| arudha_row(arudha_id(n), rasi_arudhas[n], varga_arudhas[n]))
                        .collect();
                    page.graha_arudhas = chart
                        .in_varga(Varga::RasiChart)
                        .graha_arudhas()
                        .into_iter()
                        .zip(chart.in_varga(varga).graha_arudhas())
                        .map(|((graha, rasi), (_, varga_rasi))| {
                            arudha_row(format!("arudha-{}", body_id(graha)), rasi, varga_rasi)
                        })
                        .collect();
                    page.aspects = aspect_rows(Varga::RasiChart, &chart, reference_rasi);
                    page.varga_aspects = aspect_rows(varga, &chart, varga_reference);
                    page.argalas = argala_rows(Varga::RasiChart, &chart, reference_rasi);
                    page.varga_argalas = argala_rows(varga, &chart, varga_reference);
                    page.yogas = yoga_groups(lang, Varga::RasiChart, &chart);
                    page.varga_yogas = yoga_groups(lang, varga, &chart);
                    page.varga_name = varga_name(lang, varga);
                    page.upagrahas = upagraha_rows(&sky, reference_rasi);
                    page.special_lagnas = chart
                        .special_lagnas()
                        .iter()
                        .map(|&(lagna, longitude)| {
                            point_row(
                                kb::message_id("special-lagna", lagna),
                                longitude,
                                reference_rasi,
                            )
                        })
                        .collect();
                    page.ayanamsa = dms(sky.ayanamsa);
                    page.vedic_day = sky.vedic_day.map(|day| vedic_day_rows(lang, &request, day));
                    page.panchaanga = panchaanga(lang, &sky);
                    page.limb_spans = limb_spans(lang, &request, &sky.limbs);
                    page.drawing = chakra::drawing(lang, style, &chart, reference_rasi, None);
                    page.varga = varga.parts();
                    page.varga_drawing =
                        chakra::drawing(lang, style, &chart, varga_reference, Some(varga));
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
                    page.vargas = hora_chart::VARGAS
                        .map(|varga| VargaRow {
                            parts: varga.parts(),
                            name: varga_name(lang, varga),
                            href: query_href(
                                &uri,
                                "varga",
                                &format!("d{}", varga.parts()),
                                "varga",
                            ),
                            rasis: points
                                .iter()
                                .map(|p| rasi_id(hora_chart::varga(varga, p.2)))
                                .collect(),
                        })
                        .into();
                    page.varga_columns = points
                        .into_iter()
                        .map(|(id, short, _)| (id, short))
                        .collect();
                    page.amsabala = chart
                        .grahas()
                        .iter()
                        .map(|e| {
                            let groups = hora_chart::VARGA_GROUPS.map(|group| {
                                let (count, amsa) =
                                    hora_chart::amsabala(e.body, e.longitude, group);
                                (count, amsa.map(|a| kb::message_id("amsa", a)))
                            });
                            (body_id(e.body), groups)
                        })
                        .collect();
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
                        page.before_1970 =
                            hora_swe::civil_date(local, Calendar::Gregorian)[0] < 1970;
                    }
                    StatusCode::OK
                }
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
/// karaka (ch. 7 §7.3.6, ch. 8 §8.2), then the arudha and upapada lagnas (ch. 7 §7.3.4, ch. 9 §9.3–9.4).
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
    ("arudha-lagna", ReferencePoint::Arudha(0), "bhava-arudha-1"),
    (
        "upapada-lagna",
        ReferencePoint::Arudha(11),
        "bhava-arudha-12",
    ),
];

/// The grahas table's order: `true` is by chara karaka.
const SORTS: &Choices<bool> = &[("graha", false, "graha"), ("karaka", true, "chara-karaka")];

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

/// The form as a request, the bhavas' reference point, the chart style, the varga chakra shown and whether the grahas
/// table is in chara karaka order, or each unreadable field's message id; whether the values are in range is `hora_swe`'s to say.
/// With `utc=zone`, the offset is `zone`'s at the birth time (ADR 0005).
fn parse(
    form: &HashMap<String, String>,
    zone: Option<&TimeZone>,
) -> Result<(Request, ReferencePoint, chakra::Style, Varga, bool), Refusal> {
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
    // Not a `Choices`: its default, navamsa (ADR 0003), is not its first.
    let varga = match value("varga") {
        "" => Some(Varga::Navamsa),
        value => hora_chart::VARGAS
            .into_iter()
            .find(|v| format!("d{}", v.parts()) == value),
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
        None => Ok((request, reference, style, varga, by_karaka)),
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
    /// The graha's chara karaka (ch. 8 §8.2); none for Ketu and every point that is no graha.
    karaka: Option<CharaKaraka>,
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
    }
}

/// The table rows of `chart`, in its order, with `karakas` and bhavas counted from `reference_rasi`.
fn rows(
    chart: &hora_chart::Chart,
    karakas: &[(Body, CharaKaraka)],
    reference_rasi: Rasi,
) -> Vec<Row> {
    chart
        .entries()
        .iter()
        .map(|entry| Row {
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
    /// A Raaja pair's closeness, and each graha's (message id, Dasa Varga count, amsa message id) (§11.7.2).
    magnitude: Option<(String, Vec<AmsaCell>)>,
}

/// A graha's message id, its Dasa Varga count, and its amsa's message id from 2 (ch. 6 §6.6).
type AmsaCell = (&'static str, u8, Option<String>);

/// The yogas of `d`'s chart grouped by class, each graha's bhava counted from that chart's lagna, never the reference point
/// (ch. 11, ADR 0014).
fn yoga_groups(lang: Lang, d: Varga, chart: &hora_chart::Chart) -> Vec<YogaGroup> {
    use hora_chart::{ToJudge, YogaName as N};
    let v = chart.in_varga(d);
    let lagna = v.rasi_of(Body::Lagna);
    // The rasi chart's, as hora-chart's §11.7.3 reads them in every chart.
    let karakas = chart.chara_karakas();
    let mut groups: Vec<YogaGroup> = Vec::new();
    for yoga in v.yogas() {
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
                .map(|&g| {
                    let lord = |n: u8| message(lang, "yoga-lord", [("bhava", n.to_string())]);
                    let mut lord_of: Vec<u8> = hora_chart::RASIS
                        .into_iter()
                        .filter(|&rasi| v.lords_in_yoga(rasi).contains(&g))
                        .map(|rasi| hora_chart::bhava(lagna, rasi))
                        .collect();
                    lord_of.sort();
                    let roles: Vec<String> = match (yoga.name, yoga.form) {
                        // §11.7.3 (1): form 1 is (a) AK with PK, 2 is (b) the lagna and 5th lords, 3 both.
                        (N::Raaja(1), Some(form)) => {
                            let karaka = karakas.iter().filter(|k| k.0 == g).map(|k| k.1);
                            let karakas = karaka
                                .filter(|&k| {
                                    form != 2 && matches!(k, CharaKaraka::Atma | CharaKaraka::Putra)
                                })
                                .map(|k| {
                                    LOCALES.lookup(
                                        &lang.id(),
                                        &format!("{}-short", kb::message_id("chara-karaka", k)),
                                    )
                                });
                            let lords = [1, 5]
                                .into_iter()
                                .filter(|n| form != 1 && lord_of.contains(n))
                                .map(lord);
                            karakas.chain(lords).collect()
                        }
                        _ => lord_of.into_iter().map(lord).collect(),
                    };
                    let bhava = hora_chart::bhava(lagna, v.rasi_of(g)).to_string();
                    let place = if roles.is_empty() {
                        message(lang, "yoga-graha-in", [("bhava", bhava)])
                    } else {
                        message(
                            lang,
                            "yoga-graha-roles-in",
                            [("roles", roles.join(", ")), ("bhava", bhava)],
                        )
                    };
                    (body_id(g), place)
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
                })
                .collect(),
            magnitude: yoga.magnitude.map(|m| {
                // Rounded, unlike positions: chart 6's pair, over 16′30″ apart, is the book's "17'" (p. 230), as hora-chart's golden test reads it.
                let minutes = (m.closeness + 30) / 60;
                (
                    format!("{}°{:02}′", minutes / 60, minutes % 60),
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
    format!("{}?{query}{name}={value}#{fragment}", uri.path())
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
        let special_lagna =
            r#"<li><a href="/en/kb/special-lagna">Special lagna</a> (§ 5.1–5.7)</li>"#;
        let divisional = r#"<details>
<summary><h2>Chapter 6: Divisional charts</h2></summary>
<ol id="chapter-6">"#;
        let vargas = r#"<li><a href="/en/kb/vargas">The twenty varga chakras</a> (§ 6.1–6.5)</li>"#;
        let amsabala = r#"<li><a href="/en/kb/amsabala">Amsabala</a> (§ 6.6)</li>"#;
        assert!(
            en.contains(special_lagna) && en.find(special_lagna) < en.find(divisional),
            "{en}"
        );
        assert!(en.find(divisional) < en.find(vargas), "{en}");
        assert!(en.find(vargas) < en.find(amsabala), "{en}");
        let houses = r#"<details>
<summary><h2>Chapter 7: Bhavas</h2></summary>
<ol id="chapter-7">"#;
        let significations = r#"<li><a href="/en/kb/bhava-significations">What the bhavas show</a> (§ 7.1–7.3)</li>"#;
        let categories =
            r#"<li><a href="/en/kb/bhava-categories">Kinds of bhavas</a> (§ 7.4)</li>"#;
        assert!(en.find(amsabala) < en.find(houses), "{en}");
        assert!(en.find(houses) < en.find(significations), "{en}");
        assert!(
            en.contains(categories) && en.find(significations) < en.find(categories),
            "{en}"
        );
        let karakas_chapter = r#"<details>
<summary><h2>Chapter 8: Karakas</h2></summary>
<ol id="chapter-8">"#;
        let karakas = r#"<li><a href="/en/kb/karakas">Karakas</a> (§ 8.1–8.4)</li>"#;
        assert!(en.find(categories) < en.find(karakas_chapter), "{en}");
        assert!(
            en.contains(karakas) && en.find(karakas_chapter) < en.find(karakas),
            "{en}"
        );
        let arudha_chapter = r#"<details>
<summary><h2>Chapter 9: Arudha padas</h2></summary>
<ol id="chapter-9">"#;
        let arudha_padas = r#"<li><a href="/en/kb/arudha-padas">Arudha padas</a> (§ 9.1–9.7)</li>"#;
        assert!(en.find(karakas) < en.find(arudha_chapter), "{en}");
        assert!(
            en.contains(arudha_padas) && en.find(arudha_chapter) < en.find(arudha_padas),
            "{en}"
        );
        let aspects_chapter = r#"<details>
<summary><h2>Chapter 10: Aspects and argalas</h2></summary>
<ol id="chapter-10">"#;
        let aspects = r#"<li><a href="/en/kb/aspects-and-argalas">Aspects and argalas</a> (§ 10.1–10.7)</li>"#;
        assert!(en.find(arudha_padas) < en.find(aspects_chapter), "{en}");
        assert!(
            en.contains(aspects) && en.find(aspects_chapter) < en.find(aspects),
            "{en}"
        );
        let yogas_chapter = r#"<details>
<summary><h2>Chapter 11: Yogas</h2></summary>
<ol id="chapter-11">"#;
        let yogas = r#"<li><a href="/en/kb/yogas">Yogas</a> (§ 11.1–11.11)</li>"#;
        assert!(en.find(aspects) < en.find(yogas_chapter), "{en}");
        assert!(
            en.contains(yogas) && en.find(yogas_chapter) < en.find(yogas),
            "{en}"
        );
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
        let special_lagna =
            r#"<li><a href="/hu/kb/special-lagna">Különleges lagna</a> (§ 5.1–5.7)</li>"#;
        let divisional = r#"<details>
<summary><h2>6. fejezet: Osztott chakrák</h2></summary>
<ol id="chapter-6">"#;
        let vargas = r#"<li><a href="/hu/kb/vargas">A húsz varga chakra</a> (§ 6.1–6.5)</li>"#;
        let amsabala = r#"<li><a href="/hu/kb/amsabala">Amsabala</a> (§ 6.6)</li>"#;
        assert!(
            hu.contains(special_lagna) && hu.find(special_lagna) < hu.find(divisional),
            "{hu}"
        );
        assert!(hu.find(divisional) < hu.find(vargas), "{hu}");
        assert!(hu.find(vargas) < hu.find(amsabala), "{hu}");
        let houses = r#"<details>
<summary><h2>7. fejezet: Bhavák</h2></summary>
<ol id="chapter-7">"#;
        let significations = r#"<li><a href="/hu/kb/bhava-significations">Amit a bhavák mutatnak</a> (§ 7.1–7.3)</li>"#;
        let categories =
            r#"<li><a href="/hu/kb/bhava-categories">A bhavák fajtái</a> (§ 7.4)</li>"#;
        assert!(hu.find(amsabala) < hu.find(houses), "{hu}");
        assert!(hu.find(houses) < hu.find(significations), "{hu}");
        assert!(
            hu.contains(categories) && hu.find(significations) < hu.find(categories),
            "{hu}"
        );
        let karakas_chapter = r#"<details>
<summary><h2>8. fejezet: Karakák</h2></summary>
<ol id="chapter-8">"#;
        let karakas = r#"<li><a href="/hu/kb/karakas">Karakák</a> (§ 8.1–8.4)</li>"#;
        assert!(hu.find(categories) < hu.find(karakas_chapter), "{hu}");
        assert!(
            hu.contains(karakas) && hu.find(karakas_chapter) < hu.find(karakas),
            "{hu}"
        );
        let arudha_chapter = r#"<details>
<summary><h2>9. fejezet: Arudha padák</h2></summary>
<ol id="chapter-9">"#;
        let arudha_padas = r#"<li><a href="/hu/kb/arudha-padas">Arudha padák</a> (§ 9.1–9.7)</li>"#;
        assert!(hu.find(karakas) < hu.find(arudha_chapter), "{hu}");
        assert!(
            hu.contains(arudha_padas) && hu.find(arudha_chapter) < hu.find(arudha_padas),
            "{hu}"
        );
        let aspects_chapter = r#"<details>
<summary><h2>10. fejezet: Aspektusok és argalák</h2></summary>
<ol id="chapter-10">"#;
        let aspects = r#"<li><a href="/hu/kb/aspects-and-argalas">Aspektusok és argalák</a> (§ 10.1–10.7)</li>"#;
        assert!(hu.find(arudha_padas) < hu.find(aspects_chapter), "{hu}");
        assert!(
            hu.contains(aspects) && hu.find(aspects_chapter) < hu.find(aspects),
            "{hu}"
        );
        let yogas_chapter = r#"<details>
<summary><h2>11. fejezet: Yogák</h2></summary>
<ol id="chapter-11">"#;
        let yogas = r#"<li><a href="/hu/kb/yogas">Yogák</a> (§ 11.1–11.11)</li>"#;
        assert!(hu.find(aspects) < hu.find(yogas_chapter), "{hu}");
        assert!(
            hu.contains(yogas) && hu.find(yogas_chapter) < hu.find(yogas),
            "{hu}"
        );
    }

    #[tokio::test]
    async fn the_appendix_lists_every_chart_of_the_book_in_both_languages() {
        for (lang, label) in [("en", "Chart #"), ("hu", "#. chakra")] {
            let html = body(&format!("/{lang}/kb/book-charts")).await;
            for n in 1..=96 {
                let label = label.replace('#', &n.to_string());
                if [30, 36, 45, 52].contains(&n) {
                    let cell = format!("<td>{label}</td>");
                    assert!(html.contains(&cell), "{label} is listed without a link");
                } else {
                    let link = format!(">{label}</a></td>");
                    assert!(html.contains(&link), "{label} links to its chart");
                }
            }
        }
    }

    /// The sky a golden test pins for a chart that has birth data of its own.
    async fn golden_sky(key: &str) -> Sky {
        match key {
            "1" => crate::book::chart_1_wilmington_2000().await,
            "2" => crate::book::chart_2_wilmington_2000_at_utc_minus_5().await,
            "3" => crate::book::chart_3_gwalior_1926().await,
            "4" => crate::book::chart_4_pithapuram_1968().await,
            "5" => crate::book::chart_5_aspects_exercise_2000().await,
            "6" => crate::book::chart_6_p_v_narasimha_rao_1921().await,
            "7" => crate::book::chart_7_ronald_reagan_1911().await,
            "8" => crate::book::chart_8_guess_his_livelihood_1946().await,
            "9" => crate::book::chart_9_chatrapati_shivaji_1630().await,
            "10" => crate::book::chart_10_akbar_1542().await,
            "11" => crate::book::chart_11_p_v_narasimha_rao_1921().await,
            "12" => crate::book::chart_12_sav_exercise_1958().await,
            "13" => crate::book::chart_13_an_example_1894().await,
            "14" => crate::book::chart_14_rajiv_gandhi_1944().await,
            "15" => crate::book::chart_15_satyam_gaur_1970().await,
            "15-shivam" => crate::book::chart_15_shivam_gaur_1970().await,
            "17" => crate::book::chart_17_pandit_sanjay_rath_1963().await,
            "18" => crate::book::chart_18_a_lady_1972().await,
            "19" => crate::book::chart_19_sri_navin_patnaik_1946().await,
            "20" => crate::book::chart_20_an_engineer_in_usa_1954().await,
            "21" => crate::book::chart_21_john_f_kennedy_jr_1960().await,
            "22" => crate::book::chart_22_a_lady_1954().await,
            "23" => crate::book::chart_23_dr_b_v_raman_1912().await,
            "24" => crate::book::chart_24_bill_gates_1955().await,
            "25" => crate::book::chart_25_india_1947().await,
            "26" => crate::book::chart_26_narayana_dasa_exercise_1971().await,
            "27" => crate::book::chart_27_foreign_stay_example_1970().await,
            "28" => crate::book::chart_28_a_lady_1971().await,
            "29" => crate::book::chart_29_a_lady_1969().await,
            "31" => crate::book::chart_31_an_electrical_engineer_1947().await,
            "32" => crate::book::chart_32_a_gentleman_1961().await,
            "35" => crate::book::chart_35_jayalalita_1948().await,
            "37" => crate::book::chart_37_iskcon_devotee_1971().await,
            "38" => crate::book::chart_38_sri_aurobindo_ghose_1872().await,
            "40" => crate::book::chart_40_a_male_1927().await,
            "41" => crate::book::chart_41_a_gentleman_1902().await,
            "42" => crate::book::chart_42_niryaana_shoola_exercise_1889().await,
            "43" => crate::book::chart_43_shoola_dasa_exercise_1915().await,
            "44" => crate::book::chart_44_pitri_shoola_example_1971().await,
            "48" => crate::book::chart_48_a_gentleman_1933().await,
            "49" => crate::book::chart_49_an_iskcon_devotee_1971().await,
            "50" => crate::book::chart_50_divorced_lady_1968().await,
            "51" => crate::book::chart_51_an_actor_1937().await,
            "53" => crate::book::chart_53_a_lady_1973().await,
            "54" => crate::book::chart_54_an_engineer_1970().await,
            "54-transit" => crate::book::chart_54_wedding_transit_1999().await,
            "55" => crate::book::chart_55_a_gentleman_1961().await,
            "55-transit" => crate::book::chart_55_childbirth_transit_1992().await,
            "56" => crate::book::chart_56_john_f_kennedy_jr_1960().await,
            "56-transit" => crate::book::chart_56_death_transit_1999().await,
            "58" => crate::book::chart_58_vajpayee_1926().await,
            "59" => crate::book::chart_59_vajpayee_swearing_in_1998().await,
            "61" => crate::book::chart_61_indira_gandhi_1917().await,
            "61-transit" => crate::book::chart_61_indira_gandhi_assassination_1984().await,
            "62" => crate::book::chart_62_princess_diana_1961().await,
            "66" => crate::book::chart_66_annual_2000().await,
            "67" => crate::book::chart_67_annual_1993().await,
            "68" => crate::book::chart_68_annual_1991().await,
            "69" => crate::book::chart_69_annual_1987().await,
            "70" => crate::book::chart_70_annual_1998().await,
            "71" => crate::book::chart_71_annual_1995().await,
            "73-annual" => crate::book::chart_73_indira_gandhi_annual_1976().await,
            "74-annual" => crate::book::chart_74_a_lady_annual_1998().await,
            "75" => crate::book::chart_75_rectification_example_1966().await,
            "77" => crate::book::chart_77_a_gentleman_1939().await,
            "78" => crate::book::chart_78_bill_gates_wrong_time_1955().await,
            "79" => crate::book::chart_79_a_music_teacher_1973().await,
            "80-annual" => crate::book::chart_80_india_annual_1997().await,
            "81" => crate::book::chart_81_india_lunar_year_1992().await,
            "82" => crate::book::chart_82_india_solar_year_1998().await,
            "84" => crate::book::chart_84_vajpayee_swearing_in_1996().await,
            "86" => crate::book::chart_86_writing_this_book_1997().await,
            "88" => crate::book::chart_88_an_astrologer_1949().await,
            "89" => crate::book::chart_89_annual_1999().await,
            "90" => crate::book::chart_90_an_indian_in_usa_1968().await,
            "91" => crate::book::chart_91_annual_1998().await,
            "92" => crate::book::chart_92_monthly_may_1998().await,
            "92-june" => crate::book::chart_92_monthly_june_1998().await,
            "93" => crate::book::chart_93_monthly_march_2000().await,
            "94" => crate::book::chart_94_sixty_hours_2000().await,
            "95" => crate::book::chart_95_a_lady_1968().await,
            "96" => crate::book::chart_96_tajaka_1991().await,
            key => panic!("no golden fixture for chart {key}"),
        }
    }

    #[tokio::test]
    async fn every_appendix_link_opens_the_chart_its_golden_test_pins() {
        for lang in ["en", "hu"] {
            let html = body(&format!("/{lang}/kb/book-charts")).await;
            let prefix = format!(r#"<a href="/{lang}/chart?"#);
            let links: Vec<_> = html
                .split(&prefix)
                .skip(1)
                .map(|link| {
                    let query = link[..link.find('"').unwrap()].replace("&amp;", "&");
                    format!("/{lang}/chart?{query}")
                })
                .collect();
            let linked = book_charts::CHARTS
                .iter()
                .filter(|chart| chart.request().is_some());
            assert_eq!(links.len(), linked.clone().count());
            for (chart, link) in linked.zip(links) {
                let response = get(&link, None).await;
                assert_eq!(response.status(), StatusCode::OK, "{}: {link}", chart.key);
                let uri: Uri = link.parse().unwrap();
                let Query(form) = Query::<HashMap<String, String>>::try_from_uri(&uri).unwrap();
                let Ok((request, ..)) = parse(&form, None) else {
                    panic!("{}: {link}", chart.key);
                };
                let mut source = chart;
                while let book_charts::Data::Same(key) = source.data {
                    source = book_charts::CHARTS.iter().find(|c| c.key == key).unwrap();
                }
                assert_eq!(
                    hora_swe::calculate(request).await.unwrap(),
                    golden_sky(source.key).await,
                    "{}: {link}",
                    chart.key
                );
            }
        }
    }

    #[tokio::test]
    async fn the_appendix_says_which_chart_a_repeat_shows_again_and_why_one_has_no_link() {
        let en = body("/en/kb/book-charts").await;
        assert!(en.contains("Rasi chart; the same data as Chart 7"), "{en}");
        assert!(
            en.contains("Not linked: the book prints no birth data"),
            "{en}"
        );
        let hu = body("/hu/kb/book-charts").await;
        assert!(
            hu.contains("Rasi chakra; az adatok azonosak ezzel: 7. chakra"),
            "{hu}"
        );
        assert!(hu.contains("a könyv nem közöl születési adatokat"), "{hu}");
    }

    #[tokio::test]
    async fn the_kb_index_lists_the_appendix_with_no_book_section() {
        for (lang, heading, title) in [
            ("en", "Appendix", "The book&#39;s charts"),
            ("hu", "Függelék", "A könyv chakrái"),
        ] {
            let index = body(&format!("/{lang}/kb/")).await;
            assert!(index.contains(&format!("<h2>{heading}</h2>")), "{index}");
            let link = format!(r#"<a href="/{lang}/kb/book-charts">{title}</a></li>"#);
            assert!(index.contains(&link), "{link}: {index}");
        }
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

    /// The chart table's rows, without its header links that echo the request too.
    fn tbody(page: &str) -> &str {
        let table = table(page);
        &table[table.find("<tbody>").expect("a tbody")..]
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
            assert_eq!(
                row.contains(r#"<abbr title="retrograde">"#),
                retrograde,
                "{row}"
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

    /// Tromsø, 69°39′N 18°57′E, at the winter solstice: no Vedic day.
    const TROMSO: [(&str, &str); 5] = [
        ("date", "2000-12-21"),
        ("time", "12%3A00"),
        ("offset", "%2B01%3A00"),
        ("latitude", "69.65"),
        ("longitude", "18.95"),
    ];

    /// Ch. 4: eleven upagrahas with a Vedic day, only the five Sun-based without one.
    #[tokio::test]
    async fn the_chart_page_shows_each_upagrahas_rasi_nakshatra_and_lords() {
        let tromso = with(&TROMSO);
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

    /// Ch. 5: HL, GL, SL and BL with a Vedic day, only SL without one.
    #[tokio::test]
    async fn the_chart_page_shows_each_special_lagnas_rasi_nakshatra_and_lords() {
        let tromso = with(&TROMSO);
        for (lang, query, names) in [
            (
                "en",
                HORAS_CHART,
                &["hora-lagna", "ghati-lagna", "sree-lagna", "bhaava-lagna"][..],
            ),
            (
                "hu",
                HORAS_CHART,
                &["hora-lagna", "ghati-lagna", "sree-lagna", "bhaava-lagna"],
            ),
            ("en", tromso.as_str(), &["sree-lagna"]),
        ] {
            let page = body(&format!("/{lang}/chart?{query}&style=south")).await;
            let section = section(&page, "special-lagnas");
            let kb = format!("/{lang}/kb");
            let rows: Vec<_> = section.split("<tr").skip(2).collect();
            assert_eq!(rows.len(), names.len(), "{section}");
            assert_eq!(section.contains("<p>"), names.len() == 1, "{section}");
            for (row, name) in rows.iter().zip(names) {
                for (href, count) in [
                    (format!("{kb}/special-lagna#{name}\""), 1),
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
        for name in [
            "calendar",
            "utc",
            "ayanamsa",
            "node",
            "sunrise",
            "reference",
            "style",
        ] {
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
        assert_eq!(tbody(&blank), tbody(&chart_1));
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

    /// The sky the page calculates for `query`, a test query whose only escapes are `:` and `+`.
    async fn sky_of(query: &str) -> Sky {
        let form = query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .map(|(name, value)| {
                let value = value.replace("%3A", ":").replace("%2B", "+");
                (name.to_owned(), value)
            })
            .collect();
        let (request, ..) = parse(&form, None).ok().expect(query);
        hora_swe::calculate(request).await.unwrap()
    }

    /// An English message.
    fn en(id: &str) -> String {
        LOCALES.lookup(&Lang::En.id(), id)
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
            // Twenty links, each the chart with its varga; only the shown one is current.
            assert_eq!(section.matches("#varga\"").count(), 20, "{section}");
            let current = format!(
                r#"href="/en/chart?{}&#38;varga=d{n}#varga" aria-current="page" aria-label="{heading}">D-{n}</a>"#,
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
    async fn each_body_sits_in_its_varga_rasi_in_each_style_but_hl_and_gl_not_without_a_vedic_day()
    {
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
        let polar = body(&format!("/en/chart?{}", with(&TROMSO))).await;
        let chakra = section(&polar, "varga");
        assert!(chakra.contains("<svg"), "{chakra}");
        assert!(!chakra.contains(r#"class="special""#), "{chakra}");
    }

    #[tokio::test]
    async fn an_unknown_varga_is_refused() {
        for varga in ["d13", "D9", "9", "d"] {
            let response = get(&format!("/en/chart?{CHART_1}&varga={varga}"), None).await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{varga}");
            let page = page(response).await;
            let error = r#"<strong id="varga-error" class="error">Choose one of the listed options.</strong>"#;
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
        let page = body(&format!("/en/chart?{CHART_1}")).await;
        let vargas = section(&page, "vargas");
        assert!(vargas.contains(r#"<a href="/en/kb/vargas">"#), "{vargas}");
        assert_eq!(
            vargas.matches(r#"<th scope="row">"#).count(),
            20,
            "{vargas}"
        );
        for varga in hora_chart::VARGAS {
            let n = varga.parts();
            let name = en(&kb::message_id("varga", varga));
            let mut cells = vec![format!(
                r#"<a href="/en/kb/vargas#d-{n}">{name} (D-{n})</a></th>"#
            )];
            cells.extend(points.iter().map(|&longitude| {
                let rasi = rasi_id(hora_chart::varga(varga, longitude));
                format!(
                    r#"<td><abbr title="{}">{}</abbr></td>"#,
                    en(rasi),
                    en(&format!("{rasi}-short"))
                )
            }));
            let cells: Vec<_> = cells.iter().map(String::as_str).collect();
            assert!(
                row(vargas, &cells).is_some(),
                "{varga:?}: {cells:?} {vargas}"
            );
        }
        for short in ["La", "Su", "Ke", "HL", "GL", "SL"] {
            assert!(vargas.contains(&format!(">{short}</abbr></th>")), "{short}");
        }
        let polar = body(&format!("/en/chart?{}", with(&TROMSO))).await;
        let vargas = section(&polar, "vargas");
        assert!(
            !vargas.contains(">HL<") && !vargas.contains(">GL<"),
            "{vargas}"
        );
        assert!(vargas.contains(">SL</abbr></th>"), "{vargas}");
    }

    #[tokio::test]
    async fn the_amsabala_table_counts_each_graha_over_the_four_varga_groups() {
        let chart = hora_chart::chart(&sky_of(CHART_1).await);
        for lang in [Lang::En, Lang::Hu] {
            let page = body(&format!("/{}/chart?{CHART_1}", lang.code())).await;
            let amsabala = section(&page, "amsabala");
            assert!(
                amsabala.contains(&format!(r#"<a href="/{}/kb/amsabala">"#, lang.code())),
                "{amsabala}"
            );
            let t = |id: &str| LOCALES.lookup(&lang.id(), id);
            for group in hora_chart::VARGA_GROUPS {
                let name = t(&kb::message_id("varga-group", group));
                assert!(amsabala.contains(&format!(">{name}</th>")), "{name}");
            }
            assert_eq!(amsabala.matches(r#"<th scope="row">"#).count(), 9);
            for entry in chart.grahas() {
                let mut cells = vec![format!(">{}</a></th>", t(body_id(entry.body)))];
                cells.extend(hora_chart::VARGA_GROUPS.map(|group| {
                    let (count, amsa) = hora_chart::amsabala(entry.body, entry.longitude, group);
                    let amsa = amsa.map_or("—".to_owned(), |a| t(&kb::message_id("amsa", a)));
                    format!("<td>{count} {amsa}</td>")
                }));
                let cells: Vec<_> = cells.iter().map(String::as_str).collect();
                assert!(row(amsabala, &cells).is_some(), "{cells:?}: {amsabala}");
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
    async fn bhavas_are_counted_from_a_chosen_special_lagna() {
        for (reference, name) in [
            ("hora-lagna", "Hora lagna"),
            ("ghati-lagna", "Ghati lagna"),
            ("sree-lagna", "Sree lagna"),
        ] {
            let page = body(&format!(
                "/en/chart?{}",
                with(&[("reference", reference), ("style", "north")])
            ))
            .await;
            let specials = section(&page, "special-lagnas");
            let start = specials.find(&format!(">{name}<")).expect(name);
            let rasi = specials[start..]
                .split("rasi-characteristics#")
                .nth(1)
                .and_then(|rest| rest.split_once('"'))
                .map(|(rasi, _)| rasi)
                .expect(specials);
            let number = hora_chart::RASIS
                .iter()
                .position(|r| rasi_id(*r) == format!("rasi-{rasi}"))
                .expect(rasi)
                + 1;
            let first = r#"</a></td><td><a href="/en/kb/bhava">1<"#;
            let cells = [format!(">{name}<"), first.to_owned()];
            let cells = cells.each_ref().map(String::as_str);
            assert!(row(specials, &cells).is_some(), "{reference}: {specials}");
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
        let mercury = with(&[("reference", "mercury"), ("style", "north")]);
        let mercury = body(&format!("/en/chart?{mercury}")).await;
        let lagna = body(&format!("/en/chart?{}", with(&[("style", "north")]))).await;
        assert_ne!(tbody(&page), tbody(&lagna), "Mercury is not in Virgo");
        assert_eq!(tbody(&page), tbody(&mercury));
        for id in ["special-lagnas", "upagrahas"] {
            assert_eq!(section(&page, id), section(&mercury, id), "{id}");
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

    /// Chart 3 (ch. 9 *Arudha Padas*, p. 97), at the seconds its golden test is cast with.
    const CHART_3: &str = "date=1926-12-25&time=05%3A12%3A30&utc=stated&offset=%2B05%3A30\
        &latitude=26.233333333333334&longitude=78.16666666666667";

    /// The grahas table's row of the body `id`, from its `<tr`.
    fn body_row<'a>(page: &'a str, id: &str) -> &'a str {
        let at = format!(r#" data-body="{id}">"#);
        table(page)
            .split("<tr")
            .find(|r| r.starts_with(&at))
            .expect(id)
    }

    /// The grahas table's body ids, in its order.
    fn body_order(page: &str) -> Vec<&str> {
        table(page)
            .split(r#"<tr data-body=""#)
            .skip(1)
            .filter_map(|r| r.split_once('"').map(|(id, _)| id))
            .collect()
    }

    /// An arudha table's cells of one arudha pada, as written in English: its row's start, then rasi and bhava in each chart.
    fn arudha_cells(start: &str, rasi: &str, bhava: u8, in_varga: Option<(&str, u8)>) -> String {
        let cells = |rasi: &str, bhava: u8| {
            format!(
                r##"<td><a href="/en/kb/rasi-characteristics#{}">{rasi}</a></td><td><a href="/en/kb/bhava">{bhava}</a></td>"##,
                rasi.to_lowercase()
            )
        };
        let varga = in_varga.map_or(String::new(), |(rasi, bhava)| cells(rasi, bhava));
        format!("{start}</th>{}{varga}", cells(rasi, bhava))
    }

    #[tokio::test]
    async fn the_arudha_tables_show_chart_3s_arudha_padas() {
        // Ch. 20, p. 265: "Cp contains AL and A5"; Example 32, p. 96: Jupiter's and Rahu's arudha padas in Libra. Lagna in Scorpio.
        let page = body(&format!("/en/chart?{CHART_3}")).await;
        let arudha_padas = section(&page, "arudha-padas");
        assert!(
            arudha_padas.contains(r#"<a href="/en/kb/arudha-padas">Arudha padas</a></h2>"#),
            "{arudha_padas}"
        );
        for (name, short) in [
            ("Arudha lagna", "AL"),
            ("Arudha pada of the 5th bhava", "A5"),
        ] {
            let start = format!(r#"<abbr title="{name}">{short}</abbr>"#);
            let cells = arudha_cells(&start, "Capricorn", 3, None);
            assert!(arudha_padas.contains(&cells), "{cells}: {arudha_padas}");
        }
        for (name, short) in [("Darapada", "A7"), ("Upapada lagna", "UL")] {
            assert!(
                arudha_padas.contains(&format!(r#"<abbr title="{name}">{short}</abbr>"#)),
                "{arudha_padas}"
            );
        }
        assert_eq!(
            arudha_padas.matches(r#"<th scope="row">"#).count(),
            12,
            "{arudha_padas}"
        );
        let grahas = section(&page, "graha-arudhas");
        assert!(
            grahas.contains(r#"<a href="/en/kb/arudha-padas">Graha arudhas</a></h2>"#),
            "{grahas}"
        );
        for (anchor, name) in [("jupiter", "Jupiter"), ("rahu", "Rahu")] {
            let start = format!(r##"<a href="/en/kb/graha-characteristics#{anchor}">{name}</a>"##);
            let cells = arudha_cells(&start, "Libra", 12, None);
            assert!(grahas.contains(&cells), "{cells}: {grahas}");
        }
        assert_eq!(grahas.matches(r#"<th scope="row">"#).count(), 9, "{grahas}");
        // Hover lights the row, which needs its own rule.
        for id in ["bhava-arudha-1", "arudha-graha-rahu"] {
            assert!(page.contains(&format!(r#"<tr data-body="{id}">"#)), "{id}");
            assert!(
                page.contains(&format!("[data-body={id}] {{ --lit: 1; }}")),
                "{id}"
            );
        }
        let hu = body(&format!("/hu/chart?{CHART_3}")).await;
        assert!(section(&hu, "arudha-padas").contains("Arudha padák"));
        assert!(
            section(&hu, "arudha-padas")
                .contains(r#"<abbr title="Az 5. bhava arudha padája">A5</abbr>"#)
        );
        assert!(section(&hu, "graha-arudhas").contains("Graha arudhák"));
    }

    #[tokio::test]
    async fn the_arudha_tables_varga_columns_hold_exercises_12_and_13() {
        // Chart 2: chart 1's birth at UTC−5, in D-16 (ch. 9 §9.8, p. 99); its D-16 lagna counts the varga's Bhava with the lagna reference.
        let query = with(&[("offset", "-05%3A00"), ("time", "17%3A55%3A51")]);
        let page = body(&format!("/en/chart?{query}&varga=d16")).await;
        let sky = sky_of(&query).await;
        let lagna = hora_chart::varga(Varga::Shodasamsa, hora_chart::chart(&sky).lagna().longitude);
        let d16 = "Shodasamsa (D-16)";
        let arudha_padas = section(&page, "arudha-padas");
        assert!(
            arudha_padas.contains(&format!(r#"<th scope="colgroup" colspan="2">{d16}</th>"#)),
            "{arudha_padas}"
        );
        use Rasi::*;
        for (n, rasi) in [
            Aquarius,
            Aries,
            Sagittarius,
            Scorpio,
            Sagittarius,
            Aquarius,
            Pisces,
            Virgo,
            Taurus,
            Sagittarius,
            Sagittarius,
            Aquarius,
        ]
        .into_iter()
        .enumerate()
        {
            let id = arudha_id(n);
            let row = body_row(arudha_padas, &id);
            let cell = format!(
                r##"<td><a href="/en/kb/rasi-characteristics#{}">{}</a></td><td><a href="/en/kb/bhava">{}</a></td></tr>"##,
                en(rasi_id(rasi)).to_lowercase(),
                en(rasi_id(rasi)),
                hora_chart::bhava(lagna, rasi),
            );
            assert!(row.contains(&cell), "{id}: {cell} {row}");
        }
        let grahas = section(&page, "graha-arudhas");
        assert!(
            grahas.contains(&format!(r#"<th scope="colgroup" colspan="2">{d16}</th>"#)),
            "{grahas}"
        );
        use hora_chart::Body::*;
        for (graha, rasi) in [
            (Sun, Scorpio),
            (Moon, Sagittarius),
            (Mars, Gemini),
            (Mercury, Pisces),
            (Jupiter, Scorpio),
            (Venus, Aquarius),
            (Saturn, Sagittarius),
            (Rahu, Virgo),
            (Ketu, Pisces),
        ] {
            let id = format!("arudha-{}", body_id(graha));
            let cell = format!(
                r##"<td><a href="/en/kb/rasi-characteristics#{}">{}</a></td><td><a href="/en/kb/bhava">{}</a></td></tr>"##,
                en(rasi_id(rasi)).to_lowercase(),
                en(rasi_id(rasi)),
                hora_chart::bhava(lagna, rasi),
            );
            let row = body_row(grahas, &id);
            assert!(row.contains(&cell), "{id}: {cell} {row}");
        }
    }

    /// Chart 5 (ch. 10 *Aspects and Argalas*, p. 101), at the seconds its golden test is cast with.
    fn chart_5() -> String {
        with(&[
            ("date", "2000-04-19"),
            ("time", "19%3A21%3A57"),
            ("latitude", "42.666666666666664"),
            ("longitude", "-71.13333333333334"),
        ])
    }

    /// Chart 6 (ch. 10 *Aspects and Argalas*, p. 107) as the book prints it.
    fn chart_6() -> String {
        with(&[
            ("date", "1921-06-28"),
            ("time", "12%3A49"),
            ("offset", "%2B05%3A17"),
            ("latitude", "18.433333333333334"),
            ("longitude", "79.15"),
        ])
    }

    /// The text of each cell of each row of `html`'s tbody, a `<br>` as " / ".
    fn cell_texts(html: &str) -> Vec<Vec<String>> {
        let tbody = &html[html.find("<tbody>").expect("a tbody")..];
        let tbody = &tbody[..tbody.find("</tbody>").expect("a tbody end")];
        tbody
            .split("<tr")
            .skip(1)
            .map(|row| {
                let row = row.split_once('>').expect("a row").1;
                let row = row.replace("<br>", " / ").replace("</th>", "</td>");
                let mut cells: Vec<_> = row.split("</td>").map(strip_tags).collect();
                cells.pop();
                cells
            })
            .collect()
    }

    fn strip_tags(html: &str) -> String {
        let mut text = String::new();
        for part in html.split('<') {
            text.push_str(part.split_once('>').map_or(part, |(_, after)| after));
        }
        text.trim().to_owned()
    }

    /// A section's table for the rasi chart and, in its `<details>`, for the varga shown.
    fn both_tables(section: &str) -> (&str, &str) {
        section.split_once("<details>").expect("a details")
    }

    #[tokio::test]
    async fn the_aspects_tables_hold_exercises_14_and_15_in_chart_5s_navamsa() {
        let page = body(&format!("/en/chart?{}&varga=d9", chart_5())).await;
        let aspects = section(&page, "aspects");
        assert!(
            aspects.contains(r#"<a href="/en/kb/aspects-and-argalas">Aspects</a></h2>"#),
            "{aspects}"
        );
        assert!(
            aspects.contains("<details>\n<summary>Navamsa (D-9)</summary>"),
            "{aspects}"
        );
        let (rasi_chart, navamsa) = both_tables(aspects);
        assert_eq!(cell_texts(rasi_chart).len(), 9);
        let rows = cell_texts(navamsa);
        // Ch. 10 §10.8, pp. 109–110: the rasis and houses in count order, the planets as the book lists them.
        assert_eq!(
            rows[2],
            [
                "Mars",
                "Aq (4), Ta (7), Ge (8) / Ke, Su",
                "Cp (3), Ar (6), Cn (9) / Ve, Mo",
            ]
        );
        assert_eq!(
            rows[4],
            [
                "Jupiter",
                "Cp (3), Pi (5), Ta (7) / Ve, Su",
                "Sg (2), Pi (5), Ge (8) / Me",
            ]
        );
        // Exercise 14 leaves out the nodes (ADR 0013); Exercise 15 lists them.
        assert_eq!(rows[7], ["Rahu", "—", "Li (12), Cp (3), Ar (6) / Ve, Mo"]);
        assert_eq!(rows[8], ["Ketu", "—", "Ar (6), Cn (9), Li (12) / Mo"]);
        // Moon's graha drishti falls on Libra, which holds no graha.
        assert_eq!(rows[1][1], "Li (12) / —");
        assert!(!aspects.contains("Lagna"), "{aspects}");
        // Hover lights the chakra and the grahas table through the rules their rows already have.
        assert!(
            aspects.contains(r#"<tr data-body="graha-mars">"#),
            "{aspects}"
        );
        assert!(aspects.contains(r#"<th scope="col">Graha drishti</th>"#));
        assert!(aspects.contains(r#"<th scope="col">Rasi drishti</th>"#));
        let hu = body(&format!("/hu/chart?{}&varga=d9", chart_5())).await;
        assert!(section(&hu, "aspects").contains("Aspektusok"));
    }

    #[tokio::test]
    async fn the_aspects_tables_bhavas_follow_the_reference_point_and_nothing_else_does() {
        let at = |reference: &'static str| async move {
            let page = body(&format!("/en/chart?{}&reference={reference}", chart_6())).await;
            cell_texts(both_tables(section(&page, "aspects")).0)
        };
        let (lagna, saturn) = (at("lagna").await, at("saturn").await);
        // Saturn in Leo: the Sun in Gemini aspects Sagittarius, the 5th from him.
        assert!(saturn[0][1].starts_with("Sg (5)"), "{:?}", saturn[0]);
        assert_ne!(lagna, saturn);
        let without_bhavas = |rows: Vec<Vec<String>>| {
            rows.concat()
                .join("|")
                .replace(|c: char| c.is_ascii_digit(), "")
        };
        assert_eq!(without_bhavas(lagna), without_bhavas(saturn));
    }

    #[tokio::test]
    async fn the_argala_tables_hold_exercise_16_in_chart_5s_navamsa() {
        let page = body(&format!("/en/chart?{}&varga=d9", chart_5())).await;
        let argalas = section(&page, "argalas");
        assert!(
            argalas.contains(r#"<a href="/en/kb/aspects-and-argalas">Argalas</a></h2>"#),
            "{argalas}"
        );
        assert!(
            argalas.contains("<details>\n<summary>Navamsa (D-9)</summary>"),
            "{argalas}"
        );
        let (rasi_chart, navamsa) = both_tables(argalas);
        assert_eq!(cell_texts(rasi_chart).len(), 12);
        let rows = cell_texts(navamsa);
        assert_eq!(rows.len(), 12);
        // Ch. 10 §10.8, pp. 110–111, each argala beside its obstruction: 2nd/12th, 4th/10th, 5th/9th, 11th/3rd.
        assert_eq!(
            rows[10],
            [
                "11", "Virgo", "—", "Ra", "Me", "—", "Ve", "Su", "—", "Ma, Sa"
            ]
        );
        // The book's asterisk: Ketu is in Aquarius, so its counts run in reverse.
        assert_eq!(
            rows[3],
            [
                "4",
                "Aquarius *",
                "Ve",
                "—",
                "Ma, Sa",
                "Su",
                "—",
                "—",
                "Mo",
                "Me"
            ]
        );
        assert_eq!(rows.iter().filter(|r| r[1].ends_with('*')).count(), 1);
        assert!(navamsa.contains("Counted in reverse: this rasi holds Ketu."));
        assert!(!argalas.contains("Lagna"), "{argalas}");
        // Real headers, one group per pair, for screen readers.
        for pair in ["2nd / 12th", "4th / 10th", "5th / 9th", "11th / 3rd"] {
            let header = format!(r#"<th scope="colgroup" colspan="2">{pair}</th>"#);
            assert!(rasi_chart.contains(&header), "{header}: {rasi_chart}");
        }
        assert_eq!(rasi_chart.matches(r#"<colgroup span="2">"#).count(), 5);
        assert_eq!(
            rasi_chart.matches(r#"<th scope="col">Argala</th>"#).count(),
            4
        );
        assert_eq!(
            rasi_chart
                .matches(r#"<th scope="col">Virodhargala</th>"#)
                .count(),
            4
        );
        // §10.6's several malefics are not computed (ADR 0013).
        assert!(
            argalas.contains(r##"<a href="/en/kb/aspects-and-argalas#virodhargala">"##),
            "{argalas}"
        );
        let hu = body(&format!("/hu/chart?{}&varga=d9", chart_5())).await;
        let hu = section(&hu, "argalas");
        assert!(hu.contains("Argalák"), "{hu}");
        assert!(hu.contains(r#"<th scope="colgroup" colspan="2">11. / 3.</th>"#));
        assert!(hu.contains("Visszafelé számolva"), "{hu}");
    }

    #[tokio::test]
    async fn the_varga_tables_count_from_the_reference_point_in_the_varga() {
        // Chart 5's D-9 (p. 101): the Moon in Aries, the Sun in Taurus aspecting Scorpio, the 8th from Aries.
        let page = body(&format!("/en/chart?{}&varga=d9&reference=moon", chart_5())).await;
        let argalas = cell_texts(both_tables(section(&page, "argalas")).1);
        assert_eq!(argalas[0][..2], ["1", "Aries"]);
        let aspects = cell_texts(both_tables(section(&page, "aspects")).1);
        assert!(aspects[0][1].starts_with("Sc (8)"), "{:?}", aspects[0]);
    }

    #[tokio::test]
    async fn example_35_counting_from_saturn_makes_leo_the_first_argala_row() {
        // Ch. 10 §10.7, pp. 107–108: "Mercury, Mars and Sun have an argala on Saturn, as they are in the 11th".
        let page = body(&format!("/en/chart?{}&reference=saturn", chart_6())).await;
        let rows = cell_texts(both_tables(section(&page, "argalas")).0);
        assert_eq!(rows[0][..2], ["1", "Leo"]);
        assert_eq!(rows[0][8], "Su, Ma, Me");
    }

    /// The aspects page works its example on hora's chart, counted from Saturn.
    #[tokio::test]
    async fn the_aspects_pages_example_is_horas_chart() {
        let from_saturn = HORAS_CHART.replace("reference=lagna", "reference=saturn");
        let page = body(&format!("/en/chart?{from_saturn}&style=south")).await;
        let aspects = cell_texts(both_tables(section(&page, "aspects")).0);
        assert_eq!(
            aspects[6],
            [
                "Saturn",
                "Ge (3), Li (7), Cp (10) / Mo, Ke",
                "Le (5), Sc (8), Aq (11) / Ve, Ma",
            ]
        );
        assert_eq!(
            aspects[4],
            [
                "Jupiter",
                "Le (5), Li (7), Sg (9) / Mo, Su, Me",
                "Le (5), Sc (8), Aq (11) / Ve, Ma",
            ]
        );
        assert_eq!(aspects[1][1], "Ar (1) / Ju, Sa");
        // The Moon alone aspects Saturn by graha drishti; Mars and Venus by rasi drishti.
        for (i, row) in aspects.iter().enumerate() {
            assert_eq!(row[1].contains("Ar (1)"), i == 1, "{row:?}");
            assert_eq!(
                row[2].contains("Ar (1)"),
                row[0] == "Mars" || row[0] == "Venus",
                "{row:?}"
            );
        }
        let rows = cell_texts(both_tables(section(&page, "argalas")).0);
        assert_eq!(
            rows[0],
            ["1", "Aries", "—", "—", "Ra", "Ke", "—", "Su, Me", "Ma", "—"]
        );
        // Ketu's rasi, counted in reverse.
        assert_eq!(
            rows[9],
            [
                "10",
                "Capricorn *",
                "Su, Me",
                "Ma",
                "Mo",
                "Ju, Sa",
                "—",
                "—",
                "—",
                "Ve"
            ]
        );
        // From the lagna, in Scorpio, Saturn's rasi is the 6th row, with the same grahas.
        let from_lagna = body(&format!("/en/chart?{HORAS_CHART}&style=south")).await;
        let rows = cell_texts(both_tables(section(&from_lagna, "argalas")).0);
        assert_eq!(
            rows[5],
            ["6", "Aries", "—", "—", "Ra", "Ke", "—", "Su, Me", "Ma", "—"]
        );
        for (lang, source) in [
            ("en", include_str!("../kb/en/aspects-and-argalas.md")),
            ("hu", include_str!("../kb/hu/aspects-and-argalas.md")),
        ] {
            let link = format!("(/{lang}/chart?{from_saturn}&style=south)");
            assert!(source.contains(&link), "{lang}");
        }
    }

    #[test]
    fn the_arudha_padas_page_sends_rasi_drishti_to_the_aspects_page() {
        for (lang, source) in [
            ("en", include_str!("../kb/en/arudha-padas.md")),
            ("hu", include_str!("../kb/hu/arudha-padas.md")),
        ] {
            let link = format!("(/{lang}/kb/aspects-and-argalas#rasi-drishti)");
            assert_eq!(source.matches(&link).count(), 2, "{lang}");
        }
    }

    #[tokio::test]
    async fn each_bhava_arudha_is_drawn_once_in_each_chakra_in_its_tables_rasi() {
        for style in ["south", "north", "east"] {
            let query = format!("{CHART_3}&style={style}&varga=d10");
            let page = body(&format!("/en/chart?{query}")).await;
            let chart = hora_chart::chart(&sky_of(CHART_3).await);
            let table = section(&page, "arudha-padas");
            for (chakra, d) in [
                (drawn(&page), Varga::RasiChart),
                (section(&page, "varga"), Varga::Dasamsa),
            ] {
                let arudhas = chart.in_varga(d).bhava_arudhas();
                for (n, rasi) in arudhas.into_iter().enumerate() {
                    let id = arudha_id(n);
                    let opening = format!(
                        r#"data-body="{id}" role="img" aria-label="{} in {}""#,
                        en(&id),
                        en(rasi_id(rasi))
                    );
                    assert_eq!(
                        chakra.matches(&opening).count(),
                        1,
                        "{style} {d:?} {opening}: {chakra}"
                    );
                    assert_eq!(
                        chakra.matches(&format!(r#"data-body="{id}""#)).count(),
                        1,
                        "{style} {d:?} {id}"
                    );
                    // The row's cells: rasi and bhava in the rasi chart, then in the varga.
                    let column = if d == Varga::RasiChart { 1 } else { 3 };
                    let row = body_row(table, &id);
                    let cell = row.split("<td>").nth(column).unwrap();
                    let shown = format!(">{}</a>", en(rasi_id(rasi)));
                    assert!(cell.contains(&shown), "{id} {d:?} {shown}: {row}");
                }
                // Graha arudhas are not drawn.
                assert!(!chakra.contains("arudha-graha"), "{chakra}");
            }
            for chakra in [drawn(&page), section(&page, "varga")] {
                assert!(
                    chakra.contains(r#"<tspan class="arudha-lagna">AL</tspan></text>"#),
                    "{chakra}"
                );
                assert!(
                    chakra.contains(r#"<tspan class="arudha">A7</tspan></text>"#),
                    "{chakra}"
                );
            }
            assert!(page.contains("are the bhava arudhas"), "{style}");
        }
    }

    #[tokio::test]
    async fn bhavas_are_counted_from_the_arudha_and_upapada_lagnas_each_chart_its_own() {
        let chart = hora_chart::chart(&sky_of(CHART_3).await);
        for (reference, name, n) in [
            ("arudha-lagna", "Arudha lagna", 0),
            ("upapada-lagna", "Upapada lagna", 11),
        ] {
            let query = format!("{CHART_3}&reference={reference}&style=north&varga=d10");
            let page = body(&format!("/en/chart?{query}")).await;
            let rasi = chart.in_varga(Varga::RasiChart).bhava_arudhas()[n];
            let d10 = chart.in_varga(Varga::Dasamsa).bhava_arudhas()[n];
            assert!(
                page.contains(&format!(r#"value="{reference}" selected>{name}<"#)),
                "{reference}"
            );
            // The row of the reference point itself is 1st in both charts.
            let cell = |rasi: Rasi| {
                format!(
                    r##"<td><a href="/en/kb/rasi-characteristics#{}">{}</a></td><td><a href="/en/kb/bhava">1</a></td>"##,
                    en(rasi_id(rasi)).to_lowercase(),
                    en(rasi_id(rasi)),
                )
            };
            let row = body_row(section(&page, "arudha-padas"), &arudha_id(n));
            assert!(
                row.contains(&format!("{}{}", cell(rasi), cell(d10))),
                "{reference}: {row}"
            );
            let top = |rasi: Rasi| {
                format!(
                    r#"y="222" class="mark" aria-hidden="true">{}<"#,
                    rasi as u8 + 1
                )
            };
            assert!(drawn(&page).contains(&top(rasi)), "{reference}");
            assert!(section(&page, "varga").contains(&top(d10)), "{reference}");
            // Every other table counts from it too: the lagna's row, Scorpio from Capricorn is 11th.
            let lagna = hora_chart::bhava(rasi, Rasi::Scorpio);
            assert!(
                body_row(&page, "lagna").contains(&format!(r#"<a href="/en/kb/bhava">{lagna}<"#)),
                "{reference}"
            );
        }
        // The varga chakra's 1st bhava is D-10's AL: Virgo, from a count (p. 303).
        let page = body(&format!(
            "/en/chart?{CHART_3}&reference=arudha-lagna&style=north&varga=d10"
        ))
        .await;
        assert!(section(&page, "varga").contains(r#"y="222" class="mark" aria-hidden="true">6<"#));
        // AL in Capricorn: the rasi chakra's 1st bhava is 10.
        assert!(drawn(&page).contains(r#"y="222" class="mark" aria-hidden="true">10<"#));
    }

    #[tokio::test]
    async fn the_grahas_table_shows_chart_3s_chara_karakas() {
        // Chart 3: AK Mercury and PK Saturn (ch. 8 §8.2, p. 265).
        for lang in ["en", "hu"] {
            let page = body(&format!("/{lang}/chart?{CHART_3}")).await;
            for (id, anchor, name, short) in [
                ("graha-mercury", "atma-karaka", "Atma karaka", "AK"),
                ("graha-saturn", "putra-karaka", "Putra karaka", "PK"),
            ] {
                let cell = format!(
                    r#"</th><td><a href="/{lang}/kb/karakas#{anchor}"><abbr title="{name}">{short}</abbr></a></td>"#
                );
                assert!(body_row(&page, id).contains(&cell), "{lang} {id}");
            }
            for id in ["lagna", "graha-ketu"] {
                assert!(
                    body_row(&page, id).contains("</th><td>—</td>"),
                    "{lang} {id}"
                );
            }
        }
    }

    #[tokio::test]
    async fn the_grahas_table_sorts_by_graha_or_by_chara_karaka() {
        let by_graha = [
            "lagna",
            "graha-sun",
            "graha-moon",
            "graha-mars",
            "graha-mercury",
            "graha-jupiter",
            "graha-venus",
            "graha-saturn",
            "graha-rahu",
            "graha-ketu",
        ];
        // Chart 3's karakas from its golden positions: Me, Ve, Ra, Mo, Ma, Sa, Su, Ju (ch. 8 §8.2).
        let by_karaka = [
            "lagna",
            "graha-mercury",
            "graha-venus",
            "graha-rahu",
            "graha-moon",
            "graha-mars",
            "graha-saturn",
            "graha-sun",
            "graha-jupiter",
            "graha-ketu",
        ];
        for (sort, order, active) in [
            ("", by_graha, 0),
            ("&sort=graha", by_graha, 0),
            ("&sort=karaka", by_karaka, 1),
        ] {
            let query = format!("{CHART_3}{sort}");
            let page = body(&format!("/en/chart?{query}")).await;
            assert_eq!(body_order(&page), order, "{sort}");
            let start = page.find(r#"<table id="grahas">"#).unwrap();
            let head = &page[start..start + page[start..].find("</thead>").unwrap()];
            let headers: Vec<_> = head.split("<th ").skip(1).collect();
            let sorted: Vec<_> = headers
                .iter()
                .map(|h| h.contains(r#"aria-sort="ascending""#))
                .collect();
            assert_eq!(sorted.iter().filter(|s| **s).count(), 1, "{sort}");
            let (graha, karaka) = (headers[0], headers[1]);
            assert!([graha, karaka][active].contains("aria-sort"), "{sort}");
            // Askama escapes `&` in attributes.
            let base = query.replace("&sort=graha", "").replace("&sort=karaka", "");
            for (header, value) in [(graha, "graha"), (karaka, "karaka")] {
                let href = format!(r#"href="/en/chart?{base}&sort={value}#grahas""#);
                let href = href.replace('&', "&#38;");
                assert!(header.contains(&href), "{sort}: {header}");
            }
            let hidden = sort.trim_start_matches("&sort=");
            assert!(page.contains(&format!(
                r#"<input type="hidden" name="sort" value="{hidden}">"#
            )));
        }
    }

    #[tokio::test]
    async fn an_unknown_sort_is_refused_and_dropped_from_the_form() {
        for sort in ["rasi", "Karaka", "1"] {
            let response = get(&format!("/en/chart?{CHART_1}&sort={sort}"), None).await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{sort}");
            let page = page(response).await;
            let error = r#"<strong id="sort-error" class="error">Choose one of the listed options.</strong>"#;
            assert!(page.contains(error), "{sort}: {page}");
            assert!(page.contains(r#"<input type="hidden" name="sort" value="">"#));
            assert!(!page.contains("<table"), "{page}");
        }
    }

    #[tokio::test]
    async fn bhavas_are_counted_from_the_atma_karaka() {
        // Chart 1's AK is the Sun, 26°29′ Pisces (ch. 9 *Arudha Padas*, p. 87; ch. 8 §8.2).
        let query = with(&[("reference", "atma-karaka"), ("style", "north")]);
        let sky = sky_of(&query).await;
        let karakas = hora_chart::chart(&sky).chara_karakas();
        assert!(karakas.contains(&(hora_chart::Body::Sun, CharaKaraka::Atma)));
        let page = body(&format!("/en/chart?{query}")).await;
        let sun = with(&[("reference", "sun"), ("style", "north")]);
        let sun = body(&format!("/en/chart?{sun}")).await;
        assert_eq!(tbody(&page), tbody(&sun));
        assert!(body_row(&page, "lagna").contains(r#"<a href="/en/kb/bhava">7<"#));
        for id in ["special-lagnas", "upagrahas"] {
            assert_eq!(section(&page, id), section(&sun, id), "{id}");
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
        let en = keys(include_str!("../locales/en/main.ftl"));
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
    async fn a_refused_form_keeps_the_chosen_sunrise_definition() {
        let query = with(&[("latitude", "95"), ("sunrise", "centre-true")]);
        let response = get(&format!("/en/chart?{query}"), None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let page = page(response).await;
        assert!(page.contains(r#"value="centre-true" selected"#), "{page}");
    }

    /// Chart 9 (ch. 11 *Yogas*, p. 128), at the date, time and seconds its golden test is cast with.
    fn chart_9() -> String {
        with(&[
            ("date", "1630-02-19"),
            ("time", "18%3A25%3A32"),
            ("calendar", "julian"),
            ("utc", "local-mean"),
            ("offset", ""),
            ("latitude", "18.533333333333335"),
            ("longitude", "73.88333333333334"),
        ])
    }

    /// Chart 10 (ch. 11 *Yogas*, p. 137), entered as the Julian date the book also prints.
    fn chart_10() -> String {
        with(&[
            ("date", "1542-11-24"),
            ("time", "03%3A39"),
            ("calendar", "julian"),
            ("utc", "local-mean"),
            ("offset", ""),
            ("latitude", "25.316666666666666"),
            ("longitude", "69.78333333333333"),
        ])
    }

    /// Chart 18 (ch. 16 *Vimsottari Dasa*, p. 217) as the book prints it.
    fn chart_18() -> String {
        with(&[
            ("date", "1972-06-01"),
            ("time", "04%3A16"),
            ("offset", "%2B05%3A30"),
            ("latitude", "16.25"),
            ("longitude", "81.2"),
        ])
    }

    /// Chart 90 (ch. 36, p. 493), at the seconds its golden test is cast with.
    fn chart_90() -> String {
        with(&[
            ("date", "1968-01-25"),
            ("time", "11%3A14%3A45"),
            ("offset", "%2B05%3A30"),
            ("latitude", "17.483333333333334"),
            ("longitude", "78.5"),
        ])
    }

    /// Every row of a yoga table, across its class groups; a class's heading is a row of one cell.
    fn yoga_texts(table: &str) -> Vec<Vec<String>> {
        table
            .split("<tbody>")
            .skip(1)
            .flat_map(|group| cell_texts(&format!("<tbody>{group}")))
            .collect()
    }

    /// The row of a yoga table whose first cell is `name`.
    fn yoga_row(rows: &[Vec<String>], name: &str) -> Vec<String> {
        let row = rows.iter().find(|row| row[0] == name);
        row.unwrap_or_else(|| panic!("no {name} in {rows:?}"))
            .clone()
    }

    #[tokio::test]
    async fn chart_10_lists_its_mahaapurusha_yogas_and_the_raaja_yoga_of_venus_and_saturn() {
        let page = body(&format!("/en/chart?{}", chart_10())).await;
        let yogas = section(&page, "yogas");
        assert!(
            yogas.contains(r#"<a href="/en/kb/yogas">Yogas</a></h2>"#),
            "{yogas}"
        );
        assert!(
            yogas.contains(r#"<th scope="rowgroup" colspan="5"><a href="/en/kb/mahaapurusha-yogas">Mahaapurusha yogas</a></th>"#),
            "{yogas}"
        );
        assert!(
            yogas.contains(r##"<a href="/en/kb/yogas#checked">"##),
            "{yogas}"
        );
        let rows = yoga_texts(both_tables(yogas).0);
        // Ch. 11 §11.7.2, p. 137: "Mahapurusha yogas (Maalavya and Sasa)" and "another Mahapurusha Yoga – Ruchaka Yoga".
        for (name, graha) in [
            ("Maalavya yoga", "Venus"),
            ("Sasa yoga", "Saturn"),
            ("Ruchaka yoga", "Mars"),
        ] {
            let row = yoga_row(&rows, name);
            assert!(row[1].starts_with(&format!("{graha} (")), "{row:?}");
            assert_eq!(row[2..], ["—", "—", "—"], "{row:?}");
        }
        // P. 137: Venus and Saturn "less than a degree apart"; their printed amsas are not golden (ADR 0014).
        let row = rows.iter().find(|row| {
            row[0] == "Raaja yoga" && row[1] == "Venus (1L, 8L, in 1), Saturn (4L, 5L, in 1)"
        });
        let row = row.unwrap_or_else(|| panic!("{rows:?}"));
        assert!(row[3].starts_with("0°"), "{row:?}");
        let amsas: Vec<_> = row[4].split(" / ").collect();
        assert!(
            amsas.len() == 2 && amsas[0].starts_with("Venus: ") && amsas[1].starts_with("Saturn: "),
            "{row:?}"
        );
    }

    #[tokio::test]
    async fn chart_6s_vipareeta_raaja_yoga_of_mars_and_the_sun_is_17_apart() {
        let page = body(&format!("/en/chart?{}", chart_6())).await;
        let rows = yoga_texts(both_tables(section(&page, "yogas")).0);
        // Ch. 17, p. 230: Mars and the Sun "within a 17' from each other".
        let row = rows
            .iter()
            .find(|row| row[0] == "Vipareeta Raaja yoga" && row[1].starts_with("Sun ("));
        assert_eq!(row.map(|row| row[3].as_str()), Some("0°17′"), "{rows:?}");
    }

    #[tokio::test]
    async fn chart_18s_saptamsa_lists_the_raaja_yoga_of_venus_and_rahu() {
        let page = body(&format!("/en/chart?{}&varga=d7", chart_18())).await;
        let yogas = section(&page, "yogas");
        assert!(
            yogas.contains("<details>\n<summary>Saptamsa (D-7)</summary>"),
            "{yogas}"
        );
        let saptamsa = both_tables(yogas).1;
        // A varga's table has no magnitude columns (ADR 0014).
        assert!(!saptamsa.contains("Closeness"), "{saptamsa}");
        // Ch. 16, p. 218: Rahu "acts as the 5th lord", "with lagna lord Venus. His conjunction with Venus – conjunction of 1st and
        // 5th lords – consists of a raja yoga", which is also §11.7.3 (1)'s second half.
        let rows = yoga_texts(saptamsa);
        for (name, grahas) in [
            ("Raaja yoga", "Venus (1L, 8L, in 11), Rahu (5L, in 11)"),
            (
                "Raaja yoga §11.7.3 (1)",
                "Venus (1L, in 11), Rahu (5L, in 11)",
            ),
        ] {
            let row = rows.iter().find(|row| row[0] == name && row[1] == grahas);
            assert!(row.is_some_and(|row| row[2] == "—"), "{name}: {rows:?}");
        }
    }

    #[tokio::test]
    async fn chart_3s_raaja_yoga_1_names_its_ak_and_pk() {
        // Ch. 20, p. 265: "a powerful raja yoga between AK (Mercury) and PK (Saturn)", in the lagna.
        for (lang, grahas) in [
            ("en", "Mercury (AK, in 1), Saturn (PK, in 1)"),
            (
                "hu",
                "Merkúr (AK, 1. bhavában), Szaturnusz (PK, 1. bhavában)",
            ),
        ] {
            let page = body(&format!("/{lang}/chart?{CHART_3}")).await;
            let rows = yoga_texts(both_tables(section(&page, "yogas")).0);
            assert_eq!(yoga_row(&rows, "Raaja yoga §11.7.3 (1)")[1], grahas);
        }
    }

    #[tokio::test]
    async fn chart_90s_dasamsa_has_the_vipareeta_raaja_yoga_of_the_moon_and_jupiter_to_judge() {
        let page = body(&format!("/en/chart?{}&varga=d10", chart_90())).await;
        let dasamsa = both_tables(section(&page, "yogas")).1;
        // Real-life Example 3, p. 494: the 8th and 12th lords conjoined in a kendra, effective only "in strength".
        let rows = yoga_texts(dasamsa);
        let row = rows
            .iter()
            .find(|row| row[0] == "Vipareeta Raaja yoga" && row[1].starts_with("Moon ("));
        let row = row.unwrap_or_else(|| panic!("{rows:?}"));
        assert!(row[1].contains("Jupiter ("), "{row:?}");
        assert_eq!(row[2], "Whether Moon is strong / Whether Jupiter is strong");
        assert!(
            dasamsa.contains(r#"<a href="/en/kb/raaja-yogas">Whether Moon is strong</a>"#),
            "{dasamsa}"
        );
        let hu = body(&format!("/hu/chart?{}&varga=d10", chart_90())).await;
        let rows = yoga_texts(both_tables(section(&hu, "yogas")).1);
        assert!(
            rows.iter().any(|row| row
                .get(2)
                .is_some_and(|c| c == "Erős-e a Hold / Erős-e a Jupiter")),
            "{rows:?}"
        );
    }

    #[tokio::test]
    async fn the_rasi_only_yogas_are_never_in_the_vargas_table() {
        let page = body(&format!("/en/chart?{}&varga=d9", chart_9())).await;
        let (rasi_chart, navamsa) = both_tables(section(&page, "yogas"));
        // Ch. 11 §11.6, pp. 128–129: Kalpadruma mixes the rasi chart and the navamsa (ADR 0014).
        assert!(rasi_chart.contains("Kalpadruma yoga"), "{rasi_chart}");
        for name in [
            "Kalpadruma",
            "Vishnu",
            "Gouri",
            "Chandikaa",
            "Bhaarathi",
            "Vidyut",
            "Jaya",
        ] {
            assert!(!navamsa.contains(&format!(">{name} yoga")), "{name}");
        }
        for numbered in [
            "Raaja yoga §11.7.3 (9)",
            "Daridra yoga §11.10 (8)",
            "Daridra yoga §11.10 (9)",
        ] {
            assert!(!navamsa.contains(numbered), "{numbered}");
        }
    }

    #[tokio::test]
    async fn the_yoga_tables_count_from_the_lagna_whatever_the_reference() {
        for chart in [chart_10(), chart_90()] {
            let from_lagna = body(&format!("/en/chart?{chart}&varga=d10")).await;
            let from_moon = body(&format!("/en/chart?{chart}&varga=d10&reference=moon")).await;
            assert_eq!(section(&from_lagna, "yogas"), section(&from_moon, "yogas"));
        }
    }

    /// Every yoga with a name of its own, without the numbered entries of §11.7.3–11.10.
    fn named_yogas() -> Vec<hora_chart::YogaName> {
        use hora_chart::YogaName::*;
        vec![
            Vesi,
            Vosi,
            Ubhayachara,
            BudhaAaditya,
            Sunaphaa,
            Anaphaa,
            Duradhara,
            Kemadruma,
            ChandraMangala,
            Adhi,
            Ruchaka,
            Bhadra,
            Sasa,
            Maalavya,
            Hamsa,
            Rajju,
            Musala,
            Nala,
            Maalaa,
            Sarpa,
            Gadaa,
            Sakata,
            Vihanga,
            Sringaataka,
            Hala,
            Vajra,
            Yava,
            Kamala,
            Vaapi,
            Yoopa,
            Sara,
            Sakti,
            Danda,
            Naukaa,
            Koota,
            Chatra,
            Chaapa,
            Ardhachandra,
            Chakra,
            Samudra,
            Veenaa,
            Daama,
            Paasa,
            Kedaara,
            Soola,
            Yuga,
            Gola,
            Subha,
            Asubha,
            GajaKesari,
            GuruMangala,
            Amala,
            Parvata,
            Kaahala,
            Chaamara,
            Sankha,
            Bheri,
            Mridanga,
            Sreenaatha,
            Matsya,
            Koorma,
            Khadga,
            Kusuma,
            Kalaanidhi,
            Kalpadruma,
            Lagnaadhi,
            Hari,
            Hara,
            Brahma,
            Vishnu,
            Siva,
            Trilochana,
            Gouri,
            Chandikaa,
            Lakshmi,
            Saarada,
            Bhaarathi,
            Saraswathi,
            Amsaavatara,
            Devendra,
            Indra,
            Ravi,
            Bhaaskara,
            Kulavardhana,
            Vasumati,
            Gandharva,
            Go,
            Vidyut,
            Chapa,
            Pushkala,
            Makuta,
            Jaya,
            Harsha,
            Sarala,
            Vimala,
            BasicRaaja,
            DharmaKarmadhipati,
            VipareetaRaaja,
            BasicDhana,
        ]
    }

    #[test]
    fn every_yoga_class_and_condition_has_a_name() {
        use hora_chart::{YogaClass, YogaName};
        let en = keys(include_str!("../locales/en/main.ftl"));
        let names = named_yogas();
        let classes = [
            YogaClass::Ravi,
            YogaClass::Chandra,
            YogaClass::Mahaapurusha,
            YogaClass::Naabhasa,
            YogaClass::OtherPopular,
            YogaClass::Raaja,
            YogaClass::RaajaSambandha,
            YogaClass::Dhana,
            YogaClass::Daridra,
        ];
        let numbered =
            ["raaja", "raaja-sambandha", "dhana", "daridra"].map(|class| format!("yoga-{class}-n"));
        let conditions = [
            "mercurys-nature",
            "may-not-operate-well",
            "strong",
            "combust",
            "deep-exaltation",
            "malefic-in-upachaya",
        ]
        .map(|c| format!("yoga-to-judge-{c}"));
        let missing: Vec<_> = names
            .into_iter()
            .map(|n: YogaName| kb::message_id("yoga", n))
            .chain(classes.map(|c| kb::message_id("yoga-class", c)))
            .chain(numbered)
            .chain(conditions)
            .filter(|id| !en.contains(id.as_str()))
            .collect();
        assert_eq!(missing, [""; 0]);
    }

    /// Every ch. 11 entry under its own heading: hora's 99 named yogas (the basic Dhana among them) and every numbered entry, Raaja (18) and Daridra (10) included though not computed.
    #[tokio::test]
    async fn every_yoga_has_a_section_on_its_class_page_in_both_languages() {
        use hora_chart::YogaName as N;
        let named = named_yogas().into_iter().map(|n| {
            let id = kb::message_id("yoga", n);
            (kb::yoga_page(n.class()), id["yoga-".len()..].to_owned())
        });
        let numbered = [
            (N::Raaja(1), "raaja", 18),
            (N::RaajaSambandha(1), "raaja-sambandha", 15),
            (N::Dhana(1), "dhana", 12),
            (N::Daridra(1), "daridra", 13),
        ]
        .into_iter()
        .flat_map(|(n, prefix, last)| {
            (1..=last).map(move |i| (kb::yoga_page(n.class()), format!("{prefix}-{i}")))
        });
        let anchors: Vec<_> = named.chain(numbered).collect();
        assert_eq!(anchors.len(), 99 + 18 + 15 + 12 + 13);
        for lang in ["en", "hu"] {
            let overview = body(&format!("/{lang}/kb/yogas")).await;
            assert!(overview.contains(r#"id="checked""#), "{lang}");
            let mut pages = std::collections::HashMap::new();
            for (page, _) in &anchors {
                if !pages.contains_key(page) {
                    pages.insert(*page, body(&format!("/{lang}/kb/{page}")).await);
                }
            }
            let missing: Vec<_> = anchors
                .iter()
                .filter(|(page, id)| !pages[page].contains(&format!(r#"id="{id}""#)))
                .collect();
            assert!(missing.is_empty(), "{lang}: {missing:?}");
        }
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
        let tromso = with(&TROMSO);
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
        let tromso = with(&TROMSO);
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
            tbody(&body(&format!("/en/chart?{comma}")).await),
            tbody(&body(&format!("/en/chart?{CHART_1}")).await)
        );
    }

    #[tokio::test]
    async fn local_mean_time_is_the_stated_offset_of_its_longitude() {
        // 71°12′ W is 71.2 × 4 min = 4 h 44 min 48 s behind UT.
        let stated = with(&[("offset", "-04%3A44%3A48")]);
        let mean = with(&[("utc", "local-mean"), ("offset", "")]);
        assert_eq!(
            tbody(&body(&format!("/en/chart?{mean}")).await),
            tbody(&body(&format!("/en/chart?{stated}")).await)
        );
        let minus = with(&[("offset", "%E2%88%9204%3A00")]);
        assert_eq!(
            tbody(&body(&format!("/en/chart?{minus}")).await),
            tbody(&body(&format!("/en/chart?{CHART_1}")).await)
        );
        // The comparison would pass vacuously if the offset did not move the chart.
        assert_ne!(
            tbody(&body(&format!("/en/chart?{mean}")).await),
            tbody(&body(&format!("/en/chart?{CHART_1}")).await)
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

    #[test]
    fn every_varga_has_a_name() {
        let en = keys(include_str!("../locales/en/main.ftl"));
        let missing: Vec<_> = hora_chart::VARGAS
            .map(|v| kb::message_id("varga", v))
            .into_iter()
            .filter(|id| !en.contains(id.as_str()))
            .collect();
        assert_eq!(missing, [""; 0]);
    }

    #[test]
    fn every_varga_group_and_amsa_has_a_name() {
        let en = keys(include_str!("../locales/en/main.ftl"));
        let missing: Vec<_> = hora_chart::VARGA_GROUPS
            .into_iter()
            .flat_map(|g| {
                [kb::message_id("varga-group", g)]
                    .into_iter()
                    .chain(g.amsas().iter().map(|a| kb::message_id("amsa", a)))
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

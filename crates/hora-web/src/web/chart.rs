use super::*;

/// The offset is stated, so the hour a clock change repeats is not refused as ambiguous.
pub(crate) fn moment(lang: Lang, site: &Site, uri: &Uri) -> Option<Response> {
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
pub(crate) struct ChartView {
    /// The grahas table's rows: the lagna and the grahas, in graha or chara karaka order.
    pub(crate) grahas: Vec<Row>,
    /// Its bhava arudhas' rows, AL to UL (ch. 9 §9.2).
    pub(crate) bhava_arudhas: Vec<ArudhaRow>,
    /// Its graha arudhas' rows, the Sun to Ketu (ch. 9 §9.5).
    pub(crate) graha_arudhas: Vec<ArudhaRow>,
    /// The rasi chart.
    pub(crate) rasi: ChakraView,
    /// The varga chakra shown.
    pub(crate) varga: ChakraView,
    /// The varga chakra shown, by name, heading the arudha tables' columns for it.
    pub(crate) varga_name: String,
    /// The n of the varga chakra shown (ADR 0003).
    pub(crate) parts: u8,
    /// Its upagrahas' rows, each with its section on the upagraha page.
    pub(crate) upagrahas: Vec<(&'static str, Row)>,
    /// Its special lagnas' rows, HL, GL, SL and BL, or SL alone without a Vedic day (ch. 5).
    pub(crate) special_lagnas: Vec<Row>,
    /// The ayanamsa its positions are measured with.
    pub(crate) ayanamsa: String,
    /// (message id, clock time) for its Vedic day's sunrise, sunset and next sunrise; `None` where there is none (polar latitudes).
    pub(crate) vedic_day: Option<[(&'static str, String); 3]>,
    /// Its panchaanga at the birth; vaara and hora only with a Vedic day.
    pub(crate) panchaanga: Vec<Limb>,
    /// When its tithi, nakshatra, Sun-Moon yoga and karana started and end.
    pub(crate) limb_spans: Vec<LimbSpan>,
    /// Whether the grahas table is in chara karaka order, else graha order.
    pub(crate) by_karaka: bool,
    /// The chart sorted by graha and by chara karaka, at the grahas table.
    pub(crate) sort_hrefs: [String; 2],
    /// The Vargas table's columns, (name, short) message ids: the lagna, the grahas, then HL and GL with a Vedic day, and SL.
    pub(crate) varga_columns: Vec<(String, String)>,
    /// Each varga's row of the Vargas table and link in the link row, D-1 to D-60.
    pub(crate) vargas: Vec<VargaRow>,
    /// Each graha's (message id, (count, amsa message id) in each varga group) (ch. 6 §6.6).
    pub(crate) amsabala: Vec<(&'static str, [AmsabalaCell; 4])>,
    /// Whether its stated offset is far from the place's local time, so its vaara and hora may be a day off.
    pub(crate) far_offset: bool,
    /// Rudra, Maheswara and both longevity methods, in the rasi chart only (ch. 14, ADR 0017).
    pub(crate) longevity: LongevityView,
    /// The avasthas, stronger lords and sthira karakas, in the rasi chart only (ch. 15, ch. 8 §8.3, ADR 0018).
    pub(crate) strength: StrengthView,
    /// The topic shown.
    pub(crate) topic: &'static Topic,
    /// The contents: every topic, in `TOPICS` order.
    pub(crate) contents: Vec<TopicLink>,
    /// The topics before and after the one shown, as (label message id, href).
    pub(crate) neighbours: [Option<(&'static str, String)>; 2],
}

/// A topic's entry in the contents.
pub(crate) struct TopicLink {
    pub(crate) label: &'static str,
    pub(crate) chapters: String,
    pub(crate) href: String,
    /// What it holds at a glance, or empty.
    pub(crate) fact: String,
    pub(crate) current: bool,
}

/// What one varga chakra shows, bhavas counted from the reference point's rasi in it.
pub(crate) struct ChakraView {
    /// What each graha aspects, the Sun to Ketu (ch. 10 §10.2–10.3).
    pub(crate) aspects: Vec<AspectRow>,
    /// The argalas on each rasi (ch. 10 §10.5–10.6).
    pub(crate) argalas: Vec<ArgalaRow>,
    /// The yogas by class, counted from its lagna (ch. 11).
    pub(crate) yogas: Vec<YogaGroup>,
    /// Its ashtakavarga, counted from its lagna (ch. 12).
    pub(crate) ashtakavarga: Ashtakavarga,
    /// Each bhava's baadhakas, from the bhava the reference point is in (ch. 13 §13.3).
    pub(crate) baadhakas: Vec<BaadhakaRow>,
    /// Its yogadas, from its lagna, HL and GL (ch. 13 §13.2).
    pub(crate) yogadas: Vec<YogadaRow>,
    /// Its maraka sthanas, the 2nd and 7th from its lagna, each with its bhava (ch. 14 §14.2).
    pub(crate) maraka_sthanas: [(&'static str, u8); 2],
    /// Its maraka grahas, each with its reasons in words (ch. 14 §14.2).
    pub(crate) marakas: Vec<(&'static str, Vec<String>)>,
    /// It drawn in the chosen style.
    pub(crate) drawing: chakra::Drawing,
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
pub(crate) struct Shown {
    /// The name sound, 1–5 (Table 37), if given.
    pub(crate) sound: Option<u8>,
    /// The bhavas' reference point.
    pub(crate) reference: ReferencePoint,
    pub(crate) style: chakra::Style,
    /// The varga chakra shown beside the rasi chart.
    pub(crate) varga: Varga,
    /// Whether the grahas table is in chara karaka order.
    pub(crate) by_karaka: bool,
    /// The index in `TOPICS` of the topic shown.
    pub(crate) topic: usize,
}

impl ChartView {
    /// What the chart page shows of `sky`, calculated for `request` and reached at `uri`; or the message id of a reference
    /// point `sky` does not have.
    pub(crate) fn new(
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

pub(crate) async fn chart_page(
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

#[cfg(test)]
mod tests;

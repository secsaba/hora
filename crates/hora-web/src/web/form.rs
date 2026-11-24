use super::*;

/// A plain choice field's options in form order: the value sent, the choice, the label's message id; the first is the default.
pub(crate) type Choices<T> = [(&'static str, T, &'static str)];

pub(crate) const CALENDARS: &Choices<Calendar> = &[
    ("gregorian", Calendar::Gregorian, "calendar-gregorian"),
    ("julian", Calendar::Julian, "calendar-julian"),
];

pub(crate) const AYANAMSAS: &Choices<Ayanamsa> = &[
    ("lahiri", Ayanamsa::Lahiri, "ayanamsa-lahiri"),
    ("true-chitra", Ayanamsa::TrueChitra, "ayanamsa-true-chitra"),
    ("raman", Ayanamsa::Raman, "ayanamsa-raman"),
    ("true-pushya", Ayanamsa::TruePushya, "ayanamsa-true-pushya"),
    ("true-mula", Ayanamsa::TrueMula, "ayanamsa-true-mula"),
];

pub(crate) const NODES: &Choices<NodeType> = &[
    ("mean", NodeType::Mean, "node-mean"),
    ("true", NodeType::True, "node-true"),
];

pub(crate) const SUNRISES: &Choices<SunriseDefinition> = &[
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
pub(crate) const REFERENCES: &Choices<ReferencePoint> = &[
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
pub(crate) const SOUNDS: &Choices<Option<u8>> = &[
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
pub(crate) struct Topic {
    /// The `topic` sent.
    pub(crate) key: &'static str,
    /// Its label's message id.
    pub(crate) label: &'static str,
    /// The book's chapters it covers.
    pub(crate) chapters: &'static str,
}

/// The chart page's topics in the book's order; the first is shown when none is asked for.
pub(crate) const TOPICS: [Topic; 11] = {
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
pub(crate) const STYLES: &Choices<chakra::Style> = &[
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

pub(crate) fn field<'a>(form: &'a HashMap<String, String>, name: &str) -> &'a str {
    form.get(name).map_or("", |v| v.trim())
}

/// The field and message id of a rule `hora_swe` reports broken.
pub(crate) fn field_error(rule: Rule) -> (&'static str, &'static str) {
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
pub(crate) struct Refusal {
    /// Field name → message id.
    pub(crate) errors: BTreeMap<&'static str, &'static str>,
    /// The offsets to choose from when the birth time fell in a clock change of the place's zone.
    pub(crate) offsets: Vec<i32>,
    /// When the zone refused the date or time: the request with a stand-in offset, for `hora_swe::validate` to name the other fields' broken rules.
    pub(crate) probe: Option<Request>,
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
pub(crate) fn parse(
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
pub(crate) fn offset_text(seconds: i32) -> String {
    let sign = if seconds < 0 { '−' } else { '+' };
    let s = seconds.unsigned_abs();
    let (hour, minute, second) = (s / 3600, s / 60 % 60, s % 60);
    match second {
        0 => format!("{sign}{hour:02}:{minute:02}"),
        _ => format!("{sign}{hour:02}:{minute:02}:{second:02}"),
    }
}

/// 2 to 100 letters, digits, spaces and the marks place names use; nothing else reaches the search.
pub(crate) fn place_query(value: &str) -> bool {
    (2..=100).contains(&value.chars().count())
        && value
            .chars()
            .all(|c| c.is_alphanumeric() || " -'’.".contains(c))
}

/// A search result as listed: the name matched (with the primary name when it differs), country, zone and coordinates, which tell same-named places apart.
pub(crate) fn place_label(
    lang: Lang,
    places: &Places,
    place: &place::Place,
    matched: &str,
) -> String {
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
pub(crate) fn message<const N: usize>(
    lang: Lang,
    id: &str,
    args: [(&'static str, String); N],
) -> String {
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
pub(crate) fn decimal(value: &str) -> Option<f64> {
    // Hungarian writes a decimal comma, and phones offer one.
    value.replace(',', ".").parse().ok()
}

#[cfg(test)]
mod tests;

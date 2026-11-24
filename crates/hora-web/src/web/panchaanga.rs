use super::*;

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

pub(crate) fn vedic_day_rows(
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
pub(crate) struct LimbSpan {
    pub(crate) name: &'static str,
    pub(crate) start: String,
    pub(crate) end: String,
}

/// When the birth's tithi, nakshatra, Sun-Moon yoga and karana started and end (ch. 1 §1.3.12).
pub(crate) fn limb_spans(lang: Lang, birth: &Request, limbs: &hora_swe::Limbs) -> Vec<LimbSpan> {
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
pub(crate) struct Limb {
    pub(crate) name: &'static str,
    pub(crate) value: String,
    pub(crate) lord: Option<&'static str>,
}

/// The panchaanga at the birth (ch. 1 §1.3.12), the hora and the lunar month, each limb only when its inputs exist.
pub(crate) fn panchaanga(lang: Lang, sky: &Sky) -> Vec<Limb> {
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
pub(crate) fn far_offset(request: &Request) -> bool {
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
pub(crate) fn hms(jd: f64) -> String {
    // A Julian day starts at noon; just under midnight, the product can round up to 86400.
    let seconds = (((jd + 0.5).rem_euclid(1.0) * 86400.0) as u32).min(86399);
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests;

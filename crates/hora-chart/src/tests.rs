use hora_swe::{Grahas, Limbs};

use super::*;

/// A yoga row's grahas without their roles.
pub(crate) fn bodies(grahas: &[YogaGraha]) -> Vec<Body> {
    grahas.iter().map(|g| g.graha).collect()
}

/// Sidereal longitude from a rasi (0 = Aries) and degrees, minutes, seconds within it.
pub(crate) fn dms(rasi: u8, d: u8, m: u8, s: f64) -> f64 {
    f64::from(rasi) * 30.0 + f64::from(d) + f64::from(m) / 60.0 + s / 3600.0
}

/// A Sky with the lagna and `[longitude, speed]` for Sun to Rahu; the speeds' signs are all a chart reads.
pub(crate) fn sky(lagna: f64, g: [[f64; 2]; 8]) -> Sky {
    let p = |[longitude, speed]: [f64; 2]| Position { longitude, speed };
    Sky {
        jd: 0.0,
        grahas: Grahas {
            sun: p(g[0]),
            moon: p(g[1]),
            mars: p(g[2]),
            mercury: p(g[3]),
            jupiter: p(g[4]),
            venus: p(g[5]),
            saturn: p(g[6]),
            rahu: p(g[7]),
        },
        lagna,
        ayanamsa: 0.0,
        vedic_day: None,
        new_moons: None,
        limbs: Limbs::default(),
        part_lagnas: None,
    }
}

/// Chart 1: ch. 9 *Arudha Padas*, p. 87, to the arcminute the book prints. It prints no speeds; the mean node always moves backward.
fn chart_1() -> Sky {
    sky(
        dms(5, 10, 58, 0.0),
        [
            [dms(11, 26, 29, 0.0), 1.0],
            [dms(2, 4, 45, 0.0), 13.0],
            [dms(0, 19, 9, 0.0), 0.7],
            [dms(11, 1, 36, 0.0), 1.5],
            [dms(0, 17, 21, 0.0), 0.2],
            [dms(11, 10, 1, 0.0), 1.2],
            [dms(0, 22, 41, 0.0), 0.1],
            [dms(3, 5, 55, 0.0), -0.05],
        ],
    )
}

#[test]
fn grahas_follow_the_charts_order() {
    let chart = chart(&chart_1());
    assert_eq!(chart.lagna().body, Body::Lagna);
    assert!(chart.grahas().iter().map(|e| e.body).eq(GRAHAS));
}

#[test]
fn rasi_splits_the_books_chart_1_values() {
    use Body::*;
    use Rasi::*;
    // No printed value lies within an arcminute of a rasi boundary, so the book's rounding decides nothing here.
    let books = [
        (Lagna, Virgo, (10, 58)),
        (Sun, Pisces, (26, 29)),
        (Moon, Gemini, (4, 45)),
        (Mars, Aries, (19, 9)),
        (Mercury, Pisces, (1, 36)),
        (Jupiter, Aries, (17, 21)),
        (Venus, Pisces, (10, 1)),
        (Saturn, Aries, (22, 41)),
        (Rahu, Cancer, (5, 55)),
        (Ketu, Capricorn, (5, 55)),
    ];
    let chart = chart(&chart_1());
    for (entry, (body, book_rasi, (d, m))) in chart.entries().iter().zip(books) {
        assert_eq!(entry.body, body);
        let (got_rasi, degrees) = rasi(entry.longitude);
        assert_eq!(got_rasi, book_rasi, "{body:?}");
        assert!(
            (degrees - dms(0, d, m, 0.0)).abs() < 1e-9,
            "{body:?}: {degrees}"
        );
        assert_eq!(entry.retrograde, matches!(body, Rahu | Ketu), "{body:?}");
    }
}

#[test]
#[should_panic(expected = "the lagna, then the grahas in the book's order")]
fn a_chart_of_entries_out_of_the_books_order_panics() {
    let mut entries: [Entry; 10] = chart(&chart_1()).entries().try_into().unwrap();
    entries.swap(1, 2);
    Chart::new(entries, Vec::new());
}

#[test]
fn every_graha_moving_backward_is_retrograde_and_ketu_is_opposite_rahu() {
    let chart = chart(&sky(-1.0, [[350.0, -1.0]; 8]));
    let marked: Vec<_> = chart
        .entries()
        .iter()
        .filter(|e| e.retrograde)
        .map(|e| e.body)
        .collect();
    use Body::*;
    assert_eq!(
        marked,
        [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu]
    );
    assert_eq!(chart.entry(Ketu).longitude, 170.0);
    // A true node moving forward loses its R, Ketu with it.
    let forward = super::chart(&sky(-1.0, [[350.0, 1.0]; 8]));
    assert!(forward.entries().iter().all(|e| !e.retrograde));
}

#[test]
fn a_longitude_stays_in_its_rasi_up_to_the_boundary() {
    let (r, degrees) = rasi(dms(1, 29, 59, 59.9));
    assert_eq!(r, Rasi::Taurus);
    assert!((29.99..30.0).contains(&degrees), "{degrees}");
    assert_eq!(rasi(60.0), (Rasi::Gemini, 0.0));
    let (r, degrees) = rasi(360.0 - 1e-12);
    assert_eq!(r, Rasi::Pisces);
    assert!((29.99..30.0).contains(&degrees), "{degrees}");
    // rem_euclid rounds this up to exactly 360.0.
    assert_eq!((-1e-20f64).rem_euclid(360.0), 360.0);
    assert_eq!(rasi(-1e-20), (Rasi::Aries, 0.0));
    assert_eq!(rasi(360.0), (Rasi::Aries, 0.0));
}

#[test]
fn bhavas_from_a_horalagna_in_cancer_wrap_past_pisces() {
    use Rasi::*;
    // Ch. 1 §1.3.3's example.
    for (rasi, house) in [
        (Cancer, 1),
        (Leo, 2),
        (Virgo, 3),
        (Pisces, 9),
        (Aries, 10),
        (Taurus, 11),
        (Gemini, 12),
    ] {
        assert_eq!(bhava(Cancer, rasi), house, "{rasi:?}");
    }
}

/// A Vedic day from `sunrise` to `next_sunrise`, its clock the same as UT.
pub(crate) fn day(sunrise: f64, next_sunrise: f64) -> VedicDay {
    VedicDay {
        sunrise,
        sun_at_sunrise: 0.0,
        sunset: sunrise + 0.5,
        next_sunrise,
        clock_sunrise: sunrise,
        clock_sunset: sunrise + 0.5,
        clock_next_sunrise: next_sunrise,
    }
}

/// 2000-01-05, a Wednesday, at 00:00.
pub(crate) const WEDNESDAY: f64 = 2451548.5;

pub(crate) fn hours(h: u8, m: u8) -> f64 {
    (f64::from(h) + f64::from(m) / 60.0) / 24.0
}

/// A Sky on `vaara`'s Vedic day, born in its day or `night`, whose part lagnas are `lagna(part, middle)`.
pub(crate) fn timed_sky(vaara: Vaara, night: bool, lagna: impl Fn(usize, bool) -> f64) -> Sky {
    // 06:00 on 2000-01-02, a Sunday, then on to `vaara`.
    let sunrise = WEDNESDAY - 3.0 + hours(6, 0) + vaara as usize as f64;
    let day = day(sunrise, sunrise + 1.0);
    Sky {
        jd: if night { day.sunset } else { day.sunrise },
        vedic_day: Some(day),
        part_lagnas: Some(std::array::from_fn(|k| [lagna(k, false), lagna(k, true)])),
        ..sky(0.0, [[0.0, 1.0]; 8])
    }
}

/// A chart with the lagna and Sun to Rahu at `g`, all direct; a paaka lagna reads no speeds.
pub(crate) fn placed(lagna: f64, g: [f64; 8]) -> Chart {
    chart(&sky(lagna, g.map(|longitude| [longitude, 1.0])))
}

#[test]
fn paaka_lagna_is_the_lagna_lords_rasi() {
    // §7.3.5: a Pisces lagna with Jupiter in Cancer gives Cancer; a Leo lagna with the Sun in Virgo gives Virgo.
    let mut g = [dms(0, 15, 0, 0.0); 8];
    g[4] = dms(3, 15, 0, 0.0);
    assert_eq!(
        placed(dms(11, 10, 0, 0.0), g)
            .in_varga(Varga::RasiChart)
            .paaka_lagna(),
        Rasi::Cancer
    );
    let mut g = [dms(0, 15, 0, 0.0); 8];
    g[0] = dms(5, 15, 0, 0.0);
    assert_eq!(
        placed(dms(4, 10, 0, 0.0), g)
            .in_varga(Varga::RasiChart)
            .paaka_lagna(),
        Rasi::Virgo
    );
}

#[test]
fn a_vargas_paaka_lagna_takes_that_vargas_lagna_lord() {
    // Pisces 1° is Cancer in D-9, so the Moon, not the rasi chart's lord Jupiter, decides (ADR 0003).
    let mut g = [dms(0, 1, 0, 0.0); 8];
    // Aries 5°, the 2nd navamsa: Taurus. Jupiter at Aries 1° is Aries in D-9, the carried-over reading.
    g[1] = dms(0, 5, 0, 0.0);
    assert_eq!(
        placed(dms(11, 1, 0, 0.0), g)
            .in_varga(Varga::Navamsa)
            .paaka_lagna(),
        Rasi::Taurus
    );
}

#[test]
fn hl_and_gl_are_no_reference_point_without_a_vedic_day() {
    let chart = placed(dms(0, 1, 0, 0.0), [dms(0, 1, 0, 0.0); 8]);
    let v = chart.in_varga(Varga::RasiChart);
    for lagna in [SpecialLagna::HoraLagna, SpecialLagna::GhatiLagna] {
        assert_eq!(v.reference(ReferencePoint::Special(lagna)), None);
    }
    assert!(
        v.reference(ReferencePoint::Special(SpecialLagna::SreeLagna))
            .is_some()
    );
}

#[test]
fn a_childs_lagna_counts_forward_from_an_odd_lagna_and_backward_from_an_even_one() {
    // §13.4.2, p. 171: a Ge lagna gives Venus (Li) for the 1st child and Jupiter (Sg) for the 2nd; a Cn lagna, Jupiter (Pi)
    // and Saturn (Cp). Jupiter in Aries, Venus in Taurus, Saturn in Gemini, Mars in Leo.
    let mut g = [dms(0, 15, 0, 0.0); 8];
    g[2] = dms(4, 15, 0, 0.0);
    g[5] = dms(1, 15, 0, 0.0);
    g[6] = dms(2, 15, 0, 0.0);
    let relative = |lagna, who| {
        placed(lagna, g)
            .in_varga(Varga::RasiChart)
            .reference(ReferencePoint::Relative(who))
    };
    let (ge, cn) = (dms(2, 10, 0, 0.0), dms(3, 10, 0, 0.0));
    assert_eq!(
        relative(ge, Relative::Child(Nth::First)),
        Some(Rasi::Taurus)
    );
    assert_eq!(
        relative(ge, Relative::Child(Nth::Second)),
        Some(Rasi::Aries)
    );
    assert_eq!(relative(cn, Relative::Child(Nth::First)), Some(Rasi::Aries));
    assert_eq!(
        relative(cn, Relative::Child(Nth::Second)),
        Some(Rasi::Gemini)
    );
    // Siblings too: from Cn the 3rd is Ta (Venus) and the 11th Vi (Mercury, in Aries), not Vi and Ta.
    assert_eq!(
        relative(cn, Relative::YoungerSibling(Nth::First)),
        Some(Rasi::Taurus)
    );
    assert_eq!(
        relative(cn, Relative::ElderSibling(Nth::First)),
        Some(Rasi::Aries)
    );
    // Father and mother count forward even from Cn: the 9th Pi (Jupiter), the 4th Li (Venus); backward, Mars's Sc and Ar.
    assert_eq!(relative(cn, Relative::Father), Some(Rasi::Aries));
    assert_eq!(relative(cn, Relative::Mother), Some(Rasi::Taurus));
}

#[test]
fn siblings_lagnas_are_the_lords_of_the_3rd_5th_7th_and_11th_9th_7th() {
    // §13.4.2, p. 171, from a Ge lagna: younger Le (Sun), Li (Venus), Sg (Jupiter); elder Ar (Mars), Aq (Saturn), Sg; spouse Sg.
    let mut g: [f64; 8] = std::array::from_fn(|i| dms(i as u8, 15, 0, 0.0));
    // Rahu in Aquarius makes Saturn, outside it, its stronger lord (§15.5.1).
    g[7] = dms(10, 15, 0, 0.0);
    let v = placed(dms(2, 10, 0, 0.0), g);
    let v = v.in_varga(Varga::RasiChart);
    let at = |who| v.reference(ReferencePoint::Relative(who));
    let rasi_of = |b| Some(v.rasi_of(b));
    use Relative::*;
    assert_eq!(at(YoungerSibling(Nth::First)), rasi_of(Body::Sun));
    assert_eq!(at(YoungerSibling(Nth::Second)), rasi_of(Body::Venus));
    assert_eq!(at(YoungerSibling(Nth::Third)), rasi_of(Body::Jupiter));
    assert_eq!(at(ElderSibling(Nth::First)), rasi_of(Body::Mars));
    assert_eq!(at(ElderSibling(Nth::Second)), rasi_of(Body::Saturn));
    assert_eq!(at(ElderSibling(Nth::Third)), rasi_of(Body::Jupiter));
    assert_eq!(at(Spouse), rasi_of(Body::Jupiter));
}

#[test]
fn a_bhava_is_1_to_12() {
    assert_eq!(
        [0, 1, 12, 13].map(|n| Bhava::new(n).is_some()),
        [false, true, true, false]
    );
}

#[test]
fn a_vargas_paaka_lagna_and_arudha_lagna_are_its_own() {
    // As above: a Pisces lagna is Cancer in D-9, the Moon's Taurus its paaka lagna, against Jupiter's Aries in the rasi chart.
    let mut g = [dms(0, 1, 0, 0.0); 8];
    g[1] = dms(0, 5, 0, 0.0);
    let chart = placed(dms(11, 1, 0, 0.0), g);
    let (d1, d9) = (
        chart.in_varga(Varga::RasiChart),
        chart.in_varga(Varga::Navamsa),
    );
    let paaka = ReferencePoint::PaakaLagna;
    assert_eq!(d9.reference(paaka), Some(Rasi::Taurus));
    assert_eq!(d1.reference(paaka), Some(Rasi::Aries));
    // AL: Cancer's lord in Taurus, the 11th, so the 11th from Taurus; Pisces's in Aries, the 2nd, so Taurus.
    let al = ReferencePoint::Arudha(Bhava::new(1).unwrap());
    assert_eq!(d9.reference(al), Some(Rasi::Pisces));
    assert_eq!(d1.reference(al), Some(Rasi::Taurus));
}

#[test]
fn scorpio_and_aquarius_lagnas_take_mars_and_saturn() {
    // Rahu in Aries puts Ketu in Libra; §15.5.1's stronger lord is not used here until ch. 15 decides (ADR 0003).
    let mut g = [dms(0, 15, 0, 0.0); 8];
    g[2] = dms(2, 15, 0, 0.0);
    g[6] = dms(4, 15, 0, 0.0);
    let chart = |lagna| placed(lagna, g);
    assert_eq!(
        chart(dms(7, 10, 0, 0.0))
            .in_varga(Varga::RasiChart)
            .paaka_lagna(),
        Rasi::Gemini
    );
    assert_eq!(
        chart(dms(10, 10, 0, 0.0))
            .in_varga(Varga::RasiChart)
            .paaka_lagna(),
        Rasi::Leo
    );
}

#[test]
fn the_rasi_charts_paaka_lagna_is_the_lagna_lords_rasi() {
    // Sun to Saturn each in a different rasi, Aries to Libra, so every lord is told apart.
    let g = std::array::from_fn(|i| dms(i as u8, 15, 0, 0.0));
    for r in RASIS {
        let chart = placed(dms(r as u8, 10, 0, 0.0), g);
        assert_eq!(
            chart.in_varga(Varga::RasiChart).paaka_lagna(),
            chart.in_varga(Varga::RasiChart).rasi_of(r.lord()),
            "{r:?}"
        );
    }
}

/// `r` at 15°.
pub(crate) fn mid(r: Rasi) -> f64 {
    dms(r as u8, 15, 0, 0.0)
}

/// A chart with everything at 15° of `rest` but for the bodies in `at`. The nodes move apart freely: only their rasis and
/// advancement are read, and §15.5's examples never place both.
pub(crate) fn scene(rest: Rasi, at: &[(Body, f64)]) -> Chart {
    let mut entries: [Entry; 10] = placed(mid(rest), [mid(rest); 8])
        .entries()
        .try_into()
        .unwrap();
    for &(body, longitude) in at {
        entries[body as usize].longitude = longitude;
    }
    Chart::new(entries, Vec::new())
}

/// A chart with Mercury in Virgo, `company` beside him, and everything else in Aquarius but for the nodes in Leo and Aquarius.
pub(crate) fn with_mercury(company: &[Body]) -> Chart {
    let mut at = vec![
        (Body::Mercury, mid(Rasi::Virgo)),
        (Body::Rahu, mid(Rasi::Aquarius)),
        (Body::Ketu, mid(Rasi::Leo)),
    ];
    at.extend(company.iter().map(|&g| (g, mid(Rasi::Virgo))));
    scene(Rasi::Aquarius, &at)
}

pub(crate) fn yoga_in(chart: &Chart, d: Varga, name: YogaName) -> Option<Yoga> {
    chart
        .in_varga(d)
        .yogas()
        .into_iter()
        .find(|y| y.name == name)
}

/// `name`'s row in the rasi chart of a scene with everything at 15° of `rest` but for `at`.
pub(crate) fn yoga_of(rest: Rasi, at: &[(Body, Rasi)], name: YogaName) -> Option<Yoga> {
    let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
    yoga_in(&scene(rest, &at), Varga::RasiChart, name)
}

/// `name`'s row with the lagna at `lagna` and the rest piled in `rest`, but for `at`.
pub(crate) fn from_lagna(
    lagna: Rasi,
    rest: Rasi,
    at: &[(Body, Rasi)],
    name: YogaName,
) -> Option<Yoga> {
    let mut at = at.to_vec();
    at.push((Body::Lagna, lagna));
    yoga_of(rest, &at, name)
}

pub(crate) fn present(row: Option<Yoga>) -> Vec<Body> {
    let row = row.expect("the yoga is listed");
    assert_eq!(row.to_judge, []);
    assert_eq!(row.form, None);
    bodies(&row.grahas)
}

/// `name`'s rows in the rasi chart with the lagna mid-`lagna`, the rest at 15° of `rest`, but for `at`'s rasis and degrees,
/// and `lagnas` as [`special_lagnas`] gives them.
pub(crate) fn raaja_rows(
    lagna: Rasi,
    rest: Rasi,
    at: &[(Body, Rasi, f64)],
    lagnas: &[(SpecialLagna, f64)],
    name: YogaName,
) -> Vec<Yoga> {
    let mut placed = vec![(Body::Lagna, mid(lagna))];
    placed.extend(
        at.iter()
            .map(|&(b, r, deg)| (b, f64::from(r as u8) * 30.0 + deg)),
    );
    let entries = scene(rest, &placed).entries().try_into().unwrap();
    Chart::new(entries, lagnas.to_vec())
        .in_varga(Varga::RasiChart)
        .yogas()
        .into_iter()
        .filter(|y| y.name == name)
        .collect()
}

/// The grahas of `name`'s rows, as [`raaja_rows`] places them, each present.
pub(crate) fn present_rows(
    lagna: Rasi,
    rest: Rasi,
    at: &[(Body, Rasi, f64)],
    name: YogaName,
) -> Vec<Vec<Body>> {
    let rows = raaja_rows(lagna, rest, at, &[], name);
    rows.into_iter().map(|y| present(Some(y))).collect()
}

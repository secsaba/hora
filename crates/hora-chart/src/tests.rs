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

/// A chart with each body at 15° of its rasi, but `ak` at 28°, the most advanced; for Rahu or Ketu, at 2°.
fn with_ak(at: [(Body, Rasi); 10], ak: Body) -> Chart {
    let at = at.map(|(b, r)| {
        let degrees = match b {
            _ if b != ak => 15,
            Body::Rahu | Body::Ketu => 2,
            _ => 28,
        };
        (b, dms(r as u8, degrees, 0, 0.0))
    });
    scene(Rasi::Aries, &at)
}

#[test]
fn maheswara_exception_1_takes_the_stronger_of_the_8th_and_12th_lords_from_an_exalted_saturn() {
    use Body::*;
    use Rasi::*;
    // §14.3 (1), p. 183: AK Mars in Ge, Saturn exalted in Li, so Venus (Ta) or Mercury (Vi); Mercury joins the Sun.
    let chart = with_ak(
        [
            (Lagna, Pisces),
            (Sun, Aries),
            (Moon, Sagittarius),
            (Mars, Gemini),
            (Mercury, Aries),
            (Jupiter, Capricorn),
            (Venus, Cancer),
            (Saturn, Libra),
            (Rahu, Leo),
            (Ketu, Aquarius),
        ],
        Mars,
    );
    assert_eq!(
        chart.in_varga(Varga::RasiChart).maheswara(),
        Maheswara {
            graha: Mercury,
            exceptions: vec![MaheswaraException::OwnOrExalted]
        }
    );
}

#[test]
fn maheswara_exception_1_does_not_count_mercury_as_conjoining_himself() {
    use Body::*;
    use Rasi::*;
    // §14.3 (1), p. 183, and §15.5.1 rule 2, ADR 0012: Venus (Ta) and Mercury (Ar) tie at rules 1 to 3 once Mercury's own
    // place is not his conjunction, so fixed Ta beats movable Ar at rule 4.
    let chart = with_ak(
        [
            (Lagna, Pisces),
            (Sun, Pisces),
            (Moon, Cancer),
            (Mars, Gemini),
            (Mercury, Aries),
            (Jupiter, Sagittarius),
            (Venus, Taurus),
            (Saturn, Libra),
            (Rahu, Leo),
            (Ketu, Aquarius),
        ],
        Mars,
    );
    assert_eq!(
        chart.in_varga(Varga::RasiChart).maheswara(),
        Maheswara {
            graha: Venus,
            exceptions: vec![MaheswaraException::OwnOrExalted]
        }
    );
}

#[test]
fn maheswara_exception_2_takes_the_6th_with_ketu_in_ta_or_sg() {
    use Body::*;
    use Rasi::*;
    // §14.3 (2), p. 183: AK Mars in Ta gives Jupiter (Sg); Ketu in Ta or Sg gives Venus (Li).
    let maheswara = |rahu, ketu| {
        with_ak(
            [
                (Lagna, Leo),
                (Sun, Leo),
                (Moon, Leo),
                (Mars, Taurus),
                (Mercury, Leo),
                (Jupiter, Aries),
                (Venus, Cancer),
                (Saturn, Virgo),
                (Rahu, rahu),
                (Ketu, ketu),
            ],
            Mars,
        )
        .in_varga(Varga::RasiChart)
        .maheswara()
    };
    assert_eq!(maheswara(Leo, Aquarius).graha, Jupiter);
    for (rahu, ketu) in [(Scorpio, Taurus), (Gemini, Sagittarius)] {
        assert_eq!(
            maheswara(rahu, ketu),
            Maheswara {
                graha: Venus,
                exceptions: vec![MaheswaraException::Node]
            }
        );
    }
}

#[test]
fn maheswara_exceptions_2_and_1_together_take_the_6th_and_2nd_from_the_lord() {
    use Body::*;
    use Rasi::*;
    // §14.3 (2) then (1), p. 183, ADR 0017: Ketu with AK Mars in Ta gives Venus (Li), exalted in Pi, so the stronger of the 6th
    // and 2nd lords from Pi, the Sun (Le) alone or Mars (Ar) joined by Ketu.
    let chart = with_ak(
        [
            (Lagna, Leo),
            (Sun, Gemini),
            (Moon, Leo),
            (Mars, Taurus),
            (Mercury, Leo),
            (Jupiter, Aries),
            (Venus, Pisces),
            (Saturn, Virgo),
            (Rahu, Scorpio),
            (Ketu, Taurus),
        ],
        Mars,
    );
    assert_eq!(
        chart.in_varga(Varga::RasiChart).maheswara(),
        Maheswara {
            graha: Mars,
            exceptions: vec![MaheswaraException::Node, MaheswaraException::OwnOrExalted]
        }
    );
}

#[test]
fn maheswara_exception_3_takes_mercury_for_rahu_and_jupiter_for_ketu() {
    use Body::*;
    use Rasi::*;
    // §14.3 (3), p. 183: AK Sun in Cn, Saturn in Aq makes Rahu its lord; AK Sun in Ar, Mars in Sc makes Ketu Scorpio's.
    let maheswara = |sun, saturn, mars, rahu, ketu| {
        with_ak(
            [
                (Lagna, Aries),
                (Sun, sun),
                (Moon, Aries),
                (Mars, mars),
                (Mercury, Aries),
                (Jupiter, Aries),
                (Venus, Aries),
                (Saturn, saturn),
                (Rahu, rahu),
                (Ketu, ketu),
            ],
            Sun,
        )
        .in_varga(Varga::RasiChart)
        .maheswara()
    };
    let replaced = |graha| Maheswara {
        graha,
        exceptions: vec![MaheswaraException::NodeReplaced],
    };
    assert_eq!(
        maheswara(Cancer, Aquarius, Aries, Taurus, Scorpio),
        replaced(Mercury)
    );
    assert_eq!(
        maheswara(Aries, Leo, Scorpio, Capricorn, Cancer),
        replaced(Jupiter)
    );
}

#[test]
fn maheswara_from_rahu_as_ak_ignores_rahu_joining_himself() {
    use Body::*;
    use Rasi::*;
    // (2) is Rahu or Ketu "joining" AK: a Rahu AK in Ta does not, so the 8th, Sg, and Jupiter (hora's reading).
    let chart = with_ak(
        [
            (Lagna, Leo),
            (Sun, Leo),
            (Moon, Leo),
            (Mars, Leo),
            (Mercury, Leo),
            (Jupiter, Aries),
            (Venus, Leo),
            (Saturn, Leo),
            (Rahu, Taurus),
            (Ketu, Scorpio),
        ],
        Rahu,
    );
    assert_eq!(
        chart.in_varga(Varga::RasiChart).maheswara(),
        Maheswara {
            graha: Jupiter,
            exceptions: vec![]
        }
    );
}

#[test]
fn rudra_is_decided_by_own_rasi_then_exalted_company_then_rasi_drishti() {
    use Body::*;
    use Rasi::*;
    // §14.3, p. 182: a Cn lagna's 8th lords by Table 32 are Jupiter (Sg) and Mercury (Ge); each scene ties the rules before
    // the one tested, and Mercury, the second candidate, wins, so a tie's default cannot pass it.
    let rudra = |jupiter, mercury, sun, moon, mars, venus, saturn| {
        with_ak(
            [
                (Lagna, Cancer),
                (Sun, sun),
                (Moon, moon),
                (Mars, mars),
                (Mercury, mercury),
                (Jupiter, jupiter),
                (Venus, venus),
                (Saturn, saturn),
                (Rahu, Scorpio),
                (Ketu, Taurus),
            ],
            Sun,
        )
        .in_varga(Varga::RasiChart)
        .rudra()
    };
    // Both alone; Mercury in his own Vi.
    let own = rudra(Leo, Virgo, Aries, Taurus, Aries, Taurus, Libra);
    // One companion each; Mercury's, the Sun in Ar, is exalted.
    let joins_exalted = rudra(Leo, Aries, Aries, Leo, Taurus, Gemini, Libra);
    // Both alone; the Sun, Moon and Mars aspect Mercury's Le by rasi drishti, none Jupiter's Ge.
    let aspected = rudra(Gemini, Leo, Aries, Libra, Capricorn, Taurus, Aquarius);
    for (r, rule) in [
        (own, RudraRule::ExaltedOrOwn),
        (joins_exalted, RudraRule::JoinsExalted),
        (aspected, RudraRule::Aspected),
    ] {
        assert_eq!((r.rudra, r.decided_by), (Mercury, Some(rule)));
    }
}

#[test]
fn a_debilitated_weaker_8th_lord_is_rudra_too_only_with_a_malefic_on_him() {
    use Body::*;
    use Rasi::*;
    // §14.3, p. 182: a Cn lagna's 8th lords by Table 32 are Jupiter (Sg) and Mercury (Ge); Jupiter joins two planets, Mercury none
    // and is debilitated in Pi. With Saturn in Ta no malefic is on Pi; from Vi, Saturn aspects it.
    let rudra = |saturn| {
        with_ak(
            [
                (Lagna, Cancer),
                (Sun, Cancer),
                (Moon, Cancer),
                (Mars, Aries),
                (Mercury, Pisces),
                (Jupiter, Cancer),
                (Venus, Libra),
                (Saturn, saturn),
                (Rahu, Leo),
                (Ketu, Aquarius),
            ],
            Sun,
        )
        .in_varga(Varga::RasiChart)
        .rudra()
    };
    let unafflicted = rudra(Taurus);
    assert_eq!(
        (
            unafflicted.rudra,
            unafflicted.weaker,
            unafflicted.weaker_afflicted
        ),
        (Jupiter, Mercury, false)
    );
    assert_eq!(unafflicted.decided_by, Some(RudraRule::Conjoins));
    assert_eq!(unafflicted.trishoola, [Cancer, Scorpio, Pisces]);
    assert!(rudra(Virgo).weaker_afflicted);
}

#[test]
fn marakas_count_every_2nd_and_7th_lord_beside_those_section_14_2s_leo_example_names() {
    use Body::*;
    use Rasi::*;
    // §14.2's Leo lagna, Saturn in Sg and Mars in Ge: Mercury rules the 2nd, and Rahu is Aquarius's co-lord outside it.
    let at = [
        (Lagna, Leo),
        (Sun, Cancer),
        (Moon, Libra),
        (Mars, Gemini),
        (Saturn, Sagittarius),
        (Rahu, Taurus),
        (Ketu, Scorpio),
    ];
    let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
    let chart = scene(Leo, &at);
    assert_eq!(
        chart.in_varga(Varga::RasiChart).marakas(),
        [Mars, Mercury, Saturn, Rahu]
    );
}

#[test]
fn each_av_table_has_its_classical_total_of_rekhas() {
    // Ch. 12 Tables 19–26: 48, 49, 39, 54, 56, 52 and 39 for Sun to Saturn, 49 for the lagna; any chart sums to them.
    let chart = placed(dms(3, 15, 0, 0.0), [dms(0, 15, 0, 0.0); 8]);
    let totals = AV_REFERENCES.map(|of| {
        let bav = chart.in_varga(Varga::RasiChart).bav(of);
        bav.iter().map(|&c| u32::from(c)).sum::<u32>()
    });
    assert_eq!(totals, [48, 49, 39, 54, 56, 52, 39, 49]);
}

#[test]
fn the_lagnas_bav_counts_table_26_from_each_reference() {
    // Ch. 12 Table 26, by hand: with the seven in Ar, each house's row sum lands on the rasi it counts to, the lagna's own column
    // left out; the lagna in Cn adds its 3rd, 6th, 10th and 11th from there, Vi, Sg, Ar and Ta.
    let chart = placed(dms(3, 15, 0, 0.0), [dms(0, 15, 0, 0.0); 8]);
    assert_eq!(
        chart.in_varga(Varga::RasiChart).bav(Body::Lagna),
        [6, 4, 5, 5, 2, 7, 1, 2, 3, 6, 6, 2]
    );
}

#[test]
fn bands_turn_at_the_counts_adr_0015_gives() {
    use BavBand::*;
    use SavBand::*;
    let bav = |count| bav_band(Body::Jupiter, count);
    assert_eq!(
        [0, 3, 4, 5, 8].map(bav),
        [Unfavourable, Unfavourable, Neutral, Favourable, Favourable].map(Some)
    );
    assert_eq!(bav_band(Body::Lagna, 8), None);
    assert_eq!(
        [0, 24, 25, 29, 30, 56].map(sav_band),
        [Weak, Weak, Average, Average, Strong, Strong]
    );
}

#[test]
fn example_42_ekaadhipatya_sodhana_on_taurus_and_libra() {
    // Ch. 12 Example 42, pp. 158–159: Ta and Li after trikona sodhana, and which of them is occupied.
    let (ta, li) = (Rasi::Taurus as usize, Rasi::Libra as usize);
    let case = |[vt, vl]: [u8; 2], [ot, ol]: [bool; 2]| {
        let (mut values, mut occupied) = ([0; 12], [false; 12]);
        [values[ta], values[li]] = [vt, vl];
        [occupied[ta], occupied[li]] = [ot, ol];
        let out = ekaadhipatya_sodhana(values, occupied);
        [out[ta], out[li]]
    };
    assert_eq!(case([4, 2], [true, true]), [4, 2], "(a), rule (2)");
    assert_eq!(case([4, 2], [true, false]), [4, 0], "(b), rule (3a)");
    assert_eq!(case([4, 2], [false, true]), [2, 2], "(c), rule (3b)");
    assert_eq!(case([4, 2], [false, false]), [2, 2], "(d), rule (4b)");
    assert_eq!(case([2, 2], [false, false]), [0, 0], "(e), rule (4a)");
    assert_eq!(case([4, 0], [false, false]), [4, 0], "rule (1)");
}

#[test]
fn the_knowledge_bases_worked_case_reduces_and_sums_as_its_page_says() {
    use Rasi::*;
    // Pins kb/{en,hu}/sodhya-pinda.md's worked case; only the lagna occupies Sc, and Rahu alone in Pi does not.
    let bav = [4, 3, 2, 4, 2, 4, 0, 6, 3, 4, 2, 5];
    assert_eq!(bav.iter().sum::<u8>(), 39);
    let trikona = trikona_sodhana(bav);
    assert_eq!(trikona, [2, 0, 2, 0, 0, 1, 0, 2, 1, 1, 2, 1]);
    let at = [Gemini, Taurus, Capricorn, Gemini, Virgo, Cancer, Virgo];
    let occupied = RASIS.map(|r| r == Scorpio || at.contains(&r));
    let soav = ekaadhipatya_sodhana(trikona, occupied);
    assert_eq!(soav, [0, 0, 2, 0, 0, 1, 0, 2, 0, 1, 1, 0]);
    let p = pindas(soav, at);
    assert_eq!([p.rasi, p.graha, p.sodhya()], [53, 43, 96]);
}

#[test]
fn the_lagna_alone_occupies_a_rasi_and_rahu_or_ketu_alone_does_not() {
    use Rasi::*;
    // ADR 0015. The seven in Ar, the lagna alone in Aq: Mars's BAV after trikona sodhana has 3 in Cp and 6 in Aq, so the
    // empty Cp, lower, becomes 0 (rule (3a)); were the lagna not counted, both empty would give 3 and 3 (rule (4b)).
    let mut g = [dms(0, 15, 0, 0.0); 8];
    let lagna_in_aq = placed(dms(Aquarius as u8, 15, 0, 0.0), g);
    let v = lagna_in_aq.in_varga(Varga::RasiChart);
    let t = trikona_sodhana(v.bav(Body::Mars));
    assert_eq!([t[Capricorn as usize], t[Aquarius as usize]], [3, 6]);
    let soav = v.soav(Body::Mars);
    assert_eq!([soav[Capricorn as usize], soav[Aquarius as usize]], [0, 6]);
    // Rahu alone in Cp (Ketu in Cn), then Ketu alone in Cp (Rahu in Cn): were Cp occupied, rule (2) would keep its 3.
    for rahu in [Capricorn, Cancer] {
        g[Body::Rahu as usize - 1] = dms(rahu as u8, 15, 0, 0.0);
        let node_in_cp = placed(dms(Aquarius as u8, 15, 0, 0.0), g);
        assert_eq!(
            node_in_cp.in_varga(Varga::RasiChart).soav(Body::Mars),
            soav,
            "Rahu in {rahu:?}"
        );
    }
}

#[test]
fn age_avasthas_follow_table_35s_examples() {
    use AgeAvastha::*;
    // P. 190.
    assert_eq!(age_avastha(dms(3, 23, 0, 0.0)), Kumaara);
    assert_eq!(age_avastha(dms(6, 19, 0, 0.0)), Vriddha);
    assert_eq!(age_avastha(dms(8, 14, 0, 0.0)), Yuva);
    assert_eq!(age_avastha(dms(11, 27, 0, 0.0)), Saisava);
}

#[test]
fn age_avasthas_change_at_each_6_degrees_forward_in_odd_rasis_and_backward_in_even() {
    use AgeAvastha::*;
    let odd = [Saisava, Kumaara, Yuva, Vriddha, Mrita];
    let even = [Mrita, Vriddha, Yuva, Kumaara, Saisava];
    for (rasi, order) in [(0, odd), (1, even)] {
        for step in 0..5u8 {
            let at = dms(rasi, step * 6, 0, 0.0);
            assert_eq!(age_avastha(at), order[usize::from(step)], "{at}°");
            if step > 0 {
                let below = at - 1.0 / 3600.0;
                assert_eq!(age_avastha(below), order[usize::from(step) - 1], "{below}°");
            }
        }
        // 30° is the next rasi's 0°, whose parity is the other.
        let end = dms(rasi + 1, 0, 0, 0.0) - 1.0 / 3600.0;
        assert_eq!(age_avastha(end), order[4], "{end}°");
    }
    assert_eq!(age_avastha(dms(2, 0, 0, 0.0)), Saisava);
    assert_eq!(age_avastha(dms(2, 0, 0, 0.0) - 1.0 / 3600.0), Saisava);
}

/// `graha`'s avasthas in a [`scene`] around `rest` with the bodies in `at`, each at 15° of its rasi. Ketu starts in the 7th
/// from `rest`.
fn avasthas_in(rest: Rasi, at: &[(Body, Rasi)], graha: Body) -> Avasthas {
    let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
    let all = scene(rest, &at).avasthas();
    all.into_iter().find(|a| a.graha == graha).unwrap()
}

#[test]
fn alertness_is_jaagrita_at_home_or_exalted_then_sushupta_debilitated_then_by_the_lords_relationship()
 {
    use Alertness::*;
    use Body::*;
    use Rasi::*;
    let alertness = |at: &[(Body, Rasi)], graha| avasthas_in(Aries, at, graha).alertness;
    // §15.4.2 (pp. 190–191).
    assert_eq!(alertness(&[(Sun, Leo)], Sun), Some(Jaagrita));
    // The Moon exalted in Ta beside Venus, satru to her by the compound relationship: still awake.
    assert_eq!(
        alertness(&[(Moon, Taurus), (Venus, Taurus)], Moon),
        Some(Jaagrita)
    );
    // Jupiter debilitated in Cp with Saturn in Aq, mitra to him: still asleep.
    assert_eq!(
        alertness(&[(Jupiter, Capricorn), (Saturn, Aquarius)], Jupiter),
        Some(Sushupta)
    );
    // The Sun in Sg with Jupiter in Cp, adhimitra; in Ge with Mercury in Cn, mitra; in Sg with Jupiter, sama.
    assert_eq!(
        alertness(&[(Sun, Sagittarius), (Jupiter, Capricorn)], Sun),
        Some(Swapna)
    );
    assert_eq!(
        alertness(&[(Sun, Gemini), (Mercury, Cancer)], Sun),
        Some(Swapna)
    );
    assert_eq!(
        alertness(&[(Sun, Sagittarius), (Jupiter, Sagittarius)], Sun),
        Some(Swapna)
    );
    // The Sun in Cp with Saturn, adhisatru; Mars in Ta with Venus, satru.
    assert_eq!(
        alertness(&[(Sun, Capricorn), (Saturn, Capricorn)], Sun),
        Some(Sushupta)
    );
    assert_eq!(
        alertness(&[(Mars, Taurus), (Venus, Taurus)], Mars),
        Some(Sushupta)
    );
    // The nodes: awake exalted or at home, asleep debilitated, else none (ADR 0018).
    assert_eq!(alertness(&[(Rahu, Gemini)], Rahu), Some(Jaagrita));
    assert_eq!(alertness(&[(Ketu, Scorpio)], Ketu), Some(Jaagrita));
    assert_eq!(alertness(&[(Rahu, Sagittarius)], Rahu), Some(Sushupta));
    assert_eq!(alertness(&[(Rahu, Leo)], Rahu), None);
}

/// `graha`'s mood avasthas in [`avasthas_in`]'s scene.
fn moods(rest: Rasi, at: &[(Body, Rasi)], graha: Body) -> Vec<MoodAvastha> {
    avasthas_in(rest, at, graha).moods
}

#[test]
fn the_nine_mood_avasthas_read_the_rasi_its_lord_and_company() {
    use Body::*;
    use MoodAvastha::*;
    use Rasi::*;
    // §15.4.3's nine (p. 191). Everything else at 15° of Capricorn, with the lagna: a Sukla Moon, a malefic Mercury.
    let m = |at: &[(Body, Rasi)], graha| moods(Capricorn, at, graha);
    let has = |at: &[(Body, Rasi)], graha, state| m(at, graha).contains(&state);
    assert!(has(&[(Sun, Aries)], Sun, Deepta));
    assert!(!has(&[(Sun, Taurus)], Sun, Deepta));
    assert!(has(&[(Sun, Leo)], Sun, Swastha));
    // The rungs, with the lords placed as in the alertness test; none in the own rasi.
    let by_lord = [Mudita, Saanta, Deena, Duhkhita];
    assert!(!m(&[(Sun, Leo)], Sun).iter().any(|s| by_lord.contains(s)));
    assert!(has(
        &[(Sun, Sagittarius), (Jupiter, Capricorn)],
        Sun,
        Mudita
    ));
    assert!(has(&[(Sun, Gemini), (Mercury, Cancer)], Sun, Saanta));
    assert!(has(
        &[(Sun, Sagittarius), (Jupiter, Sagittarius)],
        Sun,
        Deena
    ));
    assert!(has(&[(Mars, Taurus), (Venus, Taurus)], Mars, Duhkhita));
    // An exalted Sun in Aries is also adhimitra to Mars there: every state that holds is listed.
    assert!(has(&[(Sun, Aries)], Sun, Mudita));
    // Vikala by Mars beside Jupiter, not by Venus; Khala in Mars's rasi, and in Saturn's own, as literally as the book.
    assert!(has(&[(Jupiter, Leo), (Mars, Leo)], Jupiter, Vikala));
    assert!(!has(&[(Jupiter, Leo), (Venus, Leo)], Jupiter, Vikala));
    assert!(has(&[(Jupiter, Aries)], Jupiter, Khala));
    assert!(!has(&[(Jupiter, Taurus)], Jupiter, Khala));
    assert!(has(&[], Saturn, Khala));
    // Kopita beside the Sun, who never is himself (ADR 0018).
    assert!(has(&[(Jupiter, Leo), (Sun, Leo)], Jupiter, Kopita));
    assert!(!has(&[(Jupiter, Leo), (Sun, Leo)], Sun, Kopita));
}

#[test]
fn the_six_more_mood_avasthas_read_the_fifth_dignity_enemies_water_jupiter_and_the_sun() {
    use Body::*;
    use MoodAvastha::*;
    use Rasi::*;
    // §15.4.3's six (pp. 191–192).
    let has = |rest, at: &[(Body, Rasi)], graha, state| moods(rest, at, graha).contains(&state);
    // Lajjita: the lagna in Cp, so the 5th is Ta.
    assert!(has(
        Capricorn,
        &[(Jupiter, Taurus), (Mars, Taurus)],
        Jupiter,
        Lajjita
    ));
    assert!(!has(
        Capricorn,
        &[(Jupiter, Taurus), (Venus, Taurus)],
        Jupiter,
        Lajjita
    ));
    assert!(!has(Capricorn, &[(Sun, Taurus)], Sun, Lajjita));
    assert!(!has(
        Capricorn,
        &[(Jupiter, Gemini), (Mars, Gemini)],
        Jupiter,
        Lajjita
    ));
    assert!(has(
        Capricorn,
        &[(Rahu, Taurus), (Saturn, Taurus)],
        Rahu,
        Lajjita
    ));
    // Garvita by whole rasi.
    assert!(has(Capricorn, &[(Saturn, Aquarius)], Saturn, Garvita));
    assert!(!has(Capricorn, &[], Saturn, Garvita));
    assert!(has(Capricorn, &[(Rahu, Virgo)], Rahu, Garvita));
    // Kshudhita: conjoined by Saturn, friend to Venus; aspected by Venus, adhisatru to Jupiter; in the Sun's Le with
    // him adhisatru in Sg, out of sight. Never for a node.
    assert!(has(
        Pisces,
        &[(Venus, Taurus), (Saturn, Taurus)],
        Venus,
        Kshudhita
    ));
    assert!(!has(Pisces, &[(Venus, Taurus)], Venus, Kshudhita));
    assert!(!has(Pisces, &[(Saturn, Taurus)], Saturn, Kshudhita));
    assert!(has(
        Taurus,
        &[(Jupiter, Sagittarius), (Venus, Gemini)],
        Jupiter,
        Kshudhita
    ));
    assert!(!has(Taurus, &[(Jupiter, Sagittarius)], Jupiter, Kshudhita));
    assert!(has(
        Taurus,
        &[(Venus, Leo), (Sun, Sagittarius)],
        Venus,
        Kshudhita
    ));
    assert!(!has(
        Pisces,
        &[(Rahu, Taurus), (Saturn, Taurus)],
        Rahu,
        Kshudhita
    ));
    // Trishita: Jupiter in Pi aspected by Mercury, adhisatru and malefic beside Mars; not once Venus aspects too.
    let thirsty = [(Jupiter, Pisces), (Mercury, Virgo), (Mars, Virgo)];
    assert!(has(Capricorn, &thirsty, Jupiter, Trishita));
    assert!(!has(Capricorn, &[(Jupiter, Pisces)], Jupiter, Trishita));
    assert!(!has(
        Capricorn,
        &[thirsty.as_slice(), &[(Venus, Sagittarius)]].concat(),
        Jupiter,
        Trishita
    ));
    // A node needs only the water and no benefic's aspect (ADR 0018).
    assert!(has(Capricorn, &[(Ketu, Pisces)], Ketu, Trishita));
    assert!(!has(
        Capricorn,
        &[(Ketu, Pisces), (Venus, Sagittarius)],
        Ketu,
        Trishita
    ));
    assert!(!has(Capricorn, &[(Ketu, Sagittarius)], Ketu, Trishita));
    // Mudita's second form: the Sun in Mercury's Ge (mitra) aspected by Mars (adhimitra) and joined by Jupiter.
    let delighted = [(Sun, Gemini), (Mercury, Leo), (Mars, Virgo)];
    assert_eq!(
        moods(
            Capricorn,
            &[delighted.as_slice(), &[(Jupiter, Gemini)]].concat(),
            Sun
        ),
        [Mudita, Saanta]
    );
    assert_eq!(moods(Capricorn, &delighted, Sun), [Saanta]);
    assert_eq!(
        moods(
            Capricorn,
            &[delighted[0], delighted[1], (Jupiter, Gemini)],
            Sun
        ),
        [Saanta]
    );
    // Both forms at once list Mudita once: the Sun in Cn with the Moon in Le, Jupiter beside him, Mars in Ta aspecting.
    let both = moods(
        Capricorn,
        &[
            (Sun, Cancer),
            (Moon, Leo),
            (Jupiter, Cancer),
            (Mars, Taurus),
        ],
        Sun,
    );
    assert_eq!(both.iter().filter(|&&s| s == Mudita).count(), 1);
    // Kshobhita: Jupiter beside the Sun in Ar, aspected by Saturn from Le; never the Sun himself, nor a node.
    let shaken = [(Jupiter, Aries), (Sun, Aries), (Saturn, Leo)];
    assert!(has(Capricorn, &shaken, Jupiter, Kshobhita));
    assert!(!has(Capricorn, &shaken[..2], Jupiter, Kshobhita));
    // Or by an enemy who is no malefic: Venus, adhisatru to Jupiter, from Le.
    assert!(has(
        Capricorn,
        &[shaken[0], shaken[1], (Venus, Leo)],
        Jupiter,
        Kshobhita
    ));
    assert!(!has(Capricorn, &shaken, Sun, Kshobhita));
    assert!(!has(
        Capricorn,
        &[(Rahu, Aries), (Sun, Aries), (Saturn, Leo)],
        Rahu,
        Kshobhita
    ));
}

#[test]
fn a_node_has_no_relationship_states() {
    use Body::*;
    use MoodAvastha::*;
    use Rasi::*;
    let m = |at: &[(Body, Rasi)], graha| moods(Capricorn, at, graha);
    // ADR 0018: the Sun's rasi is a malefic's, and the Sun joins Rahu there.
    assert_eq!(m(&[(Rahu, Leo)], Rahu), [Khala]);
    assert_eq!(m(&[(Rahu, Leo), (Sun, Leo)], Rahu), [Vikala, Khala, Kopita]);
    assert_eq!(m(&[(Rahu, Aquarius)], Rahu), [Swastha, Khala]);
    assert_eq!(m(&[(Ketu, Sagittarius)], Ketu), [Deepta, Garvita]);
    assert_eq!(m(&[(Ketu, Taurus)], Ketu), []);
    // A node's age by its degree as it stands: 23° Cn is Kumaara, as for any graha (Table 35, p. 190).
    let rahu = scene(Capricorn, &[(Rahu, dms(3, 23, 0, 0.0))]).avasthas()[7].age;
    assert_eq!(rahu, AgeAvastha::Kumaara);
}

#[test]
fn a_sayanaadi_part_is_3_20_counted_from_the_rasis_start() {
    // Ch. 15 footnote 51 (p. 192): Mercury at 22Ge14 is in the 7th.
    assert_eq!(sayanaadi_part(dms(2, 22, 14, 0.0)), 7);
    assert_eq!(sayanaadi_part(dms(2, 0, 0, 0.0)), 1);
    assert_eq!(sayanaadi_part(dms(2, 3, 20, 0.0) - 1.0 / 3600.0), 1);
    assert_eq!(sayanaadi_part(dms(2, 3, 20, 0.0)), 2);
    assert_eq!(sayanaadi_part(dms(3, 0, 0, 0.0) - 1.0 / 3600.0), 9);
}

#[test]
fn the_ghati_running_counts_whole_24_minutes_since_the_vedic_days_sunrise_from_1() {
    let sunrise = WEDNESDAY + hours(6, 0);
    let d = day(sunrise, sunrise + 1.0);
    // Ch. 15 footnote 52 (p. 192): sunrise at 6 am, born at 11 pm, 42.5 ghatis over: the 43rd.
    assert_eq!(ghati(&d, sunrise + hours(17, 0)), 43);
    assert_eq!(ghati(&d, sunrise), 1);
    assert_eq!(ghati(&d, sunrise + hours(0, 24) - 1.0 / 86400.0), 1);
    assert_eq!(ghati(&d, sunrise + hours(0, 24)), 2);
    // Born at 5 am, before the next sunrise: 23 hours from the previous one, the 58th.
    assert_eq!(ghati(&d, sunrise + hours(23, 0)), 58);
}

#[test]
fn without_a_name_sound_the_sayanaadi_avasthas_have_no_strength() {
    let sky = timed_sky(Vaara::Thursday, false, |_, _| 0.0);
    let avasthas = |sound| {
        let all = sayanaadi_avasthas(&sky, sound).unwrap();
        all.into_iter()
            .map(|s| (s.avastha, s.strength))
            .collect::<Vec<_>>()
    };
    let (with, without) = (avasthas(Some(1)), avasthas(None));
    assert!(with.iter().all(|(_, strength)| strength.is_some()));
    assert_eq!(
        without,
        with.iter().map(|&(a, _)| (a, None)).collect::<Vec<_>>()
    );
}

#[test]
fn rahu_is_p_8_ketu_p_9_and_both_adjust_the_strength_by_4() {
    // Lagna Ar, Moon in Aswini, born at sunrise: L = M = G = 1. Rahu 0Ar (C 1, A 1), so Ketu 0Li (Chitra, C 14, A 1).
    let sky = Sky {
        jd: WEDNESDAY,
        vedic_day: Some(day(WEDNESDAY, WEDNESDAY + 1.0)),
        ..sky(0.0, [[0.0, 1.0]; 8])
    };
    let nodes: Vec<_> = sayanaadi_avasthas(&sky, Some(1))
        .unwrap()
        .into_iter()
        .filter(|s| matches!(s.graha, Body::Rahu | Body::Ketu))
        .map(|s| (s.avastha, s.strength))
        .collect();
    use ActivityStrength::*;
    use SayanaadiAvastha::*;
    // Rahu 1·8·1 + 3 = 11, ((121 + 1) mod 12 + 4) mod 3 = 0; Ketu 14·9·1 + 3 = 129 → 9, ((81 + 1) mod 12 + 4) mod 3 = 2.
    assert_eq!(
        nodes,
        [(Kautuka, Some(Vicheshta)), (Bhojana, Some(Cheshta))]
    );
}

#[test]
fn a_sky_without_a_vedic_day_has_no_sayanaadi_avasthas() {
    assert_eq!(
        sayanaadi_avasthas(&sky(0.0, [[0.0, 1.0]; 8]), Some(1)),
        None
    );
}

use hora_swe::{Grahas, Limbs};

use super::*;

/// A yoga row's grahas without their roles.
fn bodies(grahas: &[YogaGraha]) -> Vec<Body> {
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

#[test]
fn the_sun_based_upagrahas_are_sun_baseds_five() {
    let five: Vec<_> = UPAGRAHAS.into_iter().filter(|u| u.is_sun_based()).collect();
    assert_eq!(five, sun_based(0.0).map(|(u, _)| u));
}

#[test]
fn upaketu_is_the_sun_less_30_and_the_pairs_are_opposite() {
    // Ch. 4 §4.2: Table 9's Upaketu row, and the note under it.
    let apart = |a: f64, b: f64| (a - b).rem_euclid(360.0);
    let close = |a: f64, b: f64| {
        let d = apart(a, b);
        d.min(360.0 - d) < 1e-9
    };
    for tenth in 0..3600 {
        let sun = f64::from(tenth) / 10.0;
        let [dhuma, vyatipaata, parivesha, indrachaapa, upaketu] = sun_based(sun).map(|(_, l)| {
            assert!((0.0..360.0).contains(&l), "{sun}: {l}");
            l
        });
        assert!(close(upaketu, sun - 30.0), "{sun}: {upaketu}");
        assert!(close(apart(dhuma, indrachaapa), 180.0), "{sun}");
        assert!(close(apart(vyatipaata, parivesha), 180.0), "{sun}");
    }
}

#[test]
fn part_rulers_are_table_10s() {
    // Ch. 4 §4.3, Table 10: the day's rows, then the night's, Sunday to Saturday; `—` is lord-less.
    use Body::*;
    let (su, mo, ma, me, ju, ve, sa) = (
        Some(Sun),
        Some(Moon),
        Some(Mars),
        Some(Mercury),
        Some(Jupiter),
        Some(Venus),
        Some(Saturn),
    );
    let day = [
        [su, mo, ma, me, ju, ve, sa, None],
        [mo, ma, me, ju, ve, sa, None, su],
        [ma, me, ju, ve, sa, None, su, mo],
        [me, ju, ve, sa, None, su, mo, ma],
        [ju, ve, sa, None, su, mo, ma, me],
        [ve, sa, None, su, mo, ma, me, ju],
        [sa, None, su, mo, ma, me, ju, ve],
    ];
    let night = [
        [ju, ve, sa, None, su, mo, ma, me],
        [ve, sa, None, su, mo, ma, me, ju],
        [sa, None, su, mo, ma, me, ju, ve],
        [su, mo, ma, me, ju, ve, sa, None],
        [mo, ma, me, ju, ve, sa, None, su],
        [ma, me, ju, ve, sa, None, su, mo],
        [me, ju, ve, sa, None, su, mo, ma],
    ];
    assert_eq!(VAARAS.map(|v| part_rulers(v, false)), day);
    assert_eq!(VAARAS.map(|v| part_rulers(v, true)), night);
}

/// A Sky on `vaara`'s Vedic day, born in its day or `night`, whose part lagnas are `lagna(part, middle)`.
fn timed_sky(vaara: Vaara, night: bool, lagna: impl Fn(usize, bool) -> f64) -> Sky {
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

/// The time-based upagrahas of `sky`, in the book's order.
fn time_based_of(sky: &Sky) -> Vec<(Upagraha, f64)> {
    upagrahas(sky).split_off(5)
}

#[test]
fn a_thursday_night_from_18_00_to_06_00_has_yamaghantaka_at_23_15() {
    // Ch. 4 §4.3's worked case: Jupiter rules the 4th part, 22:30 to 24:00; a part lagna here is its moment's hour.
    let sky = timed_sky(Vaara::Thursday, true, |k, middle| {
        18.0 + 1.5 * (k as f64 + if middle { 0.5 } else { 0.0 })
    });
    assert_eq!(part_rulers(Vaara::Thursday, true)[3], Some(Body::Jupiter));
    assert!(time_based_of(&sky).contains(&(Upagraha::Yamaghantaka, 23.25)));
}

#[test]
fn each_time_based_upagraha_rises_at_the_start_or_middle_of_its_part() {
    // A Thursday's day: Jup, Ven, Sat, —, Sun, Moon, Mars, Merc; a part lagna here is 10 × its part, + 5 at its middle.
    let sky = timed_sky(Vaara::Thursday, false, |k, middle| {
        10.0 * (k + 1) as f64 + if middle { 5.0 } else { 0.0 }
    });
    use Upagraha::*;
    assert_eq!(
        time_based_of(&sky),
        [
            (Kaala, 55.0),
            (Mrityu, 75.0),
            (Arthaprahaara, 85.0),
            (Yamaghantaka, 15.0),
            (Gulika, 30.0),
            (Maandi, 35.0),
        ]
    );
}

#[test]
fn a_sky_without_a_vedic_day_has_only_the_sun_based_upagrahas() {
    let sky = sky(0.0, [[0.0, 1.0]; 8]);
    assert_eq!(upagrahas(&sky), sun_based(0.0));
}

#[test]
fn a_polar_sky_without_a_vedic_day_has_only_the_sree_lagna() {
    let sky = sky(dms(5, 25, 5, 0.0), [[dms(6, 13, 6, 0.0), 13.0]; 8]);
    let got = special_lagnas(&sky);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].0, SpecialLagna::SreeLagna);
    assert!((got[0].1 - dms(11, 18, 47, 0.0)).abs() < 1e-9, "{got:?}");
}

#[test]
fn special_lagnas_count_the_exact_time_since_sunrise_not_the_days_length() {
    // A 23-hour Vedic day, born 90 minutes and 1 second after sunrise.
    let sunrise = WEDNESDAY + hours(6, 0);
    let mut day = day(sunrise, sunrise + 23.0 / 24.0);
    day.sun_at_sunrise = dms(9, 24, 17, 0.0);
    let sky = Sky {
        jd: sunrise + hours(1, 30) + 1.0 / 86400.0,
        vedic_day: Some(day),
        ..sky(0.0, [[0.0, 1.0]; 8])
    };
    let got = special_lagnas(&sky);
    let want = [
        (SpecialLagna::HoraLagna, dms(11, 9, 17, 30.0)),
        (SpecialLagna::GhatiLagna, dms(1, 16, 48, 15.0)),
        (SpecialLagna::SreeLagna, 0.0),
        (SpecialLagna::BhaavaLagna, dms(10, 16, 47, 15.0)),
    ];
    assert_eq!(got.iter().map(|g| g.0).collect::<Vec<_>>(), SPECIAL_LAGNAS);
    for ((lagna, got), (_, want)) in got.into_iter().zip(want) {
        // A Julian day near 2.45e6 holds about 40 µs, which GL turns into about 0.01″.
        assert!(
            (got - want).abs() * 3600.0 < 0.05,
            "{lagna:?}: {got} {want}"
        );
    }
}

#[test]
fn movable_and_earthy_rasis_start_their_count_where_6_2_says() {
    use Rasi::*;
    use Varga::*;
    // No example of §6.2 has a body in a movable or earthy rasi; the 2nd part of Cancer and of Taurus.
    let in_cancer = [Ashtamsa, Shodasamsa, Vimsamsa, Akshavedamsa]
        .map(|v| varga(v, dms(3, 0, 0, 0.0) + RASI_SPAN / f64::from(v.parts())));
    assert_eq!(in_cancer, [Taurus, Taurus, Taurus, Taurus]);
    let in_taurus = [Navamsa, Nakshatramsa]
        .map(|v| varga(v, dms(1, 0, 0, 0.0) + RASI_SPAN / f64::from(v.parts())));
    assert_eq!(in_taurus, [Aquarius, Leo]);
}

#[test]
fn the_rasi_chart_is_the_rasi() {
    for r in 0..12 {
        for d in [0.0, 15.0, 29.999] {
            let longitude = f64::from(r) * RASI_SPAN + d;
            assert_eq!(varga(Varga::RasiChart, longitude), rasi(longitude).0);
        }
    }
}

#[test]
fn a_part_starts_at_its_boundary_to_the_whole_arcsecond() {
    use Rasi::*;
    let drekkana = |d, m, s| varga(Varga::Drekkana, dms(0, d, m, s));
    assert_eq!([drekkana(9, 59, 59.9), drekkana(10, 0, 0.0)], [Aries, Leo]);
    // D-30's marks, just before and at each, in Aries (odd) and Taurus (even).
    let trimsamsa = |rasi, d, s| varga(Varga::Trimsamsa, dms(rasi, d, 0, 0.0) - s / 3600.0);
    let odd = [5, 10, 18, 25].map(|d| [trimsamsa(0, d, 0.1), trimsamsa(0, d, 0.0)]);
    assert_eq!(
        odd,
        [
            [Aries, Aquarius],
            [Aquarius, Sagittarius],
            [Sagittarius, Gemini],
            [Gemini, Libra]
        ]
    );
    let even = [5, 12, 20, 25].map(|d| [trimsamsa(1, d, 0.1), trimsamsa(1, d, 0.0)]);
    assert_eq!(
        even,
        [
            [Taurus, Virgo],
            [Virgo, Pisces],
            [Pisces, Capricorn],
            [Capricorn, Scorpio]
        ]
    );
    assert_eq!(
        [trimsamsa(1, 30, 0.1), trimsamsa(0, 0, 0.0)],
        [Scorpio, Aries]
    );
}

#[test]
fn hora_and_panchamsa_differ_for_odd_and_even_rasis() {
    use Rasi::*;
    let hora =
        [(0, 14), (0, 15), (1, 14), (1, 15)].map(|(r, d)| varga(Varga::Hora, dms(r, d, 0, 0.0)));
    assert_eq!(hora, [Leo, Cancer, Cancer, Leo]);
    let panchamsa = |r| [0, 6, 12, 18, 24].map(|d| varga(Varga::Panchamsa, dms(r, d, 0, 0.0)));
    assert_eq!(panchamsa(0), [Aries, Aquarius, Sagittarius, Gemini, Libra]);
    assert_eq!(panchamsa(1), [Taurus, Virgo, Pisces, Capricorn, Scorpio]);
}

#[test]
fn each_varga_group_names_an_amsa_from_a_count_of_two_up_to_all_its_vargas() {
    use Amsa::*;
    use VargaGroup::*;
    let amsas = |group: VargaGroup| (0..=17).map(|count| group.amsa(count)).collect::<Vec<_>>();
    let named = |names: &[Amsa]| {
        [None, None]
            .into_iter()
            .chain(names.iter().copied().map(Some))
            .chain([None].repeat(16 - names.len()))
            .collect::<Vec<_>>()
    };
    // §6.6.1–6.6.4.
    let sapta = [
        Kimsukaamsa,
        Vyanjanaamsa,
        Chaamaraamsa,
        Chatraamsa,
        Kundalaamsa,
        Mukutaamsa,
    ];
    assert_eq!(amsas(Shadvarga), named(&sapta[..5]));
    assert_eq!(amsas(SaptaVarga), named(&sapta));
    assert_eq!(
        amsas(DasaVarga),
        named(&[
            Paarijaataamsa,
            Uttamaamsa,
            Gopuraamsa,
            Simhaasanaamsa,
            Paaraavataamsa,
            Devalokaamsa,
            Brahmalokamsa,
            Airaavataamsa,
            Sreedhaamaamsa,
        ])
    );
    assert_eq!(
        amsas(ShodasaVarga),
        named(&[
            Bhedakaamsa,
            Kusumaamsa,
            Nagapurushaamsa,
            Kandukaamsa,
            Keralaamsa,
            Kalpavrikshaamsa,
            Chandanavanaamsa,
            Poornachandraamsa,
            Uchchaisravaamsa,
            Dhanvantaryamsa,
            Sooryakaantaamsa,
            Vidrumaamsa,
            Indraasanaamsa,
            Golokaamsa,
            SreeVallabhaamsa,
        ])
    );
    assert_eq!(VARGA_GROUPS.map(|g| g.vargas().len()), [6, 7, 10, 16]);
}

#[test]
fn the_lagna_has_no_amsabala() {
    assert_eq!(
        amsabala(Body::Lagna, 255.0, VargaGroup::ShodasaVarga),
        (0, None)
    );
}

/// A chart with the lagna and Sun to Rahu at `g`, all direct; a paaka lagna reads no speeds.
fn placed(lagna: f64, g: [f64; 8]) -> Chart {
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

#[test]
fn grahas_at_the_same_arcsecond_share_a_karakatwa_and_the_next_is_left_empty() {
    use CharaKaraka::*;
    // §8.2: an exact tie shares a karakatwa and the next has no ruler, not even a sthira karaka (ADR 0011, ADR 0018). Moon
    // and Mars tie for 2nd.
    let g = [
        dms(0, 29, 0, 0.0),
        dms(1, 20, 0, 30.0),
        dms(2, 20, 0, 30.0),
        dms(3, 15, 0, 0.0),
        dms(4, 10, 0, 0.0),
        dms(5, 5, 0, 0.0),
        dms(6, 1, 0, 0.0),
        dms(7, 29, 30, 0.0),
    ];
    let karakas = placed(0.0, g).chara_karakas().map(|(_, k)| k);
    assert_eq!(
        karakas,
        [Atma, Amatya, Amatya, Matri, Pitri, Putra, Jnaati, Dara]
    );
    assert!(!karakas.contains(&Bhratri));
}

#[test]
fn rahu_at_the_start_of_a_rasi_is_the_atma_karaka() {
    // ADR 0011: Rahu at 0°00′00″ has advanced 30°, ahead of any graha at 29°59′59″.
    let mut g = [dms(0, 29, 59, 59.0); 8];
    g[7] = dms(3, 0, 0, 0.0);
    let karakas = placed(0.0, g).chara_karakas();
    assert_eq!(karakas[7], (Body::Rahu, CharaKaraka::Atma));
    assert!(karakas[..7].iter().all(|&(_, k)| k == CharaKaraka::Amatya));
}

#[test]
fn rasi_drishti_follows_the_examples_of_section_10_3() {
    use Rasi::*;
    // Ch. 10 §10.3: the rasis Ar, Ta, Ge and Li are said to aspect.
    for (from, aspected) in [
        (Aries, [Leo, Scorpio, Aquarius]),
        (Taurus, [Cancer, Libra, Capricorn]),
        (Gemini, [Virgo, Sagittarius, Pisces]),
        (Libra, [Aquarius, Taurus, Leo]),
    ] {
        for to in RASIS {
            assert_eq!(
                rasi_drishti(from, to),
                aspected.contains(&to),
                "{from:?} {to:?}"
            );
        }
    }
}

#[test]
fn rasi_drishti_is_symmetric() {
    // §10.3: "sign Y will aspect sign X if sign X aspects sign Y".
    for a in RASIS {
        for b in RASIS {
            assert_eq!(rasi_drishti(a, b), rasi_drishti(b, a), "{a:?} {b:?}");
        }
    }
}

#[test]
fn graha_drishti_follows_the_examples_of_section_10_2() {
    use Rasi::*;
    // §10.2's list of 7th-house examples.
    assert_eq!(graha_drishti(Body::Sun, Taurus), [Scorpio]);
    assert!(graha_drishti(Body::Mars, Gemini).contains(&Sagittarius));
    assert_eq!(graha_drishti(Body::Moon, Leo), [Aquarius]);
    assert!(graha_drishti(Body::Jupiter, Pisces).contains(&Virgo));
    assert!(graha_drishti(Body::Saturn, Capricorn).contains(&Cancer));
    // §10.2: Jupiter in Ta aspects Cp, "the 9th house from Ta".
    assert!(graha_drishti(Body::Jupiter, Taurus).contains(&Capricorn));
}

#[test]
fn graha_drishti_is_example_34_in_count_order() {
    use Rasi::*;
    assert_eq!(
        graha_drishti(Body::Jupiter, Gemini),
        [Libra, Sagittarius, Aquarius]
    );
    assert_eq!(graha_drishti(Body::Mars, Leo), [Scorpio, Aquarius, Pisces]);
    assert_eq!(
        graha_drishti(Body::Saturn, Sagittarius),
        [Aquarius, Gemini, Virgo]
    );
}

#[test]
fn the_nodes_and_the_lagna_have_no_graha_drishti() {
    // ADR 0013: Exercise 14 leaves out Rahu and Ketu, despite §10.2's "all planets".
    for body in [Body::Rahu, Body::Ketu, Body::Lagna] {
        assert!(graha_drishti(body, Rasi::Aries).is_empty());
    }
}

#[test]
fn rasi_drishtis_lists_section_10_3s_examples_in_count_order() {
    use Rasi::*;
    assert_eq!(rasi_drishtis(Aries), [Leo, Scorpio, Aquarius]);
    assert_eq!(rasi_drishtis(Taurus), [Cancer, Libra, Capricorn]);
    assert_eq!(rasi_drishtis(Gemini), [Virgo, Sagittarius, Pisces]);
    // "a planet in Libra will aspect the houses and planets in Aq, Ta and Le".
    assert_eq!(rasi_drishtis(Libra), [Aquarius, Taurus, Leo]);
}

#[test]
fn grahas_in_gives_the_aspected_saturn_of_section_10_2() {
    use Rasi::*;
    let chart = scene(
        Aries,
        &[(Body::Saturn, mid(Capricorn)), (Body::Jupiter, mid(Taurus))],
    );
    // §10.2: "Jupiter in Ta will aspect Saturn in Cp".
    let aspected: Vec<Body> = graha_drishti(Body::Jupiter, Taurus)
        .into_iter()
        .flat_map(|r| chart.in_varga(Varga::RasiChart).grahas_in(r))
        .collect();
    assert_eq!(aspected, [Body::Saturn]);
}

#[test]
fn grahas_in_runs_in_the_books_order_and_ignores_the_lagna() {
    use Rasi::*;
    let chart = scene(
        Aries,
        &[
            (Body::Saturn, mid(Leo)),
            (Body::Lagna, mid(Leo)),
            (Body::Sun, mid(Leo)),
        ],
    );
    assert_eq!(
        chart.in_varga(Varga::RasiChart).grahas_in(Leo),
        [Body::Sun, Body::Saturn]
    );
    let lagna_alone = scene(Aries, &[(Body::Lagna, mid(Virgo))]);
    assert!(
        lagna_alone
            .in_varga(Varga::RasiChart)
            .grahas_in(Virgo)
            .is_empty()
    );
}

#[test]
fn argala_follows_the_example_of_section_10_6() {
    use Body::*;
    use Rasi::*;
    // Everything else in Scorpio, which Gemini's argalas don't count, Ketu moved there off its place opposite Rahu.
    let chart = scene(
        Scorpio,
        &[
            (Mercury, mid(Gemini)),
            (Jupiter, mid(Pisces)),
            (Venus, mid(Aries)),
            (Saturn, mid(Virgo)),
            (Ketu, mid(Scorpio)),
        ],
    );
    let [second, fourth, fifth, eleventh] = chart.in_varga(Varga::RasiChart).argala(Gemini);
    assert_eq!((fourth.from, fourth.obstructed_from), (4, 10));
    assert_eq!(fourth.argala, [Saturn]);
    assert_eq!(fourth.virodhargala, [Jupiter]);
    assert_eq!((eleventh.from, eleventh.obstructed_from), (11, 3));
    assert_eq!(eleventh.argala, [Venus]);
    assert!(eleventh.virodhargala.is_empty());
    for empty in [second, fifth] {
        assert!(empty.argala.is_empty() && empty.virodhargala.is_empty());
    }
}

#[test]
fn argala_on_a_rasi_holding_ketu_counts_anti_zodiacally_as_in_section_10_6s_note() {
    use Body::*;
    use Rasi::*;
    // Le, Ge, Ta, Sc are the 2nd, 4th, 5th, 11th from Vi in reverse; Li, Sg, Cp, Cn the 12th, 10th, 9th, 3rd.
    let chart = scene(
        Pisces,
        &[
            (Ketu, mid(Virgo)),
            (Sun, mid(Leo)),
            (Moon, mid(Gemini)),
            (Venus, mid(Taurus)),
            (Mars, mid(Scorpio)),
            (Jupiter, mid(Libra)),
            (Mercury, mid(Sagittarius)),
            (Saturn, mid(Capricorn)),
            (Rahu, mid(Cancer)),
        ],
    );
    let rows = chart.in_varga(Varga::RasiChart).argala(Virgo);
    let cells = rows.iter().map(|a| {
        (
            a.from,
            a.argala.clone(),
            a.obstructed_from,
            a.virodhargala.clone(),
        )
    });
    assert!(cells.eq([
        (2, vec![Sun], 12, vec![Jupiter]),
        (4, vec![Moon], 10, vec![Mercury]),
        (5, vec![Venus], 9, vec![Saturn]),
        (11, vec![Mars], 3, vec![Rahu]),
    ]));
}

#[test]
fn ketu_causing_argala_on_another_rasi_does_not_reverse_it() {
    use Rasi::*;
    // Ketu in Cancer, the 2nd from Gemini; Saturn in Virgo stays the 4th.
    let chart = scene(
        Scorpio,
        &[(Body::Ketu, mid(Cancer)), (Body::Saturn, mid(Virgo))],
    );
    let [second, fourth, ..] = chart.in_varga(Varga::RasiChart).argala(Gemini);
    assert_eq!(second.argala, [Body::Ketu]);
    assert_eq!(fourth.argala, [Body::Saturn]);
}

#[test]
fn argalas_reversal_follows_ketus_rasi_in_the_varga_not_in_the_rasi_chart() {
    use Rasi::*;
    // Ketu is in Aries in D-1 and Leo in D-9; Mars at 27° Pisces is in Pisces in both, the Sun at 2° Pisces in Cancer in D-9.
    let chart = scene(
        Gemini,
        &[
            (Body::Ketu, mid(Aries)),
            (Body::Mars, dms(Pisces as u8, 27, 0, 0.0)),
            (Body::Sun, dms(Pisces as u8, 2, 0, 0.0)),
        ],
    );
    let [second, ..] = chart.in_varga(Varga::RasiChart).argala(Aries);
    assert!(second.argala.contains(&Body::Mars));
    let [second, ..] = chart.in_varga(Varga::Navamsa).argala(Aries);
    assert!(!second.argala.contains(&Body::Mars));
    assert!(second.virodhargala.contains(&Body::Mars));
    let [second, ..] = chart.in_varga(Varga::Navamsa).argala(Leo);
    assert!(second.argala.contains(&Body::Sun));
}

#[test]
fn a_lagna_in_a_counted_rasi_causes_no_argala() {
    use Rasi::*;
    let chart = scene(
        Scorpio,
        &[(Body::Ketu, mid(Scorpio)), (Body::Lagna, mid(Cancer))],
    );
    for a in chart.in_varga(Varga::RasiChart).argala(Gemini) {
        assert!(a.argala.is_empty() && a.virodhargala.is_empty());
    }
}

#[test]
fn a_rasis_own_grahas_never_appear_in_its_argalas() {
    for r in RASIS {
        // All nine in `r`, Ketu either opposite Rahu's place or with them: counted forwards and in reverse.
        for chart in [scene(r, &[]), scene(r, &[(Body::Ketu, mid(r))])] {
            let rows = chart.in_varga(Varga::RasiChart).argala(r);
            for a in rows {
                assert!(a.argala.is_empty() && a.virodhargala.is_empty(), "{r:?}");
            }
        }
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

#[test]
fn the_father_and_mother_are_the_stronger_by_each_rule_of_section_15_5_1() {
    use Body::*;
    use Rasi::*;
    use StrongerRule::*;
    let at = |r: Rasi, deg: u8| dms(r as u8, deg, 0, 0.0);
    // Everything else in Gemini at 15°, whose duals aspect only duals (§10.3).
    let cases = [
        // (1): Venus alone, the Sun with the other seven.
        (Sun, Venus, vec![(Venus, mid(Aries))], Sun, Joined),
        // (2): Virgo is aspected by Jupiter and Mercury, its lord, from Gemini; Leo by none, and the Sun is his own dispositor.
        (
            Sun,
            Venus,
            vec![(Sun, mid(Leo)), (Venus, mid(Virgo))],
            Venus,
            JupiterMercuryDispositor,
        ),
        // (3): neither movable rasi is aspected from Gemini; the Sun is exalted in Aries.
        (
            Sun,
            Venus,
            vec![(Sun, mid(Aries)), (Venus, mid(Libra))],
            Sun,
            Exalted,
        ),
        // (4): Leo is fixed, Cancer movable, neither aspected from Gemini.
        (
            Sun,
            Venus,
            vec![(Sun, mid(Cancer)), (Venus, mid(Leo))],
            Venus,
            Mobility,
        ),
        // (5b): together in Sagittarius, owned by neither and exalting neither.
        (
            Sun,
            Venus,
            vec![(Sun, at(Sagittarius, 10)), (Venus, at(Sagittarius, 20))],
            Venus,
            Advancement,
        ),
        (Moon, Mars, vec![(Mars, mid(Aries))], Moon, Joined),
        (
            Moon,
            Mars,
            vec![(Moon, mid(Virgo)), (Mars, mid(Leo))],
            Moon,
            JupiterMercuryDispositor,
        ),
        // (3): Mars is exalted in Capricorn; the Moon is her own dispositor in Cancer.
        (
            Moon,
            Mars,
            vec![(Moon, mid(Cancer)), (Mars, mid(Capricorn))],
            Mars,
            Exalted,
        ),
        (
            Moon,
            Mars,
            vec![(Moon, mid(Leo)), (Mars, mid(Cancer))],
            Moon,
            Mobility,
        ),
        (
            Moon,
            Mars,
            vec![(Moon, at(Sagittarius, 20)), (Mars, at(Sagittarius, 10))],
            Moon,
            Advancement,
        ),
    ];
    for (a, b, places, graha, rule) in cases {
        let row = usize::from(a == Moon);
        let karaka = scene(Gemini, &places).sthira_karakas()[row];
        assert_eq!(
            (karaka.graha, karaka.rule),
            (graha, Some(rule)),
            "{a:?}/{b:?} {places:?}"
        );
    }
}

#[test]
fn a_tie_through_rule_5b_takes_the_sun_and_the_moon_and_the_other_five_are_fixed() {
    use Body::*;
    use Kin::*;
    let karakas = scene(Rasi::Gemini, &[]).sthira_karakas();
    let rows = karakas.map(|k| (k.kin, k.graha, k.rule));
    assert_eq!(
        rows,
        [
            (Father, Sun, None),
            (Mother, Moon, None),
            (YoungerSiblings, Mars, None),
            (MaternalRelatives, Mercury, None),
            (Husband, Jupiter, None),
            (Wife, Venus, None),
            (ElderSiblings, Saturn, None),
        ]
    );
}

fn lord_of(chart: &Chart, r: Rasi) -> Body {
    chart.in_varga(Varga::RasiChart).stronger_lord(r)
}

#[test]
fn a_bhava_arudha_follows_the_example_of_section_9_2() {
    // A Gemini bhava with Mercury in Aquarius: 9 rasis, then 9 on, to Libra.
    let chart = scene(Rasi::Gemini, &[(Body::Mercury, mid(Rasi::Aquarius))]);
    assert_eq!(
        chart.in_varga(Varga::RasiChart).bhava_arudhas()[0],
        Rasi::Libra
    );
}

#[test]
fn a_graha_arudha_follows_the_example_of_section_9_5() {
    // The Sun in Gemini: 3 rasis to Leo, then 3 on, to Libra.
    let chart = scene(Rasi::Gemini, &[]);
    assert_eq!(
        chart.in_varga(Varga::RasiChart).graha_arudhas()[0],
        (Body::Sun, Rasi::Libra)
    );
}

/// Rahu's and Ketu's arudha padas with Rahu in `rahu` and Ketu in `ketu`.
fn node_arudha_padas(rahu: Rasi, ketu: Rasi) -> (Rasi, Rasi) {
    let at = [(Body::Rahu, mid(rahu)), (Body::Ketu, mid(ketu))];
    let [.., (_, rahu), (_, ketu)] = scene(Rasi::Aries, &at)
        .in_varga(Varga::RasiChart)
        .graha_arudhas();
    (rahu, ketu)
}

#[test]
fn ketu_7th_from_rahu_shares_rahus_arudha_pada() {
    // §9.5's note.
    for rahu in RASIS {
        let (a, b) = node_arudha_padas(rahu, kth(rahu, 7));
        assert_eq!(a, b, "{rahu:?}");
    }
}

#[test]
fn the_nodes_together_have_opposite_arudha_padas() {
    // §9.5's note.
    for rasi in RASIS {
        let (a, b) = node_arudha_padas(rasi, rasi);
        assert_eq!(bhava(a, b), 7, "{rasi:?}");
    }
}

#[test]
fn a_rasi_with_one_lord_has_that_lord() {
    let chart = scene(Rasi::Gemini, &[]);
    for r in RASIS {
        if !matches!(r, Rasi::Scorpio | Rasi::Aquarius) {
            assert_eq!(lord_of(&chart, r), r.lord(), "{r:?}");
        }
    }
}

#[test]
fn basic_rule_a_co_lord_in_the_rasi_gives_way_to_the_other() {
    use Body::*;
    use Rasi::*;
    // §15.5.1's basic rule, in either order.
    for (inside, other, r) in [
        (Saturn, Rahu, Aquarius),
        (Rahu, Saturn, Aquarius),
        (Mars, Ketu, Scorpio),
        (Ketu, Mars, Scorpio),
    ] {
        assert_eq!(lord_of(&scene(Gemini, &[(inside, mid(r))]), r), other);
    }
}

#[test]
fn both_co_lords_in_the_rasi_go_through_the_rules() {
    use Body::*;
    use Rasi::*;
    // "Exactly one" is the basic rule; here 5b decides, Rahu's 25° over Saturn's 20°, then Saturn's 20° over Rahu's 15°.
    let at = [(Saturn, dms(10, 20, 0, 0.0)), (Rahu, dms(10, 5, 0, 0.0))];
    assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Rahu);
    let at = [(Saturn, dms(10, 20, 0, 0.0)), (Rahu, dms(10, 15, 0, 0.0))];
    assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Saturn);
}

#[test]
fn rule_1_saturn_joined_by_more_planets_beats_rahu() {
    use Body::*;
    use Rasi::*;
    // §15.5.1 rule 1: Saturn in Pi with Mars and Sun, Rahu in Ar with Jupiter.
    let at = [
        (Saturn, mid(Pisces)),
        (Mars, mid(Pisces)),
        (Sun, mid(Pisces)),
        (Rahu, mid(Aries)),
        (Jupiter, mid(Aries)),
    ];
    assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Saturn);
}

#[test]
fn a_lagna_with_a_co_lord_does_not_count_as_joining_it() {
    use Body::*;
    use Rasi::*;
    // ADR 0012. Were the lagna a planet Rahu would win rule 1; Saturn wins rule 4 instead.
    let at = [
        (Saturn, mid(Pisces)),
        (Rahu, mid(Aries)),
        (Lagna, mid(Aries)),
    ];
    assert_eq!(lord_of(&scene(Taurus, &at), Aquarius), Saturn);
}

#[test]
fn rule_2_saturn_counts_mercury_as_dispositor_twice() {
    use Body::*;
    use Rasi::*;
    // §15.5.1 rule 2: Saturn in Ge with Mercury is 2, Rahu in Ar aspected by Mars from Le is 1. The book leaves rule 1
    // to tie, so the Sun joins Rahu.
    let at = [
        (Saturn, mid(Gemini)),
        (Mercury, mid(Gemini)),
        (Rahu, mid(Aries)),
        (Sun, mid(Aries)),
        (Mars, mid(Leo)),
        (Jupiter, mid(Taurus)),
    ];
    assert_eq!(lord_of(&scene(Capricorn, &at), Aquarius), Saturn);

    // Counted once Mercury would only tie, and Rahu, as dual and more advanced, would win.
    let at = [
        (Saturn, mid(Virgo)),
        (Mercury, mid(Virgo)),
        (Rahu, dms(8, 1, 0, 0.0)),
        (Sun, mid(Sagittarius)),
    ];
    assert_eq!(lord_of(&scene(Capricorn, &at), Aquarius), Saturn);
}

#[test]
fn a_co_lord_that_is_its_own_dispositor_gets_no_count() {
    use Body::*;
    use Rasi::*;
    // ADR 0012. Saturn's Capricorn gives him 0 to Rahu's 1 from Mars; counting himself, he would win 5b.
    let at = [
        (Saturn, dms(9, 25, 0, 0.0)),
        (Rahu, dms(0, 25, 0, 0.0)),
        (Mars, mid(Leo)),
    ];
    assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Rahu);
}

#[test]
fn a_co_lord_in_scorpio_counts_both_of_its_dispositors() {
    use Body::*;
    use Rasi::*;
    // ADR 0012. Saturn in Sc has Mars and Ketu, 2, to Rahu's 1 from Jupiter; with Mars alone, Rahu would win rule 4.
    let at = [
        (Saturn, mid(Scorpio)),
        (Mars, mid(Cancer)),
        (Ketu, mid(Aries)),
        (Rahu, mid(Virgo)),
        (Jupiter, mid(Sagittarius)),
    ];
    assert_eq!(lord_of(&scene(Taurus, &at), Aquarius), Saturn);
}

#[test]
fn rule_3_exalted_saturn_beats_rahu() {
    use Body::*;
    use Rasi::*;
    // §15.5.1 rule 3: Saturn in Li, Rahu in Cn. Rahu at 5° has 25° to Saturn's 15°, so only exaltation gives it to Saturn.
    let at = [(Saturn, mid(Libra)), (Rahu, dms(3, 5, 0, 0.0))];
    assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Saturn);
}

#[test]
fn the_nodes_are_exalted_in_gemini_and_sagittarius() {
    use Body::*;
    use Rasi::*;
    // ADR 0012, by whole rasi: at 25° each has advanced only 5°, and their rivals 29° in a dual rasi too.
    let at = [(Rahu, dms(2, 25, 0, 0.0)), (Saturn, dms(5, 29, 0, 0.0))];
    assert_eq!(lord_of(&scene(Taurus, &at), Aquarius), Rahu);
    let at = [(Ketu, dms(8, 25, 0, 0.0)), (Mars, dms(11, 29, 0, 0.0))];
    assert_eq!(lord_of(&scene(Taurus, &at), Scorpio), Ketu);
}

#[test]
fn rule_4_mars_in_a_dual_rasi_beats_ketu_in_a_fixed_one() {
    use Body::*;
    use Rasi::*;
    // §15.5.1 rule 4: Mars in Ge, Ketu in Aq. Ketu is the more advanced, so only the rasi gives it to Mars.
    let at = [(Mars, mid(Gemini)), (Ketu, dms(10, 1, 0, 0.0))];
    assert_eq!(lord_of(&scene(Taurus, &at), Scorpio), Mars);
}

#[test]
fn rule_5b_ketu_more_advanced_from_the_rasis_end_beats_mars() {
    use Body::*;
    use Rasi::*;
    // §15.5.1 rule 5b: Mars at 23Li17 has 23°17′, Ketu at 5Cn54 has 24°06′.
    let at = [(Mars, dms(6, 23, 17, 0.0)), (Ketu, dms(3, 5, 54, 0.0))];
    assert_eq!(lord_of(&scene(Gemini, &at), Scorpio), Ketu);
}

#[test]
fn a_last_tie_goes_to_the_seven_planet_lord() {
    use Body::*;
    use Rasi::*;
    // ADR 0012's shortcut: both at 15° of a movable rasi, with nothing else to tell them apart.
    let at = [(Mars, mid(Libra)), (Ketu, mid(Cancer))];
    assert_eq!(lord_of(&scene(Gemini, &at), Scorpio), Mars);
    let at = [(Saturn, mid(Cancer)), (Rahu, mid(Aries))];
    assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), Saturn);
}

/// `winner` is the stronger rasi whichever is named first.
#[track_caller]
fn assert_stronger(chart: &Chart, winner: Rasi, loser: Rasi) {
    let d = Varga::RasiChart;
    assert_eq!(chart.in_varga(d).stronger_rasi(winner, loser), winner);
    assert_eq!(chart.in_varga(d).stronger_rasi(loser, winner), winner);
}

#[test]
fn rule_1_the_rasi_with_more_planets_is_stronger() {
    use Body::*;
    use Rasi::*;
    // §15.5.2 rule 1: Saturn and Jupiter in Ar, Venus in Li.
    let at = [
        (Saturn, mid(Aries)),
        (Jupiter, mid(Aries)),
        (Venus, mid(Libra)),
    ];
    assert_stronger(&scene(Gemini, &at), Aries, Libra);
}

#[test]
fn a_lagna_in_a_rasi_does_not_count_as_a_planet_in_it() {
    use Body::*;
    use Rasi::*;
    // ADR 0012. Were the lagna a planet rule 1 would tie, and the last tie would give Aries.
    let at = [(Lagna, mid(Aries)), (Moon, mid(Libra))];
    assert_stronger(&scene(Gemini, &at), Libra, Aries);
}

#[test]
fn rule_2_the_rasi_with_more_of_jupiter_mercury_and_its_lord_is_stronger() {
    use Body::*;
    use Rasi::*;
    // §15.5.2 rule 2: Ar has Jupiter, Li is aspected by Mercury and Venus from Ta. The book leaves rule 1 to tie, so
    // the Moon is in Li.
    let at = [
        (Jupiter, mid(Aries)),
        (Mercury, mid(Taurus)),
        (Venus, mid(Taurus)),
        (Mars, mid(Virgo)),
        (Moon, mid(Libra)),
    ];
    assert_stronger(&scene(Gemini, &at), Libra, Aries);
}

#[test]
fn rahus_aspect_counts_for_aquarius_though_saturn_is_its_stronger_lord() {
    use Body::*;
    use Rasi::*;
    // Exercise 26(5), ADR 0012. Saturn is the lord by rule 1, so without Rahu's aspect rule 6 would give Le to the Sun.
    let at = [(Sun, dms(2, 25, 0, 0.0)), (Rahu, mid(Libra))];
    let chart = scene(Gemini, &at);
    assert_eq!(lord_of(&chart, Aquarius), Saturn);
    assert_stronger(&chart, Aquarius, Leo);
}

#[test]
fn rule_3_the_rasi_with_an_exalted_planet_is_stronger() {
    use Body::*;
    use Rasi::*;
    // §15.5.2 rule 3: Saturn and Mercury in Li, Jupiter and Venus in Ar.
    let at = [
        (Saturn, mid(Libra)),
        (Mercury, mid(Libra)),
        (Jupiter, mid(Aries)),
        (Venus, mid(Aries)),
    ];
    assert_stronger(&scene(Gemini, &at), Libra, Aries);
}

#[test]
fn rule_3_reads_exaltation_by_whole_rasi() {
    use Body::*;
    use Rasi::*;
    // ADR 0012: Mercury at 25° Vi is own rasi by §3.3's degrees, yet exalted here. Each rasi counts 3 in rule 2, and rule 6 would give Pi.
    let at = [
        (Mercury, dms(5, 25, 0, 0.0)),
        (Jupiter, dms(11, 28, 0, 0.0)),
    ];
    assert_stronger(&scene(Taurus, &at), Virgo, Pisces);
}

#[test]
fn rule_4_the_rasi_whose_lord_is_in_a_rasi_of_other_oddity_is_stronger() {
    use Body::*;
    use Rasi::*;
    // §15.5.2 rule 4: Venus in Cn, Mars in Le. Mars aspects Ar, so the book's tie needs Jupiter aspecting Li from Ta.
    let at = [
        (Venus, mid(Cancer)),
        (Mars, mid(Leo)),
        (Jupiter, mid(Taurus)),
    ];
    assert_stronger(&scene(Gemini, &at), Libra, Aries);
}

#[test]
fn a_rule_4_tie_on_a_grahas_own_rasis_goes_on_to_rule_5() {
    use Body::*;
    use Rasi::*;
    // ADR 0012: Scorpio's lord is Ketu, by rule 1. Mars in odd Le and Ketu in even Cn, both like the rasi they rule.
    let at = [(Mars, mid(Leo)), (Ketu, mid(Cancer)), (Moon, mid(Cancer))];
    let chart = scene(Gemini, &at);
    assert_eq!(lord_of(&chart, Scorpio), Ketu);
    assert_stronger(&chart, Scorpio, Aries);

    // Both unlike their rasi: with Mars as Scorpio's lord too, rule 4 would give Aries. Jupiter in Le evens rule 2.
    let at = [
        (Mars, mid(Cancer)),
        (Ketu, mid(Gemini)),
        (Moon, mid(Gemini)),
        (Jupiter, mid(Leo)),
    ];
    let chart = scene(Pisces, &at);
    assert_eq!(lord_of(&chart, Scorpio), Ketu);
    assert_stronger(&chart, Scorpio, Aries);
}

#[test]
fn rule_6_the_rasi_whose_lord_is_more_advanced_is_stronger() {
    use Body::*;
    use Rasi::*;
    // §15.5.2 rule 6: Mars at 23Ge17 over Venus at 19Le51. Venus aspects Li, so the book's tie needs Jupiter aspecting
    // Ar from Sc. Then the other way round, which the last tie could not give.
    let at = |mars, venus| [(Mars, mars), (Venus, venus), (Jupiter, mid(Scorpio))];
    let chart = scene(Gemini, &at(dms(2, 23, 17, 0.0), dms(4, 19, 51, 0.0)));
    assert_stronger(&chart, Aries, Libra);
    let chart = scene(Gemini, &at(dms(2, 19, 51, 0.0), dms(4, 23, 17, 0.0)));
    assert_stronger(&chart, Libra, Aries);
}

#[test]
fn rule_6_takes_the_stronger_lord_and_measures_rahu_from_the_rasis_end() {
    use Body::*;
    use Rasi::*;
    // §15.5.2 rule 6's note. Rahu at 5Ge joined by the rest is Aquarius's lord with 25° to the Sun's 20°.
    let at = [
        (Rahu, dms(2, 5, 0, 0.0)),
        (Sun, dms(2, 20, 0, 0.0)),
        (Saturn, mid(Taurus)),
    ];
    let chart = scene(Gemini, &at);
    assert_eq!(lord_of(&chart, Aquarius), Rahu);
    assert_stronger(&chart, Aquarius, Leo);
}

#[test]
fn a_last_tie_goes_to_the_first_rasi_counted_from_aries() {
    use Rasi::*;
    // ADR 0012's shortcut: Mars and Venus at the same degree in Ge, nothing else to tell Ar and Li apart.
    assert_stronger(&scene(Gemini, &[]), Aries, Libra);
}

#[test]
fn an_earlier_rule_wins_though_the_next_rule_says_otherwise() {
    use Body::*;
    use Rasi::*;
    // Rule 1 over 2, 2 over 3, 3 over 4 of §15.5.1.
    for (at, lord) in [
        (
            vec![
                (Saturn, mid(Cancer)),
                (Sun, mid(Cancer)),
                (Rahu, mid(Libra)),
                (Venus, mid(Taurus)),
            ],
            Saturn,
        ),
        (
            vec![(Saturn, mid(Libra)), (Rahu, mid(Aries)), (Mars, mid(Leo))],
            Rahu,
        ),
        (vec![(Saturn, mid(Libra)), (Rahu, mid(Leo))], Saturn),
    ] {
        assert_eq!(lord_of(&scene(Gemini, &at), Aquarius), lord);
    }
    // Rule 2 over 3, 3 over 4, 4 over 5, 5 over 6 of §15.5.2.
    for (at, winner, loser) in [
        (
            vec![(Saturn, mid(Libra)), (Jupiter, mid(Aries))],
            Aries,
            Libra,
        ),
        (
            vec![(Saturn, mid(Libra)), (Moon, mid(Aries)), (Mars, mid(Virgo))],
            Libra,
            Aries,
        ),
        (vec![(Mars, mid(Virgo))], Aries, Leo),
        (vec![(Mars, dms(2, 25, 0, 0.0))], Leo, Aries),
    ] {
        assert_stronger(&scene(Gemini, &at), winner, loser);
    }
}

#[test]
fn a_rule_tries_the_main_owner_then_a_seven_planet_co_lord_at_home() {
    use Body::*;
    use Rasi::*;
    let tried =
        |at: &[(Body, f64)], r| scene(Gemini, at).in_varga(Varga::RasiChart).lords_to_try(r);
    assert_eq!(tried(&[(Saturn, mid(Aquarius))], Aquarius), [Rahu, Saturn]);
    assert_eq!(tried(&[(Mars, mid(Scorpio))], Scorpio), [Ketu, Mars]);
    // A node at home is not tried.
    assert_eq!(tried(&[(Rahu, mid(Aquarius))], Aquarius), [Saturn]);
    assert_eq!(tried(&[(Ketu, mid(Scorpio))], Scorpio), [Mars]);
    // Neither, or both, at home: as lords_in_yoga has them.
    assert_eq!(tried(&[], Aquarius), [Saturn, Rahu]);
    let both = [(Saturn, mid(Aquarius)), (Rahu, mid(Aquarius))];
    assert_eq!(tried(&both, Aquarius), [Saturn, Rahu]);
    assert_eq!(tried(&[], Aries), [Mars]);
}

#[test]
fn saturn_at_home_in_an_aquarius_lagna_can_form_a_yoga_but_rahu_at_home_cannot() {
    use Body::*;
    use Rasi::*;
    let khadga = |at: &[(Body, Rasi)]| {
        let mut at = at.to_vec();
        at.extend([(Jupiter, Libra), (Venus, Pisces)]);
        from_lagna(Aquarius, Gemini, &at, YogaName::Khadga)
    };
    // Rahu in the 3rd fails as main owner, so Saturn in the lagna forms it.
    let row = khadga(&[(Saturn, Aquarius), (Rahu, Aries)]);
    assert_eq!(present(row), [Jupiter, Venus, Saturn]);
    // Saturn in the 3rd is the only lord tried, though Rahu is in the lagna.
    assert_eq!(khadga(&[(Rahu, Aquarius), (Saturn, Aries)]), None);
    // The main owner is named when he works: Rahu in the 5th.
    let row = khadga(&[(Saturn, Aquarius), (Rahu, Gemini)]);
    assert_eq!(present(row), [Jupiter, Venus, Rahu]);
}

#[test]
fn a_yogas_lord_is_the_co_lord_outside_the_rasi_or_either() {
    use Body::*;
    use Rasi::*;
    let lords = |at: &[(Body, f64)], r| {
        scene(Gemini, at)
            .in_varga(Varga::RasiChart)
            .lords_in_yoga(r)
    };
    // Parasara, p. 218, and chart 18's Rahu as 5th lord with Saturn in Aquarius (ADR 0014).
    assert_eq!(lords(&[(Saturn, mid(Aquarius))], Aquarius), [Rahu]);
    assert_eq!(lords(&[(Rahu, mid(Aquarius))], Aquarius), [Saturn]);
    assert_eq!(lords(&[], Aquarius), [Saturn, Rahu]);
    assert_eq!(
        lords(&[(Saturn, mid(Aquarius)), (Rahu, mid(Aquarius))], Aquarius),
        [Saturn, Rahu]
    );
    assert_eq!(lords(&[(Mars, mid(Scorpio))], Scorpio), [Ketu]);
    assert_eq!(lords(&[], Scorpio), [Mars, Ketu]);
    // A rasi with one lord keeps it, in the rasi or not.
    assert_eq!(lords(&[], Aries), [Mars]);
    assert_eq!(lords(&[(Mars, mid(Aries))], Aries), [Mars]);
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

#[test]
fn each_row_of_table_30_puts_each_planet_in_one_column_but_the_moon_for_ar_li_cp() {
    for (r, (yogakaraka, columns)) in RASIS.into_iter().zip(TABLE_30) {
        for planet in PLANETS {
            let count = columns.iter().filter(|c| c.contains(&planet)).count();
            let moon_left_out =
                planet == Body::Moon && matches!(r, Rasi::Aries | Rasi::Libra | Rasi::Capricorn);
            assert_eq!(count, usize::from(!moon_left_out), "{r:?} {planet:?}");
        }
        assert!(
            columns
                .iter()
                .copied()
                .flatten()
                .all(|g| PLANETS.contains(g)),
            "{r:?}"
        );
        assert!(yogakaraka.is_none_or(|y| columns[0].contains(&y)), "{r:?}");
    }
}

#[test]
fn the_moon_for_an_aries_lagna_is_a_functional_malefic_waxing_and_neutral_waning() {
    use Body::*;
    // The Sun at 15° Ar: the Moon 30° ahead is waxing, 30° behind waning.
    let nature = |moon| {
        scene(Rasi::Aries, &[(Moon, mid(moon))])
            .in_varga(Varga::RasiChart)
            .functional_nature(Moon, ReferencePoint::Body(Lagna))
    };
    assert_eq!(nature(Rasi::Taurus), Some(FunctionalNature::Malefic));
    assert_eq!(nature(Rasi::Pisces), Some(FunctionalNature::Neutral));
}

#[test]
fn functional_nature_and_the_yogakaraka_count_from_the_reference_point_and_the_nodes_have_none() {
    use Body::*;
    let chart = scene(Rasi::Aries, &[(Moon, mid(Rasi::Taurus))]);
    let d1 = chart.in_varga(Varga::RasiChart);
    let (lagna, moon) = (ReferencePoint::Body(Lagna), ReferencePoint::Body(Moon));
    // Saturn rules the 10th and 11th from Ar, the 9th and 10th from Ta.
    assert_eq!(
        d1.functional_nature(Saturn, lagna),
        Some(FunctionalNature::Malefic)
    );
    assert_eq!(
        d1.functional_nature(Saturn, moon),
        Some(FunctionalNature::Benefic)
    );
    assert_eq!(d1.yogakaraka(lagna), None);
    assert_eq!(d1.yogakaraka(moon), Some(Saturn));
    for g in [Rahu, Ketu, Lagna] {
        assert_eq!(d1.functional_nature(g, lagna), None, "{g:?}");
    }
}

fn yoga_in(chart: &Chart, d: Varga, name: YogaName) -> Option<Yoga> {
    chart
        .in_varga(d)
        .yogas()
        .into_iter()
        .find(|y| y.name == name)
}

/// `name`'s row in the rasi chart of a scene with everything at 15° of `rest` but for `at`.
fn yoga_of(rest: Rasi, at: &[(Body, Rasi)], name: YogaName) -> Option<Yoga> {
    let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
    yoga_in(&scene(rest, &at), Varga::RasiChart, name)
}

fn grahas_of(rest: Rasi, at: &[(Body, Rasi)], name: YogaName) -> Option<Vec<Body>> {
    yoga_of(rest, at, name).map(|y| bodies(&y.grahas))
}

#[test]
fn vesi_is_a_planet_other_than_the_moon_in_the_suns_2nd() {
    use Body::*;
    use Rasi::*;
    let at = [(Sun, Gemini), (Jupiter, Cancer), (Mercury, Cancer)];
    let vesi = yoga_of(Aquarius, &at, YogaName::Vesi).unwrap();
    assert_eq!(vesi.name.class(), YogaClass::Ravi);
    assert_eq!(bodies(&vesi.grahas), [Mercury, Jupiter]);
    assert_eq!(vesi.to_judge, []);
    let at = [(Sun, Gemini), (Moon, Cancer)];
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Vesi), None);
}

#[test]
fn a_node_alone_in_the_suns_2nd_or_12th_forms_nothing() {
    use Body::*;
    use Rasi::*;
    let at = [(Sun, Gemini), (Rahu, Cancer), (Ketu, Taurus)];
    for name in [YogaName::Vesi, YogaName::Vosi, YogaName::Ubhayachara] {
        assert_eq!(grahas_of(Aquarius, &at, name), None, "{name:?}");
    }
}

#[test]
fn vosi_is_a_planet_other_than_the_moon_in_the_suns_12th() {
    use Body::*;
    use Rasi::*;
    let at = [(Sun, Aries), (Jupiter, Pisces), (Venus, Pisces)];
    assert_eq!(
        grahas_of(Aquarius, &at, YogaName::Vosi),
        Some(vec![Jupiter, Venus])
    );
    let at = [(Sun, Aries), (Moon, Pisces)];
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Vosi), None);
}

#[test]
fn ubhayachara_replaces_vesi_and_vosi_with_planets_on_both_sides() {
    use Body::*;
    use Rasi::*;
    let at = [(Sun, Cancer), (Mars, Leo), (Venus, Gemini)];
    assert_eq!(
        grahas_of(Aquarius, &at, YogaName::Ubhayachara),
        Some(vec![Mars, Venus])
    );
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Vesi), None);
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Vosi), None);
    let at = [(Sun, Cancer), (Mars, Leo)];
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Ubhayachara), None);
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Vesi), Some(vec![Mars]));
}

#[test]
fn budha_aaditya_is_the_sun_and_mercury_in_one_rasi_however_close() {
    use Body::*;
    use Rasi::*;
    let at = [(Sun, Scorpio), (Mercury, Scorpio)];
    assert_eq!(
        grahas_of(Aquarius, &at, YogaName::BudhaAaditya),
        Some(vec![Sun, Mercury])
    );
    let at = [(Sun, Scorpio), (Mercury, Libra)];
    assert_eq!(grahas_of(Aquarius, &at, YogaName::BudhaAaditya), None);
    // §11.2.4's combustion is a note on its power, not a condition (ADR 0014).
    let close = scene(
        Aquarius,
        &[(Sun, dms(7, 10, 0, 0.0)), (Mercury, dms(7, 10, 6, 0.0))],
    );
    let row = yoga_in(&close, Varga::RasiChart, YogaName::BudhaAaditya).unwrap();
    assert_eq!(row.to_judge, []);
}

#[test]
fn a_yoga_in_a_varga_reads_the_vargas_places() {
    use Body::*;
    use Rasi::*;
    // Aries 1° and 5° share Aries in the rasi chart, not D-9: Aries and Taurus.
    let split = scene(
        Aquarius,
        &[(Sun, dms(0, 1, 0, 0.0)), (Mercury, dms(0, 5, 0, 0.0))],
    );
    assert!(yoga_in(&split, Varga::RasiChart, YogaName::BudhaAaditya).is_some());
    assert_eq!(
        yoga_in(&split, Varga::Navamsa, YogaName::BudhaAaditya),
        None
    );
    // Aries 1° and Leo 1° each begin a fiery rasi's navamsas, at Aries.
    let joined = scene(
        Aquarius,
        &[(Sun, dms(0, 1, 0, 0.0)), (Mercury, dms(4, 1, 0, 0.0))],
    );
    assert_eq!(
        yoga_in(&joined, Varga::RasiChart, YogaName::BudhaAaditya),
        None
    );
    assert!(yoga_in(&joined, Varga::Navamsa, YogaName::BudhaAaditya).is_some());
}

#[test]
fn sunaphaa_is_a_planet_other_than_the_sun_in_the_moons_2nd() {
    use Body::*;
    use Rasi::*;
    let at = [(Moon, Gemini), (Jupiter, Cancer), (Mercury, Cancer)];
    let row = yoga_of(Aquarius, &at, YogaName::Sunaphaa).unwrap();
    assert_eq!(row.name.class(), YogaClass::Chandra);
    assert_eq!(bodies(&row.grahas), [Mercury, Jupiter]);
    for other in [Sun, Rahu] {
        let at = [(Moon, Gemini), (other, Cancer)];
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Sunaphaa), None);
    }
}

#[test]
fn anaphaa_is_a_planet_other_than_the_sun_in_the_moons_12th() {
    use Body::*;
    use Rasi::*;
    let at = [(Moon, Aries), (Jupiter, Pisces), (Venus, Pisces)];
    assert_eq!(
        grahas_of(Aquarius, &at, YogaName::Anaphaa),
        Some(vec![Jupiter, Venus])
    );
    for other in [Sun, Ketu] {
        let at = [(Moon, Aries), (other, Pisces)];
        assert_eq!(grahas_of(Aquarius, &at, YogaName::Anaphaa), None);
    }
}

#[test]
fn duradhara_replaces_sunaphaa_and_anaphaa_with_planets_on_both_sides() {
    use Body::*;
    use Rasi::*;
    let at = [(Moon, Cancer), (Mars, Leo), (Venus, Gemini)];
    assert_eq!(
        grahas_of(Aquarius, &at, YogaName::Duradhara),
        Some(vec![Mars, Venus])
    );
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Sunaphaa), None);
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Anaphaa), None);
    let at = [(Moon, Cancer), (Venus, Gemini)];
    assert_eq!(grahas_of(Aquarius, &at, YogaName::Duradhara), None);
    assert_eq!(
        grahas_of(Aquarius, &at, YogaName::Anaphaa),
        Some(vec![Venus])
    );
}

/// Kemadruma's row with the book's example, a Taurus lagna and the Moon in Virgo, everything else in Capricorn but for `more`.
fn kemadruma_with(more: &[(Body, Rasi)]) -> Option<Vec<Body>> {
    let mut at = vec![(Body::Lagna, Rasi::Taurus), (Body::Moon, Rasi::Virgo)];
    at.extend_from_slice(more);
    grahas_of(Rasi::Capricorn, &at, YogaName::Kemadruma)
}

#[test]
fn kemadruma_is_no_planet_beside_the_moon_nor_in_a_kendra_from_the_lagna() {
    use Body::*;
    use Rasi::*;
    assert_eq!(kemadruma_with(&[]), Some(vec![]));
    // The Sun does not count beside the Moon, but does in a kendra.
    assert_eq!(kemadruma_with(&[(Sun, Libra)]), Some(vec![]));
    assert_eq!(kemadruma_with(&[(Sun, Taurus)]), None);
    // Mars in the Moon's 1st, 2nd and 12th, and in the other kendras.
    for rasi in [Virgo, Libra, Leo, Taurus, Scorpio, Aquarius] {
        assert_eq!(kemadruma_with(&[(Mars, rasi)]), None, "{rasi:?}");
    }
    // The Moon itself in a kendra counts for nothing.
    let at = [(Lagna, Taurus), (Moon, Leo)];
    assert_eq!(grahas_of(Capricorn, &at, YogaName::Kemadruma), Some(vec![]));
}

#[test]
fn a_node_counts_in_none_of_kemadrumas_checks() {
    use Body::*;
    use Rasi::*;
    // In a kendra, then beside the Moon (ADR 0014).
    assert_eq!(
        kemadruma_with(&[(Rahu, Taurus), (Ketu, Scorpio)]),
        Some(vec![])
    );
    assert_eq!(
        kemadruma_with(&[(Rahu, Libra), (Ketu, Virgo)]),
        Some(vec![])
    );
}

#[test]
fn chandra_mangala_is_the_moon_and_mars_in_one_rasi() {
    use Body::*;
    use Rasi::*;
    let row = yoga_of(
        Aquarius,
        &[(Moon, Virgo), (Mars, Virgo)],
        YogaName::ChandraMangala,
    );
    assert_eq!(row.map(|y| bodies(&y.grahas)), Some(vec![Moon, Mars]));
    let at = [(Moon, Virgo), (Mars, Libra)];
    assert_eq!(grahas_of(Aquarius, &at, YogaName::ChandraMangala), None);
}

/// Adhi's row with the Moon in Taurus and Jupiter and Venus at the given rasis.
fn adhi_with(jupiter: Rasi, venus: Rasi, more: &[(Body, Rasi)]) -> Option<Yoga> {
    let mut at = vec![
        (Body::Moon, Rasi::Taurus),
        (Body::Jupiter, jupiter),
        (Body::Venus, venus),
    ];
    at.extend_from_slice(more);
    yoga_of(Rasi::Aquarius, &at, YogaName::Adhi)
}

#[test]
fn adhi_is_two_or_more_natural_benefics_in_the_moons_6th_7th_and_8th() {
    use Body::*;
    use Rasi::*;
    // All three: across the places, then all in one (ADR 0014).
    let at = [(Mercury, Sagittarius)];
    let row = adhi_with(Libra, Scorpio, &at).unwrap();
    assert_eq!(row.name.class(), YogaClass::Chandra);
    assert_eq!(bodies(&row.grahas), [Mercury, Jupiter, Venus]);
    assert_eq!(row.to_judge, []);
    let row = adhi_with(Libra, Libra, &[(Mercury, Libra)]).unwrap();
    assert_eq!(bodies(&row.grahas), [Mercury, Jupiter, Venus]);
    // Any two, with the third outside: a saumya Mercury in the 11th and the 9th, then Venus and Jupiter outside.
    for rasi in [Pisces, Capricorn] {
        let row = adhi_with(Libra, Scorpio, &[(Mercury, rasi)]).unwrap();
        assert_eq!(bodies(&row.grahas), [Jupiter, Venus]);
        assert_eq!(row.to_judge, []);
    }
    let row = adhi_with(Libra, Cancer, &at).unwrap();
    assert_eq!(bodies(&row.grahas), [Mercury, Jupiter]);
    let row = adhi_with(Capricorn, Scorpio, &at).unwrap();
    assert_eq!(bodies(&row.grahas), [Mercury, Venus]);
    // One alone is not enough: Mercury, Jupiter, Venus, with a kroora Mercury left in Aquarius for the last two.
    assert_eq!(adhi_with(Cancer, Pisces, &at), None);
    assert_eq!(adhi_with(Libra, Cancer, &[]), None);
    assert_eq!(adhi_with(Cancer, Scorpio, &[]), None);
    // The rule, not its example's misprinted 5th and 4th (ADR 0014).
    assert_eq!(adhi_with(Virgo, Leo, &[(Mercury, Virgo)]), None);
}

#[test]
fn adhi_leaves_out_a_kroora_mercury() {
    use Body::*;
    use Rasi::*;
    let beside_mars = [(Mercury, Scorpio), (Mars, Scorpio)];
    // Jupiter and Venus alone form it, with him in the places or among the malefics of Aquarius outside them (ADR 0014).
    for more in [&beside_mars[..], &[]] {
        let row = adhi_with(Libra, Sagittarius, more).unwrap();
        assert_eq!(bodies(&row.grahas), [Jupiter, Venus]);
        assert_eq!(row.to_judge, []);
    }
    // He does not stand in for either of them.
    assert_eq!(adhi_with(Libra, Cancer, &beside_mars), None);
    assert_eq!(adhi_with(Capricorn, Cancer, &beside_mars), None);
}

#[test]
fn a_kroora_mercury_outside_adhi_keeps_his_bhadra() {
    use Body::*;
    use Rasi::*;
    let at = [
        (Lagna, Gemini),
        (Moon, Pisces),
        (Jupiter, Leo),
        (Mercury, Virgo),
        (Mars, Virgo),
        (Sun, Libra),
        (Venus, Libra),
        (Saturn, Libra),
    ];
    assert_eq!(
        grahas_of(Aquarius, &at, YogaName::Adhi),
        Some(vec![Jupiter, Venus])
    );
    assert_eq!(
        grahas_of(Aquarius, &at, YogaName::Bhadra),
        Some(vec![Mercury])
    );
}

#[test]
fn adhi_is_to_judge_only_if_a_tied_mercury_is_the_second_benefic() {
    use Body::*;
    use Rasi::*;
    // Tied in the places with Jupiter or Venus beside him and the other outside: it holds only if he is saumya.
    let row = adhi_with(Libra, Cancer, &[(Mercury, Libra), (Sun, Libra)]).unwrap();
    assert_eq!(bodies(&row.grahas), [Mercury, Jupiter]);
    assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
    let row = adhi_with(Cancer, Libra, &[(Mercury, Libra), (Sun, Libra)]).unwrap();
    assert_eq!(bodies(&row.grahas), [Mercury, Venus]);
    assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
    // With both in the places, it holds both ways, and the saumya reading lists him.
    let row = adhi_with(Libra, Scorpio, &[(Mercury, Libra), (Sun, Libra)]).unwrap();
    assert_eq!(bodies(&row.grahas), [Mercury, Jupiter, Venus]);
    assert_eq!(row.to_judge, []);
    // Tied outside them by the waxing Moon and Mars: it holds both ways without him.
    let more = [(Sun, Aries), (Mars, Taurus), (Mercury, Taurus)];
    let row = adhi_with(Libra, Scorpio, &more).unwrap();
    assert_eq!(bodies(&row.grahas), [Jupiter, Venus]);
    assert_eq!(row.to_judge, []);
}

#[test]
fn a_yogas_class_is_the_section_of_ch_11_listing_it() {
    use YogaClass as C;
    use YogaName as N;
    for (name, class) in [
        (N::Vesi, C::Ravi),
        (N::BudhaAaditya, C::Ravi),
        (N::Sunaphaa, C::Chandra),
        (N::Adhi, C::Chandra),
        (N::Ruchaka, C::Mahaapurusha),
        (N::Hamsa, C::Mahaapurusha),
        (N::Rajju, C::Naabhasa),
        (N::Vajra, C::Naabhasa),
        (N::Chaapa, C::Naabhasa),
        (N::Gola, C::Naabhasa),
        (N::Subha, C::OtherPopular),
        (N::Ravi, C::OtherPopular),
        (N::Chapa, C::OtherPopular),
        (N::Vimala, C::OtherPopular),
        (N::Tapaswi, C::OtherPopular),
        (N::BasicRaaja, C::Raaja),
        (N::VipareetaRaaja, C::Raaja),
        (N::Raaja(1), C::Raaja),
        (N::RaajaSambandha(15), C::RaajaSambandha),
        (N::BasicDhana, C::Dhana),
        (N::Dhana(12), C::Dhana),
        (N::Daridra(13), C::Daridra),
    ] {
        assert_eq!(name.class(), class, "{name:?}");
    }
}

#[test]
fn a_rule_differing_on_a_tied_mercury_is_to_judge_whichever_way_it_holds() {
    use Disposition::{Kroora, Saumya};
    let tied = with_mercury(&[Body::Sun, Body::Jupiter]);
    let each_way = |rule: fn(Disposition) -> Option<Vec<Body>>| {
        tied.in_varga(Varga::RasiChart)
            .with_mercury_each_way(YogaName::Maalaa, rule)
            .map(|y| (bodies(&y.grahas), y.to_judge))
    };
    let judged = Some((vec![Body::Mercury], vec![ToJudge::MercurysNature]));
    assert_eq!(
        each_way(|m| (m == Saumya).then(|| vec![Body::Mercury])),
        judged
    );
    assert_eq!(
        each_way(|m| (m == Kroora).then(|| vec![Body::Mercury])),
        judged
    );
    assert_eq!(each_way(|_| None), None);
    // Agreeing on the yoga but not its grahas, the saumya reading's stand (ADR 0014).
    assert_eq!(
        each_way(|m| Some(vec![if m == Saumya {
            Body::Mercury
        } else {
            Body::Sun
        }])),
        Some((vec![Body::Mercury], vec![]))
    );
    // Off a tie, only his nature is read.
    let kroora = with_mercury(&[Body::Sun, Body::Mars]);
    let row = kroora
        .in_varga(Varga::RasiChart)
        .with_mercury_each_way(YogaName::Maalaa, |m| {
            (m == Kroora).then(|| vec![Body::Mercury])
        })
        .unwrap();
    assert_eq!(bodies(&row.grahas), [Body::Mercury]);
    assert_eq!(row.to_judge, []);
}

#[test]
fn mahaapurusha_yogas_are_a_planet_in_own_or_uchcha_in_a_kendra_from_the_lagna() {
    use Body::*;
    use Rasi::*;
    // §11.4.1–11.4.5's examples.
    for (name, lagna, graha, rasi) in [
        (YogaName::Ruchaka, Libra, Mars, Aries),
        (YogaName::Bhadra, Gemini, Mercury, Virgo),
        (YogaName::Sasa, Capricorn, Saturn, Libra),
        (YogaName::Maalavya, Gemini, Venus, Pisces),
        (YogaName::Hamsa, Sagittarius, Jupiter, Pisces),
    ] {
        let row = yoga_of(Gemini, &[(Lagna, lagna), (graha, rasi)], name).unwrap();
        assert_eq!(row.name.class(), YogaClass::Mahaapurusha);
        assert_eq!(bodies(&row.grahas), [graha]);
    }
    // The rest of each planet's own and uchcha rasis.
    for (name, lagna, graha, rasi) in [
        (YogaName::Ruchaka, Libra, Mars, Capricorn),
        (YogaName::Ruchaka, Taurus, Mars, Scorpio),
        (YogaName::Bhadra, Gemini, Mercury, Gemini),
        (YogaName::Sasa, Aries, Saturn, Capricorn),
        (YogaName::Sasa, Aquarius, Saturn, Aquarius),
        (YogaName::Maalavya, Taurus, Venus, Taurus),
        (YogaName::Maalavya, Cancer, Venus, Libra),
        (YogaName::Hamsa, Aries, Jupiter, Cancer),
        (YogaName::Hamsa, Virgo, Jupiter, Sagittarius),
    ] {
        let at = [(Lagna, lagna), (graha, rasi)];
        assert_eq!(grahas_of(Gemini, &at, name), Some(vec![graha]), "{name:?}");
    }
    // For each, its own or uchcha rasi outside a kendra, then a kendra outside them.
    for (name, lagna, graha, rasi) in [
        (YogaName::Ruchaka, Gemini, Mars, Aries),
        (YogaName::Ruchaka, Libra, Mars, Cancer),
        (YogaName::Bhadra, Taurus, Mercury, Gemini),
        (YogaName::Bhadra, Gemini, Mercury, Sagittarius),
        (YogaName::Sasa, Gemini, Saturn, Aquarius),
        (YogaName::Sasa, Capricorn, Saturn, Aries),
        (YogaName::Maalavya, Gemini, Venus, Libra),
        (YogaName::Maalavya, Gemini, Venus, Virgo),
        (YogaName::Hamsa, Gemini, Jupiter, Cancer),
        (YogaName::Hamsa, Cancer, Jupiter, Capricorn),
    ] {
        let at = [(Lagna, lagna), (graha, rasi)];
        assert_eq!(grahas_of(Gemini, &at, name), None, "{name:?} {rasi:?}");
    }
}

#[test]
fn a_mahaapurusha_yoga_in_a_varga_counts_from_that_vargas_lagna() {
    use Body::*;
    // Pisces 1° is Cancer in D-9 and Aries 1° is Aries: the 2nd in the rasi chart, the 10th in D-9.
    let chart = scene(
        Rasi::Gemini,
        &[(Lagna, dms(11, 1, 0, 0.0)), (Mars, dms(0, 1, 0, 0.0))],
    );
    assert_eq!(yoga_in(&chart, Varga::RasiChart, YogaName::Ruchaka), None);
    assert!(yoga_in(&chart, Varga::Navamsa, YogaName::Ruchaka).is_some());
}

/// The Naabhasa rows of a chart with its lagna in Aries, the seven planets in these houses in the book's order, and the nodes in `nodes`.
fn naabhasa_rows(houses: [u8; 7], nodes: u8) -> Vec<Yoga> {
    let house = |h: u8| mid(kth(Rasi::Aries, h as usize));
    let mut at = vec![
        (Body::Lagna, mid(Rasi::Aries)),
        (Body::Rahu, house(nodes)),
        (Body::Ketu, house(nodes)),
    ];
    at.extend(PLANETS.into_iter().zip(houses.map(house)));
    scene(Rasi::Aries, &at)
        .in_varga(Varga::RasiChart)
        .yogas()
        .into_iter()
        .filter(|y| y.name.class() == YogaClass::Naabhasa)
        .collect()
}

fn naabhasa_names(houses: [u8; 7], nodes: u8) -> Vec<YogaName> {
    naabhasa_rows(houses, nodes)
        .into_iter()
        .map(|y| y.name)
        .collect()
}

/// A house none of the seven planets is in, so the nodes stand outside the shape: Rahu and Ketu are not among "all the planets" (ADR 0014).
fn empty_house(houses: [u8; 7]) -> u8 {
    (1..=12).find(|h| !houses.contains(h)).unwrap()
}

#[test]
fn aasraya_yogas_are_all_seven_in_movable_fixed_or_dual_rasis() {
    use YogaName::*;
    // §11.5.1. From an Aries lagna, the 1st, 4th, 7th and 10th are chara, the next sthira, the last dwiswabhava.
    for (name, houses, broken) in [
        (Rajju, [1, 4, 7, 10, 1, 4, 7], [1, 4, 7, 10, 1, 4, 2]),
        (Musala, [2, 5, 8, 11, 2, 5, 8], [2, 5, 8, 11, 2, 5, 3]),
        (Nala, [3, 6, 9, 12, 3, 6, 9], [3, 6, 9, 12, 3, 6, 1]),
    ] {
        let row = naabhasa_rows(houses, empty_house(houses))
            .into_iter()
            .find(|y| y.name == name)
            .unwrap();
        assert_eq!(bodies(&row.grahas), PLANETS);
        assert_eq!(row.to_judge, []);
        assert!(!naabhasa_names(broken, empty_house(broken)).contains(&name));
    }
}

#[test]
fn aakriti_shapes_hold_all_seven_with_every_place_occupied() {
    use YogaName::*;
    // §11.5.3, each with the charts forming it, then the charts that do not.
    type Charts = &'static [[u8; 7]];
    let cases: [(YogaName, Charts, Charts); 16] = [
        (
            Gadaa,
            &[
                [1, 1, 1, 1, 4, 4, 4],
                [4, 4, 4, 7, 7, 7, 7],
                [7, 7, 7, 10, 10, 10, 10],
                [10, 10, 10, 1, 1, 1, 1],
            ],
            &[[1; 7], [1, 1, 1, 1, 4, 4, 7], [1, 1, 1, 7, 7, 7, 7]],
        ),
        (
            Sakata,
            &[[1, 1, 1, 1, 7, 7, 7]],
            &[[1; 7], [1, 1, 1, 7, 7, 7, 4]],
        ),
        (
            Vihanga,
            &[[4, 4, 4, 10, 10, 10, 10]],
            &[[4; 7], [4, 4, 4, 10, 10, 10, 1]],
        ),
        (
            Sringaataka,
            &[[1, 1, 1, 5, 5, 9, 9]],
            &[[1, 1, 1, 1, 5, 5, 5], [1, 1, 5, 5, 9, 9, 2]],
        ),
        (
            Hala,
            &[
                [2, 2, 2, 6, 6, 10, 10],
                [3, 3, 3, 7, 7, 11, 11],
                [4, 4, 4, 8, 8, 12, 12],
            ],
            // Trikonas from the lagna are Sringaataka's.
            &[
                [1, 1, 1, 5, 5, 9, 9],
                [2, 2, 2, 2, 6, 6, 6],
                [2, 2, 6, 6, 10, 10, 3],
            ],
        ),
        (
            Yoopa,
            &[[1, 2, 3, 4, 1, 2, 3]],
            // §11.5.3 with the 4th left empty, then a planet outside.
            &[[1, 1, 1, 2, 2, 3, 3], [1, 2, 3, 4, 1, 2, 5]],
        ),
        (
            Sara,
            &[[4, 5, 6, 7, 4, 5, 6]],
            &[[4, 4, 4, 5, 5, 6, 6], [4, 5, 6, 7, 4, 5, 8]],
        ),
        (
            Sakti,
            &[[7, 8, 9, 10, 7, 8, 9]],
            &[[7, 7, 7, 8, 8, 9, 9], [7, 8, 9, 10, 7, 8, 11]],
        ),
        (
            Danda,
            &[[10, 11, 12, 1, 10, 11, 12]],
            &[[10, 10, 10, 11, 11, 12, 12], [10, 11, 12, 1, 10, 11, 2]],
        ),
        (
            Naukaa,
            &[[1, 2, 3, 4, 5, 6, 7]],
            &[[1, 1, 2, 3, 4, 5, 6], [1, 2, 3, 4, 5, 6, 8]],
        ),
        (
            Koota,
            &[[4, 5, 6, 7, 8, 9, 10]],
            &[[4, 4, 5, 6, 7, 8, 9], [4, 5, 6, 7, 8, 9, 11]],
        ),
        (
            Chatra,
            &[[7, 8, 9, 10, 11, 12, 1]],
            &[[7, 7, 8, 9, 10, 11, 12], [7, 8, 9, 10, 11, 12, 2]],
        ),
        (
            Chaapa,
            &[[10, 11, 12, 1, 2, 3, 4]],
            &[[10, 10, 11, 12, 1, 2, 3], [10, 11, 12, 1, 2, 3, 5]],
        ),
        (
            Ardhachandra,
            // The seven rasis from each panaphara, then each apoklima.
            &[
                [2, 3, 4, 5, 6, 7, 8],
                [5, 6, 7, 8, 9, 10, 11],
                [8, 9, 10, 11, 12, 1, 2],
                [11, 12, 1, 2, 3, 4, 5],
                [3, 4, 5, 6, 7, 8, 9],
                [6, 7, 8, 9, 10, 11, 12],
                [9, 10, 11, 12, 1, 2, 3],
                [12, 1, 2, 3, 4, 5, 6],
            ],
            // From a kendra are the four above; six of the seven rasis.
            &[
                [1, 2, 3, 4, 5, 6, 7],
                [4, 5, 6, 7, 8, 9, 10],
                [7, 8, 9, 10, 11, 12, 1],
                [10, 11, 12, 1, 2, 3, 4],
                [2, 2, 3, 4, 5, 6, 7],
            ],
        ),
        (
            Chakra,
            &[[1, 3, 5, 7, 9, 11, 1]],
            &[[1, 3, 5, 7, 9, 1, 1], [1, 3, 5, 7, 9, 11, 2]],
        ),
        (
            Samudra,
            &[[2, 4, 6, 8, 10, 12, 2]],
            &[[2, 4, 6, 8, 10, 2, 2], [2, 4, 6, 8, 10, 12, 1]],
        ),
    ];
    for (name, forming, not_forming) in cases {
        for &houses in forming {
            let row = naabhasa_rows(houses, empty_house(houses))
                .into_iter()
                .find(|y| y.name == name);
            let row = row.unwrap_or_else(|| panic!("{name:?} {houses:?}"));
            assert_eq!(bodies(&row.grahas), PLANETS);
        }
        for &houses in not_forming {
            let names = naabhasa_names(houses, empty_house(houses));
            assert!(!names.contains(&name), "{name:?} {houses:?}");
        }
    }
}

#[test]
fn kamala_and_vaapi_only_need_all_seven_within_their_places() {
    use YogaName::*;
    let has = |houses, name| naabhasa_names(houses, empty_house(houses)).contains(&name);
    // §11.5.3 says "in", so one kendra is enough (ADR 0014).
    for houses in [[1; 7], [1, 1, 1, 4, 4, 4, 4], [1, 4, 7, 10, 1, 4, 7]] {
        assert!(has(houses, Kamala), "{houses:?}");
    }
    assert!(!has([1, 4, 7, 10, 1, 4, 2], Kamala));
    for houses in [[2; 7], [2, 5, 8, 11, 2, 5, 8], [3, 6, 9, 12, 3, 6, 9]] {
        assert!(has(houses, Vaapi), "{houses:?}");
    }
    // All in panapharas or all in apoklimas, not a mix of the two (ADR 0014).
    assert!(!has([2, 2, 2, 2, 3, 3, 3], Vaapi));
    assert!(!has([2, 5, 8, 11, 2, 5, 1], Vaapi));
}

#[test]
fn a_chart_may_show_more_than_one_shape() {
    use YogaName::*;
    let houses = [1, 1, 1, 1, 7, 7, 7];
    // Both kendras are chara from an Aries lagna, so its Aasraya yoga lists too.
    assert_eq!(naabhasa_names(houses, 2), [Rajju, Sakata, Kamala]);
}

#[test]
fn maalaa_is_three_kendras_held_by_saumya_grahas() {
    use Body::*;
    use Rasi::*;
    // §11.5.2's example.
    let example = [
        (Lagna, Aries),
        (Jupiter, Cancer),
        (Venus, Capricorn),
        (Mercury, Libra),
    ];
    let row = yoga_of(Taurus, &example, YogaName::Maalaa).unwrap();
    assert_eq!(bodies(&row.grahas), [Mercury, Jupiter, Venus]);
    assert_eq!(row.to_judge, []);
    let at = [(Lagna, Aries), (Jupiter, Cancer), (Venus, Capricorn)];
    assert_eq!(yoga_of(Taurus, &at, YogaName::Maalaa), None);
    let at = [
        (Lagna, Aries),
        (Jupiter, Cancer),
        (Venus, Capricorn),
        (Mercury, Libra),
        (Mars, Libra),
    ];
    assert_eq!(yoga_of(Taurus, &at, YogaName::Maalaa), None);
    // The Moon counts by paksha: waxing ahead of the Sun in Pisces, waning behind him in Taurus.
    let with_moon = |sun| {
        [
            (Lagna, Aries),
            (Moon, Aries),
            (Jupiter, Cancer),
            (Venus, Capricorn),
            (Sun, sun),
        ]
    };
    assert_eq!(
        grahas_of(Taurus, &with_moon(Pisces), YogaName::Maalaa),
        Some(vec![Moon, Jupiter, Venus])
    );
    assert_eq!(
        grahas_of(Taurus, &with_moon(Taurus), YogaName::Maalaa),
        None
    );
}

#[test]
fn sarpa_is_three_kendras_held_by_kroora_grahas_nodes_among_them() {
    use Body::*;
    use Rasi::*;
    // §11.5.2's example.
    let example = [
        (Lagna, Scorpio),
        (Mars, Taurus),
        (Rahu, Leo),
        (Ketu, Aquarius),
    ];
    let row = yoga_of(Sagittarius, &example, YogaName::Sarpa).unwrap();
    assert_eq!(bodies(&row.grahas), [Mars, Rahu, Ketu]);
    assert_eq!(row.to_judge, []);
    let at = [(Lagna, Scorpio), (Rahu, Leo), (Ketu, Aquarius)];
    assert_eq!(yoga_of(Sagittarius, &at, YogaName::Sarpa), None);
}

#[test]
fn the_opposite_nature_in_any_of_the_four_kendras_may_stop_a_dala_yoga_operating_well() {
    use Body::*;
    use Rasi::*;
    let judged = [ToJudge::MayNotOperateWell];
    // §11.5.2: Saturn in the Maalaa's 4th, then in the kendra it leaves empty, then Rahu there.
    for (graha, rasi) in [(Saturn, Cancer), (Saturn, Aries), (Rahu, Aries)] {
        let at = [
            (Lagna, Aries),
            (Jupiter, Cancer),
            (Venus, Capricorn),
            (Mercury, Libra),
            (graha, rasi),
        ];
        let row = yoga_of(Taurus, &at, YogaName::Maalaa).unwrap();
        assert_eq!(row.to_judge, judged, "{graha:?} {rasi:?}");
        assert_eq!(bodies(&row.grahas), [Mercury, Jupiter, Venus]);
    }
    // Jupiter in the Sarpa's 7th, then in the kendra it leaves empty, then Mercury there, saumya alone with nothing else to trip the note.
    for (graha, rasi) in [(Jupiter, Taurus), (Jupiter, Scorpio), (Mercury, Scorpio)] {
        let at = [
            (Lagna, Scorpio),
            (Mars, Taurus),
            (Rahu, Leo),
            (Ketu, Aquarius),
            (graha, rasi),
        ];
        let row = yoga_of(Sagittarius, &at, YogaName::Sarpa).unwrap();
        assert_eq!(row.to_judge, judged, "{graha:?} {rasi:?}");
    }
}

#[test]
fn a_tied_mercury_in_a_dala_kendra_leaves_only_the_opposite_nature_to_judge() {
    use Body::*;
    use Rasi::*;
    // Tied beside the Sun and Jupiter, with a waxing Moon: the saumya reading lists him (ADR 0014).
    let at = [
        (Lagna, Aries),
        (Sun, Cancer),
        (Jupiter, Cancer),
        (Mercury, Cancer),
        (Venus, Capricorn),
        (Moon, Libra),
    ];
    let row = yoga_of(Taurus, &at, YogaName::Maalaa).unwrap();
    assert_eq!(bodies(&row.grahas), [Moon, Mercury, Jupiter, Venus]);
    assert_eq!(row.to_judge, [ToJudge::MayNotOperateWell]);
    // Tied beside Ketu and Jupiter: the same saumya reading leaves him out of the Sarpa.
    let at = [
        (Lagna, Scorpio),
        (Mars, Taurus),
        (Rahu, Leo),
        (Ketu, Aquarius),
        (Jupiter, Aquarius),
        (Mercury, Aquarius),
    ];
    let row = yoga_of(Sagittarius, &at, YogaName::Sarpa).unwrap();
    assert_eq!(bodies(&row.grahas), [Mars, Rahu, Ketu]);
    assert_eq!(row.to_judge, [ToJudge::MayNotOperateWell]);
}

#[test]
fn vajra_and_yava_need_each_named_house_to_hold_its_nature() {
    use Body::*;
    use Rasi::*;
    // §11.5.3: saumya grahas in the 1st and 7th with kroora ones in the 4th and 10th, then the other way round.
    let vajra = [
        (Lagna, Aries),
        (Jupiter, Aries),
        (Venus, Libra),
        (Sun, Cancer),
        (Saturn, Capricorn),
    ];
    assert_eq!(
        grahas_of(Taurus, &vajra, YogaName::Vajra),
        Some(vec![Sun, Jupiter, Venus, Saturn])
    );
    assert_eq!(grahas_of(Taurus, &vajra, YogaName::Yava), None);
    let yava = [
        (Lagna, Aries),
        (Sun, Aries),
        (Saturn, Libra),
        (Jupiter, Cancer),
        (Venus, Capricorn),
    ];
    assert_eq!(
        grahas_of(Taurus, &yava, YogaName::Yava),
        Some(vec![Sun, Jupiter, Venus, Saturn])
    );
    assert_eq!(grahas_of(Taurus, &yava, YogaName::Vajra), None);
    // The nodes are kroora (ADR 0014).
    let at = [
        (Lagna, Aries),
        (Jupiter, Aries),
        (Venus, Libra),
        (Rahu, Cancer),
        (Ketu, Capricorn),
    ];
    assert_eq!(
        grahas_of(Taurus, &at, YogaName::Vajra),
        Some(vec![Jupiter, Venus, Rahu, Ketu])
    );
    // A house without its nature breaks it; another graha beside it does not, and is not listed (ADR 0014).
    let at = [
        (Lagna, Aries),
        (Jupiter, Aries),
        (Venus, Libra),
        (Sun, Cancer),
    ];
    assert_eq!(grahas_of(Taurus, &at, YogaName::Vajra), None);
    let mut at = vajra.to_vec();
    at.push((Mars, Aries));
    assert_eq!(
        grahas_of(Taurus, &at, YogaName::Vajra),
        Some(vec![Sun, Jupiter, Venus, Saturn])
    );
}

#[test]
fn sankhya_yogas_count_the_rasis_of_the_seven_planets() {
    use YogaName::*;
    // §11.5.4, each the only Naabhasa row of its chart, the nodes not counted.
    for (name, houses) in [
        (Veenaa, [2, 3, 5, 6, 8, 9, 12]),
        (Daama, [2, 2, 3, 5, 6, 8, 9]),
        (Paasa, [2, 2, 3, 3, 5, 6, 8]),
        (Kedaara, [2, 2, 2, 3, 3, 5, 6]),
        (Soola, [2, 2, 2, 3, 3, 3, 5]),
        (Yuga, [2, 2, 2, 2, 3, 3, 3]),
    ] {
        assert_eq!(naabhasa_names(houses, 11), [name]);
        assert_eq!(bodies(&naabhasa_rows(houses, 11)[0].grahas), PLANETS);
    }
}

#[test]
fn a_kroora_mercury_forms_sarpa_vajra_and_yava_and_is_listed() {
    use Body::*;
    use Rasi::*;
    // Beside the Sun alone he is kroora (§3.2.2).
    let at = [
        (Lagna, Scorpio),
        (Mars, Taurus),
        (Rahu, Leo),
        (Sun, Aquarius),
        (Mercury, Aquarius),
    ];
    let row = yoga_of(Sagittarius, &at, YogaName::Sarpa).unwrap();
    assert_eq!(bodies(&row.grahas), [Sun, Mars, Mercury, Rahu]);
    assert_eq!(row.to_judge, []);
    let at = [
        (Lagna, Aries),
        (Jupiter, Aries),
        (Venus, Libra),
        (Sun, Cancer),
        (Mercury, Cancer),
        (Saturn, Capricorn),
    ];
    assert_eq!(
        grahas_of(Taurus, &at, YogaName::Vajra),
        Some(vec![Sun, Mercury, Jupiter, Venus, Saturn])
    );
    let at = [
        (Lagna, Aries),
        (Sun, Aries),
        (Mercury, Aries),
        (Saturn, Libra),
        (Jupiter, Cancer),
        (Venus, Capricorn),
    ];
    assert_eq!(
        grahas_of(Taurus, &at, YogaName::Yava),
        Some(vec![Sun, Mercury, Jupiter, Venus, Saturn])
    );
}

#[test]
fn a_waning_moon_is_kroora_in_a_dala_yoga() {
    use Body::*;
    use Rasi::*;
    // Behind the Sun in Taurus she is the opposite nature on a Maalaa; ahead of him in Pisces she is one of its grahas.
    let maalaa = |sun| {
        [
            (Lagna, Aries),
            (Jupiter, Cancer),
            (Venus, Capricorn),
            (Mercury, Libra),
            (Moon, Aries),
            (Sun, sun),
        ]
    };
    let row = yoga_of(Taurus, &maalaa(Taurus), YogaName::Maalaa).unwrap();
    assert_eq!(bodies(&row.grahas), [Mercury, Jupiter, Venus]);
    assert_eq!(row.to_judge, [ToJudge::MayNotOperateWell]);
    let row = yoga_of(Taurus, &maalaa(Pisces), YogaName::Maalaa).unwrap();
    assert_eq!(bodies(&row.grahas), [Moon, Mercury, Jupiter, Venus]);
    assert_eq!(row.to_judge, []);
    // She holds the Sarpa's third kendra only waning, behind the Sun in Pisces and not ahead of him in Sagittarius.
    let sarpa = |sun| {
        [
            (Lagna, Scorpio),
            (Mars, Taurus),
            (Rahu, Leo),
            (Moon, Aquarius),
            (Sun, sun),
        ]
    };
    let row = yoga_of(Sagittarius, &sarpa(Pisces), YogaName::Sarpa).unwrap();
    assert_eq!(bodies(&row.grahas), [Moon, Mars, Rahu]);
    assert_eq!(row.to_judge, []);
    assert_eq!(
        yoga_of(Sagittarius, &sarpa(Sagittarius), YogaName::Sarpa),
        None
    );
}

#[test]
fn another_naabhasa_yoga_stops_each_sankhya_yoga_its_count_would_give() {
    use YogaName::*;
    // §11.5.4, each chart with that many rasis forming the other yoga.
    for (sankhya, other, houses) in [
        (Veenaa, Naukaa, [1, 2, 3, 4, 5, 6, 7]),
        (Daama, Chakra, [1, 3, 5, 7, 9, 11, 1]),
        (Paasa, Sarpa, [1, 2, 4, 2, 3, 3, 7]),
        (Kedaara, Yoopa, [1, 2, 3, 4, 1, 2, 3]),
        (Soola, Sringaataka, [1, 1, 1, 5, 5, 9, 9]),
        (Yuga, Sakata, [1, 1, 1, 1, 7, 7, 7]),
    ] {
        assert_eq!(
            sankhya_yoga(houses.map(|h| kth(Rasi::Aries, h as usize))),
            sankhya
        );
        let names = naabhasa_names(houses, 11);
        assert!(names.contains(&other), "{other:?}");
        assert!(!names.contains(&sankhya), "{sankhya:?}");
    }
}

#[test]
fn gola_is_the_count_of_one_rasi_but_its_aasraya_yoga_always_stops_it() {
    use YogaName::*;
    assert_eq!(sankhya_yoga([Rasi::Leo; 7]), Gola);
    assert_eq!(
        sankhya_yoga([
            Rasi::Aries,
            Rasi::Leo,
            Rasi::Leo,
            Rasi::Leo,
            Rasi::Leo,
            Rasi::Leo,
            Rasi::Leo
        ]),
        Yuga
    );
    for (house, aasraya) in [(1, Rajju), (2, Musala), (3, Nala)] {
        let names = naabhasa_names([house; 7], 11);
        assert!(names.contains(&aasraya), "{house}");
        assert!(!names.contains(&Gola), "{house}");
    }
}

#[test]
fn rama_lists_sarpa_to_judge_and_so_no_daama() {
    use Body::*;
    use Rasi::*;
    // Ch. 1 Example 1: Saturn, Mars and the Sun hold his 4th, 7th and 10th; Jupiter and a waxing Moon his 1st (ADR 0014).
    let at = [
        (Lagna, Cancer),
        (Sun, Aries),
        (Mercury, Taurus),
        (Ketu, Gemini),
        (Moon, Cancer),
        (Jupiter, Cancer),
        (Saturn, Libra),
        (Rahu, Sagittarius),
        (Mars, Capricorn),
        (Venus, Pisces),
    ];
    let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
    let rows: Vec<Yoga> = scene(Aries, &at)
        .in_varga(Varga::RasiChart)
        .yogas()
        .into_iter()
        .filter(|y| y.name.class() == YogaClass::Naabhasa)
        .collect();
    let rows: Vec<_> = rows
        .into_iter()
        .map(|y| (y.name, bodies(&y.grahas), y.to_judge, y.form, y.magnitude))
        .collect();
    assert_eq!(
        rows,
        [(
            YogaName::Sarpa,
            vec![Sun, Mars, Saturn],
            vec![ToJudge::MayNotOperateWell],
            None,
            None,
        )]
    );
}

#[test]
fn a_sankhya_row_stands_alone_and_a_chart_with_no_other_has_exactly_one() {
    use YogaName::*;
    const SANKHYA: [YogaName; 7] = [Veenaa, Daama, Paasa, Kedaara, Soola, Yuga, Gola];
    // A fixed stream of charts, clustered so that shapes form as well as scatter.
    let mut state: u32 = 221;
    let mut next = |n: u32| {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 16) % n
    };
    let (mut alone, mut others) = (0, 0);
    for _ in 0..20_000 {
        let spread = next(12) + 1;
        let houses: [u8; 7] = std::array::from_fn(|_| (next(spread) + 1) as u8);
        let rows = naabhasa_rows(houses, (next(12) + 1) as u8);
        assert!(
            rows.iter()
                .all(|y| !y.to_judge.contains(&ToJudge::MercurysNature)),
            "{houses:?}"
        );
        let (sankhya, rest): (Vec<YogaName>, Vec<YogaName>) = rows
            .into_iter()
            .map(|y| y.name)
            .partition(|n| SANKHYA.contains(n));
        if rest.is_empty() {
            let planets = houses.map(|h| kth(Rasi::Aries, h as usize));
            assert_eq!(sankhya, [sankhya_yoga(planets)], "{houses:?}");
            alone += 1;
        } else {
            assert_eq!(sankhya, [], "{houses:?} {rest:?}");
            others += 1;
        }
    }
    assert!(alone > 1000 && others > 1000, "{alone} {others}");
}

#[test]
fn naabhasa_yogas_in_a_varga_read_the_vargas_places() {
    use Body::*;
    use Rasi::*;
    // All seven in Aries in the rasi chart; Jupiter at 5° is in Taurus in D-9, the others at 1° in Aries.
    let mut at: Vec<(Body, f64)> = PLANETS.iter().map(|&g| (g, dms(0, 1, 0, 0.0))).collect();
    at.push((Jupiter, dms(0, 5, 0, 0.0)));
    let chart = scene(Aquarius, &at);
    assert!(yoga_in(&chart, Varga::RasiChart, YogaName::Rajju).is_some());
    assert_eq!(yoga_in(&chart, Varga::Navamsa, YogaName::Rajju), None);
    assert!(yoga_in(&chart, Varga::Navamsa, YogaName::Yuga).is_some());
    assert_eq!(chart.in_varga(Varga::RasiChart).rasi_of(Jupiter), Aries);
}

/// `name`'s row in the rasi chart of a scene with the lagna in Aries and the rest piled in the 3rd, Gemini, but for `at`.
fn from_aries(at: &[(Body, Rasi)], name: YogaName) -> Option<Yoga> {
    from_lagna(Rasi::Aries, Rasi::Gemini, at, name)
}

/// `name`'s row with the lagna at `lagna` and the rest piled in `rest`, but for `at`.
fn from_lagna(lagna: Rasi, rest: Rasi, at: &[(Body, Rasi)], name: YogaName) -> Option<Yoga> {
    let mut at = at.to_vec();
    at.push((Body::Lagna, lagna));
    yoga_of(rest, &at, name)
}

fn present(row: Option<Yoga>) -> Vec<Body> {
    let row = row.expect("the yoga is listed");
    assert_eq!(row.to_judge, []);
    assert_eq!(row.form, None);
    bodies(&row.grahas)
}

#[test]
fn subha_is_benefics_in_the_lagna_or_on_both_its_sides() {
    use Body::*;
    use Rasi::*;
    let subha = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Subha);
    let row = subha(&[(Jupiter, Aries)]);
    assert_eq!(
        row.as_ref().map(|y| y.name.class()),
        Some(YogaClass::OtherPopular)
    );
    assert_eq!(present(row), [Jupiter]);
    assert_eq!(
        present(subha(&[(Venus, Pisces), (Jupiter, Taurus)])),
        [Jupiter, Venus]
    );
    // One side, or a malefic in the lagna, is not it.
    assert_eq!(subha(&[(Venus, Pisces)]), None);
    assert_eq!(subha(&[(Venus, Pisces), (Saturn, Taurus)]), None);
    assert_eq!(subha(&[(Mars, Aries)]), None);
}

#[test]
fn asubha_is_malefics_in_the_lagna_or_on_both_its_sides() {
    use Body::*;
    use Rasi::*;
    let asubha = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Asubha);
    assert_eq!(present(asubha(&[(Mars, Aries)])), [Mars]);
    assert_eq!(present(asubha(&[(Rahu, Aries)])), [Rahu]);
    assert_eq!(
        present(asubha(&[(Saturn, Pisces), (Sun, Taurus)])),
        [Sun, Saturn]
    );
    assert_eq!(asubha(&[(Saturn, Pisces)]), None);
    assert_eq!(asubha(&[(Jupiter, Aries)]), None);
}

#[test]
fn the_moons_paksha_decides_subha_or_asubha() {
    use Body::*;
    use Rasi::*;
    let waxing = [(Moon, Aries), (Sun, Capricorn)];
    assert_eq!(present(from_aries(&waxing, YogaName::Subha)), [Moon]);
    assert_eq!(from_aries(&waxing, YogaName::Asubha), None);
    let waning = [(Moon, Aries)];
    assert_eq!(from_aries(&waning, YogaName::Subha), None);
    assert_eq!(present(from_aries(&waning, YogaName::Asubha)), [Moon]);
}

#[test]
fn a_tied_mercury_in_the_lagna_changes_neither_subha_nor_asubha() {
    use Body::*;
    use Rasi::*;
    let at = [(Mercury, Aries), (Sun, Aries), (Jupiter, Aries)];
    assert_eq!(
        present(from_aries(&at, YogaName::Subha)),
        [Mercury, Jupiter]
    );
    // The saumya reading's grahas stand (ADR 0014).
    assert_eq!(present(from_aries(&at, YogaName::Asubha)), [Sun]);
}

#[test]
fn gaja_kesari_is_jupiter_in_a_kendra_from_the_moon_with_a_benefic_on_him() {
    use Body::*;
    use Rasi::*;
    let gaja = |at: &[(Body, Rasi)]| from_aries(at, YogaName::GajaKesari);
    // Venus's aspect, then Mercury alone beside him; the Moon here is waning, so no benefic herself.
    let row = gaja(&[(Moon, Aries), (Jupiter, Cancer), (Venus, Capricorn)]);
    assert_eq!(present(row), [Moon, Jupiter, Venus]);
    let row = gaja(&[(Moon, Aries), (Jupiter, Cancer), (Mercury, Cancer)]);
    assert_eq!(present(row), [Moon, Mercury, Jupiter]);
    // Not in a kendra; no benefic on him; a malefic's aspect is no benefic's.
    assert_eq!(
        gaja(&[(Moon, Aries), (Jupiter, Leo), (Venus, Aquarius)]),
        None
    );
    assert_eq!(gaja(&[(Moon, Aries), (Jupiter, Cancer)]), None);
    assert_eq!(
        gaja(&[(Moon, Aries), (Jupiter, Cancer), (Saturn, Capricorn)]),
        None
    );
    // A waxing Moon is a benefic, and aspects him from the 7th.
    let row = gaja(&[(Moon, Capricorn), (Sun, Leo), (Jupiter, Cancer)]);
    assert_eq!(present(row), [Moon, Jupiter]);
    // A tied Mercury beside Venus, who is on Jupiter just the same, so his reading decides nothing here.
    let row = gaja(&[
        (Moon, Aries),
        (Jupiter, Cancer),
        (Mercury, Capricorn),
        (Venus, Capricorn),
        (Mars, Capricorn),
    ]);
    assert_eq!(present(row), [Moon, Mercury, Jupiter, Venus]);
    // Tied by Jupiter himself, he is the only benefic on Jupiter, so his reading decides it, as in chart 13.
    let row = gaja(&[
        (Moon, Aries),
        (Jupiter, Cancer),
        (Mercury, Cancer),
        (Mars, Cancer),
    ])
    .unwrap();
    assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
    assert_eq!(bodies(&row.grahas), [Moon, Mercury, Jupiter]);
}

#[test]
fn gaja_kesari_fails_on_a_debilitated_jupiter_or_in_an_enemys_house() {
    use Body::*;
    use Rasi::*;
    let gaja = |at: &[(Body, Rasi)]| from_aries(at, YogaName::GajaKesari);
    assert_eq!(
        gaja(&[(Moon, Aries), (Jupiter, Capricorn), (Venus, Cancer)]),
        None
    );
    // Venus in the 8th from Taurus is his bad enemy; the 2nd, a temporary friend, makes her only neutral (§3.4.3).
    let enemy = [(Moon, Leo), (Jupiter, Taurus), (Venus, Scorpio)];
    assert_eq!(gaja(&enemy), None);
    let neutral = [
        (Moon, Leo),
        (Jupiter, Taurus),
        (Venus, Gemini),
        (Mercury, Taurus),
    ];
    assert_eq!(present(gaja(&neutral)), [Moon, Mercury, Jupiter]);
}

#[test]
fn gaja_kesaris_combustion_is_to_judge_beside_the_sun_in_the_rasi_chart_only() {
    use Body::*;
    use Rasi::*;
    let at = [
        (Moon, Aries),
        (Jupiter, Cancer),
        (Sun, Cancer),
        (Venus, Capricorn),
    ];
    let row = from_aries(&at, YogaName::GajaKesari).unwrap();
    assert_eq!(row.to_judge, [ToJudge::Combust(Jupiter)]);
    // Everything at 15°: in the navamsa Ar is Le, Cn is Sc, Ta is Ta, so the same shape beside the Sun (ADR 0014).
    let at = [
        (Lagna, Aries),
        (Moon, Aries),
        (Jupiter, Cancer),
        (Sun, Cancer),
        (Venus, Taurus),
    ]
    .map(|(b, r)| (b, mid(r)));
    let row = yoga_in(&scene(Gemini, &at), Varga::Navamsa, YogaName::GajaKesari).unwrap();
    assert_eq!(row.to_judge, []);
}

#[test]
fn guru_mangala_is_jupiter_and_mars_together_or_opposite() {
    use Body::*;
    use Rasi::*;
    let guru = |at: &[(Body, Rasi)]| from_aries(at, YogaName::GuruMangala);
    assert_eq!(
        present(guru(&[(Jupiter, Leo), (Mars, Leo)])),
        [Mars, Jupiter]
    );
    assert_eq!(
        present(guru(&[(Jupiter, Leo), (Mars, Aquarius)])),
        [Mars, Jupiter]
    );
    assert_eq!(guru(&[(Jupiter, Leo), (Mars, Virgo)]), None);
}

#[test]
fn amala_is_only_benefics_in_the_10th_from_the_lagna_or_the_moon() {
    use Body::*;
    use Rasi::*;
    let amala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Amala);
    // The Moon in the pile counts from Gemini, so her 10th is Pisces.
    assert_eq!(present(amala(&[(Venus, Capricorn)])), [Venus]);
    assert_eq!(present(amala(&[(Jupiter, Pisces)])), [Jupiter]);
    assert_eq!(amala(&[(Venus, Capricorn), (Saturn, Capricorn)]), None);
    assert_eq!(amala(&[(Rahu, Capricorn)]), None);
    // Empty, then the wrong house.
    assert_eq!(amala(&[]), None);
    assert_eq!(amala(&[(Venus, Aquarius)]), None);
    // A waxing Moon in the lagna's 10th, then a waning one.
    assert_eq!(present(amala(&[(Moon, Capricorn), (Sun, Leo)])), [Moon]);
    assert_eq!(amala(&[(Moon, Capricorn)]), None);
    // Mercury alone is saumya; beside a malefic, kroora; tied, a malefic is beside him anyway.
    assert_eq!(present(amala(&[(Mercury, Capricorn)])), [Mercury]);
    assert_eq!(amala(&[(Mercury, Capricorn), (Mars, Capricorn)]), None);
    let tied = [(Mercury, Capricorn), (Mars, Capricorn), (Venus, Capricorn)];
    assert_eq!(amala(&tied), None);
}

#[test]
fn parvata_is_only_benefics_in_the_kendras_and_the_8th() {
    use Body::*;
    use Rasi::*;
    let parvata = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Parvata);
    assert_eq!(
        present(parvata(&[(Jupiter, Aries), (Venus, Cancer)])),
        [Jupiter, Venus]
    );
    let row = parvata(&[(Jupiter, Aries), (Mercury, Scorpio)]);
    assert_eq!(present(row), [Mercury, Jupiter]);
    // A malefic in a kendra, a node, a kroora Mercury, or a malefic in the 8th.
    assert_eq!(parvata(&[(Jupiter, Aries), (Mars, Cancer)]), None);
    assert_eq!(parvata(&[(Jupiter, Aries), (Ketu, Libra)]), None);
    assert_eq!(
        parvata(&[(Jupiter, Aries), (Mercury, Libra), (Saturn, Libra)]),
        None
    );
    assert_eq!(parvata(&[(Jupiter, Aries), (Sun, Scorpio)]), None);
    // Vacant kendras all: (2) leaves the 7th free to be vacant, but something must occupy a kendra.
    assert_eq!(parvata(&[]), None);
    assert_eq!(parvata(&[(Jupiter, Scorpio)]), None);
}

/// A row's form, what is left to judge and grahas.
fn formed(row: Option<Yoga>) -> (Option<u8>, Vec<ToJudge>, Vec<Body>) {
    let row = row.expect("the yoga is listed");
    (row.form, row.to_judge, bodies(&row.grahas))
}

#[test]
fn kaahalas_first_form_is_the_4th_lord_and_jupiter_in_mutual_kendras_with_a_strong_lagna_lord() {
    use Body::*;
    use Rasi::*;
    let kaahala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Kaahala);
    // The Moon rules Aries's 4th and Mars its lagna.
    let row = kaahala(&[(Moon, Libra), (Jupiter, Cancer)]);
    assert_eq!(
        formed(row),
        (
            Some(1),
            vec![ToJudge::Strong(Mars)],
            vec![Moon, Mars, Jupiter]
        )
    );
    assert_eq!(kaahala(&[(Moon, Libra), (Jupiter, Leo)]), None);
    // For Virgo he is the 4th lord himself: no pair, so no first form. "Some say 9th lord" is not computed.
    assert_eq!(from_lagna(Virgo, Gemini, &[], YogaName::Kaahala), None);
}

#[test]
fn kaahalas_second_form_is_the_4th_lord_exalted_or_in_own_rasi_joined_by_the_10th_lord() {
    use Body::*;
    use Rasi::*;
    let kaahala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Kaahala);
    for rasi in [Taurus, Cancer] {
        let row = kaahala(&[(Moon, rasi), (Saturn, rasi), (Jupiter, Virgo)]);
        assert_eq!(
            formed(row),
            (Some(2), vec![], vec![Moon, Saturn]),
            "{rasi:?}"
        );
    }
    assert_eq!(
        kaahala(&[(Moon, Taurus), (Saturn, Aries), (Jupiter, Virgo)]),
        None
    );
    assert_eq!(
        kaahala(&[(Moon, Leo), (Saturn, Leo), (Jupiter, Virgo)]),
        None
    );
    // Both forms hold: the present one is the row, not the to-judge one.
    let row = kaahala(&[(Moon, Taurus), (Saturn, Taurus), (Jupiter, Leo)]);
    assert_eq!(formed(row), (Some(2), vec![], vec![Moon, Saturn]));
}

#[test]
fn kaahalas_4th_lord_in_his_own_scorpio_or_aquarius_counts_wherever_his_co_lord_is() {
    use Body::*;
    use Rasi::*;
    // Scorpio's 4th is Aq and 10th Le, with Jupiter out of a kendra from Rahu: only the second form could hold.
    let kaahala = |sun, rahu| {
        let at = [
            (Saturn, Aquarius),
            (Sun, sun),
            (Rahu, rahu),
            (Jupiter, Virgo),
        ];
        from_lagna(Scorpio, Gemini, &at, YogaName::Kaahala)
    };
    for rahu in [Taurus, Aquarius] {
        assert_eq!(
            formed(kaahala(Aquarius, rahu)),
            (Some(2), vec![], vec![Sun, Saturn]),
            "{rahu:?}"
        );
    }
    // Without the 10th lord beside him.
    assert_eq!(kaahala(Gemini, Taurus), None);
}

#[test]
fn chaamaras_lagna_lord_is_exalted_in_a_kendra_with_jupiters_aspect() {
    use Body::*;
    use Rasi::*;
    let chaamara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Chaamara);
    assert_eq!(
        present(chaamara(&[(Mars, Capricorn), (Jupiter, Cancer)])),
        [Mars, Jupiter]
    );
    assert_eq!(chaamara(&[(Mars, Capricorn)]), None);
    // At home, not exalted; then exalted outside a kendra, in Aquarius's 9th.
    assert_eq!(chaamara(&[(Mars, Aries), (Jupiter, Leo)]), None);
    let at = [(Saturn, Libra), (Jupiter, Aries)];
    assert_eq!(from_lagna(Aquarius, Gemini, &at, YogaName::Chaamara), None);
}

#[test]
fn chaamaras_two_benefics_join_in_the_7th_9th_or_10th() {
    use Body::*;
    use Rasi::*;
    let chaamara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Chaamara);
    for rasi in [Libra, Sagittarius, Capricorn] {
        let row = chaamara(&[(Venus, rasi), (Jupiter, rasi)]);
        assert_eq!(present(row), [Jupiter, Venus], "{rasi:?}");
    }
    assert_eq!(chaamara(&[(Venus, Libra), (Jupiter, Sagittarius)]), None);
    assert_eq!(chaamara(&[(Venus, Scorpio), (Jupiter, Scorpio)]), None);
    assert_eq!(chaamara(&[(Venus, Libra)]), None);
    // A tied Mercury is the second benefic only if saumya.
    let row = chaamara(&[(Mercury, Libra), (Venus, Libra), (Mars, Libra)]).unwrap();
    assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
    assert_eq!(bodies(&row.grahas), [Mercury, Venus]);
}

#[test]
fn sankhas_first_form_is_the_5th_and_6th_lords_in_mutual_kendras_with_a_strong_lagna_lord() {
    use Body::*;
    use Rasi::*;
    let sankha = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Sankha);
    for mercury in [Scorpio, Leo] {
        let row = sankha(&[(Sun, Leo), (Mercury, mercury)]);
        assert_eq!(
            formed(row),
            (
                Some(1),
                vec![ToJudge::Strong(Mars)],
                vec![Sun, Mars, Mercury]
            )
        );
    }
    assert_eq!(sankha(&[(Sun, Leo), (Mercury, Virgo)]), None);
}

#[test]
fn sankhas_second_form_is_the_lagna_and_10th_lords_in_a_movable_rasi_with_a_strong_9th_lord() {
    use Body::*;
    use Rasi::*;
    let sankha = |mars, saturn| {
        // The 5th and 6th lords apart, to fail the first form.
        from_aries(
            &[(Sun, Leo), (Mercury, Virgo), (Mars, mars), (Saturn, saturn)],
            YogaName::Sankha,
        )
    };
    assert_eq!(
        formed(sankha(Libra, Libra)),
        (
            Some(2),
            vec![ToJudge::Strong(Jupiter)],
            vec![Mars, Jupiter, Saturn]
        )
    );
    assert_eq!(sankha(Taurus, Taurus), None);
    assert_eq!(sankha(Libra, Cancer), None);
}

#[test]
fn bheris_first_form_is_planets_in_the_1st_2nd_7th_and_12th_with_a_strong_9th_lord() {
    use Body::*;
    use Rasi::*;
    let bheri = |twelfth: (Body, Rasi)| {
        from_aries(
            &[(Sun, Aries), (Moon, Taurus), (Mars, Libra), twelfth],
            YogaName::Bheri,
        )
    };
    assert_eq!(
        formed(bheri((Saturn, Pisces))),
        (
            Some(1),
            vec![ToJudge::Strong(Jupiter)],
            vec![Sun, Moon, Mars, Jupiter, Saturn]
        )
    );
    assert_eq!(bheri((Saturn, Cancer)), None);
    // A node is no planet here (ADR 0014).
    assert_eq!(bheri((Rahu, Pisces)), None);
}

#[test]
fn bheris_second_form_is_jupiter_venus_and_the_lagna_lord_in_mutual_kendras_with_a_strong_9th_lord()
{
    use Body::*;
    use Rasi::*;
    let bheri = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Bheri);
    let row = bheri(&[(Jupiter, Aries), (Venus, Cancer), (Mars, Libra)]);
    assert_eq!(
        formed(row),
        (
            Some(2),
            vec![ToJudge::Strong(Jupiter)],
            vec![Mars, Jupiter, Venus]
        )
    );
    assert_eq!(
        bheri(&[(Jupiter, Aries), (Venus, Leo), (Mars, Libra)]),
        None
    );
    // Venus is both for Taurus: Jupiter and Venus are the only pair. Saturn is its 9th lord.
    let row = from_lagna(
        Taurus,
        Gemini,
        &[(Jupiter, Aries), (Venus, Cancer)],
        YogaName::Bheri,
    );
    assert_eq!(
        formed(row),
        (
            Some(2),
            vec![ToJudge::Strong(Saturn)],
            vec![Jupiter, Venus, Saturn]
        )
    );
}

#[test]
fn mridanga_is_a_planet_in_own_or_uchcha_in_a_kendra_or_trikona_with_a_strong_lagna_lord() {
    use Body::*;
    use Rasi::*;
    let mridanga = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Mridanga);
    for at in [
        (Saturn, Libra),
        (Moon, Cancer),
        (Sun, Leo),
        (Jupiter, Sagittarius),
    ] {
        let row = mridanga(&[at]);
        assert_eq!(
            formed(row),
            (
                None,
                vec![ToJudge::Strong(Mars)],
                in_graha_order(&[at.0, Mars])
            ),
            "{at:?}"
        );
    }
    // Mercury's own rasi in the pile is the 3rd; uchcha in the 6th is outside too; neecha is no dignity.
    assert_eq!(mridanga(&[]), None);
    assert_eq!(mridanga(&[(Mercury, Virgo)]), None);
    assert_eq!(mridanga(&[(Saturn, Aries)]), None);
    // A node is no planet here (ADR 0014): Rahu's own Aquarius in Taurus's 10th.
    assert_eq!(
        from_lagna(
            Taurus,
            Pisces,
            &[(Rahu, Aquarius), (Mars, Aries)],
            YogaName::Mridanga
        ),
        None
    );
}

#[test]
fn sreenaatha_is_the_7th_lord_exalted_in_the_10th_and_the_10th_lord_with_the_9th() {
    use Body::*;
    use Rasi::*;
    let row = from_lagna(
        Sagittarius,
        Gemini,
        &[(Mercury, Virgo), (Sun, Virgo)],
        YogaName::Sreenaatha,
    );
    assert_eq!(present(row), [Sun, Mercury]);
    let row = from_lagna(
        Sagittarius,
        Gemini,
        &[(Mercury, Virgo), (Sun, Libra)],
        YogaName::Sreenaatha,
    );
    assert_eq!(row, None);
    // Footnote 33: only Sagittarius can have it, co-lords and all.
    for lagna in RASIS {
        let row = from_lagna(lagna, kth(lagna, 10), &[], YogaName::Sreenaatha);
        assert_eq!(row.is_some(), lagna == Sagittarius, "{lagna:?}");
    }
}

#[test]
fn matsya_is_benefics_in_the_lagna_and_9th_a_planet_in_the_5th_and_malefics_in_the_4th_and_8th() {
    use Body::*;
    use Rasi::*;
    let whole = [
        (Jupiter, Aries),
        (Venus, Sagittarius),
        (Sun, Leo),
        (Mars, Cancer),
        (Saturn, Scorpio),
    ];
    let matsya = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Matsya);
    assert_eq!(present(matsya(&whole)), [Sun, Mars, Jupiter, Venus, Saturn]);
    // Each named house emptied in turn, all to the pile.
    for missing in 0..whole.len() {
        let mut at = whole.to_vec();
        at[missing].1 = Gemini;
        assert_eq!(matsya(&at), None, "{:?}", whole[missing]);
    }
    // A node is a malefic in the 4th (ADR 0014) but no planet in the 5th.
    let mut node = whole.to_vec();
    node[3] = (Rahu, Cancer);
    assert!(matsya(&node).is_some());
    let mut node = whole.to_vec();
    node[2] = (Rahu, Leo);
    assert_eq!(matsya(&node), None);
}

#[test]
fn koorma_is_dignified_benefics_in_the_5th_to_7th_and_malefics_in_own_or_uchcha_in_the_1st_3rd_and_11th()
 {
    use Body::*;
    use Rasi::*;
    let koorma = |at: &[(Body, Rasi)]| from_lagna(Aries, Taurus, at, YogaName::Koorma);
    let whole = [
        // Jupiter's friend the Sun is in the pile, in Leo's 10th, so his close friend. Mercury exalted, Venus at home.
        (Jupiter, Leo),
        (Mercury, Virgo),
        (Venus, Libra),
        // Mars at home, Rahu exalted, Saturn at home.
        (Mars, Aries),
        (Rahu, Gemini),
        (Saturn, Aquarius),
    ];
    assert_eq!(
        present(koorma(&whole)),
        [Mars, Mercury, Jupiter, Venus, Saturn, Rahu]
    );
    // Each house emptied in turn.
    for missing in 0..whole.len() {
        let mut at = whole.to_vec();
        at[missing].1 = Taurus;
        assert_eq!(koorma(&at), None, "{:?}", whole[missing]);
    }
    // The Sun in Leo's 9th only neutral to Jupiter (§3.4.3), so not friendly, though himself exalted in the 1st.
    let mut neutral = whole.to_vec();
    neutral.push((Sun, Aries));
    assert_eq!(koorma(&neutral), None);
    // A malefic neither at home nor exalted: the Sun alone in the 3rd, Jupiter's friend still.
    let mut undignified = whole.to_vec();
    undignified[4] = (Rahu, Taurus);
    undignified.push((Sun, Gemini));
    assert_eq!(koorma(&undignified), None);
}

#[test]
fn khadga_is_the_2nd_and_9th_lords_exchanged_with_the_lagna_lord_in_a_kendra_or_trikona() {
    use Body::*;
    use Rasi::*;
    let khadga = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Khadga);
    for mars in [Cancer, Leo] {
        let row = khadga(&[(Venus, Sagittarius), (Jupiter, Taurus), (Mars, mars)]);
        assert_eq!(present(row), [Mars, Jupiter, Venus], "{mars:?}");
    }
    assert_eq!(khadga(&[(Venus, Sagittarius), (Jupiter, Taurus)]), None);
    assert_eq!(
        khadga(&[(Venus, Libra), (Jupiter, Taurus), (Mars, Cancer)]),
        None
    );
    assert_eq!(
        khadga(&[(Venus, Sagittarius), (Jupiter, Leo), (Mars, Cancer)]),
        None
    );
}

#[test]
fn kusuma_is_a_fixed_lagna_venus_in_a_kendra_the_moon_in_a_trikona_with_a_benefic_and_saturn_in_the_10th()
 {
    use Body::*;
    use Rasi::*;
    let whole = [
        (Venus, Leo),
        (Moon, Virgo),
        (Jupiter, Virgo),
        (Saturn, Aquarius),
    ];
    let kusuma = |at: &[(Body, Rasi)]| from_lagna(Taurus, Gemini, at, YogaName::Kusuma);
    assert_eq!(present(kusuma(&whole)), [Moon, Jupiter, Venus, Saturn]);
    for missing in 0..whole.len() {
        let mut at = whole.to_vec();
        at[missing].1 = Gemini;
        assert_eq!(kusuma(&at), None, "{:?}", whole[missing]);
    }
    // A malefic is no company for her; the same shape from a movable lagna is not it.
    let malefic = [
        (Venus, Leo),
        (Moon, Virgo),
        (Mars, Virgo),
        (Saturn, Aquarius),
    ];
    assert_eq!(kusuma(&malefic), None);
    let movable = [
        (Venus, Cancer),
        (Moon, Leo),
        (Jupiter, Leo),
        (Saturn, Capricorn),
    ];
    assert_eq!(from_aries(&movable, YogaName::Kusuma), None);
}

#[test]
fn kalaanidhi_is_jupiter_in_the_2nd_or_5th_with_both_mercury_and_venus_on_him() {
    use Body::*;
    use Rasi::*;
    let kalaanidhi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Kalaanidhi);
    let all = [Mercury, Jupiter, Venus];
    assert_eq!(
        present(kalaanidhi(&[
            (Jupiter, Taurus),
            (Mercury, Taurus),
            (Venus, Scorpio)
        ])),
        all
    );
    assert_eq!(
        present(kalaanidhi(&[
            (Jupiter, Leo),
            (Mercury, Aquarius),
            (Venus, Leo)
        ])),
        all
    );
    assert_eq!(kalaanidhi(&[(Jupiter, Taurus), (Mercury, Taurus)]), None);
    assert_eq!(kalaanidhi(&[(Jupiter, Taurus), (Venus, Scorpio)]), None);
    assert_eq!(
        kalaanidhi(&[(Jupiter, Cancer), (Mercury, Cancer), (Venus, Cancer)]),
        None
    );
}

/// Kalpadruma's row in the given chart with the lagna in Aries and the rest in Gemini.
fn kalpadruma_in(d: Varga, at: &[(Body, f64)]) -> Option<Vec<Body>> {
    let mut at = at.to_vec();
    at.push((Body::Lagna, mid(Rasi::Aries)));
    yoga_in(&scene(Rasi::Gemini, &at), d, YogaName::Kalpadruma).map(|y| bodies(&y.grahas))
}

#[test]
fn kalpadrumas_fourth_graha_is_the_dispositors_dispositor_in_the_navamsa() {
    use Body::*;
    use Rasi::*;
    // Mars in Cn, so the Moon; she in Li, so Venus in the rasi chart. Her navamsa picks the fourth.
    let with_moon_at = |moon: f64, more: &[(Body, f64)]| {
        let mut at = vec![(Mars, mid(Cancer)), (Moon, moon), (Venus, mid(Leo))];
        at.extend_from_slice(more);
        kalpadruma_in(Varga::RasiChart, &at)
    };
    // At 15° Li her navamsa is Aq, so Saturn, who must then stand in a kendra of the rasi chart.
    let saturn_placed = [(Saturn, mid(Libra))];
    assert_eq!(
        with_moon_at(mid(Libra), &saturn_placed),
        Some(vec![Mars, Moon, Venus, Saturn])
    );
    // Saturn in the 3rd, and Rahu, the co-lord, out of his uchcha Gemini.
    assert_eq!(with_moon_at(mid(Libra), &[(Rahu, mid(Taurus))]), None);
    // At 1° Li her navamsa is Li, so Venus again, with Saturn left in the 3rd.
    assert_eq!(
        with_moon_at(dms(6, 1, 0, 0.0), &[]),
        Some(vec![Mars, Moon, Venus, Venus])
    );
}

#[test]
fn kalpadruma_needs_all_four_placed_and_a_graha_at_home_disposes_of_himself() {
    use Body::*;
    use Rasi::*;
    let kalpadruma = |at: &[(Body, f64)]| kalpadruma_in(Varga::RasiChart, at);
    // Venus in the 3rd.
    let at = [(Mars, mid(Cancer)), (Moon, dms(6, 1, 0, 0.0))];
    assert_eq!(kalpadruma(&at), None);
    // Mars in Gemini, the 3rd, is not placed, whatever follows.
    let at = [(Moon, mid(Libra)), (Venus, mid(Leo)), (Saturn, mid(Libra))];
    assert_eq!(kalpadruma(&at), None);
    // Mars at home three times over; at 15° Ar his navamsa is Le, so the Sun.
    let at = [(Mars, mid(Aries)), (Sun, mid(Leo))];
    assert_eq!(kalpadruma(&at), Some(vec![Mars, Mars, Mars, Sun]));
    // Mars at home in the lagna is tried once Ketu, the main owner, fails in the 8th; at 1° Sc his navamsa is Cn, so the Moon.
    let at = [
        (Lagna, mid(Scorpio)),
        (Mars, dms(7, 1, 0, 0.0)),
        (Moon, mid(Scorpio)),
        (Ketu, mid(Gemini)),
    ];
    let row = yoga_in(&scene(Gemini, &at), Varga::RasiChart, YogaName::Kalpadruma);
    assert_eq!(
        row.map(|y| bodies(&y.grahas)),
        Some(vec![Mars, Mars, Mars, Moon])
    );
    // At 15° Sc his navamsa is Sc, so he would be all four, and one graha forms no yoga (ADR 0014).
    let at = [
        (Lagna, mid(Scorpio)),
        (Mars, mid(Scorpio)),
        (Ketu, mid(Gemini)),
    ];
    let row = yoga_in(&scene(Gemini, &at), Varga::RasiChart, YogaName::Kalpadruma);
    assert_eq!(row, None);
}

#[test]
fn kalpadruma_counts_an_uchcha_rasi_outside_the_kendras_and_trikonas() {
    use Body::*;
    use Rasi::*;
    // Leo's lagna lord in Aq, Saturn exalted in its 3rd, and Venus at 1° Li's navamsa: chart 9's shape, with her in Ar.
    let at = [
        (Lagna, mid(Leo)),
        (Sun, mid(Aquarius)),
        (Saturn, dms(6, 1, 0, 0.0)),
        (Venus, mid(Aries)),
    ];
    let row = yoga_in(&scene(Gemini, &at), Varga::RasiChart, YogaName::Kalpadruma);
    assert_eq!(
        row.map(|y| bodies(&y.grahas)),
        Some(vec![Sun, Saturn, Venus, Venus])
    );
}

#[test]
fn kalpadruma_is_only_in_the_rasi_chart_list() {
    use Body::*;
    use Rasi::*;
    let at = [(Mars, mid(Aries)), (Sun, mid(Leo))];
    assert!(kalpadruma_in(Varga::RasiChart, &at).is_some());
    for d in VARGAS.into_iter().filter(|&d| d != Varga::RasiChart) {
        assert_eq!(kalpadruma_in(d, &at), None, "{d:?}");
    }
}

#[test]
fn lagnaadhi_is_two_or_more_benefics_in_the_7th_and_8th_in_any_arrangement() {
    use Body::*;
    use Rasi::*;
    let lagnaadhi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Lagnaadhi);
    assert_eq!(
        present(lagnaadhi(&[(Jupiter, Libra), (Venus, Libra)])),
        [Jupiter, Venus]
    );
    assert_eq!(
        present(lagnaadhi(&[(Jupiter, Scorpio), (Venus, Scorpio)])),
        [Jupiter, Venus]
    );
    assert_eq!(
        present(lagnaadhi(&[(Jupiter, Libra), (Venus, Scorpio)])),
        [Jupiter, Venus]
    );
    // One alone, and the 6th not added (ADR 0014).
    assert_eq!(lagnaadhi(&[(Jupiter, Libra)]), None);
    assert_eq!(lagnaadhi(&[(Jupiter, Virgo), (Venus, Libra)]), None);
    assert_eq!(lagnaadhi(&[(Jupiter, Virgo), (Venus, Virgo)]), None);
}

#[test]
fn a_malefic_conjoining_either_benefic_or_on_it_by_graha_drishti_stops_lagnaadhi() {
    use Body::*;
    use Rasi::*;
    let lagnaadhi = |more: &[(Body, Rasi)]| {
        let mut at = vec![(Jupiter, Libra), (Venus, Scorpio)];
        at.extend_from_slice(more);
        from_aries(&at, YogaName::Lagnaadhi)
    };
    // Conjoining each, a node among them, which has no drishti but is a malefic.
    assert_eq!(lagnaadhi(&[(Mars, Libra)]), None);
    assert_eq!(lagnaadhi(&[(Saturn, Scorpio)]), None);
    assert_eq!(lagnaadhi(&[(Rahu, Libra)]), None);
    // Graha drishti: Saturn's 7th on Li from Ar, his 3rd on Li from Le, and the Sun's 7th on Sc from Ta.
    assert_eq!(lagnaadhi(&[(Saturn, Aries)]), None);
    assert_eq!(lagnaadhi(&[(Saturn, Leo)]), None);
    assert_eq!(lagnaadhi(&[(Sun, Taurus)]), None);
    // A node's rasi drishti is not graha drishti.
    assert!(lagnaadhi(&[(Rahu, Aries)]).is_some());
}

#[test]
fn lagnaadhi_reads_the_moon_by_paksha_and_mercury_by_his_company() {
    use Body::*;
    use Rasi::*;
    let lagnaadhi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Lagnaadhi);
    // A waxing Moon is the second benefic, or a third.
    assert_eq!(
        present(lagnaadhi(&[(Moon, Scorpio), (Jupiter, Libra)])),
        [Moon, Jupiter]
    );
    let row = lagnaadhi(&[(Moon, Scorpio), (Jupiter, Libra), (Venus, Libra)]);
    assert_eq!(present(row), [Moon, Jupiter, Venus]);
    // A waning one is no benefic, and a malefic beside him or on him by graha drishti.
    assert_eq!(
        lagnaadhi(&[(Moon, Scorpio), (Sun, Capricorn), (Jupiter, Libra)]),
        None
    );
    let beside = [
        (Moon, Scorpio),
        (Sun, Capricorn),
        (Jupiter, Libra),
        (Venus, Scorpio),
    ];
    assert_eq!(lagnaadhi(&beside), None);
    let aspecting = [(Moon, Aries), (Jupiter, Libra), (Venus, Libra)];
    assert_eq!(lagnaadhi(&aspecting), None);
    // A saumya Mercury is a benefic.
    assert_eq!(
        present(lagnaadhi(&[(Mercury, Libra), (Jupiter, Scorpio)])),
        [Mercury, Jupiter]
    );
    // A kroora Mercury's graha drishti, with Rahu beside him, stops it: Rahu has none, so it is his alone.
    let aspecting = [
        (Mercury, Aries),
        (Rahu, Aries),
        (Jupiter, Libra),
        (Venus, Libra),
    ];
    assert_eq!(lagnaadhi(&aspecting), None);
    // Alone there, he is saumya and his graha drishti is no harm.
    let harmless = [(Mercury, Aries), (Jupiter, Libra), (Venus, Libra)];
    assert_eq!(present(lagnaadhi(&harmless)), [Jupiter, Venus]);
}

#[test]
fn a_tied_mercurys_reading_of_lagnaadhi_is_to_judge_only_if_it_changes_the_result() {
    use Body::*;
    use Rasi::*;
    let lagnaadhi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Lagnaadhi);
    // In Ar with Venus and Rahu, he is tied: saumya, his graha drishti is harmless; kroora, it stops the Moon and Jupiter in Li.
    let at = [
        (Mercury, Aries),
        (Venus, Aries),
        (Rahu, Aries),
        (Moon, Libra),
        (Jupiter, Libra),
    ];
    let row = lagnaadhi(&at).unwrap();
    assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
    assert_eq!(bodies(&row.grahas), [Moon, Jupiter]);
    // Tied in Pi, his graha drishti falls on Vi, so it stands either way.
    let at = [
        (Mercury, Pisces),
        (Venus, Pisces),
        (Rahu, Pisces),
        (Moon, Libra),
        (Jupiter, Libra),
    ];
    assert_eq!(present(lagnaadhi(&at)), [Moon, Jupiter]);
}

#[test]
fn hari_is_every_benefic_in_the_2nd_12th_and_8th_from_the_2nd_lord() {
    use Body::*;
    use Rasi::*;
    let hari = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Hari);
    // Venus rules Aries's 2nd; from Li they are Sc, Vi and Ta. The Moon waxes in Sc, Mercury in the pile is kroora.
    let row = hari(&[(Venus, Libra), (Jupiter, Virgo), (Moon, Scorpio)]);
    assert_eq!(present(row), [Moon, Jupiter, Venus]);
    // Every benefic, not any: Jupiter outside; a waxing Moon outside.
    assert_eq!(hari(&[(Venus, Libra), (Moon, Scorpio)]), None);
    assert_eq!(hari(&[(Venus, Libra), (Jupiter, Virgo)]), None);
    // A waning Moon need not be there.
    let row = hari(&[(Venus, Libra), (Jupiter, Virgo), (Sun, Leo)]);
    assert_eq!(present(row), [Jupiter, Venus]);
    // A saumya Mercury must be, and a kroora one need not.
    let saumya = [
        (Mercury, Cancer),
        (Venus, Libra),
        (Jupiter, Virgo),
        (Moon, Scorpio),
    ];
    assert_eq!(hari(&saumya), None);
    let saumya = [
        (Mercury, Taurus),
        (Venus, Libra),
        (Jupiter, Virgo),
        (Moon, Scorpio),
    ];
    assert_eq!(present(hari(&saumya)), [Moon, Mercury, Jupiter, Venus]);
}

#[test]
fn a_tied_mercury_outside_haris_places_is_to_judge() {
    use Body::*;
    use Rasi::*;
    // Beside Venus and Mars in Li he is tied, and Li is none of its own places.
    let at = [
        (Mercury, Libra),
        (Venus, Libra),
        (Mars, Libra),
        (Jupiter, Virgo),
        (Moon, Scorpio),
    ];
    let row = from_aries(&at, YogaName::Hari).unwrap();
    assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
    assert_eq!(bodies(&row.grahas), [Moon, Jupiter, Venus]);
}

#[test]
fn hari_works_from_either_co_lord_and_names_him() {
    use Body::*;
    use Rasi::*;
    // Scorpio is Libra's 2nd. From Ketu in Ar the places are Ta, Pi and Sc; from Mars in the Cp pile, Aq, Sg and Le.
    let at = [
        (Ketu, Aries),
        (Jupiter, Taurus),
        (Venus, Pisces),
        (Moon, Virgo),
    ];
    let row = from_lagna(Libra, Capricorn, &at, YogaName::Hari);
    assert_eq!(present(row), [Jupiter, Venus, Ketu]);
    // And from Mars, the same benefics placed from him.
    let at = [
        (Mars, Aries),
        (Ketu, Capricorn),
        (Jupiter, Taurus),
        (Venus, Pisces),
        (Moon, Virgo),
    ];
    let row = from_lagna(Libra, Capricorn, &at, YogaName::Hari);
    assert_eq!(present(row), [Mars, Jupiter, Venus]);
}

#[test]
fn hara_is_every_benefic_in_the_4th_9th_and_8th_from_the_7th_lord() {
    use Body::*;
    use Rasi::*;
    let hara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Hara);
    // Venus rules Aries's 7th; from Li they are Cp, Ge and Ta, with the waxing Moon and Jupiter in the Ge pile.
    assert_eq!(present(hara(&[(Venus, Libra)])), [Moon, Jupiter, Venus]);
    assert_eq!(hara(&[(Venus, Libra), (Jupiter, Leo)]), None);
}

#[test]
fn brahmas_first_form_is_every_benefic_in_the_4th_10th_and_11th_from_the_lagna_lord() {
    use Body::*;
    use Rasi::*;
    let brahma = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Brahma);
    // Mars in Ar: Cn, Cp and Aq. The Moon wanes in Aq. Jupiter is Aries's 9th lord, so has no second form.
    let at = [
        (Mars, Aries),
        (Jupiter, Cancer),
        (Venus, Capricorn),
        (Moon, Aquarius),
    ];
    assert_eq!(
        formed(brahma(&at)),
        (Some(1), vec![], vec![Mars, Jupiter, Venus])
    );
    let at = [
        (Mars, Aries),
        (Jupiter, Leo),
        (Venus, Capricorn),
        (Moon, Aquarius),
    ];
    assert_eq!(brahma(&at), None);
}

#[test]
fn brahmas_second_form_is_jupiter_venus_and_mercury_each_in_a_kendra_from_its_lord() {
    use Body::*;
    use Rasi::*;
    // Taurus: the 9th lord is Saturn, the 11th Jupiter, the 1st Venus, the 10th Saturn or Rahu.
    let brahma = |at: &[(Body, Rasi)]| from_lagna(Taurus, Gemini, at, YogaName::Brahma);
    let row = brahma(&[
        (Saturn, Aries),
        (Jupiter, Cancer),
        (Venus, Libra),
        (Mercury, Libra),
    ]);
    assert_eq!(
        formed(row),
        (Some(2), vec![], vec![Mercury, Jupiter, Venus, Saturn])
    );
    // Mercury from the 10th lord alone: Rahu's kendra, 4th from Le, is none of Venus's or Saturn's.
    let row = brahma(&[
        (Saturn, Aries),
        (Rahu, Leo),
        (Jupiter, Cancer),
        (Venus, Libra),
        (Mercury, Scorpio),
    ]);
    assert_eq!(
        formed(row),
        (Some(2), vec![], vec![Mercury, Jupiter, Venus, Saturn, Rahu])
    );
    // Each graha out of its kendra in turn.
    let whole = [
        (Saturn, Aries),
        (Jupiter, Cancer),
        (Venus, Libra),
        (Mercury, Libra),
    ];
    assert_eq!(
        brahma(&[whole[0], (Jupiter, Leo), whole[2], whole[3]]),
        None
    );
    assert_eq!(
        brahma(&[whole[0], whole[1], (Venus, Scorpio), (Mercury, Scorpio)]),
        None
    );
    assert_eq!(
        brahma(&[whole[0], whole[1], whole[2], (Mercury, Scorpio)]),
        None
    );
}

#[test]
fn harsha_sarala_and_vimala_are_the_dusthana_lord_in_his_own_dusthana() {
    use Body::*;
    use Rasi::*;
    // From Aries: Mercury rules the 6th, Mars or Ketu the 8th, Jupiter the 12th.
    assert_eq!(
        present(from_aries(&[(Mercury, Virgo)], YogaName::Harsha)),
        [Mercury]
    );
    assert_eq!(from_aries(&[], YogaName::Harsha), None);
    assert_eq!(
        present(from_aries(&[(Jupiter, Pisces)], YogaName::Vimala)),
        [Jupiter]
    );
    assert_eq!(from_aries(&[], YogaName::Vimala), None);
    assert_eq!(
        present(from_aries(&[(Mars, Scorpio)], YogaName::Sarala)),
        [Mars]
    );
    assert_eq!(from_aries(&[], YogaName::Sarala), None);
    // A node at home is not tried (ADR 0014).
    assert_eq!(from_aries(&[(Ketu, Scorpio)], YogaName::Sarala), None);
    // Mars at home with Ketu outside is the 8th lord §11.6 tried, so the row names him so, beside the 1st.
    let sarala = from_aries(&[(Mars, Scorpio)], YogaName::Sarala).unwrap();
    assert_eq!(sarala.grahas[0].named_lord_of, [1, 8]);
}

#[test]
fn harsha_in_a_varga_reads_that_vargas_lagna_and_places() {
    use Body::*;
    use Rasi::*;
    // 1° Ar is D-9's Aries and 8° Cn its Virgo.
    let at = [(Lagna, dms(0, 1, 0, 0.0)), (Mercury, dms(3, 8, 0, 0.0))];
    let chart = scene(Gemini, &at);
    assert_eq!(yoga_in(&chart, Varga::RasiChart, YogaName::Harsha), None);
    let row = yoga_in(&chart, Varga::Navamsa, YogaName::Harsha);
    assert_eq!(present(row), [Mercury]);
}

#[test]
fn siva_is_the_5th_lord_in_the_9th_the_9th_in_the_10th_and_the_10th_in_the_5th() {
    use Body::*;
    use Rasi::*;
    let siva = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Siva);
    let at = [(Sun, Sagittarius), (Jupiter, Capricorn), (Saturn, Leo)];
    assert_eq!(present(siva(&at)), [Sun, Jupiter, Saturn]);
    assert_eq!(siva(&[at[0], at[1]]), None);
    assert_eq!(siva(&[at[0], at[2]]), None);
    assert_eq!(siva(&[at[1], at[2]]), None);
}

#[test]
fn trilochana_is_the_sun_moon_and_mars_in_mutual_trines_the_same_rasi_counting() {
    use Body::*;
    use Rasi::*;
    let trilochana = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Trilochana);
    let at = [(Sun, Aries), (Moon, Leo), (Mars, Sagittarius)];
    assert_eq!(present(trilochana(&at)), [Sun, Moon, Mars]);
    // All three in the pile (ADR 0014).
    assert_eq!(present(trilochana(&[])), [Sun, Moon, Mars]);
    assert_eq!(
        trilochana(&[(Sun, Aries), (Moon, Leo), (Mars, Virgo)]),
        None
    );
    assert_eq!(
        trilochana(&[(Sun, Aries), (Moon, Taurus), (Mars, Sagittarius)]),
        None
    );
}

#[test]
fn lakshmi_is_the_9th_lord_own_or_uchcha_in_a_kendra_with_a_strong_lagna_lord() {
    use Body::*;
    use Rasi::*;
    // Uchcha in the 4th from Aries.
    let row = from_aries(&[(Jupiter, Cancer)], YogaName::Lakshmi);
    assert_eq!(
        formed(row),
        (None, vec![ToJudge::Strong(Mars)], vec![Mars, Jupiter])
    );
    // Own in the 4th from Leo, whose 9th is Aries.
    let row = from_lagna(Leo, Gemini, &[(Mars, Scorpio)], YogaName::Lakshmi);
    assert_eq!(
        formed(row),
        (None, vec![ToJudge::Strong(Sun)], vec![Sun, Mars])
    );
    // Own in the 9th and the 12th, in no kendra; neither own nor uchcha in the 1st.
    for jupiter in [Sagittarius, Pisces, Aries] {
        assert_eq!(
            from_aries(&[(Jupiter, jupiter)], YogaName::Lakshmi),
            None,
            "{jupiter:?}"
        );
    }
    // Uchcha in Pi, the 2nd from Aquarius, whose 9th is Libra.
    let row = from_lagna(Aquarius, Gemini, &[(Venus, Pisces)], YogaName::Lakshmi);
    assert_eq!(row, None);
}

#[test]
fn saarada_needs_all_five_of_its_conditions_and_leaves_the_suns_strength_to_judge() {
    use Body::*;
    use Rasi::*;
    let saarada = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Saarada);
    // Mercury in the 9th from the Moon in Sc, and Jupiter in the pile, her 8th, is left out.
    let whole = [
        (Saturn, Leo),
        (Mercury, Cancer),
        (Sun, Leo),
        (Mars, Aquarius),
        (Moon, Scorpio),
    ];
    assert_eq!(
        formed(saarada(&whole)),
        (
            None,
            vec![ToJudge::Strong(Sun)],
            vec![Sun, Moon, Mars, Mercury, Saturn]
        )
    );
    // Jupiter in the 9th from the Moon in Li, and Mercury in her 10th.
    let row = saarada(&[whole[0], whole[1], whole[2], whole[3], (Moon, Libra)]);
    assert_eq!(formed(row).2, [Sun, Moon, Mars, Mercury, Jupiter, Saturn]);
    // Mercury out of the kendras, with Jupiter in the 9th from the Moon in Li all the same.
    let at = [
        whole[0],
        (Mercury, Virgo),
        whole[2],
        whole[3],
        (Moon, Libra),
    ];
    assert_eq!(saarada(&at), None);
    // Neither is in a trine from the Moon in Vi.
    let row = saarada(&[whole[0], whole[1], whole[2], whole[3], (Moon, Virgo)]);
    assert_eq!(row, None);
    // Each other condition out in turn.
    for (i, moved) in [
        (0, (Saturn, Virgo)),
        (1, (Mercury, Virgo)),
        (2, (Sun, Virgo)),
        (3, (Mars, Virgo)),
    ] {
        let mut at = whole;
        at[i] = moved;
        assert_eq!(saarada(&at), None, "{moved:?}");
    }
}

#[test]
fn saraswathi_is_all_three_in_kendras_trikonas_or_the_2nd_with_jupiter_well_placed() {
    use Body::*;
    use Rasi::*;
    let saraswathi = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Saraswathi);
    // Jupiter's own rasi in the 9th.
    let at = [(Mercury, Cancer), (Jupiter, Sagittarius), (Venus, Taurus)];
    assert_eq!(present(saraswathi(&at)), [Mercury, Jupiter, Venus]);
    // Uchcha in the 4th.
    let row = saraswathi(&[(Mercury, Cancer), (Jupiter, Cancer), (Venus, Taurus)]);
    assert_eq!(present(row), [Mercury, Jupiter, Venus]);
    // Adhimitra: Mars is his friend, and in the pile in the 3rd from Ar a friend for now.
    let row = saraswathi(&[(Mercury, Cancer), (Jupiter, Aries), (Venus, Taurus)]);
    assert_eq!(present(row), [Mercury, Jupiter, Venus]);
    // Mitra: Saturn is neutral by nature, and in Aq, the 2nd from Cp, a friend for now.
    let row = saraswathi(&[
        (Mercury, Cancer),
        (Jupiter, Capricorn),
        (Venus, Taurus),
        (Saturn, Aquarius),
    ]);
    assert_eq!(present(row), [Mercury, Jupiter, Venus]);
    // Every house of the rule.
    for mercury in [Aries, Taurus, Cancer, Leo, Libra, Sagittarius, Capricorn] {
        let row = saraswathi(&[(Mercury, mercury), at[1], at[2]]);
        assert_eq!(present(row), [Mercury, Jupiter, Venus], "{mercury:?}");
    }
    // Adhisatru: beside Venus, an enemy by nature, in Ta.
    let row = saraswathi(&[(Mercury, Cancer), (Jupiter, Taurus), (Venus, Taurus)]);
    assert_eq!(row, None);
    // Each of the three in the 3rd, the pile.
    assert_eq!(saraswathi(&[at[1], at[2]]), None);
    assert_eq!(saraswathi(&[at[0], at[1]]), None);
    assert_eq!(saraswathi(&[at[0], at[2]]), None);
}

#[test]
fn amsaavatara_is_jupiter_venus_and_an_exalted_saturn_in_kendras() {
    use Body::*;
    use Rasi::*;
    let amsaavatara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Amsaavatara);
    let at = [(Jupiter, Cancer), (Venus, Aries), (Saturn, Libra)];
    assert_eq!(present(amsaavatara(&at)), [Jupiter, Venus, Saturn]);
    // In a kendra but not exalted.
    assert_eq!(amsaavatara(&[at[0], at[1], (Saturn, Capricorn)]), None);
    // Jupiter and Venus out of the kendras in turn.
    assert_eq!(amsaavatara(&[(Jupiter, Leo), at[1], at[2]]), None);
    assert_eq!(amsaavatara(&[at[0], (Venus, Leo), at[2]]), None);
    // Exalted, but in the 6th from Taurus.
    let at = [(Jupiter, Leo), (Venus, Taurus), (Saturn, Libra)];
    let row = from_lagna(Taurus, Gemini, &at, YogaName::Amsaavatara);
    assert_eq!(row, None);
}

#[test]
fn devendra_is_a_fixed_lagna_with_two_parivartanas() {
    use Body::*;
    use Rasi::*;
    // Taurus: the 2nd and 10th lords, Mercury and Saturn; the lagna and 11th lords, Venus and Jupiter.
    let devendra = |at: &[(Body, Rasi)]| from_lagna(Taurus, Cancer, at, YogaName::Devendra);
    let whole = [
        (Mercury, Aquarius),
        (Saturn, Gemini),
        (Venus, Pisces),
        (Jupiter, Taurus),
    ];
    assert_eq!(present(devendra(&whole)), [Mercury, Jupiter, Venus, Saturn]);
    // Half of either exchange.
    assert_eq!(devendra(&whole[1..]), None);
    assert_eq!(devendra(&whole[..3]), None);
    // Both exchanges from Cancer, a movable lagna.
    let at = [(Sun, Aries), (Mars, Leo), (Moon, Taurus), (Venus, Cancer)];
    assert_eq!(from_lagna(Cancer, Gemini, &at, YogaName::Devendra), None);
}

#[test]
fn indra_is_the_5th_and_11th_lords_exchanged_with_the_moon_in_the_5th() {
    use Body::*;
    use Rasi::*;
    let indra = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Indra);
    let at = [(Sun, Aquarius), (Saturn, Leo), (Moon, Leo)];
    assert_eq!(present(indra(&at)), [Sun, Moon, Saturn]);
    assert_eq!(indra(&[at[0], at[1]]), None);
    assert_eq!(indra(&[at[1], at[2]]), None);
    assert_eq!(indra(&[at[0], at[2]]), None);
}

#[test]
fn the_single_ravi_yoga_is_the_sun_in_the_10th_and_its_lord_in_the_3rd_with_saturn() {
    use Body::*;
    use Rasi::*;
    let ravi = |at: &[(Body, Rasi)]| from_lagna(Gemini, Cancer, at, YogaName::Ravi);
    let at = [(Sun, Pisces), (Jupiter, Leo), (Saturn, Leo)];
    assert_eq!(present(ravi(&at)), [Sun, Jupiter, Saturn]);
    assert_eq!(ravi(&[at[0], at[1]]), None);
    assert_eq!(ravi(&[at[1], at[2]]), None);
    assert_eq!(ravi(&[at[0], at[2]]), None);
    // With Saturn, but in the 4th.
    let at = [(Sun, Pisces), (Jupiter, Virgo), (Saturn, Virgo)];
    assert_eq!(ravi(&at), None);
    // Taurus: Rahu, the 10th's other lord, stands with Saturn, who cannot stand with himself.
    let at = [(Sun, Aquarius), (Rahu, Cancer), (Saturn, Cancer)];
    let row = from_lagna(Taurus, Gemini, &at, YogaName::Ravi);
    assert_eq!(present(row), [Sun, Saturn, Rahu]);
    let at = [(Sun, Aquarius), (Saturn, Cancer)];
    assert_eq!(from_lagna(Taurus, Gemini, &at, YogaName::Ravi), None);
}

#[test]
fn the_single_ravi_yoga_is_listed_apart_from_the_class_of_ravi_yogas() {
    use Body::*;
    use Rasi::*;
    // Venus in the Sun's 2nd adds a Vesi, of the class, to the single yoga.
    let at = [
        (Sun, Pisces),
        (Jupiter, Leo),
        (Saturn, Leo),
        (Venus, Aries),
        (Lagna, Gemini),
    ];
    let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
    let rows = scene(Cancer, &at).in_varga(Varga::RasiChart).yogas();
    let class_of = |name| rows.iter().find(|y| y.name == name).unwrap().name.class();
    assert_eq!(class_of(YogaName::Vesi), YogaClass::Ravi);
    assert_eq!(class_of(YogaName::Ravi), YogaClass::OtherPopular);
}

#[test]
fn bhaaskara_counts_the_moon_and_mercury_from_the_sun_and_jupiter_from_the_moon() {
    use Body::*;
    use Rasi::*;
    let bhaaskara = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Bhaaskara);
    // The Moon stays in the pile in Ge, the Sun's 12th.
    for jupiter in [Libra, Aquarius] {
        let at = [(Sun, Cancer), (Mercury, Leo), (Jupiter, jupiter)];
        assert_eq!(present(bhaaskara(&at)), [Sun, Moon, Mercury, Jupiter]);
    }
    let at = [(Sun, Cancer), (Mercury, Leo), (Jupiter, Scorpio)];
    assert_eq!(bhaaskara(&at), None);
    // Mercury in the Sun's 12th, and then the Moon in his 2nd.
    let at = [(Sun, Cancer), (Jupiter, Libra)];
    assert_eq!(bhaaskara(&at), None);
    let at = [
        (Sun, Cancer),
        (Mercury, Leo),
        (Moon, Leo),
        (Jupiter, Sagittarius),
    ];
    assert_eq!(bhaaskara(&at), None);
}

#[test]
fn kulavardhana_is_each_planet_in_the_5th_from_the_lagna_moon_or_sun() {
    use Body::*;
    use Rasi::*;
    let kulavardhana = |at: &[(Body, Rasi)]| from_lagna(Aries, Leo, at, YogaName::Kulavardhana);
    // Everything in Le, the lagna's 5th; then the Sun in the Moon's 5th and Mars in the Sun's.
    assert_eq!(present(kulavardhana(&[])), PLANETS);
    let at = [(Sun, Sagittarius), (Mars, Aries)];
    assert_eq!(present(kulavardhana(&at)), PLANETS);
    // The nodes are no planets here (ADR 0014).
    let at = [(Rahu, Taurus), (Ketu, Scorpio)];
    assert_eq!(present(kulavardhana(&at)), PLANETS);
    assert_eq!(kulavardhana(&[(Jupiter, Cancer)]), None);
}

#[test]
fn vasumati_is_every_benefic_in_the_upachayas_from_the_lagna_its_full_results_to_judge() {
    use Body::*;
    use Rasi::*;
    use ToJudge::{MaleficInUpachaya, Strong};
    let vasumati = |at: &[(Body, Rasi)]| from_lagna(Aries, Aries, at, YogaName::Vasumati);
    let whole = [
        (Moon, Gemini),
        (Mercury, Virgo),
        (Jupiter, Capricorn),
        (Venus, Aquarius),
    ];
    assert_eq!(
        formed(vasumati(&whole)),
        (
            None,
            vec![
                Strong(Moon),
                Strong(Mercury),
                Strong(Jupiter),
                Strong(Venus)
            ],
            vec![Moon, Mercury, Jupiter, Venus]
        )
    );
    // Every benefic, not any.
    assert_eq!(vasumati(&whole[1..]), None);
    assert_eq!(vasumati(&whole[..3]), None);
    // A malefic there is named, and Mars beside him makes Mercury kroora, so not a benefic to place.
    let at = [whole[0], whole[1], whole[2], whole[3], (Mars, Virgo)];
    assert_eq!(
        formed(vasumati(&at)),
        (
            None,
            vec![
                Strong(Moon),
                Strong(Jupiter),
                Strong(Venus),
                MaleficInUpachaya(Mars),
                MaleficInUpachaya(Mercury)
            ],
            vec![Moon, Jupiter, Venus]
        )
    );
    // A waning Moon need not be there; the Sun in Ta has her 30° behind him, and a kroora Mercury may stay out.
    let at = [
        (Sun, Taurus),
        (Mercury, Aries),
        (Jupiter, Capricorn),
        (Venus, Aquarius),
    ];
    assert_eq!(formed(vasumati(&at)).2, [Jupiter, Venus]);
    // Counted from the lagna, not the Moon: Vi and Ta are her 3rd and 11th.
    let at = [(Moon, Cancer), (Jupiter, Virgo), (Venus, Taurus)];
    assert_eq!(vasumati(&at), None);
}

#[test]
fn a_tied_mercury_never_decides_vasumati() {
    use Body::*;
    use Rasi::*;
    use ToJudge::{MaleficInUpachaya, Strong};
    // Tied beside Venus and Mars in Vi he has a benefic there, and is listed with no to-judge of his nature.
    let at = [
        (Moon, Gemini),
        (Mercury, Virgo),
        (Venus, Virgo),
        (Mars, Virgo),
        (Jupiter, Capricorn),
    ];
    assert_eq!(
        formed(from_lagna(Aries, Aries, &at, YogaName::Vasumati)),
        (
            None,
            vec![
                Strong(Moon),
                Strong(Mercury),
                Strong(Jupiter),
                Strong(Venus),
                MaleficInUpachaya(Mars)
            ],
            vec![Moon, Mercury, Jupiter, Venus]
        )
    );
    // Tied out of them, the benefic beside him is out of them too.
    let at = [
        (Moon, Gemini),
        (Mercury, Taurus),
        (Venus, Taurus),
        (Mars, Taurus),
        (Jupiter, Capricorn),
    ];
    assert_eq!(from_lagna(Aries, Aries, &at, YogaName::Vasumati), None);
}

#[test]
fn gandharva_needs_its_four_conditions_and_leaves_the_suns_strength_to_judge() {
    use Body::*;
    use Rasi::*;
    let gandharva = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Gandharva);
    // Saturn in the pile, the 9th from the 7th; Mars conjoined there by Jupiter.
    let whole = [(Sun, Aries), (Moon, Sagittarius)];
    assert_eq!(
        formed(gandharva(&whole)),
        (
            None,
            vec![ToJudge::Strong(Sun)],
            vec![Sun, Moon, Mars, Jupiter, Saturn]
        )
    );
    // Jupiter's 5th on Mars in Le.
    let at = [whole[0], whole[1], (Jupiter, Aries), (Mars, Leo)];
    assert_eq!(formed(gandharva(&at)).2, [Sun, Moon, Mars, Jupiter, Saturn]);
    // The 10th lord in the 7th itself and in the 11th, its 5th: the same rasi counts (ADR 0014).
    for saturn in [Libra, Aquarius] {
        let at = [whole[0], whole[1], (Saturn, saturn)];
        assert!(gandharva(&at).is_some(), "{saturn:?}");
    }
    // Neither conjoined nor aspected, with Jupiter's aspects on Li, Sg and Aq.
    assert_eq!(gandharva(&[whole[0], whole[1], (Mars, Taurus)]), None);
    // The 10th lord in the 2nd, no trine of the 7th.
    assert_eq!(gandharva(&[whole[0], whole[1], (Saturn, Taurus)]), None);
    // The Sun not exalted, and the Moon out of the 9th.
    assert_eq!(gandharva(&[whole[1]]), None);
    assert_eq!(gandharva(&[whole[0]]), None);
    // A Jupiter who is the lagna lord does not conjoin himself.
    let at = [(Sun, Aries), (Moon, Leo)];
    let row = from_lagna(Sagittarius, Gemini, &at, YogaName::Gandharva);
    assert_eq!(row, None);
}

#[test]
fn go_is_jupiter_in_sagittarius_with_the_2nd_lord_and_an_exalted_lagna_lord() {
    use Body::*;
    use Rasi::*;
    let go = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Go);
    let whole = [
        (Jupiter, Sagittarius),
        (Venus, Sagittarius),
        (Mars, Capricorn),
    ];
    assert_eq!(
        formed(go(&whole)),
        (
            None,
            vec![ToJudge::Strong(Jupiter)],
            vec![Mars, Jupiter, Venus]
        )
    );
    // His own Pisces is not his moolatrikona.
    assert_eq!(go(&[(Jupiter, Pisces), (Venus, Pisces), whole[2]]), None);
    // The 2nd lord away; the lagna lord not exalted.
    assert_eq!(go(&[whole[0], (Venus, Aries), whole[2]]), None);
    assert_eq!(go(&[whole[0], whole[1], (Mars, Aries)]), None);
    // A Jupiter who is the 2nd lord is not with himself.
    let at = [whole[0], whole[2]];
    assert_eq!(from_lagna(Scorpio, Gemini, &at, YogaName::Go), None);
}

#[test]
fn gos_moolatrikona_is_the_whole_sign_past_the_first_10_degrees_of_note_5() {
    use Body::*;
    use Rasi::*;
    // Exercise 41's Poorvaashaadha and the 29°49′ of §6.6's example both count as his moolatrikona.
    for jupiter in [dms(8, 0, 0, 0.0), dms(8, 15, 0, 0.0), dms(8, 29, 49, 0.0)] {
        let at = [
            (Jupiter, jupiter),
            (Venus, mid(Sagittarius)),
            (Mars, mid(Capricorn)),
        ];
        let row = with_degrees(Aries, &at, YogaName::Go);
        assert_eq!(formed(row).2, [Mars, Jupiter, Venus]);
    }
}

#[test]
fn go_in_a_varga_reads_jupiter_in_that_vargas_sagittarius() {
    use Body::*;
    // 28° Ar is D-9's Sagittarius, 22° Cn its Capricorn and 1° Ar its Aries.
    let at = [
        (Lagna, dms(0, 1, 0, 0.0)),
        (Jupiter, dms(0, 28, 0, 0.0)),
        (Venus, dms(0, 28, 0, 0.0)),
        (Mars, dms(3, 22, 0, 0.0)),
    ];
    let row = yoga_in(&scene(Rasi::Gemini, &at), Varga::Navamsa, YogaName::Go);
    assert_eq!(
        formed(row),
        (
            None,
            vec![ToJudge::Strong(Jupiter)],
            vec![Mars, Jupiter, Venus]
        )
    );
}

#[test]
fn chapa_is_the_4th_and_10th_lords_exchanged_with_an_exalted_lagna_lord() {
    use Body::*;
    use Rasi::*;
    let chapa = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Chapa);
    let at = [(Moon, Capricorn), (Saturn, Cancer), (Mars, Capricorn)];
    assert_eq!(present(chapa(&at)), [Moon, Mars, Saturn]);
    assert_eq!(chapa(&[at[0], at[1]]), None);
    assert_eq!(chapa(&[at[0], at[2]]), None);
    assert_eq!(chapa(&[at[1], at[2]]), None);
}

#[test]
fn chaapa_of_the_naabhasa_is_not_chapa_of_the_other_popular_yogas() {
    assert_ne!(YogaName::Chaapa, YogaName::Chapa);
    assert_eq!(YogaName::Chaapa.class(), YogaClass::Naabhasa);
    assert_eq!(YogaName::Chapa.class(), YogaClass::OtherPopular);
}

#[test]
fn pushkala_is_its_four_stated_conditions() {
    use Body::*;
    use Rasi::*;
    let pushkala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Pushkala);
    // Mars with the Moon in Li; Venus, her dispositor, in the 7th aspecting the lagna; the Sun in it.
    let whole = [(Mars, Libra), (Moon, Libra), (Venus, Libra), (Sun, Aries)];
    assert_eq!(present(pushkala(&whole)), [Sun, Moon, Mars, Venus]);
    // Each condition out: Mars away, Venus not aspecting, and no planet in the lagna, though nodes may be.
    assert_eq!(pushkala(&whole[1..]), None);
    let at = [whole[0], whole[1], (Venus, Cancer), whole[3]];
    assert_eq!(pushkala(&at), None);
    assert_eq!(pushkala(&whole[..3]), None);
    assert_eq!(
        pushkala(&[whole[0], whole[1], whole[2], (Rahu, Aries)]),
        None
    );
    // A Cancer lagna's lord is the Moon, who is not with herself.
    let at = [(Sun, Cancer), (Moon, Scorpio), (Mars, Capricorn)];
    assert_eq!(from_lagna(Cancer, Gemini, &at, YogaName::Pushkala), None);
}

#[test]
fn pushkalas_dispositor_may_stand_out_of_a_kendra_in_an_adhimitras_house() {
    use Body::*;
    use Rasi::*;
    let pushkala = |at: &[(Body, Rasi)]| from_aries(at, YogaName::Pushkala);
    // Jupiter, the Moon's dispositor from Sg, in the 5th: his 9th is on the lagna, and his friend the Sun is in Le's 2nd.
    let at = [
        (Moon, Sagittarius),
        (Mars, Sagittarius),
        (Jupiter, Leo),
        (Venus, Aries),
        (Sun, Virgo),
    ];
    assert_eq!(present(pushkala(&at)), [Moon, Mars, Jupiter, Venus]);
    // In Le's 5th the Sun is a friend by nature but an enemy for now, and so only sama.
    let at = [at[0], at[1], at[2], at[3], (Sun, Sagittarius)];
    assert_eq!(pushkala(&at), None);
}

#[test]
fn makuta_is_jupiter_9th_from_the_9th_lord_a_benefic_9th_from_him_and_saturn_in_the_10th() {
    use Body::*;
    use Rasi::*;
    // Taurus: Saturn, the 9th lord, in Aq, Jupiter 9th from him in Li, and Ge 9th from Jupiter.
    let makuta = |rest, at: &[(Body, Rasi)]| from_lagna(Taurus, rest, at, YogaName::Makuta);
    let whole = [(Saturn, Aquarius), (Jupiter, Libra)];
    assert_eq!(
        present(makuta(Gemini, &whole)),
        [Moon, Jupiter, Venus, Saturn]
    );
    // No benefic in Ge.
    assert_eq!(makuta(Cancer, &whole), None);
    // A Mercury alone there is one, and with Rahu beside him is not.
    let at = [whole[0], whole[1], (Mercury, Gemini)];
    assert_eq!(present(makuta(Cancer, &at)), [Mercury, Jupiter, Saturn]);
    let at = [whole[0], whole[1], (Mercury, Gemini), (Rahu, Gemini)];
    assert_eq!(makuta(Cancer, &at), None);
    // Jupiter not 9th from Saturn, with Venus 9th from him in Ta; Saturn out of the 10th, with Jupiter still 9th from him.
    assert_eq!(
        makuta(Cancer, &[whole[0], (Jupiter, Virgo), (Venus, Taurus)]),
        None
    );
    let at = [(Saturn, Pisces), (Jupiter, Scorpio), (Venus, Cancer)];
    assert_eq!(makuta(Gemini, &at), None);
    // An Aries lagna's 9th lord is Jupiter himself, never in his own 9th, with Venus 9th from him in Ar.
    let at = [(Saturn, Capricorn), (Jupiter, Leo), (Venus, Aries)];
    assert_eq!(from_aries(&at, YogaName::Makuta), None);
}

/// `name`'s row in the rasi chart with the lagna at 15° of `lagna`, the rest in Gemini, but for `at`.
fn with_degrees(lagna: Rasi, at: &[(Body, f64)], name: YogaName) -> Option<Yoga> {
    let mut at = at.to_vec();
    at.push((Body::Lagna, mid(lagna)));
    yoga_in(&scene(Rasi::Gemini, &at), Varga::RasiChart, name)
}

#[test]
fn a_lagna_lord_alone_dignified_forms_no_mridanga() {
    use Body::*;
    use Rasi::*;
    // Mars at home in the lagna, and no other planet dignified in a kendra or trikona: one graha forms no yoga (ADR 0014).
    let at = [(Mars, Aries), (Rahu, Cancer), (Ketu, Capricorn)];
    assert_eq!(from_aries(&at, YogaName::Mridanga), None);
    // Venus at home in the 7th beside him in the rule makes two.
    let at = [(Mars, Aries), (Venus, Libra)];
    assert_eq!(formed(from_aries(&at, YogaName::Mridanga)).2, [Mars, Venus]);
}

#[test]
fn vishnu_is_the_9th_and_10th_lords_and_the_9th_lords_navamsa_dispositor_in_the_2nd() {
    use Body::*;
    use Rasi::*;
    let vishnu = |at: &[(Body, f64)]| with_degrees(Aries, at, YogaName::Vishnu);
    // Jupiter at 15° Ta is in D-9's Taurus, so Venus.
    let whole = [
        (Jupiter, mid(Taurus)),
        (Saturn, mid(Taurus)),
        (Venus, mid(Taurus)),
    ];
    assert_eq!(present(vishnu(&whole)), [Jupiter, Venus, Saturn]);
    assert_eq!(vishnu(&whole[1..]), None);
    assert_eq!(vishnu(&[whole[0], whole[2]]), None);
    assert_eq!(vishnu(&whole[..2]), None);
    // At 1° Ta he is in D-9's Capricorn, so Saturn, and Venus has no part.
    let at = [(Jupiter, dms(1, 1, 0, 0.0)), (Saturn, mid(Taurus))];
    assert_eq!(present(vishnu(&at)), [Jupiter, Saturn]);
}

#[test]
fn a_taurus_lagnas_vishnu_needs_a_navamsa_dispositor_other_than_saturn() {
    use Body::*;
    use Rasi::*;
    let vishnu = |at: &[(Body, f64)]| {
        let mut at = at.to_vec();
        at.push((Lagna, mid(Taurus)));
        yoga_in(&scene(Cancer, &at), Varga::RasiChart, YogaName::Vishnu)
    };
    // At 11° and 15° Ge he is in D-9's Capricorn and Aquarius, at home, so his own dispositor.
    for saturn in [dms(2, 11, 0, 0.0), mid(Gemini)] {
        assert_eq!(vishnu(&[(Saturn, saturn)]), None);
        // Rahu, the 10th's other lord, beside him does not help.
        assert_eq!(vishnu(&[(Saturn, saturn), (Rahu, mid(Gemini))]), None);
    }
    // At 1° Ge he is in D-9's Libra, so Venus, who must then be in the 2nd.
    let saturn = (Saturn, dms(2, 1, 0, 0.0));
    assert_eq!(vishnu(&[saturn]), None);
    assert_eq!(
        present(vishnu(&[saturn, (Venus, mid(Gemini))])),
        [Venus, Saturn]
    );
}

#[test]
fn a_9th_lord_disposing_of_himself_forms_no_vishnu_even_with_another_10th_lord() {
    use Body::*;
    use Rasi::*;
    // From Aries: Jupiter at 8° Ta is in D-9's Pisces, at home, with Saturn the 10th lord beside him.
    let at = [(Jupiter, dms(1, 8, 0, 0.0)), (Saturn, mid(Taurus))];
    assert_eq!(with_degrees(Aries, &at, YogaName::Vishnu), None);
}

#[test]
fn only_vishnu_needs_a_dispositor_other_than_the_lord_it_is_taken_from() {
    use Body::*;
    use Rasi::*;
    // Sg's 10th lord and Le's 2nd lord: Mercury at 18° Vi is in D-9's Gemini, at home (ADR 0014).
    let mercury = (Mercury, dms(5, 18, 0, 0.0));
    let at = [mercury, (Jupiter, mid(Virgo))];
    let row = with_degrees(Sagittarius, &at, YogaName::Gouri);
    assert_eq!(present(row), [Mercury, Jupiter]);
    let at = [mercury, (Mars, mid(Virgo))];
    let row = with_degrees(Leo, &at, YogaName::Bhaarathi);
    assert_eq!(present(row), [Mars, Mercury]);
    // Le's 6th and 9th lords: Saturn at 21° Sc is in D-9's Capricorn and Mars at 15° Sc in its Scorpio, each at home.
    let at = [
        (Saturn, dms(7, 21, 0, 0.0)),
        (Mars, mid(Scorpio)),
        (Sun, mid(Scorpio)),
    ];
    let row = with_degrees(Leo, &at, YogaName::Chandikaa);
    assert_eq!(present(row), [Sun, Mars, Saturn]);
    // Cp's lagna lord beside the Moon in Cn, who is her own dispositor and aspects the lagna.
    let at = [
        (Saturn, mid(Cancer)),
        (Moon, mid(Cancer)),
        (Sun, mid(Capricorn)),
    ];
    let row = with_degrees(Capricorn, &at, YogaName::Pushkala);
    assert_eq!(present(row), [Sun, Moon, Saturn]);
}

#[test]
fn gouri_is_the_10th_lords_navamsa_dispositor_exalted_in_the_10th_with_the_lagna_lord() {
    use Body::*;
    use Rasi::*;
    let gouri = |at: &[(Body, f64)]| with_degrees(Libra, at, YogaName::Gouri);
    // The Moon rules Libra's 10th; at 28° Ar she is in D-9's Sagittarius, so Jupiter, exalted in Cn beside Venus.
    let whole = [
        (Moon, dms(0, 28, 0, 0.0)),
        (Jupiter, mid(Cancer)),
        (Venus, mid(Cancer)),
    ];
    assert_eq!(present(gouri(&whole)), [Moon, Jupiter, Venus]);
    // Her navamsa elsewhere, Jupiter out of Cancer, and Venus away.
    assert_eq!(gouri(&[(Moon, mid(Taurus)), whole[1], whole[2]]), None);
    assert_eq!(gouri(&[whole[0], (Jupiter, mid(Leo)), whole[2]]), None);
    assert_eq!(gouri(&[whole[0], whole[1], (Venus, mid(Leo))]), None);
    // At 15° Le she is in D-9's Leo, so the Sun: exalted in Ar out of the 10th, and in the 10th unexalted.
    let at = [(Moon, mid(Leo)), (Sun, mid(Aries)), (Venus, mid(Aries))];
    assert_eq!(gouri(&at), None);
    let at = [(Moon, mid(Leo)), (Sun, mid(Cancer)), (Venus, mid(Cancer))];
    assert_eq!(gouri(&at), None);
    // Aries: Saturn at 15° Cn is in D-9's Scorpio, so Mars exalted in the 10th, but he is the lagna lord and joins no one.
    let at = [(Saturn, mid(Cancer)), (Mars, mid(Capricorn))];
    assert_eq!(with_degrees(Aries, &at, YogaName::Gouri), None);
}

#[test]
fn chandikaa_is_a_fixed_lagna_aspected_by_the_6th_lord_with_the_sun_beside_the_dispositors() {
    use Body::*;
    use Rasi::*;
    let chandikaa = |at: &[(Body, f64)]| with_degrees(Leo, at, YogaName::Chandikaa);
    // Saturn, Leo's 6th lord, aspects it from Sc and gives Mars in D-9; Mars, the 9th lord, gives Venus from D-9's Taurus.
    let whole = [
        (Saturn, mid(Scorpio)),
        (Mars, mid(Taurus)),
        (Sun, mid(Taurus)),
        (Venus, mid(Taurus)),
    ];
    assert_eq!(present(chandikaa(&whole)), [Sun, Mars, Venus, Saturn]);
    // Saturn in Ta aspects Cn, Sc and Aq, and both dispositors are then Venus.
    let at = [(Saturn, mid(Taurus)), whole[1], whole[2], whole[3]];
    assert_eq!(chandikaa(&at), None);
    // Venus or the Sun away.
    assert_eq!(
        chandikaa(&[whole[0], whole[1], whole[2], (Venus, mid(Aries))]),
        None
    );
    assert_eq!(
        chandikaa(&[whole[0], whole[1], (Sun, mid(Aries)), whole[3]]),
        None
    );
    // Aries is movable, though Mercury in Li aspects it and the pile holds the Sun and D-9 Aquarius's lords.
    let at = [(Mercury, mid(Libra))];
    assert_eq!(with_degrees(Aries, &at, YogaName::Chandikaa), None);
}

#[test]
fn the_sun_as_one_of_chandikaas_dispositors_leaves_the_other_but_not_as_both() {
    use Body::*;
    use Rasi::*;
    let chandikaa = |at: &[(Body, f64)]| with_degrees(Leo, at, YogaName::Chandikaa);
    // Mars at 15° Ar is in D-9's Leo, so the Sun, who is beside Mars, Saturn's dispositor.
    let at = [
        (Saturn, mid(Scorpio)),
        (Mars, mid(Aries)),
        (Sun, mid(Aries)),
    ];
    assert_eq!(present(chandikaa(&at)), [Sun, Mars, Saturn]);
    // Saturn at 5° Sc is in D-9's Leo as well, so the Sun would join only himself.
    let at = [
        (Saturn, dms(7, 5, 0, 0.0)),
        (Mars, mid(Aries)),
        (Sun, mid(Aries)),
    ];
    assert_eq!(chandikaa(&at), None);
}

#[test]
fn bhaarathi_is_a_navamsa_dispositor_of_the_2nd_5th_or_11th_lord_exalted_with_the_9th_lord() {
    use Body::*;
    use Rasi::*;
    let bhaarathi = |at: &[(Body, f64)]| with_degrees(Aries, at, YogaName::Bhaarathi);
    // At 15° Cn each lord is in D-9's Scorpio, so Mars, exalted in Cp beside Jupiter, the 9th lord.
    let exalted_pair = [(Mars, mid(Capricorn)), (Jupiter, mid(Capricorn))];
    for (lord, expected) in [
        (Venus, [Mars, Jupiter, Venus]),
        (Sun, [Sun, Mars, Jupiter]),
        (Saturn, [Mars, Jupiter, Saturn]),
    ] {
        let at = [(lord, mid(Cancer)), exalted_pair[0], exalted_pair[1]];
        assert_eq!(present(bhaarathi(&at)), expected, "{lord:?}");
    }
    // Mercury rules the 3rd and 6th, no house of the rule.
    let at = [(Mercury, mid(Cancer)), exalted_pair[0], exalted_pair[1]];
    assert_eq!(bhaarathi(&at), None);
    let lord = (Saturn, mid(Cancer));
    assert_eq!(
        bhaarathi(&[lord, (Mars, mid(Aries)), exalted_pair[1]]),
        None
    );
    assert_eq!(
        bhaarathi(&[lord, exalted_pair[0], (Jupiter, mid(Aries))]),
        None
    );
    // At 28° Ar Saturn is in D-9's Sagittarius, so Jupiter exalted, but he is the 9th lord and joins no one.
    let at = [(Saturn, dms(0, 28, 0, 0.0)), (Jupiter, mid(Cancer))];
    assert_eq!(bhaarathi(&at), None);
}

#[test]
fn vidyut_is_the_11th_lord_exalted_with_venus_in_a_kendra_from_the_lagna_lord_to_judge_at_any_degree()
 {
    use Body::*;
    use Rasi::*;
    let vidyut = |saturn: f64, more: &[(Body, f64)]| {
        let mut at = vec![(Saturn, saturn), (Venus, mid(Libra)), (Mars, mid(Aries))];
        at.extend_from_slice(more);
        with_degrees(Aries, &at, YogaName::Vidyut)
    };
    // At his deep degree and far from it alike.
    for degree in [dms(6, 20, 0, 0.0), dms(6, 1, 0, 0.0), dms(6, 29, 0, 0.0)] {
        assert_eq!(
            formed(vidyut(degree, &[])),
            (
                None,
                vec![ToJudge::DeepExaltation(Saturn)],
                vec![Mars, Venus, Saturn]
            )
        );
    }
    // Out of his uchcha rasi, even with Venus beside him.
    assert_eq!(vidyut(mid(Capricorn), &[(Venus, mid(Capricorn))]), None);
    let saturn = dms(6, 20, 0, 0.0);
    // Venus away, and the lagna lord in Ta, from whom Li is the 6th.
    assert_eq!(vidyut(saturn, &[(Venus, mid(Scorpio))]), None);
    assert_eq!(vidyut(saturn, &[(Mars, mid(Taurus))]), None);
    // Cancer: Venus rules the 11th and joins no one.
    let at = [(Venus, mid(Pisces)), (Moon, mid(Pisces))];
    assert_eq!(with_degrees(Cancer, &at, YogaName::Vidyut), None);
    // Taurus: Venus is the lagna lord too, leaving Jupiter in a kendra from her, as with Bheri (ADR 0014).
    let at = [(Jupiter, mid(Cancer)), (Venus, mid(Cancer))];
    let row = with_degrees(Taurus, &at, YogaName::Vidyut);
    assert_eq!(
        formed(row),
        (
            None,
            vec![ToJudge::DeepExaltation(Jupiter)],
            vec![Jupiter, Venus]
        )
    );
}

#[test]
fn jaya_is_the_10th_lord_exalted_and_the_6th_lord_debilitated_to_judge_at_any_degree() {
    use Body::*;
    use Rasi::*;
    let jaya = |saturn: f64, mercury: Rasi| {
        let at = [(Saturn, saturn), (Mercury, mid(mercury))];
        with_degrees(Aries, &at, YogaName::Jaya)
    };
    for degree in [dms(6, 20, 0, 0.0), dms(6, 1, 0, 0.0)] {
        assert_eq!(
            formed(jaya(degree, Pisces)),
            (
                None,
                vec![ToJudge::DeepExaltation(Saturn)],
                vec![Mercury, Saturn]
            )
        );
    }
    assert_eq!(jaya(mid(Capricorn), Pisces), None);
    assert_eq!(jaya(dms(6, 20, 0, 0.0), Virgo), None);
}

#[test]
fn a_node_lord_in_his_uchcha_leaves_vidyut_and_jaya_to_judge_with_no_deep_degree_printed() {
    use Body::*;
    use Rasi::*;
    // Rahu rules Aries's 11th and is exalted in the pile in Ge, beside Venus and Mars; Saturn is not exalted there.
    let row = with_degrees(Aries, &[], YogaName::Vidyut);
    assert_eq!(
        formed(row),
        (
            None,
            vec![ToJudge::DeepExaltation(Rahu)],
            vec![Mars, Venus, Rahu]
        )
    );
    // Rahu rules Taurus's 10th, and Venus, its 6th lord, is debilitated in Vi.
    let row = with_degrees(Taurus, &[(Venus, mid(Virgo))], YogaName::Jaya);
    assert_eq!(
        formed(row),
        (None, vec![ToJudge::DeepExaltation(Rahu)], vec![Venus, Rahu])
    );
}

#[test]
fn the_rasi_only_six_are_never_in_a_vargas_list() {
    use Body::*;
    use Rasi::*;
    // Every place below is the same rasi in D-9, so each rule would hold there too but for its being rasi-only.
    let first = |r: Rasi| dms(r as u8, 1, 0, 0.0);
    let last = |r: Rasi| dms(r as u8, 28, 0, 0.0);
    let cases = [
        (
            YogaName::Vishnu,
            vec![
                (Lagna, first(Aries)),
                (Jupiter, mid(Taurus)),
                (Saturn, mid(Taurus)),
                (Venus, mid(Taurus)),
            ],
        ),
        (
            YogaName::Gouri,
            vec![
                (Lagna, first(Libra)),
                (Moon, last(Sagittarius)),
                (Jupiter, first(Cancer)),
                (Venus, first(Cancer)),
            ],
        ),
        (
            YogaName::Chandikaa,
            vec![
                (Lagna, mid(Leo)),
                (Saturn, mid(Scorpio)),
                (Mars, mid(Taurus)),
                (Sun, mid(Taurus)),
                (Venus, mid(Taurus)),
            ],
        ),
        (
            YogaName::Bhaarathi,
            vec![
                (Lagna, first(Aries)),
                (Saturn, mid(Scorpio)),
                (Mars, first(Capricorn)),
                (Jupiter, first(Capricorn)),
            ],
        ),
        (
            YogaName::Vidyut,
            vec![
                (Lagna, first(Aries)),
                (Mars, first(Aries)),
                (Saturn, first(Libra)),
                (Venus, first(Libra)),
            ],
        ),
        (
            YogaName::Jaya,
            vec![
                (Lagna, first(Aries)),
                (Saturn, first(Libra)),
                (Mercury, last(Pisces)),
            ],
        ),
    ];
    for (name, at) in cases {
        let chart = scene(Gemini, &at);
        assert!(
            yoga_in(&chart, Varga::RasiChart, name).is_some(),
            "{name:?}"
        );
        for d in VARGAS.into_iter().filter(|&d| d != Varga::RasiChart) {
            assert_eq!(yoga_in(&chart, d, name), None, "{name:?} in {d:?}");
        }
    }
}

#[test]
fn vishnu_to_vimala_come_in_the_books_order() {
    use Body::*;
    use Rasi::*;
    use YogaName as N;
    let names_in = |at: &[(Body, Rasi)]| -> Vec<YogaName> {
        let mut at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
        at.push((Lagna, mid(Aries)));
        scene(Gemini, &at)
            .in_varga(Varga::RasiChart)
            .yogas()
            .into_iter()
            .map(|y| y.name)
            .filter(|y| {
                [
                    N::Trilochana,
                    N::Lakshmi,
                    N::Amsaavatara,
                    N::Vidyut,
                    N::Jaya,
                    N::Harsha,
                    N::Sarala,
                    N::Vimala,
                ]
                .contains(y)
            })
            .collect()
    };
    let at = [
        (Sun, Aries),
        (Moon, Leo),
        (Mars, Aries),
        (Mercury, Pisces),
        (Jupiter, Cancer),
        (Venus, Libra),
        (Saturn, Libra),
    ];
    assert_eq!(
        names_in(&at),
        [
            N::Trilochana,
            N::Lakshmi,
            N::Amsaavatara,
            N::Vidyut,
            N::Jaya
        ]
    );
    let at = [
        (Sun, Pisces),
        (Moon, Pisces),
        (Mars, Scorpio),
        (Mercury, Virgo),
        (Jupiter, Pisces),
    ];
    assert_eq!(
        names_in(&at),
        [N::Trilochana, N::Harsha, N::Sarala, N::Vimala]
    );
}

#[test]
fn yogas_come_in_the_books_order() {
    use Body::*;
    use Rasi::*;
    // Mars in the Sun's 12th and Jupiter in his 2nd; Mercury in the Moon's 2nd, and nothing in her 12th.
    let at = [
        (Lagna, Aquarius),
        (Sun, Scorpio),
        (Mercury, Scorpio),
        (Moon, Libra),
        (Mars, Libra),
        (Jupiter, Sagittarius),
        (Saturn, Aquarius),
        (Venus, Taurus),
    ];
    let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
    let names: Vec<YogaName> = scene(Gemini, &at)
        .in_varga(Varga::RasiChart)
        .yogas()
        .into_iter()
        .map(|y| y.name)
        .collect();
    assert_eq!(
        names,
        [
            YogaName::Ubhayachara,
            YogaName::BudhaAaditya,
            YogaName::Sunaphaa,
            YogaName::ChandraMangala,
            YogaName::Sasa,
            YogaName::Maalavya,
            YogaName::Paasa,
            YogaName::Asubha,
            YogaName::Mridanga,
            YogaName::Kalpadruma,
            YogaName::Brahma,
            YogaName::Lakshmi,
            YogaName::BasicRaaja,
            YogaName::BasicRaaja,
            YogaName::BasicRaaja,
            YogaName::Raaja(5),
            YogaName::Raaja(12),
            YogaName::RaajaSambandha(11),
            YogaName::RaajaSambandha(15),
            YogaName::Daridra(9),
            YogaName::Daridra(11),
        ]
    );
}

#[test]
fn naabhasa_yogas_come_in_the_books_order() {
    use YogaName::*;
    // Dala before Aakriti, and Vajra between Hala and Kamala: Maalaa beside Sarpa, then Vajra and Kamala.
    assert_eq!(
        naabhasa_names([1, 4, 7, 10, 7, 1, 10], 4),
        [Rajju, Maalaa, Sarpa, Vajra, Kamala]
    );
}

/// `name`'s rows in the rasi chart with the lagna mid-`lagna`, the rest at 15° of `rest`, but for `at`'s rasis and degrees,
/// and `lagnas` as [`special_lagnas`] gives them.
fn raaja_rows(
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

/// The grahas of `name`'s rows from an Aries lagna with the rest in Gemini, each at 15° but for `at`.
fn raaja_from_aries(at: &[(Body, Rasi)], name: YogaName) -> Vec<Vec<Body>> {
    let at: Vec<(Body, Rasi, f64)> = at.iter().map(|&(b, r)| (b, r, 15.0)).collect();
    let rows = raaja_rows(Rasi::Aries, Rasi::Gemini, &at, &[], name);
    rows.into_iter().map(|y| bodies(&y.grahas)).collect()
}

#[test]
fn association_is_a_conjunction_a_mutual_graha_drishti_or_a_parivartana() {
    use Body::*;
    use Rasi::*;
    let associated_at = |a: (Body, Rasi), b: (Body, Rasi)| {
        let chart = scene(Gemini, &[(a.0, mid(a.1)), (b.0, mid(b.1))]);
        chart.in_varga(Varga::RasiChart).associated(a.0, b.0)
    };
    assert!(associated_at((Sun, Aries), (Moon, Aries)));
    // Mars's 4th and Saturn's 10th.
    assert!(associated_at((Mars, Aries), (Saturn, Cancer)));
    // §11.7.1's example from an Aries lagna: the 4th lord in the 5th and the 5th lord in the 4th.
    assert!(associated_at((Moon, Leo), (Sun, Cancer)));
    // Saturn's 7th falls on Rahu, who has no graha drishti back (ADR 0013).
    assert!(!associated_at((Rahu, Aries), (Saturn, Libra)));
    // Saturn's 3rd falls on the Sun, whose 7th misses Saturn.
    assert!(!associated_at((Saturn, Aries), (Sun, Gemini)));
    let chart = scene(Gemini, &[]);
    assert!(!chart.in_varga(Varga::RasiChart).associated(Sun, Sun));
}

#[test]
fn a_kendra_and_a_trikona_lord_associated_are_a_basic_raaja_yoga() {
    use Body::*;
    use Rasi::*;
    // §11.7.1's parivartana of the 4th and 5th lords from an Aries lagna.
    let pair = vec![Sun, Moon];
    assert!(raaja_from_aries(&[(Moon, Leo), (Sun, Cancer)], YogaName::BasicRaaja).contains(&pair));
    assert!(!raaja_from_aries(&[(Moon, Leo), (Sun, Virgo)], YogaName::BasicRaaja).contains(&pair));
}

#[test]
fn closeness_subtracts_the_degrees_within_the_rasis_for_any_association() {
    use Body::*;
    use Rasi::*;
    let closeness = |at: &[(Body, Rasi, f64)], pair: &[Body]| {
        let rows = raaja_rows(Aries, Gemini, at, &[], YogaName::BasicRaaja);
        rows.into_iter()
            .find(|y| bodies(&y.grahas) == pair)
            .and_then(|y| y.magnitude)
            .map(|m| m.closeness)
    };
    // An aspect: Mars at 27° Ar and Saturn at 28° Cn are the lagna and 10th lords 1° apart.
    assert_eq!(
        closeness(
            &[(Mars, Aries, 27.0), (Saturn, Cancer, 28.0)],
            &[Mars, Saturn]
        ),
        Some(3600)
    );
    // A parivartana: the 4th lord at 3° Le and the 5th lord at 10° Cn.
    assert_eq!(
        closeness(&[(Moon, Leo, 3.0), (Sun, Cancer, 10.0)], &[Sun, Moon]),
        Some(7 * 3600)
    );
}

#[test]
fn a_raaja_row_lists_each_functional_malefic_conjoining_or_aspecting_one_of_its_grahas() {
    use Body::*;
    use Rasi::*;
    // From Ar: lagna lord Mars and 5th lord Sun in Le, Venus with them, Saturn's 3rd aspect from Ge, the waning Moon neutral,
    // Mercury, a malefic, not aspecting from Ge, and Venus no affliction of her own row with Mars.
    let at = [(Mars, Leo, 10.0), (Sun, Leo, 12.0), (Venus, Leo, 20.0)];
    let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::BasicRaaja);
    let afflictions = |pair: &[Body]| {
        let row = rows.iter().find(|y| bodies(&y.grahas) == pair).unwrap();
        row.afflictions
            .iter()
            .map(|a| (a.graha, a.afflicts, a.closeness))
            .collect::<Vec<_>>()
    };
    let degrees = |d: u32| Some(d * 3600);
    assert_eq!(
        afflictions(&[Sun, Mars]),
        [
            (Venus, Sun, degrees(8)),
            (Venus, Mars, degrees(10)),
            (Saturn, Sun, degrees(3)),
            (Saturn, Mars, degrees(5)),
        ]
    );
    assert!(afflictions(&[Mars, Venus]).iter().all(|a| a.0 != Venus));
}

#[test]
fn a_raaja_pairs_magnitude_holds_each_grahas_dasa_varga_amsa_in_the_rasi_chart_only() {
    use Body::*;
    use Rasi::*;
    // Conjoined at 1° Ar, so in Aries in D-9 too, where the lagna's Leo makes them the 9th and 7th lords.
    let (mars_at, saturn_at) = (dms(0, 1, 0, 0.0), dms(0, 1, 30, 0.0));
    let at = [
        (Body::Lagna, mid(Aries)),
        (Mars, mars_at),
        (Saturn, saturn_at),
        // A functional malefic from Ar and from Leo, so an affliction in both charts.
        (Venus, dms(0, 1, 45, 0.0)),
    ];
    let chart = scene(Gemini, &at);
    let pair = |d| {
        chart
            .in_varga(d)
            .yogas()
            .into_iter()
            .find(|y| y.name == YogaName::BasicRaaja && bodies(&y.grahas) == [Mars, Saturn])
    };
    let magnitude = pair(Varga::RasiChart).unwrap().magnitude.unwrap();
    let mars = amsabala(Mars, mars_at, VargaGroup::DasaVarga);
    let saturn = amsabala(Saturn, saturn_at, VargaGroup::DasaVarga);
    assert_eq!(
        magnitude.amsas,
        [(Mars, mars.0, mars.1), (Saturn, saturn.0, saturn.1)]
    );
    assert_eq!(magnitude.closeness, 30 * 60);
    let venus = |d| {
        let row = pair(d).unwrap();
        let found = row
            .afflictions
            .iter()
            .find(|a| a.graha == Venus)
            .map(|a| a.closeness);
        (row.magnitude.is_some(), found)
    };
    assert_eq!(venus(Varga::RasiChart), (true, Some(Some(45 * 60))));
    assert_eq!(venus(Varga::Navamsa), (false, Some(None)));
}

#[test]
fn the_9th_and_10th_lords_are_dharma_karmadhipati_instead() {
    use Body::*;
    use Rasi::*;
    // §11.7.2's Cp lagna: Mercury and Venus at 2° and 26° Ta, or at 2° and 3°.
    let rows = |venus: f64, name| {
        let at = [(Mercury, Taurus, 2.0), (Venus, Taurus, venus)];
        raaja_rows(Capricorn, Gemini, &at, &[], name)
    };
    let dk = rows(26.0, YogaName::DharmaKarmadhipati);
    assert_eq!(dk.len(), 1);
    assert_eq!(bodies(&dk[0].grahas), [Mercury, Venus]);
    assert_eq!(dk[0].magnitude.unwrap().closeness, 24 * 3600);
    assert_eq!(
        rows(3.0, YogaName::DharmaKarmadhipati)[0]
            .magnitude
            .unwrap()
            .closeness,
        3600
    );
    // From a Virgo lagna they are also the lagna and 9th lords, still one row, named for the 9th and 10th.
    let at = [(Mercury, Taurus, 2.0), (Venus, Taurus, 3.0)];
    let virgo = |name| raaja_rows(Virgo, Gemini, &at, &[], name);
    assert_eq!(virgo(YogaName::DharmaKarmadhipati).len(), 1);
    assert!(
        virgo(YogaName::BasicRaaja)
            .iter()
            .all(|y| bodies(&y.grahas) != [Mercury, Venus])
    );
}

#[test]
fn two_lords_of_one_house_are_no_raaja_pair() {
    use Body::*;
    use Rasi::*;
    let pairs = |lagna, at: &[(Body, Rasi)]| -> Vec<Vec<Body>> {
        let at: Vec<(Body, Rasi, f64)> = at.iter().map(|&(b, r)| (b, r, 15.0)).collect();
        [YogaName::BasicRaaja, YogaName::DharmaKarmadhipati]
            .into_iter()
            .flat_map(|name| raaja_rows(lagna, Gemini, &at, &[], name))
            .map(|y| bodies(&y.grahas))
            .collect()
    };
    // From Taurus Saturn rules the 9th and 10th, Rahu the 10th with him.
    assert!(!pairs(Taurus, &[(Saturn, Leo), (Rahu, Leo)]).contains(&vec![Saturn, Rahu]));
    // From Leo Mars rules the 4th and 9th, Ketu the 4th with him.
    assert!(!pairs(Leo, &[(Mars, Gemini), (Ketu, Gemini)]).contains(&vec![Mars, Ketu]));
}

#[test]
fn vipareeta_is_a_dusthana_lord_in_a_dusthana_or_two_conjoined_elsewhere_to_judge() {
    use Body::*;
    use Rasi::*;
    // From an Aries lagna: the 6th lord Mercury, the 8th lords Mars and Ketu, the 12th lord Jupiter.
    let rows = |at: &[(Body, Rasi)]| {
        let at: Vec<(Body, Rasi, f64)> = at.iter().map(|&(b, r)| (b, r, 15.0)).collect();
        raaja_rows(Aries, Gemini, &at, &[], YogaName::VipareetaRaaja)
    };
    let in_dusthanas = rows(&[
        (Mercury, Pisces),
        (Mars, Scorpio),
        (Ketu, Scorpio),
        (Jupiter, Cancer),
    ]);
    let grahas: Vec<Vec<Body>> = in_dusthanas.iter().map(|y| bodies(&y.grahas)).collect();
    assert_eq!(grahas, [vec![Mars], vec![Mercury], vec![Ketu]]);
    assert!(
        in_dusthanas
            .iter()
            .all(|y| y.to_judge.is_empty() && y.magnitude.is_none())
    );
    // Mercury and Jupiter together in the 4th; Mars and Ketu conjoined in the 8th are no pair.
    let rows = rows(&[
        (Mercury, Cancer),
        (Mars, Scorpio),
        (Ketu, Scorpio),
        (Jupiter, Cancer),
    ]);
    let pair = rows
        .iter()
        .find(|y| bodies(&y.grahas) == [Mercury, Jupiter])
        .unwrap();
    assert_eq!(
        pair.to_judge,
        [ToJudge::Strong(Mercury), ToJudge::Strong(Jupiter)]
    );
    assert_eq!(pair.magnitude.unwrap().closeness, 0);
    assert_eq!(rows.len(), 3);
}

/// Sun to Rahu at degrees ranking Venus AK and the Moon PK (ch. 8 §8.2), in the rasis given.
fn venus_ak_moon_pk(rasis: [Rasi; 8]) -> Vec<(Body, Rasi, f64)> {
    let degrees = [1.0, 2.0, 3.0, 4.0, 5.0, 29.0, 6.0, 29.5];
    GRAHAS[..8]
        .iter()
        .zip(rasis)
        .zip(degrees)
        .map(|((&b, r), deg)| (b, r, deg))
        .collect()
}

#[test]
fn raaja_1_is_ak_with_pk_or_the_lagna_and_5th_lords_conjoined_saying_which() {
    use Body::*;
    use Rasi::*;
    // From an Aries lagna, Mars and the Sun are the lagna and 5th lords.
    let raaja_1 = |sun, moon, mars, venus| {
        let at = venus_ak_moon_pk([sun, moon, mars, Gemini, Gemini, venus, Gemini, Gemini]);
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(1));
        assert!(rows.len() <= 1);
        rows.first().map(|y| (y.form, bodies(&y.grahas)))
    };
    assert_eq!(
        raaja_1(Gemini, Cancer, Aries, Cancer),
        Some((Some(1), vec![Moon, Venus]))
    );
    assert_eq!(
        raaja_1(Gemini, Leo, Gemini, Cancer),
        Some((Some(2), vec![Sun, Mars]))
    );
    assert_eq!(
        raaja_1(Cancer, Cancer, Cancer, Cancer),
        Some((Some(3), vec![Sun, Moon, Mars, Venus]))
    );
    assert_eq!(raaja_1(Gemini, Leo, Aries, Cancer), None);
}

#[test]
fn raaja_1_names_its_grahas_by_the_half_that_held() {
    use Body::*;
    use CharaKaraka::{Atma, Putra};
    use Rasi::*;
    // As above, from Aries: Mars lords the 1st (and 8th), the Sun the 5th, Venus is AK and the Moon PK.
    let roles = |sun, moon, mars, venus| {
        let at = venus_ak_moon_pk([sun, moon, mars, Gemini, Gemini, venus, Gemini, Gemini]);
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(1));
        rows[0]
            .grahas
            .iter()
            .map(|g| (g.graha, g.karaka, g.named_lord_of.clone(), g.bhava))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        roles(Gemini, Cancer, Aries, Cancer),
        [
            (Moon, Some(Putra), vec![], 4),
            (Venus, Some(Atma), vec![], 4)
        ]
    );
    assert_eq!(
        roles(Gemini, Leo, Gemini, Cancer),
        [(Sun, None, vec![5], 3), (Mars, None, vec![1], 3)]
    );
    assert_eq!(
        roles(Cancer, Cancer, Cancer, Cancer),
        [
            (Sun, None, vec![5], 4),
            (Moon, Some(Putra), vec![], 4),
            (Mars, None, vec![1], 4),
            (Venus, Some(Atma), vec![], 4),
        ]
    );
}

#[test]
fn a_karaka_left_empty_by_a_tie_forms_no_raaja_1() {
    use Body::*;
    use Rasi::*;
    // Everything at 15° shares the AK, leaving the PK empty; Mars and the Sun are apart.
    let rows = raaja_rows(
        Aries,
        Gemini,
        &[(Mars, Aries, 15.0)],
        &[],
        YogaName::Raaja(1),
    );
    assert_eq!(rows, []);
}

#[test]
fn raaja_2_is_the_lagna_and_5th_lords_exchanged_with_ak_and_pk_there_dignified() {
    use Body::*;
    use Rasi::*;
    // From an Aries lagna: AK Sun exalted in the lagna, PK Mars in the 5th, in his own navamsa at 1° Le, Taurus's at 5°.
    let raaja_2 = |mars: f64| {
        let at = [
            (Sun, Aries, 29.0),
            (Moon, Gemini, 10.0),
            (Mercury, Gemini, 9.0),
            (Jupiter, Gemini, 8.0),
            (Venus, Gemini, 7.0),
            (Mars, Leo, mars),
            (Saturn, Gemini, 0.8),
            (Rahu, Gemini, 29.5),
        ];
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(2));
        rows.iter().map(|y| bodies(&y.grahas)).collect::<Vec<_>>()
    };
    assert_eq!(raaja_2(1.0), [vec![Sun, Mars]]);
    assert_eq!(raaja_2(5.0), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_3_is_the_9th_lord_and_ak_in_the_1st_5th_or_7th_aspected_by_benefics() {
    use Body::*;
    use Rasi::*;
    // From an Aries lagna, Jupiter rules the 9th; the Sun at 29° is AK. Jupiter and Venus aspect each other's rasi.
    let raaja_3 = |venus| {
        let at = [
            (Sun, Libra, 29.0),
            (Jupiter, Aries, 15.0),
            (Venus, venus, 15.0),
        ];
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(3));
        rows.iter().map(|y| bodies(&y.grahas)).collect::<Vec<_>>()
    };
    assert_eq!(raaja_3(Libra), [vec![Sun, Jupiter]]);
    assert_eq!(raaja_3(Gemini), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_4_is_benefics_in_the_2nd_4th_and_5th_from_the_lagna_lord_and_ak() {
    use Body::*;
    use Rasi::*;
    // Mars, lagna lord of Aries and AK at 29°, in Aries.
    let raaja_4 = |mercury| {
        let at = [
            (Mars, Aries, 29.0),
            (Venus, Taurus, 15.0),
            (Jupiter, Cancer, 15.0),
            (Mercury, mercury, 15.0),
        ];
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(4));
        rows.iter().map(|y| bodies(&y.grahas)).collect::<Vec<_>>()
    };
    assert_eq!(raaja_4(Leo), [vec![Mars, Mercury, Jupiter, Venus]]);
    assert_eq!(raaja_4(Gemini), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_5_is_malefics_on_the_3rd_and_6th_from_the_lagna_lord_and_ak() {
    use Body::*;
    use Rasi::*;
    // Mars, lagna lord of Aries and AK at 29°, in Aries; the Sun and Rahu in the 3rd, the benefics out of it.
    let raaja_5 = |saturn| {
        let mut at = vec![(Mars, Aries, 29.0), (Saturn, saturn, 15.0)];
        at.extend([Moon, Mercury, Jupiter, Venus].map(|g| (g, Cancer, 15.0)));
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(5));
        rows.iter().map(|y| bodies(&y.grahas)).collect::<Vec<_>>()
    };
    assert_eq!(raaja_5(Virgo), [vec![Sun, Mars, Saturn, Rahu]]);
    // From Cancer Saturn's 3rd aspect reaches the 6th.
    assert_eq!(raaja_5(Cancer).len(), 1);
    assert_eq!(raaja_5(Gemini), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_5_never_counts_saturns_aspect_on_his_own_3rd() {
    use Body::*;
    use Rasi::*;
    // From Aquarius: lagna lord and AK Saturn at 29° Ar casts his 3rd on Gemini; Mars in Virgo is on his 6th.
    let raaja_5 = |sun| {
        let at = [(Saturn, Aries, 29.0), (Mars, Virgo, 15.0), (Sun, sun, 15.0)];
        raaja_rows(Aquarius, Cancer, &at, &[], YogaName::Raaja(5)).len()
    };
    assert_eq!(raaja_5(Cancer), 0);
    assert_eq!(raaja_5(Gemini), 1);
}

#[test]
fn raaja_6_is_a_planet_joining_or_aspecting_the_lagna_hl_and_gl() {
    use Body::*;
    use Rasi::*;
    // Jupiter in an Aries lagna aspects Leo and Libra.
    let raaja_6 = |gl| {
        let lagnas = [
            (SpecialLagna::HoraLagna, mid(Libra)),
            (SpecialLagna::GhatiLagna, mid(gl)),
        ];
        let rows = raaja_rows(
            Aries,
            Gemini,
            &[(Jupiter, Aries, 15.0)],
            &lagnas,
            YogaName::Raaja(6),
        );
        rows.iter().map(|y| bodies(&y.grahas)).collect::<Vec<_>>()
    };
    assert_eq!(raaja_6(Leo), [vec![Jupiter]]);
    assert_eq!(raaja_6(Virgo), Vec::<Vec<Body>>::new());
    // Without HL and GL there is nothing to count.
    assert_eq!(
        raaja_rows(
            Aries,
            Gemini,
            &[(Jupiter, Aries, 15.0)],
            &[],
            YogaName::Raaja(6)
        ),
        []
    );
}

#[test]
fn raaja_7_cannot_hold_in_the_books_d2_since_no_graha_drishti_joins_its_two_rasis() {
    for g in PLANETS {
        for (a, b) in [(Rasi::Cancer, Rasi::Leo), (Rasi::Leo, Rasi::Cancer)] {
            assert!(!graha_drishti(g, a).contains(&b), "{g:?}");
        }
    }
    assert!(
        RASIS
            .iter()
            .all(|&r| matches!(varga(Varga::Hora, mid(r)), Rasi::Cancer | Rasi::Leo))
    );
}

#[test]
fn raaja_8_is_the_lagna_hl_and_gl_each_holding_a_planet_in_own_or_uchcha_rasi() {
    use Body::*;
    use Rasi::*;
    let lagnas = [
        (SpecialLagna::HoraLagna, mid(Libra)),
        (SpecialLagna::GhatiLagna, mid(Cancer)),
    ];
    let raaja_8 = |moon| {
        let at = [
            (Mars, Aries, 15.0),
            (Saturn, Libra, 15.0),
            (Moon, moon, 15.0),
        ];
        let rows = raaja_rows(Aries, Gemini, &at, &lagnas, YogaName::Raaja(8));
        rows.iter().map(|y| bodies(&y.grahas)).collect::<Vec<_>>()
    };
    assert_eq!(raaja_8(Cancer), [vec![Moon, Mars, Saturn]]);
    assert_eq!(raaja_8(Gemini), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_9_is_the_lagna_of_d1_d9_and_d3_each_holding_a_planet_in_own_or_uchcha_rasi() {
    use Body::*;
    use Rasi::*;
    // A lagna at 1° Ar is Aries in D-1, D-9 and D-3; so is Mars at 2°, but at 5° his navamsa is Taurus.
    let raaja_9 = |mars| {
        let at = [(Lagna, Aries, 1.0), (Mars, Aries, mars)];
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(9));
        rows.iter().map(|y| bodies(&y.grahas)).collect::<Vec<_>>()
    };
    assert_eq!(raaja_9(2.0), [vec![Mars]]);
    assert_eq!(raaja_9(5.0), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_10_is_debilitated_planets_in_the_3rd_6th_or_8th_and_a_dignified_lagna_lord_aspecting_the_lagna()
 {
    use Body::*;
    use Rasi::*;
    // Mars exalted in the 10th aspects the Aries lagna with his 4th; the Moon is debilitated in the 8th.
    assert_eq!(
        raaja_from_aries(&[(Mars, Capricorn), (Moon, Scorpio)], YogaName::Raaja(10)),
        [vec![Moon, Mars]]
    );
    assert_eq!(
        raaja_from_aries(&[(Mars, Capricorn)], YogaName::Raaja(10)),
        Vec::<Vec<Body>>::new()
    );
    // Mars in his own Aries is in the lagna, not aspecting it.
    assert_eq!(
        raaja_from_aries(&[(Mars, Aries), (Moon, Scorpio)], YogaName::Raaja(10)),
        Vec::<Vec<Body>>::new()
    );
}

#[test]
fn raaja_11_is_the_dusthana_lords_weak_and_a_dignified_lagna_lord_aspecting_the_lagna() {
    use Body::*;
    use Rasi::*;
    // From Aries: Mars exalted, Saturn keeping his rasi friendly; Mercury, Ketu and Jupiter, the dusthana lords, debilitated.
    let raaja_11 = |at: &[(Body, Rasi)]| {
        let mut at: Vec<(Body, Rasi, f64)> = at.iter().map(|&(b, r)| (b, r, 15.0)).collect();
        at.extend([Mars, Jupiter].map(|g| (g, Capricorn, 15.0)));
        at.extend([(Saturn, Aquarius, 15.0), (Ketu, Gemini, 15.0)]);
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(11));
        rows.into_iter()
            .map(|y| (y.to_judge, bodies(&y.grahas)))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        raaja_11(&[(Mercury, Pisces)]),
        [(vec![], vec![Mars, Mercury, Jupiter, Ketu])]
    );
    // Mercury with the Sun in Taurus, a friend's rasi, is weak only if combust, so to judge (ADR 0014).
    assert_eq!(
        raaja_11(&[(Mercury, Taurus), (Sun, Taurus)]),
        [(
            vec![ToJudge::Combust(Mercury)],
            vec![Mars, Mercury, Jupiter, Ketu]
        )]
    );
    assert_eq!(raaja_11(&[(Mercury, Gemini), (Sun, Taurus)]), []);
}

#[test]
fn raaja_12_is_the_5th_and_9th_lords_conjoined_or_in_mutual_aspect() {
    use Body::*;
    use Rasi::*;
    assert_eq!(
        raaja_from_aries(&[], YogaName::Raaja(12)),
        [vec![Sun, Jupiter]]
    );
    assert_eq!(
        raaja_from_aries(&[(Sun, Libra), (Jupiter, Aries)], YogaName::Raaja(12)),
        [vec![Sun, Jupiter]]
    );
    assert_eq!(
        raaja_from_aries(&[(Sun, Libra), (Jupiter, Taurus)], YogaName::Raaja(12)),
        Vec::<Vec<Body>>::new()
    );
}

#[test]
fn raaja_13_is_the_4th_and_10th_lords_exchanged_each_aspected_by_the_5th_or_9th_lord() {
    use Body::*;
    use Rasi::*;
    // From Aries: the Moon in Cp and Saturn in Cn; the Sun's 7th reaches the Moon, Jupiter's 9th from Sc reaches Saturn.
    let raaja_13 = |jupiter| {
        let at = [
            (Moon, Capricorn),
            (Saturn, Cancer),
            (Sun, Cancer),
            (Jupiter, jupiter),
        ];
        raaja_from_aries(&at, YogaName::Raaja(13))
    };
    assert_eq!(raaja_13(Scorpio), [vec![Sun, Moon, Jupiter, Saturn]]);
    assert_eq!(raaja_13(Gemini), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_14_is_the_5th_lord_in_the_1st_4th_or_10th_joined_by_the_lagna_or_9th_lord() {
    use Body::*;
    use Rasi::*;
    assert_eq!(
        raaja_from_aries(&[(Sun, Cancer), (Jupiter, Cancer)], YogaName::Raaja(14)),
        [vec![Sun, Jupiter]]
    );
    assert_eq!(
        raaja_from_aries(&[(Sun, Cancer)], YogaName::Raaja(14)),
        Vec::<Vec<Body>>::new()
    );
    // Together in the 3rd.
    assert_eq!(
        raaja_from_aries(&[], YogaName::Raaja(14)),
        Vec::<Vec<Body>>::new()
    );
}

#[test]
fn raaja_15_is_a_vargottama_moon_aspected_by_four_planets_her_strength_to_judge() {
    use Body::*;
    use Rasi::*;
    // At 1° Ar the Moon is Aries in D-9 too, at 5° Taurus.
    let raaja_15 = |moon| {
        let mut at = vec![(Moon, Aries, moon)];
        at.extend([Sun, Mercury, Venus, Saturn].map(|g| (g, Libra, 15.0)));
        let rows = raaja_rows(Aries, Gemini, &at, &[], YogaName::Raaja(15));
        rows.into_iter()
            .map(|y| (y.to_judge, bodies(&y.grahas)))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        raaja_15(1.0),
        [(
            vec![ToJudge::Strong(Moon)],
            vec![Sun, Moon, Mercury, Venus, Saturn]
        )]
    );
    assert_eq!(raaja_15(5.0), []);
}

#[test]
fn raaja_16_is_four_planets_in_moolatrikona_or_uchcha_rasis() {
    use Body::*;
    use Rasi::*;
    let raaja_16 = |jupiter| {
        let at = [
            (Sun, Aries),
            (Moon, Taurus),
            (Mars, Capricorn),
            (Jupiter, jupiter),
        ];
        raaja_from_aries(&at, YogaName::Raaja(16))
    };
    assert_eq!(raaja_16(Sagittarius), [vec![Sun, Moon, Mars, Jupiter]]);
    assert_eq!(raaja_16(Gemini), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_17_is_every_benefic_in_a_kendra_and_a_malefic_in_each_of_the_3rd_6th_and_11th() {
    use Body::*;
    use Rasi::*;
    // From Aries: the Sun and Rahu in the 3rd, Ketu opposite; a waxing Moon and Mercury beside her in the 4th.
    let raaja_17 = |mars| {
        let at = [
            (Moon, Cancer),
            (Mercury, Cancer),
            (Jupiter, Libra),
            (Venus, Capricorn),
            (Saturn, Virgo),
            (Mars, mars),
        ];
        raaja_from_aries(&at, YogaName::Raaja(17))
    };
    assert_eq!(raaja_17(Aquarius), [GRAHAS[..8].to_vec()]);
    assert_eq!(raaja_17(Gemini), Vec::<Vec<Body>>::new());
}

/// §11.9's two combinations for each lagna from Aries: the 5th's graha and the 11th's, then the lagna lord and those
/// conjoining or aspecting him. Pisces's first, the Moon in both the 5th and 11th, is impossible (ADR 0014).
const DHANA: [(Option<Combination>, Body, &[Body]); 12] = {
    use Body::*;
    [
        (
            Some((Sun, &[Moon, Jupiter, Saturn])),
            Mars,
            &[Mercury, Venus, Saturn],
        ),
        (
            Some((Mercury, &[Moon, Mars, Jupiter])),
            Venus,
            &[Mercury, Saturn],
        ),
        (Some((Venus, &[Mars])), Mercury, &[Jupiter, Saturn]),
        (Some((Mars, &[Venus])), Moon, &[Mercury, Jupiter]),
        (Some((Jupiter, &[Mercury])), Sun, &[Mars, Jupiter]),
        (Some((Saturn, &[Sun, Moon])), Mercury, &[Jupiter, Saturn]),
        (Some((Saturn, &[Sun, Moon])), Venus, &[Mercury, Saturn]),
        (Some((Jupiter, &[Mercury])), Mars, &[Mercury, Venus, Saturn]),
        (Some((Mars, &[Venus])), Jupiter, &[Mars, Mercury]),
        (Some((Venus, &[Mars])), Saturn, &[Mars, Jupiter]),
        (
            Some((Mercury, &[Moon, Mars, Jupiter])),
            Saturn,
            &[Mars, Jupiter],
        ),
        (None, Jupiter, &[Mars, Mercury]),
    ]
};

/// The Dhana rows of a lagna mid-`lagna` with the rest in its 3rd, but for `at`.
fn dhana_rows(lagna: Rasi, at: &[(Body, u8)]) -> Vec<(YogaName, Option<u8>, Vec<Body>)> {
    let mut placed: Vec<(Body, f64)> = vec![(Body::Lagna, mid(lagna))];
    placed.extend(at.iter().map(|&(b, k)| (b, mid(kth(lagna, k.into())))));
    scene(kth(lagna, 3), &placed)
        .in_varga(Varga::RasiChart)
        .yogas()
        .into_iter()
        .filter(|y| y.name.class() == YogaClass::Dhana)
        .map(|y| {
            assert_eq!(y.to_judge, []);
            (y.name, y.form, bodies(&y.grahas))
        })
        .collect()
}

#[test]
fn each_lagnas_dhana_row_says_which_of_its_two_combinations_held() {
    for (i, &(first, lord, by)) in DHANA.iter().enumerate() {
        let lagna = RASIS[i];
        let name = YogaName::Dhana(i as u8 + 1);
        let sorted = |gs: Vec<Body>| in_graha_order(&gs);
        if let Some((fifth, eleventh)) = first {
            let mut at = vec![(fifth, 5)];
            at.extend(eleventh.iter().map(|&g| (g, 11)));
            let grahas = sorted([vec![fifth], eleventh.to_vec()].concat());
            assert_eq!(
                dhana_rows(lagna, &at),
                [(name, Some(1), grahas)],
                "{lagna:?}"
            );
            // One graha short of the 11th, or the 5th's elsewhere, is not it.
            assert_eq!(dhana_rows(lagna, &at[..at.len() - 1]), [], "{lagna:?}");
            assert_eq!(dhana_rows(lagna, &at[1..]), [], "{lagna:?}");
        }
        let mut at = vec![(lord, 1)];
        at.extend(by.iter().map(|&g| (g, 1)));
        let grahas = sorted([vec![lord], by.to_vec()].concat());
        assert_eq!(
            dhana_rows(lagna, &at),
            [(name, Some(2), grahas)],
            "{lagna:?}"
        );
        // From the 3rd none of them reaches the lagna, and the lagna lord must be in it.
        assert_eq!(dhana_rows(lagna, &at[..at.len() - 1]), [], "{lagna:?}");
        assert_eq!(dhana_rows(lagna, &at[1..]), [], "{lagna:?}");
    }
}

#[test]
fn a_dhana_rows_second_combination_counts_aspects_and_both_can_hold() {
    use Body::*;
    use Rasi::*;
    // Aries: Mars in the lagna, Mercury and Venus aspecting from the 7th, Saturn's 10th aspect from the 4th.
    let at = [(Mars, 1), (Mercury, 7), (Venus, 7), (Saturn, 4)];
    let grahas = vec![Mars, Mercury, Venus, Saturn];
    assert_eq!(
        dhana_rows(Aries, &at),
        [(YogaName::Dhana(1), Some(2), grahas)]
    );
    // Gemini: Venus in the 5th, Mars in the 11th, and Mercury in the lagna with Jupiter and Saturn.
    let at = [
        (Venus, 5),
        (Mars, 11),
        (Mercury, 1),
        (Jupiter, 1),
        (Saturn, 1),
    ];
    let grahas = vec![Mars, Mercury, Jupiter, Venus, Saturn];
    assert_eq!(
        dhana_rows(Gemini, &at),
        [(YogaName::Dhana(3), Some(3), grahas)]
    );
}

#[test]
fn pisces_dhana_row_comes_only_from_its_second_combination() {
    use Body::*;
    use Rasi::*;
    assert_eq!(dhana_rows(Pisces, &[(Moon, 5)]), []);
    assert_eq!(dhana_rows(Pisces, &[(Moon, 11)]), []);
    let at = [(Jupiter, 1), (Mars, 1), (Mercury, 1)];
    assert_eq!(
        dhana_rows(Pisces, &at),
        [(YogaName::Dhana(12), Some(2), vec![Mars, Mercury, Jupiter])]
    );
}

#[test]
fn the_dhana_basic_rule_is_the_moon_mercury_jupiter_or_venus_exalted_in_the_2nd() {
    use Body::*;
    use Rasi::*;
    let basic = |lagna, g| dhana_rows(lagna, &[(g, 2)]);
    assert_eq!(
        basic(Gemini, Jupiter),
        [(YogaName::BasicDhana, None, vec![Jupiter])]
    );
    assert_eq!(
        basic(Aries, Moon),
        [(YogaName::BasicDhana, None, vec![Moon])]
    );
    assert_eq!(
        basic(Aquarius, Venus),
        [(YogaName::BasicDhana, None, vec![Venus])]
    );
    // The Sun exalted in the 2nd, or Jupiter there unexalted, is not it.
    assert_eq!(basic(Pisces, Sun), []);
    assert_eq!(basic(Aries, Jupiter), []);
}

/// Raaja Sambandha (`n`)'s rows, as [`raaja_rows`] places them.
fn sambandha(lagna: Rasi, rest: Rasi, at: &[(Body, Rasi, f64)], n: u8) -> Vec<Yoga> {
    raaja_rows(lagna, rest, at, &[], YogaName::RaajaSambandha(n))
}

/// The grahas of `name`'s rows, as [`raaja_rows`] places them, each present.
fn present_rows(
    lagna: Rasi,
    rest: Rasi,
    at: &[(Body, Rasi, f64)],
    name: YogaName,
) -> Vec<Vec<Body>> {
    let rows = raaja_rows(lagna, rest, at, &[], name);
    rows.into_iter().map(|y| present(Some(y))).collect()
}

/// The grahas of Raaja Sambandha (`n`)'s rows from an Aries lagna, the rest at 15° of `rest`, each present.
fn sambandha_grahas(rest: Rasi, at: &[(Body, Rasi, f64)], n: u8) -> Vec<Vec<Body>> {
    present_rows(Rasi::Aries, rest, at, YogaName::RaajaSambandha(n))
}

#[test]
fn raaja_sambandha_1_is_the_10th_lord_conjoined_or_aspected_by_amk_or_his_dispositor() {
    use Body::*;
    use Rasi::*;
    // From Aries, AK Sun at 29° Ge and AmK Jupiter at 28° Le, whose 7th falls on Aq; the Sun, his dispositor, stays in Ge.
    let one = |saturn| {
        let at = [
            (Sun, Gemini, 29.0),
            (Jupiter, Leo, 28.0),
            (Saturn, saturn, 15.0),
        ];
        sambandha_grahas(Gemini, &at, 1)
    };
    assert_eq!(one(Aquarius), [vec![Jupiter, Saturn]]);
    assert_eq!(one(Gemini), [vec![Sun, Saturn]]);
    assert_eq!(one(Cancer), Vec::<Vec<Body>>::new());
    // AmK Mars at 28° in his own Sc is his own dispositor, so Ketu, Scorpio's other lord, beside Saturn in Le is not it.
    let own = |saturn| {
        let at = [
            (Sun, Gemini, 29.0),
            (Mars, Scorpio, 28.0),
            (Saturn, saturn, 15.0),
            (Ketu, Leo, 15.0),
        ];
        sambandha_grahas(Gemini, &at, 1)
    };
    assert_eq!(own(Scorpio), [vec![Mars, Saturn]]);
    assert_eq!(own(Leo), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_sambandha_2_is_the_11th_lord_aspecting_the_11th_and_no_malefic_on_the_10th_or_11th_but_that_aspect()
 {
    use Body::*;
    use Rasi::*;
    // From Aries, the 11th lord Saturn in Ta casts his 10th on Aq; the rest in Ta reach only Cn, Le, Sc and Sg.
    let two = |at: &[(Body, Rasi, f64)]| sambandha_grahas(Taurus, at, 2);
    assert_eq!(two(&[]), [vec![Saturn]]);
    assert_eq!(two(&[(Mars, Leo, 15.0)]), Vec::<Vec<Body>>::new());
    assert_eq!(two(&[(Saturn, Gemini, 15.0)]), Vec::<Vec<Body>>::new());
    // From Gemini, the 11th lord Mars casts his 7th from Li on Ar; from Vi his 8th too, but his 7th then falls on the 10th.
    let gemini = |mars| {
        present_rows(
            Gemini,
            Taurus,
            &[(Mars, mars, 15.0)],
            YogaName::RaajaSambandha(2),
        )
    };
    assert_eq!(gemini(Libra), [vec![Mars]]);
    assert_eq!(gemini(Virgo), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_sambandha_3_to_6_place_amk_with_ak_in_dignity_in_a_trine_or_from_ak() {
    use Body::*;
    use Rasi::*;
    let ak_amk = |n, ak, amk: (Body, Rasi)| {
        sambandha(Aries, Gemini, &[(Sun, ak, 29.0), (amk.0, amk.1, 28.0)], n)
    };
    // (3): AK and AmK conjoined.
    assert_eq!(present(ak_amk(3, Leo, (Moon, Leo)).pop()), [Sun, Moon]);
    assert_eq!(ak_amk(3, Leo, (Moon, Virgo)), []);
    // (4): own or uchcha, "very strong" to judge.
    let rows = ak_amk(4, Gemini, (Mars, Aries));
    assert_eq!(rows.len(), 1);
    assert_eq!(bodies(&rows[0].grahas), [Mars]);
    assert_eq!(rows[0].to_judge, [ToJudge::Strong(Mars)]);
    assert_eq!(ak_amk(4, Gemini, (Mars, Gemini)), []);
    // (5): a trine from the lagna.
    assert_eq!(present(ak_amk(5, Gemini, (Venus, Leo)).pop()), [Venus]);
    assert_eq!(ak_amk(5, Gemini, (Venus, Gemini)), []);
    // (6): a kendra or trine from AK, his own rasi among them.
    assert_eq!(
        present(ak_amk(6, Gemini, (Venus, Libra)).pop()),
        [Sun, Venus]
    );
    assert_eq!(
        present(ak_amk(6, Gemini, (Venus, Gemini)).pop()),
        [Sun, Venus]
    );
    assert_eq!(ak_amk(6, Gemini, (Venus, Cancer)), []);
}

#[test]
fn raaja_sambandha_7_is_malefics_in_the_3rd_and_6th_from_the_lagna_al_and_ak() {
    use Body::*;
    use Rasi::*;
    // From Aries, Mars in Ta puts the AL in Ge; AK Jupiter at 29° in the lagna. A waning Moon in Ge, Saturn in Vi, the Sun
    // in Le and Rahu in Sc; Mercury beside Venus in Pi.
    let seven = |saturn| {
        let at = [
            (Jupiter, Aries, 29.0),
            (Mars, Taurus, 15.0),
            (Moon, Gemini, 15.0),
            (Saturn, saturn, 15.0),
            (Sun, Leo, 15.0),
            (Rahu, Scorpio, 15.0),
            (Ketu, Taurus, 15.0),
        ];
        sambandha_grahas(Pisces, &at, 7)
    };
    assert_eq!(seven(Virgo), [vec![Sun, Moon, Jupiter, Saturn, Rahu]]);
    assert_eq!(seven(Pisces), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_sambandha_8_is_a_dignified_ak_in_a_kendra_or_trine_on_the_9th_lord() {
    use Body::*;
    use Rasi::*;
    // From Aries, AK Sun at 29° in his own Le, the 5th; Jupiter rules the 9th.
    let eight =
        |jupiter| sambandha_grahas(Gemini, &[(Sun, Leo, 29.0), (Jupiter, jupiter, 15.0)], 8);
    assert_eq!(eight(Leo), [vec![Sun, Jupiter]]);
    assert_eq!(eight(Aquarius), [vec![Sun, Jupiter]]);
    assert_eq!(eight(Gemini), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_sambandha_9_is_ak_the_moons_dispositor_in_the_lagna_with_a_benefic() {
    use Body::*;
    use Rasi::*;
    // From Aries, AK Mars at 29° in the lagna disposes of the Moon in Sc.
    let nine = |jupiter| {
        let at = [
            (Mars, Aries, 29.0),
            (Moon, Scorpio, 15.0),
            (Jupiter, jupiter, 15.0),
        ];
        sambandha_grahas(Gemini, &at, 9)
    };
    assert_eq!(nine(Aries), [vec![Moon, Mars, Jupiter]]);
    assert_eq!(nine(Gemini), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_sambandha_10_is_ak_in_the_5th_7th_9th_or_10th_with_a_benefic() {
    use Body::*;
    use Rasi::*;
    let ten =
        |sun, jupiter| sambandha_grahas(Gemini, &[(Sun, sun, 29.0), (Jupiter, jupiter, 15.0)], 10);
    assert_eq!(ten(Leo, Leo), [vec![Sun, Jupiter]]);
    assert_eq!(ten(Leo, Gemini), Vec::<Vec<Body>>::new());
    assert_eq!(ten(Taurus, Taurus), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_sambandha_11_is_a9_in_the_lagna_or_ak_in_the_9th_saying_which() {
    use Body::*;
    use Rasi::*;
    // From Aries, the 9th lord Jupiter in Aq, the 3rd from Sg, puts A9 in the lagna; AK is the Sun at 29°.
    let eleven = |sun, jupiter| {
        let rows = sambandha(
            Aries,
            Gemini,
            &[(Sun, sun, 29.0), (Jupiter, jupiter, 15.0)],
            11,
        );
        rows.into_iter()
            .map(|y| (y.form, bodies(&y.grahas)))
            .collect::<Vec<_>>()
    };
    assert_eq!(eleven(Gemini, Aquarius), [(Some(1), vec![])]);
    assert_eq!(eleven(Sagittarius, Gemini), [(Some(2), vec![Sun])]);
    assert_eq!(eleven(Sagittarius, Aquarius), [(Some(3), vec![Sun])]);
    assert_eq!(eleven(Gemini, Gemini), []);
}

#[test]
fn raaja_sambandha_12_is_the_11th_lord_in_the_11th_unaspected_by_malefics_and_ak_with_benefics() {
    use Body::*;
    use Rasi::*;
    // From Gemini, Mars in Ar rules the 11th; AK Sun at 29° with Jupiter in Cn; the rest in Ta reach Cn, Sc and Aq.
    let twelve = |saturn| {
        let at = [
            (Mars, Aries, 15.0),
            (Sun, Cancer, 29.0),
            (Jupiter, Cancer, 15.0),
            (Saturn, saturn, 15.0),
        ];
        present_rows(Gemini, Taurus, &at, YogaName::RaajaSambandha(12))
    };
    assert_eq!(twelve(Taurus), [vec![Sun, Mars, Jupiter]]);
    // Saturn's 10th aspect from Cn falls on Ar.
    assert_eq!(twelve(Cancer), Vec::<Vec<Body>>::new());
}

#[test]
fn raaja_sambandha_13_to_15_exchange_the_lagna_and_10th_lords_or_join_the_karakas() {
    use Body::*;
    use Rasi::*;
    // (13): from Aries, Mars rules the lagna and Saturn the 10th.
    let thirteen = |saturn| {
        sambandha_grahas(
            Gemini,
            &[(Mars, Capricorn, 15.0), (Saturn, saturn, 15.0)],
            13,
        )
    };
    assert_eq!(thirteen(Aries), [vec![Mars, Saturn]]);
    assert_eq!(thirteen(Gemini), Vec::<Vec<Body>>::new());
    // (14): the Moon and Venus in the 4th from AK Sun at 29° Ge.
    let fourteen = |venus| {
        let at = [
            (Sun, Gemini, 29.0),
            (Moon, Virgo, 15.0),
            (Venus, venus, 15.0),
        ];
        sambandha_grahas(Gemini, &at, 14)
    };
    assert_eq!(fourteen(Virgo), [vec![Sun, Moon, Venus]]);
    assert_eq!(fourteen(Gemini), Vec::<Vec<Body>>::new());
    // (15): the lagna lord Mars or AK Jupiter at 29° with the 5th lord Sun in a kendra or trine.
    let fifteen = |at: &[(Body, Rasi, f64)]| sambandha_grahas(Gemini, at, 15);
    assert_eq!(
        fifteen(&[(Sun, Cancer, 15.0), (Mars, Cancer, 15.0)]),
        [vec![Sun, Mars]]
    );
    assert_eq!(
        fifteen(&[(Sun, Leo, 15.0), (Jupiter, Leo, 29.0)]),
        [vec![Sun, Jupiter]]
    );
    assert_eq!(fifteen(&[]), Vec::<Vec<Body>>::new());
}

/// The grahas of Daridra (`n`)'s rows from an Aries lagna with the rest in `rest`, each present. Venus rules the 2nd and 7th.
fn daridra(rest: Rasi, at: &[(Body, Rasi)], n: u8) -> Vec<Vec<Body>> {
    let at: Vec<(Body, Rasi, f64)> = at.iter().map(|&(b, r)| (b, r, 15.0)).collect();
    present_rows(Rasi::Aries, rest, &at, YogaName::Daridra(n))
}

const NONE: Vec<Vec<Body>> = Vec::new();

#[test]
fn daridra_1_and_2_exchange_the_lagna_lord_with_the_12th_or_6th_lord_each_touched_by_a_maraka() {
    use Body::*;
    use Rasi::*;
    // (1): Mars in Pi and Jupiter in Ar; Venus's 7th falls on Pi, and Saturn, a malefic in the 7th, aspects Ar.
    let one = |venus| {
        daridra(
            Gemini,
            &[
                (Mars, Pisces),
                (Jupiter, Aries),
                (Venus, venus),
                (Saturn, Libra),
            ],
            1,
        )
    };
    assert_eq!(one(Virgo), [vec![Mars, Jupiter, Venus, Saturn]]);
    assert_eq!(one(Gemini), NONE);
    // (2): Mars in Vi joined by Venus, so a maraka whose 8th falls on Mercury in Ar, as Saturn's 7th from Li does.
    let two = |venus| {
        daridra(
            Gemini,
            &[
                (Mars, Virgo),
                (Mercury, Aries),
                (Venus, venus),
                (Saturn, Libra),
            ],
            2,
        )
    };
    assert_eq!(two(Virgo), [vec![Mars, Mercury, Venus, Saturn]]);
    assert_eq!(two(Cancer), NONE);
}

#[test]
fn daridra_3_is_ketu_with_the_lagna_or_moon_and_the_lagna_lord_in_the_8th_touched_by_a_maraka() {
    use Body::*;
    use Rasi::*;
    // Ketu beside the Moon in Ge; Mars in Sc aspected by Venus from Ta.
    let three = |moon| {
        daridra(
            Gemini,
            &[
                (Mars, Scorpio),
                (Venus, Taurus),
                (Ketu, Gemini),
                (Moon, moon),
            ],
            3,
        )
    };
    assert_eq!(three(Gemini), [vec![Moon, Mars, Venus, Ketu]]);
    assert_eq!(three(Cancer), NONE);
    let in_lagna = daridra(
        Gemini,
        &[(Mars, Scorpio), (Venus, Taurus), (Ketu, Aries)],
        3,
    );
    assert_eq!(in_lagna, [vec![Mars, Venus, Ketu]]);
}

#[test]
fn a_tied_mercury_aspecting_the_2nd_leaves_the_daridra_row_he_makes_a_maraka_to_judge() {
    use Body::*;
    use Rasi::*;
    // Mercury in Sc between kroora Mars and saumya Jupiter casts his 7th on Ta; Ketu in the lagna, Rahu in the 7th.
    let at = [
        (Lagna, mid(Aries)),
        (Mars, mid(Scorpio)),
        (Mercury, mid(Scorpio)),
        (Jupiter, mid(Scorpio)),
        (Ketu, mid(Aries)),
        (Rahu, mid(Libra)),
    ];
    let chart = scene(Gemini, &at);
    assert_eq!(
        chart.in_varga(Varga::RasiChart).natural_nature(Mercury),
        None
    );
    let row = yoga_in(&chart, Varga::RasiChart, YogaName::Daridra(3)).expect("listed");
    assert_eq!(bodies(&row.grahas), [Mars, Mercury, Ketu]);
    assert_eq!(row.to_judge, [ToJudge::MercurysNature]);
    // Alone, a tied Mercury is listed only as a lord.
    assert!(
        !chart
            .in_varga(Varga::RasiChart)
            .marakas()
            .contains(&Mercury)
    );
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
fn daridra_4_is_the_lagna_lord_with_a_malefic_in_a_dusthana_and_the_2nd_lord_low() {
    use Body::*;
    use Rasi::*;
    let four = |venus, moon| {
        daridra(
            Gemini,
            &[
                (Mars, Scorpio),
                (Saturn, Scorpio),
                (Venus, venus),
                (Moon, moon),
            ],
            4,
        )
    };
    assert_eq!(four(Virgo, Gemini), [vec![Mars, Venus, Saturn]]);
    // In the Moon's Cn beside her, an adhisatru.
    assert_eq!(four(Cancer, Cancer), [vec![Mars, Venus, Saturn]]);
    assert_eq!(four(Taurus, Gemini), NONE);
    assert_eq!(daridra(Gemini, &[(Mars, Scorpio), (Venus, Virgo)], 4), NONE);
}

#[test]
fn daridra_5_is_the_5th_lord_in_the_6th_and_9th_lord_in_the_12th_each_aspected_by_a_maraka() {
    use Body::*;
    use Rasi::*;
    // The Sun in Vi and Jupiter in Pi aspect each other. Venus in Pi aspects the Sun, who then aspects a 2nd and 7th lord, so
    // is a maraka on Jupiter, as is Saturn from Ge.
    let five = |venus| {
        daridra(
            Gemini,
            &[(Sun, Virgo), (Jupiter, Pisces), (Venus, venus)],
            5,
        )
    };
    assert_eq!(five(Pisces), [vec![Sun, Jupiter, Venus, Saturn]]);
    assert_eq!(five(Cancer), NONE);
}

#[test]
fn daridra_6_is_malefics_in_the_lagna_without_the_9th_and_10th_lords_touched_by_a_maraka() {
    use Body::*;
    use Rasi::*;
    let six = |at: &[(Body, Rasi)]| daridra(Gemini, at, 6);
    assert_eq!(six(&[(Sun, Aries), (Venus, Libra)]), [vec![Sun, Venus]]);
    assert_eq!(six(&[(Sun, Aries), (Venus, Libra), (Saturn, Aries)]), NONE);
    assert_eq!(six(&[(Sun, Aries)]), NONE);
    // Jupiter, the 9th lord, in the lagna too.
    assert_eq!(six(&[(Sun, Aries), (Venus, Libra), (Jupiter, Aries)]), NONE);
}

#[test]
fn daridra_7_is_the_dusthana_lords_dispositors_in_dusthanas_touched_by_malefics() {
    use Body::*;
    use Rasi::*;
    // Mercury, Mars and Jupiter, the 6th, 8th and 12th lords, in Le; the Sun, their dispositor, in Vi.
    let seven = |saturn| {
        let at = [
            (Mercury, Leo),
            (Mars, Leo),
            (Jupiter, Leo),
            (Sun, Virgo),
            (Saturn, saturn),
        ];
        daridra(Gemini, &at, 7)
    };
    assert_eq!(seven(Virgo), [vec![Sun, Saturn]]);
    assert_eq!(seven(Gemini), NONE);
}

#[test]
fn daridra_8_and_9_read_the_navamsa_in_the_rasi_chart_only() {
    use Body::*;
    use Rasi::*;
    // (8): the Moon at 1° Ar is in D-9's Aries, so Mars disposes of her; Venus in Cn.
    let eight = |mars| {
        let at = [
            (Moon, Aries, 1.0),
            (Mars, mars, 15.0),
            (Venus, Cancer, 15.0),
        ];
        present_rows(Aries, Gemini, &at, YogaName::Daridra(8))
    };
    assert_eq!(eight(Libra), [vec![Moon, Mars]]);
    assert_eq!(eight(Cancer), [vec![Moon, Mars, Venus]]);
    assert_eq!(eight(Gemini), NONE);
    // (9): the lagna at 15° Ar is in D-9's Leo, so Mars and the Sun, both aspected by Venus.
    assert_eq!(
        daridra(Gemini, &[(Mars, Libra), (Sun, Libra), (Venus, Aries)], 9),
        [vec![Sun, Mars, Venus]]
    );
    assert_eq!(
        daridra(Gemini, &[(Mars, Leo), (Sun, Leo), (Saturn, Sagittarius)], 9),
        NONE
    );
    let chart = scene(
        Gemini,
        &[
            (Lagna, mid(Aries)),
            (Mars, mid(Libra)),
            (Sun, mid(Libra)),
            (Venus, mid(Aries)),
        ],
    );
    assert_eq!(yoga_in(&chart, Varga::Navamsa, YogaName::Daridra(9)), None);
}

#[test]
fn daridra_11_is_planets_joined_by_dusthana_lords_untouched_by_trine_lords() {
    use Body::*;
    use Rasi::*;
    // The Moon joined by Mercury, the 6th lord, in Cp; the rest in Le, each joined by Mars, the lagna lord.
    let eleven = |jupiter| {
        daridra(
            Leo,
            &[(Moon, Capricorn), (Mercury, Capricorn), (Jupiter, jupiter)],
            11,
        )
    };
    assert_eq!(eleven(Leo), [vec![Moon, Mercury]]);
    // Jupiter, the 9th lord, aspects Cp from Cn.
    assert_eq!(eleven(Cancer), NONE);
}

#[test]
fn daridra_12_and_13_are_mars_and_saturn_or_the_sun_in_the_2nd_without_their_exception() {
    use Body::*;
    use Rasi::*;
    let twelve = |mercury| {
        daridra(
            Gemini,
            &[(Mars, Taurus), (Saturn, Taurus), (Mercury, mercury)],
            12,
        )
    };
    assert_eq!(twelve(Gemini), [vec![Mars, Saturn]]);
    // Mercury's 7th from Sc on them turns the poverty to wealth.
    assert_eq!(twelve(Scorpio), NONE);
    let thirteen = |saturn| daridra(Gemini, &[(Sun, Taurus), (Saturn, saturn)], 13);
    assert_eq!(thirteen(Scorpio), [vec![Sun, Saturn]]);
    assert_eq!(thirteen(Gemini), NONE);
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
fn baadhakas_are_table_31() {
    use Body::*;
    use Rasi::*;
    // Ch. 13 §13.3, Table 31 (pp. 168–169), row by row.
    let table_31: [(Rasi, Rasi, &[Body]); 12] = [
        (Aries, Aquarius, &[Saturn, Rahu]),
        (Taurus, Capricorn, &[Saturn]),
        (Gemini, Sagittarius, &[Jupiter]),
        (Cancer, Taurus, &[Venus]),
        (Leo, Aries, &[Mars]),
        (Virgo, Pisces, &[Jupiter]),
        (Libra, Leo, &[Sun]),
        (Scorpio, Cancer, &[Moon]),
        (Sagittarius, Gemini, &[Mercury]),
        (Capricorn, Scorpio, &[Mars, Ketu]),
        (Aquarius, Libra, &[Venus]),
        (Pisces, Virgo, &[Mercury]),
    ];
    for (rasi, sthaana, lords) in table_31 {
        assert_eq!(baadhaka_sthaana(rasi), sthaana, "{rasi:?}");
        assert_eq!(baadhakas(rasi), lords, "{rasi:?}");
    }
}

/// The yogadas of an Aries lagna with Mars in it and the rest in Gemini, HL and GL as given.
fn yogadas_from_aries(lagnas: &[(SpecialLagna, f64)]) -> Vec<Yogada> {
    let at = [
        (Body::Lagna, mid(Rasi::Aries)),
        (Body::Mars, mid(Rasi::Aries)),
    ];
    let entries = scene(Rasi::Gemini, &at).entries().try_into().unwrap();
    Chart::new(entries, lagnas.to_vec())
        .in_varga(Varga::RasiChart)
        .yogadas()
}

#[test]
fn a_yogada_reaches_hl_by_rasi_drishti_never_graha_drishti_alone() {
    use Rasi::*;
    // Mars in Aries owns and is in the lagna; Leo is in his rasi drishti, Cancer only in his graha drishti.
    let lagnas = [
        (SpecialLagna::HoraLagna, mid(Leo)),
        (SpecialLagna::GhatiLagna, mid(Cancer)),
    ];
    assert_eq!(
        yogadas_from_aries(&lagnas),
        [Yogada {
            graha: Body::Mars,
            gives: Gives::Money,
            links: [vec![Link::Owns, Link::Conjoins], vec![Link::Aspects]],
        }]
    );
}

#[test]
fn a_chart_without_a_vedic_day_has_no_yogadas() {
    assert_eq!(yogadas_from_aries(&[]), []);
}

/// The Tapaswi row of a Leo lagna with the rest in Gemini, each at 15° but for `at`.
fn tapaswi(at: &[(Body, Rasi, f64)]) -> Option<Yoga> {
    raaja_rows(Rasi::Leo, Rasi::Gemini, at, &[], YogaName::Tapaswi).pop()
}

#[test]
fn tapaswi_is_two_of_saturn_ketu_and_venus_together_and_the_third_aspecting_them() {
    use Body::*;
    use Rasi::*;
    // Example 44's placement: AK Saturn with Ketu in Vi, Venus aspecting from Pi, the 8th from Le (p. 174).
    let row = tapaswi(&[
        (Saturn, Virgo, 29.0),
        (Ketu, Virgo, 15.0),
        (Venus, Pisces, 15.0),
    ])
    .unwrap();
    assert_eq!(bodies(&row.grahas), [Venus, Saturn, Ketu]);
    assert_eq!(row.to_judge, []);
    assert_eq!(
        row.notes,
        [Note::AtmaKaraka(Saturn), Note::InTheEighth(Venus)]
    );
    // AK the Sun in Gemini reaches none of them.
    let row = tapaswi(&[
        (Saturn, Virgo, 10.0),
        (Ketu, Virgo, 15.0),
        (Venus, Pisces, 15.0),
        (Sun, Gemini, 29.0),
    ])
    .unwrap();
    assert_eq!(row.notes, [Note::InTheEighth(Venus)]);
    // Mars and Jupiter tie for AK outside the yoga, Mars aspecting Virgo by his 4th, Jupiter with Venus: each notes it.
    let row = tapaswi(&[
        (Saturn, Virgo, 10.0),
        (Ketu, Virgo, 15.0),
        (Venus, Pisces, 15.0),
        (Mars, Gemini, 29.0),
        (Jupiter, Pisces, 29.0),
    ])
    .unwrap();
    assert_eq!(
        row.notes,
        [
            Note::AtmaKaraka(Mars),
            Note::AtmaKaraka(Jupiter),
            Note::InTheEighth(Venus)
        ]
    );
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
fn tapaswi_never_has_ketu_as_the_one_aspecting_and_all_three_together_is_to_judge() {
    use Body::*;
    use Rasi::*;
    let ketu_third = [
        (Saturn, Virgo, 15.0),
        (Venus, Virgo, 15.0),
        (Ketu, Pisces, 15.0),
    ];
    assert_eq!(tapaswi(&ketu_third), None);
    let together = [
        (Saturn, Virgo, 15.0),
        (Venus, Virgo, 15.0),
        (Ketu, Virgo, 15.0),
    ];
    assert_eq!(
        tapaswi(&together).unwrap().to_judge,
        [ToJudge::AllInOneRasi]
    );
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

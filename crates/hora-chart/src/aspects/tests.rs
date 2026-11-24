use super::*;
use crate::tests::*;

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

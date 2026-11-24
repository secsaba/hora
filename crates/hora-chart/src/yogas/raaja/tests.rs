use super::*;
use crate::tests::*;

/// The grahas of `name`'s rows from an Aries lagna with the rest in Gemini, each at 15° but for `at`.
fn raaja_from_aries(at: &[(Body, Rasi)], name: YogaName) -> Vec<Vec<Body>> {
    let at: Vec<(Body, Rasi, f64)> = at.iter().map(|&(b, r)| (b, r, 15.0)).collect();
    let rows = raaja_rows(Rasi::Aries, Rasi::Gemini, &at, &[], name);
    rows.into_iter().map(|y| bodies(&y.grahas)).collect()
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

/// Raaja Sambandha (`n`)'s rows, as [`raaja_rows`] places them.
fn sambandha(lagna: Rasi, rest: Rasi, at: &[(Body, Rasi, f64)], n: u8) -> Vec<Yoga> {
    raaja_rows(lagna, rest, at, &[], YogaName::RaajaSambandha(n))
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

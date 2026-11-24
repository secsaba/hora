use super::*;
use crate::tests::*;

/// `name`'s row in the rasi chart of a scene with the lagna in Aries and the rest piled in the 3rd, Gemini, but for `at`.
fn from_aries(at: &[(Body, Rasi)], name: YogaName) -> Option<Yoga> {
    from_lagna(Rasi::Aries, Rasi::Gemini, at, name)
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

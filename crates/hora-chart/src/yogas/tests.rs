use super::*;
use crate::tests::*;

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

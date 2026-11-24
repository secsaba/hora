use super::*;
use crate::tests::*;

#[test]
fn avataras_are_section_3_2_1s() {
    use Avatara::*;
    let book = [
        Rama, Krishna, Narasimha, Buddha, Vaamana, ParasuRama, Koorma, Varaaha, Matsya,
    ];
    assert_eq!(GRAHAS.map(Body::avatara), book.map(Some));
}

#[test]
fn saumya_and_kroora_grahas_are_section_3_2_2s_with_saturn_from_13_2() {
    use Disposition::*;
    let book = [
        Some(Kroora),
        Some(ByPaksha),
        Some(Kroora),
        Some(ByCompany),
        Some(Saumya),
        Some(Saumya),
        Some(Kroora),
        Some(Kroora),
        Some(Kroora),
    ];
    assert_eq!(GRAHAS.map(Body::disposition), book);
}

#[test]
fn main_governances_are_section_3_2_3s() {
    use Governance::*;
    let book = [
        Some(Soul),
        Some(Mind),
        Some(Strength),
        Some(Speech),
        Some(KnowledgeAndHappiness),
        Some(Potency),
        Some(Grief),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::governance), book);
}

#[test]
fn graha_colours_are_section_3_2_4s() {
    use Colour::*;
    let book = [
        Some(BloodRed),
        Some(Tawny),
        Some(BloodRed),
        Some(GrassGreen),
        Some(Tawny),
        Some(Variegated),
        Some(Black),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::colour), book);
}

#[test]
fn the_planetary_cabinet_is_section_3_2_5s() {
    use Cabinet::*;
    let book = [
        King, King, Leader, Prince, Minister, Minister, Servant, Army, Army,
    ];
    assert_eq!(GRAHAS.map(Body::cabinet), book.map(Some));
}

#[test]
fn deities_are_section_3_2_6s() {
    use Deity::*;
    let book = [
        Some(Agni),
        Some(Varuna),
        Some(Subrahmanya),
        Some(MahaVishnu),
        Some(Indra),
        Some(SachiDevi),
        Some(Brahma),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::deity), book);
}

#[test]
fn sexes_are_section_3_2_7s_with_mercury_and_saturn_female() {
    use Sex::*;
    let book = [
        Some(Male),
        Some(Female),
        Some(Male),
        Some(Female),
        Some(Male),
        Some(Female),
        Some(Female),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::sex), book);
}

#[test]
fn graha_elements_are_section_3_2_8s() {
    use Element::*;
    let book = [
        Some(Agni),
        Some(Jala),
        Some(Agni),
        Some(Bhoo),
        Some(Aakaasa),
        Some(Jala),
        Some(Vaayu),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::element), book);
}

#[test]
fn graha_varnas_are_section_3_2_9s() {
    use Class::*;
    let book = [
        Some(Kshatriya),
        Some(Vaisya),
        Some(Kshatriya),
        Some(Vaisya),
        Some(Brahmana),
        Some(Brahmana),
        Some(Sudra),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::class), book);
}

#[test]
fn graha_gunas_are_section_3_2_10s() {
    use Quality::*;
    let book = [
        Some(Sattwa),
        Some(Sattwa),
        Some(Tamas),
        Some(Rajas),
        Some(Sattwa),
        Some(Rajas),
        Some(Tamas),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::quality), book);
}

#[test]
fn abodes_are_section_3_2_11s_with_none_for_mars() {
    use Abode::*;
    let book = [
        Some(Temple),
        Some(WateryPlace),
        None,
        Some(SportsGround),
        Some(TreasureHouse),
        Some(Bedroom),
        Some(FilthyArea),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::abode), book);
}

#[test]
fn sapta_dhaatus_are_section_3_2_12s() {
    use SaptaDhaatu::*;
    let book = [
        Some(Bones),
        Some(Blood),
        Some(Marrow),
        Some(Skin),
        Some(Fat),
        Some(Semen),
        Some(Muscles),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::sapta_dhaatu), book);
}

#[test]
fn time_periods_are_section_3_2_13s() {
    use Period::*;
    let book = [
        Some(Ayana),
        Some(Minute),
        Some(Week),
        Some(Ritu),
        Some(Month),
        Some(Fortnight),
        Some(Year),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::period), book);
}

#[test]
fn tastes_are_section_3_2_14s() {
    use Taste::*;
    let book = [
        Some(Pungent),
        Some(Saline),
        Some(Bitter),
        Some(Mixed),
        Some(Sweet),
        Some(Sour),
        Some(Astringent),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::taste), book);
}

#[test]
fn digbalas_are_section_3_2_15s() {
    use Direction::*;
    let book = [
        Some(South),
        Some(North),
        Some(South),
        Some(East),
        Some(East),
        Some(North),
        Some(West),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::digbala), book);
}

#[test]
fn day_and_night_strength_is_section_3_2_15s() {
    use StrongBy::*;
    let book = [
        Some(Divaa),
        Some(Nishaa),
        Some(Nishaa),
        Some(Always),
        Some(Divaa),
        Some(Divaa),
        Some(Nishaa),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::strong_by), book);
}

#[test]
fn paksha_and_ayana_strength_is_section_3_2_15s() {
    // Natural malefics in Krishna paksha and Dakshina ayana, benefics in Sukla and Uttara; Mercury's and the Moon's nature turns (§3.2.2); the seven planets only.
    use Ayana::*;
    use Paksha::*;
    let paksha = [
        Some(Krishna),
        None,
        Some(Krishna),
        None,
        Some(Sukla),
        Some(Sukla),
        Some(Krishna),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::strong_paksha), paksha);
    let ayana = [
        Some(Dakshina),
        None,
        Some(Dakshina),
        None,
        Some(Uttara),
        Some(Uttara),
        Some(Dakshina),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::strong_ayana), ayana);
}

#[test]
fn ritus_are_section_3_2_16s_with_none_for_the_sun() {
    use Ritu::*;
    let book = [
        None,
        Some(Varsha),
        Some(Greeshma),
        Some(Hemanta),
        Some(Seeta),
        Some(Vasanta),
        Some(Sisira),
        None,
        None,
    ];
    assert_eq!(GRAHAS.map(Body::ritu), book);
}

#[test]
fn dhatu_moola_and_jeeva_are_section_3_2_17s() {
    use Realm::*;
    let book = [
        Moola, Dhatu, Dhatu, Jeeva, Jeeva, Moola, Dhatu, Dhatu, Jeeva,
    ];
    assert_eq!(GRAHAS.map(Body::realm), book.map(Some));
}

#[test]
fn the_lagna_has_no_graha_characteristics() {
    let l = Body::Lagna;
    assert_eq!(
        (
            l.avatara(),
            l.disposition(),
            l.governance(),
            l.colour(),
            l.cabinet(),
            l.deity()
        ),
        (None, None, None, None, None, None)
    );
    assert_eq!(
        (
            l.sex(),
            l.element(),
            l.class(),
            l.quality(),
            l.abode(),
            l.sapta_dhaatu()
        ),
        (None, None, None, None, None, None)
    );
    assert_eq!(
        (
            l.period(),
            l.taste(),
            l.digbala(),
            l.strong_by(),
            l.strong_paksha(),
            l.strong_ayana()
        ),
        (None, None, None, None, None, None)
    );
    assert_eq!((l.ritu(), l.realm()), (None, None));
}

#[test]
fn dignities_are_table_6s() {
    use Rasi::*;
    // Table 6 (ch. 3 §3.3) row by row: own rasis, uchcha (deep exaltation), neecha (deep debilitation), moolatrikona.
    let table_6: [(&[Rasi], _, _, _); 9] = [
        (&[Leo], (Aries, Some(10)), (Libra, Some(10)), Leo),
        (&[Cancer], (Taurus, Some(3)), (Scorpio, Some(3)), Taurus),
        (
            &[Aries, Scorpio],
            (Capricorn, Some(28)),
            (Cancer, Some(28)),
            Aries,
        ),
        (
            &[Gemini, Virgo],
            (Virgo, Some(15)),
            (Pisces, Some(15)),
            Virgo,
        ),
        (
            &[Sagittarius, Pisces],
            (Cancer, Some(5)),
            (Capricorn, Some(5)),
            Sagittarius,
        ),
        (
            &[Taurus, Libra],
            (Pisces, Some(27)),
            (Virgo, Some(27)),
            Libra,
        ),
        (
            &[Capricorn, Aquarius],
            (Libra, Some(20)),
            (Aries, Some(20)),
            Aquarius,
        ),
        (&[Aquarius], (Gemini, None), (Sagittarius, None), Virgo),
        (&[Scorpio], (Sagittarius, None), (Gemini, None), Pisces),
    ];
    for (graha, (own, uchcha, neecha, moolatrikona)) in GRAHAS.into_iter().zip(table_6) {
        let d = graha.dignities().unwrap();
        assert_eq!(
            (d.own, d.uchcha, d.neecha, d.moolatrikona.0),
            (own, uchcha, neecha, moolatrikona),
            "{graha:?}"
        );
    }
    assert_eq!(Body::Lagna.dignities(), None);
}

#[test]
fn every_rasis_lord_owns_it_in_table_6() {
    for r in RASIS {
        assert!(r.lord().dignities().unwrap().own.contains(&r), "{r:?}");
    }
    assert_eq!(Rasi::Scorpio.lord(), Body::Mars);
    assert_eq!(Rasi::Aquarius.lord(), Body::Saturn);
}

#[test]
fn of_the_two_rasis_a_graha_owns_one_is_divaa_and_one_nishaa() {
    // Ch. 2 §2.2.10.
    for graha in GRAHAS {
        if let [a, b] = graha.dignities().unwrap().own {
            assert_ne!(a.strong_by(), b.strong_by(), "{graha:?}");
        }
    }
}

#[test]
fn degree_splits_are_section_3_3s_at_their_boundaries() {
    use Body::*;
    use Dignity::*;
    // (graha, rasi, degree of the boundary, dignity just below it, dignity from it), §3.3 notes 1–7.
    for (graha, r, at, below, from) in [
        (Moon, 1, 3, Uchcha, Moolatrikona),
        (Mercury, 5, 15, Uchcha, Moolatrikona),
        (Mercury, 5, 20, Moolatrikona, OwnRasi),
        (Sun, 4, 20, Moolatrikona, OwnRasi),
        (Mars, 0, 12, Moolatrikona, OwnRasi),
        (Jupiter, 8, 10, Moolatrikona, OwnRasi),
        (Venus, 6, 15, Moolatrikona, OwnRasi),
        (Saturn, 10, 20, Moolatrikona, OwnRasi),
    ] {
        let at = dms(r, at, 0, 0.0);
        assert_eq!(
            dignity(graha, at - 1.0 / 60.0),
            Some(below),
            "{graha:?} {at}"
        );
        assert_eq!(dignity(graha, at), Some(from), "{graha:?} {at}");
        assert_eq!(
            dignity(graha, at + 1.0 / 60.0),
            Some(from),
            "{graha:?} {at}"
        );
    }
    // Each rasi's first and last arcsecond.
    assert_eq!(dignity(Moon, dms(1, 0, 0, 0.0)), Some(Uchcha));
    assert_eq!(dignity(Moon, dms(1, 2, 59, 59.0)), Some(Uchcha));
    assert_eq!(dignity(Moon, dms(1, 29, 59, 59.0)), Some(Moolatrikona));
    assert_eq!(dignity(Mercury, dms(5, 29, 59, 59.0)), Some(OwnRasi));
    assert_eq!(dignity(Mars, dms(0, 0, 0, 0.0)), Some(Moolatrikona));
    assert_eq!(dignity(Mars, dms(7, 0, 0, 0.0)), Some(OwnRasi));
    assert_eq!(dignity(Saturn, dms(0, 22, 41, 0.0)), Some(Neecha));
    assert_eq!(dignity(Saturn, dms(5, 0, 0, 0.0)), None);
    assert_eq!(dignity(Rahu, dms(5, 29, 59, 59.0)), Some(Moolatrikona));
    assert_eq!(dignity(Ketu, dms(2, 0, 0, 0.0)), Some(Neecha));
    assert_eq!(dignity(Lagna, 0.0), None);
}

#[test]
fn natural_relationships_are_table_7s() {
    use Body::*;
    // Table 7 (ch. 3 §3.4.1): each planet's friends, neutrals, enemies.
    let table_7 = [
        (
            Sun,
            vec![Moon, Mars, Jupiter],
            vec![Mercury],
            vec![Venus, Saturn],
        ),
        (
            Moon,
            vec![Sun, Mercury],
            vec![Mars, Jupiter, Venus, Saturn],
            vec![],
        ),
        (
            Mars,
            vec![Sun, Moon, Jupiter],
            vec![Venus, Saturn],
            vec![Mercury],
        ),
        (
            Mercury,
            vec![Sun, Venus],
            vec![Mars, Jupiter, Saturn],
            vec![Moon],
        ),
        (
            Jupiter,
            vec![Sun, Moon, Mars],
            vec![Saturn],
            vec![Mercury, Venus],
        ),
        (
            Venus,
            vec![Mercury, Saturn],
            vec![Mars, Jupiter],
            vec![Sun, Moon],
        ),
        (
            Saturn,
            vec![Mercury, Venus],
            vec![Jupiter],
            vec![Sun, Moon, Mars],
        ),
    ];
    for (graha, friends, neutrals, enemies) in table_7 {
        let with = |value| -> Vec<Body> {
            GRAHAS
                .into_iter()
                .filter(|&other| natural(graha, other) == Some(value))
                .collect()
        };
        assert_eq!(with(Natural::Mitra), friends, "{graha:?}");
        assert_eq!(with(Natural::Sama), neutrals, "{graha:?}");
        assert_eq!(with(Natural::Satru), enemies, "{graha:?}");
    }
    assert_eq!(natural(Sun, Sun), None);
    assert_eq!(natural(Sun, Rahu), None);
    assert_eq!(natural(Ketu, Moon), None);
    assert_eq!(natural(Lagna, Moon), None);
}

#[test]
fn tatkaala_friends_are_in_the_2nd_3rd_4th_10th_11th_and_12th() {
    let friends: Vec<u8> = RASIS
        .into_iter()
        .filter(|&r| tatkaala(Rasi::Cancer, r) == Tatkaala::Mitra)
        .map(|r| bhava(Rasi::Cancer, r))
        .collect();
    assert_eq!(friends, [10, 11, 12, 2, 3, 4]);
}

#[test]
fn compound_relationships_are_table_8s() {
    use Compound::*;
    // Table 8 (ch. 3 §3.4.3): rows natural friend, neutral, enemy; columns temporary friend, enemy.
    let table_8 = [[Adhimitra, Sama], [Mitra, Satru], [Sama, Adhisatru]];
    let got = [Natural::Mitra, Natural::Sama, Natural::Satru]
        .map(|n| [Tatkaala::Mitra, Tatkaala::Satru].map(|t| compound(n, t)));
    assert_eq!(got, table_8);
}

#[test]
fn saturn_in_aries_is_neecha_and_sama_with_mars_in_taurus() {
    // Mars in Taurus is Saturn's tatkaala friend, a natural enemy: sama (Table 8); neecha all the same (§3.3).
    let mut g = [[dms(4, 1, 0, 0.0), 1.0]; 8];
    g[2] = [dms(1, 5, 0, 0.0), 1.0];
    g[6] = [dms(0, 22, 41, 0.0), 1.0];
    let chart = chart(&sky(dms(4, 0, 0, 0.0), g));
    assert_eq!(
        dignity(Body::Saturn, chart.entry(Body::Saturn).longitude),
        Some(Dignity::Neecha)
    );
    assert_eq!(chart.lord_relationship(Body::Saturn), Some(Compound::Sama));
    // The rest are in Leo: the Sun in his own rasi has no relationship, the others have the Sun's.
    assert_eq!(chart.lord_relationship(Body::Sun), None);
    assert_eq!(chart.lord_relationship(Body::Moon), Some(Compound::Sama));
    assert_eq!(chart.lord_relationship(Body::Lagna), None);
    assert_eq!(chart.lord_relationship(Body::Rahu), None);
    assert_eq!(chart.lord_relationship(Body::Ketu), None);
    // Nor in the nodes' own rasis, whose lords are Saturn and Mars; Ketu opposite Rahu, so one at a time.
    for (rahu, node) in [(10, Body::Rahu), (1, Body::Ketu)] {
        let mut g = [[dms(4, 1, 0, 0.0), 1.0]; 8];
        g[7] = [dms(rahu, 5, 0, 0.0), -0.05];
        let chart = super::chart(&sky(0.0, g));
        assert_eq!(
            dignity(node, chart.entry(node).longitude),
            Some(Dignity::OwnRasi)
        );
        assert_eq!(chart.lord_relationship(node), None);
    }
}

#[test]
fn mercurys_nature_turns_on_his_company_in_the_chart_read() {
    use Body::*;
    use Disposition::*;
    // §3.2.2; a tie has no rule (ADR 0014).
    for (company, nature) in [
        (&[][..], Some(Saumya)),
        (&[Jupiter], Some(Saumya)),
        (&[Sun, Mars], Some(Kroora)),
        (&[Jupiter, Venus, Sun], Some(Saumya)),
        (&[Sun, Mars, Jupiter], Some(Kroora)),
        (&[Sun, Jupiter], None),
        // The nodes are malefics in his company.
        (&[Rahu], Some(Kroora)),
        (&[Ketu, Venus], None),
    ] {
        let chart = with_mercury(company);
        assert_eq!(
            chart.in_varga(Varga::RasiChart).natural_nature(Mercury),
            nature,
            "{company:?}"
        );
    }
}

#[test]
fn mercurys_company_is_counted_in_the_varga_read() {
    use Body::*;
    // Together at Aries 1° and 5° in the rasi chart, apart in D-9: Aries and Taurus.
    let chart = scene(
        Rasi::Aquarius,
        &[(Mercury, dms(0, 5, 0, 0.0)), (Mars, dms(0, 1, 0, 0.0))],
    );
    assert_eq!(
        chart.in_varga(Varga::RasiChart).natural_nature(Mercury),
        Some(Disposition::Kroora)
    );
    assert_eq!(
        chart.in_varga(Varga::Navamsa).natural_nature(Mercury),
        Some(Disposition::Saumya)
    );
}

#[test]
fn the_moon_is_saumya_in_sukla_and_kroora_in_krishna_and_counts_so_beside_mercury() {
    use Body::*;
    use Disposition::*;
    // The Moon 30° ahead of the Sun is waxing, 30° behind waning.
    let beside_mercury = |sun| {
        scene(
            Rasi::Aquarius,
            &[
                (Sun, mid(sun)),
                (Moon, mid(Rasi::Taurus)),
                (Mercury, mid(Rasi::Taurus)),
            ],
        )
    };
    let waxing = beside_mercury(Rasi::Aries);
    assert_eq!(
        waxing.in_varga(Varga::RasiChart).natural_nature(Moon),
        Some(Saumya)
    );
    assert_eq!(
        waxing.in_varga(Varga::RasiChart).natural_nature(Mercury),
        Some(Saumya)
    );
    let waning = beside_mercury(Rasi::Gemini);
    assert_eq!(
        waning.in_varga(Varga::RasiChart).natural_nature(Moon),
        Some(Kroora)
    );
    assert_eq!(
        waning.in_varga(Varga::RasiChart).natural_nature(Mercury),
        Some(Kroora)
    );
}

#[test]
fn the_other_natures_are_fixed_and_a_lagna_has_none() {
    use Body::*;
    use Disposition::*;
    let chart = with_mercury(&[]);
    let nature = |g| chart.in_varga(Varga::RasiChart).natural_nature(g);
    assert_eq!([Jupiter, Venus].map(nature), [Some(Saumya), Some(Saumya)]);
    assert_eq!(
        [Sun, Mars, Saturn, Rahu, Ketu].map(nature),
        [Some(Kroora); 5]
    );
    assert_eq!(nature(Lagna), None);
}

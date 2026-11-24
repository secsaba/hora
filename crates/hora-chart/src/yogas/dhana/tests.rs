use super::*;
use crate::tests::*;

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

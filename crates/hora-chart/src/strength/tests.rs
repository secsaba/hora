use super::*;
use crate::tests::*;

fn lord_of(chart: &Chart, r: Rasi) -> Body {
    chart.in_varga(Varga::RasiChart).stronger_lord(r)
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

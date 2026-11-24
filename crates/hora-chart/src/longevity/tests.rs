use super::*;
use crate::tests::*;

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

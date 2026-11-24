use super::*;
use crate::tests::*;

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

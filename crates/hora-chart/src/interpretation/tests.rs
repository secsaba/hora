use super::*;
use crate::tests::*;

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

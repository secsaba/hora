use super::*;
use crate::tests::*;

#[test]
fn a_bhava_arudha_follows_the_example_of_section_9_2() {
    // A Gemini bhava with Mercury in Aquarius: 9 rasis, then 9 on, to Libra.
    let chart = scene(Rasi::Gemini, &[(Body::Mercury, mid(Rasi::Aquarius))]);
    assert_eq!(
        chart.in_varga(Varga::RasiChart).bhava_arudhas()[0],
        Rasi::Libra
    );
}

#[test]
fn a_graha_arudha_follows_the_example_of_section_9_5() {
    // The Sun in Gemini: 3 rasis to Leo, then 3 on, to Libra.
    let chart = scene(Rasi::Gemini, &[]);
    assert_eq!(
        chart.in_varga(Varga::RasiChart).graha_arudhas()[0],
        (Body::Sun, Rasi::Libra)
    );
}

/// Rahu's and Ketu's arudha padas with Rahu in `rahu` and Ketu in `ketu`.
fn node_arudha_padas(rahu: Rasi, ketu: Rasi) -> (Rasi, Rasi) {
    let at = [(Body::Rahu, mid(rahu)), (Body::Ketu, mid(ketu))];
    let [.., (_, rahu), (_, ketu)] = scene(Rasi::Aries, &at)
        .in_varga(Varga::RasiChart)
        .graha_arudhas();
    (rahu, ketu)
}

#[test]
fn ketu_7th_from_rahu_shares_rahus_arudha_pada() {
    // §9.5's note.
    for rahu in RASIS {
        let (a, b) = node_arudha_padas(rahu, kth(rahu, 7));
        assert_eq!(a, b, "{rahu:?}");
    }
}

#[test]
fn the_nodes_together_have_opposite_arudha_padas() {
    // §9.5's note.
    for rasi in RASIS {
        let (a, b) = node_arudha_padas(rasi, rasi);
        assert_eq!(bhava(a, b), 7, "{rasi:?}");
    }
}

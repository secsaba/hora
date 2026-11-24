use super::*;
use crate::tests::*;

#[test]
fn each_av_table_has_its_classical_total_of_rekhas() {
    // Ch. 12 Tables 19–26: 48, 49, 39, 54, 56, 52 and 39 for Sun to Saturn, 49 for the lagna; any chart sums to them.
    let chart = placed(dms(3, 15, 0, 0.0), [dms(0, 15, 0, 0.0); 8]);
    let totals = AV_REFERENCES.map(|of| {
        let bav = chart.in_varga(Varga::RasiChart).bav(of);
        bav.iter().map(|&c| u32::from(c)).sum::<u32>()
    });
    assert_eq!(totals, [48, 49, 39, 54, 56, 52, 39, 49]);
}

#[test]
fn the_lagnas_bav_counts_table_26_from_each_reference() {
    // Ch. 12 Table 26, by hand: with the seven in Ar, each house's row sum lands on the rasi it counts to, the lagna's own column
    // left out; the lagna in Cn adds its 3rd, 6th, 10th and 11th from there, Vi, Sg, Ar and Ta.
    let chart = placed(dms(3, 15, 0, 0.0), [dms(0, 15, 0, 0.0); 8]);
    assert_eq!(
        chart.in_varga(Varga::RasiChart).bav(Body::Lagna),
        [6, 4, 5, 5, 2, 7, 1, 2, 3, 6, 6, 2]
    );
}

#[test]
fn bands_turn_at_the_counts_adr_0015_gives() {
    use BavBand::*;
    use SavBand::*;
    let bav = |count| bav_band(Body::Jupiter, count);
    assert_eq!(
        [0, 3, 4, 5, 8].map(bav),
        [Unfavourable, Unfavourable, Neutral, Favourable, Favourable].map(Some)
    );
    assert_eq!(bav_band(Body::Lagna, 8), None);
    assert_eq!(
        [0, 24, 25, 29, 30, 56].map(sav_band),
        [Weak, Weak, Average, Average, Strong, Strong]
    );
}

#[test]
fn example_42_ekaadhipatya_sodhana_on_taurus_and_libra() {
    // Ch. 12 Example 42, pp. 158–159: Ta and Li after trikona sodhana, and which of them is occupied.
    let (ta, li) = (Rasi::Taurus as usize, Rasi::Libra as usize);
    let case = |[vt, vl]: [u8; 2], [ot, ol]: [bool; 2]| {
        let (mut values, mut occupied) = ([0; 12], [false; 12]);
        [values[ta], values[li]] = [vt, vl];
        [occupied[ta], occupied[li]] = [ot, ol];
        let out = ekaadhipatya_sodhana(values, occupied);
        [out[ta], out[li]]
    };
    assert_eq!(case([4, 2], [true, true]), [4, 2], "(a), rule (2)");
    assert_eq!(case([4, 2], [true, false]), [4, 0], "(b), rule (3a)");
    assert_eq!(case([4, 2], [false, true]), [2, 2], "(c), rule (3b)");
    assert_eq!(case([4, 2], [false, false]), [2, 2], "(d), rule (4b)");
    assert_eq!(case([2, 2], [false, false]), [0, 0], "(e), rule (4a)");
    assert_eq!(case([4, 0], [false, false]), [4, 0], "rule (1)");
}

#[test]
fn the_knowledge_bases_worked_case_reduces_and_sums_as_its_page_says() {
    use Rasi::*;
    // Pins kb/{en,hu}/sodhya-pinda.md's worked case; only the lagna occupies Sc, and Rahu alone in Pi does not.
    let bav = [4, 3, 2, 4, 2, 4, 0, 6, 3, 4, 2, 5];
    assert_eq!(bav.iter().sum::<u8>(), 39);
    let trikona = trikona_sodhana(bav);
    assert_eq!(trikona, [2, 0, 2, 0, 0, 1, 0, 2, 1, 1, 2, 1]);
    let at = [Gemini, Taurus, Capricorn, Gemini, Virgo, Cancer, Virgo];
    let occupied = RASIS.map(|r| r == Scorpio || at.contains(&r));
    let soav = ekaadhipatya_sodhana(trikona, occupied);
    assert_eq!(soav, [0, 0, 2, 0, 0, 1, 0, 2, 0, 1, 1, 0]);
    let p = pindas(soav, at);
    assert_eq!([p.rasi, p.graha, p.sodhya()], [53, 43, 96]);
}

#[test]
fn the_lagna_alone_occupies_a_rasi_and_rahu_or_ketu_alone_does_not() {
    use Rasi::*;
    // ADR 0015. The seven in Ar, the lagna alone in Aq: Mars's BAV after trikona sodhana has 3 in Cp and 6 in Aq, so the
    // empty Cp, lower, becomes 0 (rule (3a)); were the lagna not counted, both empty would give 3 and 3 (rule (4b)).
    let mut g = [dms(0, 15, 0, 0.0); 8];
    let lagna_in_aq = placed(dms(Aquarius as u8, 15, 0, 0.0), g);
    let v = lagna_in_aq.in_varga(Varga::RasiChart);
    let t = trikona_sodhana(v.bav(Body::Mars));
    assert_eq!([t[Capricorn as usize], t[Aquarius as usize]], [3, 6]);
    let soav = v.soav(Body::Mars);
    assert_eq!([soav[Capricorn as usize], soav[Aquarius as usize]], [0, 6]);
    // Rahu alone in Cp (Ketu in Cn), then Ketu alone in Cp (Rahu in Cn): were Cp occupied, rule (2) would keep its 3.
    for rahu in [Capricorn, Cancer] {
        g[Body::Rahu as usize - 1] = dms(rahu as u8, 15, 0, 0.0);
        let node_in_cp = placed(dms(Aquarius as u8, 15, 0, 0.0), g);
        assert_eq!(
            node_in_cp.in_varga(Varga::RasiChart).soav(Body::Mars),
            soav,
            "Rahu in {rahu:?}"
        );
    }
}

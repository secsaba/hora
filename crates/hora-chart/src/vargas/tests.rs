use super::*;
use crate::tests::*;

#[test]
fn movable_and_earthy_rasis_start_their_count_where_6_2_says() {
    use Rasi::*;
    use Varga::*;
    // No example of §6.2 has a body in a movable or earthy rasi; the 2nd part of Cancer and of Taurus.
    let in_cancer = [Ashtamsa, Shodasamsa, Vimsamsa, Akshavedamsa]
        .map(|v| varga(v, dms(3, 0, 0, 0.0) + RASI_SPAN / f64::from(v.parts())));
    assert_eq!(in_cancer, [Taurus, Taurus, Taurus, Taurus]);
    let in_taurus = [Navamsa, Nakshatramsa]
        .map(|v| varga(v, dms(1, 0, 0, 0.0) + RASI_SPAN / f64::from(v.parts())));
    assert_eq!(in_taurus, [Aquarius, Leo]);
}

#[test]
fn the_rasi_chart_is_the_rasi() {
    for r in 0..12 {
        for d in [0.0, 15.0, 29.999] {
            let longitude = f64::from(r) * RASI_SPAN + d;
            assert_eq!(varga(Varga::RasiChart, longitude), rasi(longitude).0);
        }
    }
}

#[test]
fn a_part_starts_at_its_boundary_to_the_whole_arcsecond() {
    use Rasi::*;
    let drekkana = |d, m, s| varga(Varga::Drekkana, dms(0, d, m, s));
    assert_eq!([drekkana(9, 59, 59.9), drekkana(10, 0, 0.0)], [Aries, Leo]);
    // D-30's marks, just before and at each, in Aries (odd) and Taurus (even).
    let trimsamsa = |rasi, d, s| varga(Varga::Trimsamsa, dms(rasi, d, 0, 0.0) - s / 3600.0);
    let odd = [5, 10, 18, 25].map(|d| [trimsamsa(0, d, 0.1), trimsamsa(0, d, 0.0)]);
    assert_eq!(
        odd,
        [
            [Aries, Aquarius],
            [Aquarius, Sagittarius],
            [Sagittarius, Gemini],
            [Gemini, Libra]
        ]
    );
    let even = [5, 12, 20, 25].map(|d| [trimsamsa(1, d, 0.1), trimsamsa(1, d, 0.0)]);
    assert_eq!(
        even,
        [
            [Taurus, Virgo],
            [Virgo, Pisces],
            [Pisces, Capricorn],
            [Capricorn, Scorpio]
        ]
    );
    assert_eq!(
        [trimsamsa(1, 30, 0.1), trimsamsa(0, 0, 0.0)],
        [Scorpio, Aries]
    );
}

#[test]
fn hora_and_panchamsa_differ_for_odd_and_even_rasis() {
    use Rasi::*;
    let hora =
        [(0, 14), (0, 15), (1, 14), (1, 15)].map(|(r, d)| varga(Varga::Hora, dms(r, d, 0, 0.0)));
    assert_eq!(hora, [Leo, Cancer, Cancer, Leo]);
    let panchamsa = |r| [0, 6, 12, 18, 24].map(|d| varga(Varga::Panchamsa, dms(r, d, 0, 0.0)));
    assert_eq!(panchamsa(0), [Aries, Aquarius, Sagittarius, Gemini, Libra]);
    assert_eq!(panchamsa(1), [Taurus, Virgo, Pisces, Capricorn, Scorpio]);
}

#[test]
fn each_varga_group_names_an_amsa_from_a_count_of_two_up_to_all_its_vargas() {
    use Amsa::*;
    use VargaGroup::*;
    let amsas = |group: VargaGroup| (0..=17).map(|count| group.amsa(count)).collect::<Vec<_>>();
    let named = |names: &[Amsa]| {
        [None, None]
            .into_iter()
            .chain(names.iter().copied().map(Some))
            .chain([None].repeat(16 - names.len()))
            .collect::<Vec<_>>()
    };
    // §6.6.1–6.6.4.
    let sapta = [
        Kimsukaamsa,
        Vyanjanaamsa,
        Chaamaraamsa,
        Chatraamsa,
        Kundalaamsa,
        Mukutaamsa,
    ];
    assert_eq!(amsas(Shadvarga), named(&sapta[..5]));
    assert_eq!(amsas(SaptaVarga), named(&sapta));
    assert_eq!(
        amsas(DasaVarga),
        named(&[
            Paarijaataamsa,
            Uttamaamsa,
            Gopuraamsa,
            Simhaasanaamsa,
            Paaraavataamsa,
            Devalokaamsa,
            Brahmalokamsa,
            Airaavataamsa,
            Sreedhaamaamsa,
        ])
    );
    assert_eq!(
        amsas(ShodasaVarga),
        named(&[
            Bhedakaamsa,
            Kusumaamsa,
            Nagapurushaamsa,
            Kandukaamsa,
            Keralaamsa,
            Kalpavrikshaamsa,
            Chandanavanaamsa,
            Poornachandraamsa,
            Uchchaisravaamsa,
            Dhanvantaryamsa,
            Sooryakaantaamsa,
            Vidrumaamsa,
            Indraasanaamsa,
            Golokaamsa,
            SreeVallabhaamsa,
        ])
    );
    assert_eq!(VARGA_GROUPS.map(|g| g.vargas().len()), [6, 7, 10, 16]);
}

#[test]
fn the_lagna_has_no_amsabala() {
    assert_eq!(
        amsabala(Body::Lagna, 255.0, VargaGroup::ShodasaVarga),
        (0, None)
    );
}

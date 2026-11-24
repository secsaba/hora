use super::*;
use crate::tests::*;

#[test]
fn the_sun_based_upagrahas_are_sun_baseds_five() {
    let five: Vec<_> = UPAGRAHAS.into_iter().filter(|u| u.is_sun_based()).collect();
    assert_eq!(five, sun_based(0.0).map(|(u, _)| u));
}

#[test]
fn upaketu_is_the_sun_less_30_and_the_pairs_are_opposite() {
    // Ch. 4 §4.2: Table 9's Upaketu row, and the note under it.
    let apart = |a: f64, b: f64| (a - b).rem_euclid(360.0);
    let close = |a: f64, b: f64| {
        let d = apart(a, b);
        d.min(360.0 - d) < 1e-9
    };
    for tenth in 0..3600 {
        let sun = f64::from(tenth) / 10.0;
        let [dhuma, vyatipaata, parivesha, indrachaapa, upaketu] = sun_based(sun).map(|(_, l)| {
            assert!((0.0..360.0).contains(&l), "{sun}: {l}");
            l
        });
        assert!(close(upaketu, sun - 30.0), "{sun}: {upaketu}");
        assert!(close(apart(dhuma, indrachaapa), 180.0), "{sun}");
        assert!(close(apart(vyatipaata, parivesha), 180.0), "{sun}");
    }
}

#[test]
fn part_rulers_are_table_10s() {
    // Ch. 4 §4.3, Table 10: the day's rows, then the night's, Sunday to Saturday; `—` is lord-less.
    use Body::*;
    let (su, mo, ma, me, ju, ve, sa) = (
        Some(Sun),
        Some(Moon),
        Some(Mars),
        Some(Mercury),
        Some(Jupiter),
        Some(Venus),
        Some(Saturn),
    );
    let day = [
        [su, mo, ma, me, ju, ve, sa, None],
        [mo, ma, me, ju, ve, sa, None, su],
        [ma, me, ju, ve, sa, None, su, mo],
        [me, ju, ve, sa, None, su, mo, ma],
        [ju, ve, sa, None, su, mo, ma, me],
        [ve, sa, None, su, mo, ma, me, ju],
        [sa, None, su, mo, ma, me, ju, ve],
    ];
    let night = [
        [ju, ve, sa, None, su, mo, ma, me],
        [ve, sa, None, su, mo, ma, me, ju],
        [sa, None, su, mo, ma, me, ju, ve],
        [su, mo, ma, me, ju, ve, sa, None],
        [mo, ma, me, ju, ve, sa, None, su],
        [ma, me, ju, ve, sa, None, su, mo],
        [me, ju, ve, sa, None, su, mo, ma],
    ];
    assert_eq!(VAARAS.map(|v| part_rulers(v, false)), day);
    assert_eq!(VAARAS.map(|v| part_rulers(v, true)), night);
}

/// The time-based upagrahas of `sky`, in the book's order.
fn time_based_of(sky: &Sky) -> Vec<(Upagraha, f64)> {
    upagrahas(sky).split_off(5)
}

#[test]
fn a_thursday_night_from_18_00_to_06_00_has_yamaghantaka_at_23_15() {
    // Ch. 4 §4.3's worked case: Jupiter rules the 4th part, 22:30 to 24:00; a part lagna here is its moment's hour.
    let sky = timed_sky(Vaara::Thursday, true, |k, middle| {
        18.0 + 1.5 * (k as f64 + if middle { 0.5 } else { 0.0 })
    });
    assert_eq!(part_rulers(Vaara::Thursday, true)[3], Some(Body::Jupiter));
    assert!(time_based_of(&sky).contains(&(Upagraha::Yamaghantaka, 23.25)));
}

#[test]
fn each_time_based_upagraha_rises_at_the_start_or_middle_of_its_part() {
    // A Thursday's day: Jup, Ven, Sat, —, Sun, Moon, Mars, Merc; a part lagna here is 10 × its part, + 5 at its middle.
    let sky = timed_sky(Vaara::Thursday, false, |k, middle| {
        10.0 * (k + 1) as f64 + if middle { 5.0 } else { 0.0 }
    });
    use Upagraha::*;
    assert_eq!(
        time_based_of(&sky),
        [
            (Kaala, 55.0),
            (Mrityu, 75.0),
            (Arthaprahaara, 85.0),
            (Yamaghantaka, 15.0),
            (Gulika, 30.0),
            (Maandi, 35.0),
        ]
    );
}

#[test]
fn a_sky_without_a_vedic_day_has_only_the_sun_based_upagrahas() {
    let sky = sky(0.0, [[0.0, 1.0]; 8]);
    assert_eq!(upagrahas(&sky), sun_based(0.0));
}

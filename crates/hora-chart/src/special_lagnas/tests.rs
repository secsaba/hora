use super::*;
use crate::tests::*;

#[test]
fn a_polar_sky_without_a_vedic_day_has_only_the_sree_lagna() {
    let sky = sky(dms(5, 25, 5, 0.0), [[dms(6, 13, 6, 0.0), 13.0]; 8]);
    let got = special_lagnas(&sky);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].0, SpecialLagna::SreeLagna);
    assert!((got[0].1 - dms(11, 18, 47, 0.0)).abs() < 1e-9, "{got:?}");
}

#[test]
fn special_lagnas_count_the_exact_time_since_sunrise_not_the_days_length() {
    // A 23-hour Vedic day, born 90 minutes and 1 second after sunrise.
    let sunrise = WEDNESDAY + hours(6, 0);
    let mut day = day(sunrise, sunrise + 23.0 / 24.0);
    day.sun_at_sunrise = dms(9, 24, 17, 0.0);
    let sky = Sky {
        jd: sunrise + hours(1, 30) + 1.0 / 86400.0,
        vedic_day: Some(day),
        ..sky(0.0, [[0.0, 1.0]; 8])
    };
    let got = special_lagnas(&sky);
    let want = [
        (SpecialLagna::HoraLagna, dms(11, 9, 17, 30.0)),
        (SpecialLagna::GhatiLagna, dms(1, 16, 48, 15.0)),
        (SpecialLagna::SreeLagna, 0.0),
        (SpecialLagna::BhaavaLagna, dms(10, 16, 47, 15.0)),
    ];
    assert_eq!(got.iter().map(|g| g.0).collect::<Vec<_>>(), SPECIAL_LAGNAS);
    for ((lagna, got), (_, want)) in got.into_iter().zip(want) {
        // A Julian day near 2.45e6 holds about 40 µs, which GL turns into about 0.01″.
        assert!(
            (got - want).abs() * 3600.0 < 0.05,
            "{lagna:?}: {got} {want}"
        );
    }
}

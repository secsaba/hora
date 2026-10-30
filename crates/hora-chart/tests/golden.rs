//! Facts the book shows for its example charts, derived from hora-swe's positions, and its examples and exercises on given positions (chapters 1 and 3):
//! P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*.
//!
//! The book prints birth times to the minute but cast its charts with seconds; where a chart's seconds are estimated
//! (in `book/mod.rs`), they stay within the printed minute and keep every printed value within 60″.

use hora_chart::{
    Body, Compound, LunarMonth, Nakshatra, PLANETS, Paksha, Rasi, SpecialLagna, SunMoonYoga,
    Tatkaala, Tithi, Upagraha, VARGAS, Vaara, Varga, bhaava_lagna, bhava, chart, compound,
    ghati_lagna, hora, hora_lagna, lunar_month, nakshatra, natural, rasi, special_lagnas,
    sree_lagna, sun_moon_yoga, tatkaala, tithi, upagrahas, vaara, varga,
};
use hora_swe::{Sky, VedicDay};

// Each crate pins only some of the charts.
#[allow(dead_code)]
#[path = "../../hora-swe/tests/book/mod.rs"]
mod book;

/// The grahas the chart marks (R) of those the book marks: it prints no R on the nodes (chart 4, p. 98), which hora marks too (#43).
fn marked_retrograde(sky: &Sky) -> Vec<Body> {
    chart(sky)
        .iter()
        .filter(|e| e.retrograde && !matches!(e.body, Body::Rahu | Body::Ketu))
        .map(|e| e.body)
        .collect()
}

/// The nakshatra of `body` in the chart of `sky`.
fn nakshatra_of(sky: &Sky, body: Body) -> (Nakshatra, u8) {
    let entry = chart(sky).into_iter().find(|e| e.body == body).unwrap();
    nakshatra(entry.longitude)
}

/// The rasi `upagraha` occupies in the chart of `sky`.
fn upagraha_rasi(sky: &Sky, upagraha: Upagraha) -> Rasi {
    let (_, longitude) = upagrahas(sky)
        .into_iter()
        .find(|(u, _)| *u == upagraha)
        .unwrap();
    rasi(longitude).0
}

/// Exercise 2 and its answer: ch. 1 §1.3.3 and §1.6.
#[test]
fn exercise_2_counts_bhavas_from_the_lagna_then_from_the_moon() {
    use Rasi::*;
    let (lagna, sun, moon, mars) = (Cancer, Aries, Taurus, Capricorn);
    assert_eq!([sun, moon, mars].map(|r| bhava(lagna, r)), [10, 11, 7]);
    assert_eq!([sun, moon, mars].map(|r| bhava(moon, r)), [12, 1, 9]);
}

/// The seven planets' rasis in Lord Sree Rama's chart: ch. 1 Example 1, Figure 1.
fn rama(graha: Body) -> Rasi {
    use Body::*;
    use Rasi::*;
    match graha {
        Sun => Aries,
        Mercury => Taurus,
        Moon | Jupiter => Cancer,
        Saturn => Libra,
        Mars => Capricorn,
        Venus => Pisces,
        _ => unreachable!("{graha:?}"),
    }
}

/// `graha`'s tatkaala friends and enemies among the seven planets in Rama's chart, in the book's graha order.
fn rama_tatkaala(graha: Body) -> (Vec<Body>, Vec<Body>) {
    PLANETS
        .into_iter()
        .filter(|&other| other != graha)
        .partition(|&other| tatkaala(rama(graha), rama(other)) == Tatkaala::Mitra)
}

/// `graha`'s compound relationship with `other` in Rama's chart.
fn rama_compound(graha: Body, other: Body) -> Compound {
    compound(
        natural(graha, other).unwrap(),
        tatkaala(rama(graha), rama(other)),
    )
}

/// Example 4 and Exercise 5's answer: ch. 3 §3.4.2 and §3.5.
#[test]
fn ramas_chart_has_the_tatkaala_friends_and_enemies_of_example_4_and_exercise_5() {
    use Body::*;
    let book = [
        (Sun, vec![Moon, Mars, Mercury, Jupiter, Venus], vec![Saturn]),
        (Moon, vec![Sun, Mercury, Saturn], vec![Mars, Jupiter, Venus]),
        (Jupiter, vec![Sun, Mercury, Saturn], vec![Moon, Mars, Venus]),
        (Venus, vec![Sun, Mars, Mercury], vec![Moon, Jupiter, Saturn]),
    ];
    for (graha, friends, enemies) in book {
        assert_eq!(rama_tatkaala(graha), (friends, enemies), "{graha:?}");
    }
}

/// Example 5 and Exercise 6's answer: ch. 3 §3.4.3 and §3.5.
#[test]
fn ramas_chart_has_the_compound_relationships_of_example_5_and_exercise_6() {
    use Body::*;
    use Compound::*;
    let book = [
        (
            Sun,
            [
                (Moon, Adhimitra),
                (Mars, Adhimitra),
                (Jupiter, Adhimitra),
                (Mercury, Mitra),
                (Venus, Sama),
                (Saturn, Adhisatru),
            ],
        ),
        (
            Moon,
            [
                (Sun, Adhimitra),
                (Mercury, Adhimitra),
                (Saturn, Mitra),
                (Mars, Satru),
                (Jupiter, Satru),
                (Venus, Satru),
            ],
        ),
        // Erratum: §3.5 calls Venus Jupiter's natural neutral and so his satru; Table 7 makes her his enemy, so adhisatru.
        // (Jupiter is Venus's natural neutral, and her satru, as §3.5 prints below.)
        (
            Jupiter,
            [
                (Sun, Adhimitra),
                (Saturn, Mitra),
                (Mercury, Sama),
                (Moon, Sama),
                (Mars, Sama),
                (Venus, Adhisatru),
            ],
        ),
        (
            Venus,
            [
                (Mercury, Adhimitra),
                (Mars, Mitra),
                (Sun, Sama),
                (Saturn, Sama),
                (Jupiter, Satru),
                (Moon, Adhisatru),
            ],
        ),
    ];
    for (graha, relationships) in book {
        for (other, want) in relationships {
            assert_eq!(rama_compound(graha, other), want, "{graha:?} to {other:?}");
        }
    }
}

/// A longitude from a rasi (0 = Aries) and the degrees and arcminutes the book prints.
fn dm(rasi: u8, d: u8, m: u8) -> f64 {
    f64::from(rasi) * 30.0 + f64::from(d) + f64::from(m) / 60.0
}

/// Example 2: ch. 1 §1.3.8.1.
#[test]
fn example_2_moon_24_12_ge_sun_17_46_sc_is_krishna_chaturthi() {
    assert_eq!(
        tithi(dm(7, 17, 46), dm(2, 24, 12)),
        (19, Paksha::Krishna, Tithi::Chaturthi)
    );
}

/// Exercise 3 and its answer: ch. 1 §1.3.8.1 and §1.6.
#[test]
fn exercise_3_moon_14_43_le_sun_28_13_cp_is_krishna_dwitiya() {
    assert_eq!(
        tithi(dm(9, 28, 13), dm(4, 14, 43)),
        (17, Paksha::Krishna, Tithi::Dwitiya)
    );
}

/// Example 3: ch. 1 §1.3.9.
#[test]
fn example_3_sun_23_50_cp_moon_17_20_li_is_the_10th_yoga_ganda() {
    let yoga = sun_moon_yoga(dm(9, 23, 50), dm(6, 17, 20));
    assert_eq!((yoga, yoga as u8 + 1), (SunMoonYoga::Ganda, 10));
}

/// Exercise 4 and its answer: ch. 1 §1.3.9 and §1.6.
#[test]
fn exercise_4_moon_14_43_le_sun_28_13_cp_is_atiganda() {
    assert_eq!(
        sun_moon_yoga(dm(9, 28, 13), dm(4, 14, 43)),
        SunMoonYoga::Atiganda
    );
}

/// The hora example: ch. 1 §1.3.11. The book counts hours from sunrise; a 24-hour Vedic day makes them horas.
#[test]
fn a_wednesday_with_sunrise_at_6_10_has_the_moons_hora_at_21_40() {
    // 2000-01-05, a Wednesday, at 00:00, in a clock that is UT.
    let wednesday = 2451548.5;
    let sunrise = wednesday + (6.0 + 10.0 / 60.0) / 24.0;
    let day = VedicDay {
        sunrise,
        sun_at_sunrise: 0.0,
        sunset: sunrise + 0.5,
        next_sunrise: sunrise + 1.0,
        clock_sunrise: sunrise,
        clock_sunset: sunrise + 0.5,
        clock_next_sunrise: sunrise + 1.0,
    };
    assert_eq!(vaara(&day), Vaara::Wednesday);
    let birth = wednesday + (21.0 + 40.0 / 60.0) / 24.0;
    assert_eq!(hora(&day, birth), (16, Body::Moon));
}

/// `got` is the book's `book` longitude to `arcseconds`.
#[track_caller]
fn assert_near(got: f64, book: f64, arcseconds: f64) {
    let diff = (got - book) * 3600.0;
    assert!(
        diff.abs() <= arcseconds,
        "got {got}°, book {book}°, off by {diff:.1}″"
    );
}

/// Examples 7–9 and Exercise 8 print the sunrise; 6:37 to 19:23 is 766 minutes.
const EXAMPLE_7_HOURS: f64 = 766.0 / 60.0;

/// Example 7: ch. 5 §5.2. Erratum: §5.2's method step (2), and Example 7 after it, take the minutes as degrees, which
/// gives 10°17′ Pi; §5.2's own rate is one rasi per 2 hours, "1° per 4 minutes (i.e., 15° per hour)": 191°30′.
#[test]
fn example_7_bhaava_lagna_766_minutes_after_sunrise_is_5_47_le() {
    assert_near(
        bhaava_lagna(dm(9, 24, 17), EXAMPLE_7_HOURS),
        dm(4, 5, 47),
        1e-6,
    );
}

/// Example 8: ch. 5 §5.3.
#[test]
fn example_8_hora_lagna_766_minutes_after_sunrise_is_17_17_aq() {
    assert_near(
        hora_lagna(dm(9, 24, 17), EXAMPLE_7_HOURS),
        dm(10, 17, 17),
        1e-6,
    );
}

/// Example 9: ch. 5 §5.4.
#[test]
fn example_9_ghati_lagna_766_minutes_after_sunrise_is_21_47_vi() {
    assert_near(
        ghati_lagna(dm(9, 24, 17), EXAMPLE_7_HOURS),
        dm(5, 21, 47),
        1e-6,
    );
}

/// Exercise 8 and its answer: ch. 5 §5.4 and §5.8. Birth 3:11:48, sunrise 6:19:18 the previous day.
#[test]
fn exercise_8_hora_lagna_8_26_aq_and_ghati_lagna_17_48_5_vi() {
    let hours = 24.0 + (3.0 - 6.0) + (11.0 - 19.0) / 60.0 + (48.0 - 18.0) / 3600.0;
    let sun = dm(1, 12, 11);
    assert_near(hora_lagna(sun, hours), dm(10, 8, 26), 1e-6);
    assert_near(ghati_lagna(sun, hours), dm(5, 17, 48) + 0.5 / 60.0, 1e-6);
}

/// Example 10: ch. 5 §5.7.
#[test]
fn example_10_moon_13_06_li_lagna_25_05_vi_is_sree_lagna_18_47_pi() {
    assert_near(sree_lagna(dm(5, 25, 5), dm(6, 13, 6)), dm(11, 18, 47), 1e-6);
}

/// Exercise 10 and its answer: ch. 5 §5.7 and §5.8.
#[test]
fn exercise_10_moon_15_29_le_lagna_14_19_sc_is_sree_lagna_12_22_cp() {
    assert_near(
        sree_lagna(dm(7, 14, 19), dm(4, 15, 29)),
        dm(9, 12, 22),
        1e-6,
    );
}

/// Not golden (ADR 0008): chart 3 is born at 05:12 IST, before sunrise, so its hora, ghati and bhaava lagnas count
/// from the previous date's sunrise (ch. 5, Exercise 8).
#[tokio::test]
async fn chart_3_born_before_sunrise_counts_from_the_previous_days_sunrise() {
    let sky = book::chart_3_gwalior_1926().await;
    let day = sky.vedic_day.unwrap();
    let hours = (sky.jd - day.sunrise) * 24.0;
    // Sunrise in Gwalior in December is near 07:00 IST.
    assert!((21.0..23.0).contains(&hours), "{hours}");
    let got = special_lagnas(&sky);
    let sun = day.sun_at_sunrise;
    for (lagna, want) in [
        (SpecialLagna::HoraLagna, hora_lagna(sun, hours)),
        (SpecialLagna::GhatiLagna, ghati_lagna(sun, hours)),
        (SpecialLagna::BhaavaLagna, bhaava_lagna(sun, hours)),
    ] {
        assert!(got.contains(&(lagna, want)), "{lagna:?}: {got:?}");
    }
}

/// The sree lagna of `sky`.
fn sree_lagna_of(sky: &Sky) -> f64 {
    special_lagnas(sky)
        .into_iter()
        .find(|(l, _)| *l == SpecialLagna::SreeLagna)
        .unwrap()
        .1
}

/// Example 77 on chart 3: SL in 12°21′ Cp (ch. 20).
#[tokio::test]
async fn chart_3_gwalior_1926_has_sree_lagna_12_21_cp() {
    let sky = book::chart_3_gwalior_1926().await;
    assert_near(sree_lagna_of(&sky), dm(9, 12, 21), 60.0);
}

/// Example 79 on chart 35: SL in 22°22′ Sg (ch. 20).
#[tokio::test]
async fn chart_35_jayalalita_1948_has_sree_lagna_22_22_sg() {
    let sky = book::chart_35_jayalalita_1948().await;
    assert_near(sree_lagna_of(&sky), dm(8, 22, 22), 60.0);
}

/// Example 131 on chart 77: SL printed 14°38′5″ Pi (ch. 32).
#[tokio::test]
async fn chart_77_a_gentleman_1939_has_sree_lagna_14_38_pi() {
    let sky = book::chart_77_a_gentleman_1939().await;
    assert_near(sree_lagna_of(&sky), dm(11, 14, 38), 60.0);
}

/// Example 132 on chart 78: SL in 28°15′ Ar (ch. 32).
#[tokio::test]
async fn chart_78_bill_gates_wrong_time_1955_has_sree_lagna_28_15_ar() {
    let sky = book::chart_78_bill_gates_wrong_time_1955().await;
    assert_near(sree_lagna_of(&sky), dm(0, 28, 15), 60.0);
}

/// The two Jyeshtha maasas of 1999, one Adhika and one Nija: ch. 1 §1.3.8.2, p. 16. Which is which is not from the book.
#[tokio::test]
async fn births_from_the_1999_conjunctions_in_taurus_are_in_two_jyeshtha_months() {
    // The book's 17:35:32 plus hora-swe's 3 s tolerance, as hora's falls at 17:35:33.5 (maintainer's decision, #42).
    let first = book::jyeshtha_1999((1999, 5, 15), (17, 35, 35)).await;
    let second = book::jyeshtha_1999((1999, 6, 14), (0, 33, 27)).await;

    let [(first, a), (second, b)] = [first, second].map(|sky| lunar_month(&sky.new_moons.unwrap()));
    assert_eq!([first, second], [LunarMonth::Jyeshtha; 2]);
    assert!(a.is_some() && b.is_some() && a != b, "{a:?} {b:?}");
}

/// Chart 20 (An engineer in USA): ch. 16 *Vimsottari Dasa*, example 55, p. 221.
#[tokio::test]
async fn chart_20_an_engineer_in_usa_1954_has_moon_in_rohini() {
    let sky = book::chart_20_an_engineer_in_usa_1954().await;

    assert_eq!(nakshatra_of(&sky, Body::Moon).0, Nakshatra::Rohini);
}

/// Chart 22 (A lady): ch. 16 *Vimsottari Dasa*, example 57, p. 223.
#[tokio::test]
async fn chart_22_a_lady_1954_has_moon_in_bharani_pada_1() {
    let sky = book::chart_22_a_lady_1954().await;

    assert_eq!(nakshatra_of(&sky, Body::Moon), (Nakshatra::Bharani, 1));
}

/// Chart 24 (Bill Gates): ch. 26 *Transits: Miscellaneous Topics*, §26.4, p. 348.
#[tokio::test]
async fn chart_24_bill_gates_1955_has_moon_in_uttaraabhaadra() {
    let sky = book::chart_24_bill_gates_1955().await;

    assert_eq!(nakshatra_of(&sky, Body::Moon).0, Nakshatra::Uttaraabhaadra);
}

/// Chart 27's birth data is example 49's: ch. 15 *Strength of Planets and Rasis*, §15.4, pp. 199–200.
#[tokio::test]
async fn chart_27_foreign_stay_example_1970_has_the_nakshatras_of_example_49() {
    let sky = book::chart_27_foreign_stay_example_1970().await;

    use Body::*;
    use Nakshatra::*;
    for (body, book) in [
        (Moon, Poorvaabhaadra),
        (Mercury, Aswini),
        (Jupiter, Swaati),
        (Venus, Aswini),
    ] {
        assert_eq!(nakshatra_of(&sky, body).0, book, "{body:?}");
    }
}

/// Chart 4 (an engineer in the US): ch. 9 *Arudha Padas*, p. 98.
#[tokio::test]
async fn chart_4_marks_only_jupiter_retrograde() {
    let sky = book::chart_4_pithapuram_1968().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 6 (P.V. Narasimha Rao): ch. 10 *Aspects and Argalas*, p. 107.
#[tokio::test]
async fn chart_6_p_v_narasimha_rao_1921_marks_only_mercury_retrograde() {
    let sky = book::chart_6_p_v_narasimha_rao_1921().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 8 (Guess his livelihood!): ch. 10 *Aspects and Argalas*, p. 109.
#[tokio::test]
async fn chart_8_guess_his_livelihood_1946_marks_venus_saturn_retrograde() {
    let sky = book::chart_8_guess_his_livelihood_1946().await;

    assert_eq!(marked_retrograde(&sky), [Body::Venus, Body::Saturn]);
}

/// Chart 9 (Chatrapati Shivaji): ch. 11 *Yogas*, §11.6, p. 128.
#[tokio::test]
async fn chart_9_chatrapati_shivaji_1630_marks_only_saturn_retrograde() {
    let sky = book::chart_9_chatrapati_shivaji_1630().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 11 (Mercury's Ashtakavarga): ch. 12 *Ashtakavarga*, p. 152.
#[tokio::test]
async fn chart_11_p_v_narasimha_rao_1921_marks_only_mercury_retrograde() {
    let sky = book::chart_11_p_v_narasimha_rao_1921().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 12 (SAV Exercise): ch. 12 *Ashtakavarga*, p. 155.
#[tokio::test]
async fn chart_12_sav_exercise_1958_marks_mercury_saturn_retrograde() {
    let sky = book::chart_12_sav_exercise_1958().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury, Body::Saturn]);
}

/// Chart 13 (An example): ch. 13 *Interpreting Charts*, p. 173.
#[tokio::test]
async fn chart_13_an_example_1894_marks_only_saturn_retrograde() {
    let sky = book::chart_13_an_example_1894().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 15 (Satyam Gaur): ch. 13 *Interpreting Charts*, p. 177.
#[tokio::test]
async fn chart_15_satyam_gaur_1970_marks_venus_saturn_retrograde() {
    let sky = book::chart_15_satyam_gaur_1970().await;

    assert_eq!(marked_retrograde(&sky), [Body::Venus, Body::Saturn]);
}

/// Chart 15's second chart (Shivam Gaur): ch. 13 *Interpreting Charts*, p. 177.
#[tokio::test]
async fn chart_15_shivam_gaur_1970_marks_venus_saturn_retrograde() {
    let sky = book::chart_15_shivam_gaur_1970().await;

    assert_eq!(marked_retrograde(&sky), [Body::Venus, Body::Saturn]);
}

/// Chart 17 (Pandit Sanjay Rath): ch. 16 *Vimsottari Dasa*, p. 216.
#[tokio::test]
async fn chart_17_pandit_sanjay_rath_1963_marks_only_saturn_retrograde() {
    let sky = book::chart_17_pandit_sanjay_rath_1963().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 18 (A lady): ch. 16 *Vimsottari Dasa*, p. 217.
#[tokio::test]
async fn chart_18_a_lady_1972_marks_jupiter_venus_retrograde() {
    let sky = book::chart_18_a_lady_1972().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter, Body::Venus]);
}

/// Chart 20 (An engineer in USA): ch. 16 *Vimsottari Dasa*, p. 220.
#[tokio::test]
async fn chart_20_an_engineer_in_usa_1954_marks_only_venus_retrograde() {
    let sky = book::chart_20_an_engineer_in_usa_1954().await;

    assert_eq!(marked_retrograde(&sky), [Body::Venus]);
}

/// Chart 21 (John F. Kennedy, Jr.): ch. 16 *Vimsottari Dasa*, p. 222.
#[tokio::test]
async fn chart_21_john_f_kennedy_jr_1960_marks_only_mars_retrograde() {
    let sky = book::chart_21_john_f_kennedy_jr_1960().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mars]);
}

/// Chart 22 (A lady): ch. 16 *Vimsottari Dasa*, p. 223.
#[tokio::test]
async fn chart_22_a_lady_1954_marks_only_jupiter_retrograde() {
    let sky = book::chart_22_a_lady_1954().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 23 (Dr. B.V. Raman): ch. 16 *Vimsottari Dasa*, p. 224.
#[tokio::test]
async fn chart_23_dr_b_v_raman_1912_marks_only_mercury_retrograde() {
    let sky = book::chart_23_dr_b_v_raman_1912().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 26 (Narayana Dasa Exercise): ch. 18 *Narayana Dasa*, p. 243.
#[tokio::test]
async fn chart_26_narayana_dasa_exercise_1971_marks_only_jupiter_retrograde() {
    let sky = book::chart_26_narayana_dasa_exercise_1971().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 27 (Foreign Stay Example): ch. 18 *Narayana Dasa*, p. 246.
#[tokio::test]
async fn chart_27_foreign_stay_example_1970_marks_only_jupiter_retrograde() {
    let sky = book::chart_27_foreign_stay_example_1970().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 29 (A lady): ch. 18 *Narayana Dasa*, p. 249.
#[tokio::test]
async fn chart_29_a_lady_1969_marks_only_mars_retrograde() {
    let sky = book::chart_29_a_lady_1969().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mars]);
}

/// Chart 32 (A gentleman): ch. 18 *Narayana Dasa*, p. 252.
#[tokio::test]
async fn chart_32_a_gentleman_1961_marks_jupiter_saturn_retrograde() {
    let sky = book::chart_32_a_gentleman_1961().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter, Body::Saturn]);
}

/// Chart 35 (Jayalalita): ch. 20 *Sudasa*, p. 266.
#[tokio::test]
async fn chart_35_jayalalita_1948_marks_mars_mercury_saturn_retrograde() {
    let sky = book::chart_35_jayalalita_1948().await;

    assert_eq!(
        marked_retrograde(&sky),
        [Body::Mars, Body::Mercury, Body::Saturn]
    );
}

/// Chart 38 (Sri Aurobindo Ghose): ch. 21 *Drigdasa*, p. 271.
#[tokio::test]
async fn chart_38_sri_aurobindo_ghose_1872_marks_only_saturn_retrograde() {
    let sky = book::chart_38_sri_aurobindo_ghose_1872().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 42 (Niryaana Shoola Exercise): ch. 22 *Niryaana Shoola Dasa*, p. 279.
#[tokio::test]
async fn chart_42_niryaana_shoola_exercise_1889_marks_only_venus_retrograde() {
    let sky = book::chart_42_niryaana_shoola_exercise_1889().await;

    assert_eq!(marked_retrograde(&sky), [Body::Venus]);
}

/// Chart 43 (Shoola dasa exercise): ch. 23 *Shoola Dasa*, p. 284.
#[tokio::test]
async fn chart_43_shoola_dasa_exercise_1915_marks_only_saturn_retrograde() {
    let sky = book::chart_43_shoola_dasa_exercise_1915().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 44 (Pitri Shoola Example): ch. 23 *Shoola Dasa*, p. 286.
#[tokio::test]
async fn chart_44_pitri_shoola_example_1971_marks_only_jupiter_retrograde() {
    let sky = book::chart_44_pitri_shoola_example_1971().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 48 (A gentleman): ch. 24 *Kalachakra Dasa*, p. 302.
#[tokio::test]
async fn chart_48_a_gentleman_1933_marks_only_saturn_retrograde() {
    let sky = book::chart_48_a_gentleman_1933().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 51 (An actor): ch. 24 *Kalachakra Dasa*, p. 307.
#[tokio::test]
async fn chart_51_an_actor_1937_marks_only_jupiter_retrograde() {
    let sky = book::chart_51_an_actor_1937().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Example 27 (Bill Cosby): ch. 6 §6.6.4, p. 65, Jupiter in the sixteen vargas of shodasa varga. Chart 51 has the same
/// printed birth data; its Jupiter, 29°51′ Sg (ch. 24), and Example 27's 29°49′ are both at least 9′ inside every part.
#[tokio::test]
async fn example_27_bill_cosby_1937_has_jupiter_in_the_rasis_of_shodasa_varga() {
    use Rasi::*;
    let sky = book::chart_51_an_actor_1937().await;

    let got: Vec<_> = VARGAS
        .into_iter()
        // D-5, D-6, D-8 and D-11 are the four vargas outside shodasa varga.
        .filter(|v| ![5, 6, 8, 11].contains(&v.parts()))
        .map(|v| (v.parts(), varga(v, sky.grahas.jupiter.longitude)))
        .collect();
    let want = [
        (1, Sagittarius),
        (2, Cancer),
        // Erratum: the book prints D-3 Li, which §6.2.3 cannot give: the last 10° of Sg go to the 9th from it, Le.
        (3, Leo),
        (4, Virgo),
        (7, Gemini),
        (9, Sagittarius),
        (10, Virgo),
        (12, Scorpio),
        (16, Pisces),
        (20, Pisces),
        (24, Cancer),
        (27, Gemini),
        (30, Libra),
        (40, Cancer),
        (45, Leo),
        (60, Scorpio),
    ];
    assert_eq!(got, want);
}

/// Chart 53 (A lady): ch. 25 *Transits and Natal References*, p. 321.
#[tokio::test]
async fn chart_53_a_lady_1973_marks_mercury_jupiter_retrograde() {
    let sky = book::chart_53_a_lady_1973().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury, Body::Jupiter]);
}

/// Chart 54 (An engineer): ch. 25 *Transits and Natal References*, p. 324.
#[tokio::test]
async fn chart_54_an_engineer_1970_marks_only_saturn_retrograde() {
    let sky = book::chart_54_an_engineer_1970().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 55 (A gentleman): ch. 25 *Transits and Natal References*, p. 326.
#[tokio::test]
async fn chart_55_a_gentleman_1961_marks_mercury_jupiter_saturn_retrograde() {
    let sky = book::chart_55_a_gentleman_1961().await;

    assert_eq!(
        marked_retrograde(&sky),
        [Body::Mercury, Body::Jupiter, Body::Saturn]
    );
}

/// Chart 55's second chart (Childbirth (transit)): ch. 25 *Transits and Natal References*, p. 326.
#[tokio::test]
async fn chart_55_childbirth_transit_1992_marks_only_saturn_retrograde() {
    let sky = book::chart_55_childbirth_transit_1992().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 56 (John F. Kennedy, Jr): ch. 25 *Transits and Natal References*, p. 327.
#[tokio::test]
async fn chart_56_john_f_kennedy_jr_1960_marks_only_mars_retrograde() {
    let sky = book::chart_56_john_f_kennedy_jr_1960().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mars]);
}

/// Chart 56's second chart (Death (transit)): ch. 25 *Transits and Natal References*, p. 327.
#[tokio::test]
async fn chart_56_death_transit_1999_marks_only_mercury_retrograde() {
    let sky = book::chart_56_death_transit_1999().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 61 (Indira Gandhi): ch. 25 *Transits and Natal References*, p. 336.
#[tokio::test]
async fn chart_61_indira_gandhi_1917_marks_only_jupiter_retrograde() {
    let sky = book::chart_61_indira_gandhi_1917().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 62 (Princess Diana): ch. 25 *Transits and Natal References*, p. 339.
#[tokio::test]
async fn chart_62_princess_diana_1961_marks_mercury_jupiter_saturn_retrograde() {
    let sky = book::chart_62_princess_diana_1961().await;

    assert_eq!(
        marked_retrograde(&sky),
        [Body::Mercury, Body::Jupiter, Body::Saturn]
    );
}

/// Chart 66 (Annual chart example): ch. 27 *Tajaka Chart Basics*, p. 368.
#[tokio::test]
async fn chart_66_annual_2000_marks_only_mercury_retrograde() {
    let sky = book::chart_66_annual_2000().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 68 (Going abroad (Tajaka)): ch. 30 *Annual Dasas*, p. 403.
#[tokio::test]
async fn chart_68_annual_1991_marks_only_mercury_retrograde() {
    let sky = book::chart_68_annual_1991().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 69 (Academic success): ch. 30 *Annual Dasas*, p. 404.
#[tokio::test]
async fn chart_69_annual_1987_marks_only_saturn_retrograde() {
    let sky = book::chart_69_annual_1987().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 70 (Childbirth): ch. 30 *Annual Dasas*, p. 405.
#[tokio::test]
async fn chart_70_annual_1998_marks_only_mercury_retrograde() {
    let sky = book::chart_70_annual_1998().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 71 (Bought a car): ch. 30 *Annual Dasas*, p. 407.
#[tokio::test]
async fn chart_71_annual_1995_marks_only_jupiter_retrograde() {
    let sky = book::chart_71_annual_1995().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 73's second chart (Indira Gandhi 1976-77): ch. 31 *Sudarsana Chakra Dasa*, p. 413.
#[tokio::test]
async fn chart_73_indira_gandhi_annual_1976_marks_only_jupiter_retrograde() {
    let sky = book::chart_73_indira_gandhi_annual_1976().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 74's second chart (A lady 1998-99): ch. 31 *Sudarsana Chakra Dasa*, p. 415.
#[tokio::test]
async fn chart_74_a_lady_annual_1998_marks_only_jupiter_retrograde() {
    let sky = book::chart_74_a_lady_annual_1998().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 75 (Rectification Example 1): ch. 32 *Impact of Birthtime Error*, p. 429.
#[tokio::test]
async fn chart_75_rectification_example_1966_marks_only_jupiter_retrograde() {
    let sky = book::chart_75_rectification_example_1966().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 77 (A gentleman): ch. 32 *Impact of Birthtime Error*, p. 432.
#[tokio::test]
async fn chart_77_a_gentleman_1939_marks_jupiter_saturn_retrograde() {
    let sky = book::chart_77_a_gentleman_1939().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter, Body::Saturn]);
}

/// Example 131 on chart 77: "Cn contains two highly malefic upagrahas also – Indrachapa and Upaketu" (ch. 32, p. 433).
#[tokio::test]
async fn chart_77_a_gentleman_1939_has_indrachaapa_and_upaketu_in_cancer() {
    let sky = book::chart_77_a_gentleman_1939().await;

    for upagraha in [Upagraha::Indrachaapa, Upagraha::Upaketu] {
        assert_eq!(upagraha_rasi(&sky, upagraha), Rasi::Cancer, "{upagraha:?}");
    }
}

/// Example 131 on chart 77: "Though two good upagrahas (Artha Prahara and Yamaghantaka [...]) are present in it" (ch. 32, p. 433).
#[tokio::test]
async fn chart_77_a_gentleman_1939_has_arthaprahaara_and_yamaghantaka_in_cancer() {
    let sky = book::chart_77_a_gentleman_1939().await;

    for upagraha in [Upagraha::Arthaprahaara, Upagraha::Yamaghantaka] {
        assert_eq!(upagraha_rasi(&sky, upagraha), Rasi::Cancer, "{upagraha:?}");
    }
}

/// Chart 79 (A Music Teacher): ch. 32 *Impact of Birthtime Error*, p. 436.
#[tokio::test]
async fn chart_79_a_music_teacher_1973_marks_only_jupiter_retrograde() {
    let sky = book::chart_79_a_music_teacher_1973().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 80's second chart (India Tajaka 1997-98): ch. 35 *Mundane Astrology*, p. 463.
#[tokio::test]
async fn chart_80_india_annual_1997_marks_jupiter_saturn_retrograde() {
    let sky = book::chart_80_india_annual_1997().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter, Body::Saturn]);
}

/// Chart 81 (India: Lunar 1992-93): ch. 35 *Mundane Astrology*, p. 465.
#[tokio::test]
async fn chart_81_india_lunar_year_1992_marks_mercury_jupiter_retrograde() {
    let sky = book::chart_81_india_lunar_year_1992().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury, Body::Jupiter]);
}

/// Chart 82 (India: Solar year 1998): ch. 35 *Mundane Astrology*, p. 467.
#[tokio::test]
async fn chart_82_india_solar_year_1998_marks_only_venus_retrograde() {
    let sky = book::chart_82_india_solar_year_1998().await;

    assert_eq!(marked_retrograde(&sky), [Body::Venus]);
}

/// Chart 84 (Vajpayee Swearing-in 1): ch. 36 *Muhurta or Electional Astrology*, p. 476.
#[tokio::test]
async fn chart_84_vajpayee_swearing_in_1996_marks_mercury_jupiter_retrograde() {
    let sky = book::chart_84_vajpayee_swearing_in_1996().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury, Body::Jupiter]);
}

/// Example 142 on chart 85, chart 59's birth: "Situation of Gulika within 1º from lagna" (ch. 36, p. 478).
#[tokio::test]
async fn chart_85_vajpayee_swearing_in_1998_has_gulika_in_the_lagnas_rasi() {
    let sky = book::chart_59_vajpayee_swearing_in_1998().await;

    // At rasi level only: hora's Gulika is 1°14′ from the lagna, the book's later sunrise (#71).
    assert_eq!(upagraha_rasi(&sky, Upagraha::Gulika), Rasi::Taurus);
    assert_eq!(rasi(sky.lagna).0, Rasi::Taurus);
}

/// Chart 88 (An astrologer): ch. 36 *Muhurta or Electional Astrology*, p. 490.
#[tokio::test]
async fn chart_88_an_astrologer_1949_marks_only_saturn_retrograde() {
    let sky = book::chart_88_an_astrologer_1949().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// Chart 89 (Tajaka 1999-2000): ch. 36 *Muhurta or Electional Astrology*, p. 491.
#[tokio::test]
async fn chart_89_annual_1999_marks_only_mercury_retrograde() {
    let sky = book::chart_89_annual_1999().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 90 (An Indian in USA): ch. 36 *Muhurta or Electional Astrology*, p. 493.
#[tokio::test]
async fn chart_90_an_indian_in_usa_1968_marks_only_jupiter_retrograde() {
    let sky = book::chart_90_an_indian_in_usa_1968().await;

    assert_eq!(marked_retrograde(&sky), [Body::Jupiter]);
}

/// Chart 91 (Tajaka 1998-99): ch. 36 *Muhurta or Electional Astrology*, p. 494.
#[tokio::test]
async fn chart_91_annual_1998_marks_only_venus_retrograde() {
    let sky = book::chart_91_annual_1998().await;

    assert_eq!(marked_retrograde(&sky), [Body::Venus]);
}

/// Chart 93 (March-April, 2000): ch. 36 *Muhurta or Electional Astrology*, p. 497.
#[tokio::test]
async fn chart_93_monthly_march_2000_marks_only_mercury_retrograde() {
    let sky = book::chart_93_monthly_march_2000().await;

    assert_eq!(marked_retrograde(&sky), [Body::Mercury]);
}

/// Chart 96 (Tajaka 1991-92): ch. 36 *Muhurta or Electional Astrology*, p. 501.
#[tokio::test]
async fn chart_96_tajaka_1991_marks_only_saturn_retrograde() {
    let sky = book::chart_96_tajaka_1991().await;

    assert_eq!(marked_retrograde(&sky), [Body::Saturn]);
}

/// The rasi of the lagna and of each of the nine grahas in `chart`'s order, in `v` of the chart of `sky`.
fn drawn(sky: &Sky, v: Varga) -> [Rasi; 10] {
    chart(sky).map(|e| varga(v, e.longitude))
}

/// Chart 2 (Arudha exercise): D-16, ch. 9, p. 89.
#[tokio::test]
async fn chart_2_wilmington_2000_at_utc_minus_5_shodasamsa() {
    use Rasi::*;
    let sky = book::chart_2_wilmington_2000_at_utc_minus_5().await;

    assert_eq!(
        drawn(&sky, Varga::Shodasamsa),
        [
            Sagittarius,
            Aquarius,
            Aquarius,
            Aquarius,
            Sagittarius,
            Capricorn,
            Taurus,
            Aries,
            Cancer,
            Cancer
        ]
    );
}

/// Chart 5 (Aspects Exercise): D-9, ch. 10, p. 101.
#[tokio::test]
async fn chart_5_aspects_exercise_2000_navamsa() {
    use Rasi::*;
    let sky = book::chart_5_aspects_exercise_2000().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Scorpio,
            Taurus,
            Aries,
            Scorpio,
            Sagittarius,
            Virgo,
            Capricorn,
            Scorpio,
            Leo,
            Aquarius
        ]
    );
}

/// Chart 9 (Chatrapati Shivaji): D-9, ch. 11, p. 128.
#[tokio::test]
async fn chart_9_chatrapati_shivaji_1630_navamsa() {
    use Rasi::*;
    let sky = book::chart_9_chatrapati_shivaji_1630().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Sagittarius,
            Aries,
            Virgo,
            Aquarius,
            Leo,
            Scorpio,
            Gemini,
            Aquarius,
            Virgo,
            Pisces
        ]
    );
}

/// Chart 12 (SAV Exercise): D-10, ch. 12, p. 155.
#[tokio::test]
async fn chart_12_sav_exercise_1958_dasamsa() {
    use Rasi::*;
    let sky = book::chart_12_sav_exercise_1958().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Virgo,
            Sagittarius,
            Capricorn,
            Scorpio,
            Sagittarius,
            Scorpio,
            Taurus,
            Pisces,
            Libra,
            Aries
        ]
    );
}

/// Chart 13 (An example): D-20, ch. 13, p. 173.
#[tokio::test]
async fn chart_13_an_example_1894_vimsamsa() {
    use Rasi::*;
    let sky = book::chart_13_an_example_1894().await;

    assert_eq!(
        drawn(&sky, Varga::Vimsamsa),
        [
            Pisces, Aries, Virgo, Virgo, Aries, Scorpio, Libra, Capricorn, Gemini, Gemini
        ]
    );
}

/// Chart 14 (Rajiv Gandhi): D-3, ch. 13, p. 175.
#[tokio::test]
async fn chart_14_rajiv_gandhi_1944_drekkana() {
    use Rasi::*;
    let sky = book::chart_14_rajiv_gandhi_1944().await;

    assert_eq!(
        drawn(&sky, Varga::Drekkana),
        [
            Sagittarius,
            Leo,
            Sagittarius,
            Virgo,
            Aries,
            Sagittarius,
            Sagittarius,
            Libra,
            Cancer,
            Capricorn
        ]
    );
}

/// Chart 15 (Satyam Gaur): D-24, ch. 13, p. 177.
#[tokio::test]
async fn chart_15_satyam_gaur_1970_chaturvimsamsa() {
    use Rasi::*;
    let sky = book::chart_15_satyam_gaur_1970().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturvimsamsa),
        [
            Libra,
            Libra,
            Capricorn,
            Cancer,
            Aquarius,
            Capricorn,
            Gemini,
            Taurus,
            Sagittarius,
            Sagittarius
        ]
    );
}

/// Chart 15 (Shivam Gaur): D-24, ch. 13, p. 177.
#[tokio::test]
async fn chart_15_shivam_gaur_1970_chaturvimsamsa() {
    use Rasi::*;
    let sky = book::chart_15_shivam_gaur_1970().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturvimsamsa),
        [
            Scorpio,
            Libra,
            Capricorn,
            Cancer,
            Aquarius,
            Capricorn,
            Gemini,
            Taurus,
            Sagittarius,
            Sagittarius
        ]
    );
}

/// Chart 16 (Satyam Gaur): D-27, ch. 13, p. 178.
#[tokio::test]
#[ignore = "hora's Rahu, 5°33′20.15″ Aq, is 0.15″ past the D-27 part boundary at 5°33′20″ (Ketu likewise); the book draws them before it (#160)"]
async fn chart_16_satyam_gaur_1970_nakshatramsa() {
    use Rasi::*;
    let sky = book::chart_15_satyam_gaur_1970().await;

    assert_eq!(
        drawn(&sky, Varga::Nakshatramsa),
        [
            Gemini, Aquarius, Scorpio, Virgo, Gemini, Taurus, Libra, Pisces, Aquarius, Leo
        ]
    );
}

/// Chart 16 (Shivam Gaur): D-27, ch. 13, p. 178.
#[tokio::test]
async fn chart_16_shivam_gaur_1970_nakshatramsa() {
    use Rasi::*;
    let sky = book::chart_15_shivam_gaur_1970().await;

    assert_eq!(
        drawn(&sky, Varga::Nakshatramsa),
        [
            Cancer, Aquarius, Scorpio, Virgo, Gemini, Taurus, Libra, Pisces, Aquarius, Leo
        ]
    );
}

/// Chart 17 (Pandit Sanjay Rath): D-10, ch. 16, p. 216.
#[tokio::test]
async fn chart_17_pandit_sanjay_rath_1963_dasamsa() {
    use Rasi::*;
    let sky = book::chart_17_pandit_sanjay_rath_1963().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Pisces,
            Libra,
            Leo,
            Virgo,
            Sagittarius,
            Cancer,
            Cancer,
            Taurus,
            Aquarius,
            Leo
        ]
    );
}

/// Chart 18 (A lady): D-7, ch. 16, p. 217.
#[tokio::test]
async fn chart_18_a_lady_1972_saptamsa() {
    use Rasi::*;
    let sky = book::chart_18_a_lady_1972().await;

    assert_eq!(
        drawn(&sky, Varga::Saptamsa),
        [
            Libra, Aquarius, Gemini, Libra, Capricorn, Aquarius, Leo, Aquarius, Leo, Aquarius
        ]
    );
}

/// Chart 27 (Foreign Stay Example): D-4, ch. 18, p. 246.
#[tokio::test]
async fn chart_27_foreign_stay_example_1970_chaturthamsa() {
    use Rasi::*;
    let sky = book::chart_27_foreign_stay_example_1970().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturthamsa),
        [
            Sagittarius,
            Virgo,
            Scorpio,
            Capricorn,
            Aries,
            Capricorn,
            Cancer,
            Libra,
            Leo,
            Aquarius
        ]
    );
}

/// Chart 28 (A lady): D-9, ch. 18, p. 248.
#[tokio::test]
async fn chart_28_a_lady_1971_navamsa() {
    use Rasi::*;
    let sky = book::chart_28_a_lady_1971().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Libra,
            Scorpio,
            Virgo,
            Gemini,
            Gemini,
            Leo,
            Sagittarius,
            Aries,
            Gemini,
            Sagittarius
        ]
    );
}

/// Chart 29 (A lady): D-9, ch. 18, p. 249.
#[tokio::test]
async fn chart_29_a_lady_1969_navamsa() {
    use Rasi::*;
    let sky = book::chart_29_a_lady_1969().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Aquarius,
            Aries,
            Cancer,
            Virgo,
            Sagittarius,
            Aquarius,
            Pisces,
            Leo,
            Cancer,
            Capricorn
        ]
    );
}

/// Chart 31 (An electrical engineer): D-10, ch. 18, p. 251.
#[tokio::test]
async fn chart_31_an_electrical_engineer_1947_dasamsa() {
    use Rasi::*;
    let sky = book::chart_31_an_electrical_engineer_1947().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Cancer, Aquarius, Taurus, Capricorn, Taurus, Cancer, Pisces, Libra, Aquarius, Leo
        ]
    );
}

/// Chart 32 (A gentleman): D-10, ch. 18, p. 252.
#[tokio::test]
async fn chart_32_a_gentleman_1961_dasamsa() {
    use Rasi::*;
    let sky = book::chart_32_a_gentleman_1961().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Pisces,
            Taurus,
            Aquarius,
            Pisces,
            Sagittarius,
            Scorpio,
            Virgo,
            Virgo,
            Virgo,
            Pisces
        ]
    );
}

/// Chart 33 (Accident example): D-16, ch. 18, p. 253.
#[tokio::test]
async fn chart_33_accident_example_1970_shodasamsa() {
    use Rasi::*;
    let sky = book::chart_27_foreign_stay_example_1970().await;

    assert_eq!(
        drawn(&sky, Varga::Shodasamsa),
        [
            Cancer,
            Scorpio,
            Scorpio,
            Gemini,
            Taurus,
            Virgo,
            Leo,
            Sagittarius,
            Taurus,
            Taurus
        ]
    );
}

/// Chart 46 (A male): D-9, ch. 24, p. 300.
#[tokio::test]
async fn chart_46_a_male_1971_navamsa() {
    use Rasi::*;
    let sky = book::chart_44_pitri_shoola_example_1971().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Pisces,
            Scorpio,
            Sagittarius,
            Pisces,
            Aries,
            Virgo,
            Aquarius,
            Capricorn,
            Leo,
            Aquarius
        ]
    );
}

/// Chart 47 (An astrologer): D-24, ch. 24, p. 301.
#[tokio::test]
async fn chart_47_an_astrologer_1970_chaturvimsamsa() {
    use Rasi::*;
    let sky = book::chart_27_foreign_stay_example_1970().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturvimsamsa),
        [
            Gemini, Scorpio, Gemini, Taurus, Libra, Pisces, Aquarius, Leo, Virgo, Virgo
        ]
    );
}

/// Chart 48 (A gentleman): D-12, ch. 24, p. 302.
#[tokio::test]
async fn chart_48_a_gentleman_1933_dwadasamsa() {
    use Rasi::*;
    let sky = book::chart_48_a_gentleman_1933().await;

    assert_eq!(
        drawn(&sky, Varga::Dwadasamsa),
        [
            Virgo, Taurus, Taurus, Capricorn, Pisces, Gemini, Pisces, Virgo, Taurus, Scorpio
        ]
    );
}

/// Chart 49 (An ISKCON devotee): D-20, ch. 24, p. 304.
#[tokio::test]
async fn chart_49_an_iskcon_devotee_1971_vimsamsa() {
    use Rasi::*;
    let sky = book::chart_49_an_iskcon_devotee_1971().await;

    assert_eq!(
        drawn(&sky, Varga::Vimsamsa),
        [
            Aries,
            Sagittarius,
            Gemini,
            Taurus,
            Leo,
            Taurus,
            Taurus,
            Gemini,
            Sagittarius,
            Sagittarius
        ]
    );
}

/// Chart 50 (Divorced Lady): D-9, ch. 24, p. 306.
#[tokio::test]
async fn chart_50_divorced_lady_1968_navamsa() {
    use Rasi::*;
    let sky = book::chart_50_divorced_lady_1968().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Pisces,
            Pisces,
            Leo,
            Taurus,
            Virgo,
            Taurus,
            Sagittarius,
            Pisces,
            Capricorn,
            Cancer
        ]
    );
}

/// Chart 51 (An actor): D-10, ch. 24, p. 307.
#[tokio::test]
async fn chart_51_an_actor_1937_dasamsa() {
    use Rasi::*;
    let sky = book::chart_51_an_actor_1937().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Libra, Aquarius, Aquarius, Cancer, Pisces, Virgo, Aries, Pisces, Capricorn, Cancer
        ]
    );
}

/// Chart 54 (An engineer): D-9, ch. 25, p. 324.
#[tokio::test]
async fn chart_54_an_engineer_1970_navamsa() {
    use Rasi::*;
    let sky = book::chart_54_an_engineer_1970().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Virgo,
            Gemini,
            Libra,
            Sagittarius,
            Capricorn,
            Aquarius,
            Gemini,
            Sagittarius,
            Sagittarius,
            Gemini
        ]
    );
}

/// Chart 55 (A gentleman): D-7, ch. 25, p. 326.
#[tokio::test]
async fn chart_55_a_gentleman_1961_saptamsa() {
    use Rasi::*;
    let sky = book::chart_55_a_gentleman_1961().await;

    assert_eq!(
        drawn(&sky, Varga::Saptamsa),
        [
            Taurus, Leo, Scorpio, Leo, Virgo, Virgo, Virgo, Leo, Virgo, Pisces
        ]
    );
}

/// Chart 56 (John F. Kennedy, Jr): D-11, ch. 25, p. 327.
#[tokio::test]
async fn chart_56_john_f_kennedy_jr_1960_rudramsa() {
    use Rasi::*;
    let sky = book::chart_56_john_f_kennedy_jr_1960().await;

    assert_eq!(
        drawn(&sky, Varga::Rudramsa),
        [
            Gemini,
            Sagittarius,
            Cancer,
            Scorpio,
            Taurus,
            Sagittarius,
            Aquarius,
            Aries,
            Gemini,
            Sagittarius
        ]
    );
}

/// Chart 58 (A.B. Vajpayee): D-10, ch. 25, p. 331.
#[tokio::test]
async fn chart_58_vajpayee_1926_dasamsa() {
    use Rasi::*;
    let sky = book::chart_58_vajpayee_1926().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Scorpio, Pisces, Capricorn, Leo, Capricorn, Aquarius, Taurus, Libra, Libra, Aries
        ]
    );
}

/// Chart 68 (Going abroad (Tajaka)): D-4, ch. 30, p. 403.
#[tokio::test]
async fn chart_68_annual_1991_chaturthamsa() {
    use Rasi::*;
    let sky = book::chart_68_annual_1991().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturthamsa),
        [
            Libra, Virgo, Leo, Gemini, Aries, Libra, Capricorn, Aries, Capricorn, Cancer
        ]
    );
}

/// Chart 69 (Academic success): D-24, ch. 30, p. 404.
#[tokio::test]
async fn chart_69_annual_1987_chaturvimsamsa() {
    use Rasi::*;
    let sky = book::chart_69_annual_1987().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturvimsamsa),
        [
            Taurus, Scorpio, Scorpio, Scorpio, Pisces, Gemini, Cancer, Aries, Virgo, Virgo
        ]
    );
}

/// Chart 70 (Childbirth): D-7, ch. 30, p. 405.
#[tokio::test]
async fn chart_70_annual_1998_saptamsa() {
    use Rasi::*;
    let sky = book::chart_70_annual_1998().await;

    assert_eq!(
        drawn(&sky, Varga::Saptamsa),
        [
            Leo, Capricorn, Capricorn, Pisces, Aquarius, Gemini, Pisces, Pisces, Scorpio, Taurus
        ]
    );
}

/// Chart 71 (Bought a car): D-16, ch. 30, p. 407.
#[tokio::test]
async fn chart_71_annual_1995_shodasamsa() {
    use Rasi::*;
    let sky = book::chart_71_annual_1995().await;

    assert_eq!(
        drawn(&sky, Varga::Shodasamsa),
        [
            Libra, Scorpio, Pisces, Aquarius, Taurus, Cancer, Aries, Virgo, Libra, Libra
        ]
    );
}

/// Chart 74 (A lady): D-9, ch. 31, p. 415.
#[tokio::test]
async fn chart_74_a_lady_1973_navamsa() {
    use Rasi::*;
    let sky = book::chart_53_a_lady_1973().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Cancer, Libra, Leo, Pisces, Cancer, Taurus, Gemini, Scorpio, Cancer, Capricorn
        ]
    );
}

/// Chart 74 (1998-99): D-9, ch. 31, p. 415.
#[tokio::test]
async fn chart_74_a_lady_annual_1998_navamsa() {
    use Rasi::*;
    let sky = book::chart_74_a_lady_annual_1998().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Pisces,
            Libra,
            Libra,
            Aries,
            Taurus,
            Leo,
            Aquarius,
            Gemini,
            Gemini,
            Sagittarius
        ]
    );
}

/// Chart 75 (Rectification Example 1): D-4, ch. 32, p. 429.
#[tokio::test]
async fn chart_75_rectification_example_1966_chaturthamsa() {
    use Rasi::*;
    let sky = book::chart_75_rectification_example_1966().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturthamsa),
        [
            Aries, Aquarius, Virgo, Virgo, Cancer, Libra, Taurus, Scorpio, Libra, Aries
        ]
    );
}

/// Chart 75 (Rectification Example 1): D-9, ch. 32, p. 429.
#[tokio::test]
async fn chart_75_rectification_example_1966_navamsa() {
    use Rasi::*;
    let sky = book::chart_75_rectification_example_1966().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Aries,
            Scorpio,
            Sagittarius,
            Aquarius,
            Taurus,
            Libra,
            Sagittarius,
            Gemini,
            Libra,
            Aries
        ]
    );
}

/// Chart 76 (Rectification Example 1): D-10, ch. 32, p. 430.
#[tokio::test]
async fn chart_76_rectification_example_1966_dasamsa() {
    use Rasi::*;
    let sky = book::chart_75_rectification_example_1966().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Capricorn, Scorpio, Virgo, Gemini, Gemini, Gemini, Capricorn, Scorpio, Scorpio, Taurus
        ]
    );
}

/// Chart 76 (Rectification Example 1): D-24, ch. 32, p. 430.
#[tokio::test]
async fn chart_76_rectification_example_1966_chaturvimsamsa() {
    use Rasi::*;
    let sky = book::chart_75_rectification_example_1966().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturvimsamsa),
        [
            Taurus, Gemini, Pisces, Libra, Pisces, Pisces, Libra, Cancer, Capricorn, Capricorn
        ]
    );
}

/// Chart 79 (A Music Teacher): D-10, ch. 32, p. 436.
#[tokio::test]
async fn chart_79_a_music_teacher_1973_dasamsa() {
    use Rasi::*;
    let sky = book::chart_79_a_music_teacher_1973().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Scorpio,
            Aquarius,
            Sagittarius,
            Leo,
            Pisces,
            Sagittarius,
            Aquarius,
            Virgo,
            Pisces,
            Virgo
        ]
    );
}

/// Chart 80 (India's Independence): D-24, ch. 35, p. 463.
#[tokio::test]
async fn chart_80_india_1947_chaturvimsamsa() {
    use Rasi::*;
    let sky = book::chart_25_india_1947().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturvimsamsa),
        [
            Capricorn, Taurus, Libra, Capricorn, Taurus, Aries, Capricorn, Scorpio, Scorpio,
            Scorpio
        ]
    );
}

/// Chart 80 (Tajaka 1997-1998): D-24, ch. 35, p. 463.
#[tokio::test]
async fn chart_80_india_annual_1997_chaturvimsamsa() {
    use Rasi::*;
    let sky = book::chart_80_india_annual_1997().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturvimsamsa),
        [
            Aries,
            Taurus,
            Sagittarius,
            Capricorn,
            Capricorn,
            Capricorn,
            Virgo,
            Aries,
            Taurus,
            Taurus
        ]
    );
}

/// Chart 82 (India: Solar year 1998): D-24, ch. 35, p. 467.
#[tokio::test]
async fn chart_82_india_solar_year_1998_chaturvimsamsa() {
    use Rasi::*;
    let sky = book::chart_82_india_solar_year_1998().await;

    assert_eq!(
        drawn(&sky, Varga::Chaturvimsamsa),
        [
            Virgo, Cancer, Leo, Aries, Aquarius, Virgo, Virgo, Scorpio, Scorpio, Scorpio
        ]
    );
}

/// Chart 83 (Vajpayee Swearing-in): D-9, ch. 35, p. 468.
#[tokio::test]
async fn chart_83_vajpayee_swearing_in_1998_navamsa() {
    use Rasi::*;
    let sky = book::chart_59_vajpayee_swearing_in_1998().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Aquarius,
            Leo,
            Virgo,
            Sagittarius,
            Capricorn,
            Aquarius,
            Gemini,
            Aquarius,
            Leo,
            Aquarius
        ]
    );
}

/// Chart 88 (An astrologer): D-7, ch. 36, p. 490.
#[tokio::test]
async fn chart_88_an_astrologer_1949_saptamsa() {
    use Rasi::*;
    let sky = book::chart_88_an_astrologer_1949().await;

    assert_eq!(
        drawn(&sky, Varga::Saptamsa),
        [
            Taurus, Leo, Leo, Leo, Pisces, Cancer, Gemini, Virgo, Taurus, Scorpio
        ]
    );
}

/// Chart 90 (An Indian in USA): D-10, ch. 36, p. 493.
#[tokio::test]
async fn chart_90_an_indian_in_usa_1968_dasamsa() {
    use Rasi::*;
    let sky = book::chart_90_an_indian_in_usa_1968().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Leo,
            Sagittarius,
            Scorpio,
            Leo,
            Gemini,
            Scorpio,
            Capricorn,
            Pisces,
            Leo,
            Aquarius
        ]
    );
}

/// Chart 91 (Tajaka 1998-99): D-10, ch. 36, p. 494.
#[tokio::test]
async fn chart_91_annual_1998_dasamsa() {
    use Rasi::*;
    let sky = book::chart_91_annual_1998().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Aries,
            Sagittarius,
            Aries,
            Pisces,
            Cancer,
            Pisces,
            Virgo,
            Gemini,
            Aquarius,
            Leo
        ]
    );
}

/// Chart 92 (May-June 1998): D-10, ch. 36, p. 495.
#[tokio::test]
async fn chart_92_monthly_may_1998_dasamsa() {
    use Rasi::*;
    let sky = book::chart_92_monthly_may_1998().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Taurus,
            Aries,
            Gemini,
            Pisces,
            Sagittarius,
            Scorpio,
            Aries,
            Taurus,
            Sagittarius,
            Gemini
        ]
    );
}

/// Chart 92 (June-July 1998): D-10, ch. 36, p. 495.
#[tokio::test]
async fn chart_92_monthly_june_1998_dasamsa() {
    use Rasi::*;
    let sky = book::chart_92_monthly_june_1998().await;

    assert_eq!(
        drawn(&sky, Varga::Dasamsa),
        [
            Pisces,
            Virgo,
            Gemini,
            Libra,
            Pisces,
            Sagittarius,
            Pisces,
            Gemini,
            Scorpio,
            Taurus
        ]
    );
}

/// Chart 95 (A lady): D-9, ch. 36, p. 500.
#[tokio::test]
async fn chart_95_a_lady_1968_navamsa() {
    use Rasi::*;
    let sky = book::chart_95_a_lady_1968().await;

    assert_eq!(
        drawn(&sky, Varga::Navamsa),
        [
            Pisces,
            Pisces,
            Leo,
            Taurus,
            Virgo,
            Taurus,
            Sagittarius,
            Pisces,
            Capricorn,
            Cancer
        ]
    );
}

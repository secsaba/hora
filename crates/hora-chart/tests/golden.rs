//! Facts the book shows for its example charts, derived from hora-swe's positions, and its examples and exercises on given positions (chapters 1 and 3):
//! P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*.

use hora_chart::{
    Body, Compound, LunarMonth, Nakshatra, PLANETS, Paksha, Rasi, SunMoonYoga, Tatkaala, Tithi,
    Upagraha, Vaara, bhava, chart, compound, hora, lunar_month, nakshatra, natural, rasi,
    sun_moon_yoga, tatkaala, tithi, upagrahas, vaara,
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

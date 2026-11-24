use super::*;
use crate::tests::*;

#[test]
fn age_avasthas_follow_table_35s_examples() {
    use AgeAvastha::*;
    // P. 190.
    assert_eq!(age_avastha(dms(3, 23, 0, 0.0)), Kumaara);
    assert_eq!(age_avastha(dms(6, 19, 0, 0.0)), Vriddha);
    assert_eq!(age_avastha(dms(8, 14, 0, 0.0)), Yuva);
    assert_eq!(age_avastha(dms(11, 27, 0, 0.0)), Saisava);
}

#[test]
fn age_avasthas_change_at_each_6_degrees_forward_in_odd_rasis_and_backward_in_even() {
    use AgeAvastha::*;
    let odd = [Saisava, Kumaara, Yuva, Vriddha, Mrita];
    let even = [Mrita, Vriddha, Yuva, Kumaara, Saisava];
    for (rasi, order) in [(0, odd), (1, even)] {
        for step in 0..5u8 {
            let at = dms(rasi, step * 6, 0, 0.0);
            assert_eq!(age_avastha(at), order[usize::from(step)], "{at}°");
            if step > 0 {
                let below = at - 1.0 / 3600.0;
                assert_eq!(age_avastha(below), order[usize::from(step) - 1], "{below}°");
            }
        }
        // 30° is the next rasi's 0°, whose parity is the other.
        let end = dms(rasi + 1, 0, 0, 0.0) - 1.0 / 3600.0;
        assert_eq!(age_avastha(end), order[4], "{end}°");
    }
    assert_eq!(age_avastha(dms(2, 0, 0, 0.0)), Saisava);
    assert_eq!(age_avastha(dms(2, 0, 0, 0.0) - 1.0 / 3600.0), Saisava);
}

/// `graha`'s avasthas in a [`scene`] around `rest` with the bodies in `at`, each at 15° of its rasi. Ketu starts in the 7th
/// from `rest`.
fn avasthas_in(rest: Rasi, at: &[(Body, Rasi)], graha: Body) -> Avasthas {
    let at: Vec<(Body, f64)> = at.iter().map(|&(b, r)| (b, mid(r))).collect();
    let all = scene(rest, &at).avasthas();
    all.into_iter().find(|a| a.graha == graha).unwrap()
}

#[test]
fn alertness_is_jaagrita_at_home_or_exalted_then_sushupta_debilitated_then_by_the_lords_relationship()
 {
    use Alertness::*;
    use Body::*;
    use Rasi::*;
    let alertness = |at: &[(Body, Rasi)], graha| avasthas_in(Aries, at, graha).alertness;
    // §15.4.2 (pp. 190–191).
    assert_eq!(alertness(&[(Sun, Leo)], Sun), Some(Jaagrita));
    // The Moon exalted in Ta beside Venus, satru to her by the compound relationship: still awake.
    assert_eq!(
        alertness(&[(Moon, Taurus), (Venus, Taurus)], Moon),
        Some(Jaagrita)
    );
    // Jupiter debilitated in Cp with Saturn in Aq, mitra to him: still asleep.
    assert_eq!(
        alertness(&[(Jupiter, Capricorn), (Saturn, Aquarius)], Jupiter),
        Some(Sushupta)
    );
    // The Sun in Sg with Jupiter in Cp, adhimitra; in Ge with Mercury in Cn, mitra; in Sg with Jupiter, sama.
    assert_eq!(
        alertness(&[(Sun, Sagittarius), (Jupiter, Capricorn)], Sun),
        Some(Swapna)
    );
    assert_eq!(
        alertness(&[(Sun, Gemini), (Mercury, Cancer)], Sun),
        Some(Swapna)
    );
    assert_eq!(
        alertness(&[(Sun, Sagittarius), (Jupiter, Sagittarius)], Sun),
        Some(Swapna)
    );
    // The Sun in Cp with Saturn, adhisatru; Mars in Ta with Venus, satru.
    assert_eq!(
        alertness(&[(Sun, Capricorn), (Saturn, Capricorn)], Sun),
        Some(Sushupta)
    );
    assert_eq!(
        alertness(&[(Mars, Taurus), (Venus, Taurus)], Mars),
        Some(Sushupta)
    );
    // The nodes: awake exalted or at home, asleep debilitated, else none (ADR 0018).
    assert_eq!(alertness(&[(Rahu, Gemini)], Rahu), Some(Jaagrita));
    assert_eq!(alertness(&[(Ketu, Scorpio)], Ketu), Some(Jaagrita));
    assert_eq!(alertness(&[(Rahu, Sagittarius)], Rahu), Some(Sushupta));
    assert_eq!(alertness(&[(Rahu, Leo)], Rahu), None);
}

/// `graha`'s mood avasthas in [`avasthas_in`]'s scene.
fn moods(rest: Rasi, at: &[(Body, Rasi)], graha: Body) -> Vec<MoodAvastha> {
    avasthas_in(rest, at, graha).moods
}

#[test]
fn the_nine_mood_avasthas_read_the_rasi_its_lord_and_company() {
    use Body::*;
    use MoodAvastha::*;
    use Rasi::*;
    // §15.4.3's nine (p. 191). Everything else at 15° of Capricorn, with the lagna: a Sukla Moon, a malefic Mercury.
    let m = |at: &[(Body, Rasi)], graha| moods(Capricorn, at, graha);
    let has = |at: &[(Body, Rasi)], graha, state| m(at, graha).contains(&state);
    assert!(has(&[(Sun, Aries)], Sun, Deepta));
    assert!(!has(&[(Sun, Taurus)], Sun, Deepta));
    assert!(has(&[(Sun, Leo)], Sun, Swastha));
    // The rungs, with the lords placed as in the alertness test; none in the own rasi.
    let by_lord = [Mudita, Saanta, Deena, Duhkhita];
    assert!(!m(&[(Sun, Leo)], Sun).iter().any(|s| by_lord.contains(s)));
    assert!(has(
        &[(Sun, Sagittarius), (Jupiter, Capricorn)],
        Sun,
        Mudita
    ));
    assert!(has(&[(Sun, Gemini), (Mercury, Cancer)], Sun, Saanta));
    assert!(has(
        &[(Sun, Sagittarius), (Jupiter, Sagittarius)],
        Sun,
        Deena
    ));
    assert!(has(&[(Mars, Taurus), (Venus, Taurus)], Mars, Duhkhita));
    // An exalted Sun in Aries is also adhimitra to Mars there: every state that holds is listed.
    assert!(has(&[(Sun, Aries)], Sun, Mudita));
    // Vikala by Mars beside Jupiter, not by Venus; Khala in Mars's rasi, and in Saturn's own, as literally as the book.
    assert!(has(&[(Jupiter, Leo), (Mars, Leo)], Jupiter, Vikala));
    assert!(!has(&[(Jupiter, Leo), (Venus, Leo)], Jupiter, Vikala));
    assert!(has(&[(Jupiter, Aries)], Jupiter, Khala));
    assert!(!has(&[(Jupiter, Taurus)], Jupiter, Khala));
    assert!(has(&[], Saturn, Khala));
    // Kopita beside the Sun, who never is himself (ADR 0018).
    assert!(has(&[(Jupiter, Leo), (Sun, Leo)], Jupiter, Kopita));
    assert!(!has(&[(Jupiter, Leo), (Sun, Leo)], Sun, Kopita));
}

#[test]
fn the_six_more_mood_avasthas_read_the_fifth_dignity_enemies_water_jupiter_and_the_sun() {
    use Body::*;
    use MoodAvastha::*;
    use Rasi::*;
    // §15.4.3's six (pp. 191–192).
    let has = |rest, at: &[(Body, Rasi)], graha, state| moods(rest, at, graha).contains(&state);
    // Lajjita: the lagna in Cp, so the 5th is Ta.
    assert!(has(
        Capricorn,
        &[(Jupiter, Taurus), (Mars, Taurus)],
        Jupiter,
        Lajjita
    ));
    assert!(!has(
        Capricorn,
        &[(Jupiter, Taurus), (Venus, Taurus)],
        Jupiter,
        Lajjita
    ));
    assert!(!has(Capricorn, &[(Sun, Taurus)], Sun, Lajjita));
    assert!(!has(
        Capricorn,
        &[(Jupiter, Gemini), (Mars, Gemini)],
        Jupiter,
        Lajjita
    ));
    assert!(has(
        Capricorn,
        &[(Rahu, Taurus), (Saturn, Taurus)],
        Rahu,
        Lajjita
    ));
    // Garvita by whole rasi.
    assert!(has(Capricorn, &[(Saturn, Aquarius)], Saturn, Garvita));
    assert!(!has(Capricorn, &[], Saturn, Garvita));
    assert!(has(Capricorn, &[(Rahu, Virgo)], Rahu, Garvita));
    // Kshudhita: conjoined by Saturn, friend to Venus; aspected by Venus, adhisatru to Jupiter; in the Sun's Le with
    // him adhisatru in Sg, out of sight. Never for a node.
    assert!(has(
        Pisces,
        &[(Venus, Taurus), (Saturn, Taurus)],
        Venus,
        Kshudhita
    ));
    assert!(!has(Pisces, &[(Venus, Taurus)], Venus, Kshudhita));
    assert!(!has(Pisces, &[(Saturn, Taurus)], Saturn, Kshudhita));
    assert!(has(
        Taurus,
        &[(Jupiter, Sagittarius), (Venus, Gemini)],
        Jupiter,
        Kshudhita
    ));
    assert!(!has(Taurus, &[(Jupiter, Sagittarius)], Jupiter, Kshudhita));
    assert!(has(
        Taurus,
        &[(Venus, Leo), (Sun, Sagittarius)],
        Venus,
        Kshudhita
    ));
    assert!(!has(
        Pisces,
        &[(Rahu, Taurus), (Saturn, Taurus)],
        Rahu,
        Kshudhita
    ));
    // Trishita: Jupiter in Pi aspected by Mercury, adhisatru and malefic beside Mars; not once Venus aspects too.
    let thirsty = [(Jupiter, Pisces), (Mercury, Virgo), (Mars, Virgo)];
    assert!(has(Capricorn, &thirsty, Jupiter, Trishita));
    assert!(!has(Capricorn, &[(Jupiter, Pisces)], Jupiter, Trishita));
    assert!(!has(
        Capricorn,
        &[thirsty.as_slice(), &[(Venus, Sagittarius)]].concat(),
        Jupiter,
        Trishita
    ));
    // A node needs only the water and no benefic's aspect (ADR 0018).
    assert!(has(Capricorn, &[(Ketu, Pisces)], Ketu, Trishita));
    assert!(!has(
        Capricorn,
        &[(Ketu, Pisces), (Venus, Sagittarius)],
        Ketu,
        Trishita
    ));
    assert!(!has(Capricorn, &[(Ketu, Sagittarius)], Ketu, Trishita));
    // Mudita's second form: the Sun in Mercury's Ge (mitra) aspected by Mars (adhimitra) and joined by Jupiter.
    let delighted = [(Sun, Gemini), (Mercury, Leo), (Mars, Virgo)];
    assert_eq!(
        moods(
            Capricorn,
            &[delighted.as_slice(), &[(Jupiter, Gemini)]].concat(),
            Sun
        ),
        [Mudita, Saanta]
    );
    assert_eq!(moods(Capricorn, &delighted, Sun), [Saanta]);
    assert_eq!(
        moods(
            Capricorn,
            &[delighted[0], delighted[1], (Jupiter, Gemini)],
            Sun
        ),
        [Saanta]
    );
    // Both forms at once list Mudita once: the Sun in Cn with the Moon in Le, Jupiter beside him, Mars in Ta aspecting.
    let both = moods(
        Capricorn,
        &[
            (Sun, Cancer),
            (Moon, Leo),
            (Jupiter, Cancer),
            (Mars, Taurus),
        ],
        Sun,
    );
    assert_eq!(both.iter().filter(|&&s| s == Mudita).count(), 1);
    // Kshobhita: Jupiter beside the Sun in Ar, aspected by Saturn from Le; never the Sun himself, nor a node.
    let shaken = [(Jupiter, Aries), (Sun, Aries), (Saturn, Leo)];
    assert!(has(Capricorn, &shaken, Jupiter, Kshobhita));
    assert!(!has(Capricorn, &shaken[..2], Jupiter, Kshobhita));
    // Or by an enemy who is no malefic: Venus, adhisatru to Jupiter, from Le.
    assert!(has(
        Capricorn,
        &[shaken[0], shaken[1], (Venus, Leo)],
        Jupiter,
        Kshobhita
    ));
    assert!(!has(Capricorn, &shaken, Sun, Kshobhita));
    assert!(!has(
        Capricorn,
        &[(Rahu, Aries), (Sun, Aries), (Saturn, Leo)],
        Rahu,
        Kshobhita
    ));
}

#[test]
fn a_node_has_no_relationship_states() {
    use Body::*;
    use MoodAvastha::*;
    use Rasi::*;
    let m = |at: &[(Body, Rasi)], graha| moods(Capricorn, at, graha);
    // ADR 0018: the Sun's rasi is a malefic's, and the Sun joins Rahu there.
    assert_eq!(m(&[(Rahu, Leo)], Rahu), [Khala]);
    assert_eq!(m(&[(Rahu, Leo), (Sun, Leo)], Rahu), [Vikala, Khala, Kopita]);
    assert_eq!(m(&[(Rahu, Aquarius)], Rahu), [Swastha, Khala]);
    assert_eq!(m(&[(Ketu, Sagittarius)], Ketu), [Deepta, Garvita]);
    assert_eq!(m(&[(Ketu, Taurus)], Ketu), []);
    // A node's age by its degree as it stands: 23° Cn is Kumaara, as for any graha (Table 35, p. 190).
    let rahu = scene(Capricorn, &[(Rahu, dms(3, 23, 0, 0.0))]).avasthas()[7].age;
    assert_eq!(rahu, AgeAvastha::Kumaara);
}

#[test]
fn a_sayanaadi_part_is_3_20_counted_from_the_rasis_start() {
    // Ch. 15 footnote 51 (p. 192): Mercury at 22Ge14 is in the 7th.
    assert_eq!(sayanaadi_part(dms(2, 22, 14, 0.0)), 7);
    assert_eq!(sayanaadi_part(dms(2, 0, 0, 0.0)), 1);
    assert_eq!(sayanaadi_part(dms(2, 3, 20, 0.0) - 1.0 / 3600.0), 1);
    assert_eq!(sayanaadi_part(dms(2, 3, 20, 0.0)), 2);
    assert_eq!(sayanaadi_part(dms(3, 0, 0, 0.0) - 1.0 / 3600.0), 9);
}

#[test]
fn the_ghati_running_counts_whole_24_minutes_since_the_vedic_days_sunrise_from_1() {
    let sunrise = WEDNESDAY + hours(6, 0);
    let d = day(sunrise, sunrise + 1.0);
    // Ch. 15 footnote 52 (p. 192): sunrise at 6 am, born at 11 pm, 42.5 ghatis over: the 43rd.
    assert_eq!(ghati(&d, sunrise + hours(17, 0)), 43);
    assert_eq!(ghati(&d, sunrise), 1);
    assert_eq!(ghati(&d, sunrise + hours(0, 24) - 1.0 / 86400.0), 1);
    assert_eq!(ghati(&d, sunrise + hours(0, 24)), 2);
    // Born at 5 am, before the next sunrise: 23 hours from the previous one, the 58th.
    assert_eq!(ghati(&d, sunrise + hours(23, 0)), 58);
}

#[test]
fn without_a_name_sound_the_sayanaadi_avasthas_have_no_strength() {
    let sky = timed_sky(Vaara::Thursday, false, |_, _| 0.0);
    let avasthas = |sound| {
        let all = sayanaadi_avasthas(&sky, sound).unwrap();
        all.into_iter()
            .map(|s| (s.avastha, s.strength))
            .collect::<Vec<_>>()
    };
    let (with, without) = (avasthas(Some(1)), avasthas(None));
    assert!(with.iter().all(|(_, strength)| strength.is_some()));
    assert_eq!(
        without,
        with.iter().map(|&(a, _)| (a, None)).collect::<Vec<_>>()
    );
}

#[test]
fn rahu_is_p_8_ketu_p_9_and_both_adjust_the_strength_by_4() {
    // Lagna Ar, Moon in Aswini, born at sunrise: L = M = G = 1. Rahu 0Ar (C 1, A 1), so Ketu 0Li (Chitra, C 14, A 1).
    let sky = Sky {
        jd: WEDNESDAY,
        vedic_day: Some(day(WEDNESDAY, WEDNESDAY + 1.0)),
        ..sky(0.0, [[0.0, 1.0]; 8])
    };
    let nodes: Vec<_> = sayanaadi_avasthas(&sky, Some(1))
        .unwrap()
        .into_iter()
        .filter(|s| matches!(s.graha, Body::Rahu | Body::Ketu))
        .map(|s| (s.avastha, s.strength))
        .collect();
    use ActivityStrength::*;
    use SayanaadiAvastha::*;
    // Rahu 1·8·1 + 3 = 11, ((121 + 1) mod 12 + 4) mod 3 = 0; Ketu 14·9·1 + 3 = 129 → 9, ((81 + 1) mod 12 + 4) mod 3 = 2.
    assert_eq!(
        nodes,
        [(Kautuka, Some(Vicheshta)), (Bhojana, Some(Cheshta))]
    );
}

#[test]
fn a_sky_without_a_vedic_day_has_no_sayanaadi_avasthas() {
    assert_eq!(
        sayanaadi_avasthas(&sky(0.0, [[0.0, 1.0]; 8]), Some(1)),
        None
    );
}

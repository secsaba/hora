use super::*;

/// The rasis, in zodiac order, whose `characteristic` is `value`; the book lists most characteristics so.
fn rasis_with<T: PartialEq>(characteristic: fn(Rasi) -> T, value: T) -> Vec<Rasi> {
    RASIS
        .into_iter()
        .filter(|&r| characteristic(r) == value)
        .collect()
}

#[test]
fn limbs_of_vishnu_are_section_2_2_1s() {
    use Limb::*;
    let book = [
        Head,
        Face,
        Arms,
        Heart,
        Stomach,
        Hip,
        BelowNavel,
        PrivateParts,
        Thighs,
        Knees,
        Ankles,
        Feet,
    ];
    assert_eq!(RASIS.map(Rasi::limb), book);
}

#[test]
fn odd_and_even_rasis_are_section_2_2_2s() {
    use Rasi::*;
    assert_eq!(
        rasis_with(Rasi::parity, Parity::Vishama),
        [Aries, Gemini, Leo, Libra, Sagittarius, Aquarius]
    );
    assert_eq!(
        rasis_with(Rasi::parity, Parity::Sama),
        [Taurus, Cancer, Virgo, Scorpio, Capricorn, Pisces]
    );
}

#[test]
fn odd_and_even_footed_rasis_are_section_2_2_3s() {
    use Rasi::*;
    assert_eq!(
        rasis_with(Rasi::footedness, Footedness::Vishamapada),
        [Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius]
    );
    assert_eq!(
        rasis_with(Rasi::footedness, Footedness::Samapada),
        [Cancer, Leo, Virgo, Capricorn, Aquarius, Pisces]
    );
}

#[test]
fn movable_fixed_and_dual_rasis_are_section_2_2_4s() {
    use Rasi::*;
    let with = |value| rasis_with(Rasi::mobility, value);
    assert_eq!(with(Mobility::Chara), [Aries, Cancer, Libra, Capricorn]);
    assert_eq!(with(Mobility::Sthira), [Taurus, Leo, Scorpio, Aquarius]);
    assert_eq!(
        with(Mobility::Dwiswabhava),
        [Gemini, Virgo, Sagittarius, Pisces]
    );
}

#[test]
fn elements_are_section_2_2_5s() {
    use Rasi::*;
    let with = |value| rasis_with(Rasi::element, value);
    assert_eq!(with(Element::Agni), [Aries, Leo, Sagittarius]);
    assert_eq!(with(Element::Bhoo), [Taurus, Virgo, Capricorn]);
    assert_eq!(with(Element::Vaayu), [Gemini, Libra, Aquarius]);
    assert_eq!(with(Element::Jala), [Cancer, Scorpio, Pisces]);
}

#[test]
fn pitta_vaata_and_kapha_are_section_2_2_6s() {
    use Rasi::*;
    let with = |value| rasis_with(Rasi::nature, value);
    assert_eq!(with(Nature::Pitta), [Aries, Leo, Sagittarius]);
    assert_eq!(with(Nature::Vaata), [Taurus, Virgo, Capricorn]);
    assert_eq!(with(Nature::Kapha), [Cancer, Scorpio, Pisces]);
    assert_eq!(with(Nature::Mixed), [Gemini, Libra, Aquarius]);
}

#[test]
fn gunas_are_section_2_2_7s() {
    use Rasi::*;
    let with = |value| rasis_with(Rasi::quality, value);
    assert_eq!(with(Quality::Sattwa), [Cancer, Leo, Sagittarius, Pisces]);
    assert_eq!(with(Quality::Rajas), [Aries, Taurus, Libra, Scorpio]);
    assert_eq!(with(Quality::Tamas), [Gemini, Virgo, Capricorn, Aquarius]);
}

#[test]
fn directions_are_section_2_2_8s() {
    use Rasi::*;
    let with = |value| rasis_with(Rasi::direction, value);
    assert_eq!(with(Direction::East), [Aries, Leo, Sagittarius]);
    assert_eq!(with(Direction::South), [Taurus, Virgo, Capricorn]);
    assert_eq!(with(Direction::West), [Gemini, Libra, Aquarius]);
    assert_eq!(with(Direction::North), [Cancer, Scorpio, Pisces]);
}

#[test]
fn colours_are_section_2_2_9s() {
    use Colour::*;
    let book = [
        BloodRed,
        White,
        GrassGreen,
        PaleRed,
        White,
        Variegated,
        Black,
        ReddishBrown,
        HuskOfGrass,
        Variegated,
        Brown,
        Cream,
    ];
    assert_eq!(RASIS.map(Rasi::colour), book);
}

#[test]
fn day_and_night_rasis_are_section_2_2_10s() {
    use Rasi::*;
    assert_eq!(
        rasis_with(Rasi::strong_by, StrongBy::Nishaa),
        [Aries, Taurus, Gemini, Cancer, Sagittarius, Capricorn]
    );
    assert_eq!(
        rasis_with(Rasi::strong_by, StrongBy::Divaa),
        [Leo, Virgo, Libra, Scorpio, Aquarius, Pisces]
    );
}

#[test]
fn rising_is_section_2_2_11s() {
    use Rasi::*;
    let with = |value| rasis_with(Rasi::rising, value);
    assert_eq!(
        with(Rising::Seershodaya),
        [Gemini, Leo, Virgo, Libra, Scorpio, Aquarius]
    );
    assert_eq!(
        with(Rising::Prishthodaya),
        [Aries, Taurus, Cancer, Sagittarius, Capricorn]
    );
    assert_eq!(with(Rising::Both), [Pisces]);
}

#[test]
fn varnas_are_section_2_2_12s() {
    use Rasi::*;
    let with = |value| rasis_with(Rasi::class, value);
    assert_eq!(with(Class::Brahmana), [Cancer, Scorpio, Pisces]);
    assert_eq!(with(Class::Kshatriya), [Aries, Leo, Sagittarius]);
    assert_eq!(with(Class::Vaisya), [Taurus, Virgo, Capricorn]);
    assert_eq!(with(Class::Sudra), [Gemini, Libra, Aquarius]);
}

// Each §3.2 test writes the book's list out for the nine grahas in GRAHAS's order: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu.

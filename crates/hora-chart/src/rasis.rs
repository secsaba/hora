use crate::*;

/// A limb of Vishnu, whose body the zodiac is (ch. 2 §2.2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limb {
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
}

/// Odd (male) or even (female) (ch. 2 §2.2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    Vishama,
    Sama,
}

/// Odd-footed or even-footed (ch. 2 §2.2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Footedness {
    Vishamapada,
    Samapada,
}

/// Movable, fixed or dual (ch. 2 §2.2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mobility {
    Chara,
    Sthira,
    Dwiswabhava,
}

/// Fire, earth, air, water or ether; aakaasa is in every rasi (ch. 2 §2.2.5) but is the element of only one graha, Jupiter (ch. 3 §3.2.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Agni,
    Bhoo,
    Vaayu,
    Jala,
    Aakaasa,
}

/// The Ayurvedic nature (ch. 2 §2.2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    Pitta,
    Vaata,
    Kapha,
    Mixed,
}

/// The guna (ch. 2 §2.2.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Sattwa,
    Rajas,
    Tamas,
}

/// The direction a rasi shows (ch. 2 §2.2.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    East,
    South,
    West,
    North,
}

/// The colour a rasi or graha shows; Taurus and Leo share white, Virgo and Capricorn variegated (ch. 2 §2.2.9, ch. 3 §3.2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    BloodRed,
    White,
    GrassGreen,
    PaleRed,
    Variegated,
    Black,
    ReddishBrown,
    HuskOfGrass,
    /// That of a mongoose.
    Brown,
    /// Or the colour of fish.
    Cream,
    Tawny,
}

/// Whether a rasi or graha is strong in the daytime (divaa) or at night (nishaa); Mercury always is (ch. 2 §2.2.10, ch. 3 §3.2.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrongBy {
    Divaa,
    Nishaa,
    Always,
}

/// Whether a rasi rises head first, feet first, or both (ch. 2 §2.2.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rising {
    Seershodaya,
    Prishthodaya,
    Both,
}

/// The varna (ch. 2 §2.2.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Brahmana,
    Kshatriya,
    Vaisya,
    Sudra,
}

impl Rasi {
    /// Ch. 2 §2.2.1.
    pub fn limb(self) -> Limb {
        use Rasi::*;
        match self {
            Aries => Limb::Head,
            Taurus => Limb::Face,
            Gemini => Limb::Arms,
            Cancer => Limb::Heart,
            Leo => Limb::Stomach,
            Virgo => Limb::Hip,
            Libra => Limb::BelowNavel,
            Scorpio => Limb::PrivateParts,
            Sagittarius => Limb::Thighs,
            Capricorn => Limb::Knees,
            Aquarius => Limb::Ankles,
            Pisces => Limb::Feet,
        }
    }

    /// Ch. 2 §2.2.2.
    pub fn parity(self) -> Parity {
        use Rasi::*;
        match self {
            Aries | Gemini | Leo | Libra | Sagittarius | Aquarius => Parity::Vishama,
            Taurus | Cancer | Virgo | Scorpio | Capricorn | Pisces => Parity::Sama,
        }
    }

    /// Ch. 2 §2.2.3.
    pub fn footedness(self) -> Footedness {
        use Rasi::*;
        match self {
            Aries | Taurus | Gemini | Libra | Scorpio | Sagittarius => Footedness::Vishamapada,
            Cancer | Leo | Virgo | Capricorn | Aquarius | Pisces => Footedness::Samapada,
        }
    }

    /// Ch. 2 §2.2.4.
    pub fn mobility(self) -> Mobility {
        use Rasi::*;
        match self {
            Aries | Cancer | Libra | Capricorn => Mobility::Chara,
            Taurus | Leo | Scorpio | Aquarius => Mobility::Sthira,
            Gemini | Virgo | Sagittarius | Pisces => Mobility::Dwiswabhava,
        }
    }

    /// Ch. 2 §2.2.5.
    pub fn element(self) -> Element {
        use Rasi::*;
        match self {
            Aries | Leo | Sagittarius => Element::Agni,
            Taurus | Virgo | Capricorn => Element::Bhoo,
            Gemini | Libra | Aquarius => Element::Vaayu,
            Cancer | Scorpio | Pisces => Element::Jala,
        }
    }

    /// Ch. 2 §2.2.6.
    pub fn nature(self) -> Nature {
        use Rasi::*;
        match self {
            Aries | Leo | Sagittarius => Nature::Pitta,
            Taurus | Virgo | Capricorn => Nature::Vaata,
            Cancer | Scorpio | Pisces => Nature::Kapha,
            Gemini | Libra | Aquarius => Nature::Mixed,
        }
    }

    /// Ch. 2 §2.2.7.
    pub fn quality(self) -> Quality {
        use Rasi::*;
        match self {
            Cancer | Leo | Sagittarius | Pisces => Quality::Sattwa,
            Aries | Taurus | Libra | Scorpio => Quality::Rajas,
            Gemini | Virgo | Capricorn | Aquarius => Quality::Tamas,
        }
    }

    /// Ch. 2 §2.2.8.
    pub fn direction(self) -> Direction {
        use Rasi::*;
        match self {
            Aries | Leo | Sagittarius => Direction::East,
            Taurus | Virgo | Capricorn => Direction::South,
            Gemini | Libra | Aquarius => Direction::West,
            Cancer | Scorpio | Pisces => Direction::North,
        }
    }

    /// Ch. 2 §2.2.9.
    pub fn colour(self) -> Colour {
        use Rasi::*;
        match self {
            Aries => Colour::BloodRed,
            Taurus | Leo => Colour::White,
            Gemini => Colour::GrassGreen,
            Cancer => Colour::PaleRed,
            Virgo | Capricorn => Colour::Variegated,
            Libra => Colour::Black,
            Scorpio => Colour::ReddishBrown,
            Sagittarius => Colour::HuskOfGrass,
            Aquarius => Colour::Brown,
            Pisces => Colour::Cream,
        }
    }

    /// Ch. 2 §2.2.10.
    pub fn strong_by(self) -> StrongBy {
        use Rasi::*;
        match self {
            Leo | Virgo | Libra | Scorpio | Aquarius | Pisces => StrongBy::Divaa,
            Aries | Taurus | Gemini | Cancer | Sagittarius | Capricorn => StrongBy::Nishaa,
        }
    }

    /// Ch. 2 §2.2.11.
    pub fn rising(self) -> Rising {
        use Rasi::*;
        match self {
            Gemini | Leo | Virgo | Libra | Scorpio | Aquarius => Rising::Seershodaya,
            Aries | Taurus | Cancer | Sagittarius | Capricorn => Rising::Prishthodaya,
            Pisces => Rising::Both,
        }
    }

    /// Ch. 2 §2.2.12.
    pub fn class(self) -> Class {
        use Rasi::*;
        match self {
            Cancer | Scorpio | Pisces => Class::Brahmana,
            Aries | Leo | Sagittarius => Class::Kshatriya,
            Taurus | Virgo | Capricorn => Class::Vaisya,
            Gemini | Libra | Aquarius => Class::Sudra,
        }
    }

    /// The seven-planet lord (Table 6's own rasis, ch. 3 §3.3); Rahu's Aquarius and Ketu's Scorpio count only in their own dignity (ADR 0003).
    pub fn lord(self) -> Body {
        use Rasi::*;
        match self {
            Aries | Scorpio => Body::Mars,
            Taurus | Libra => Body::Venus,
            Gemini | Virgo => Body::Mercury,
            Cancer => Body::Moon,
            Leo => Body::Sun,
            Sagittarius | Pisces => Body::Jupiter,
            Capricorn | Aquarius => Body::Saturn,
        }
    }
}

#[cfg(test)]
mod tests;

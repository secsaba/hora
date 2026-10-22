# hora

Vedic (Jyotish) astrology as taught in P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*.

## Language

**Graha**:
One of the nine bodies a chart places: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu and Ketu.
_Avoid_: Planet (Rahu and Ketu are not planets)

**Rahu**:
The Moon's north node, treated as a graha.

**Ketu**:
The Moon's south node, always exactly opposite Rahu.

**Node type**:
Whether Rahu and Ketu are placed at the mean or the true lunar node; mean by default.

**Ayanamsha**:
The angle between the tropical and the sidereal zodiac at a given moment; subtracting it from a tropical longitude gives the sidereal one. hora offers Raman, True Chitra (the default), True Mula and True Pushya.
_Avoid_: Ayanamsa, precession

**Calendar**:
Whether a birth date is given in the Gregorian (the default, from 1582-10-15) or the Julian calendar. The caller states it, since countries switched at different dates.

**Sunrise definition**:
Which point of the Sun's disc must be on the horizon (centre or upper limb), and whether that is the true or the apparent position (refraction ignored or included). The default is the upper limb, apparent.
_Avoid_: Hindu rising (it names only one of the four)

**Vedic day**:
The day from one sunrise to the next. A birth before sunrise belongs to the previous civil date's Vedic day. Near the poles a birth has none when the sunrise before it or the one after it is missing (polar night, midnight sun).
_Avoid_: Day (ambiguous with the civil date)

**Lagna**:
The sidereal longitude of the eastern horizon's intersection with the ecliptic at the moment and place of birth.
_Avoid_: Ascendant, rising sign

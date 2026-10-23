# hora

Vedic (Jyotish) astrology as taught in P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*. Sanskrit terms are spelled as in the book.

## Language

**Graha**:
One of the nine bodies a chart places: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu and Ketu.
_Hungarian_: graha
_Avoid_: Planet (Rahu and Ketu are not planets)

**Rahu**:
The Moon's north node, treated as a graha.
_Hungarian_: Rahu

**Ketu**:
The Moon's south node, always exactly opposite Rahu.
_Hungarian_: Ketu

**Node type**:
Whether Rahu and Ketu are placed at the mean or the true lunar node; mean by default.
_Hungarian_: holdcsomó típusa

**Ayanamsa**:
The angle between the tropical and the sidereal zodiac at a given moment; subtracting it from a tropical longitude gives the sidereal one. hora offers Raman, True Chitra (the default), True Mula and True Pushya.
_Hungarian_: ayanamsa
_Avoid_: Ayanamsha, precession

**Rasi**:
One of the twelve 30° parts of the sidereal zodiac, Aries to Pisces.
_Hungarian_: rasi
_Avoid_: Rashi, sign, zodiac sign

**Retrograde**:
The apparent backward motion of Mars, Mercury, Jupiter, Venus or Saturn. Rahu and Ketu move backward by nature and are never called retrograde.
_Hungarian_: retrográd

**Calendar**:
Whether a birth date is given in the Gregorian (the default, from 1582-10-15) or the Julian calendar. The caller states it, since countries switched at different dates.
_Hungarian_: naptár

**Sunrise definition**:
Which point of the Sun's disc must be on the horizon (centre or upper limb), and whether that is the true or the apparent position (refraction ignored or included). The four are the options of the author's software, Jagannatha Hora. The default is the upper limb, apparent.
_Hungarian_: napkelte-meghatározás
_Avoid_: Hindu rising (the library's Hindu rising is the centre, true and seen from the Earth's centre, none of the four)

**Vedic day**:
The day from one sunrise to the next. A birth before sunrise belongs to the previous civil date's Vedic day. Near the poles a birth has none when the sunrise before it or the one after it is missing (polar night, midnight sun).
_Hungarian_: védikus nap
_Avoid_: Day (ambiguous with the civil date)

**Lagna**:
The sidereal longitude of the eastern horizon's intersection with the ecliptic at the moment and place of birth.
_Hungarian_: lagna
_Avoid_: Ascendant, rising sign

**Sky**:
What the ephemeris gives for one moment and place: the grahas' sidereal longitudes and speeds, the lagna, the ayanamsa and the Vedic day. A chart is prepared from it.
_Hungarian_: égbolt
_Avoid_: Positions (it holds more than positions), chart data

**Chart**:
The twelve rasis, each with the grahas and lagnas that occupy it, usually drawn as twelve boxes. Its Sanskrit name is chakra.
_Hungarian_: chakra
_Avoid_: Horoscope, kundli, képlet

## Birth data

**Birth data**:
The birth date, birth time, UTC offset and birth place from which a chart is prepared. It never includes a name.
_Hungarian_: születési adatok
_Avoid_: Birth details

**Birth time**:
The local clock time of birth, as recorded.
_Hungarian_: születési idő
_Avoid_: Birth moment

**Birth place**:
The latitude, longitude and altitude of the place of birth.
_Hungarian_: születési hely

**UTC offset**:
How far the birth time is ahead of UT: a stated amount, or derived from the birth place when the birth time is in local mean time or local apparent time.
_Hungarian_: UTC-eltérés
_Avoid_: Time zone (a zone is only a way to find the offset)

**Local mean time**:
Clock time set by the mean Sun at the birth place's longitude: UT plus four minutes per degree east. Many births before standard time were recorded in it.
_Hungarian_: helyi középidő
_Avoid_: LMT, local time

**Local apparent time**:
Clock time set by the true Sun at the birth place, as a sundial shows it: local mean time plus the equation of time, which varies over the year by up to about 16 minutes. Some old records, especially Indian ones, give it.
_Hungarian_: helyi valódi idő
_Avoid_: LAT, sundial time

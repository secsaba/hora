---
status: accepted
---

# The calculation choices hora offers, and their defaults

A chart depends on choices the book makes once for all its examples but other schools make differently. hora lets the user pick each one; the default is whatever reproduces the book's example charts, since those are the golden tests.

## Decision

- **Calendar:** Gregorian (default) or Julian. The user states it, since countries switched at different dates.
- **Ayanamsa:** Lahiri (default), True Chitra, Raman, True Pushya, True Mula. The book names its ayanamsa "Chitrapaksha/Lahiri" (ch. 1, §1.3.13), and its printed example charts (charts 1–4 and 10, to the arcminute) match the library's Lahiri: every graha within a minute (chart 10, from 1542, within 100″), where True Chitra is 1–1.8′ ahead throughout. True Chitra was the default until the golden tests were found to pin Jagannatha Hora's values, which match True Chitra, rather than the book's.
- **Node type:** mean (default) or true.
- **Chart style:** South Indian (default), North Indian, East Indian: the three of ch. 1, §1.3.4. A display choice only; it never changes a calculation.
- **Reference point:** the lagna (default; the book counts bhavas from it when no reference point is stated, ch. 1, §1.3.3) or any graha; special lagnas join when hora computes them (ch. 5). A display choice only, like chart style.
- **Sunrise definition:** upper limb, apparent (default; the book recommends the first visible ray, ch. 5, p. 48), or centre/upper limb with true/apparent position: the four options of the author's software, Jagannatha Hora.
- **Lunar month:** amanta only, from one new Moon to the next, as the book defines it (ch. 1, §1.3.8.2); purnimanta months (full Moon to full Moon) are not offered. Of two months started in the same rasi the first is Adhika and the second Nija, the common rule, since the book does not say.
- **Rahu's and Ketu's dignities:** the book's only (ch. 3, Table 6): Rahu exalted in Gemini, Ketu in Sagittarius. Schools that exalt them in Taurus and Scorpio are not offered.
- **Upagrahas:** Jagannatha Hora's defaults only: Gulika rises at the beginning of Saturn's part and Maandi at its middle, Kaala, Mrityu, Arthaprahaara and Yamaghantaka at the middle of their parts, Vyatipaata is 360° − Dhuma, and day and night start at sunrise and sunset. Ch. 4 §4.3 prints Gulika and Maandi the other way round, but the book's own Example 142 (Chart 85, "Gulika within 1º from lagna", ch. 36, p. 478) fits only Gulika at the beginning, so §4.3 is taken as a misprint. Footnote 9's "beginning of each part" and other Dhuma–Vyatipaata relations are not offered.

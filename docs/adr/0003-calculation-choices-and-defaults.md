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
- **Sunrise definition:** upper limb, apparent (default; the book recommends the first visible ray, ch. 5, p. 48), or centre/upper limb with true/apparent position: the four options of the author's software, Jagannatha Hora.

---
status: accepted
---

# A body's place in a varga chakra is a rasi, not a longitude

Ch. 6 §6.2 places a body in each varga chakra by the part of its rasi it falls in, and gives the rasi that part maps to. Some software (Jagannatha Hora among them) also gives a "varga longitude": the body's place within its part stretched to a whole rasi, so a graha has degrees in the navamsa too. hora gives a rasi only.

## Evidence

- The book never uses degrees within a varga. §6.2 and every worked example (Examples 11–27) give rasis; its varga chakra drawings show no degrees.
- Amsabala (§6.6) counts the vargas in which a graha occupies its uchcha, moolatrikona or own rasi, by whole rasi: Example 27 counts Jupiter in Sagittarius as moolatrikona without §3.3's degree split.
- The dasas that reach the vargas use rasis too. Vimsottari dasa starts from the natal Moon's nakshatra and is only read in a varga ("the period of the 7th lord in D-9", ch. 16). Narayana dasa of a varga takes a rasi in that varga as its lagna (ch. 18, p. 245).

## Decision

- `hora-chart` places any longitude in any varga as a rasi. Which part a body falls in is decided by its degrees within its rasi in whole arcseconds, truncated, as for dignities: part = ⌊arcseconds × n / 108 000⌋, and D-30 compares arcseconds with its printed degree marks.
- The varga chakras draw no degrees; nothing on the chart page states a varga longitude.
- Nothing extra is stored. The longitude a varga is computed from stays in the chart, so a later chapter that needed degrees within a varga would add a second function from the same longitude.

## Considered options

- **Compute a varga longitude as well:** it would be displayed and tested with nothing in the book to check it against, and it invites using a degree the book never reads.
- **Store each varga position on the chart:** every varga is a cheap pure function of the longitude; storing it would only duplicate it.

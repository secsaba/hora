---
status: accepted
---

# The book's printed hora and ghati lagnas are not golden values

Almost every chart table in the book prints the hora lagna (HL) and ghati lagna (GL). Golden tests normally pin what the book prints, but these values fit no sunrise hora can reproduce. So hora keeps the sunrise its text recommends (ch. 5 §5.5, ADR 0003) and pins HL and GL another way.

## Evidence

For each of the 72 chart tables hora can recompute (70 charts; chart 15 prints twins and chart 92 two months), the printed HL and the printed GL each imply how long after sunrise the birth was. HL moves 30° an hour and GL 30° every 24 minutes. On every chart the two agree within about 2 s, and the book's GL error is about 2.5 times its HL error. So the printed sunrise moment differs from hora's, and nothing else does. The formula, the birth time and the Sun's longitude are not the cause, and neither is the ayanamsa: True Chitra moves the Sun about 1′, which would move HL and GL by the same arcminutes.

- The book's sunrise is 0 to 256 s later than hora's upper limb with refraction. 18 charts fit within ±5 s; GL moves 1¼′ per second of time.
- No sunrise definition fits all the charts. Of `swe_rise_trans`'s four (upper limb or centre, with or without refraction), each also with `SE_BIT_GEOCTR_NO_ECL_LAT` or `SE_BIT_HINDU_RISING`, the best is hora's default (18 charts within ±5 s). Centre with refraction and upper limb without refraction each fit 1 chart, and centre without refraction none, so even a mix of definitions from chart to chart leaves most charts unexplained.
- The implied altitude of the Sun's centre at the book's sunrise runs from −49′ to −11′.
- The gap grows with latitude and with how much earlier sunrise comes each day: Ireland 53° N in spring 255 s, Linz 48° N 205 s, Seattle 47.6° N in autumn 40 s, India 0–87 s. That pattern looks like an approximate sunrise routine in the software that printed the charts, but it was not confirmed.

Even rasi-level pins fail on some charts. Chart 7's GL is printed 29 Sg 42, while hora puts it at 2 Cp 40.

## Decision

- hora keeps its sunrise definitions and default (ADR 0003). HL, GL and the bhaava lagna count from the Vedic day's sunrise at the exact moment.
- No golden test pins a printed HL or GL from a book chart. They are pinned by:
  - ch. 5's worked Examples 7–9 and Exercise 8, as ordinary tests of the formula (they print the sunrise and the Sun's longitude at sunrise);
  - an ordinary test that HL and GL count from `VedicDay::sunrise`, so a birth before sunrise uses the previous day's.
- The sree lagna depends on no sunrise; its printed values are golden as usual.
- A Jagannatha Hora check of the charts outside ±5 s is a `ready-for-human` issue. If today's Jagannatha Hora gives hora's values, the book's come from an older version, and this ADR cites that; if it gives the book's, the cause is reopened.

## Considered options

- **Pin the printed values with a tolerance:** the gap reaches about 5° of GL, so the test would check nothing.
- **Reproduce the book's sunrise as another definition:** the routine is unknown, the fit is only empirical (about ±15 s), and §5.5 recommends the visible upper limb.
- **Pin at rasi level:** still fails where a printed value is near a rasi's edge.

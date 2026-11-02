---
status: accepted
---

# Eight chara karakas, ranked in whole arcseconds

Ch. 8 §8.2 assigns the chara karakas by ranking the seven planets and Rahu by their advancement in their rasis: Rahu's is measured back from the end of his rasi, and the most advanced is the atma karaka. Other schools, and software that follows them, use seven chara karakas (no Rahu, no pitri karaka) or let the user choose. hora uses the book's eight only.

## Evidence

- §8.2 and Table 13 name eight karakas, AK to DK, and both worked answers (Table 14, Table 17) rank Rahu at 30° minus his degrees.
- The book's later chapters name chara karakas of six charts pinned in `hora-swe`'s golden tests (charts 3, 7, 8, 21, 53, 88); each agrees with the eight-karaka ranking of their pinned positions.
- §8.2 compares degrees, then minutes, then seconds, and calls only an exact tie a shared karakatwa.

## Decision

- Chara karakas come from the rasi chart's longitudes (D-1) and are the same in every varga chakra: a varga has no degrees (ADR 0009).
- Advancement is the graha's place in its rasi in whole arcseconds; Rahu's is 108 000″ minus his, from the node type the user chose, so Rahu at exactly 0° of a rasi ranks first.
- Grahas with the same advancement share a karakatwa, and the karakatwa after theirs is left empty. The book fills it with the corresponding sthira karaka; hora shows none until sthira karakas are computed (they need ch. 15's strength).
- No seven-karaka scheme and no choice between the two.

## Considered options

- **Offer the seven-karaka scheme as a choice:** every result built on chara karakas (raja yogas in ch. 11, Maheswara in ch. 14, the dasas) would need a second meaning the book never uses or tests.
- **Rank at full floating-point precision:** an exact tie would never happen, and hora would rank grahas the book calls equal.

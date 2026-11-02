---
status: accepted
---

# A bhava is a whole rasi

Ch. 7 §7.5 names the house-division methods other books teach: the bhaava chakra or chalit chakra, with the lagna's longitude as the middle of the 1st house; the equal house method, a 30° arc centred on the lagna; and Sripati's. In all of them a house can start in one rasi and end in the next. The author recommends none: each rasi is a bhava, the rasi holding the reference point is the 1st. hora counts bhavas that way only and offers no house system.

## Evidence

- §7.5: Parasara counts houses by rasis from the reference point, and only that works in the varga chakras, which have no degrees (ADR 0009) and which the book reads as charts of their own (§7.1).
- §7.1 and §7.3 count bhavas from many reference points (the Moon, the Sun, paaka lagna, the special lagnas, any graha) in every varga chakra; the division methods count only from the lagna in the rasi chart.

## Decision

- A bhava is `hora_chart::bhava(reference rasi, rasi)`: a whole rasi, from any reference point, in any chart. No cusps or house longitudes are computed.
- No house-system choice on the home page, and no bhava chalit chakra.

## Considered options

- **Offer the bhaava chakra, equal houses or Sripati as a choice, as most software does:** every bhava the app shows (the Bhava columns, the North Indian drawings, later the dasas and yogas built on bhavas) would then need a second meaning the book rejects and never tests.

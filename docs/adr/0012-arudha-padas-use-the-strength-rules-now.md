---
status: accepted
---

# Arudha padas use the strength rules ahead of chapter 15

Ch. 9 §9.2 and §9.5 need "the stronger" of two co-lords and of the two rasis a graha owns, and defer the rules to ch. 15 §15.5, which use rasi drishti from ch. 10 §10.3. hora computes the stronger lord (§15.5.1, its rules 1–4 and 5b) and the stronger rasi (§15.5.2, its rules 1–6) now, and works through ch. 10 and ch. 15 later. This differs from sthira karakas, which wait for ch. 15 (ADR 0011): those only fill a gap left by a tie, while an arudha pada has no value at all without these rules.

## Evidence

- The seven-planet lord alone is wrong: chart 3's AL is in Capricorn only with Ketu as Scorpio's lord (p. 265); Mars would put it in Virgo.
- A throwaway implementation of the decisions below reproduced all of Examples 29–30, Exercises 12–13 and 25–26, and every one of the 48 arudha padas it was tested on from the rest of the book, using its printed positions to the arcminute in the D-1, D-9, D-10, D-12, D-16, D-20 and D-24 (`docs/book-charts.md` lists them).

## Decision

Where the book is explicit, hora follows it. Where it is silent or contradicts itself, these readings apply:

- **What counts as a planet:** the nine grahas, Rahu and Ketu included; the lagna and special lagnas do not ("Vi is empty" with the lagna in it, "Cp has Ketu", Example 30).
- **Roles in rule 2:** Jupiter, Mercury and the lord or dispositor are three roles, so Mercury as dispositor counts twice (§15.5.1's worked example). A co-lord does not join itself as its own dispositor, which the book does not say.
- **The lord of Scorpio or Aquarius in rule 2:** both co-lords count (Exercise 26(5)), which also keeps the rule from depending on its own answer.
- **Exalted in rule 3:** by whole rasi, with Rahu in Gemini and Ketu in Sagittarius (ADR 0003), since a varga has no degrees (ADR 0009).
- **Advancement in rule 5b and rule 6:** from the body's longitude in the rasi chart, in whole arcseconds, in every varga (ADR 0009, ADR 0011), Rahu's and Ketu's measured back from the rasi's end.
- **A graha arudha's stronger rasi:** all of §15.5.2 applies. Its note under rule 4 says that rule always settles a graha's two rasis, but the note under rule 6 gives Scorpio and Aquarius their stronger lord, and then rule 4 can tie; rules 5 and 6 settle it.
- **A tie after the last rule:** the seven-planet lord, or the first rasi counted from Aries. The book gives no rule; this is marked `shortcut:` in the code.
- **Only what arudha padas need:** §15.5.1's rule 5a (dasa length) waits for the dasas. The paaka lagna and the Rasi lord column keep the seven-planet lord (ADR 0003).

## Considered options

- **The seven-planet lord until ch. 15:** simple, but wrong on the book's own charts, and those wrong rasis would be displayed.
- **Only rule 1, the planet count:** decides nothing when the counts are equal, as with Venus's two empty rasis in Example 30, which the book settles for Libra.
- **Treat a graha's own rasis as having that graha for lord:** makes §15.5.2's rule 4 note true, but contradicts its rule 6 note and Exercise 26(5), and would need a second rule for ch. 18's Narayana dasa.

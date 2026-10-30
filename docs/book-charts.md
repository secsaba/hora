# The book's charts

P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*, numbers 96 charts. This index says which are distinct births, which show a birth again, and which cannot be pinned. Pages are the book's printed page numbers (the PDF page is 12 more).

A chart is unique when its printed birth data (date, time, offset, place) differs from every earlier chart's. Each unique chart has a golden test in `crates/hora-swe/tests/golden.rs`, and one in `crates/hora-chart/tests/golden.rs` if the book marks a graha (R). Some charts print a second chart with its own data under the same number; those are pinned too.

Every varga chakra the book draws for a pinned birth is pinned in `crates/hora-chart/tests/golden.rs`: the rasi of the lagna and of each graha in that varga, against the drawing (#160), including the natal vargas drawn beside transits (charts 63, 65). Chart 16 (Satyam Gaur) is ignored: hora's Rahu is 0.15″ past a D-27 part boundary the book draws it before.

## Unique charts

- Chart 1 (ch. 9, p. 87): rasi, arudha example.
- Chart 2 (ch. 9, p. 89): D-16 drawn; chart 1's birth at UTC−5.
- Chart 3 (ch. 9, p. 97): rasi of A.B. Vajpayee.
- Chart 4 (ch. 9, p. 98): rasi of an engineer in the US.
- Chart 5 (ch. 10, p. 101): Navamsa drawn, rasi table; an aspects exercise.
- Chart 6 (ch. 10, p. 107): rasi of P.V. Narasimha Rao.
- Chart 7 (ch. 10, p. 108): rasi of Ronald Reagan. Repeated by Chart 34 (ch. 19, p. 261: rasi again).
- Chart 8 (ch. 10, p. 109): rasi, a livelihood exercise.
- Chart 9 (ch. 11, p. 128): rasi and Navamsa of Chatrapati Shivaji; printed as a tithi and 12:05 after sunrise, pinned as 19 Feb 1630 (Julian) 18:24:48 LMT.
- Chart 10 (ch. 11, p. 136): rasi of Emperor Akbar; birth data in footnote 39.
- Chart 11 (ch. 12, p. 152): Mercury's ashtakavarga; chart 6's native, time printed as 1:08 pm IST instead of 12:49 pm at 5:17 East.
- Chart 12 (ch. 12, p. 155): D-10 drawn, rasi table; an ashtakavarga exercise.
- Chart 13 (ch. 13, p. 173): rasi and D-20 of a renunciate.
- Chart 14 (ch. 13, p. 175): D-3 of Rajiv Gandhi (seconds of the time in footnote 37), and a rasi of his younger brother with no birth data. Repeated by Chart 39 (ch. 22, p. 275: rasi), Chart 60 (ch. 25, p. 334: D-10, with a transit, no data).
- Chart 15 (ch. 13, p. 177): D-24 of twins Satyam and Shivam Gaur, two minutes apart; both pinned. Repeated by Chart 16 (ch. 13, p. 178: D-27).
- Chart 17 (ch. 16, p. 216): D-10 of Sanjay Rath. Repeated by Chart 72 (ch. 31, p. 408: rasi, a Sudarsana chakra example).
- Chart 18 (ch. 16, p. 217): D-7 of a lady.
- Chart 19 (ch. 16, p. 219): rasi of Navin Patnaik.
- Chart 20 (ch. 16, p. 220): rasi of an engineer in the USA.
- Chart 21 (ch. 16, p. 222): rasi of John F. Kennedy Jr.
- Chart 22 (ch. 16, p. 223): rasi of a lady.
- Chart 23 (ch. 16, p. 224): rasi of B.V. Raman.
- Chart 24 (ch. 18, p. 240): rasi of Bill Gates.
- Chart 25 (ch. 18, p. 242): rasi of India's independence. Repeated by Chart 80 (ch. 35, p. 463: D-24, with an annual chart 1997–98, pinned).
- Chart 26 (ch. 18, p. 243): rasi, a Narayana dasa exercise.
- Chart 27 (ch. 18, p. 246): D-4, a foreign-stay example. Repeated by Chart 33 (ch. 18, p. 253: D-16), Chart 47 (ch. 24, p. 301: D-24), Chart 63 (ch. 25, p. 342: D-4, with transits, no time), Chart 64 (ch. 25, p. 343: rasi, with a D-4 transit, no table), Chart 65 (ch. 25, p. 345: D-24, with a transit, no table).
- Chart 28 (ch. 18, p. 248): Navamsa of a lady.
- Chart 29 (ch. 18, p. 249): Navamsa of a lady.
- Chart 31 (ch. 18, p. 251): D-10 of an electrical engineer.
- Chart 32 (ch. 18, p. 252): D-10 of a gentleman.
- Chart 35 (ch. 20, p. 266): rasi of Jayalalita.
- Chart 37 (ch. 21, p. 270): rasi of an ISKCON devotee.
- Chart 38 (ch. 21, p. 271): rasi of Sri Aurobindo.
- Chart 40 (ch. 22, p. 277): rasi of a male.
- Chart 41 (ch. 22, p. 278): rasi of a gentleman.
- Chart 42 (ch. 22, p. 279): rasi, a Niryaana Shoola exercise.
- Chart 43 (ch. 23, p. 284): rasi, a Shoola dasa exercise.
- Chart 44 (ch. 23, p. 286): rasi, a Pitri Shoola example; chart 26's birth data at 9:22 instead of 9:20. Repeated by Chart 46 (ch. 24, p. 300: Navamsa).
- Chart 48 (ch. 24, p. 302): D-12 of a gentleman.
- Chart 49 (ch. 24, p. 304): D-20 of chart 37's native, time printed 10:43 instead of 10:44; positions fit True Chitra within 1′, not Lahiri.
- Chart 50 (ch. 24, p. 306): rasi and Navamsa of a divorced lady; positions fit True Chitra within 1′, not Lahiri. Repeated by Chart 95 (ch. 36, p. 500: Navamsa; its table is about 2′ lower and fits Lahiri, so it is pinned as well).
- Chart 51 (ch. 24, p. 307): D-10 of an actor; positions fit True Chitra within 1′, not Lahiri.
- Chart 53 (ch. 25, p. 321): rasi of a lady, with her wedding transit (no time). Repeated by Chart 74 (ch. 31, p. 415: Navamsa, with an annual chart 1998–99, pinned), Chart 87 (ch. 36, p. 488: rasi).
- Chart 54 (ch. 25, p. 324): Navamsa of an engineer, with his wedding transit (pinned).
- Chart 55 (ch. 25, p. 326): D-7 of a gentleman, with a childbirth transit (pinned).
- Chart 56 (ch. 25, p. 327): D-11 of John F. Kennedy Jr., chart 21's birth at 38 N 53 instead of 38 N 54, with his death transit (pinned). Repeated by Chart 57 (ch. 25, p. 328: rasi, with the same death transit).
- Chart 58 (ch. 25, p. 331): D-10 of A.B. Vajpayee, chart 3's birth at 26 N 13 instead of 26 N 14.
- Chart 59 (ch. 25, p. 332): rasi of Vajpayee's 1998 swearing-in. Repeated by Chart 83 (ch. 35, p. 468: Navamsa), Chart 85 (ch. 36, p. 477: rasi again).
- Chart 61 (ch. 25, p. 336): rasi of Indira Gandhi, with her assassination transit (pinned). Repeated by Chart 73 (ch. 31, p. 413: rasi, with an annual chart 1976–77, pinned).
- Chart 62 (ch. 25, p. 339): rasi of Princess Diana.
- Chart 66 (ch. 27, p. 368): Tajaka annual chart 2000–01 (seconds of the time in Example 118).
- Chart 67 (ch. 30, p. 397): Tajaka annual chart 1993–94 of chart 18's native (seconds in Example 122).
- Chart 68 (ch. 30, p. 403): D-4 of an annual chart 1991–92 of chart 27's native (seconds in Example 123).
- Chart 69 (ch. 30, p. 404): D-24 of an annual chart 1987–88 of chart 27's native (seconds in Example 124).
- Chart 70 (ch. 30, p. 405): D-7 of an annual chart 1998–99 of chart 27's native (seconds in Example 125).
- Chart 71 (ch. 30, p. 407): D-16 of an annual chart 1995–96 of chart 27's native (seconds in Exercise 49's answer).
- Chart 75 (ch. 32, p. 429): D-4 and Navamsa of a rectification example. Repeated by Chart 76 (ch. 32, p. 430: D-10 and D-24).
- Chart 77 (ch. 32, p. 432): rasi of a gentleman, a rectification example.
- Chart 78 (ch. 32, p. 435): rasi of Bill Gates at a wrong time, chart 24's birth at 9:15 instead of 9:18.
- Chart 79 (ch. 32, p. 436): D-10 of a music teacher.
- Chart 81 (ch. 35, p. 465): rasi of India's lunar year 1992–93.
- Chart 82 (ch. 35, p. 467): D-24 of India's solar year 1998.
- Chart 84 (ch. 36, p. 476): rasi of Vajpayee's 1996 swearing-in.
- Chart 86 (ch. 36, p. 479): rasi of the muhurta of starting this book.
- Chart 88 (ch. 36, p. 490): D-7 of an astrologer.
- Chart 89 (ch. 36, p. 491): Tajaka annual chart 1999–2000 of chart 88's native.
- Chart 90 (ch. 36, p. 493): D-10 of an Indian in the USA.
- Chart 91 (ch. 36, p. 494): D-10 of chart 90's annual chart 1998–99.
- Chart 92 (ch. 36, p. 495): D-10 of two monthly charts of chart 90's native, May–June and June–July 1998; both pinned.
- Chart 93 (ch. 36, p. 497): rasi of a monthly chart, March–April 2000.
- Chart 94 (ch. 36, p. 498): rasi of a sixty-hour chart, March 26–28, 2000.
- Chart 96 (ch. 36, p. 501): Tajaka annual chart 1991–92 of chart 50's native.

## Unnumbered examples

- Example 27 (ch. 6, p. 65): Bill Cosby, 12 July 1937, 12:30 am at UTC−5, 75 W 10, 39 N 57; no chart is drawn, but Jupiter's rasi in sixteen vargas and his amsabala are printed. Pinned at the printed offset.

## Hora and ghati lagnas

Nearly every chart table prints HL and GL, but they are not pinned (ADR 0008): they fit a sunrise 0–256 s later than hora's upper limb with refraction, and no sunrise definition fits every chart. Within ±5 s: charts 10, 14, 17, 19, 20, 23, 25, 28, 29, 31, 32, 38, 48, 53, 61, 75, 77, 79. Every other chart table with birth data is outside, most of all at high latitudes (charts 88 and 89, Ireland, about 255 s).

## Skipped

- Chart 30 (ch. 18, p. 250): only the birth date is printed; no time, zone or place.
- Chart 36 (ch. 21, p. 268): no birth data printed anywhere.
- Chart 45 (ch. 23, p. 287): only the birth year is printed, and no position table.
- Chart 52 (ch. 25, p. 318): transit chart with only a date; no time, place or position table.

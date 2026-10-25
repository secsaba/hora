# Time zones and historical UTC offsets for hora

Researched 2026-10-25, by web search with each claim checked against its source; tzdb facts were also read from the tzdb source files (`europe`, `asia`, `backward`, `backzone` on [eggert/tz](https://github.com/eggert/tz) `main`) that day. It deepens §2.5 of [rust-stack-and-wasm.md](rust-stack-and-wasm.md); the decision is [ADR 0005](../adr/0005-utc-offset-from-tzdb.md).

The question: hora turns a birth time into UT. When the user does not state the UTC offset, where does a correct historical offset come from, including before 1970?

## Recommendation summary

| Area | Pick | Rejected |
|---|---|---|
| Offset data | **IANA tzdb**, plain build (no backzone), the nixpkgs `tzdata` pinned by `flake.lock` | Commercial astrology atlas (ACS); tzdb with backzone |
| Rust access | **jiff** reading that tzdata through `TZDIR` | chrono-tz (data compiled in, update needs a crate release) |
| Place to zone | **The IANA zone name on each GeoNames `cities500` row** | tzf-rs (only needed for typed coordinates; deferred) |
| Before 1970 | **Show the zone and offset used, warn, let the user state the offset or choose local mean or local apparent time** | Trusting any database silently |

No free, AGPL-compatible source gives reliable pre-1970 offsets everywhere, and the commercial atlases are built largely on the same Shanks data that tzdb distrusts. So the user's override is part of the design, not a fallback.

## 1. What astrology sites use

- **Astrodienst (astro.com)**, the reference atlas in Western astrology, uses tzdb mainly for 1970 onward. For older dates it adds Thomas Shanks' ACS International and American Atlases, Doane's *Time Changes* books, French and Russian references and its own research ([Astrodienst atlas help](https://www.astro.com/atlas?nhor=28)).
- Its founder, Alois Treindl, checked about 24.7M birth records against tzdb in 2020: about 96% covered correctly, 3% correct but resting on unreliable data, and about 1% wrong. Most errors were before 1970, worst in the USA and Canada, where towns set their own DST rules until the 1967 Uniform Time Act ([tz list, 2020-06](https://mm.icann.org/pipermail/tz/2020-June/029071.html)). "Correct" there means agreeing with Astrodienst's own atlas, not an independent check.
- **deva.guru** stores an IANA zone name per chart: a 1938 St. Louis chart shows "Timezone: America/Chicago", rendered CST −06:00 ([example chart](https://deva.guru/public/charts/Downing%2C_Mary_B.)). That suggests tzdb, not a commercial atlas. It rests on one page; the site documents no method.
- Not researched: Jagannatha Hora, Kala, Parashara's Light, AstroSage, Solar Fire.

## 2. tzdb before 1970

- tzdb records pre-1970 history only for the city a zone is named after; other places in that zone may be wrong. Its own docs say the data "is not designed for… accurate handling of all past times everywhere" and that "LMT offsets should not be considered meaningful" ([theory.html](https://data.iana.org/time-zones/theory.html)). hora computes local mean time from the birth place's longitude, never from tzdb's LMT.
- Much of tzdb's older data, and most of backzone, comes from Shanks & Pottenger's atlases, the same base the commercial atlases use. The maintainer, Paul Eggert, calls them unreliable: many offsets look guessed, with no sources and no marking of guesses; he has called a Shanks-only entry "almost negative evidence" ([tz-link.html](https://data.iana.org/time-zones/tz-link.html), [tz list thread](https://lists.iana.org/hyperkitty/list/tz@iana.org/thread/CSSNDQFKGZE6ML3SWM4XTU2CQHOHV2JM/), [tz list, 2021-09](https://mm.icann.org/pipermail/tz/2021-September/055888.html)). So buying an atlas would not clearly be more accurate.
- **backzone** (`PACKRATDATA=backzone PACKRATLIST=zone.tab`) restores separate pre-1970 zones for more cities. Its header calls it lower quality and less maintained, and Eggert advises distributions to leave it out ([backzone](https://raw.githubusercontent.com/eggert/tz/main/backzone), [tz list, 2023-05](https://mm.icann.org/pipermail/tz/2023-May/057993.html)). It can also change some modern data and zone IDs, so it is not purely historical.

### Checked for hora's users (tzdb source, 2026-10-25)

- **Hungary and neighbours:** `Europe/Budapest`, `Europe/Bucharest`, `Europe/Belgrade` and `Europe/Vienna` are full zones in the default build. `Europe/Bratislava` (link to Prague) and `Europe/Uzhgorod` (link to Kyiv) are links only, and backzone has no separate zone for either. Backzone adds only `Europe/Zagreb` and `Europe/Ljubljana` in this region.
- **India:** `Asia/Kolkata` uses Madras time (+5:21:10) for all of India from 1870 to 1906, then IST. Bombay Time, used in Bombay until 1955, is in no tzdb build, backzone included. A pre-1955 birth in western India therefore needs the user to state the offset.

So backzone would not help hora's main users, and it costs data quality.

## 3. Rust access

- **jiff** reads the system zoneinfo by default and honours `TZDIR`; `jiff-tzdb` (the bundled copy, default build, no backzone) is used only where no system copy exists ([jiff-tzdb docs](https://docs.rs/jiff-tzdb)). Pointing `TZDIR` at the flake's tzdata lets `flake.lock` pin and update the data without a crate release.
- **chrono-tz** compiles tzdb into the binary by build script, so new data needs a new crate release and a rebuild, and it can lag tzdb ([chrono-tz docs](https://docs.rs/crate/chrono-tz/latest)).
- For local times in a DST gap or repeated hour, jiff exposes the ambiguity (`to_ambiguous_zoned`); hora reports it rather than taking jiff's `Compatible` default.

## 4. Coordinates and places to a zone

- GeoNames `cities500` rows carry an IANA zone name, so choosing a place gives its zone. Licence CC BY 4.0, attribution in the UI (see §2.5 of the stack research).
- For typed coordinates, **tzf-rs** maps (longitude, latitude) to a zone offline, from `tzf-dist` data (about 4 MB "lite") built from [timezone-boundary-builder](https://github.com/evansiroky/timezone-boundary-builder) polygons: code MIT, data ODbL from OpenStreetMap ([tzf-rs docs](https://docs.rs/tzf-rs)). Unmodified use needs attribution only, and ODbL covers data, not code, so it does not conflict with AGPL. The simplified polygons can misplace points near a border. Deferred until typed coordinates need a zone.

## Unverified

- Whether jiff re-reads `TZDIR` at runtime or caches it for the process (affects tzdata updates without restart).
- Whether GeoNames `alternateNamesV2` has Hungarian names for the places Hungarian users will type (e.g. Kolozsvár, Pozsony).
- What the Vedic programs listed in §1 use, and whether they let users state the offset.
- Astrodienst's error rates are from 2020 and measured against its own atlas.

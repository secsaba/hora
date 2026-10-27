---
status: accepted
---

# The UTC offset comes from plain tzdb, and the user can always override it

When the user does not state the UTC offset, hora derives it from the birth place's IANA zone with jiff, reading the nixpkgs `tzdata` pinned by `flake.lock` (no backzone), and never from a commercial astrology atlas. No available source is reliable before 1970: tzdb is only meant to be right for the city each zone is named after, and the commercial atlases rest on the same Shanks data that tzdb's maintainer distrusts. So the override is part of the design: the user can always state the offset or choose local mean or local apparent time. Research: [time-zones-and-historical-offsets.md](../research/time-zones-and-historical-offsets.md).

## Decision

- **Where:** `hora-web` resolves place, date and time to seconds and calls `hora-swe` with `UtcOffset::Stated`. `hora-swe` has no time zone dependency, and the zone is shown only as the reason for the offset (a zone is only a way to find the offset, per `GLOSSARY.md`).
- **No backzone:** it adds no zone for Hungary's region except Zagreb and Ljubljana, none for India (Bombay Time is in no tzdb build), its maintainers call it lower quality, and it can change modern data.
- **Places:** GeoNames `cities500` plus every populated place in its Hungary file, since `cities500` keeps only places with a recorded population of 500 or more and GeoNames records none for most villages: it holds 1,240 Hungarian places of about 3,150 settlements. Hungarian names such as Kolozsvár are found through the rows' alternate names, so `alternateNamesV2` is not needed. GeoNames regenerates its dumps daily, so the flake pins a snapshot uploaded as a release asset of this repo.
- **Zone from a chosen place only:** the GeoNames row carries the zone. Typed coordinates mean the user states the offset; a coordinate-to-zone lookup (tzf-rs) waits until users ask for it.
- **Ambiguous local times:** a time in a DST gap or a repeated hour is refused, naming the rule and the possible offsets, so the user picks one as a stated offset. hora never guesses.
- **Shown, not hidden:** a chart shows the zone and offset used, and warns when the offset comes from a zone for a date before 1970.

## Considered options

- **A commercial atlas (ACS International/American Atlas):** not clearly more accurate (same Shanks source), proprietary data that could not ship in a public AGPL repo, and it costs money.
- **tzdb with backzone:** more pre-1970 zones, but none that hora's main users need, at lower quality.

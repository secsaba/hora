# Rust stack and WebAssembly for hora

Researched 2026-10-21. Versions, dates and licenses come from the crates.io API (`https://crates.io/api/v1/crates/<name>`) and GitHub on that date unless another source is linked. Constraints come from [`CLAUDE.md`](../../CLAUDE.md).

## Recommendation summary

| Area | Pick | Alternative |
|---|---|---|
| Toolchain | **Stable Rust, edition 2024**, pinned in `rust-toolchain.toml` (1.99.1 today). MSRV = the pinned version | none (nightly not needed) |
| Nix provisioning | **oxalica/rust-overlay** + `fromRustupToolchainFile`, on nixpkgs **26.05** (the dev host is x86_64-darwin) | nix-community/fenix |
| Swiss Ephemeris | **Our own small `-sys` crate**: C from our patched fork of upstream vendored into the repo, built with `cc`, a few hand-written `extern` declarations, TLS left on | `sweph-sys` 0.3.0 (AGPL-3.0-only) |
| Ephemeris data | `sepl_12/18` + `semo_12/18` `.se1` files (about 3.6 MB, years 1200–2399), fetched by Nix at a pinned commit with hashes. Reject dates outside that range | Moshier (built in, no files, less accurate) |
| Web server | **axum 0.8 + tokio + tower-http** | actix-web 4 |
| Rendering | **Server-rendered HTML with askama 0.16**. Charts drawn as **inline SVG from the same templates**. Plain forms; htmx 4 (0BSD) only where it helps | minijinja |
| i18n | **fluent-templates** (Fluent `.ftl` files, `/en/` and `/hu/` URL prefixes) | rust-i18n |
| Date/time | **jiff** (+ Nix `tzdata`), show the UTC offset it resolved and let the user override it | chrono + chrono-tz |
| Place lookup | **GeoNames `cities500` extract** (CC BY 4.0), searched in-process | OSM Nominatim (external service, so no) |
| Persistence | None for the first slice. When accounts or saved charts arrive: **sqlx 0.9 + SQLite** | sqlx + PostgreSQL |
| Errors / logs | thiserror (library code) + anyhow (binary only); tracing + tracing-subscriber | n/a |
| Validation | Newtypes parsed at the boundary (`TryFrom`, serde `try_from`); add garde only if forms grow | validator |
| Tests | built-in `#[test]` with values pinned from the book's example charts; insta for HTML/SVG snapshots; proptest for invariants | n/a |
| Supply chain | **cargo-deny** (licenses, advisories, bans, sources), from Nix | cargo-audit |
| Knowledge base | Markdown files per language (`content/kb/{en,hu}/*.md`), rendered with **pulldown-cmark**, raw HTML turned off | comrak |
| WebAssembly | **Not in v1.** The evidence supports "maybe later, for client-side interactivity" (details in section 3) | n/a |

**Two risks to know now:**
1. **The dev machine is x86_64-darwin (Intel Mac).** nixpkgs 26.11 refuses to evaluate on it. 26.05 is the last nixpkgs release that supports it, and it gets fixes only "until Nixpkgs 26.05 goes out of support at the end of 2026" ([nixpkgs release notes](https://nixos.org/manual/nixpkgs/unstable/release-notes#x86_64-darwin-26.11)). That is about 10 weeks away. Plan for an aarch64 Mac or a Linux dev box/VM.
2. **No Rust crate for Swiss Ephemeris is established yet.** Every candidate is small and young (table in 2.1), so we own the thin FFI layer ourselves.

---

## 1. Toolchain

### Stable and edition 2024
- Latest stable is **Rust 1.99.1** (2026-10-15). 1.99.0 shipped 2026-10-01 ([Rust releases](https://blog.rust-lang.org/releases/), [1.99.1 post](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/)).
- **Edition 2024** has been stable since Rust 1.85.0 (2025-02-20) ([announcement](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)). Use `edition = "2024"`.
- No nightly. Nothing in this stack needs it, and nightly works against correctness.
- **MSRV policy:** hora is an application, not a published library. So the MSRV is simply the toolchain pinned in `rust-toolchain.toml`. Set `rust-version` in `Cargo.toml` to the same version and bump both together in their own PR. Dependency MSRVs already reach 1.94 (sqlx 0.9.0) and 1.89 (comrak), so an old pin would block upgrades.
- x86_64-apple-darwin has been "Tier 2 with host tools" since 1.90. The Rust project still ships its compiler and std builds, but tests are no longer guaranteed to run on it ([Rust blog](https://blog.rust-lang.org/2025/08/19/demoting-x86-64-apple-darwin-to-tier-2-with-host-tools/)).

### Nix: rust-overlay
| Option | Verdict |
|---|---|
| nixpkgs `rustc` | **No.** nixpkgs 26.05 has rustc **1.95.0** (`nix eval github:NixOS/nixpkgs/nixpkgs-26.05-darwin#rustc.version`, run 2026-10-21), four releases behind stable. The dev host is stuck on 26.05 (risk 1), so the compiler would be frozen there too. |
| **oxalica/rust-overlay** (MIT, pushed 2026-10-20) | **Pick.** It packages the official rustup binary toolchains. Hashes are prefetched in the repo, so evaluation stays pure. `rust-bin.fromRustupToolchainFile ./rust-toolchain.toml` makes that one file the source of truth for both Nix and editors ([README](https://github.com/oxalica/rust-overlay)). `manifests/stable/1.99.1.nix` is already in the repo. |
| nix-community/fenix (MPL-2.0) | Fine alternative ([repo](https://github.com/nix-community/fenix)). It is strongest for nightly and rust-analyzer-nightly, which we don't need. |

Caveat: rust-overlay's README says CI runs only on x86_64-linux and nixos-unstable, and "other platforms and nixpkgs channels may also work". It works on x86_64-darwin with nixpkgs 26.05 (verified 2026-10-21 by building the dev shell).

Put the dev tools in the same flake from nixpkgs: `cargo-deny`, `cargo-insta`, and `sqlx-cli` later. Do not use `cargo install`.

---

## 2. Libraries

Every license below is compatible with AGPL-3.0-or-later. Per the [FSF license list](https://www.gnu.org/licenses/license-list.html), MIT/Expat, BSD-2/3, 0BSD-style, Unlicense and Apache-2.0 are GPLv3-compatible, and MPL-2.0 is compatible through its §3.3. AGPLv3 §13 allows combining with GPLv3 code.

### 2.1 Swiss Ephemeris

**Upstream facts**
- Official source: [aloistr/swisseph](https://github.com/aloistr/swisseph), "official release by Astrodienst". The header says `SE_VERSION "2.10.03"` (`sweph.h`). Latest tag is `v2.10.3bfinal` (2026-08-02, commit `f4dcd18e`). The repo was last pushed 2026-10-20.
- License ([LICENSE](https://github.com/aloistr/swisseph/blob/master/LICENSE)): dual AGPL or Swiss Ephemeris Professional. AGPL replaced GPL starting with 2.10.01 (`doc/swisseph.htm`). Under AGPL "the whole software project" must be AGPL or compatible. Also: "The names of the authors or of the copyright holder (Astrodienst) must not be used for promoting any software". The label "Swiss Ephemeris Inside" is allowed ([swephinfo](https://www.astro.com/swisseph/swephinfo_e.htm)).
- **Data files have no separate license.** They live in the same repo under the same LICENSE.
- Accuracy ([swephinfo](https://www.astro.com/swisseph/swephinfo_e.htm), `doc/swisseph.htm`):
  - The `.se1` files reproduce JPL to 0.001″. "Since May 2026, it is based on DE441".
  - The built-in Moshier model needs no files. It is below 1″ for planets and "a few arc seconds" for the Moon.
  - Each `.se1` file covers 600 years. `sepl_18.se1` + `semo_18.se1` cover 1800–2399, and `_12` covers 1200–1799. Sizes: 484 KB + 1.30 MB each pair, about **3.6 MB for 1200–2399**. The full set is 97 MB.
- **Thread safety:** upstream has been "thread safe" since 2.03 (`doc/swisseph.htm` changelog). It does this with thread-local storage for its global state (`sweodef.h`: `#define TLS __thread`). So `swe_set_ephe_path`, `swe_set_sid_mode` and similar calls apply **only to the calling thread**, but only on Linux/gcc/clang: `sweodef.h:86` disables TLS on macOS (`!defined(__APPLE__)`), where all state is process-global. We vendor a fork with that exclusion removed, proposed upstream as [aloistr/swisseph#23](https://github.com/aloistr/swisseph/pull/23) (see [ADR 0001](../adr/0001-swiss-ephemeris-worker-pool.md)). The official source is [aloistr/swisseph](https://github.com/aloistr/swisseph), which [astro.com's download page](https://www.astro.com/swisseph/swedownload_e.htm) names as the "always the most up-to-date" location.

**Existing crates**

| Crate | Latest | License | Notes |
|---|---|---|---|
| `sweph` / `sweph-sys` ([repo](https://github.com/invents-us/sweph-rs)) | 0.3.0, 2026-07-06 | AGPL-3.0-only | Vendors C 2.10.03 and builds it with `cc`. Builds with **`TLSOFF`**, and the safe crate puts **one global mutex** around every call. The safe API has only 5 ayanamshas. `set_sidereal_mode` is a separate process-wide call, so two requests can race between "set mode" and "calc". Repo created July 2026, 0 stars, 473 downloads. |
| `swisseph-rs` ([repo](https://github.com/ninthhousestudios/swisseph-rs)) | 0.2.1, 2026-08-29 | AGPL-3.0-or-later (LICENSE file: derivative of SwissEph, AGPL v3+) | A pure-Rust port that claims bit-compatibility with 2.10.03. Interesting for wasm later, but a port is a second implementation of the astronomy, and `CLAUDE.md` says not to hand-roll it. 974 downloads. |
| `swiss-eph` + `swiss-eph-data` ([repo](https://github.com/fusionstrings/swiss-eph)) | 0.2.1, 2026-01-22 | AGPL-3.0 | Last push January 2026. |
| `libswe-sys` ([repo](https://github.com/stephaneworkspace/libswe-sys)) | 0.2.7, **2022** | "non-standard" | Its LICENSE text offers "GNU public license version 2 or later", which is the pre-2.10.01 GPL-era tree. **Disqualified.** |
| `swisseph` (crates.io) | 0.1.1, 2024 | AGPL-3.0-only | No repository link. |

**Recommendation: write our own `hora-swe` crate.** The whole build is about 20 lines of `build.rs`: `cc::Build` over 9 `.c` files. You can see this in [sweph-sys's build.rs](https://github.com/invents-us/sweph-rs/blob/HEAD/sweph-sys/build.rs). We need maybe 8 functions: `swe_set_ephe_path`, `swe_set_sid_mode`, `swe_utc_to_jd`, `swe_calc_ut`, `swe_houses_ex`, `swe_get_ayanamsa_ex_ut`, `swe_revjul`, `swe_close`. Write their `extern "C"` declarations by hand. bindgen is not worth it for that few.
- **Vendor the C source into the repo** from our fork at tag `v2.10.3bfinal-hora.1` (upstream `v2.10.3bfinal` + two patches; see [ADR 0001](../adr/0001-swiss-ephemeris-worker-pool.md)), keeping the license and notice files. Record the commit in a file next to it.
- Copy the `NO_SWE_GLP` define from sweph-sys, which removes the `dladdr` dependency.
- **Thread model: keep TLS on, and run the library only on a fixed pool of dedicated worker threads.** Decided in [ADR 0001](../adr/0001-swiss-ephemeris-worker-pool.md); the details (pool size, queue, worker lifecycle, tests) live there only.
- **Fail closed on the ephemeris fallback.** `swe_calc_ut` returns the flags it actually used. If the result says Moshier was used when `.se1` was requested, treat it as an error and do not show it silently. Moshier and DE441 differ by arcseconds on the Moon.
- **Ship only the files we need.** Fetch `sepl_12`, `sepl_18`, `semo_12` and `semo_18` in the flake with `fetchurl` and hashes, pinned to the same upstream commit. Pass their directory in through runtime config. Server-side validation rejects birth dates outside 1200–2399.
- Julian vs Gregorian calendar: countries switched in different years (Hungary 1587, Britain 1752, Russia 1918), so a fixed 1582 boundary misreads old Julian-dated records. Let the user choose the calendar on the input form, default Gregorian on or after 1582-10-15, and pass the matching `gregflag`. Pin one Julian-dated chart in a test.
- Alternative: depend on `sweph-sys` for the build and write our own safe layer on top. But we would inherit `TLSOFF`, and the crate is three months old with a single maintainer.

### 2.2 Web server and runtime

| | Version / date | License | Why |
|---|---|---|---|
| **axum** | 0.8.9, 2026-04-14 | MIT | Part of tokio-rs and by far the most used (533 M downloads). Built on tower, so middleware comes from tower-http. No `unsafe`-heavy custom runtime. |
| **tokio** | 1.54.0, 2026-10-19 | MIT | The runtime axum needs. FFI calls go to the dedicated Swiss Ephemeris worker threads, not the async runtime or `spawn_blocking`. |
| **tower-http** | 0.7.1, 2026-08-31 | MIT | Request body limits, timeouts, compression, static files, and security headers. |
| alt: actix-web | 4.17.0, 2026-10-15 | MIT OR Apache-2.0 | Mature, but has its own actor-style runtime and a smaller middleware ecosystem. |

The production hostname and the AGPL source URL come from configuration (environment) at startup and are never in the repo. The page footer links to the source URL (AGPL §13).

### 2.3 Rendering and chart drawing

**Pick: server-rendered HTML with askama**, 0.16.1 (2026-09-04, MIT OR Apache-2.0, [repo](https://github.com/askama-rs/askama)).
- Templates are compiled and type-checked at build time and HTML-escaped by default.
- The `askama_web` 0.16.0 crate wires askama into axum.
- **Charts (chakras): inline `<svg>` written by the same askama templates.** No drawing crate is needed. North and South Indian charts are fixed geometry plus text. SVG `<title>`/`<desc>` and real text keep them accessible, and the labels go through i18n like everything else.
- **htmx** 4.0.0 (2026-08-28, 0BSD, [repo](https://github.com/bigskysoftware/htmx)) is optional. The baseline is plain `<form>` posts and full-page renders, which work everywhere. Add htmx only for partial updates, and serve it from our own origin, not a CDN.
- Alternative: **minijinja** 3.0.1 (Apache-2.0). Templates load at runtime, which is more flexible but loses compile-time checks.

Full-stack wasm frameworks were rejected for v1:
- Leptos 0.8.22 (MIT), Dioxus 0.7.10 (MIT OR Apache-2.0) and Yew 0.23.0 (MIT OR Apache-2.0) are all maintained.
- Their value is client-side reactivity, which hora v1 does not need. They add a wasm build target, hydration and a bigger client payload (see section 3).
- maud 0.27.0 has had no release since 2025-02-02.

### 2.4 i18n

**Pick: fluent-templates** 0.15.1 (2026-08-05, MIT/Apache-2.0), on Mozilla's **fluent** 0.17.0 (Apache-2.0 OR MIT, [projectfluent/fluent-rs](https://github.com/projectfluent/fluent-rs), pushed 2026-09-27).
- Fluent supports plural and select variants through CLDR rules, which Hungarian needs.
- `fluent-templates`' `static_loader!` ([docs](https://docs.rs/fluent-templates/latest/fluent_templates/)) loads `locales/{en,hu}/*.ftl` once, and an askama filter or function looks messages up.
- **Write whole-sentence messages per locale.** Hungarian suffixes follow vowel harmony (‑ban/‑ben), so building sentences by inserting nouns breaks.
- Put the locale in the URL (`/en/…`, `/hu/…`) for SEO and shareable links.
- Add a test that every message ID exists in both locales.
- Alternative: **rust-i18n** 4.2.4 (MIT). Simpler YAML key/value, but weaker grammar features.

### 2.5 Date/time, time zones, places

**Pick: jiff** 0.2.38 (2026-10-06, Unlicense OR MIT, [docs](https://docs.rs/jiff/latest/jiff/)).
- It reads the system tz database by default. Nix-built containers have no `/usr/share/zoneinfo`, so set `TZDIR=${pkgs.tzdata}/share/zoneinfo`, which jiff honors ([`TimeZoneDatabase` docs](https://docs.rs/jiff/latest/jiff/tz/struct.TimeZoneDatabase.html)). `flake.lock` then pins the tzdata version.
- For local times that fall in a DST gap or are repeated, it uses `Disambiguation::Compatible` by default and exposes the ambiguity through `to_ambiguous_zoned`. Show that ambiguity to the user instead of guessing silently.
- Alternative: chrono 0.4.45 + **chrono-tz 0.10.4**. chrono-tz's last release was 2025-07-11, and it embeds tzdb at compile time, so that copy is now over a year old. Which tzdb version it embeds is **unverified**.

**Historical accuracy (the real limit, which no crate fixes).** IANA itself says:
- "many, perhaps most, of the tz database's pre-1970 and future timestamps are either wrong or misleading"
- zones whose clocks "have agreed since 1970" get one name "even if some of the clocks disagreed before 1970"

([theory.html](https://data.iana.org/time-zones/theory.html)). Many births are before 1970. So:
1. Always display the UTC offset and DST status that we resolved.
2. Let the user override the offset (validated server-side to a sane range).
3. Store what the user entered and what we resolved.

Deeper research on historical offsets, backzone and atlases: [time-zones-and-historical-offsets.md](time-zones-and-historical-offsets.md) (decision in ADR 0005).

**Place lookup: GeoNames dump** ([readme](https://download.geonames.org/export/dump/readme.txt)).
- License: CC BY 4.0, which is GPL-compatible per the FSF (data, so the "not for software" note does not apply). Attribution is needed in the UI.
- `cities500` has about 185k places. Each row includes the **IANA timezone ID**, which gives us the place-to-zone step. `alternateNamesV2` has Hungarian names (`isolanguage` field; that `hu` is present is **unverified**).
- Load it in-process (a small indexed table or SQLite). Birth places are personal data, so do not send them to a third-party geocoding API.
- Alternative: OSM Nominatim, an external service with usage limits. Rejected for privacy and uptime.

### 2.6 Persistence, serialization, validation, errors, logging, tests

| Area | Pick | Version / date | License | Note |
|---|---|---|---|---|
| Persistence | sqlx (+ SQLite) | 0.9.0, 2026-05-21 | MIT OR Apache-2.0 | Only once accounts or saved charts exist. SQL is checked at compile time. MSRV 1.94. Alternative: PostgreSQL if we run several instances. |
| Serialization | serde | 1.0.229, 2026-07-18 | MIT OR Apache-2.0 | Use `#[serde(deny_unknown_fields, try_from = …)]` on input types. |
| Validation | own newtypes (`BirthDate`, `Latitude`, `UtcOffset`…) | n/a | n/a | Parse, don't validate: invalid values cannot be built. Alternative: **garde** 0.23.0 (MIT OR Apache-2.0). |
| Errors | thiserror / anyhow | 2.0.21 / 1.0.104 | MIT OR Apache-2.0 | thiserror for typed errors in library crates, anyhow only in `main`. Never show internal error text to users. |
| Logging | tracing + tracing-subscriber | 0.1.44 (2025-12-18) / 0.3.23 (2026-03-13) | MIT | Do not log birth data (personal data). |
| Snapshot tests | insta | 1.49.0, 2026-10-03 | Apache-2.0 | For rendered SVG and HTML, in both locales. |
| Property tests | proptest | 1.12.0, 2026-10-10 | MIT OR Apache-2.0 | For example: longitudes stay in [0, 360), and date parsing round-trips. |
| Reference tests | built-in `#[test]` | n/a | n/a | Pin values from the book's example charts and cite the chapter in each test (per `CLAUDE.md`). |

### 2.7 Supply chain

**Pick: cargo-deny** 0.20.2 (2026-07-09, MIT OR Apache-2.0, [repo](https://github.com/EmbarkStudios/cargo-deny)). It checks licenses, RustSec advisories, banned or duplicate crates, and allowed registries and sources, so it covers everything cargo-audit does. Alternative: cargo-audit 0.22.2 (advisories only).

The `deny.toml` license allow-list will need:
- AGPL-3.0-or-later (our own crates). Do not allow AGPL-3.0-only: it would conflict with the project's "or later" license.
- MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause (bindgen, if used), Unlicense, and the Unicode license that `unicode-ident` and ICU-style crates bring in. The exact SPDX IDs are **unverified** until the first `cargo deny check` runs.

Run it in CI and in the flake's `checks`.

### 2.8 Knowledge base

**Pick: pulldown-cmark** 0.13.4 (2026-05-20, MIT).
- Files live at `content/kb/en/<slug>.md` and `content/kb/hu/<slug>.md`, with front matter for title and summary.
- Render at startup or build time and cache the result.
- Drop raw HTML: pulldown-cmark has no option to disable it and always emits `Event::Html`/`InlineHtml`, so filter those events out. A test feeds in `<script>` and asserts it is gone. That makes an ammonia sanitizer unnecessary for content we write ourselves.
- One test asserts that the EN and HU slug sets are identical.
- The content explains ideas in our own words and links to the [book page](https://www.vedicastrologer.org/articles/astro_books.htm).
- Alternative: comrak 0.57.0 (BSD-2-Clause). It covers all of GFM but is heavier.

---

## 3. WebAssembly

### What it is
- WebAssembly (Wasm) is "a binary instruction format for a stack-based virtual machine … designed as a portable compilation target for programming languages" ([webassembly.org](https://webassembly.org/)).
- In practice: you compile Rust or C into a compact `.wasm` file, and the browser runs it at close to native speed inside the same sandbox as JavaScript. It obeys "the browser's same-origin and permissions policies" ([MDN](https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Concepts)).
- It complements JavaScript rather than replacing it. It "cannot currently directly access the DOM; it can only call JavaScript" (MDN).
- Specs: [Core 1.0](https://www.w3.org/TR/wasm-core-1/) is a W3C Recommendation. [Core 2.0](https://www.w3.org/TR/wasm-core-2/) is a W3C Candidate Recommendation Draft.

### Should hora use it? **Not in v1.** Revisit when a feature needs instant client-side interaction.

| Factor | Evidence | Weight |
|---|---|---|
| Trust and validation | Input must be validated and charts computed on the server anyway (`CLAUDE.md`). A client calculation would be a **second implementation** that could disagree with the server. Moshier (no files) differs from DE441 `.se1` by "a few arc seconds" on the Moon. That is bad for a project that puts correctness first. | **Decisive** |
| Need | No v1 feature (chart calculation, display, knowledge base) needs computation in the browser. SSR plus forms covers it. | Decisive |
| Data size | **Not a blocker, contrary to the common assumption.** 1800–2399 needs about 1.8 MB, and Moshier needs zero files. | Weak |
| Toolchain | Building Swiss Ephemeris's C for `wasm32` needs clang plus a C sysroot (wasi-libc or emscripten) in the flake, and a replacement for its `fopen`-based file loading. That is doable, and the `swisseph-wasm` crate exists, but it is extra build surface. A pure-Rust port (`swisseph-rs`) would compile directly but is a different implementation. | Moderate |
| Phones | Only a server-rendered page has zero client payload. Wasm frameworks ship a runtime bundle plus hydration. | Moderate |
| SEO and accessibility | Server-rendered HTML is indexable and works without JS. The knowledge base especially benefits. | Moderate |
| AGPL | Shipping a `.wasm` counts as conveying object code, but the app already offers its source (§13), so there is **no new obligation**, just keep the link. | Neutral |
| Complexity | A second build target, a JS/wasm boundary, and duplicated i18n on the client. | Moderate |

**Your expectation holds, but for different reasons.** The decisive reasons are trust and "no v1 need". Data-file size is not one of them. Possible later uses: live transit scrubbing, or dragging a time slider and redrawing the chart instantly. Then compile the existing Rust chart-geometry code to wasm, and keep the server as the authority for every saved or shared result.

---

## Unverified
- ~~rust-overlay with nixpkgs 26.05 on x86_64-darwin (README: CI covers only x86_64-linux).~~ Answered 2026-10-21: the dev shell builds and links a binary on x86_64-darwin with 1.99.1; all four systems evaluate.
- The exact SPDX IDs `cargo deny` will report (Unicode etc.).
- ~~Whether any Swiss Ephemeris state is not covered by TLS.~~ Answered: only `swehouse.c:636` (house system I) and `sweph.c:258` (`swe_get_library_path`) are non-TLS mutable state on Linux; both are excluded by [ADR 0001](../adr/0001-swiss-ephemeris-worker-pool.md).
- Whether a sidereal Vedic setup (all ayanamshas, `swe_houses_ex` with `SEFLG_SIDEREAL`) behaves as expected through our hand-written FFI. Pin it against the book's example charts.
- What changed between upstream tags `v2.10.3final` (2026-04-14) and `v2.10.3bfinal` (2026-08-02). Read the diff before vendoring.
- Whether GeoNames `alternateNamesV2` has good Hungarian coverage for small places.
- Whether "Swiss Ephemeris" itself (the product name, not the authors' names) may appear in UI credits beyond "Swiss Ephemeris Inside". Ask Astrodienst if it matters.

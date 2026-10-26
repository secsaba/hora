# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

A Cargo workspace with two crates:

- `crates/hora-swe`: the only code that calls Swiss Ephemeris (C vendored in `crates/hora-swe/swisseph/`, built by `build.rs`), run on a fixed worker pool behind async `calculate(request)` (ADR 0001). Golden tests are in `crates/hora-swe/tests/golden.rs`.
- `crates/hora-web`: the axum server, the only caller of `hora-swe`. Config comes from the environment (`HORA_EPHE_PATH`, `HORA_WORKERS`, `HORA_QUEUE`, `HORA_BIND`); pages are askama templates in `templates/`, their text in `locales/{en,hu}/*.ftl`.

Run inside the dev shell (the tests need `HORA_EPHE_PATH`):

- `cargo build`
- `cargo run -p hora-web` (serves on `127.0.0.1:3000` unless `HORA_BIND` says otherwise)
- `cargo test --locked` (one file: `cargo test --locked --test golden`)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo deny check` (AGPL-compatible licences per `deny.toml`, bans, sources, plus RustSec advisories)

CI (`.github/workflows/ci.yml`) runs fmt, test, clippy and `cargo deny check licenses bans sources` through `nix develop -c` on Linux (Ubuntu) only, since production is Linux and macOS is covered by local development; `cargo deny check advisories` runs as its own job, also weekly.

## What hora is

- A public, multi-user **web application** for Vedic (Jyotish) astrology, published as a public GitHub repository.
- Web only: no CLI, no MCP server. The UI must work on desktop and phone browsers.
- Written in **Rust**. Priorities, in order: **correctness, security, performance**.
- **Bilingual: English and Hungarian, both first-class.** Every user-facing string, including knowledge base content, must exist in both languages; no hardcoded UI text. A feature is not done until both translations are.
- Features: chart calculations, display of charts (chakras) and related information, and a knowledge base.
- Do not hardcode or commit the production hostname anywhere in the repo; take it from runtime configuration.

## Domain sources

- **Calculations:** Swiss Ephemeris. Do not hand-roll astronomical math that the ephemeris provides.
- **Methods and interpretation:** P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*. The project works through this book; calculation rules and chart-reading logic should follow it. When implementing a technique, cite the chapter/section in a code comment or test so it can be checked against the book. Work from the local PDF in `book/` (gitignored; download it from the link below), never from memory of the book. Read its text from `book/book.txt` (extract it once with `pdftotext -layout book/*.pdf book/book.txt`); for a page with a chart, render it with `pdftoppm -png -r 150 -f N -l N book/*.pdf book/pN` and read the image.
- The book's text is copyrighted: the knowledge base explains concepts in its own words and never reproduces the book. Where the UI references the book, link users to https://www.vedicastrologer.org/articles/astro_books.htm.
- Correctness is verified against known reference values (e.g. example charts from the book), so calculation code should come with tests that pin those values.
- **Golden tests** pin what the book prints for its example charts, at the precision it prints (whole arcminutes), in `tests/golden.rs` of the crate that computes it: positions in `hora-swe`; derived facts the book shows (a graha marked R, a graha's house, an arudha) in the layer above, computed from `hora-swe`'s output. What the book does not print for a chart (sunrise, say, outside a chapter that works it out) gets an ordinary test. Never change a golden test's inputs or expected values to make it pass; fix the code. A golden value changes only for a cited erratum of the book.

## Development environment

- Dependencies and tooling are managed with a **Nix flake** (`flake.nix`). Everything is installed through it, including the Rust toolchain (compiler, cargo, clippy, rustfmt) and any native libraries.
- The environment loads automatically on `cd` into the repo via **direnv** + nix-direnv (`.envrc` containing `use flake`; run `direnv allow` once). `nix develop` also works. Run build/test commands inside this environment.
- The dev shell sets `HORA_EPHE_PATH` to the Swiss Ephemeris data files (`nix build .#ephe`), fetched and hash-pinned by the flake (never committed).
- Do not install tools via rustup, Homebrew, cargo install, or other system package managers; add them to the flake instead. This covers build and runtime tooling only; Claude Code plugins are per-user and configured in `.claude/settings.json`.

## Code and comments

- **Smallest complete change** (the ponytail approach): solve the whole task with the least new code. Reach first for what already exists: this codebase, then std, then an existing dependency. Write only what the task needs now; skip speculative options, abstractions and "for later" code. Finish every place the change reaches (callers, tests, fixtures). Validation at trust boundaries, error handling, security and accessibility always stay, whatever they cost in lines.
- **Comments carry only the non-obvious *why***, in one short line: an invariant, a hidden constraint, a past failure. The code says what it does; the commit message says what changed.
- A deliberate shortcut gets `// shortcut: <the limit>, <when to upgrade>`.

## Constraints to keep in mind

- License is **AGPL-3.0-or-later** (`LICENSE`; use SPDX id `AGPL-3.0-or-later` in `Cargo.toml`), required by using Swiss Ephemeris under its AGPL option. As a network service, the running app must offer users a link to its source. Every dependency must be AGPL-compatible.
- It is a public, multi-user service: all user input (birth date/time/place, etc.) crosses a trust boundary and must be validated server-side.

## Agent skills

The project is developed with Claude Code using the [Matt Pocock skills](https://github.com/mattpocock/skills) and [ponytail](https://github.com/DietrichGebert/ponytail) plugins; `.claude/settings.json` offers both when you trust the repo.

### Issue tracker

Issues live in GitHub Issues (secsaba/hora), via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Merging PRs

Squash-merge a PR whose commits are not worth keeping apart (one commit, or review fix-ups); use a merge commit when its commits were split on purpose, so each stays on `main`.

### Triage labels

Default vocabulary: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

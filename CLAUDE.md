# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

Only the development environment exists (`flake.nix`, `rust-toolchain.toml`, `.envrc`); no Cargo workspace or code yet. Update this file with real build/test/lint commands and the architecture once the first crate lands.

## What hora is

- A public, multi-user **web application** for Vedic (Jyotish) astrology, published as a public GitHub repository.
- Web only: no CLI, no MCP server. The UI must work on desktop and phone browsers.
- Written in **Rust**. Priorities, in order: **correctness, security, performance**.
- **Bilingual: English and Hungarian, both first-class.** Every user-facing string, including knowledge base content, must exist in both languages; no hardcoded UI text. A feature is not done until both translations are.
- Features: chart calculations, display of charts (chakras) and related information, and a knowledge base.
- Do not hardcode or commit the production hostname anywhere in the repo; take it from runtime configuration.

## Domain sources

- **Calculations:** Swiss Ephemeris. Do not hand-roll astronomical math that the ephemeris provides.
- **Methods and interpretation:** P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*. The project works through this book; calculation rules and chart-reading logic should follow it. When implementing a technique, cite the chapter/section in a code comment or test so it can be checked against the book.
- The book's text is copyrighted: the knowledge base explains concepts in its own words and never reproduces the book. Where the UI references the book, link users to https://www.vedicastrologer.org/articles/astro_books.htm.
- Correctness is verified against known reference values (e.g. example charts from the book), so calculation code should come with tests that pin those values.

## Development environment

- Dependencies and tooling are managed with a **Nix flake** (`flake.nix`). Everything is installed through it, including the Rust toolchain (compiler, cargo, clippy, rustfmt) and any native libraries.
- The environment loads automatically on `cd` into the repo via **direnv** + nix-direnv (`.envrc` containing `use flake`; run `direnv allow` once). `nix develop` also works. Run build/test commands inside this environment.
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

### Triage labels

Default vocabulary: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

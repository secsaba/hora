# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Status

The repository is empty: no code, build system or git history yet. Update this file with real build/test/lint commands and the architecture once the project is scaffolded.

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
- Do not install tools via rustup, Homebrew, cargo install, or other system package managers; add them to the flake instead.

## Constraints to keep in mind

- License is **AGPL-3.0-or-later** (`LICENSE`; use SPDX id `AGPL-3.0-or-later` in `Cargo.toml`), required by using Swiss Ephemeris under its AGPL option. As a network service, the running app must offer users a link to its source. Every dependency must be AGPL-compatible.
- It is a public, multi-user service: all user input (birth date/time/place, etc.) crosses a trust boundary and must be validated server-side.

## Agent skills

### Issue tracker

Issues live in GitHub Issues (secsaba/hora), via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

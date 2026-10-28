# hora

A web application for Vedic (Jyotish) astrology: it casts a birth chart (chakra), shows the grahas' positions and the panchaanga of the birth, and teaches the concepts behind them in a knowledge base. Everything is in English and Hungarian.

**Live at [hora.guide](https://hora.guide).** It stores nothing about you; see the [privacy page](https://hora.guide/en/privacy).

hora is for study and reflection, not medical, legal, financial or psychological advice; see the [disclaimer](https://hora.guide/en/disclaimer).

## Sources

- **Methods:** P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach* ([where to get it](https://www.vedicastrologer.org/articles/astro_books.htm)). hora works through the book in order. The knowledge base explains its concepts in its own words and does not reproduce the book.
- **Calculations:** [Swiss Ephemeris](https://www.astro.com/swisseph/). Tests pin the values the book prints for its example charts.

## Development

The toolchain and data files come from the Nix flake. Run `direnv allow` once, or use `nix develop`. Then:

```sh
cargo run -p hora-web   # http://127.0.0.1:3000
cargo test --locked
```

[`CLAUDE.md`](CLAUDE.md) covers the layout, commands and rules, [`GLOSSARY.md`](GLOSSARY.md) the domain language, and [`docs/adr/`](docs/adr/) the decisions behind them. Report bugs and ideas in [GitHub Issues](https://github.com/secsaba/hora/issues).

## License

[AGPL-3.0-or-later](LICENSE), as required by Swiss Ephemeris under its AGPL option. The running site links to this source.

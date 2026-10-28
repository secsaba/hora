---
status: accepted
---

# The knowledge base is Markdown in the repo, its tables rendered from calculation data

The knowledge base (KB) explains concepts in both languages, one page per glossary concept, and every value a chart shows links to its page. Its prose is long-form and reviewed like code; its tables (nakshatra spans and lords, tithi names, lunar months…) are the same facts the calculations use.

## Decision

- **Storage:** one Markdown file per page per language in the repo, compiled into the binary. No runtime database or CMS. A test fails when a page exists in one language only.
- **Tables:** written as placeholders in the Markdown and rendered from the data the calculations use, with names translated through the `.ftl` files, so a page and a chart can never disagree.
- **URLs:** `/{lang}/kb/{slug}`; the slug is the glossary term in the book's spelling and is the same in both languages. A page for several glossary terms that one book section teaches together (ch. 2 §2.2, *Characteristics of Rasis*) takes that section's title as its slug (`rasi-characteristics`) and gives each term and each rasi an anchor, so the chart page can link to a rasi's row.
- **Content:** organised by concept, each page citing the book's chapter and section; a learning path orders them as the book does. Prose is our own words, examples are our own and computed by hora; only factual tables are reproduced. Pages link to https://www.vedicastrologer.org/articles/astro_books.htm.

## Considered options

- **Fluent messages:** already used for UI text, but unsuited to multi-paragraph prose with headings and tables.
- **Database or CMS edited at runtime:** needs accounts, storage and an editing security surface; content would escape PR review.
- **One page per book section:** drifts toward reproducing the book, and chart links need concept pages.

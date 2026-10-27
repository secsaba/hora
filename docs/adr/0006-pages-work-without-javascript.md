---
status: accepted
---

# Every page works without JavaScript; scripts only enhance

hora is a public, multi-user service whose priorities are correctness, security, then performance. Its pages are server-rendered askama templates, and every input is validated on the server anyway (it crosses a trust boundary), so a script never has to be the only path to a feature.

## Decision

- **Baseline:** every feature works with HTML forms and links alone: casting a chart, searching for a birth place (a submit, not type-ahead), picking an offset, reading the knowledge base.
- **Scripts:** may only enhance a page that already works without them, and are first-party files served by hora. No third-party scripts, fonts or styles, so a future CSP can stay at `'self'` and visitors' requests never reach another host.
- **CSS:** newer features (`:has()`, `:user-invalid`, `color-mix()`) are used only where the page still works without them; an older browser loses the effect, not the feature.

## Considered options

- **No JavaScript at all:** the strictest CSP (`script-src 'none'`), but rules out enhancements such as type-ahead place search that a form submit can back up.
- **JavaScript-first (a client-rendered app):** smoother interaction, but every feature then depends on the script, the attack surface grows, and the server still has to validate everything.

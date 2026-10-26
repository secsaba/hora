# Coding standards

Rules for review that no automated check can enforce. `CLAUDE.md` holds the rest.

- **Rounded bounds round inward.** A time or angle shown as the bound of an interval (a limb's start, a sunrise) rounds toward the inside of that interval: starts up, ends down. A test checks that a birth at the shown value falls inside. `hms` truncates, so a start goes through `first_second` (`crates/hora-web/src/web.rs`).

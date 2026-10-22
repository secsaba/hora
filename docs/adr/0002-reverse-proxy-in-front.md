---
status: accepted
---

# hora serves plain HTTP on loopback behind a reverse proxy

`axum::serve` gives hyper no timer, so hyper's 30 s header read timeout is off (`axum-0.8.9/src/serve/mod.rs:391`), and axum exposes no way to set it. A client that sends a header byte every few seconds keeps its connection, task and file handle for as long as it likes; enough of them exhaust the process's file handles. hora is a public, multi-user service, so it must not face clients directly in that state.

## Decision

- **hora's side:** plain HTTP only, bound to loopback (`HORA_BIND` defaults to `127.0.0.1:3000`). hora has no TLS and no client timeouts or connection limits of its own.
- **The proxy's side:** any reverse proxy, not named here, that terminates TLS and enforces what hora does not: a timeout for reading request headers and body, a cap on connections per client, and a request body size limit. It forwards complete requests only, so slow clients wait on the proxy, never on hora.
- **Shutdown:** after SIGTERM or Ctrl-C, hora waits at most `SHUTDOWN_GRACE` (10 s, `crates/hora-web/src/main.rs`) for open requests, then shuts the ephemeris pool down and exits. That bound holds even if a request is somehow left open, so the supervisor's stop never escalates to a kill that skips `hora_swe::shutdown` (ADR 0001).
- **Hostname:** the production hostname belongs to the proxy's and the runtime's configuration, never to this repo.

## Considered options

- **Timeouts in hora:** replace `axum::serve` with hyper-util's server builder plus a timer, so hora sets the header read timeout itself. About 30–40 lines that a proxy makes redundant, and hora would still need TLS to face clients. Revisit if hora ever has to run without a proxy.
- **hora faces clients directly:** rejected for the slow-client exposure above.

---
status: accepted
---

# A Cloudflare Tunnel is the reverse proxy in production

Supersedes the proxy's side of ADR 0002 for production. The rest of ADR 0002 stands: hora has no TLS, timeouts or connection limits of its own, the code still defaults `HORA_BIND` to loopback, and shutdown is bounded by `SHUTDOWN_GRACE`.

Accepted on 2026-10-28, when a slow-client test passed through the tunnel: a client trickling header bytes held no hora connection. If that ever stops holding, hora sets its own timeouts (ADR 0002's "timeouts in hora" option) or a proxy runs between the tunnel and hora.

## Decision

- **Front door:** the production host's existing Cloudflare Tunnel (`cloudflared`) carries the production hostname to `http://hora:3000`. No port opens on the host; Cloudflare terminates TLS and renews the certificate.
- **Container:** the image from `packages.hora-web` (`flake.nix`) sets `HORA_BIND=0.0.0.0:3000`, the container's own interface; the container's Docker network is joined only by `cloudflared`, so nothing else reaches it.
- **Slow clients:** clients only ever connect to Cloudflare, which forwards complete requests through the tunnel.
- **Body size:** hora reads no request body (every route is a GET), so ADR 0002's body size limit has nothing left to guard.
- **Per-client cap:** a Cloudflare rate-limiting rule (more than 60 requests in 10 s from one IP blocks it for 10 s) replaces ADR 0002's cap on connections per client.
- **Birth data passes through Cloudflare:** the chart form is a GET, so birth details are in URLs that Cloudflare sees and hora's error logs keep (a failing chart is reproducible only from its inputs). The privacy page says so.
- **Hostname:** still only in the runtime's configuration (Cloudflare, the host's compose file), never in this repo's code or config (only the README names the live site).

## Considered options

- **Traefik with ports 80 and 443 open:** a second front door beside the tunnel the host already runs, and the host would publish ports. Caddy was ruled out for lacking a per-client connection cap.

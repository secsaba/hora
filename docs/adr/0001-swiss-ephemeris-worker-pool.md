---
status: accepted
---

# Swiss Ephemeris runs only on a fixed pool of dedicated worker threads

Swiss Ephemeris keeps mutable global state (ephemeris path, sidereal mode, topocentric position, open files, caches). It is per OS thread only where it is compiled with thread-local storage; upstream disables TLS on macOS (`sweodef.h:86`, `!defined(__APPLE__)`), which our patched source removes. So hora calls it only from a fixed pool of long-lived `std::thread` workers, never from the async runtime or `tokio::task::spawn_blocking`. tokio reuses and retires blocking threads, so a request would inherit the previous request's settings, an exiting thread would leak its files and memory (there is no TLS destructor), and the pool would grow to 512 threads under load on a public service.

## Decision

- **Source:** our fork [secsaba/swisseph](https://github.com/secsaba/swisseph), vendored at tag `v2.10.3bfinal-hora.1` (commit `e041a00`): the official [aloistr/swisseph](https://github.com/aloistr/swisseph) tag `v2.10.3bfinal` plus two patches proposed upstream, TLS on macOS ([aloistr/swisseph#23](https://github.com/aloistr/swisseph/pull/23)) and a `swe_close` fix ([aloistr/swisseph#22](https://github.com/aloistr/swisseph/pull/22)). Never master (SE 3.0 work lands there). Each new upstream tag gets the unmerged patches rebased onto it as a new `-hora.N` tag; once upstream releases both, vendor upstream directly and drop the fork. The commit is recorded next to the vendored source. TLS stays on.
- **Pool size:** N workers from runtime config, default `std::thread::available_parallelism`, on Linux (production is Debian) and macOS (development). There is exactly one pool per process (a process-wide `OnceLock`), and tests use that shared pool rather than building their own. Each worker opens its own ephemeris files and buffers, so memory and file handles grow with N; do not set N far above the core count.
- **Queue:** bounded, length from runtime config (default 4 × N). A full queue or a queue wait over ~2 s returns HTTP 503 with `Retry-After`; the 2 s counts time spent waiting in the queue only, not calculation time. Each job carries that deadline, and a worker drops a job without calling C if its deadline has passed or its reply channel is closed (`is_closed()`), so jobs abandoned by a 503 or a disconnected client do not occupy workers. A calculation that has started always finishes; C calls cannot be cancelled.
- **One trip per chart:** a worker job returns every raw ephemeris value for one moment and place (planets, houses, ayanamsha). Everything derived from longitudes (vargas, nakshatras, dashas) is computed in plain Rust outside the worker.
- **Worker lifecycle:** `swe_set_ephe_path` once at startup, with the path from hora's config (it closes open files, so calling it per request would defeat the file cache). Before every chart, set the sidereal mode and topocentric position explicitly. `swe_close` only at shutdown: it closes the cached `.se1` files, so calling it per request would defeat the file cache (it also resets the sidereal mode to Fagan/Bradley, which the per-chart setting above already covers).
- **Misuse is unrepresentable:** the `extern "C"` functions are private to the wrapper crate; the only public entry point is the pool's async `calculate(request)`. The worker-side handle is `!Send`/`!Sync`. No Rust API exists for house system 'I' (Sunshine; writes non-TLS `static saved_sundec`, `swehouse.c:636`), `SE_SIDBIT_PREC_ORIG` (leaves `astro_models` changed for later calls), or `swe_get_library_path` (non-TLS `static Dl_info`).
- **Environment:** the server refuses to start if `SE_EPHE_PATH` is set, because the library lets it override the path argument (`sweph.c:1333`).
- **Crash handling:** one process; a crash in the C library takes the process down and the supervisor restarts it. Subprocess isolation is not worth its cost until crashes are observed.
- **Verification:** a test runs concurrent charts with different ayanamshas, each of which must match its single-threaded reference value. CI runs it on Debian; it also runs on macOS.

## Considered options

- **`spawn_blocking` per chart**: rejected for the reasons above.
- **`TLSOFF` and one worker everywhere**: simplest and identical on every platform, but caps each process at one chart at a time. Rejected because Linux TLS works and production is Linux.
- **Upstream `v2.10.3bfinal` unpatched, with N = 1 forced on macOS**: the previous decision. Rejected once the TLS patch was written and tested: the fork is two small commits, and it gives the same thread model, and the concurrency test, on every platform we run.

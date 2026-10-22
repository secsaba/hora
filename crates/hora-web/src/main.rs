//! The hora web server, the only caller of `hora-swe`.

use std::ffi::OsString;
use std::future::IntoFuture;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::time::Duration;

use anyhow::Context;

mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_vars(|name| std::env::var_os(name))?;
    hora_swe::configure(config.ephe_path, config.workers, config.queue)
        .map_err(anyhow::Error::msg)?;
    // A wrong path or missing files fail any chart (no Moshier fallback), so a bad deploy stops here.
    hora_swe::calculate(hora_swe::Request {
        year: 2000,
        month: 1,
        day: 1,
        hour: 12,
        minute: 0,
        second: 0,
        calendar: Default::default(),
        utc_offset: hora_swe::UtcOffset::Stated(0),
        latitude: 0.0,
        longitude: 0.0,
        altitude: 0.0,
        ayanamsa: Default::default(),
        node: Default::default(),
        sunrise_definition: Default::default(),
    })
    .await
    .context("a first chart with HORA_EPHE_PATH failed")?;
    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .with_context(|| format!("binding {}", config.bind))?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .context("installing the SIGTERM handler")?;
    tracing::info!("listening on {}", config.bind);
    let (signalled, on_signal) = tokio::sync::oneshot::channel();
    let server = axum::serve(listener, web::app())
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = terminate.recv() => {}
            }
            tracing::info!("shutting down");
            let _ = signalled.send(());
        })
        .into_future();
    // axum sets no header read timeout, so a slow client could hold graceful shutdown open forever (ADR 0002).
    tokio::select! {
        result = server => result?,
        _ = async {
            let _ = on_signal.await;
            tokio::time::sleep(SHUTDOWN_GRACE).await;
        } => tracing::warn!("requests still open after {SHUTDOWN_GRACE:?}; closing them"),
    }
    // Blocks until every worker has closed its files, so off the async threads.
    tokio::task::spawn_blocking(hora_swe::shutdown).await?;
    Ok(())
}

const SHUTDOWN_GRACE: Duration = Duration::from_secs(10);

#[derive(Debug)]
struct Config {
    ephe_path: OsString,
    workers: NonZeroUsize,
    queue: NonZeroUsize,
    bind: SocketAddr,
}

impl Config {
    /// Reads `var` (the environment, outside tests); every value is checked here (the ephemeris path by a first chart in `main`), so a bad one stops startup.
    fn from_vars(var: impl Fn(&str) -> Option<OsString>) -> anyhow::Result<Self> {
        // ADR 0001: the library would take it over HORA_EPHE_PATH (sweph.c:1327).
        if var("SE_EPHE_PATH").is_some() {
            anyhow::bail!("SE_EPHE_PATH is set; unset it and give the path in HORA_EPHE_PATH");
        }
        let parsed = |name: &str| -> anyhow::Result<Option<String>> {
            var(name)
                .map(|v| {
                    v.into_string()
                        .map_err(|_| anyhow::anyhow!("{name} is not UTF-8"))
                })
                .transpose()
        };
        let ephe_path = var("HORA_EPHE_PATH").context("HORA_EPHE_PATH is not set")?;
        let workers = match parsed("HORA_WORKERS")? {
            Some(v) => v
                .parse()
                .context("HORA_WORKERS is not a whole number above 0")?,
            None => std::thread::available_parallelism()?,
        };
        let queue = match parsed("HORA_QUEUE")? {
            Some(v) => v
                .parse()
                .context("HORA_QUEUE is not a whole number above 0")?,
            None => workers.saturating_mul(NonZeroUsize::new(4).expect("4 > 0")),
        };
        let bind = match parsed("HORA_BIND")? {
            Some(v) => v
                .parse()
                .context("HORA_BIND is not an IP address and port")?,
            None => SocketAddr::from(([127, 0, 0, 1], 3000)),
        };
        Ok(Config {
            ephe_path,
            workers,
            queue,
            bind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars<'a>(pairs: &'a [(&str, &str)]) -> impl Fn(&str) -> Option<OsString> + 'a {
        |name| {
            pairs
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| v.into())
        }
    }

    #[test]
    fn config_refuses_se_ephe_path() {
        let err = Config::from_vars(vars(&[("HORA_EPHE_PATH", "/e"), ("SE_EPHE_PATH", "/x")]))
            .unwrap_err();
        assert!(err.to_string().contains("SE_EPHE_PATH"), "{err}");
    }

    #[test]
    fn config_defaults_to_one_worker_per_core_and_a_queue_of_four_each() {
        let config = Config::from_vars(vars(&[("HORA_EPHE_PATH", "/e")])).unwrap();
        let n = std::thread::available_parallelism().unwrap();
        assert_eq!(config.ephe_path, "/e");
        assert_eq!(config.workers, n);
        assert_eq!(config.queue.get(), 4 * n.get());
        assert!(config.bind.ip().is_loopback());
    }

    #[test]
    fn config_reads_workers_queue_and_bind() {
        let config = Config::from_vars(vars(&[
            ("HORA_EPHE_PATH", "/e"),
            ("HORA_WORKERS", "3"),
            ("HORA_QUEUE", "5"),
            ("HORA_BIND", "0.0.0.0:8080"),
        ]))
        .unwrap();
        assert_eq!((config.workers.get(), config.queue.get()), (3, 5));
        assert_eq!(config.bind, "0.0.0.0:8080".parse().unwrap());
    }

    #[test]
    fn config_refuses_a_missing_path_zero_workers_or_queue_and_garbage() {
        for pairs in [
            &[][..],
            &[("HORA_EPHE_PATH", "/e"), ("HORA_WORKERS", "0")],
            &[("HORA_EPHE_PATH", "/e"), ("HORA_QUEUE", "0")],
            &[("HORA_EPHE_PATH", "/e"), ("HORA_WORKERS", "x")],
            &[("HORA_EPHE_PATH", "/e"), ("HORA_BIND", "localhost")],
        ] {
            assert!(Config::from_vars(vars(pairs)).is_err(), "{pairs:?}");
        }
    }
}

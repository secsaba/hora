//! Shuts the pool down; its own process, since a shut-down shared pool would fail other tests.

use std::task::{Context, Waker};

use hora_swe::{
    Ayanamsha, Calendar, Error, NodeType, Request, SunriseDefinition, calculate, shutdown,
};

#[tokio::test]
async fn after_shutdown_every_job_is_answered_and_new_ones_are_overloaded() {
    let request = Request {
        year: 2000,
        month: 4,
        day: 9,
        hour: 21,
        minute: 55,
        second: 0,
        calendar: Calendar::Gregorian,
        latitude: 42.5,
        longitude: -71.2,
        altitude: 0.0,
        ayanamsha: Ayanamsha::TrueChitra,
        node: NodeType::Mean,
        sunrise_definition: SunriseDefinition::UpperLimbApparent,
    };
    let n = std::thread::available_parallelism().unwrap().get();
    // Fills the default queue (4 × N) without overflowing it, so every Overloaded comes from shutdown.
    let mut charts: Vec<_> = (0..4 * n)
        .map(|_| Box::pin(calculate(request.clone())))
        .collect();
    // One poll each enqueues the job before the queue closes.
    let mut cx = Context::from_waker(Waker::noop());
    for chart in &mut charts {
        assert!(chart.as_mut().poll(&mut cx).is_pending());
    }
    // Hangs here if a worker never exits.
    shutdown();

    // Whether any were still queued is timing; the unit test in lib.rs pins that on a private pool.
    for chart in charts {
        match chart.await {
            Ok(_) | Err(Error::Overloaded) => {}
            Err(e) => panic!("{e}"),
        }
    }
    assert_eq!(calculate(request).await.unwrap_err(), Error::Overloaded);
}

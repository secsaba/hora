//! Floods the pool; its own process, since an overloaded shared pool would fail other tests.

use hora_swe::{Ayanamsha, Calendar, Error, NodeType, Request, calculate};

#[tokio::test]
async fn a_flood_past_the_queue_length_is_answered_overloaded() {
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
    };
    // 20 × capacity (4 × N queued, N being calculated): the first join_next polls every task in one
    // burst, µs each, while each worker takes ms per chart.
    let n = std::thread::available_parallelism().unwrap().get();
    let mut charts = tokio::task::JoinSet::new();
    for _ in 0..20 * 5 * n {
        charts.spawn(calculate(request.clone()));
    }
    let (mut ok, mut overloaded) = (0, 0);
    while let Some(chart) = charts.join_next().await {
        match chart.unwrap() {
            Ok(_) => ok += 1,
            Err(Error::Overloaded) => overloaded += 1,
            Err(e) => panic!("{e}"),
        }
    }
    assert!(ok > 0 && overloaded > 0, "ok {ok}, overloaded {overloaded}");
}

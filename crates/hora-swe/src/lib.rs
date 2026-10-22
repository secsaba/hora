//! The only code in hora that calls Swiss Ephemeris, on a fixed pool of worker threads (ADR 0001).

use std::ffi::{CStr, CString, c_char, c_int};
use std::marker::PhantomData;
use std::os::unix::ffi::OsStringExt;
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::{env, fmt, thread};

use tokio::sync::oneshot;

/// One chart's moment (UT, in `calendar`), place (degrees, east and north positive; metres) and choices.
#[derive(Debug, Clone)]
pub struct Request {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub calendar: Calendar,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub ayanamsha: Ayanamsha,
    pub node: NodeType,
}

/// The calendar of the request's date; the discriminants are the library's gregflag values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Calendar {
    Julian = 0,
    #[default]
    Gregorian = 1,
}

/// The ayanamshas hora offers; the discriminants are the library's SE_SIDM_* values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Ayanamsha {
    Raman = 3,
    #[default]
    TrueChitra = 27,
    TruePushya = 29,
    TrueMula = 35,
}

/// Where Rahu is placed; the discriminants are the library's body numbers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NodeType {
    #[default]
    Mean = 10,
    True = 11,
}

/// A sidereal longitude (degrees) and its speed (degrees per day).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Position {
    pub longitude: f64,
    pub speed: f64,
}

/// Rahu is the requested node; Ketu is left to the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct Grahas {
    pub sun: Position,
    pub moon: Position,
    pub mars: Position,
    pub mercury: Position,
    pub jupiter: Position,
    pub venus: Position,
    pub saturn: Position,
    pub rahu: Position,
}

/// Raw values for one chart, sidereal (the requested ayanamsha), in degrees.
#[derive(Debug, Clone, PartialEq)]
pub struct Chart {
    pub grahas: Grahas,
    pub lagna: f64,
    pub ayanamsha: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The request is outside what hora calculates; the one error shown to the user.
    InvalidInput,
    /// The library failed or fell back to its Moshier model; the details are logged, not returned.
    Ephemeris,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidInput => f.write_str("invalid input"),
            Error::Ephemeris => f.write_str("ephemeris calculation failed"),
        }
    }
}

impl std::error::Error for Error {}

/// Calculates one chart on the process-wide pool.
pub async fn calculate(request: Request) -> Result<Chart, Error> {
    static POOL: OnceLock<mpsc::Sender<Job>> = OnceLock::new();
    let pool = || {
        POOL.get_or_init(|| {
            // shortcut: path from the environment and N = available_parallelism; take both from the server's config once that crate exists.
            let path = env::var_os("HORA_EPHE_PATH").expect("HORA_EPHE_PATH is set");
            let n = thread::available_parallelism().map_or(1, |n| n.get());
            spawn(
                CString::new(path.into_vec()).expect("env vars hold no NUL"),
                n,
            )
        })
    };
    calculate_on(pool, request).await
}

struct Job {
    request: Request,
    reply: oneshot::Sender<Result<Chart, Error>>,
}

/// Starts the pool only for a valid request, so a bad one never reads the environment or spawns workers.
async fn calculate_on<'a>(
    pool: impl FnOnce() -> &'a mpsc::Sender<Job>,
    request: Request,
) -> Result<Chart, Error> {
    validate(&request)?;
    let (reply, answer) = oneshot::channel();
    if pool().send(Job { request, reply }).is_err() {
        tracing::error!("Swiss Ephemeris pool has no workers left");
        return Err(Error::Ephemeris);
    }
    answer.await.unwrap_or_else(|_| {
        tracing::error!("Swiss Ephemeris worker dropped a job");
        Err(Error::Ephemeris)
    })
}

/// The request rules of issue #2; every request passes here before it is queued.
fn validate(r: &Request) -> Result<(), Error> {
    // Range checks fail on NaN, so these also reject NaN and the infinities.
    let place = r.latitude > -90.0
        && r.latitude < 90.0
        && (-180.0..=180.0).contains(&r.longitude)
        && (-500.0..=9000.0).contains(&r.altitude);
    let time =
        (0..24).contains(&r.hour) && (0..60).contains(&r.minute) && (0..60).contains(&r.second);
    // Bounding month and day keeps swe_revjul's C int casts in range for any i32 input.
    let date =
        (1200..=2399).contains(&r.year) && (1..=12).contains(&r.month) && (1..=31).contains(&r.day);
    if !(place && time && date) {
        return Err(Error::InvalidInput);
    }
    let calendar = match r.calendar {
        Calendar::Gregorian => b'g',
        Calendar::Julian => b'j',
    };
    let mut jd = 0.0;
    // SAFETY: valid out-pointer; pure date arithmetic, which ADR 0001 allows off the workers.
    let ret = unsafe {
        ffi::swe_date_conversion(r.year, r.month, r.day, 0.0, calendar as c_char, &mut jd)
    };
    // Both calendars are proleptic in the library, so only this rejects Gregorian dates before 1582-10-15.
    let before_gregorian = r.calendar == Calendar::Gregorian && jd < FIRST_GREGORIAN_DAY;
    // Julian dates late in 2399 fall past the files' end, where the library would fall back to Moshier.
    let after_files = jd >= FILES_END;
    if ret != ffi::OK || before_gregorian || after_files {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

/// 1582-10-15 0h UT, the first day of the Gregorian calendar.
const FIRST_GREGORIAN_DAY: f64 = 2_299_160.5;
/// 2400-01-01 0h UT (Gregorian), where the ephemeris files' 1200–2399 ends.
const FILES_END: f64 = 2_597_641.5;

// shortcut: unbounded queue, no wait deadline or is_closed() drop, no shutdown()/swe_close; issue #9 adds them.
fn spawn(path: CString, workers: usize) -> mpsc::Sender<Job> {
    // Empty or longer paths make swe_set_ephe_path silently use its compiled-in default (sweph.c:1331).
    assert!(!path.as_bytes().is_empty(), "ephemeris path is empty");
    assert!(path.as_bytes().len() <= 242, "ephemeris path too long");
    // shortcut: SE_EPHE_PATH overrides this path (sweph.c:1327); the server crate refuses to start with it set.
    let (tx, rx) = mpsc::channel::<Job>();
    let rx = Arc::new(Mutex::new(rx));
    for i in 0..workers {
        let (path, rx) = (path.clone(), Arc::clone(&rx));
        thread::Builder::new()
            .name(format!("swe-{i}"))
            .spawn(move || {
                let swe = Swe::new(&path);
                loop {
                    let job = rx.lock().expect("no worker panics holding the lock").recv();
                    let Ok(job) = job else { return };
                    let _ = job.reply.send(swe.chart(&job.request));
                }
            })
            .expect("spawn Swiss Ephemeris worker");
    }
    tx
}

/// The library's state on this thread (it is thread-local), so this handle never leaves it.
struct Swe(PhantomData<*const ()>);

impl Swe {
    fn new(path: &CStr) -> Self {
        // SAFETY: path is a NUL-terminated string the library only reads.
        unsafe { ffi::swe_set_ephe_path(path.as_ptr()) };
        Swe(PhantomData)
    }

    fn chart(&self, r: &Request) -> Result<Chart, Error> {
        let mut serr = [0 as c_char; 256];
        let hour = f64::from(r.hour) + f64::from(r.minute) / 60.0 + f64::from(r.second) / 3600.0;
        // SAFETY: plain values in, no pointers.
        let jd = unsafe {
            ffi::swe_set_sid_mode(r.ayanamsha as i32, 0.0, 0.0);
            // ADR 0001: set per chart so no chart inherits the previous one's topo state; positions stay geocentric.
            ffi::swe_set_topo(r.longitude, r.latitude, r.altitude);
            ffi::swe_julday(r.year, r.month, r.day, hour, r.calendar as c_int)
        };

        // TRUEPOS (no light-time, no aberration) is what the book's values use; without it every graha is 20–60″ off.
        let flags = ffi::SEFLG_SWIEPH | ffi::SEFLG_SIDEREAL | ffi::SEFLG_TRUEPOS | ffi::SEFLG_SPEED;
        let mut positions = [Position::default(); 8];
        for (pos, ipl) in positions
            .iter_mut()
            .zip(ffi::GRAHAS.into_iter().chain([r.node as i32]))
        {
            let mut xx = [0.0; 6];
            // SAFETY: xx holds the 6 doubles and serr the 256 bytes the library writes.
            let ret =
                unsafe { ffi::swe_calc_ut(jd, ipl, flags, xx.as_mut_ptr(), serr.as_mut_ptr()) };
            check(ret, &mut serr)?;
            *pos = Position {
                longitude: xx[0],
                speed: xx[3],
            };
        }
        let [sun, moon, mars, mercury, jupiter, venus, saturn, rahu] = positions;

        // The value swe_calc_ut subtracts under SEFLG_SIDEREAL (sweph.c:6095); swe_get_ayanamsa_ut can silently give 0.
        let mut ayanamsha = 0.0;
        let aya_flags = ffi::SEFLG_SWIEPH | ffi::SEFLG_NONUT | ffi::SEFLG_TRUEPOS;
        // SAFETY: valid out-pointers; serr holds 256 bytes.
        let ret = unsafe {
            ffi::swe_get_ayanamsa_ex_ut(jd, aya_flags, &mut ayanamsha, serr.as_mut_ptr())
        };
        check(ret, &mut serr)?;

        // Equal houses never fail (Placidus and Koch do inside the polar circle); only ascmc[0] is used.
        let (mut cusps, mut ascmc) = ([0.0; 13], [0.0; 10]);
        // SAFETY: cusps holds 13 doubles and ascmc 10, as the library requires for this house system.
        let ret = unsafe {
            ffi::swe_houses_ex(
                jd,
                0,
                r.latitude,
                r.longitude,
                c_int::from(b'E'),
                cusps.as_mut_ptr(),
                ascmc.as_mut_ptr(),
            )
        };
        if ret < 0 {
            tracing::error!("swe_houses_ex failed");
            return Err(Error::Ephemeris);
        }
        // Deliberate nutation mismatch, do not "fix": the book's method (chart 1: 10°49′44.53″); sidereal houses add 15.87″.
        let lagna = (ascmc[0] - ayanamsha).rem_euclid(360.0);

        Ok(Chart {
            grahas: Grahas {
                sun,
                moon,
                mars,
                mercury,
                jupiter,
                venus,
                saturn,
                rahu,
            },
            lagna,
            ayanamsha,
        })
    }
}

/// Fails on a C error or on any ephemeris other than the .se1 files, e.g. a Moshier fallback.
fn check(ret: i32, serr: &mut [c_char; 256]) -> Result<(), Error> {
    let failed = ret < 0 || ret & ffi::EPHEMERIS_MASK != ffi::SEFLG_SWIEPH;
    if failed {
        let bytes = serr.map(|c| c as u8);
        let msg = CStr::from_bytes_until_nul(&bytes)
            .map(CStr::to_string_lossy)
            .unwrap_or_default();
        tracing::error!(ret, serr = %msg, "Swiss Ephemeris call failed");
    }
    // Not every call clears serr on entry (swe_get_ayanamsa_ex_ut), so a stale message would be logged.
    serr[0] = 0;
    if failed {
        Err(Error::Ephemeris)
    } else {
        Ok(())
    }
}

mod ffi {
    use std::ffi::{c_char, c_double, c_int};

    // The reference tests pass it literally, so a wrong `Calendar` discriminant fails there.
    #[cfg(test)]
    pub const SE_GREG_CAL: c_int = 1;
    pub const OK: c_int = 0;
    // Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, then the requested node: the order of `Grahas`.
    pub const GRAHAS: [i32; 7] = [0, 1, 4, 2, 5, 3, 6];

    pub const SEFLG_JPLEPH: i32 = 1;
    pub const SEFLG_SWIEPH: i32 = 2;
    pub const SEFLG_MOSEPH: i32 = 4;
    pub const SEFLG_TRUEPOS: i32 = 16;
    pub const SEFLG_NONUT: i32 = 64;
    pub const SEFLG_SPEED: i32 = 256;
    pub const SEFLG_SIDEREAL: i32 = 64 * 1024;
    // SEFLG_EPHMASK is private to swecl.c.
    pub const EPHEMERIS_MASK: i32 = SEFLG_JPLEPH | SEFLG_SWIEPH | SEFLG_MOSEPH;

    unsafe extern "C" {
        pub fn swe_set_ephe_path(path: *const c_char);
        pub fn swe_set_sid_mode(sid_mode: i32, t0: c_double, ayan_t0: c_double);
        pub fn swe_set_topo(geolon: c_double, geolat: c_double, geoalt: c_double);
        pub fn swe_julday(
            year: c_int,
            month: c_int,
            day: c_int,
            hour: c_double,
            gregflag: c_int,
        ) -> c_double;
        pub fn swe_date_conversion(
            year: c_int,
            month: c_int,
            day: c_int,
            hour: c_double,
            calendar: c_char,
            jd: *mut c_double,
        ) -> c_int;
        pub fn swe_calc_ut(
            tjd_ut: c_double,
            ipl: i32,
            iflag: i32,
            xx: *mut c_double,
            serr: *mut c_char,
        ) -> i32;
        pub fn swe_get_ayanamsa_ex_ut(
            tjd_ut: c_double,
            iflag: i32,
            daya: *mut c_double,
            serr: *mut c_char,
        ) -> i32;
        pub fn swe_houses_ex(
            tjd_ut: c_double,
            iflag: i32,
            geolat: c_double,
            geolon: c_double,
            hsys: c_int,
            cusps: *mut c_double,
            ascmc: *mut c_double,
        ) -> c_int;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_ephemeris_files_fail_instead_of_falling_back_to_moshier() {
        // Its own pool: the shared one points at the real files.
        let pool = spawn(c"/nonexistent/hora-ephe".to_owned(), 1);
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
        assert_eq!(
            calculate_on(|| &pool, request).await.unwrap_err(),
            Error::Ephemeris
        );
    }
    /// The golden charts' moments and places, with the given choices.
    fn golden(ayanamsha: Ayanamsha, node: NodeType) -> [Request; 3] {
        let at = |year, month, day, hour, minute, latitude, longitude| Request {
            year,
            month,
            day,
            hour,
            minute,
            second: 0,
            calendar: Calendar::Gregorian,
            latitude,
            longitude,
            altitude: 0.0,
            ayanamsha,
            node,
        };
        [
            at(2000, 4, 9, 21, 55, 42.5, -(71.0 + 12.0 / 60.0)),
            at(1926, 12, 24, 23, 42, 26.0 + 14.0 / 60.0, 78.0 + 10.0 / 60.0),
            at(1968, 3, 19, 13, 0, 17.0 + 7.0 / 60.0, 82.0 + 19.0 / 60.0),
        ]
    }

    /// Runs `f` alone on a fresh thread with the library pointed at the real files.
    fn single_threaded<T: Send + 'static>(f: impl FnOnce(&Swe) -> T + Send + 'static) -> T {
        thread::spawn(move || {
            let path = env::var_os("HORA_EPHE_PATH").expect("HORA_EPHE_PATH is set");
            f(&Swe::new(&CString::new(path.into_vec()).unwrap()))
        })
        .join()
        .unwrap()
    }

    #[tokio::test]
    async fn true_node_matches_swe_calc_ut() {
        let [request, ..] = golden(Ayanamsha::TrueChitra, NodeType::True);
        let mean = calculate(Request {
            node: NodeType::Mean,
            ..request.clone()
        })
        .await
        .unwrap();
        let chart = calculate(request).await.unwrap();

        let reference = single_threaded(|_| {
            let mut xx = [0.0; 6];
            let mut serr = [0 as c_char; 256];
            let flags = ffi::SEFLG_SWIEPH | ffi::SEFLG_SIDEREAL | ffi::SEFLG_TRUEPOS;
            // SAFETY: plain values in; xx holds 6 doubles and serr 256 bytes.
            let ret = unsafe {
                // Literal SE_SIDM_TRUE_CITRA and SE_TRUE_NODE, not the enums, so a wrong discriminant fails here.
                ffi::swe_set_sid_mode(27, 0.0, 0.0);
                let jd = ffi::swe_julday(2000, 4, 9, 21.0 + 55.0 / 60.0, ffi::SE_GREG_CAL);
                ffi::swe_calc_ut(jd, 11, flags, xx.as_mut_ptr(), serr.as_mut_ptr())
            };
            check(ret, &mut serr).unwrap();
            xx[0]
        });

        assert_eq!(chart.grahas.rahu.longitude, reference);
        // The true node oscillates up to ~1.7° around the mean one; guards against the choice being ignored.
        assert!((chart.grahas.rahu.longitude - mean.grahas.rahu.longitude).abs() > 0.01);
    }

    #[tokio::test]
    async fn each_ayanamsha_matches_swe_get_ayanamsa_ex_ut() {
        // Literal SE_SIDM_* values, not the enums, so a wrong discriminant fails here.
        let choices = [
            (Ayanamsha::Raman, 3),
            (Ayanamsha::TrueChitra, 27),
            (Ayanamsha::TruePushya, 29),
            (Ayanamsha::TrueMula, 35),
        ];
        for (ayanamsha, sid_mode) in choices {
            let [request, ..] = golden(ayanamsha, NodeType::Mean);
            let chart = calculate(request).await.unwrap();

            let reference = single_threaded(move |_| {
                let mut daya = 0.0;
                let mut serr = [0 as c_char; 256];
                let flags = ffi::SEFLG_SWIEPH | ffi::SEFLG_NONUT | ffi::SEFLG_TRUEPOS;
                // SAFETY: plain values in; valid out-pointer; serr holds 256 bytes.
                let ret = unsafe {
                    ffi::swe_set_sid_mode(sid_mode, 0.0, 0.0);
                    let jd = ffi::swe_julday(2000, 4, 9, 21.0 + 55.0 / 60.0, ffi::SE_GREG_CAL);
                    ffi::swe_get_ayanamsa_ex_ut(jd, flags, &mut daya, serr.as_mut_ptr())
                };
                check(ret, &mut serr).unwrap();
                daya
            });

            assert_eq!(chart.ayanamsha, reference, "{ayanamsha:?}");
        }
    }

    /// ADR 0001's verification: no chart inherits another chart's ayanamsha, node or place from a worker.
    #[tokio::test]
    async fn concurrent_charts_match_their_single_threaded_reference() {
        let requests: Vec<Request> = [
            Ayanamsha::Raman,
            Ayanamsha::TrueChitra,
            Ayanamsha::TrueMula,
            Ayanamsha::TruePushya,
        ]
        .into_iter()
        .flat_map(|a| [NodeType::Mean, NodeType::True].map(|n| golden(a, n)))
        .flatten()
        .collect();
        let references = {
            let requests = requests.clone();
            single_threaded(move |swe| {
                requests
                    .iter()
                    .map(|r| swe.chart(r).unwrap())
                    .collect::<Vec<_>>()
            })
        };

        let mut charts = tokio::task::JoinSet::new();
        for round in 0..8 {
            for (i, request) in requests.iter().enumerate() {
                let request = request.clone();
                charts.spawn(async move { (round, i, calculate(request).await.unwrap()) });
            }
        }
        while let Some(joined) = charts.join_next().await {
            let (round, i, chart) = joined.unwrap();
            assert_eq!(chart, references[i], "round {round}: {:?}", requests[i]);
        }
    }

    /// Golden chart 1 on the given date and time.
    fn on(calendar: Calendar, [year, month, day, hour, minute, second]: [i32; 6]) -> Request {
        let [chart_1, ..] = golden(Ayanamsha::TrueChitra, NodeType::Mean);
        Request {
            year,
            month,
            day,
            hour,
            minute,
            second,
            calendar,
            ..chart_1
        }
    }

    async fn assert_invalid(request: Request) {
        let got = calculate(request.clone()).await;
        assert_eq!(got.unwrap_err(), Error::InvalidInput, "{request:?}");
    }

    #[tokio::test]
    async fn thirty_february_is_rejected_in_both_calendars() {
        for calendar in [Calendar::Gregorian, Calendar::Julian] {
            assert_invalid(on(calendar, [2000, 2, 30, 12, 0, 0])).await;
        }
    }

    #[tokio::test]
    async fn a_date_in_the_gregorian_gap_is_only_valid_as_julian() {
        assert_invalid(on(Calendar::Gregorian, [1582, 10, 10, 12, 0, 0])).await;
        calculate(on(Calendar::Julian, [1582, 10, 10, 12, 0, 0]))
            .await
            .unwrap();
        // The first Gregorian day.
        calculate(on(Calendar::Gregorian, [1582, 10, 15, 0, 0, 0]))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn the_julian_day_before_the_reform_is_the_gregorian_first_day() {
        // Pins the Julian path end to end: Julian 1582-10-05 is Gregorian 1582-10-15.
        let julian = calculate(on(Calendar::Julian, [1582, 10, 5, 12, 0, 0])).await;
        let gregorian = calculate(on(Calendar::Gregorian, [1582, 10, 15, 12, 0, 0])).await;
        assert_eq!(julian.unwrap(), gregorian.unwrap());
    }

    #[tokio::test]
    async fn years_outside_1200_to_2399_are_rejected() {
        // Julian at the start, because a Gregorian date before 1582-10-15 is rejected.
        assert_invalid(on(Calendar::Julian, [1199, 12, 31, 23, 59, 59])).await;
        assert_invalid(on(Calendar::Gregorian, [2400, 1, 1, 0, 0, 0])).await;
        // Julian 2399-12-16 is Gregorian 2400-01-01, past the files: InvalidInput, not Ephemeris.
        assert_invalid(on(Calendar::Julian, [2399, 12, 16, 0, 0, 0])).await;
        calculate(on(Calendar::Julian, [2399, 12, 15, 23, 59, 59]))
            .await
            .unwrap();
        calculate(on(Calendar::Julian, [1200, 1, 1, 0, 0, 0]))
            .await
            .unwrap();
        calculate(on(Calendar::Gregorian, [2399, 12, 31, 23, 59, 59]))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn out_of_range_times_are_rejected() {
        for time in [
            [24, 0, 0],
            [-1, 0, 0],
            [0, 60, 0],
            [0, -1, 0],
            [0, 0, 60],
            [0, 0, -1],
        ] {
            let [hour, minute, second] = time;
            assert_invalid(on(Calendar::Gregorian, [2000, 4, 9, hour, minute, second])).await;
        }
    }

    #[tokio::test]
    async fn out_of_range_places_are_rejected() {
        let [chart_1, ..] = golden(Ayanamsha::TrueChitra, NodeType::Mean);
        let bad = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
        for latitude in [90.0, -90.0].into_iter().chain(bad) {
            assert_invalid(Request {
                latitude,
                ..chart_1.clone()
            })
            .await;
        }
        for longitude in [180.001, -180.001].into_iter().chain(bad) {
            assert_invalid(Request {
                longitude,
                ..chart_1.clone()
            })
            .await;
        }
        for altitude in [-500.1, 9000.1].into_iter().chain(bad) {
            assert_invalid(Request {
                altitude,
                ..chart_1.clone()
            })
            .await;
        }
        for (latitude, longitude, altitude) in [(89.999, 180.0, -500.0), (-89.999, -180.0, 9000.0)]
        {
            let request = Request {
                latitude,
                longitude,
                altitude,
                ..chart_1.clone()
            };
            calculate(request).await.unwrap();
        }
    }

    #[tokio::test]
    async fn invalid_requests_never_start_or_reach_the_pool() {
        let request = on(Calendar::Gregorian, [2000, 2, 30, 12, 0, 0]);
        assert_eq!(
            calculate_on(
                || unreachable!("an invalid request started the pool"),
                request
            )
            .await
            .unwrap_err(),
            Error::InvalidInput
        );
    }
}

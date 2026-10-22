//! The only code in hora that calls Swiss Ephemeris, on a fixed pool of worker threads (ADR 0001).

use std::ffi::{CStr, CString, c_char, c_int};
use std::marker::PhantomData;
use std::os::unix::ffi::OsStringExt;
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::{env, fmt, thread};

use tokio::sync::oneshot;

/// One chart's moment (UT, Gregorian calendar) and place (degrees, east and north positive; metres).
#[derive(Debug, Clone)]
pub struct Request {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

/// A sidereal longitude (degrees) and its speed (degrees per day).
#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub longitude: f64,
    pub speed: f64,
}

/// Rahu is the mean node; Ketu is left to the caller.
#[derive(Debug, Clone)]
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

/// Raw values for one chart, sidereal (True Chitra), in degrees.
#[derive(Debug, Clone)]
pub struct Chart {
    pub grahas: Grahas,
    pub lagna: f64,
    pub ayanamsha: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The library failed or fell back to its Moshier model; the details are logged, not returned.
    Ephemeris,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Ephemeris => f.write_str("ephemeris calculation failed"),
        }
    }
}

impl std::error::Error for Error {}

/// Calculates one chart on the process-wide pool.
// shortcut: no request validation (bad dates/coords reach C); issue #8 adds InvalidInput.
pub async fn calculate(request: Request) -> Result<Chart, Error> {
    static POOL: OnceLock<mpsc::Sender<Job>> = OnceLock::new();
    let pool = POOL.get_or_init(|| {
        // shortcut: path from the environment and N = available_parallelism; take both from the server's config once that crate exists.
        let path = env::var_os("HORA_EPHE_PATH").expect("HORA_EPHE_PATH is set");
        let n = thread::available_parallelism().map_or(1, |n| n.get());
        spawn(
            CString::new(path.into_vec()).expect("env vars hold no NUL"),
            n,
        )
    });
    calculate_on(pool, request).await
}

struct Job {
    request: Request,
    reply: oneshot::Sender<Result<Chart, Error>>,
}

async fn calculate_on(pool: &mpsc::Sender<Job>, request: Request) -> Result<Chart, Error> {
    let (reply, answer) = oneshot::channel();
    if pool.send(Job { request, reply }).is_err() {
        tracing::error!("Swiss Ephemeris pool has no workers left");
        return Err(Error::Ephemeris);
    }
    answer.await.unwrap_or_else(|_| {
        tracing::error!("Swiss Ephemeris worker dropped a job");
        Err(Error::Ephemeris)
    })
}

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
            ffi::swe_set_sid_mode(ffi::SE_SIDM_TRUE_CITRA, 0.0, 0.0);
            // ADR 0001: set per chart so no chart inherits the previous one's topo state; positions stay geocentric.
            ffi::swe_set_topo(r.longitude, r.latitude, r.altitude);
            ffi::swe_julday(r.year, r.month, r.day, hour, ffi::SE_GREG_CAL)
        };

        // TRUEPOS (no light-time, no aberration) is what the book's values use; without it every graha is 20–60″ off.
        let flags = ffi::SEFLG_SWIEPH | ffi::SEFLG_SIDEREAL | ffi::SEFLG_TRUEPOS | ffi::SEFLG_SPEED;
        let mut positions = [Position::default(); 8];
        for (pos, ipl) in positions.iter_mut().zip(ffi::GRAHAS) {
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

    pub const SE_GREG_CAL: c_int = 1;
    pub const SE_SIDM_TRUE_CITRA: i32 = 27;
    // Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, mean node: the order of `Grahas`.
    pub const GRAHAS: [i32; 8] = [0, 1, 4, 2, 5, 3, 6, 10];

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
            latitude: 42.5,
            longitude: -71.2,
            altitude: 0.0,
        };
        assert_eq!(
            calculate_on(&pool, request).await.unwrap_err(),
            Error::Ephemeris
        );
    }
}

//! The only code in hora that calls Swiss Ephemeris, on a fixed pool of worker threads (ADR 0001).

use std::ffi::{CStr, CString, OsString, c_char, c_int};
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::ops::RangeInclusive;
use std::os::unix::ffi::OsStringExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock, mpsc};
use std::time::{Duration, Instant};
use std::{env, fmt, thread};

use tokio::sync::oneshot;

/// One chart's birth time (local clock time, in `calendar`) with its UTC offset, place (degrees, east and north positive; metres) and choices.
#[derive(Debug, Clone)]
pub struct Request {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub calendar: Calendar,
    pub utc_offset: UtcOffset,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub ayanamsa: Ayanamsa,
    pub node: NodeType,
    pub sunrise_definition: SunriseDefinition,
}

/// The calendar of the request's date.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Calendar {
    Julian,
    #[default]
    Gregorian,
}

/// How far the birth time is ahead of UT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtcOffset {
    /// Seconds, within −12:00 … +14:00; whole seconds, as historical offsets such as Amsterdam's +00:19:32 need.
    Stated(i32),
    /// The birth place's longitude / 15 hours, exact.
    LocalMean,
    /// Local mean time plus the equation of time, which the worker takes from the ephemeris.
    LocalApparent,
}

/// The ayanamsas hora offers; the discriminants are the library's SE_SIDM_* values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Ayanamsa {
    /// The book's "Chitrapaksha/Lahiri" (ch. 1, §1.3.13); its printed example charts match it (ADR 0003).
    #[default]
    Lahiri = 1,
    Raman = 3,
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

/// Which point of the Sun's disc rises, with or without refraction; the discriminants are the library's `rsmi` bits.
/// The four are the options of the author's software, Jagannatha Hora.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SunriseDefinition {
    CentreTrue = 256 | 512,
    UpperLimbTrue = 512,
    CentreApparent = 256,
    /// The book recommends the upper limb, the first visible ray (ch. 5 *Special Lagnas*, p. 48).
    #[default]
    UpperLimbApparent = 0,
}

/// The Vedic day of a birth (ch. 1 *Basic Concepts*, p. 17): its sunrise (the last at or before the birth),
/// sunset and the next sunrise, as UT Julian days and as Julian days of the birth time's clock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VedicDay {
    pub sunrise: f64,
    pub sunset: f64,
    pub next_sunrise: f64,
    pub clock_sunrise: f64,
    pub clock_sunset: f64,
    pub clock_next_sunrise: f64,
}

/// A Sun-Moon conjunction: its UT Julian day and the sidereal longitude (degrees) where it falls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewMoon {
    pub jd: f64,
    pub longitude: f64,
}

/// When a panchaanga limb running at the birth started and ends, as UT Julian days and as Julian days of the birth time's clock.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Span {
    pub start: f64,
    pub end: f64,
    pub clock_start: f64,
    pub clock_end: f64,
}

/// The spans of the tithi, the Moon's nakshatra, the Sun-Moon yoga and the karana running at the birth (ch. 1 §1.3.12).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Limbs {
    pub tithi: Span,
    pub nakshatra: Span,
    pub yoga: Span,
    pub karana: Span,
}

impl Limbs {
    /// The tithi, nakshatra, yoga and karana, in that order.
    pub fn spans(&self) -> [Span; 4] {
        [self.tithi, self.nakshatra, self.yoga, self.karana]
    }
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

/// The sky at one moment and place, which a chart is prepared from; sidereal (the requested ayanamsa), in degrees.
#[derive(Debug, Clone, PartialEq)]
pub struct Sky {
    /// The birth's UT Julian day.
    pub jd: f64,
    pub grahas: Grahas,
    pub lagna: f64,
    pub ayanamsa: f64,
    /// `None` where the sunrise before or after the birth is not found within about a day (polar latitudes).
    pub vedic_day: Option<VedicDay>,
    /// The new Moons starting the previous, the birth's and the next lunar month (ch. 1 §1.3.8.2); `None` near the files' ends.
    pub new_moons: Option<[NewMoon; 3]>,
    /// When the limbs of the panchaanga running at the birth started and end (ch. 1 §1.3.12).
    pub limbs: Limbs,
    /// The lagna at the start and middle of each eighth of the birth's day or night (ch. 4 §4.3), in order; `None` without a Vedic day.
    pub part_lagnas: Option<[[f64; 2]; 8]>,
}

/// A birth data rule a request can break.
// Exhaustive on purpose: a new rule must make every caller decide where to show it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// Strictly inside ±90°.
    Latitude,
    /// −180° … 180°.
    Longitude,
    /// −500 … 9000 m.
    Altitude,
    /// Hour 0–23, minute and second 0–59.
    BirthTime,
    /// A stated offset within −12:00 … +14:00.
    UtcOffset,
    /// No such day in the calendar, such as 30 February.
    ImpossibleDate,
    /// A Gregorian date before 1582-10-15.
    BeforeGregorian,
    /// The moment is outside the ephemeris files' window, less two days at each end.
    OutsideFiles,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The request is outside what hora calculates, with every rule it breaks; the one error the user is told about, in their language (Display is English, for logs).
    InvalidInput(Vec<Rule>),
    /// The library failed or fell back to its Moshier model; the details are logged, not returned.
    Ephemeris,
    /// The queue is full, the job waited in it too long, or the pool is shut down; the server answers 503 with `Retry-After`.
    Overloaded,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidInput(rules) => write!(f, "invalid input: {rules:?}"),
            Error::Ephemeris => f.write_str("ephemeris calculation failed"),
            Error::Overloaded => f.write_str("overloaded"),
        }
    }
}

impl std::error::Error for Error {}

/// Calculates the sky for one request on the process-wide pool.
pub async fn calculate(request: Request) -> Result<Sky, Error> {
    calculate_on(pool, request).await
}

/// Blocks until every worker has exited; `main` calls it once, after the runtime stops or via `spawn_blocking`.
pub fn shutdown() {
    // Leaves a closed pool even if none started, so a later `calculate` is `Overloaded`.
    POOL.get_or_init(Pool::default).shutdown();
}

/// Sets the ephemeris path, worker count and queue length; `main` calls it once, before the first `calculate`.
pub fn configure(
    path: OsString,
    workers: NonZeroUsize,
    queue: NonZeroUsize,
) -> Result<(), &'static str> {
    let path = ephe_path(path)?;
    if POOL.get().is_some() {
        return Err("the Swiss Ephemeris pool is already running");
    }
    CONFIG
        .set((path, workers.get(), queue.get()))
        .map_err(|_| "the Swiss Ephemeris pool is already configured")
}

/// Empty or longer paths make swe_set_ephe_path silently use its compiled-in default (sweph.c:1331).
fn ephe_path(path: OsString) -> Result<CString, &'static str> {
    let path = CString::new(path.into_vec()).map_err(|_| "ephemeris path holds a NUL byte")?;
    match path.as_bytes().len() {
        0 => Err("ephemeris path is empty"),
        1..=242 => Ok(path),
        _ => Err("ephemeris path is longer than 242 bytes"),
    }
}

static CONFIG: OnceLock<(CString, usize, usize)> = OnceLock::new();
static POOL: OnceLock<Pool> = OnceLock::new();

fn pool() -> &'static Pool {
    POOL.get_or_init(|| {
        let (path, workers, queue) = CONFIG.get().cloned().unwrap_or_else(|| {
            // Without `configure`, as in this crate's tests: N = available_parallelism and queue 4 × N.
            let path = env::var_os("HORA_EPHE_PATH").expect("HORA_EPHE_PATH is set");
            let n = thread::available_parallelism().map_or(1, |n| n.get());
            (ephe_path(path).expect("HORA_EPHE_PATH is valid"), n, 4 * n)
        });
        spawn(path, workers, queue)
    })
}

#[derive(Default)]
struct Pool {
    /// `None` once shut down.
    queue: RwLock<Option<mpsc::SyncSender<Job>>>,
    workers: Mutex<Vec<thread::JoinHandle<()>>>,
    closing: Arc<AtomicBool>,
}

impl Pool {
    fn shutdown(&self) {
        self.closing.store(true, Ordering::SeqCst);
        // After the sender drops, workers still receive the queued jobs; `closing` makes them skip those.
        self.queue
            .write()
            .expect("no panics holding the lock")
            .take();
        let workers =
            std::mem::take(&mut *self.workers.lock().expect("no panics holding the lock"));
        for worker in workers {
            if worker.join().is_err() {
                tracing::error!("Swiss Ephemeris worker panicked");
            }
        }
    }
}

/// ADR 0001: the longest a job may wait in the queue; calculation time does not count.
const QUEUE_WAIT: Duration = Duration::from_secs(2);

struct Job {
    request: Request,
    /// The UT Julian day from `validate`, so the moment checked is the moment calculated (to within the equation of time, for local apparent time).
    jd: f64,
    deadline: Instant,
    /// Whoever sets it first owns the job: a worker calculates it, the caller at its deadline gives up on it.
    taken: Arc<AtomicBool>,
    reply: oneshot::Sender<Result<Sky, Error>>,
}

impl Job {
    /// A job the worker answers `Overloaded` without calling C; to a gone caller that answer goes nowhere.
    /// The swap comes last, so a skipped job is never claimed.
    fn skip(&self, closing: bool) -> bool {
        closing
            || Instant::now() > self.deadline
            || self.reply.is_closed()
            || self.taken.swap(true, Ordering::SeqCst)
    }
}

/// Starts the pool only for a valid request, so a bad one never reads the environment or spawns workers.
async fn calculate_on<'a>(pool: impl FnOnce() -> &'a Pool, request: Request) -> Result<Sky, Error> {
    let jd = validate(&request)?;
    let (reply, mut answer) = oneshot::channel();
    let deadline = Instant::now() + QUEUE_WAIT;
    let taken = Arc::new(AtomicBool::new(false));
    let job = Job {
        request,
        jd,
        deadline,
        taken: Arc::clone(&taken),
        reply,
    };
    let sent = match &*pool().queue.read().expect("no panics holding the lock") {
        Some(queue) => queue.try_send(job),
        None => return Err(Error::Overloaded),
    };
    match sent {
        Ok(()) => {}
        Err(mpsc::TrySendError::Full(_)) => return Err(Error::Overloaded),
        Err(mpsc::TrySendError::Disconnected(_)) => {
            tracing::error!("Swiss Ephemeris pool has no workers left");
            return Err(Error::Ephemeris);
        }
    }
    let sky = match tokio::time::timeout_at(deadline.into(), &mut answer).await {
        Ok(sky) => sky,
        // No worker took it within QUEUE_WAIT; one that did keeps calculating, per ADR 0001.
        Err(_) if !taken.swap(true, Ordering::SeqCst) => return Err(Error::Overloaded),
        Err(_) => answer.await,
    };
    sky.unwrap_or_else(|_| {
        tracing::error!("Swiss Ephemeris worker dropped a job");
        Err(Error::Ephemeris)
    })
}

/// The request rules of issue #2; every request passes here before it is queued.
/// Returns the moment as a UT Julian day; for local apparent time, the worker's `ut` still corrects it.
/// Names every broken rule; the window is judged only on a valid moment, so it never stands in for another rule.
fn validate(r: &Request) -> Result<f64, Error> {
    let mut broken = Vec::new();
    // Range checks fail on NaN, so these also reject NaN and the infinities.
    let latitude = r.latitude.abs() < MAX_LATITUDE;
    if !latitude {
        broken.push(Rule::Latitude);
    }
    let longitude = LONGITUDE.contains(&r.longitude);
    if !longitude {
        broken.push(Rule::Longitude);
    }
    if !ALTITUDE.contains(&r.altitude) {
        broken.push(Rule::Altitude);
    }
    let time =
        (0..24).contains(&r.hour) && (0..60).contains(&r.minute) && (0..60).contains(&r.second);
    if !time {
        broken.push(Rule::BirthTime);
    }
    // In days; `None` when broken, already named.
    let offset = match r.utc_offset {
        UtcOffset::Stated(seconds) if UTC_OFFSET.contains(&seconds) => {
            Some(f64::from(seconds) / 86400.0)
        }
        UtcOffset::Stated(_) => {
            broken.push(Rule::UtcOffset);
            None
        }
        // Local apparent time is within about 16 min of this; the worker corrects it, inside the window's margin.
        UtcOffset::LocalMean | UtcOffset::LocalApparent => longitude.then(|| r.longitude / 360.0),
    };
    // A broken time still leaves the date to judge, at 0h.
    let hour = if time {
        f64::from(r.hour) + f64::from(r.minute) / 60.0 + f64::from(r.second) / 3600.0
    } else {
        0.0
    };
    let local = local_day(r.year, r.month, r.day, hour, r.calendar).and_then(|local| {
        // Both calendars are proleptic in the library, so only this rejects Gregorian dates before 1582-10-15.
        if r.calendar == Calendar::Gregorian && local < FIRST_GREGORIAN_DAY {
            Err(Rule::BeforeGregorian)
        } else {
            Ok(local)
        }
    });
    match (local, offset) {
        (Err(rule), _) => broken.push(rule),
        (Ok(local), Some(offset)) if time => {
            let jd = local - offset;
            // Outside the files the library would fall back to Moshier; Julian 2399-12-13 23h on is past the end.
            // shortcut: the end is checked in UT with a fixed margin for ΔT; check TT on the worker if the margin ever needs to shrink.
            if !(FILES_START + SUN_SEARCH..FILES_END - SUN_SEARCH).contains(&jd) {
                broken.push(Rule::OutsideFiles);
            } else if broken.is_empty() {
                return Ok(jd);
            }
        }
        _ => {}
    }
    Err(Error::InvalidInput(broken))
}

/// The UT Julian day of a date and fractional hour; `InvalidInput` naming the broken rule for an impossible date such as 30 February, an hour outside 0..24 or a year outside 1200–2399.
pub fn julian_day(
    year: i32,
    month: i32,
    day: i32,
    hour: f64,
    calendar: Calendar,
) -> Result<f64, Error> {
    local_day(year, month, day, hour, calendar).map_err(|rule| Error::InvalidInput(vec![rule]))
}

/// The [year, month, day] of a Julian day in `calendar`.
pub fn civil_date(jd: f64, calendar: Calendar) -> [i32; 3] {
    let (mut year, mut month, mut day, mut hour) = (0, 0, 0, 0.0);
    // SAFETY: valid out-pointers; pure date arithmetic, which ADR 0001 allows off the workers.
    unsafe {
        ffi::swe_revjul(
            jd,
            i32::from(calendar == Calendar::Gregorian),
            &mut year,
            &mut month,
            &mut day,
            &mut hour,
        );
    }
    [year, month, day]
}

fn local_day(year: i32, month: i32, day: i32, hour: f64, calendar: Calendar) -> Result<f64, Rule> {
    // NaN fails this too, guarding swe_revjul's (int) casts.
    if !(0.0..24.0).contains(&hour) {
        return Err(Rule::BirthTime);
    }
    // Only guards swe_revjul's (int) casts; the files' window in `validate` decides the range.
    if !(1200..=2399).contains(&year) {
        return Err(Rule::OutsideFiles);
    }
    let calendar = match calendar {
        Calendar::Gregorian => b'g',
        Calendar::Julian => b'j',
    };
    let mut jd = 0.0;
    // SAFETY: valid out-pointer; pure date arithmetic, which ADR 0001 allows off the workers.
    let ret =
        unsafe { ffi::swe_date_conversion(year, month, day, hour, calendar as c_char, &mut jd) };
    if ret != ffi::OK {
        return Err(Rule::ImpossibleDate);
    }
    Ok(jd)
}

/// Latitudes hora accepts lie strictly inside ±this, in degrees.
const MAX_LATITUDE: f64 = 90.0;
/// Longitudes hora accepts, in degrees.
const LONGITUDE: RangeInclusive<f64> = -180.0..=180.0;
/// Altitudes hora accepts, in metres.
const ALTITUDE: RangeInclusive<f64> = -500.0..=9000.0;
/// Stated UTC offsets hora accepts, in seconds.
const UTC_OFFSET: RangeInclusive<i32> = -12 * 3600..=14 * 3600;
/// 1582-10-15 0h UT, the first day of the Gregorian calendar.
const FIRST_GREGORIAN_DAY: f64 = 2_299_160.5;
/// Julian 1200-01-01 0h UT, the start of the files' 1200–2399 range.
const FILES_START: f64 = 2_159_357.5;
/// 2399-12-31 23h UT (Gregorian): the files' 1200–2399 ends at 2400-01-01 in TT, which picks the file.
/// Past it the library seeks a 2400 file and falls back to Moshier unless `sepl_18` is already open
/// (`sweph.c:2182`); ΔT here is about 520 s, so the hour keeps TT inside.
const FILES_END: f64 = 2_597_641.5 - HOUR;
/// Days kept clear at each end of the files: `swe_rise_trans` reports no flags, so a Moshier fallback
/// would go unseen, and the Sun searches reach from 25 h before the birth to about 29 h after (`swecl.c` `jmax`).
/// It also covers the worker's local apparent time correction, at most about 16 min.
const SUN_SEARCH: f64 = 2.0;

fn spawn(path: CString, workers: usize, queue: usize) -> Pool {
    // SE_EPHE_PATH would override this path (sweph.c:1327); hora-web refuses to start with it set.
    let (tx, rx) = mpsc::sync_channel::<Job>(queue);
    let rx = Arc::new(Mutex::new(rx));
    let closing = Arc::new(AtomicBool::new(false));
    let workers = (0..workers)
        .map(|i| {
            let (path, rx, closing) = (path.clone(), Arc::clone(&rx), Arc::clone(&closing));
            thread::Builder::new()
                .name(format!("swe-{i}"))
                .spawn(move || {
                    let swe = Swe::new(&path);
                    loop {
                        let job = rx.lock().expect("no worker panics holding the lock").recv();
                        // shutdown() dropped the sender and the queue is empty; dropping swe calls swe_close.
                        let Ok(job) = job else { return };
                        let sky = if job.skip(closing.load(Ordering::SeqCst)) {
                            Err(Error::Overloaded)
                        } else {
                            swe.sky(&job.request, job.jd)
                        };
                        let _ = job.reply.send(sky);
                    }
                })
                .expect("spawn Swiss Ephemeris worker")
        })
        .collect();
    Pool {
        queue: RwLock::new(Some(tx)),
        workers: Mutex::new(workers),
        closing,
    }
}

/// The library's state on this thread (it is thread-local), so this handle never leaves it.
struct Swe(PhantomData<*const ()>);

// The workers' swe_close at exit is this Drop; removing it must not compile.
const _: () = assert!(std::mem::needs_drop::<Swe>());

impl Drop for Swe {
    fn drop(&mut self) {
        // SAFETY: no arguments; closes this thread's files and frees its buffers.
        unsafe { ffi::swe_close() };
    }
}

impl Swe {
    fn new(path: &CStr) -> Self {
        // SAFETY: path is a NUL-terminated string the library only reads.
        unsafe { ffi::swe_set_ephe_path(path.as_ptr()) };
        Swe(PhantomData)
    }

    fn sky(&self, r: &Request, jd: f64) -> Result<Sky, Error> {
        let jd = self.ut(r, jd)?;
        let mut serr = [0 as c_char; 256];
        // SAFETY: plain values in, no pointers.
        unsafe {
            ffi::swe_set_sid_mode(r.ayanamsa as i32, 0.0, 0.0);
            // ADR 0001: set per chart so no chart inherits the previous one's topo state; positions stay geocentric.
            ffi::swe_set_topo(r.longitude, r.latitude, r.altitude);
        }

        let mut positions = [Position::default(); 8];
        for (pos, ipl) in positions
            .iter_mut()
            .zip(ffi::GRAHAS.into_iter().chain([r.node as i32]))
        {
            *pos = self.position(jd, ipl)?;
        }
        let [sun, moon, mars, mercury, jupiter, venus, saturn, rahu] = positions;

        // The value swe_calc_ut subtracts under SEFLG_SIDEREAL (sweph.c:6095); swe_get_ayanamsa_ut can silently give 0.
        let mut ayanamsa = 0.0;
        let aya_flags = ffi::SEFLG_SWIEPH | ffi::SEFLG_NONUT | ffi::SEFLG_TRUEPOS;
        // SAFETY: valid out-pointers; serr holds 256 bytes.
        let ret =
            unsafe { ffi::swe_get_ayanamsa_ex_ut(jd, aya_flags, &mut ayanamsa, serr.as_mut_ptr()) };
        check(ret, &mut serr)?;

        let lagna = self.lagna(r, jd)?;

        // A NaN would show as a confident Aries 0°; Ephemeris sends the user to the failure page instead.
        let finite = positions
            .iter()
            .all(|p| p.longitude.is_finite() && p.speed.is_finite());
        if !(finite && ayanamsa.is_finite()) {
            tracing::error!("non-finite result at jd {jd}");
            return Err(Error::Ephemeris);
        }

        let vedic_day = self.vedic_day(r, jd)?;
        let part_lagnas = vedic_day
            .map(|day| self.part_lagnas(r, half(&day, jd)))
            .transpose()?;
        let new_moons = self.new_moons(jd)?;
        let limbs = self.limbs(r, jd)?;

        Ok(Sky {
            jd,
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
            ayanamsa,
            vedic_day,
            new_moons,
            limbs,
            part_lagnas,
        })
    }

    /// The sidereal lagna at `jd` (UT) for the request's place; the sidereal mode must already be set.
    fn lagna(&self, r: &Request, jd: f64) -> Result<f64, Error> {
        // Equal houses never fail (Placidus and Koch do inside the polar circle); only ascmc[0] is used.
        let (mut cusps, mut ascmc) = ([0.0; 13], [0.0; 10]);
        // SAFETY: cusps holds 13 doubles and ascmc 10, as the library requires for this house system.
        let ret = unsafe {
            ffi::swe_houses_ex(
                jd,
                ffi::SEFLG_SIDEREAL,
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
        let lagna = ascmc[0];
        if !lagna.is_finite() {
            tracing::error!("non-finite lagna at jd {jd}");
            return Err(Error::Ephemeris);
        }
        Ok(lagna)
    }

    /// `Sky::part_lagnas` for the half of the Vedic day from `start` to `end` (UT).
    fn part_lagnas(&self, r: &Request, [start, end]: [f64; 2]) -> Result<[[f64; 2]; 8], Error> {
        let mut lagnas = [[0.0; 2]; 8];
        for (k, lagna) in lagnas.as_flattened_mut().iter_mut().enumerate() {
            *lagna = self.lagna(r, start + (end - start) * k as f64 / 16.0)?;
        }
        Ok(lagnas)
    }

    /// A body's sidereal position at `jd` (UT), in the requested ayanamsa.
    fn position(&self, jd: f64, ipl: i32) -> Result<Position, Error> {
        // TRUEPOS (no light-time, no aberration) is what Jagannatha Hora uses; without it every graha is 20–60″ off its values.
        let flags = ffi::SEFLG_SWIEPH | ffi::SEFLG_SIDEREAL | ffi::SEFLG_TRUEPOS | ffi::SEFLG_SPEED;
        let (mut xx, mut serr) = ([0.0; 6], [0 as c_char; 256]);
        // SAFETY: xx holds the 6 doubles and serr the 256 bytes the library writes.
        let ret = unsafe { ffi::swe_calc_ut(jd, ipl, flags, xx.as_mut_ptr(), serr.as_mut_ptr()) };
        check(ret, &mut serr)?;
        Ok(Position {
            longitude: xx[0],
            speed: xx[3],
        })
    }

    /// The new Moons of `Sky::new_moons` for a birth at `jd` (UT).
    fn new_moons(&self, jd: f64) -> Result<Option<[NewMoon; 3]>, Error> {
        let Some(lead) = self.angle(jd, Measure::MoonAhead)? else {
            return Ok(None);
        };
        let Some(start) = self.new_moon(jd - lead.degrees / 360.0 * SYNODIC_MONTH)? else {
            return Ok(None);
        };
        let previous = self.new_moon(start.jd - SYNODIC_MONTH)?;
        let next = self.new_moon(start.jd + SYNODIC_MONTH)?;
        Ok(previous
            .zip(next)
            .map(|(previous, next)| [previous, start, next]))
    }

    /// The new Moon nearest `jd` (UT); `None` if the search leaves the files.
    fn new_moon(&self, jd: f64) -> Result<Option<NewMoon>, Error> {
        let Some(jd) = self.crossing(jd, Measure::MoonAhead, 0.0)? else {
            return Ok(None);
        };
        let longitude = self.position(jd, ffi::SE_SUN)?.longitude;
        Ok(Some(NewMoon { jd, longitude }))
    }

    /// The `Sky::limbs` of a birth at `jd` (UT).
    fn limbs(&self, r: &Request, jd: f64) -> Result<Limbs, Error> {
        let [tithi, nakshatra, yoga, karana] = LIMBS.map(|(measure, size)| {
            // The validated window keeps a limb, at most about 1.2 days, inside the files.
            let outside = || {
                tracing::error!(jd, "limb search left the files");
                Error::Ephemeris
            };
            let Angle { degrees, rate } = self.angle(jd, measure)?.ok_or_else(outside)?;
            let part = part(degrees, size);
            let [from, to] = [part, part + 1].map(|p| f64::from(p * size) / 3600.0);
            let start = self
                .crossing(jd - toward(degrees - from) / rate, measure, from)?
                .ok_or_else(outside)?;
            let end = self
                .crossing(jd + toward(to - degrees) / rate, measure, to)?
                .ok_or_else(outside)?;
            // A birth under 1µ″ short of a boundary is past it (`part`), and Newton stops within a millisecond: the birth stays in its limb.
            let (start, end) = (start.min(jd), end.max(jd.next_up()));
            Ok(Span {
                start,
                end,
                clock_start: self.clock(r, start)?,
                clock_end: self.clock(r, end)?,
            })
        });
        Ok(Limbs {
            tithi: tithi?,
            nakshatra: nakshatra?,
            yoga: yoga?,
            karana: karana?,
        })
    }

    /// The moment (UT) nearest `jd` when `measure` is `target` degrees, by Newton's method; `None` if the search leaves the files.
    fn crossing(&self, mut jd: f64, measure: Measure, target: f64) -> Result<Option<f64>, Error> {
        // From a mean-rate estimate it takes about four steps.
        for _ in 0..20 {
            let Some(Angle { degrees, rate }) = self.angle(jd, measure)? else {
                return Ok(None);
            };
            // Toward the nearer crossing, behind or ahead.
            let step = toward(degrees - target) / rate;
            jd -= step;
            // About a millisecond.
            if step.abs() < 1e-8 {
                return Ok(Some(jd));
            }
        }
        tracing::error!(jd, target, "crossing search did not converge");
        Err(Error::Ephemeris)
    }

    /// `measure` at `jd` (UT); `None` outside the files, where the library would fall back to Moshier.
    fn angle(&self, jd: f64, measure: Measure) -> Result<Option<Angle>, Error> {
        if !(FILES_START..FILES_END).contains(&jd) {
            return Ok(None);
        }
        let sun = self.position(jd, ffi::SE_SUN)?;
        let moon = self.position(jd, ffi::SE_MOON)?;
        let (degrees, rate) = match measure {
            Measure::MoonAhead => (moon.longitude - sun.longitude, moon.speed - sun.speed),
            Measure::Moon => (moon.longitude, moon.speed),
            Measure::SunPlusMoon => (sun.longitude + moon.longitude, sun.speed + moon.speed),
        };
        let degrees = degrees.rem_euclid(360.0);
        if !(degrees.is_finite() && rate > 0.0) {
            tracing::error!(jd, rate, "bad Sun-Moon angle");
            return Err(Error::Ephemeris);
        }
        Ok(Some(Angle { degrees, rate }))
    }

    /// The birth's UT Julian day from `validate`'s, which treats local apparent time as local mean time.
    fn ut(&self, r: &Request, jd: f64) -> Result<f64, Error> {
        if r.utc_offset != UtcOffset::LocalApparent {
            return Ok(jd);
        }
        let mean_offset = r.longitude / 360.0;
        let (mut mean, mut serr) = (0.0, [0 as c_char; 256]);
        // SAFETY: valid out-pointer; serr holds the 256 bytes the library writes.
        let ret = unsafe {
            ffi::swe_lat_to_lmt(jd + mean_offset, r.longitude, &mut mean, serr.as_mut_ptr())
        };
        // swe_time_equ drops swe_calc_ut's flags, so a Moshier fallback goes unseen; the validated window rules it out.
        if ret != ffi::OK {
            // ERR: check logs serr.
            return check(ret, &mut serr).and(Err(Error::Ephemeris));
        }
        Ok(mean - mean_offset)
    }

    /// A day exists only with both its sunrises, so the answer never depends on how far back the search looks.
    fn vedic_day(&self, r: &Request, jd: f64) -> Result<Option<VedicDay>, Error> {
        // Each search returns the first event after its start; the hour keeps a chained search
        // from finding the event it started on, and no two sunrises are an hour apart.
        let rise_after = |from| self.sun_event(r, from, ffi::SE_CALC_RISE);
        let Some(mut sunrise) = rise_after(jd - 1.0 - HOUR)?.filter(|&rise| rise <= jd) else {
            return Ok(None);
        };
        let Some(mut next_sunrise) = rise_after(sunrise + HOUR)? else {
            return Ok(None);
        };
        // Sunrises are over 12 h apart, so the 25 h look-back holds at most two: the third is past the birth.
        if next_sunrise <= jd {
            sunrise = next_sunrise;
            match rise_after(sunrise + HOUR)? {
                Some(rise) if rise > jd => next_sunrise = rise,
                Some(_) => {
                    tracing::error!(jd, "three sunrises within 25 h");
                    return Err(Error::Ephemeris);
                }
                None => return Ok(None),
            }
        }
        let Some(sunset) = self.sun_event(r, sunrise, ffi::SE_CALC_SET)? else {
            return Ok(None);
        };
        Ok(Some(VedicDay {
            sunrise,
            sunset,
            next_sunrise,
            clock_sunrise: self.clock(r, sunrise)?,
            clock_sunset: self.clock(r, sunset)?,
            clock_next_sunrise: self.clock(r, next_sunrise)?,
        }))
    }

    /// A UT Julian day in the birth time's clock; local apparent time takes the equation of time at that moment.
    fn clock(&self, r: &Request, ut: f64) -> Result<f64, Error> {
        let mean = ut + r.longitude / 360.0;
        match r.utc_offset {
            UtcOffset::Stated(seconds) => Ok(ut + f64::from(seconds) / 86400.0),
            UtcOffset::LocalMean => Ok(mean),
            UtcOffset::LocalApparent => {
                let (mut apparent, mut serr) = (0.0, [0 as c_char; 256]);
                // SAFETY: valid out-pointer; serr holds the 256 bytes the library writes.
                let ret = unsafe {
                    ffi::swe_lmt_to_lat(mean, r.longitude, &mut apparent, serr.as_mut_ptr())
                };
                if ret != ffi::OK {
                    // ERR: check logs serr.
                    return check(ret, &mut serr).and(Err(Error::Ephemeris));
                }
                Ok(apparent)
            }
        }
    }

    /// The first sunrise or sunset after `from` (UT Julian day); `None` if there is none within about a day.
    fn sun_event(&self, r: &Request, from: f64, event: i32) -> Result<Option<f64>, Error> {
        let geopos = [r.longitude, r.latitude, r.altitude];
        let (mut t, mut serr) = (0.0, [0 as c_char; 256]);
        // SAFETY: geopos holds the 3 doubles read, t the 1 written, serr the 256 bytes; no star name.
        let ret = unsafe {
            ffi::swe_rise_trans(
                from,
                ffi::SE_SUN,
                std::ptr::null_mut(),
                ffi::SEFLG_SWIEPH,
                event | r.sunrise_definition as i32,
                geopos.as_ptr(),
                0.0,
                // With 0 (the library's sea-level pressure reduced for altitude), 20 °C reproduces Jagannatha Hora.
                20.0,
                &mut t,
                serr.as_mut_ptr(),
            )
        };
        match ret {
            ffi::NO_RISE_OR_SET => Ok(None),
            ffi::OK => Ok(Some(t)),
            // ERR: check logs serr.
            _ => check(ret, &mut serr).and(Err(Error::Ephemeris)),
        }
    }
}

/// The half of `day` a birth at `jd` (UT) is in, as [start, end]: sunrise to sunset, or from sunset (itself included) to the next sunrise.
fn half(day: &VedicDay, jd: f64) -> [f64; 2] {
    if jd < day.sunset {
        [day.sunrise, day.sunset]
    } else {
        [day.sunset, day.next_sunrise]
    }
}

/// What a panchaanga limb counts in equal parts of the zodiac.
#[derive(Debug, Clone, Copy)]
enum Measure {
    /// How far the Moon is ahead of the Sun; 0° is a conjunction.
    MoonAhead,
    /// The Moon's longitude.
    Moon,
    SunPlusMoon,
}

/// The tithi's, nakshatra's, Sun-Moon yoga's and karana's measures and parts in arcseconds (ch. 1 §1.3.8.1, §1.3.6, §1.3.9, §1.3.10).
const LIMBS: [(Measure, u32); 4] = [
    (Measure::MoonAhead, 12 * 3600),
    (Measure::Moon, 48000),
    (Measure::SunPlusMoon, 48000),
    (Measure::MoonAhead, 6 * 3600),
];

/// An angle's difference in degrees as the shorter way round, in [−180°, 180°): 360° is 0°, and a hair below 0° stays negative.
fn toward(degrees: f64) -> f64 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

/// Which part of `size` arcseconds `degrees`, in [0°, 360°], is in, split as `hora_chart`'s `rasi` and `arcseconds` split it,
/// so the limb searched is the one the chart names.
fn part(degrees: f64, size: u32) -> u32 {
    let within = degrees % 30.0;
    let rasi = ((degrees - within) / 30.0).round() as u32 % 12;
    (rasi * 30 * 3600 + ((within * 3600.0 + 1e-6) as u32).min(30 * 3600 - 1)) / size
}

/// A measure at one moment.
struct Angle {
    /// In [0°, 360°]: rem_euclid can round up to 360°.
    degrees: f64,
    /// How fast `degrees` grows, in degrees per day.
    rate: f64,
}

const HOUR: f64 = 1.0 / 24.0;
/// The mean time from one new Moon to the next, in days; a single month is up to about 7 h off it.
const SYNODIC_MONTH: f64 = 29.530_589;

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

    // The reference tests pass it literally, so a wrong b'g'/b'j' mapping in `validate` fails there.
    #[cfg(test)]
    pub const SE_GREG_CAL: c_int = 1;
    pub const OK: c_int = 0;
    pub const NO_RISE_OR_SET: c_int = -2;
    pub const SE_SUN: i32 = 0;
    pub const SE_MOON: i32 = 1;
    pub const SE_CALC_RISE: i32 = 1;
    pub const SE_CALC_SET: i32 = 2;
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
        pub fn swe_close();
        pub fn swe_set_sid_mode(sid_mode: i32, t0: c_double, ayan_t0: c_double);
        pub fn swe_set_topo(geolon: c_double, geolat: c_double, geoalt: c_double);
        #[cfg(test)]
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
        pub fn swe_revjul(
            tjd: c_double,
            gregflag: c_int,
            jyear: *mut c_int,
            jmon: *mut c_int,
            jday: *mut c_int,
            jut: *mut c_double,
        );
        pub fn swe_lat_to_lmt(
            tjd_lat: c_double,
            geolon: c_double,
            tjd_lmt: *mut c_double,
            serr: *mut c_char,
        ) -> i32;
        pub fn swe_lmt_to_lat(
            tjd_lmt: c_double,
            geolon: c_double,
            tjd_lat: *mut c_double,
            serr: *mut c_char,
        ) -> i32;
        pub fn swe_rise_trans(
            tjd_ut: c_double,
            ipl: i32,
            starname: *mut c_char,
            epheflag: i32,
            rsmi: i32,
            geopos: *const c_double,
            atpress: c_double,
            attemp: c_double,
            tret: *mut c_double,
            serr: *mut c_char,
        ) -> i32;
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

    #[test]
    fn configure_refuses_a_path_the_library_would_silently_replace() {
        let one = NonZeroUsize::MIN;
        for path in ["", "a\0b", &"a".repeat(243)] {
            assert!(configure(path.into(), one, one).is_err(), "{path:?}");
        }
    }

    #[tokio::test]
    async fn missing_ephemeris_files_fail_instead_of_falling_back_to_moshier() {
        // Its own pool: the shared one points at the real files.
        let pool = spawn(c"/nonexistent/hora-ephe".to_owned(), 1, 1);
        let request = Request {
            year: 2000,
            month: 4,
            day: 9,
            hour: 21,
            minute: 55,
            second: 0,
            calendar: Calendar::Gregorian,
            utc_offset: UtcOffset::Stated(0),
            latitude: 42.5,
            longitude: -71.2,
            altitude: 0.0,
            ayanamsa: Ayanamsa::TrueChitra,
            node: NodeType::Mean,
            sunrise_definition: SunriseDefinition::UpperLimbApparent,
        };
        assert_eq!(
            calculate_on(|| &pool, request).await.unwrap_err(),
            Error::Ephemeris
        );
    }
    /// The book example charts' moments (UT) and places, with the given choices.
    fn book_births(ayanamsa: Ayanamsa, node: NodeType) -> [Request; 3] {
        let at = |year, month, day, hour, minute, latitude, longitude| Request {
            year,
            month,
            day,
            hour,
            minute,
            second: 0,
            calendar: Calendar::Gregorian,
            utc_offset: UtcOffset::Stated(0),
            latitude,
            longitude,
            altitude: 0.0,
            ayanamsa,
            node,
            sunrise_definition: SunriseDefinition::UpperLimbApparent,
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
        let [request, ..] = book_births(Ayanamsa::TrueChitra, NodeType::True);
        let mean = calculate(Request {
            node: NodeType::Mean,
            ..request.clone()
        })
        .await
        .unwrap();
        let sky = calculate(request).await.unwrap();

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

        assert_eq!(sky.grahas.rahu.longitude, reference);
        // The true node oscillates up to ~1.7° around the mean one; guards against the choice being ignored.
        assert!((sky.grahas.rahu.longitude - mean.grahas.rahu.longitude).abs() > 0.01);
    }

    #[tokio::test]
    async fn each_ayanamsa_matches_swe_get_ayanamsa_ex_ut() {
        // Literal SE_SIDM_* values, not the enums, so a wrong discriminant fails here.
        let choices = [
            (Ayanamsa::Lahiri, 1),
            (Ayanamsa::Raman, 3),
            (Ayanamsa::TrueChitra, 27),
            (Ayanamsa::TruePushya, 29),
            (Ayanamsa::TrueMula, 35),
        ];
        for (ayanamsa, sid_mode) in choices {
            let [request, ..] = book_births(ayanamsa, NodeType::Mean);
            let sky = calculate(request).await.unwrap();

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

            assert_eq!(sky.ayanamsa, reference, "{ayanamsa:?}");
        }
    }

    /// ADR 0001's verification: no chart inherits another chart's ayanamsa, node or place from a worker.
    #[tokio::test]
    async fn concurrent_charts_match_their_single_threaded_reference() {
        let requests: Vec<Request> = [
            Ayanamsa::Lahiri,
            Ayanamsa::Raman,
            Ayanamsa::TrueChitra,
            Ayanamsa::TrueMula,
            Ayanamsa::TruePushya,
        ]
        .into_iter()
        .flat_map(|a| [NodeType::Mean, NodeType::True].map(|n| book_births(a, n)))
        .flatten()
        .collect();
        let references = {
            let requests = requests.clone();
            single_threaded(move |swe| {
                requests
                    .iter()
                    .map(|r| swe.sky(r, validate(r).unwrap()).unwrap())
                    .collect::<Vec<_>>()
            })
        };

        let jobs: Vec<_> = (0..8)
            .flat_map(|round| (0..requests.len()).map(move |i| (round, i)))
            .collect();
        // Capacity is 5 × N: this test holds ≤ 2 × N, every other test ≤ 1, under cargo's default N test threads.
        // 2 × N already keeps all N workers busy, so more in flight would not add concurrency in C.
        let n = thread::available_parallelism().unwrap().get();
        for chunk in jobs.chunks(2 * n) {
            let mut skies = tokio::task::JoinSet::new();
            for &(round, i) in chunk {
                let request = requests[i].clone();
                skies.spawn(async move { (round, i, calculate(request).await.unwrap()) });
            }
            while let Some(joined) = skies.join_next().await {
                let (round, i, sky) = joined.unwrap();
                assert_eq!(sky, references[i], "round {round}: {:?}", requests[i]);
            }
        }
    }

    // Internal on purpose: no chart waits 2 s and a gone caller sees nothing, so calculate() can't show these drops.
    #[test]
    fn a_worker_skips_stale_abandoned_and_closing_jobs() {
        let job = |deadline| {
            let (reply, answer) = oneshot::channel();
            let [request, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
            let jd = validate(&request).unwrap();
            let job = Job {
                request,
                jd,
                deadline,
                taken: Arc::default(),
                reply,
            };
            (job, answer)
        };
        let later = Instant::now() + QUEUE_WAIT;
        let (fresh, _answer) = job(later);
        assert!(!fresh.skip(false));
        // The first skip(false) claimed it, as a caller giving up at its deadline would have.
        assert!(fresh.skip(false));
        let (closing, _answer) = job(later);
        assert!(closing.skip(true));
        let (abandoned, answer) = job(later);
        drop(answer);
        assert!(abandoned.skip(false));
        let (stale, _answer) = job(Instant::now().checked_sub(Duration::from_secs(1)).unwrap());
        assert!(stale.skip(false));
    }

    #[tokio::test(start_paused = true)]
    async fn a_job_no_worker_takes_is_overloaded_at_its_deadline() {
        // No worker reads this queue; the paused clock jumps to the deadline once the caller waits.
        let (tx, rx) = mpsc::sync_channel(1);
        let pool = Pool {
            queue: RwLock::new(Some(tx)),
            ..Pool::default()
        };
        let [request, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        assert_eq!(
            calculate_on(|| &pool, request).await.unwrap_err(),
            Error::Overloaded
        );
        assert!(rx.try_recv().unwrap().skip(false));
    }

    #[tokio::test]
    async fn shutdown_answers_queued_jobs_overloaded_and_waits_for_every_worker() {
        // Its own pool: shutting the shared one would fail the other tests.
        let path = env::var_os("HORA_EPHE_PATH").expect("HORA_EPHE_PATH is set");
        let pool = spawn(CString::new(path.into_vec()).unwrap(), 1, 64);
        let [request, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let mut skies: Vec<_> = (0..64)
            .map(|_| Box::pin(calculate_on(|| &pool, request.clone())))
            .collect();
        // One poll each enqueues the job; 64 sends take µs, while the one cold worker takes ms per chart.
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        for sky in &mut skies {
            assert!(sky.as_mut().poll(&mut cx).is_pending());
        }
        pool.shutdown();
        assert_eq!(
            Arc::strong_count(&pool.closing),
            1,
            "a worker is still running"
        );

        let (mut ok, mut overloaded) = (0, 0);
        for sky in skies {
            match sky.await {
                Ok(_) => ok += 1,
                Err(Error::Overloaded) => overloaded += 1,
                Err(e) => panic!("{e}"),
            }
        }
        assert!(overloaded > 0, "ok {ok}, overloaded {overloaded}");
    }

    /// Chart 1's place on the given date and time.
    fn on(calendar: Calendar, [year, month, day, hour, minute, second]: [i32; 6]) -> Request {
        let [chart_1, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
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

    async fn assert_invalid(request: Request, rules: &[Rule]) {
        let got = calculate(request.clone()).await;
        assert_eq!(
            got.unwrap_err(),
            Error::InvalidInput(rules.to_vec()),
            "{request:?}"
        );
    }

    #[tokio::test]
    async fn thirty_february_is_rejected_in_both_calendars() {
        for calendar in [Calendar::Gregorian, Calendar::Julian] {
            assert_invalid(
                on(calendar, [2000, 2, 30, 12, 0, 0]),
                &[Rule::ImpossibleDate],
            )
            .await;
            assert_invalid(
                on(calendar, [2000, 13, 1, 12, 0, 0]),
                &[Rule::ImpossibleDate],
            )
            .await;
        }
    }

    #[tokio::test]
    async fn every_broken_rule_is_named_together() {
        let request = Request {
            latitude: 95.0,
            utc_offset: UtcOffset::Stated(15 * 3600),
            ..on(Calendar::Gregorian, [2000, 2, 30, 24, 0, 0])
        };
        let rules = [
            Rule::Latitude,
            Rule::BirthTime,
            Rule::UtcOffset,
            Rule::ImpossibleDate,
        ];
        assert_invalid(request, &rules).await;
    }

    #[tokio::test]
    async fn a_date_in_the_gregorian_gap_is_only_valid_as_julian() {
        let before = [Rule::BeforeGregorian];
        assert_invalid(on(Calendar::Gregorian, [1582, 10, 10, 12, 0, 0]), &before).await;
        // The last moment before the first Gregorian day.
        assert_invalid(on(Calendar::Gregorian, [1582, 10, 14, 23, 59, 59]), &before).await;
        // Also outside the files, but the calendar is what to fix.
        assert_invalid(on(Calendar::Gregorian, [1200, 1, 1, 12, 0, 0]), &before).await;
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
    async fn moments_within_two_days_of_the_files_ends_are_rejected() {
        // Julian at the start, because a Gregorian date before 1582-10-15 is rejected.
        let outside = [Rule::OutsideFiles];
        assert_invalid(on(Calendar::Julian, [1199, 12, 31, 23, 59, 59]), &outside).await;
        assert_invalid(on(Calendar::Julian, [1200, 1, 2, 23, 59, 59]), &outside).await;
        assert_invalid(on(Calendar::Gregorian, [2399, 12, 29, 23, 0, 0]), &outside).await;
        // Julian 2399-12-13 is Gregorian 2399-12-29, so the same hour ends it: InvalidInput, not Ephemeris.
        assert_invalid(on(Calendar::Julian, [2399, 12, 13, 23, 0, 0]), &outside).await;
        assert_invalid(on(Calendar::Gregorian, [2400, 1, 1, 0, 0, 0]), &outside).await;
        for edge in [
            on(Calendar::Julian, [2399, 12, 13, 22, 59, 59]),
            on(Calendar::Julian, [1200, 1, 3, 0, 0, 0]),
            on(Calendar::Gregorian, [2399, 12, 29, 22, 59, 59]),
        ] {
            // The month's new Moons are outside the files, so the chart comes without them.
            assert_eq!(calculate(edge).await.unwrap().new_moons, None);
        }
    }

    #[test]
    fn new_moons_bracket_every_birth_a_month_apart() {
        single_threaded(|swe| {
            // 2000-01-01 0h UT, every 9 h 7 min for a year, so births fall at every lead of the Moon.
            let mut births: Vec<f64> = (0..960).map(|i| 2451544.5 + f64::from(i) * 0.38).collect();
            let [_, start, _] = swe.new_moons(births[0]).unwrap().unwrap();
            // A birth at the new Moon is in the month it starts.
            births.push(start.jd);
            for jd in births {
                let [previous, start, next] = swe.new_moons(jd).unwrap().unwrap();
                assert!(start.jd <= jd && jd < next.jd, "{jd}");
                for month in [start.jd - previous.jd, next.jd - start.jd] {
                    assert!((29.2..29.9).contains(&month), "{jd}: {month}");
                }
            }
        });
    }

    #[test]
    fn a_part_is_the_limb_hora_chart_names() {
        use hora_chart::{karana, nakshatra, sun_moon_yoga, tithi};
        // The limb named at `degrees` of the measure, with the Sun at 0°.
        let names: [fn(f64) -> u32; 4] = [
            |d| tithi(0.0, d).0.into(),
            |d| nakshatra(d).0 as u32,
            |d| sun_moon_yoga(0.0, d) as u32,
            |d| karana(0.0, d) as u32,
        ];
        for ((_, size), name) in LIMBS.into_iter().zip(names) {
            for boundary in (0..=360 * 3600 / size).map(|i| f64::from(i * size) / 3600.0) {
                // hora_chart's +1µ″ makes 1e-12° below a boundary the next part; rem_euclid can round to 360°.
                for d in
                    [-1e-5 / 3600.0, -1e-12, 0.0, 1e-12].map(|e| (boundary + e).clamp(0.0, 360.0))
                {
                    let middle = (f64::from(part(d, size)) + 0.5) * f64::from(size) / 3600.0;
                    assert_eq!(name(d), name(middle), "{d}° in parts of {size}″");
                }
            }
        }
    }

    #[test]
    fn each_limb_starts_before_the_birth_and_runs_until_it_ends() {
        single_threaded(|swe| {
            let [r, ..] = book_births(Ayanamsa::Lahiri, NodeType::Mean);
            // 2000-01-01 0h UT, every 9 h 7 min for a year, as for the new Moons.
            let mut births: Vec<f64> = (0..960).map(|i| 2451544.5 + f64::from(i) * 0.38).collect();
            // A birth at a limb's start is in that limb.
            let limbs = swe.limbs(&r, births[0]).unwrap();
            births.extend([limbs.tithi.start, limbs.karana.end]);
            let second = 1.0 / 86400.0;
            for jd in births {
                let limbs = swe.limbs(&r, jd).unwrap();
                for (span, (measure, size)) in limbs.spans().into_iter().zip(LIMBS) {
                    let part = |at| part(swe.angle(at, measure).unwrap().unwrap().degrees, size);
                    assert!(span.start <= jd && jd < span.end, "{jd}: {span:?}");
                    assert_eq!(part(span.start + second), part(jd), "{jd}: {span:?}");
                    assert_eq!(part(span.end - second), part(jd), "{jd}: {span:?}");
                    assert_ne!(part(span.end + second), part(jd), "{jd}: {span:?}");
                    // Births at UT +0.
                    assert_eq!([span.clock_start, span.clock_end], [span.start, span.end]);
                }
            }
        });
    }

    #[test]
    fn the_last_moment_calculates_whichever_file_the_worker_has_open() {
        // On the shared pool a worker that last used sepl_12 fell back to Moshier for 2399-12-31 23:59:59 (CI only).
        let r = on(Calendar::Julian, [1200, 1, 3, 0, 0, 0]);
        let skies = single_threaded(move |swe| {
            [
                validate(&r).unwrap(),
                FILES_END - SUN_SEARCH - 1.0 / 86400.0,
            ]
            .map(|jd| swe.sky(&r, jd))
        });
        for sky in skies {
            sky.unwrap();
        }
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
            let request = on(Calendar::Gregorian, [2000, 4, 9, hour, minute, second]);
            assert_invalid(request, &[Rule::BirthTime]).await;
        }
    }

    #[tokio::test]
    async fn out_of_range_places_are_rejected() {
        let [chart_1, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let bad = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
        for latitude in [90.0, -90.0].into_iter().chain(bad) {
            let request = Request {
                latitude,
                ..chart_1.clone()
            };
            assert_invalid(request, &[Rule::Latitude]).await;
        }
        for longitude in [180.001, -180.001].into_iter().chain(bad) {
            // In local time the moment depends on the longitude, so no verdict on the window either.
            for utc_offset in [UtcOffset::Stated(0), UtcOffset::LocalMean] {
                let request = Request {
                    longitude,
                    utc_offset,
                    ..chart_1.clone()
                };
                assert_invalid(request, &[Rule::Longitude]).await;
            }
        }
        for altitude in [-500.1, 9000.1].into_iter().chain(bad) {
            let request = Request {
                altitude,
                ..chart_1.clone()
            };
            assert_invalid(request, &[Rule::Altitude]).await;
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
    async fn stated_offsets_outside_minus_12_to_plus_14_hours_are_rejected() {
        let [chart_1, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let with = |seconds| Request {
            utc_offset: UtcOffset::Stated(seconds),
            ..chart_1.clone()
        };
        assert_invalid(with(-12 * 3600 - 1), &[Rule::UtcOffset]).await;
        assert_invalid(with(14 * 3600 + 1), &[Rule::UtcOffset]).await;
        calculate(with(-12 * 3600)).await.unwrap();
        calculate(with(14 * 3600)).await.unwrap();
    }

    #[test]
    fn the_budapest_local_mean_offset_is_plus_1_16_09_6() {
        let budapest = Request {
            latitude: 47.5,
            longitude: 19.04,
            utc_offset: UtcOffset::LocalMean,
            ..on(Calendar::Gregorian, [2000, 4, 9, 12, 0, 0])
        };
        let local = julian_day(2000, 4, 9, 12.0, Calendar::Gregorian).unwrap();
        let offset = (local - validate(&budapest).unwrap()) * 86400.0;
        assert!(
            (offset - (3600.0 + 16.0 * 60.0 + 9.6)).abs() < 1e-3,
            "{offset} s"
        );
    }

    #[tokio::test]
    async fn a_chart_in_local_mean_time_equals_it_with_the_stated_offset() {
        // A whole degree, so longitude / 360 and its 82 × 240 s / 86400 are the same double.
        let [.., chart_4] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let at = |utc_offset| Request {
            longitude: 82.0,
            utc_offset,
            ..chart_4.clone()
        };
        let mean = calculate(at(UtcOffset::LocalMean)).await.unwrap();
        let stated = calculate(at(UtcOffset::Stated(82 * 240))).await.unwrap();
        assert_eq!(mean, stated);
    }

    #[test]
    fn local_apparent_time_is_local_mean_time_plus_the_equation_of_time() {
        // Not a book value: J. Meeus, *Astronomical Algorithms* (2nd ed.), Example 28.a, 1992-10-13 0h TD: E = +13m42.6s.
        let greenwich = Request {
            latitude: 51.48,
            longitude: 0.0,
            utc_offset: UtcOffset::LocalApparent,
            ..on(Calendar::Gregorian, [1992, 10, 13, 0, 0, 0])
        };
        let mean = validate(&greenwich).unwrap();
        let ut = single_threaded(move |swe| swe.ut(&greenwich, mean).unwrap());
        let e = (mean - ut) * 86400.0;
        // Ours is 822.25 s: the library's model, at 0h UT minus E rather than 0h TD.
        assert!((e - (13.0 * 60.0 + 42.6)).abs() < 0.5, "E = {e} s");
    }

    #[tokio::test]
    async fn a_chart_in_local_apparent_time_is_calculated_for_its_corrected_moment() {
        let [chart_1, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let at = |utc_offset| Request {
            utc_offset,
            ..chart_1.clone()
        };
        let apparent = calculate(at(UtcOffset::LocalApparent)).await.unwrap();
        let r = at(UtcOffset::LocalApparent);
        let reference = single_threaded(move |swe| {
            let ut = swe.ut(&r, validate(&r).unwrap()).unwrap();
            // Stated(0): sky takes the corrected moment as UT, so dropping its own correction fails here.
            swe.sky(
                &Request {
                    utc_offset: UtcOffset::Stated(0),
                    ..r
                },
                ut,
            )
        });
        let mut reference = reference.unwrap();
        // The clock moments are the other test's; only the UT ones are this one's.
        let (got, want) = (
            apparent.vedic_day.unwrap(),
            reference.vedic_day.as_mut().unwrap(),
        );
        (
            want.clock_sunrise,
            want.clock_sunset,
            want.clock_next_sunrise,
        ) = (got.clock_sunrise, got.clock_sunset, got.clock_next_sunrise);
        for (got, want) in apparent
            .limbs
            .spans()
            .into_iter()
            .zip(reference.limbs.spans())
        {
            assert_eq!([got.start, got.end], [want.start, want.end]);
        }
        reference.limbs = apparent.limbs;
        assert_eq!(apparent, reference);
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
            Error::InvalidInput(vec![Rule::ImpossibleDate])
        );
    }

    #[tokio::test]
    async fn each_sunrise_definition_matches_swe_rise_trans() {
        // Literal rsmi bits, not the enum, so a wrong discriminant fails here.
        let choices = [
            (SunriseDefinition::CentreTrue, 256 | 512),
            (SunriseDefinition::UpperLimbTrue, 512),
            (SunriseDefinition::CentreApparent, 256),
            (SunriseDefinition::UpperLimbApparent, 0),
        ];
        let mut sunrises = Vec::new();
        for (sunrise_definition, rsmi) in choices {
            let [chart_1, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
            let jd = validate(&chart_1).unwrap();
            let geopos = [chart_1.longitude, chart_1.latitude, chart_1.altitude];
            let sky = calculate(Request {
                sunrise_definition,
                ..chart_1
            })
            .await
            .unwrap();

            let reference = single_threaded(move |_| {
                let (mut t, mut serr) = (0.0, [0 as c_char; 256]);
                // Chart 1 is born at 21:55 UT, so the first sunrise after a day (and the hour) earlier is that
                // morning's; the same start as the chart's search, since the result depends on it in the last bits.
                // SAFETY: geopos holds 3 doubles, t 1, serr 256 bytes; no star name.
                let ret = unsafe {
                    ffi::swe_rise_trans(
                        jd - 1.0 - HOUR,
                        0,
                        std::ptr::null_mut(),
                        ffi::SEFLG_SWIEPH,
                        1 | rsmi,
                        geopos.as_ptr(),
                        0.0,
                        20.0,
                        &mut t,
                        serr.as_mut_ptr(),
                    )
                };
                assert_eq!(ret, ffi::OK);
                t
            });

            let day = sky.vedic_day.unwrap();
            assert_eq!(day.sunrise, reference, "{sunrise_definition:?}");
            sunrises.push(reference);
        }
        // Centre against upper limb is about a minute, refraction about three.
        for (i, a) in sunrises.iter().enumerate() {
            for b in &sunrises[i + 1..] {
                assert!((a - b).abs() * 86400.0 > 20.0, "{sunrises:?}");
            }
        }
    }

    #[tokio::test]
    async fn the_altitude_moves_the_sunrise() {
        // At 3000 m the library lowers the pressure, so the refraction (sunrise) changes by about a minute.
        let [chart_1, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let high = Request {
            altitude: 3000.0,
            ..chart_1.clone()
        };
        let low = calculate(chart_1).await.unwrap().vedic_day.unwrap();
        let high = calculate(high).await.unwrap().vedic_day.unwrap();
        assert!(
            (low.sunrise - high.sunrise).abs() * 86400.0 > 10.0,
            "{low:?} {high:?}"
        );
    }

    /// Ordered, and the birth inside its day.
    fn assert_vedic_day(day: VedicDay, jd: f64) {
        assert!(day.sunrise <= jd && jd < day.next_sunrise, "{day:?} {jd}");
        assert!(
            day.sunrise < day.sunset && day.sunset < day.next_sunrise,
            "{day:?}"
        );
    }

    #[tokio::test]
    async fn the_book_births_lie_inside_their_vedic_day() {
        for request in book_births(Ayanamsa::TrueChitra, NodeType::Mean) {
            let jd = validate(&request).unwrap();
            let day = calculate(request).await.unwrap().vedic_day.unwrap();
            assert_vedic_day(day, jd);
        }
        // Chart 3 is born at 05:12 IST, before sunrise: its day ends that morning, about 2 h later.
        let [_, chart_3, _] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let jd = validate(&chart_3).unwrap();
        let day = calculate(chart_3).await.unwrap().vedic_day.unwrap();
        assert!(day.next_sunrise - jd < 2.0 * HOUR, "{day:?} {jd}");
    }

    #[tokio::test]
    async fn the_vedic_day_in_a_stated_offset_or_local_mean_time_is_ut_plus_the_offset() {
        let [chart_1, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        // 71°12′ W is 4 h 44 min 48 s behind UT.
        for (utc_offset, seconds) in [
            (UtcOffset::Stated(-4 * 3600), -4.0 * 3600.0),
            (UtcOffset::LocalMean, -(4.0 * 3600.0 + 44.0 * 60.0 + 48.0)),
        ] {
            let request = Request {
                utc_offset,
                ..chart_1.clone()
            };
            let d = calculate(request).await.unwrap().vedic_day.unwrap();
            for (clock, ut) in [
                (d.clock_sunrise, d.sunrise),
                (d.clock_sunset, d.sunset),
                (d.clock_next_sunrise, d.next_sunrise),
            ] {
                // A Julian day near 2.45e6 holds about 40 µs.
                let off = (clock - ut) * 86400.0 - seconds;
                assert!(off.abs() < 1e-3, "{utc_offset:?}: {off} s");
            }
        }
    }

    #[tokio::test]
    async fn the_vedic_day_in_local_apparent_time_is_converted_at_each_event() {
        let [chart_1, ..] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let request = Request {
            utc_offset: UtcOffset::LocalApparent,
            ..chart_1
        };
        let longitude = request.longitude;
        let d = calculate(request).await.unwrap().vedic_day.unwrap();
        let events = [d.sunrise, d.sunset, d.next_sunrise];
        let reference = single_threaded(move |_| {
            events.map(|ut| {
                let (mut lat, mut serr) = (0.0, [0 as c_char; 256]);
                // SAFETY: valid out-pointer; serr holds 256 bytes.
                let ret = unsafe {
                    ffi::swe_lmt_to_lat(
                        ut + longitude / 360.0,
                        longitude,
                        &mut lat,
                        serr.as_mut_ptr(),
                    )
                };
                assert_eq!(ret, ffi::OK);
                lat
            })
        });
        assert_eq!(
            [d.clock_sunrise, d.clock_sunset, d.clock_next_sunrise],
            reference
        );
        // In April the equation of time is about a minute, so a mean-time answer fails here.
        assert!((d.clock_sunrise - (d.sunrise + longitude / 360.0)).abs() * 86400.0 > 30.0);
    }

    #[tokio::test]
    async fn a_birth_before_sunrise_has_a_vedic_day_that_began_on_the_previous_date() {
        // Chart 3 is born at 05:12 IST on 25 Dec (UT 23:42 on the 24th), before that morning's sunrise.
        let [chart_1, chart_3, _] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        let ist = Request {
            day: 25,
            hour: 5,
            minute: 12,
            utc_offset: UtcOffset::Stated(5 * 3600 + 30 * 60),
            ..chart_3
        };
        let day = |r| async { calculate(r).await.unwrap().vedic_day.unwrap() };
        let date = |jd| civil_date(jd, Calendar::Gregorian);
        let ist = day(ist).await;
        assert_eq!(date(ist.clock_sunrise), [1926, 12, 24]);
        assert_eq!(date(ist.clock_next_sunrise), [1926, 12, 25]);
        assert_eq!(date(day(chart_1).await.clock_sunrise), [2000, 4, 9]);
    }

    #[test]
    fn a_birth_at_sunset_is_in_the_night_and_one_just_before_in_the_day() {
        let day = VedicDay {
            sunrise: 10.25,
            sunset: 10.75,
            next_sunrise: 11.25,
            clock_sunrise: 0.0,
            clock_sunset: 0.0,
            clock_next_sunrise: 0.0,
        };
        assert_eq!(half(&day, 10.75), [10.75, 11.25]);
        assert_eq!(half(&day, 10.75_f64.next_down()), [10.25, 10.75]);
        assert_eq!(half(&day, 10.25), [10.25, 10.75]);
    }

    #[tokio::test]
    async fn each_part_lagna_is_the_lagna_at_the_start_or_middle_of_its_eighth() {
        // Chart 1 is born at 17:55 EDT, before sunset; chart 3 at 05:12 IST, before sunrise, so in the previous Vedic day's night.
        let [chart_1, chart_3, _] = book_births(Ayanamsa::TrueChitra, NodeType::Mean);
        for (request, night) in [(chart_1, false), (chart_3, true)] {
            let jd = validate(&request).unwrap();
            let sky = calculate(request.clone()).await.unwrap();
            let day = sky.vedic_day.unwrap();
            let [start, end] = if night {
                [day.sunset, day.next_sunrise]
            } else {
                [day.sunrise, day.sunset]
            };
            assert!(start <= jd && jd < end, "{day:?} {jd}");
            // A separate chart at each moment: the first part starts at `start`, the eighth ends at `end`.
            let moments: [f64; 16] =
                std::array::from_fn(|k| start + (end - start) * k as f64 / 16.0);
            let reference =
                single_threaded(move |swe| moments.map(|t| swe.sky(&request, t).unwrap().lagna));
            let lagnas = sky.part_lagnas.unwrap();
            assert_eq!(lagnas.as_flattened(), reference);
        }
    }

    #[test]
    fn a_civil_date_is_in_the_chosen_calendar() {
        assert_eq!(
            civil_date(FIRST_GREGORIAN_DAY, Calendar::Gregorian),
            [1582, 10, 15]
        );
        assert_eq!(
            civil_date(FIRST_GREGORIAN_DAY, Calendar::Julian),
            [1582, 10, 5]
        );
    }

    /// Tromsø, 69°39′N 18°57′E: inside the polar circle.
    fn tromso(month: i32, day: i32, hour: i32, minute: i32) -> Request {
        Request {
            latitude: 69.65,
            longitude: 18.95,
            ..on(Calendar::Gregorian, [2000, month, day, hour, minute, 0])
        }
    }

    #[tokio::test]
    async fn a_polar_day_without_both_sunrises_has_no_vedic_day() {
        // Accepted: polar night and midnight sun have no Vedic day; the equinox shows the place has them.
        for (month, day, hour) in [(12, 21, 12), (6, 21, 12), (1, 15, 12), (7, 24, 21)] {
            let sky = calculate(tromso(month, day, hour, 0)).await.unwrap();
            assert_eq!(sky.vedic_day, None, "2000-{month}-{day} {hour}h");
            assert_eq!(sky.part_lagnas, None, "2000-{month}-{day} {hour}h");
        }
        assert!(
            calculate(tromso(3, 20, 12, 0))
                .await
                .unwrap()
                .vedic_day
                .is_some()
        );
    }

    #[tokio::test]
    async fn into_polar_night_and_out_of_midnight_sun_a_vedic_day_is_whole_or_none() {
        // Hourly across both edges, so no answer depends on how long ago the last sunrise was.
        for (month, days) in [(11, 22..29), (7, 23..31)] {
            let (mut some, mut none) = (0, 0);
            for day in days {
                for hour in 0..24 {
                    let request = tromso(month, day, hour, 0);
                    let jd = validate(&request).unwrap();
                    match calculate(request).await.unwrap().vedic_day {
                        Some(d) => {
                            assert_vedic_day(d, jd);
                            let length = (d.next_sunrise - d.sunrise) * 24.0;
                            assert!(
                                (20.0..28.0).contains(&length),
                                "{month}-{day} {hour}h: {d:?}"
                            );
                            some += 1;
                        }
                        None => none += 1,
                    }
                }
            }
            assert!(
                some > 0 && none > 0,
                "month {month}: {some} days, {none} without"
            );
        }
    }

    #[tokio::test]
    async fn inside_the_polar_circle_the_lagna_jumps_by_180_degrees() {
        // Accepted: when the ecliptic passes the horizon, the library swaps AC/DC so the lagna stays
        // on the MC's eastern side (swehouse.c:998), and the lagna jumps half the zodiac in a minute.
        let before = calculate(tromso(1, 1, 7, 57)).await.unwrap().lagna;
        let after = calculate(tromso(1, 1, 8, 0)).await.unwrap().lagna;
        let jump = (after - before).rem_euclid(360.0);
        assert!((jump - 180.0).abs() < 1.0, "{before}° to {after}°");
    }
}

//! The rasi chart drawn in the three styles of ch. 1 §1.3.4, Figure 1, as the parts of an inline SVG.

use fluent_templates::Loader;
use fluent_templates::fluent_bundle::FluentValue;
use hora_chart::{Body, Chart, Entry, RASIS, Rasi, SpecialLagna, Varga};

use super::{LOCALES, Lang, body_id, kb, rasi_id};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    South,
    North,
    East,
}

/// What `home.html` draws: the lines, then each area's labels.
#[derive(Default)]
pub struct Drawing {
    /// SVG path data in a 480-unit square.
    pub lines: &'static str,
    /// South and East Indian: one per rasi from Aries; North Indian: one per bhava from the 1st.
    pub areas: Vec<Area>,
}

pub struct Area {
    /// North Indian: where the area's rasi number (1 = Aries) goes, and the number.
    pub number: Option<(i32, i32, u8)>,
    pub labels: Vec<Label>,
    /// The bhava arudhas in this area, AL to UL (ch. 9 §9.2).
    pub arudhas: Vec<Label>,
}

/// One body's or special lagna's label, placed.
pub struct Label {
    pub x: i32,
    pub y: i32,
    /// The message id, shared with its table row.
    pub id: String,
    /// Two letters; an arudha pada's may have three (A10).
    pub short: String,
    /// `lagna`, `graha`, `special`, `arudha` or `arudha-lagna`.
    pub class: &'static str,
    pub retrograde: bool,
    /// Within the rasi, to the arcminute, truncated; empty in a varga chakra (ADR 0009) and for HL, GL and the arudha padas.
    pub degrees: String,
    /// The accessible name.
    pub name: String,
}

/// Where an area's labels centre in the 480-unit square, and how many labels go to a row.
type Place = (i32, i32, usize);

/// A 4×4 grid, its central 2×2 one empty cell; by rasi from Aries, Pisces taking the top-left corner.
const SOUTH: [Place; 12] = [
    (180, 60, 2),
    (300, 60, 2),
    (420, 60, 2),
    (420, 180, 2),
    (420, 300, 2),
    (420, 420, 2),
    (300, 420, 2),
    (180, 420, 2),
    (60, 420, 2),
    (60, 300, 2),
    (60, 180, 2),
    (60, 60, 2),
];
const SOUTH_LINES: &str = "M0 0H480V480H0ZM120 0V480M360 0V480M0 120H480M0 360H480M240 0V120M240 360V480M0 240H120M360 240H480";

/// Four diamonds and eight triangles; by bhava from the 1st, the top diamond, with where the rasi number goes.
const NORTH: [(Place, (i32, i32)); 12] = [
    ((240, 110, 2), (240, 222)),
    ((120, 45, 2), (120, 100)),
    ((45, 120, 1), (100, 125)),
    ((120, 240, 2), (222, 245)),
    ((45, 360, 1), (100, 365)),
    ((120, 435, 2), (120, 390)),
    ((240, 370, 2), (240, 268)),
    ((360, 435, 2), (360, 390)),
    ((435, 360, 1), (380, 365)),
    ((360, 240, 2), (258, 245)),
    ((435, 120, 1), (380, 125)),
    ((360, 45, 2), (360, 100)),
];
const NORTH_LINES: &str = "M0 0H480V480H0ZM0 0L480 480M480 0L0 480M240 0L480 240L240 480L0 240Z";

/// A 3×3 grid with an empty centre, each corner split into the triangles beside its two edge cells; by rasi from Aries.
const EAST: [Place; 12] = [
    (240, 80, 2),
    (107, 53, 1),
    (53, 107, 1),
    (80, 240, 2),
    (53, 373, 1),
    (107, 427, 1),
    (240, 400, 2),
    (373, 427, 1),
    (427, 373, 1),
    (400, 240, 2),
    (427, 107, 1),
    (373, 53, 1),
];
// No enclosing rectangle, as the book draws it.
const EAST_LINES: &str = "M0 160H480M0 320H480M160 0V480M320 0V480M0 0L160 160M480 0L320 160M0 480L160 320M480 480L320 320";

/// Label spacing: a column's width and a row's height, the rows tightening to the later ones when an area is crowded.
const COLUMN: i32 = 58;
const ROW: i32 = 20;
const TIGHTER_ROWS: [i32; 5] = [ROW, 18, 17, 16, 15];

/// An area's arudha strip.
#[derive(Clone, Copy)]
enum Strip {
    /// Its centre, its first baseline and how far on each wrapped row goes, away from the area's edge.
    Row(i32, i32, i32),
    /// Against a side triangle's outer edge: its x and the y it centres on.
    Column(i32, i32),
}

/// A strip's spacing: along a row, between rows, along a column; a row wraps after `STRIP_WRAP`.
const STRIP_PITCH: i32 = 26;
const STRIP_ROW: i32 = 16;
const STRIP_COLUMN: i32 = 15;
const STRIP_WRAP: usize = 4;

/// South Indian: the strip on a cell's bottom edge, below its centre, and the grahas lifted to clear it.
const SOUTH_STRIP: i32 = 47;
const SOUTH_LIFT: i32 = 8;

/// By bhava from the 1st: the outer edge of a triangle, near the lower tip of a diamond (the bottom one's bottom tip).
const NORTH_STRIPS: [Strip; 12] = [
    Strip::Row(240, 185, -STRIP_ROW),
    Strip::Row(120, 15, STRIP_ROW),
    Strip::Column(14, 125),
    Strip::Row(120, 305, -STRIP_ROW),
    Strip::Column(14, 365),
    Strip::Row(120, 472, -STRIP_ROW),
    Strip::Row(240, 425, -STRIP_ROW),
    Strip::Row(360, 472, -STRIP_ROW),
    Strip::Column(466, 365),
    Strip::Row(360, 305, -STRIP_ROW),
    Strip::Column(466, 125),
    Strip::Row(360, 15, STRIP_ROW),
];

/// By rasi from Aries: an edge cell's edge toward the centre (the side cells' lower one), a corner triangle's horizontal leg.
const EAST_STRIPS: [Strip; 12] = [
    Strip::Row(240, 147, -STRIP_ROW),
    Strip::Row(104, 14, STRIP_ROW),
    Strip::Row(56, 152, -STRIP_ROW),
    Strip::Row(80, 307, -STRIP_ROW),
    Strip::Row(56, 333, STRIP_ROW),
    Strip::Row(104, 472, -STRIP_ROW),
    Strip::Row(240, 334, STRIP_ROW),
    Strip::Row(376, 472, -STRIP_ROW),
    Strip::Row(424, 333, STRIP_ROW),
    Strip::Row(400, 307, -STRIP_ROW),
    Strip::Row(424, 152, -STRIP_ROW),
    Strip::Row(376, 14, STRIP_ROW),
];

/// The areas' corners, in `NORTH`'s order.
const NORTH_AREAS: [&[(i32, i32)]; 12] = [
    &[(240, 0), (360, 120), (240, 240), (120, 120)],
    &[(0, 0), (240, 0), (120, 120)],
    &[(0, 0), (120, 120), (0, 240)],
    &[(120, 120), (240, 240), (120, 360), (0, 240)],
    &[(0, 240), (120, 360), (0, 480)],
    &[(120, 360), (240, 480), (0, 480)],
    &[(240, 240), (360, 360), (240, 480), (120, 360)],
    &[(360, 360), (480, 480), (240, 480)],
    &[(480, 240), (480, 480), (360, 360)],
    &[(360, 120), (480, 240), (360, 360), (240, 240)],
    &[(480, 0), (480, 240), (360, 120)],
    &[(240, 0), (480, 0), (360, 120)],
];

/// The areas' corners, in `EAST`'s order.
const EAST_AREAS: [&[(i32, i32)]; 12] = [
    &[(160, 0), (320, 0), (320, 160), (160, 160)],
    &[(0, 0), (160, 0), (160, 160)],
    &[(0, 0), (160, 160), (0, 160)],
    &[(0, 160), (160, 160), (160, 320), (0, 320)],
    &[(0, 320), (160, 320), (0, 480)],
    &[(160, 320), (160, 480), (0, 480)],
    &[(160, 320), (320, 320), (320, 480), (160, 480)],
    &[(320, 320), (480, 480), (320, 480)],
    &[(320, 320), (480, 320), (480, 480)],
    &[(320, 160), (480, 160), (480, 320), (320, 320)],
    &[(480, 0), (480, 160), (320, 160)],
    &[(320, 0), (480, 0), (320, 160)],
];

/// The corners of area `i`.
fn outline(style: Style, i: usize) -> Vec<(i32, i32)> {
    match style {
        Style::South => {
            let (x, y, _) = SOUTH[i];
            vec![
                (x - 60, y - 60),
                (x + 60, y - 60),
                (x + 60, y + 60),
                (x - 60, y + 60),
            ]
        }
        Style::North => NORTH_AREAS[i].to_vec(),
        Style::East => EAST_AREAS[i].to_vec(),
    }
}

/// The left and right ends of a convex `outline` at height `y`, if it reaches that height.
fn slice(outline: &[(i32, i32)], y: i32) -> Option<(i32, i32)> {
    let edges = outline.iter().zip(outline.iter().cycle().skip(1));
    edges
        .filter(|(a, b)| a.1.min(b.1) <= y && y <= a.1.max(b.1))
        .flat_map(|(a, b)| {
            // Every edge is level, upright or at 45°, so the division is exact.
            let x = if a.1 == b.1 {
                a.0
            } else {
                a.0 + (y - a.1) * (b.0 - a.0) / (b.1 - a.1)
            };
            [x, if a.1 == b.1 { b.0 } else { x }]
        })
        .fold(None, |ends, x| match ends {
            None => Some((x, x)),
            Some((left, right)) => Some((x.min(left), x.max(right))),
        })
}

impl Label {
    /// An upper bound of its drawn width over the fonts of `--serif`, from measuring them: two letters, R and the degrees.
    fn width(&self) -> i32 {
        if self.class.starts_with("arudha") {
            return STRIP_PITCH;
        }
        22 + if self.retrograde { 7 } else { 0 } + 5 * self.degrees.chars().count() as i32
    }

    /// How far it reaches above its baseline: to R when it has one.
    fn rise(&self) -> i32 {
        if self.retrograde { 12 } else { 10 }
    }

    /// How far it reaches below its baseline: the degrees sit 4 lower.
    fn fall(&self) -> i32 {
        if self.degrees.is_empty() { 3 } else { 5 }
    }
}

/// Where an area's grahas and special lagnas may go.
struct Room {
    outline: Vec<(i32, i32)>,
    /// The area itself, which a row that follows it centres in even when it spills, to read as in this rasi.
    area: Vec<(i32, i32)>,
    /// Beyond these the arudha strip or the rasi number is in the way.
    x: (i32, i32),
    y: (i32, i32),
    centre: (i32, i32),
    /// Each row centres in the area at its height instead of on `centre`: East's corner triangles narrow toward their diagonal.
    follows_outline: bool,
}

impl Room {
    /// The rows from a first baseline: as many labels to a row as `cap` and its width allow, HL and GL (from `first_special`) in their own rows.
    /// `upward` fills from the last label and the last row, for an area that widens downward.
    fn rows(
        &self,
        labels: &[Label],
        first_special: usize,
        (rise, fall): (i32, i32),
        (row, cap, column): (i32, usize, i32),
        first: i32,
        upward: bool,
    ) -> Option<Vec<(i32, i32)>> {
        let mut places = vec![(0, 0); labels.len()];
        let mut order: Vec<usize> = (0..labels.len()).collect();
        if upward {
            order.reverse();
        }
        let mut order = order.into_iter().peekable();
        let mut y = first;
        let pitch = |a: &Label, b: &Label| column.max((a.width() + b.width()) / 2 + 2);
        while order.peek().is_some() {
            if y - rise < self.y.0 || y + fall > self.y.1 {
                return None;
            }
            let (top, bottom) = (
                slice(&self.outline, y - rise)?,
                slice(&self.outline, y + fall)?,
            );
            let left = top.0.max(bottom.0).max(self.x.0);
            let right = top.1.min(bottom.1).min(self.x.1);
            let mut members: Vec<usize> = Vec::new();
            let mut span = 0;
            while let Some(&next) = order.peek() {
                let label = &labels[next];
                let wider = match members.last() {
                    None => 0,
                    Some(&last) => {
                        if (last < first_special) != (next < first_special) || members.len() == cap
                        {
                            break;
                        }
                        pitch(&labels[last], label)
                    }
                };
                let first_half = labels[members.first().copied().unwrap_or(next)].width() / 2;
                if first_half + span + wider + label.width() / 2 > right - left {
                    break;
                }
                span += wider;
                members.push(next);
                order.next();
            }
            if members.is_empty() {
                return None;
            }
            if upward {
                members.reverse();
            }
            let wanted = if self.follows_outline {
                slice(&self.area, y - 5).map_or(self.centre.0, |(a, b)| (a + b) / 2)
            } else {
                self.centre.0
            };
            // Slide the row inward, off the strip or the area's edge.
            let start = wanted - span / 2;
            let (head, tail) = (members[0], members[members.len() - 1]);
            let slid = start
                + (left + labels[head].width() / 2 - start).max(0)
                + (right - labels[tail].width() / 2 - (start + span)).min(0);
            let mut x = slid;
            for (n, &member) in members.iter().enumerate() {
                if n > 0 {
                    x += pitch(&labels[members[n - 1]], &labels[member]);
                }
                places[member] = (x, y);
            }
            y += if upward { -row } else { row };
        }
        Some(places)
    }

    /// This room without the area's outline: the whole chart, still clear of the strip and the number.
    fn open(&self) -> Room {
        Room {
            outline: vec![(0, 0), (480, 0), (480, 480), (0, 480)],
            area: self.area.clone(),
            follows_outline: self.follows_outline,
            x: self.x,
            y: self.y,
            centre: self.centre,
        }
    }

    /// Places `labels`, the widest spacing and the closest to the centre that fits, if any does.
    fn fit(
        &self,
        spacings: [i32; 5],
        labels: &[Label],
        first_special: usize,
        columns: usize,
    ) -> Option<Vec<(i32, i32)>> {
        let rise = labels.iter().map(Label::rise).max().unwrap_or(0);
        let fall = labels.iter().map(Label::fall).max().unwrap_or(0);
        let height = |row: i32, cap: usize| {
            let rows = first_special.div_ceil(cap) + (labels.len() - first_special).div_ceil(cap);
            (rows as i32 - 1) * row
        };
        // Rows and columns centre on the area's point; +5 centres the text on its baseline.
        let centred = |row, cap| self.centre.1 + 5 - height(row, cap) / 2;
        for row in spacings.into_iter().filter(|row| *row >= rise + fall) {
            for (cap, column) in [(columns, COLUMN), (usize::MAX, 0)] {
                let layout = (row, cap, column);
                for shift in (0..=60).flat_map(|n| [-2 * n, 2 * n]) {
                    let first = centred(row, cap) + shift;
                    let extent = (rise, fall);
                    let found = self
                        .rows(labels, first_special, extent, layout, first, false)
                        .or_else(|| {
                            let last = first + height(row, cap);
                            self.rows(labels, first_special, extent, layout, last, true)
                        });
                    if found.is_some() {
                        return found;
                    }
                }
            }
        }
        None
    }
}

/// North Indian counts from `reference`, the table's rasi, so the two agree; of the special lagnas only HL and GL are drawn, as the book
/// does (ch. 5), and of the arudha padas only the bhava arudhas (ch. 9 §9.2), each chart its own.
/// With `varga`, the varga chakra: each body in its varga rasi (ch. 6 §6.2), with no degrees (ADR 0009).
pub fn drawing(
    lang: Lang,
    style: Style,
    chart: &Chart,
    reference: Rasi,
    varga: Option<Varga>,
) -> Drawing {
    let first = reference as usize;
    let bhava_arudhas = hora_chart::bhava_arudhas(varga.unwrap_or(Varga::RasiChart), chart);
    let areas = (0..12)
        .map(|i| {
            let (rasi, (x, y, columns), number) = match style {
                Style::South => (RASIS[i], SOUTH[i], None),
                Style::East => (RASIS[i], EAST[i], None),
                Style::North => {
                    let rasi = RASIS[(first + i) % 12];
                    let (place, (nx, ny)) = NORTH[i];
                    (rasi, place, Some((nx, ny, rasi as u8 + 1)))
                }
            };
            let mut here: Vec<_> = chart
                .entries()
                .iter()
                .filter(|e| placed(varga, e.longitude) == rasi)
                .collect();
            // Reading order follows the degrees, so a label's place says which body is further along the rasi;
            // a varga chakra has none, so it keeps the chart's order.
            if varga.is_none() {
                here.sort_by(|a, b| {
                    let degrees = |e: &&Entry| hora_chart::rasi(e.longitude).1;
                    degrees(a).total_cmp(&degrees(b))
                });
            }
            let specials: Vec<_> = chart
                .special_lagnas()
                .iter()
                .filter(|(lagna, longitude)| {
                    matches!(lagna, SpecialLagna::HoraLagna | SpecialLagna::GhatiLagna)
                        && placed(varga, *longitude) == rasi
                })
                .collect();
            let grahas = here.iter().map(|entry| {
                let id = body_id(entry.body);
                let class = if entry.body == Body::Lagna {
                    "lagna"
                } else {
                    "graha"
                };
                let short = format!("{id}-short");
                let mark = (
                    id.to_owned(),
                    short,
                    class,
                    entry.longitude,
                    entry.retrograde,
                );
                label(lang, mark, varga)
            });
            let lagnas = specials.iter().map(|&&(lagna, longitude)| {
                let id = kb::message_id("special-lagna", lagna);
                let short = kb::message_id("special-lagna-abbreviation", lagna);
                let mark = (id, short, "special", longitude, false);
                label(lang, mark, varga)
            });
            let mut labels: Vec<_> = grahas.chain(lagnas).collect();
            let mut arudhas: Vec<_> = (0..12)
                .filter(|&n| bhava_arudhas[n] == rasi)
                .map(|n| arudha_label(lang, n, rasi))
                .collect();
            let strip = match style {
                Style::South => Strip::Row(x, y + SOUTH_STRIP, -STRIP_ROW),
                Style::North => NORTH_STRIPS[i],
                Style::East => EAST_STRIPS[i],
            };
            let room = room(style, i, (x, y), number, strip, &mut arudhas);
            // shortcut: too full for the area at the tightest rows, labels keep their degrees and spill past its edges, clear of each other, its strip and its number but maybe not a neighbour's labels; upgrade if a real chart shows that.
            let places = room
                .fit(TIGHTER_ROWS, &labels, here.len(), columns)
                // Tightest first, to spill as little as it can.
                .or_else(|| {
                    let mut tightest = TIGHTER_ROWS;
                    tightest.reverse();
                    room.open().fit(tightest, &labels, here.len(), columns)
                })
                .unwrap_or_else(|| vec![room.centre; labels.len()]);
            for (label, (x, y)) in labels.iter_mut().zip(places) {
                (label.x, label.y) = (x, y);
            }
            Area {
                number,
                labels,
                arudhas,
            }
        })
        .collect();
    let lines = match style {
        Style::South => SOUTH_LINES,
        Style::North => NORTH_LINES,
        Style::East => EAST_LINES,
    };
    Drawing { lines, areas }
}

/// Places area `i`'s `arudhas` on its strip and returns the room left for the rest: clear of them and of the rasi `number`.
fn room(
    style: Style,
    i: usize,
    (x, y): (i32, i32),
    number: Option<(i32, i32, u8)>,
    strip: Strip,
    arudhas: &mut [Label],
) -> Room {
    let outline = outline(style, i);
    let along = |n: usize, len: usize, pitch: i32| n as i32 * pitch - (len as i32 - 1) * pitch / 2;
    let mut xs = (0, 480);
    let mut ys = (0, 480);
    let len = arudhas.len();
    for (n, label) in arudhas.iter_mut().enumerate() {
        (label.x, label.y) = match strip {
            Strip::Row(x, y, wrap) => {
                let in_row = (len - n / STRIP_WRAP * STRIP_WRAP).min(STRIP_WRAP);
                (
                    x + along(n % STRIP_WRAP, in_row, STRIP_PITCH),
                    y + (n / STRIP_WRAP) as i32 * wrap,
                )
            }
            Strip::Column(x, y) => (x, y + along(n, len, STRIP_COLUMN)),
        };
    }
    if let Some(last) = arudhas.last() {
        match strip {
            Strip::Row(_, _, wrap) if wrap < 0 => ys.1 = last.y - last.rise() - 1,
            Strip::Row(..) => ys.0 = last.y + last.fall() + 1,
            Strip::Column(x, _) if x < 240 => xs.0 = x + last.width() / 2 + 1,
            Strip::Column(x, _) => xs.1 = x - last.width() / 2 - 1,
        }
    }
    if let Some((nx, ny, _)) = number {
        let (dx, dy) = (nx - x, ny - y);
        if dx.abs() > dy.abs() {
            if dx > 0 {
                xs.1 = xs.1.min(nx - 9);
            } else {
                xs.0 = xs.0.max(nx + 9);
            }
        } else if dy > 0 {
            ys.1 = ys.1.min(ny - 9);
        } else {
            ys.0 = ys.0.max(ny + 3);
        }
    }
    Room {
        area: outline.clone(),
        outline,
        x: xs,
        y: ys,
        centre: (
            x,
            if style == Style::South {
                y - SOUTH_LIFT
            } else {
                y
            },
        ),
        follows_outline: style == Style::East && EAST[i].2 == 1,
    }
}

/// The label of the arudha pada of the `n`th bhava from 0 (ch. 9 §9.2), its name shared with its table row. AL and UL are lagnas, so they
/// look it.
fn arudha_label(lang: Lang, n: usize, rasi: Rasi) -> Label {
    let t = |id: &str| LOCALES.lookup(&lang.id(), id);
    let id = super::arudha_id(n);
    let args = [
        ("graha", FluentValue::from(t(&id))),
        ("retrograde", FluentValue::from("no")),
        ("rasi", FluentValue::from(t(rasi_id(rasi)))),
    ]
    .map(|(name, value)| (name.into(), value))
    .into();
    Label {
        x: 0,
        y: 0,
        short: t(&format!("{id}-short")),
        name: LOCALES.lookup_with_args(&lang.id(), "chakra-label-varga", &args),
        id,
        class: if matches!(n, 0 | 11) {
            "arudha-lagna"
        } else {
            "arudha"
        },
        retrograde: false,
        degrees: String::new(),
    }
}

/// The rasi `longitude` is drawn in: its own, or its rasi in `varga`.
fn placed(varga: Option<Varga>, longitude: f64) -> Rasi {
    match varga {
        Some(varga) => hora_chart::varga(varga, longitude),
        None => hora_chart::rasi(longitude).0,
    }
}

/// The label of `(id, short id, class, longitude, retrograde)`, not yet placed, in the rasi chakra or `varga`'s.
fn label(
    lang: Lang,
    (id, short, class, longitude, retrograde): (String, String, &'static str, f64, bool),
    varga: Option<Varga>,
) -> Label {
    let t = |id: &str| LOCALES.lookup(&lang.id(), id);
    let degrees = hora_chart::rasi(longitude).1;
    let rasi = placed(varga, longitude);
    let minutes = hora_chart::arcseconds(degrees) / 60;
    // Only the lagna and the grahas show degrees, and only in the rasi chakra; the Special lagnas table gives HL's and GL's.
    let (message, degrees) = if varga.is_some() || class == "special" {
        ("chakra-label-varga", String::new())
    } else {
        (
            "chakra-label",
            format!("{}°{:02}′", minutes / 60, minutes % 60),
        )
    };
    let args = [
        ("graha", FluentValue::from(t(&id))),
        (
            "retrograde",
            FluentValue::from(if retrograde { "yes" } else { "no" }),
        ),
        ("degrees", FluentValue::from(minutes / 60)),
        ("minutes", FluentValue::from(minutes % 60)),
        ("rasi", FluentValue::from(t(rasi_id(rasi)))),
    ]
    .map(|(name, value)| (name.into(), value))
    .into();
    Label {
        x: 0,
        y: 0,
        short: t(&short),
        id,
        class,
        retrograde,
        degrees,
        name: LOCALES.lookup_with_args(&lang.id(), message, &args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A chart with `body` at `degrees` into the rasi `rasi` (0 = Aries), each moving forward.
    fn entry(body: Body, rasi: u8, degrees: f64) -> Entry {
        Entry {
            body,
            longitude: f64::from(rasi) * 30.0 + degrees,
            retrograde: false,
        }
    }

    /// Lord Sree Rama's chart, ch. 1 §1.3.4, Example 1: the book gives only rasis, so the degrees are made up.
    fn rama() -> Chart {
        Chart::new(rama_entries(), Vec::new())
    }

    /// [`rama`]'s entries, to edit before building a chart of them.
    fn rama_entries() -> [Entry; 10] {
        use Body::*;
        [
            entry(Lagna, 3, 10.0),
            entry(Sun, 0, 10.0),
            entry(Moon, 3, 5.0),
            entry(Mars, 9, 10.0),
            entry(Mercury, 1, 10.0),
            entry(Jupiter, 3, 20.0),
            entry(Venus, 11, 10.0),
            entry(Saturn, 6, 10.0),
            entry(Rahu, 8, 10.0),
            entry(Ketu, 2, 10.0),
        ]
    }

    /// Each area's rasi number and its labels' two letters.
    fn contents(drawing: &Drawing) -> Vec<(Option<u8>, Vec<&str>)> {
        drawing
            .areas
            .iter()
            .map(|a| {
                let shorts = a.labels.iter().map(|l| l.short.as_str()).collect();
                (a.number.map(|(_, _, n)| n), shorts)
            })
            .collect()
    }

    /// Where each area's labels centre, in the drawing's order.
    fn places(style: Style) -> [(i32, i32); 12] {
        match style {
            Style::South => SOUTH.map(|(x, y, _)| (x, y)),
            Style::North => NORTH.map(|((x, y, _), _)| (x, y)),
            Style::East => EAST.map(|(x, y, _)| (x, y)),
        }
    }

    /// Whether the areas' centres turn steadily one way round the chart's centre, once.
    fn turn(style: Style, counter_clockwise: bool) -> bool {
        let angles =
            places(style).map(|(x, y)| f64::from(240 - y).atan2(f64::from(x - 240)).to_degrees());
        let sign = if counter_clockwise { 1.0 } else { -1.0 };
        let steps: Vec<f64> = angles
            .iter()
            .zip(angles.iter().cycle().skip(1))
            .map(|(a, b)| (sign * (b - a)).rem_euclid(360.0))
            .collect();
        steps.iter().all(|s| *s > 0.0) && (steps.iter().sum::<f64>() - 360.0).abs() < 1e-6
    }

    const RAMA_BY_RASI: [&[&str]; 12] = [
        &["Su"],
        &["Me"],
        &["Ke"],
        // By degree: Mo 5°, La 10°, Ju 20°.
        &["Mo", "La", "Ju"],
        &[],
        &[],
        &["Sa"],
        &[],
        &["Ra"],
        &["Ma"],
        &[],
        &["Ve"],
    ];

    #[test]
    fn south_indian_has_fixed_rasis_clockwise_from_pisces_at_the_top_left() {
        let drawing = drawing(Lang::En, Style::South, &rama(), Rasi::Cancer, None);
        let expected: Vec<_> = RAMA_BY_RASI.iter().map(|s| (None, s.to_vec())).collect();
        assert_eq!(contents(&drawing), expected);
        let places = places(Style::South);
        assert_eq!(places[11], (60, 60));
        assert!(turn(Style::South, false));
        // Gemini and Virgo take the corners, so Cancer and Leo are the right column's two cells.
        assert_eq!(
            places[2..6],
            [(420, 60), (420, 180), (420, 300), (420, 420)]
        );
    }

    #[test]
    fn north_indian_has_the_lagnas_bhava_in_the_top_diamond_and_rasi_numbers() {
        let drawing = drawing(Lang::En, Style::North, &rama(), Rasi::Cancer, None);
        let rasis = [4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3];
        let expected: Vec<_> = rasis
            .iter()
            .map(|&n| (Some(n), RAMA_BY_RASI[usize::from(n) - 1].to_vec()))
            .collect();
        assert_eq!(contents(&drawing), expected);
        let places = places(Style::North);
        let (top, right) = (places[0], places[9]);
        assert_eq!((top.0, right.1), (240, 240));
        assert!(top.1 < 240 && right.0 > 240);
        assert!(turn(Style::North, true));
    }

    #[test]
    fn east_indian_has_fixed_rasis_counter_clockwise_from_aries_at_the_top() {
        let drawing = drawing(Lang::En, Style::East, &rama(), Rasi::Cancer, None);
        let expected: Vec<_> = RAMA_BY_RASI.iter().map(|s| (None, s.to_vec())).collect();
        assert_eq!(contents(&drawing), expected);
        let places = places(Style::East);
        // Aries, Cancer, Libra and Capricorn are the edges' middle cells.
        assert_eq!(
            [0, 3, 6, 9].map(|rasi| places[rasi]),
            [(240, 80), (80, 240), (240, 400), (400, 240)]
        );
        assert!(turn(Style::East, true));
        // Each corner cell holds two rasis, mirrored across its diagonal.
        for (a, b) in [(1, 2), (4, 5), (7, 8), (10, 11)].map(|(a, b)| (places[a], places[b])) {
            assert_eq!(
                (a.0 / 160, a.1 / 160),
                (b.0 / 160, b.1 / 160),
                "{a:?} {b:?}"
            );
            assert!(
                a != b && (a.0 - b.0).abs() == (a.1 - b.1).abs(),
                "{a:?} {b:?}"
            );
        }
    }

    #[test]
    fn north_indian_counts_from_the_bhavas_reference_point() {
        // Rama's lagna is in Cancer (4), his Sun in Aries (1).
        for (reference, first) in [(Rasi::Cancer, 4), (Rasi::Aries, 1)] {
            let drawing = drawing(Lang::En, Style::North, &rama(), reference, None);
            assert_eq!(drawing.areas[0].number.map(|(_, _, n)| n), Some(first));
        }
    }

    #[test]
    fn east_indian_corner_rows_centre_between_the_diagonal_and_the_edge() {
        // Taurus spans x from y to 160 at height y, Pisces from 320 to 480 − y.
        for (rasi, slant, edges) in [(1, 1, 160), (11, -1, 800)] {
            let mut entries = rama_entries();
            // Sun to Mercury join Rama's Mercury (Taurus) or Venus (Pisces).
            for (n, e) in entries[1..5].iter_mut().enumerate() {
                *e = entry(e.body, rasi, n as f64);
            }
            let chart = Chart::new(entries, Vec::new());
            let drawing = drawing(Lang::En, Style::East, &chart, Rasi::Cancer, None);
            let labels = &drawing.areas[usize::from(rasi)].labels;
            assert!(labels.len() >= 4);
            for l in labels {
                // A row near the tip slides inward, off the diagonal.
                assert!(
                    (l.x - (edges + slant * (l.y - 5)) / 2).abs() <= 3,
                    "{} {}",
                    l.x,
                    l.y
                );
            }
        }
    }

    #[test]
    fn hl_and_gl_follow_their_rasis_grahas_in_each_style_but_sl_and_bl_are_not_drawn() {
        use SpecialLagna::*;
        // HL in Pisces with Rama's Venus, GL in Cancer with his Moon, lagna and Jupiter.
        let specials = vec![
            (HoraLagna, 11.0 * 30.0 + 16.0 + 4.5 / 60.0),
            (GhatiLagna, 3.0 * 30.0 + 1.0),
            (SreeLagna, 5.0 * 30.0),
            (BhaavaLagna, 6.0 * 30.0),
        ];
        let chart = Chart::new(rama_entries(), specials);
        for style in [Style::South, Style::North, Style::East] {
            let drawing = drawing(Lang::En, style, &chart, Rasi::Cancer, None);
            let area = |rasi: usize| {
                let i = match style {
                    Style::North => (rasi + 12 - 3) % 12,
                    _ => rasi,
                };
                &drawing.areas[i]
            };
            let shorts = |rasi| contents_of(area(rasi));
            assert_eq!(shorts(11), ["Ve", "HL"], "{style:?}");
            // GL at 1° comes after Jupiter at 20°: it is not ordered among the grahas.
            assert_eq!(shorts(3), ["Mo", "La", "Ju", "GL"], "{style:?}");
            assert_eq!(shorts(5), [""; 0], "{style:?}");
            assert_eq!(shorts(6), ["Sa"], "{style:?}");
            let hl = area(11).labels.last().unwrap();
            assert_eq!(
                (hl.class, hl.degrees.as_str(), hl.name.as_str()),
                ("special", "", "Hora lagna in Pisces")
            );
            // Its own row, below the graha's.
            assert!(hl.y > area(11).labels[0].y, "{style:?}");
        }
        let hu = drawing(Lang::Hu, Style::South, &chart, Rasi::Cancer, None);
        let names: Vec<_> = hu
            .areas
            .iter()
            .flat_map(|a| &a.labels)
            .filter(|l| l.class == "special")
            .map(|l| (l.short.as_str(), l.name.as_str()))
            .collect();
        assert_eq!(
            names,
            [("GL", "Ghati lagna, Rák"), ("HL", "Hora lagna, Halak")]
        );
    }

    #[test]
    fn a_varga_chakra_places_each_body_by_its_varga_rasi_in_chart_order_without_degrees() {
        // Navamsa (ch. 6 §6.2.9): each of Rama's made-up 10° is the 4th part, Moon's 5° the 2nd, Jupiter's 20° the 7th.
        let navamsa: [&[&str]; 12] = [
            &["Ma", "Me"],
            &[],
            &[],
            &["Su", "Ra"],
            &["Mo"],
            &[],
            &["La", "Ve"],
            &[],
            &[],
            &["Ju", "Sa", "Ke"],
            &[],
            &[],
        ];
        let mut entries = rama_entries();
        entries[5].retrograde = true;
        let chart = Chart::new(entries, Vec::new());
        for style in [Style::South, Style::East] {
            let drawing = drawing(Lang::En, style, &chart, Rasi::Libra, Some(Varga::Navamsa));
            let expected: Vec<_> = navamsa.iter().map(|s| (None, s.to_vec())).collect();
            assert_eq!(contents(&drawing), expected, "{style:?}");
        }
        let north = drawing(
            Lang::En,
            Style::North,
            &chart,
            Rasi::Libra,
            Some(Varga::Navamsa),
        );
        assert_eq!(contents(&north)[0], (Some(7), vec!["La", "Ve"]));
        let labels: Vec<_> = north.areas.iter().flat_map(|a| &a.labels).collect();
        assert!(labels.iter().all(|l| l.degrees.is_empty()));
        let ju = labels.iter().find(|l| l.short == "Ju").unwrap();
        assert_eq!(ju.name, "Jupiter, retrograde, in Capricorn");
        let hu = drawing(
            Lang::Hu,
            Style::South,
            &chart,
            Rasi::Libra,
            Some(Varga::Navamsa),
        );
        let su = &hu.areas[3].labels[0];
        assert_eq!(su.name, "Nap, Rák");
    }

    /// A label's box: left, top, right, bottom.
    fn bounds(l: &Label) -> (i32, i32, i32, i32) {
        (
            l.x - l.width() / 2,
            l.y - l.rise(),
            l.x + l.width() / 2,
            l.y + l.fall(),
        )
    }

    /// Whether a convex `outline` holds the box's four corners.
    fn holds(outline: &[(i32, i32)], (left, top, right, bottom): (i32, i32, i32, i32)) -> bool {
        [top, bottom]
            .iter()
            .all(|&y| slice(outline, y).is_some_and(|(a, b)| a <= left && right <= b))
    }

    /// Every label lies in its area, and no two labels or a label and a rasi number overlap; the failures, named by `what`.
    fn misfits(what: &str, style: Style, drawing: &Drawing) -> Vec<String> {
        let mut problems = Vec::new();
        let mut boxes: Vec<(String, (i32, i32, i32, i32))> = Vec::new();
        for (i, area) in drawing.areas.iter().enumerate() {
            let outline = outline(style, i);
            for label in area.labels.iter().chain(&area.arudhas) {
                let (bounds, name) = (bounds(label), format!("{i}/{}", label.short));
                if !holds(&outline, bounds) {
                    problems.push(format!(
                        "{what} {style:?}: {name} {bounds:?} leaves its area"
                    ));
                }
                boxes.push((name, bounds));
            }
            if let Some((x, y, n)) = area.number {
                boxes.push((format!("{i}/#{n}"), (x - 7, y - 8, x + 7, y + 2)));
            }
        }
        for (n, (a, x)) in boxes.iter().enumerate() {
            for (b, y) in &boxes[n + 1..] {
                if x.0 < y.2 && y.0 < x.2 && x.1 < y.3 && y.1 < x.3 {
                    problems.push(format!("{what} {style:?}: {a} {x:?} overlaps {b} {y:?}"));
                }
            }
        }
        problems
    }

    #[tokio::test]
    async fn chart_14s_crowded_leo_fits_with_hl_and_gl_there_in_each_style() {
        use SpecialLagna::*;
        // Chart 14's lagna and five grahas are in Leo (ch. 13, p. 175); HL and GL join them as the worst case.
        let sky = crate::book::chart_14_rajiv_gandhi_1944().await;
        let specials = vec![
            (HoraLagna, 4.0 * 30.0 + 20.0),
            (GhatiLagna, 4.0 * 30.0 + 25.0),
        ];
        // In place of the chart's own special lagnas, not beside them.
        let chart = Chart::new(
            hora_chart::chart(&sky).entries().try_into().unwrap(),
            specials,
        );
        for style in [Style::South, Style::North, Style::East] {
            let drawing = drawing(Lang::En, style, &chart, Rasi::Leo, None);
            let i = if style == Style::North {
                0
            } else {
                Rasi::Leo as usize
            };
            let shorts = contents_of(&drawing.areas[i]);
            assert_eq!(shorts, ["Su", "Ju", "La", "Mo", "Ve", "Me", "HL", "GL"]);
            assert_eq!(misfits("chart 14", style, &drawing), [""; 0]);
        }
    }

    #[tokio::test]
    async fn the_most_crowded_pinned_charts_never_overlap_and_fit_counted_from_the_lagna() {
        // Of all the charts in `book`, those with the fullest areas: up to 11 labels and an arudha pada in one D-2 rasi.
        let charts = [
            (
                "chart 8",
                crate::book::chart_8_guess_his_livelihood_1946().await,
            ),
            ("chart 12", crate::book::chart_12_sav_exercise_1958().await),
            ("chart 14", crate::book::chart_14_rajiv_gandhi_1944().await),
            (
                "chart 21",
                crate::book::chart_21_john_f_kennedy_jr_1960().await,
            ),
            (
                "chart 31",
                crate::book::chart_31_an_electrical_engineer_1947().await,
            ),
        ];
        for (name, sky) in charts {
            let chart = hora_chart::chart(&sky);
            for varga in hora_chart::VARGAS {
                let shown = (varga != Varga::RasiChart).then_some(varga);
                for style in [Style::South, Style::North, Style::East] {
                    for reference in RASIS {
                        let drawing = drawing(Lang::En, style, &chart, reference, shown);
                        let what = format!("{name} {varga:?} from {reference:?}");
                        // Only a chart counted from elsewhere than the lagna may put a full rasi in a narrow triangle and spill.
                        let from_lagna = style != Style::North
                            || reference == placed(shown, chart.lagna().longitude);
                        let problems: Vec<_> = misfits(&what, style, &drawing)
                            .into_iter()
                            .filter(|p| from_lagna || p.contains("overlaps"))
                            .collect();
                        assert_eq!(problems, [""; 0]);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn an_area_too_full_keeps_its_degrees_and_spills() {
        let sky = crate::book::chart_14_rajiv_gandhi_1944().await;
        let chart = hora_chart::chart(&sky);
        // Leo as the 5th bhava's side triangle: too narrow for its seven labels and A10.
        let drawing = drawing(Lang::En, Style::North, &chart, Rasi::Aries, None);
        let leo = &drawing.areas[4];
        assert_eq!(leo.labels.len(), 7);
        let grahas = leo.labels.iter().filter(|l| l.class != "special");
        assert!(grahas.into_iter().all(|l| !l.degrees.is_empty()));
        assert!(!misfits("chart 14", Style::North, &drawing).is_empty());
    }

    #[tokio::test]
    async fn a_spilled_label_stays_in_its_own_rasi() {
        use hora_swe::{Ayanamsa, Calendar, NodeType, Request, SunriseDefinition, UtcOffset};
        // A birth with Aquarius too full for its East Indian triangle; its Sun once spilled into Pisces's.
        let sky = hora_swe::calculate(Request {
            year: 1998,
            month: 2,
            day: 15,
            hour: 3,
            minute: 48,
            second: 17,
            calendar: Calendar::Gregorian,
            utc_offset: UtcOffset::Stated(0),
            latitude: -34.0,
            longitude: 35.0,
            altitude: 0.0,
            ayanamsa: Ayanamsa::Lahiri,
            node: NodeType::Mean,
            sunrise_definition: SunriseDefinition::UpperLimbApparent,
        })
        .await
        .unwrap();
        let chart = hora_chart::chart(&sky);
        let drawing = drawing(Lang::En, Style::East, &chart, Rasi::Aquarius, None);
        for (i, area) in drawing.areas.iter().enumerate() {
            for l in &area.labels {
                // Where the eye reads it: its middle, not its corners, which a spill may cross.
                let (left, right) = slice(&outline(Style::East, i), l.y - 5).unwrap();
                assert!(left <= l.x && l.x <= right, "{i}/{} at {}", l.short, l.x);
            }
        }
    }

    #[test]
    fn labels_too_many_for_their_area_spill_but_never_overlap() {
        // Unreal, but every graha retrograde in one rasi with HL and GL fits no North Indian bottom triangle.
        let mut entries = rama_entries();
        for (n, e) in entries.iter_mut().enumerate() {
            *e = entry(e.body, 4, n as f64 * 2.0);
            e.retrograde = true;
        }
        let specials = vec![
            (SpecialLagna::HoraLagna, 4.0 * 30.0 + 25.0),
            (SpecialLagna::GhatiLagna, 4.0 * 30.0 + 26.0),
        ];
        let chart = Chart::new(entries, specials);
        for reference in RASIS {
            let drawing = drawing(Lang::En, Style::North, &chart, reference, None);
            let leo = drawing.areas.iter().find(|a| a.labels.len() == 12).unwrap();
            let boxes: Vec<_> = leo.labels.iter().chain(&leo.arudhas).map(bounds).collect();
            for (n, x) in boxes.iter().enumerate() {
                for y in &boxes[n + 1..] {
                    let overlap = x.0 < y.2 && y.0 < x.2 && x.1 < y.3 && y.1 < x.3;
                    assert!(!overlap, "{reference:?}: {x:?} {y:?}");
                }
            }
        }
    }

    #[test]
    fn a_strip_stays_in_order_wrapping_after_four_away_from_its_edge() {
        let mut six: Vec<_> = (0..6)
            .map(|n| arudha_label(Lang::En, n, Rasi::Aries))
            .collect();
        let row = Strip::Row(180, 107, -STRIP_ROW);
        room(Style::South, 0, (180, 60), None, row, &mut six);
        let places: Vec<_> = six.iter().map(|l| (l.short.as_str(), l.x, l.y)).collect();
        assert_eq!(
            places,
            [
                ("AL", 141, 107),
                ("A2", 167, 107),
                ("A3", 193, 107),
                ("A4", 219, 107),
                ("A5", 167, 91),
                ("A6", 193, 91),
            ]
        );
        let side = NORTH_STRIPS[2];
        let mut two: Vec<_> = (0..2)
            .map(|n| arudha_label(Lang::En, n, Rasi::Aries))
            .collect();
        let room = room(Style::North, 2, (45, 120), None, side, &mut two);
        let (first, second) = (&two[0], &two[1]);
        assert_eq!(
            (first.x, second.x, second.y - first.y),
            (14, 14, STRIP_COLUMN)
        );
        // The grahas keep clear of the column.
        assert!(room.x.0 > first.x + first.width() / 2);
    }

    /// An area's labels' two letters.
    fn contents_of(area: &Area) -> Vec<&str> {
        area.labels.iter().map(|l| l.short.as_str()).collect()
    }

    #[test]
    fn degrees_are_truncated_to_the_arcminute() {
        let mut entries = rama_entries();
        entries[1] = entry(Body::Sun, 0, 29.0 + 59.0 / 60.0 + 59.0 / 3600.0);
        entries[2] = entry(Body::Moon, 3, 4.0 + 5.0 / 60.0);
        let chart = Chart::new(entries, Vec::new());
        let drawing = drawing(Lang::En, Style::South, &chart, Rasi::Cancer, None);
        let degrees = |rasi: usize, short: &str| {
            let label = drawing.areas[rasi].labels.iter().find(|l| l.short == short);
            label.unwrap().degrees.clone()
        };
        assert_eq!(degrees(0, "Su"), "29°59′");
        assert_eq!(degrees(3, "Mo"), "4°05′");
    }

    #[test]
    fn labels_carry_r_and_their_names_in_both_languages() {
        let mut entries = rama_entries();
        entries[7] = Entry {
            body: Body::Saturn,
            longitude: 11.0 * 30.0 + 14.0 + 52.5 / 60.0,
            retrograde: true,
        };
        entries[0] = entry(Body::Lagna, 3, 1.0);
        let chart = Chart::new(entries, Vec::new());
        for (lang, saturn, lagna) in [
            (
                Lang::En,
                ("Sa", "Saturn, retrograde, 14 degrees 52 minutes in Pisces"),
                ("La", "Lagna, 1 degree 0 minutes in Cancer"),
            ),
            (
                Lang::Hu,
                ("Sz", "Szaturnusz, retrográd, 14 fok 52 perc, Halak"),
                ("La", "Lagna, 1 fok 0 perc, Rák"),
            ),
        ] {
            let drawing = drawing(lang, Style::South, &chart, Rasi::Cancer, None);
            let labels: Vec<_> = drawing.areas.iter().flat_map(|a| &a.labels).collect();
            let find = |short| labels.iter().find(|l| l.short == short).unwrap();
            let (sa, la) = (find(saturn.0), find(lagna.0));
            assert_eq!(
                (sa.retrograde, sa.class, sa.name.as_str()),
                (true, "graha", saturn.1)
            );
            assert_eq!(
                (la.retrograde, la.class, la.name.as_str()),
                (false, "lagna", lagna.1)
            );
            assert!(labels.iter().filter(|l| l.retrograde).count() == 1);
        }
        let hu = drawing(Lang::Hu, Style::South, &rama(), Rasi::Cancer, None);
        let shorts: Vec<_> = hu
            .areas
            .iter()
            .flat_map(|a| &a.labels)
            .map(|l| l.short.as_str())
            .collect();
        // Rasi order: Ar Su, Ta Me, Ge Ke, Cn Mo La Ju, Li Sa, Sg Ra, Cp Ma, Pi Ve.
        assert_eq!(
            shorts,
            ["Na", "Me", "Ke", "Ho", "La", "Ju", "Sz", "Ra", "Ma", "Ve"]
        );
    }
}

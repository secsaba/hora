//! The rasi chart drawn in the three styles of ch. 1 §1.3.4, Figure 1, as the parts of an inline SVG.

use fluent_templates::Loader;
use fluent_templates::fluent_bundle::FluentValue;
use hora_chart::{Body, Entry, RASIS};

use super::{LOCALES, Lang, body_id, rasi_id};

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
}

/// One body's label, placed.
pub struct Label {
    pub x: i32,
    pub y: i32,
    /// Two letters.
    pub short: String,
    pub lagna: bool,
    pub retrograde: bool,
    /// Within the rasi, to the arcminute, truncated.
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

// shortcut: labels stack at a fixed spacing, so five or more in one triangle spill past its edges; fit them to the area when real charts show it.
/// Label spacing: a column's width and a row's height.
const COLUMN: i32 = 58;
const ROW: i32 = 20;

pub fn drawing(lang: Lang, style: Style, chart: &[Entry; 10]) -> Drawing {
    // North Indian puts the lagna's rasi in the 1st bhava, whatever reference point the table counts bhavas from.
    let lagna = hora_chart::rasi_of(chart, Body::Lagna) as usize;
    let areas = (0..12)
        .map(|i| {
            let (rasi, (x, y, columns), number) = match style {
                Style::South => (RASIS[i], SOUTH[i], None),
                Style::East => (RASIS[i], EAST[i], None),
                Style::North => {
                    let rasi = RASIS[(lagna + i) % 12];
                    let (place, (nx, ny)) = NORTH[i];
                    (rasi, place, Some((nx, ny, rasi as u8 + 1)))
                }
            };
            let here: Vec<_> = chart
                .iter()
                .filter(|e| hora_chart::rasi(e.longitude).0 == rasi)
                .collect();
            let rows = here.len().div_ceil(columns) as i32;
            let labels = here
                .iter()
                .enumerate()
                .map(|(n, entry)| {
                    let (row, column) = ((n / columns) as i32, (n % columns) as i32);
                    let in_row = (here.len() - n / columns * columns).min(columns) as i32;
                    // Rows and columns centre on the area's point; +5 centres the text on its baseline.
                    let x = x + (2 * column - in_row + 1) * COLUMN / 2;
                    let y = y + (2 * row - rows + 1) * ROW / 2 + 5;
                    label(lang, entry, x, y)
                })
                .collect();
            Area { number, labels }
        })
        .collect();
    let lines = match style {
        Style::South => SOUTH_LINES,
        Style::North => NORTH_LINES,
        Style::East => EAST_LINES,
    };
    Drawing { lines, areas }
}

fn label(lang: Lang, entry: &Entry, x: i32, y: i32) -> Label {
    let t = |id: &str| LOCALES.lookup(&lang.id(), id);
    let (rasi, degrees) = hora_chart::rasi(entry.longitude);
    let minutes = hora_chart::arcseconds(degrees) / 60;
    let id = body_id(entry.body);
    let args = [
        ("graha", FluentValue::from(t(id))),
        (
            "retrograde",
            FluentValue::from(if entry.retrograde { "yes" } else { "no" }),
        ),
        ("degrees", FluentValue::from(minutes / 60)),
        ("minutes", FluentValue::from(minutes % 60)),
        ("rasi", FluentValue::from(t(rasi_id(rasi)))),
    ]
    .map(|(name, value)| (name.into(), value))
    .into();
    Label {
        x,
        y,
        short: t(&format!("{id}-short")),
        lagna: entry.body == Body::Lagna,
        retrograde: entry.retrograde,
        degrees: format!("{}°{:02}′", minutes / 60, minutes % 60),
        name: LOCALES.lookup_with_args(&lang.id(), "chakra-label", &args),
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
    fn rama() -> [Entry; 10] {
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
        &["La", "Mo", "Ju"],
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
        let drawing = drawing(Lang::En, Style::South, &rama());
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
        let drawing = drawing(Lang::En, Style::North, &rama());
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
        let drawing = drawing(Lang::En, Style::East, &rama());
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
    fn north_indian_counts_from_the_lagna_whatever_the_bhavas_reference_point() {
        // The drawing takes no reference point; the 1st bhava's rasi is the lagna's.
        let mut chart = rama();
        chart[0] = entry(Body::Lagna, 7, 1.0);
        let drawing = drawing(Lang::En, Style::North, &chart);
        assert_eq!(drawing.areas[0].number.map(|(_, _, n)| n), Some(8));
    }

    #[test]
    fn degrees_are_truncated_to_the_arcminute() {
        let mut chart = rama();
        chart[1] = entry(Body::Sun, 0, 29.0 + 59.0 / 60.0 + 59.0 / 3600.0);
        chart[2] = entry(Body::Moon, 3, 4.0 + 5.0 / 60.0);
        let drawing = drawing(Lang::En, Style::South, &chart);
        let degrees = |rasi: usize, short: &str| {
            let label = drawing.areas[rasi].labels.iter().find(|l| l.short == short);
            label.unwrap().degrees.clone()
        };
        assert_eq!(degrees(0, "Su"), "29°59′");
        assert_eq!(degrees(3, "Mo"), "4°05′");
    }

    #[test]
    fn labels_carry_r_and_their_names_in_both_languages() {
        let mut chart = rama();
        chart[7] = Entry {
            body: Body::Saturn,
            longitude: 11.0 * 30.0 + 14.0 + 52.5 / 60.0,
            retrograde: true,
        };
        chart[0] = entry(Body::Lagna, 3, 1.0);
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
            let drawing = drawing(lang, Style::South, &chart);
            let labels: Vec<_> = drawing.areas.iter().flat_map(|a| &a.labels).collect();
            let find = |short| labels.iter().find(|l| l.short == short).unwrap();
            let (sa, la) = (find(saturn.0), find(lagna.0));
            assert_eq!(
                (sa.retrograde, sa.lagna, sa.name.as_str()),
                (true, false, saturn.1)
            );
            assert_eq!(
                (la.retrograde, la.lagna, la.name.as_str()),
                (false, true, lagna.1)
            );
            assert!(labels.iter().filter(|l| l.retrograde).count() == 1);
        }
        let hu = drawing(Lang::Hu, Style::South, &rama());
        let shorts: Vec<_> = hu
            .areas
            .iter()
            .flat_map(|a| &a.labels)
            .map(|l| l.short.as_str())
            .collect();
        // Rasi order: Ar Su, Ta Me, Ge Ke, Cn La Mo Ju, Li Sa, Sg Ra, Cp Ma, Pi Ve.
        assert_eq!(
            shorts,
            ["Na", "Me", "Ke", "La", "Ho", "Ju", "Sz", "Ra", "Ma", "Ve"]
        );
    }
}

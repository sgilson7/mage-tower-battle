//! The draw list: what the canvas shows, as a flat list of integers the
//! page replays. Part of the canvas kit (`kits/canvas/README.md`), after
//! Peter's renderer for Mandible.
//!
//! The core decides every position, size, colour and word. The page's
//! `web/render2d.js` reads an op code, then that op's arguments, and calls
//! the matching canvas method. It holds no number of its own: the op codes,
//! the palette's role names, the shapes, the scales and the canvas size come
//! from `assets()`, which the page reads once.
//!
//! Colours are role names (`ink`, `paper`, ...). The page looks each one up
//! as a CSS custom property (`--ink`), so the stylesheet keeps the colours
//! and the dark theme, and the core keeps which role each thing is drawn in.
//!
//! Integers only. A shape's points are in `UNIT`ths of the shape's size, and
//! a scale is that size in `UNIT`ths of a canvas pixel. The page divides
//! once, when it reads the assets.

use serde_json::json;

/// The denominator for shape points and scales.
pub const UNIT: i32 = 1000;

/// The canvas size in CSS pixels. The page multiplies by the device pixel
/// ratio for a sharp picture; that is presentation.
pub const WIDTH: i32 = 1280;
pub const HEIGHT: i32 = 720;

/// What the page does next. The discriminant is the code on the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// colour: wipes the canvas, then fills it
    Clear = 0,
    /// x, y, w, h, fill
    Rect = 1,
    /// shape, x, y, scale, fill, stroke, line width
    Shape = 2,
    /// stroke, line width, n, then n (x, y) pairs
    Line = 3,
    /// x, y, size, fill, align, n, then n Unicode code points
    Text = 4,
}

pub const OPS: [(Op, &str); 5] = [
    (Op::Clear, "clear"),
    (Op::Rect, "rect"),
    (Op::Shape, "shape"),
    (Op::Line, "line"),
    (Op::Text, "text"),
];

/// A palette role. The page draws it in the CSS custom property `--<name>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Paper = 0,
    Ink = 1,
    Muted = 2,
    Accent = 3,
    Shade = 4,
    Panel = 5,
    Band = 6,
    Guide = 7,
    Drawn = 8,
    Good = 9,
    Bad = 10,
    Gold = 11,
    CellOpen = 12,
    CellScenery = 13,
    CellRoute = 14,
    CellTower = 15,
    CellEntry = 16,
    Glass = 17,
    Ivory = 18,
    Lava = 19,
    Bait = 20,
    Sub = 21,
    GhostOk = 22,
    GhostBad = 23,
    Note0 = 24,
    Note1 = 25,
    Note2 = 26,
    Note3 = 27,
    Destruction = 28,
    Alteration = 29,
    Conjuration = 30,
    Illusion = 31,
    Restoration = 32,
    Dial = 33,
    RingPay = 34,
    RingMod = 35,
    RingTrig = 36,
    /// Nothing: a clear in this colour leaves the canvas see-through.
    Clear = 37,
}

pub const PALETTE: [(Color, &str); 38] = [
    (Color::Paper, "paper"),
    (Color::Ink, "ink"),
    (Color::Muted, "muted"),
    (Color::Accent, "accent"),
    (Color::Shade, "shade"),
    (Color::Panel, "panel"),
    (Color::Band, "band"),
    (Color::Guide, "guide"),
    (Color::Drawn, "drawn"),
    (Color::Good, "good"),
    (Color::Bad, "bad"),
    (Color::Gold, "gold"),
    (Color::CellOpen, "cell_open"),
    (Color::CellScenery, "cell_scenery"),
    (Color::CellRoute, "cell_route"),
    (Color::CellTower, "cell_tower"),
    (Color::CellEntry, "cell_entry"),
    (Color::Glass, "glass"),
    (Color::Ivory, "ivory"),
    (Color::Lava, "lava"),
    (Color::Bait, "bait"),
    (Color::Sub, "sub"),
    (Color::GhostOk, "ghost_ok"),
    (Color::GhostBad, "ghost_bad"),
    (Color::Note0, "note0"),
    (Color::Note1, "note1"),
    (Color::Note2, "note2"),
    (Color::Note3, "note3"),
    (Color::Destruction, "destruction"),
    (Color::Alteration, "alteration"),
    (Color::Conjuration, "conjuration"),
    (Color::Illusion, "illusion"),
    (Color::Restoration, "restoration"),
    (Color::Dial, "dial"),
    (Color::RingPay, "ring_pay"),
    (Color::RingMod, "ring_mod"),
    (Color::RingTrig, "ring_trig"),
    (Color::Clear, "clear"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Square = 0,
    Diamond = 1,
    Arrow = 2,
    Triangle = 3,
    Star = 4,
    Cross = 5,
    Disc = 6,
    /// A triangle pointing down, for the seal's mark (T32).
    Down = 7,
}

/// Each shape is one or more closed polygons, centred on 0, 0, with points
/// in `UNIT`ths of the shape's size: a square of size 1 runs from -500 to
/// 500. Indexed by `Shape`.
pub const SHAPES: [&[&[[i32; 2]]]; 8] = [
    &[&[[-500, -500], [500, -500], [500, 500], [-500, 500]]],
    &[&[[0, -600], [600, 0], [0, 600], [-600, 0]]],
    &[&[
        [-500, -300],
        [100, -300],
        [100, -600],
        [600, 0],
        [100, 600],
        [100, 300],
        [-500, 300],
    ]],
    &[&[[0, -560], [520, 420], [-520, 420]]],
    &[&[
        [0, -600],
        [140, -190],
        [570, -185],
        [225, 75],
        [355, 485],
        [0, 235],
        [-355, 485],
        [-225, 75],
        [-570, -185],
        [-140, -190],
    ]],
    &[&[
        [-180, -560],
        [180, -560],
        [180, -180],
        [560, -180],
        [560, 180],
        [180, 180],
        [180, 560],
        [-180, 560],
        [-180, 180],
        [-560, 180],
        [-560, -180],
        [-180, -180],
    ]],
    &[&[
        [0, -500],
        [354, -354],
        [500, 0],
        [354, 354],
        [0, 500],
        [-354, 354],
        [-500, 0],
        [-354, -354],
    ]],
    &[&[[-520, -420], [520, -420], [0, 560]]],
];

/// A shape's size in canvas pixels, in `UNIT`ths: `Tile` draws a square
/// 24 pixels wide. Indexed by `Scale`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Small = 0,
    Tile = 1,
    Large = 2,
}

pub const SCALES: [i32; 3] = [8 * UNIT, 24 * UNIT, 64 * UNIT];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left = 0,
    Center = 1,
    Right = 2,
}

/// Canvas `textAlign` values, indexed by `Align`.
pub const ALIGNS: [&str; 3] = ["left", "center", "right"];

/// A draw list being built. `finish` hands the integers to the shim, which
/// passes them to the page as an `Int32Array`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DrawList {
    out: Vec<i32>,
}

impl DrawList {
    pub fn new() -> DrawList {
        DrawList::default()
    }

    pub fn clear(&mut self, color: Color) -> &mut Self {
        self.out.extend([Op::Clear as i32, color as i32]);
        self
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, fill: Color) -> &mut Self {
        self.out.extend([Op::Rect as i32, x, y, w, h, fill as i32]);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn shape(
        &mut self,
        shape: Shape,
        x: i32,
        y: i32,
        scale: Scale,
        fill: Color,
        stroke: Color,
        width: i32,
    ) -> &mut Self {
        self.out.extend([
            Op::Shape as i32,
            shape as i32,
            x,
            y,
            scale as i32,
            fill as i32,
            stroke as i32,
            width,
        ]);
        self
    }

    pub fn line(&mut self, points: &[(i32, i32)], stroke: Color, width: i32) -> &mut Self {
        self.out
            .extend([Op::Line as i32, stroke as i32, width, points.len() as i32]);
        for (x, y) in points {
            self.out.extend([*x, *y]);
        }
        self
    }

    /// Text the core chose, usually a string from `data/`. Sent as code
    /// points, so the list stays integers.
    pub fn text(&mut self, x: i32, y: i32, size: i32, fill: Color, align: Align, words: &str) -> &mut Self {
        let codes: Vec<i32> = words.chars().map(|c| c as i32).collect();
        self.out.extend([
            Op::Text as i32,
            x,
            y,
            size,
            fill as i32,
            align as i32,
            codes.len() as i32,
        ]);
        self.out.extend(codes);
        self
    }

    pub fn finish(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.out)
    }
}

/// Everything the page needs to replay a list, as JSON text. Sent once.
pub fn assets() -> String {
    let ops: serde_json::Map<String, serde_json::Value> = OPS
        .iter()
        .map(|(op, name)| (name.to_string(), json!(*op as i32)))
        .collect();
    let palette: Vec<&str> = PALETTE.iter().map(|(_, name)| *name).collect();
    json!({
        "ops": ops,
        "palette": palette,
        "shapes": SHAPES,
        "scales": SCALES,
        "unit": UNIT,
        "aligns": ALIGNS,
        "size": [WIDTH, HEIGHT],
    })
    .to_string()
}

/// Walks a list as the page does and returns the number of ops, or `None`
/// if an op code is unknown or the list ends in the middle of an op. A
/// builder that forgot an argument fails here before it fails on screen.
pub fn walk(list: &[i32]) -> Option<usize> {
    let mut at = 0usize;
    let mut ops = 0usize;
    let arg = |k: usize| list.get(k).copied();
    while at < list.len() {
        let code = list[at];
        let fixed = match code {
            c if c == Op::Clear as i32 => 1,
            c if c == Op::Rect as i32 => 5,
            c if c == Op::Shape as i32 => 7,
            c if c == Op::Line as i32 => 3 + 2 * usize::try_from(arg(at + 3)?).ok()?,
            c if c == Op::Text as i32 => 6 + usize::try_from(arg(at + 6)?).ok()?,
            _ => return None,
        };
        at += 1 + fixed;
        if at > list.len() {
            return None;
        }
        ops += 1;
    }
    Some(ops)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Vec<i32> {
        DrawList::new()
            .clear(Color::Paper)
            .rect(10, 20, 30, 40, Color::Muted)
            .shape(Shape::Diamond, 100, 50, Scale::Tile, Color::Accent, Color::Ink, 2)
            .line(&[(0, 0), (5, 6)], Color::Ink, 1)
            .text(320, 20, 16, Color::Ink, Align::Center, "Hé")
            .finish()
    }

    #[test]
    fn each_op_writes_its_code_then_its_arguments() {
        assert_eq!(
            example(),
            vec![
                0, 0, //
                1, 10, 20, 30, 40, 2, //
                2, 1, 100, 50, 1, 3, 1, 2, //
                3, 1, 1, 2, 0, 0, 5, 6, //
                4, 320, 20, 16, 1, 1, 2, 72, 233,
            ]
        );
    }

    #[test]
    fn a_built_list_walks_to_its_end_and_a_cut_one_does_not() {
        let list = example();
        assert_eq!(walk(&list), Some(5));
        for cut in 1..list.len() {
            let short = &list[..cut];
            let ends_on_an_op = walk(short).is_some();
            // a cut is only whole when it falls between two ops
            let boundaries = [2, 8, 16, 24];
            assert_eq!(ends_on_an_op, boundaries.contains(&cut), "cut at {cut}");
        }
        assert_eq!(walk(&[99]), None, "an unknown op code");
        assert_eq!(walk(&[]), Some(0));
    }

    #[test]
    fn the_assets_name_every_op_role_shape_and_scale_in_order() {
        let a: serde_json::Value = serde_json::from_str(&assets()).expect("json");
        // the names web/render2d.js and the stylesheet's custom properties use
        assert_eq!(
            a["ops"],
            json!({"clear": 0, "rect": 1, "shape": 2, "line": 3, "text": 4})
        );
        // every role is a custom property in web/style.css (checked by tests/browser)
        assert_eq!(a["palette"][0], json!("paper"));
        assert_eq!(a["palette"][Color::Restoration as usize], json!("restoration"));
        assert_eq!(a["palette"].as_array().map(Vec::len), Some(PALETTE.len()));
        for (op, name) in OPS {
            assert_eq!(a["ops"][name], json!(op as i32), "{name}");
        }
        for (color, name) in PALETTE {
            assert_eq!(a["palette"][color as usize], json!(name));
        }
        assert_eq!(a["shapes"].as_array().map(Vec::len), Some(SHAPES.len()));
        assert_eq!(a["scales"][Scale::Large as usize], json!(SCALES[2]));
        assert_eq!(a["aligns"][Align::Right as usize], json!("right"));
        assert_eq!(a["unit"], json!(UNIT));
        assert_eq!(a["size"], json!([WIDTH, HEIGHT]));
    }

    #[test]
    fn op_codes_and_role_indices_are_distinct_and_dense() {
        for (k, (op, _)) in OPS.iter().enumerate() {
            assert_eq!(*op as usize, k);
        }
        for (k, (color, _)) in PALETTE.iter().enumerate() {
            assert_eq!(*color as usize, k);
        }
    }

    #[test]
    fn every_shape_is_closed_polygons_of_three_points_or_more() {
        for (k, shape) in SHAPES.iter().enumerate() {
            assert!(!shape.is_empty(), "shape {k} is empty");
            for polygon in shape.iter() {
                assert!(polygon.len() >= 3, "shape {k} has a polygon of {}", polygon.len());
            }
        }
    }
}

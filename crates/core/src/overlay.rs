//! The 2D overlays the mini-games and the layout editor draw on, as the
//! canvas kit's draw lists (`draw.rs`), 1280 × 720 logical pixels. The page
//! replays them and sends pointer positions back; the core maps those to
//! glyph units or grid cells.

use crate::arena::{self, Arena, Cell};
use crate::battle::Placed;
use crate::content::Content;
use crate::draw::{Align, Color, DrawList, Scale, Shape};
use crate::glyph;
use crate::prep::{self, Activity};
use crate::rhythm::{Lane, CAST_LANE};

/// The layout map's top-left corner and cell size in overlay pixels.
pub const MAP_X: i32 = 352;
pub const MAP_Y: i32 = 72;
pub const MAP_CELL: i32 = 18;

/// Pointer CSS pixels on an overlay of CSS size (w, h) to logical pixels.
pub fn logical(x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    (x * draw_w() / w.max(1), y * draw_h() / h.max(1))
}
fn draw_w() -> i32 {
    crate::draw::WIDTH
}
fn draw_h() -> i32 {
    crate::draw::HEIGHT
}

/// The layout map cell under a logical point.
pub fn map_cell(lx: i32, ly: i32) -> Option<(i32, i32)> {
    let (x, z) = (
        (lx - MAP_X).div_euclid(MAP_CELL),
        (ly - MAP_Y).div_euclid(MAP_CELL),
    );
    arena::inside(x, z).then_some((x, z))
}

/// The drawing board: the guide, its band, the player's line, and each
/// glyph's name under it.
pub fn drawing(a: &Activity, names: &[String], heading: &str, live: &str) -> Vec<i32> {
    let mut d = DrawList::new();
    let Activity::Drawing {
        guide,
        drawn,
        band,
        glyphs,
        ..
    } = a
    else {
        return d.finish();
    };
    let (ox, oy) = prep::draw_origin(*glyphs);
    let u = prep::units_per_px(*glyphs);
    let px = |p: &(i32, i32)| (ox + p.0 / u, oy + p.1 / u);
    d.rect(0, 0, draw_w(), draw_h(), Color::Shade);
    d.rect(40, 150, draw_w() - 80, 460, Color::Panel);
    d.text(draw_w() / 2, 186, 26, Color::Ink, Align::Center, heading);
    let g: Vec<(i32, i32)> = guide.iter().map(px).collect();
    d.line(&g, Color::Band, (band * 2 / u).max(4));
    d.line(&g, Color::Guide, 3);
    if let Some(first) = g.first() {
        d.shape(
            Shape::Disc,
            first.0,
            first.1,
            Scale::Tile,
            Color::Gold,
            Color::Ink,
            2,
        );
    }
    if drawn.len() > 1 {
        let l: Vec<(i32, i32)> = drawn.iter().map(px).collect();
        d.line(&l, Color::Drawn, 6);
    }
    for (k, n) in names.iter().enumerate() {
        let x = ox + (k as i32 * (glyph::BOX + glyph::GAP) + glyph::BOX / 2) / u;
        d.text(x, oy + glyph::BOX / u + 30, 18, Color::Ink, Align::Center, n);
    }
    d.text(draw_w() - 80, 186, 30, Color::Good, Align::Right, live);
    d.finish()
}

/// The sorting game's and the seal's panels, in overlay pixels: x, y, w,
/// h. The page places the panel's text and buttons in the same box, and
/// the overlay draws the moving parts over it (T32).
pub const SORT_BOX: [i32; 4] = [30, 80, 940, 600];
pub const SEAL_BOX: [i32; 4] = [190, 80, 900, 580];

/// A box in overlay pixels as CSS percentages of the stage.
pub fn css_box(b: [i32; 4]) -> serde_json::Value {
    let pct = |v: i32, of: i32| crate::strings::css_pct(v * 1000 / of);
    serde_json::json!({"left": pct(b[0], draw_w()), "top": pct(b[1], draw_h()), "width": pct(b[2], draw_w()), "height": pct(b[3], draw_h())})
}

fn school_colour(k: usize) -> Color {
    [
        Color::Destruction,
        Color::Alteration,
        Color::Conjuration,
        Color::Illusion,
        Color::Restoration,
    ][k.min(4)]
}

/// The book-sorting game (T20): the book falling toward the shelves, in
/// its school's colour and shape; `fallen` is how far, in thousandths. The
/// shelves, the pile and the count are the page's panel underneath.
pub fn sorting(book: u8, fallen: i32) -> Vec<i32> {
    let mut d = DrawList::new();
    let shapes = [
        Shape::Triangle,
        Shape::Square,
        Shape::Diamond,
        Shape::Star,
        Shape::Cross,
    ];
    d.clear(Color::Clear);
    let b = (book as usize).min(4);
    let [bx, by, bw, _] = SORT_BOX;
    let (w, h) = (96, 120);
    let x = bx + bw / 2 - w / 2;
    let y = by + 90 + fallen.clamp(0, 1000) * 220 / 1000;
    d.rect(x - 4, y - 4, w + 8, h + 8, Color::Ink);
    d.rect(x, y, w, h, school_colour(b));
    d.shape(
        shapes[b],
        x + w / 2,
        y + h / 2,
        Scale::Large,
        Color::Paper,
        Color::Ink,
        3,
    );
    d.finish()
}

fn circle(d: &mut DrawList, cx: i32, cy: i32, r: i32, colour: Color, width: i32) {
    use crate::fx;
    let ring: Vec<(i32, i32)> = (0..=64)
        .map(|i| {
            let a = fx::TURN * i / 64;
            (
                cx + fx::mul(fx::sin(a), r * fx::ONE) / fx::ONE,
                cy - fx::mul(fx::cos(a), r * fx::ONE) / fx::ONE,
            )
        })
        .collect();
    d.line(&ring, colour, width);
}

/// Sealing a spell (T22, T32): the dial, a band for each piece in its
/// kind's colour, its glyph where the ring has turned it, a sealed ring's
/// glyph ringed in green under the gold mark. The words are the page's
/// panel beside it.
pub fn seal(c: &Content, pieces: &[u16], rings: &[i32], at: usize) -> Vec<i32> {
    use crate::content::PieceKind;
    use crate::fx;
    let mut d = DrawList::new();
    d.clear(Color::Clear);
    let [bx, by, _, _] = SEAL_BOX;
    let (cx, cy, r) = (bx + 40 + 235, by + 84 + 235, 235);
    // the dial: a disc drawn as a thick ring, then its rim
    circle(&mut d, cx, cy, r / 2, Color::Dial, r);
    circle(&mut d, cx, cy, r, Color::Ink, 5);
    let n = rings.len().max(1) as i32;
    let step = 150 / n;
    let band = (step * 7 / 10).max(6);
    let point = |r: i32, a: i32| {
        (
            cx + fx::mul(fx::sin(a), r * fx::ONE) / fx::ONE,
            cy - fx::mul(fx::cos(a), r * fx::ONE) / fx::ONE,
        )
    };
    for (k, &turn) in rings.iter().enumerate() {
        let rr = 205 - k as i32 * step - step / 2;
        let piece = pieces.get(k).copied().unwrap_or(0) as usize;
        let colour = match c.file.pieces.get(piece).map(|p| &p.kind) {
            Some(PieceKind::Payload { .. }) => Color::RingPay,
            Some(PieceKind::Trigger { .. }) | Some(PieceKind::Branch) => Color::RingTrig,
            _ => Color::RingMod,
        };
        circle(&mut d, cx, cy, rr, colour, band);
        if k == at {
            circle(&mut d, cx, cy, rr - band / 2, Color::Gold, 3);
            circle(&mut d, cx, cy, rr + band / 2, Color::Gold, 3);
        }
        let a = turn * fx::TURN / crate::prep::SEAL_STEPS;
        let (gx, gy) = point(rr, a);
        let size = (band + 4).min(36);
        let g: Vec<(i32, i32)> = glyph::glyph(&c.file.pieces[piece].glyph)
            .iter()
            .map(|&(x, y)| {
                (
                    gx + (x - glyph::BOX / 2) * size / glyph::BOX,
                    gy + (y - glyph::BOX / 2) * size / glyph::BOX,
                )
            })
            .collect();
        d.line(&g, school_colour(c.piece_school[piece]), 4);
        if k < at {
            circle(&mut d, gx, gy, size * 3 / 4, Color::Good, 4);
        }
    }
    // the mark, and a dashed line from it to the middle
    let top = cy - r;
    let mut y = top + 20;
    while y < cy - 44 {
        d.line(&[(cx, y), (cx, (y + 8).min(cy - 44))], Color::Gold, 3);
        y += 16;
    }
    d.shape(Shape::Down, cx, top + 4, Scale::Tile, Color::Gold, Color::Ink, 3);
    circle(&mut d, cx, cy, 22, Color::Paper, 44);
    circle(&mut d, cx, cy, 44, Color::Ink, 4);
    d.text(
        cx,
        cy,
        28,
        Color::Ink,
        Align::Center,
        &(at + 1).min(rings.len()).to_string(),
    );
    d.finish()
}

/// A seat on a picture of the field (T32, T33): which seat, where its
/// middle is in thousandths of a cell, and who holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spot {
    pub seat: i32,
    pub x: i32,
    pub z: i32,
    /// The holder, and their initials, empty for a free seat.
    pub who: Option<u16>,
    pub text: String,
    /// The student picked at office hours holds it.
    pub on: bool,
    /// The holder chose it, rather than being given the first free seat.
    pub chose: bool,
}

/// The tower's seats stand on a ring this far round its middle, and a
/// seat's disc is this wide, both in thousandths of a cell.
pub const SPOT_RING: i32 = 3600;
pub const SPOT_SIZE: i32 = 1500;

/// Where the seats are on the field: the tower's in a ring from the top,
/// clockwise, then each sub-tower's just above it.
pub fn spots(a: &Arena, tower_seats: i32, subs: &[(i32, i32)]) -> Vec<(i32, i32, i32)> {
    use crate::fx;
    let (cx, cz) = (a.tower.0 * 1000 + 500, a.tower.1 * 1000 + 500);
    let n = tower_seats.max(1);
    let mut out: Vec<(i32, i32, i32)> = (0..tower_seats)
        .map(|i| {
            let ang = i * fx::TURN / n;
            (
                i,
                cx + (fx::sin(ang) as i64 * SPOT_RING as i64 / fx::ONE as i64) as i32,
                cz - (fx::cos(ang) as i64 * SPOT_RING as i64 / fx::ONE as i64) as i32,
            )
        })
        .collect();
    for (k, &(x, z)) in subs.iter().enumerate() {
        out.push((
            crate::battle::SUB_SEAT + k as i32,
            x * 1000 + 500,
            z * 1000 + 500 - 1100,
        ));
    }
    out
}

/// The spot under a logical point on the layout map.
pub fn spot_at(spots: &[Spot], lx: i32, ly: i32) -> Option<&Spot> {
    let r = SPOT_SIZE * MAP_CELL / 2000 + 2;
    spots.iter().find(|s| {
        let (x, y) = (MAP_X + s.x * MAP_CELL / 1000, MAP_Y + s.z * MAP_CELL / 1000);
        (lx - x).abs() <= r && (ly - y).abs() <= r
    })
}

/// A feature's swatch: the colour role its cells are drawn in, by name.
pub fn swatch(id: &str) -> &'static str {
    match id {
        "glass_wall" => "glass",
        "ivory_wall" => "ivory",
        "lava_pool" => "lava",
        "bait" => "bait",
        _ => "sub",
    }
}

/// The interview's note lane: notes slide left to the hit line.
pub fn interview(lane: &Lane, heading: &str, keys: &[String; 4], last: &str) -> Vec<i32> {
    let mut d = DrawList::new();
    d.rect(0, 0, draw_w(), draw_h(), Color::Shade);
    d.rect(160, 470, 960, 170, Color::Panel);
    d.text(draw_w() / 2, 500, 22, Color::Ink, Align::Center, heading);
    let (hit_x, y) = (240, 575);
    d.rect(hit_x - 4, 530, 8, 90, Color::Accent);
    for n in &lane.notes {
        if n.got.is_some() {
            continue;
        }
        let x = hit_x + (n.at - lane.tick) * 4;
        if !(150..=1110).contains(&x) {
            continue;
        }
        let colour = [Color::Note0, Color::Note1, Color::Note2, Color::Note3][n.lane as usize];
        let shape = if n.lane == CAST_LANE {
            Shape::Star
        } else {
            Shape::Disc
        };
        d.shape(shape, x, y, Scale::Large, colour, Color::Ink, 3);
        d.text(x, y, 22, Color::Paper, Align::Center, &keys[n.lane as usize]);
    }
    d.text(1090, 500, 22, Color::Ink, Align::Right, last);
    d.finish()
}

/// The layout editor's map of the field, and a preview of the piece being placed.
pub fn layout(
    c: &Content,
    a: &Arena,
    placed: &[Placed],
    rising: &[Placed],
    ghost: Option<(&Placed, bool)>,
    seats: &[Spot],
) -> Vec<i32> {
    let mut d = DrawList::new();
    d.rect(0, 0, draw_w(), draw_h(), Color::Shade);
    d.rect(
        MAP_X - 12,
        MAP_Y - 12,
        arena::SIZE * MAP_CELL + 24,
        arena::SIZE * MAP_CELL + 24,
        Color::Panel,
    );
    let route = a.route();
    for z in 0..arena::SIZE {
        for x in 0..arena::SIZE {
            let colour = match a.at(x, z) {
                Cell::Scenery => Color::CellScenery,
                Cell::Tower => Color::CellTower,
                Cell::Feature(_) => Color::Ivory,
                Cell::Open if (x, z) == a.entry => Color::CellEntry,
                Cell::Open if route.contains(&(x, z)) => Color::CellRoute,
                Cell::Open => Color::CellOpen,
            };
            d.rect(
                MAP_X + x * MAP_CELL,
                MAP_Y + z * MAP_CELL,
                MAP_CELL - 1,
                MAP_CELL - 1,
                colour,
            );
        }
    }
    // what is still rising shows as a smaller square of its colour
    let all = placed.iter().map(|p| (p, 2)).chain(rising.iter().map(|p| (p, 5)));
    for (p, inset) in all {
        let colour = match swatch(&c.file.features[p.feature as usize].id) {
            "glass" => Color::Glass,
            "ivory" => Color::Ivory,
            "lava" => Color::Lava,
            "bait" => Color::Bait,
            _ => Color::Sub,
        };
        for &(x, z) in &p.cells {
            d.rect(
                MAP_X + x * MAP_CELL + inset,
                MAP_Y + z * MAP_CELL + inset,
                MAP_CELL - 1 - inset * 2,
                MAP_CELL - 1 - inset * 2,
                colour,
            );
        }
    }
    if let Some((g, ok)) = ghost {
        let colour = if ok { Color::GhostOk } else { Color::GhostBad };
        for &(x, z) in &g.cells {
            if arena::inside(x, z) {
                d.rect(
                    MAP_X + x * MAP_CELL,
                    MAP_Y + z * MAP_CELL,
                    MAP_CELL - 1,
                    MAP_CELL - 1,
                    colour,
                );
            }
        }
    }
    // seating students (T33): a disc for each seat, the holder's initials on it
    for sp in seats {
        let (x, y) = (MAP_X + sp.x * MAP_CELL / 1000, MAP_Y + sp.z * MAP_CELL / 1000);
        let (fill, ink) = match (sp.on, sp.text.is_empty(), sp.chose) {
            (true, _, _) => (Color::Gold, Color::Ink),
            (_, true, _) => (Color::Paper, Color::Muted),
            (_, _, true) => (Color::Accent, Color::Paper),
            _ => (Color::Sub, Color::Ink),
        };
        d.shape(Shape::Disc, x, y, Scale::Tile, fill, Color::Ink, 2);
        d.text(
            x,
            y + 1,
            11,
            ink,
            Align::Center,
            if sp.text.is_empty() { "+" } else { &sp.text },
        );
    }
    d.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::walk;

    #[test]
    fn a_pointer_maps_to_logical_pixels_and_map_cells() {
        // a 640 × 360 CSS overlay is half size: (100, 50) is (200, 100)
        assert_eq!(logical(100, 50, 640, 360), (200, 100));
        assert_eq!(map_cell(MAP_X, MAP_Y), Some((0, 0)));
        assert_eq!(
            map_cell(MAP_X + MAP_CELL * 31 + 17, MAP_Y + MAP_CELL * 5 + 1),
            Some((31, 5))
        );
        assert_eq!(map_cell(MAP_X - 1, MAP_Y), None);
        assert_eq!(map_cell(MAP_X + MAP_CELL * 32, MAP_Y), None);
    }

    #[test]
    fn each_overlay_is_a_complete_draw_list() {
        let c = crate::content::shipped();
        let a = crate::battle::field(3);
        assert!(
            walk(&layout(&c, &a, &[], &[], None, &[])).unwrap() > arena::SIZE as usize * arena::SIZE as usize
        );
        let lane = Lane::new(&mut crate::rng::Rng::new(1), false, 4, 9);
        let keys = ["J".to_string(), "K".to_string(), "L".to_string(), "✦".to_string()];
        assert!(walk(&interview(&lane, "Interview", &keys, "")).is_some());
        let mut p = crate::prep::Prep::new(&c, 0, 1, 1);
        p.start_day(&c, 600);
        p.pos = crate::prep::station_pos(crate::prep::Station::Podium).add(crate::geom::v3(
            0,
            0,
            -(crate::fx::ONE * 18 / 10),
        ));
        p.act(&c, "podium.teach", "").unwrap();
        assert!(walk(&drawing(&p.activity, &["Fire Bolt".into()], "Teach", "0%")).is_some());
    }
}

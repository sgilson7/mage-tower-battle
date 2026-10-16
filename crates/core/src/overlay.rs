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
        let colour = match c.file.features[p.feature as usize].id.as_str() {
            "glass_wall" => Color::Glass,
            "ivory_wall" => Color::Ivory,
            "lava_pool" => Color::Lava,
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
        assert!(walk(&layout(&c, &a, &[], &[], None)).unwrap() > arena::SIZE as usize * arena::SIZE as usize);
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

//! Jinxes (T62): spells the archmage casts into a rival's tower from the
//! invasion book. A barrier no one walks through, a bog that slows walking
//! to a third, black ice on a walkway that slides. And the walkways
//! themselves: the flagstone ways between stations, where walking is
//! quicker. Distances are Fx metres on the room's floor; a jinx keeps its
//! place in whole centimetres so it travels between browsers unchanged.

use serde::{Deserialize, Serialize};

use crate::content::{self, Content, JinxShape, PieceKind};
use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, TPS, V3};
use crate::prep::{station_pos, Station, BODY, PROP, ROOM_RADIUS};

/// The ways worn into the floor between stations (T59): down each column
/// and across the middle through the invasion book, so the room shows the
/// order of the work. Walking on one is quicker (T62).
pub const WALKWAYS: [(Station, Station); 7] = [
    (Station::Recruit, Station::Podium),
    (Station::Podium, Station::Office),
    (Station::Library, Station::Lab),
    (Station::Lab, Station::Map),
    (Station::Podium, Station::Book),
    (Station::Book, Station::Lab),
    (Station::Office, Station::Book),
];

/// A walkway's flagstones stop this far short of each station's centre.
pub const WALKWAY_GAP: Fx = PROP + ONE * 40 / 100;
/// Half a walkway's width: the flagstones are a metre across.
pub const WALKWAY_HALF: Fx = ONE * 55 / 100;
/// A click this close to a walkway puts black ice on it.
pub const ICE_PICK: Fx = ONE * 3 / 2;
/// A barrier keeps this far from every station's centre, so each station
/// can still be reached from some side.
pub const STATION_CLEAR: Fx = PROP + BODY * 2;
/// A barrier turns in steps of an eighth of a turn: 0 runs along x.
pub const TURNS: u8 = 4;

/// A jinx standing in a tower (T62).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jinx {
    pub piece: u16,
    /// The seat of the archmage who cast it.
    pub from: u8,
    /// A barrier's or a bog's centre, in cm.
    pub x: i32,
    pub z: i32,
    /// A barrier's turn, 0 to `TURNS - 1`.
    pub turn: u8,
    /// The walkway black ice lies on.
    pub way: u8,
    /// Ticks it has left.
    pub left: i32,
}

pub fn shape(c: &Content, piece: u16) -> Option<JinxShape> {
    match c.file.pieces.get(piece as usize).map(|p| &p.kind) {
        Some(PieceKind::Jinx { shape, .. }) => Some(*shape),
        _ => None,
    }
}

pub fn mana(c: &Content, piece: u16) -> Option<i32> {
    match c.file.pieces.get(piece as usize).map(|p| &p.kind) {
        Some(PieceKind::Jinx { mana, .. }) => Some(*mana),
        _ => None,
    }
}

/// How long a jinx drawn this well lasts, in seconds.
pub fn seconds(c: &Content, score: i32) -> i32 {
    let [lo, hi] = c.balance().jinx_s;
    lo + (hi - lo) * score.clamp(0, 100) / 100
}

/// Where a walkway's flagstones start and stop.
pub fn walkway_ends(k: usize) -> (V3, V3) {
    let (a, b) = WALKWAYS[k % WALKWAYS.len()];
    let (a, b) = (station_pos(a), station_pos(b));
    let u = b.sub(a).flat().with_len(WALKWAY_GAP);
    (a.add(u), b.sub(u))
}

/// How far a point is from a walkway's middle line.
pub fn from_walkway(p: V3, k: usize) -> Fx {
    let (a, b) = walkway_ends(k);
    let ab = b.sub(a).flat();
    let ap = p.sub(a).flat();
    let len2 = ab.dot(ab).max(1);
    let t = (ap.dot(ab).clamp(0, len2) * ONE as i64 / len2) as Fx;
    p.sub(a.add(ab.scale(t))).flat_len()
}

pub fn on_walkway(p: V3) -> bool {
    (0..WALKWAYS.len()).any(|k| from_walkway(p, k) <= WALKWAY_HALF)
}

/// The walkway nearest a point, and how far it is.
pub fn nearest_walkway(p: V3) -> (usize, Fx) {
    (0..WALKWAYS.len())
        .map(|k| (k, from_walkway(p, k)))
        .min_by_key(|w| w.1)
        .unwrap_or((0, Fx::MAX))
}

/// A barrier's long axis, as (cos, sin) of its turn.
fn axis(turn: u8) -> (Fx, Fx) {
    let a = (turn % TURNS) as i32 * fx::TURN / (TURNS as i32 * 2);
    (fx::cos(a), fx::sin(a))
}

pub fn centre(j: &Jinx) -> V3 {
    v3(content::cm(j.x), 0, content::cm(j.z))
}

/// A point in a barrier's own frame: along its length, then across it.
fn local(at: V3, turn: u8, p: V3) -> (Fx, Fx) {
    let (c, s) = axis(turn);
    let d = p.sub(at);
    (
        fx::mul(d.x, c) + fx::mul(d.z, s),
        fx::mul(d.z, c) - fx::mul(d.x, s),
    )
}

fn world(at: V3, turn: u8, along: Fx, across: Fx) -> V3 {
    let (c, s) = axis(turn);
    at.add(v3(
        fx::mul(along, c) - fx::mul(across, s),
        0,
        fx::mul(along, s) + fx::mul(across, c),
    ))
}

/// A barrier's half length and half depth.
pub fn halves(c: &Content) -> (Fx, Fx) {
    let [l, d] = c.balance().barrier_cm;
    (content::cm(l) / 2, content::cm(d) / 2)
}

/// A barrier's four corners, for drawing.
pub fn corners(c: &Content, j: &Jinx) -> [V3; 4] {
    let (hl, hd) = halves(c);
    let at = centre(j);
    [
        world(at, j.turn, -hl, -hd),
        world(at, j.turn, hl, -hd),
        world(at, j.turn, hl, hd),
        world(at, j.turn, -hl, hd),
    ]
}

/// Where a jinx's name is shown: a barrier's or a bog's centre, the
/// middle of an iced walkway.
pub fn label_at(c: &Content, j: &Jinx) -> V3 {
    match shape(c, j.piece) {
        Some(JinxShape::Ice) => {
            let (a, b) = walkway_ends(j.way as usize);
            a.add(b.sub(a).ratio(1, 2))
        }
        _ => centre(j),
    }
}

/// Whether a jinx can go there, and for ice the walkway it lies on. A
/// barrier stays inside the room and clear of every station; a bog's
/// centre is inside the room; black ice needs a walkway near the click.
pub fn fits(c: &Content, shape: JinxShape, x: i32, z: i32, turn: u8) -> Result<u8, &'static str> {
    let at = v3(content::cm(x), 0, content::cm(z));
    match shape {
        JinxShape::Barrier => {
            if turn >= TURNS {
                return Err("jinx.bad");
            }
            let j = Jinx {
                piece: 0,
                from: 0,
                x,
                z,
                turn,
                way: 0,
                left: 0,
            };
            if corners(c, &j).iter().any(|p| p.flat_len() > ROOM_RADIUS - BODY) {
                return Err("jinx.outside");
            }
            let (hl, hd) = halves(c);
            for (s, _, _) in crate::prep::STATIONS {
                let (a, b) = local(at, turn, station_pos(s));
                let gap = v3((a.abs() - hl).max(0), 0, (b.abs() - hd).max(0)).flat_len();
                if gap < STATION_CLEAR {
                    return Err("jinx.on_station");
                }
            }
            Ok(0)
        }
        JinxShape::Bog => {
            if at.flat_len() > ROOM_RADIUS - BODY {
                return Err("jinx.outside");
            }
            Ok(0)
        }
        JinxShape::Ice => {
            let (k, d) = nearest_walkway(at);
            if d > ICE_PICK {
                return Err("jinx.no_walkway");
            }
            Ok(k as u8)
        }
    }
}

/// The pace at a point, in percent of walking: quicker on a walkway,
/// slower in a bog, the bog winning.
pub fn pace_pct(c: &Content, jinxes: &[Jinx], p: V3) -> i32 {
    let b = c.balance();
    let r = content::cm(b.bog_cm);
    let bog = jinxes
        .iter()
        .any(|j| shape(c, j.piece) == Some(JinxShape::Bog) && p.sub(centre(j)).flat_len() <= r);
    if bog {
        b.bog_pct
    } else if on_walkway(p) {
        b.walkway_pct
    } else {
        100
    }
}

/// Whether a point is on black ice.
pub fn icy(c: &Content, jinxes: &[Jinx], p: V3) -> bool {
    jinxes.iter().any(|j| {
        shape(c, j.piece) == Some(JinxShape::Ice) && from_walkway(p, j.way as usize) <= WALKWAY_HALF + BODY
    })
}

/// A body at `p` put back outside every barrier, by the shortest way.
pub fn push_out(c: &Content, jinxes: &[Jinx], mut p: V3) -> V3 {
    let (hl, hd) = halves(c);
    for j in jinxes
        .iter()
        .filter(|j| shape(c, j.piece) == Some(JinxShape::Barrier))
    {
        let at = centre(j);
        let (a, b) = local(at, j.turn, p);
        let (ea, eb) = (hl + BODY - a.abs(), hd + BODY - b.abs());
        if ea <= 0 || eb <= 0 {
            continue;
        }
        let side = |v: Fx| if v < 0 { -1 } else { 1 };
        p = if ea < eb {
            world(at, j.turn, side(a) * (hl + BODY), b)
        } else {
            world(at, j.turn, a, side(b) * (hd + BODY))
        };
    }
    p
}

/// Where a computer rival puts a jinx of this shape on walkway `way`: at
/// its middle, a barrier turned across it.
pub fn spot(shape: JinxShape, way: usize) -> (i32, i32, u8) {
    let (a, b) = walkway_ends(way);
    let mid = a.add(b.sub(a).ratio(1, 2));
    let d = b.sub(a).flat();
    // the turn whose axis is most nearly square to the walkway
    let turn = (0..TURNS)
        .min_by_key(|&t| {
            let (co, si) = axis(t);
            (fx::mul(d.x, co) + fx::mul(d.z, si)).abs()
        })
        .unwrap_or(0);
    let cm = |v: Fx| (v as i64 * 100 / ONE as i64) as i32;
    let turn = if shape == JinxShape::Barrier { turn } else { 0 };
    (cm(mid.x), cm(mid.z), turn)
}

/// Seconds a jinx takes from a computer rival's next day.
pub fn bot_loss_s(c: &Content, j: &Jinx) -> i32 {
    j.left / TPS * c.balance().bot_jinx_pct / 100
}

#[cfg(test)]
mod tests;

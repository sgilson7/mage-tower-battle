//! The tower's round room during preparation, drawn the way Ape Race draws
//! its maps (T34, journal entry 8): faceted stone and timber, windows that
//! let dusk light fall across the planks, full shelves, and each station
//! dressed so you can tell what happens there. Its furniture goes to the
//! static buffer once, in two parts: the room, and the near wall, which is
//! drawn only when the camera has come down inside the room (T35). The
//! people, flames, bubbles and floating things are rebuilt every frame.

use crate::content::Content;
use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, V3};
use crate::look::{Look, Pose};
use crate::mesh::Mesh;
use crate::prep::{self, Activity, Prep, Station, Work};
use crate::render::{mix, Batch, Camera, Frame3, Mood, Pass, Rgb};
use crate::rng::Rng;

const fn cm(v: i32) -> Fx {
    v * ONE / 100
}
fn sz(x: Fx, y: Fx, z: Fx) -> Frame3 {
    Frame3::upright(v3(x, y, z))
}
/// An angle in thousandths of a turn.
const fn per_mille(a: i32) -> i32 {
    (fx::TURN as i64 * a as i64 / 1000) as i32
}
/// A point at angle `a` round the room's centre, `r` out: angle 0 is
/// toward the camera (+z), a quarter turn is east (+x).
fn polar(a: i32, r: Fx, y: Fx) -> V3 {
    v3(fx::mul(fx::sin(a), r), y, fx::mul(fx::cos(a), r))
}
/// A shade of a colour: lighter or darker by up to `amt`, a little warmer
/// or cooler, so repeated parts are not copies.
fn vary(c: Rgb, r: &mut Rng, amt: Fx) -> Rgb {
    let l = r.between(-amt, amt);
    let h = r.between(-amt / 3, amt / 3);
    [
        (c[0] + l + h).clamp(0, ONE),
        (c[1] + l).clamp(0, ONE),
        (c[2] + l - h).clamp(0, ONE),
    ]
}
fn place(at: V3, yaw: i32, off: V3) -> V3 {
    at.add(crate::render::rotate(off, yaw, 0, 0))
}

/// The wall: rows of stone blocks round the room.
pub const WALL_R: Fx = cm(1520);
const ROW_H: Fx = cm(56);
const WALL_ROWS: i32 = 18;
const WALL_BLOCKS: i32 = 84;
pub const WALL_TOP: Fx = ROW_H * WALL_ROWS;
/// The windows, in thousandths of a turn: two flanking the invasion book
/// at the back, one east and one west.
const WINDOWS: [i32; 4] = [433, 567, 262, 738];
const WINDOW_HALF: Fx = cm(105);
const WINDOW_LOW: Fx = cm(320);
const WINDOW_HIGH: Fx = cm(820);
/// The library's three cases, either side of its station's bearing.
const CASES: [i32; 3] = [612, 659, 706];
/// Timber posts on the full-height wall, between windows and cases.
const POSTS: [i32; 5] = [300, 382, 467, 533, 588];
/// The laboratory's wall of jars.
const JARS: i32 = 341;

/// How many rows of the wall stand at angle `a`: all of them at the back,
/// stepping down toward the camera to two, so the room is seen into from
/// above (a cut-away, as in the mockup).
pub fn wall_rows(a: i32) -> i32 {
    let c = fx::cos(a);
    if c < -ONE / 20 {
        return WALL_ROWS;
    }
    let keep = WALL_ROWS - (WALL_ROWS * (c + ONE / 20) / (ONE * 55 / 100));
    keep.clamp(2, WALL_ROWS)
}

/// Whether the near wall is drawn: only once the camera is inside the room
/// and below the wall's top, as it is beside a station (T35).
pub fn near_wall_shown(cam: &Camera) -> bool {
    cam.eye.y < WALL_TOP && cam.eye.flat_len() < WALL_R
}

/// Metres between angles `a` and `b` along the wall.
fn along_wall(a: i32, b: i32) -> Fx {
    let d = (a - b).rem_euclid(fx::TURN);
    let d = d.min(fx::TURN - d);
    // a turn is 2π radii long
    ((d as i64 * WALL_R as i64 * 6283) / (fx::TURN as i64 * 1000)) as Fx
}

fn in_window(a: i32, y: Fx) -> bool {
    WINDOWS.iter().any(|&w| {
        let d = along_wall(a, per_mille(w));
        if d > WINDOW_HALF {
            return false;
        }
        // an arched top: highest at the middle
        let k = ONE - fx::mul(fx::div(d, WINDOW_HALF), fx::div(d, WINDOW_HALF));
        let top = WINDOW_HIGH + fx::mul(cm(100), fx::sqrt(k.max(0)));
        y > WINDOW_LOW && y < top
    })
}

/// The room at dusk: violet stone, the planks' warm bounce, a low sun
/// through the back windows, haze toward the far wall.
pub fn room_mood(look: &Look) -> Mood {
    Mood {
        fog: look.colour("fog_room"),
        fog_near: 24 * ONE,
        fog_far: 56 * ONE,
        sky: look.colour("light_sky_room"),
        ground: look.colour("light_ground_room"),
        sun: look.colour("sun_room"),
        sun_dir: v3(-ONE * 3 / 10, ONE * 6 / 10, -ONE * 3 / 4),
    }
}

/// A table `w` by `d`, its top and legs in the two colour roles of `wood`.
fn table(b: &mut Batch, look: &Look, at: V3, yaw: i32, (w, d): (Fx, Fx), (top, legs): (&str, &str)) {
    b.solid(
        Mesh::Chunk,
        at.add(v3(0, cm(100), 0)),
        &Frame3::new(yaw, 0, 0, v3(w, cm(14), d)),
        look.colour(top),
    );
    b.solid(
        Mesh::Cube,
        at.add(v3(0, cm(86), 0)),
        &Frame3::new(yaw, 0, 0, v3(w - cm(24), cm(14), d - cm(24))),
        look.colour(legs),
    );
    for (sx, sz_) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let off = v3(sx * (w / 2 - cm(18)), cm(47), sz_ * (d / 2 - cm(18)));
        b.solid(
            Mesh::Cube,
            place(at, yaw, off),
            &Frame3::new(yaw, 0, 0, v3(cm(15), cm(94), cm(15))),
            look.colour(legs),
        );
    }
    shadow(b, look, at, yaw, w * 6 / 10, d * 7 / 10);
}

/// A soft dark disc under a thing, in place of a computed shadow.
fn shadow(b: &mut Batch, look: &Look, at: V3, yaw: i32, rx: Fx, rz: Fx) {
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        v3(at.x, cm(1), at.z),
        &Frame3::new(yaw, 0, 0, v3(rx * 2, cm(1), rz * 2)),
        look.colour("shadow"),
        0,
    );
}

/// The podium's dais, and the way from it toward the room's middle that
/// its benches follow.
pub fn dais() -> (V3, V3) {
    let d = prep::station_pos(Station::Podium).add(v3(cm(140), 0, cm(120)));
    (d, v3(-d.x, 0, -d.z).with_len(ONE))
}

/// Where the students sit and stand to listen: `i` from 0; the first eight
/// on two benches, the rest standing in rows behind. With the yaw that
/// faces the dais.
pub fn listener(i: usize) -> (V3, i32, bool) {
    let (d, u) = dais();
    let side = v3(u.z, 0, -u.x);
    let face = crate::geom::yaw_of(-u.x, -u.z);
    if i < 8 {
        let (row, col) = ((i / 4) as i32, (i % 4) as i32);
        let at = d
            .add(u.scale(cm(330) + row * cm(170)))
            .add(side.scale(cm(-165) + col * cm(110)));
        (at, face, true)
    } else {
        let k = (i - 8) as i32;
        let (row, col) = (k / 6, k % 6);
        let at = d
            .add(u.scale(cm(700) + row * cm(130)))
            .add(side.scale(cm(-300) + col * cm(120)));
        (at, face, false)
    }
}

/// Which way the applicants queue from the recruitment table.
fn queue_dir() -> V3 {
    v3(-ONE * 35 / 100, 0, ONE * 94 / 100)
}

/// The room's furniture: (the room, the near wall).
pub fn room_static(look: &Look) -> (Batch, Batch) {
    let mut b = Batch::new();
    let mut near = Batch::new();
    let mut r = Rng::new(43);
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        v3(0, -cm(30), 0),
        &sz(32 * ONE, cm(40), 32 * ONE),
        look.colour("shelf_back"),
        0,
    );
    // planks running toward the camera, each board its own shade
    let bw = cm(62);
    let mut x = -WALL_R;
    while x < WALL_R {
        let cx = x + bw / 2;
        let half = fx::sqrt(fx::mul(WALL_R, WALL_R) - fx::mul(cx, cx)) + cm(40);
        let mut z = -half - r.between(0, cm(80));
        while z < half {
            let len = r.between(cm(170), cm(380)).min(half - z);
            if len > cm(20) {
                b.push(
                    Mesh::Cube,
                    Pass::Solid,
                    v3(cx, -cm(6) + r.between(-cm(1), cm(1)), z + len / 2),
                    &sz(bw - cm(4), cm(12), len - cm(4)),
                    vary(look.colour("floor"), &mut r, ONE / 16),
                    0,
                );
            }
            z += len;
        }
        x += bw;
    }
    // the rug under the map table
    let map = prep::station_pos(Station::Map);
    let rug = map.add(v3(0, 0, cm(140)));
    for (i, &(rad, role)) in [
        (490, "rug_dark"),
        (462, "rug_trim"),
        (435, "rug"),
        (310, "rug_light"),
        (290, "rug_trim"),
        (270, "rug_dark"),
    ]
    .iter()
    .enumerate()
    {
        b.push(
            Mesh::Cyl,
            Pass::Solid,
            rug.add(v3(0, cm(1) + i as i32 * cm(1) / 2, 0)),
            &sz(cm(rad * 2), cm(3), cm(rad * 2)),
            look.colour(role),
            0,
        );
    }
    for k in 0..8 {
        let a = k * fx::TURN / 8;
        b.push(
            Mesh::Cube,
            Pass::Solid,
            rug.add(polar(a, cm(372), cm(5))),
            &Frame3::new(a + fx::TURN / 8, 0, 0, v3(cm(42), cm(2), cm(42))),
            look.colour("gold"),
            0,
        );
    }
    // the wall, block by block
    for row in 0..WALL_ROWS {
        let y = row * ROW_H + ROW_H / 2;
        for i in 0..WALL_BLOCKS {
            let a = (i * 2 + row % 2) * fx::TURN / (WALL_BLOCKS * 2);
            if in_window(a, y) {
                continue;
            }
            let w = ((WALL_R as i64 * 6283 / 1000) / WALL_BLOCKS as i64) as Fx * r.between(90, 97) / 100;
            let dep = r.between(cm(70), cm(90));
            let rad = WALL_R - cm(35) + dep / 2 + r.between(-cm(2), cm(2));
            let mut c = vary(
                look.colour(if r.below(8) == 0 { "wall_light" } else { "wall" }),
                &mut r,
                ONE / 22,
            );
            if row < 2 {
                c = mix(c, look.colour("ink"), ONE / 6);
            }
            let to = if row < wall_rows(a) { &mut b } else { &mut near };
            // half the blocks turned end for end, so their nudged corners differ
            let flip = if r.below(2) == 0 { fx::HALF_TURN } else { 0 };
            to.push(
                Mesh::Chunk,
                Pass::Solid,
                polar(a, rad, y + r.between(-cm(2), cm(2))),
                &Frame3::new(a + flip, 0, 0, v3(w, ROW_H * r.between(90, 95) / 100, dep)),
                c,
                0,
            );
        }
    }
    // timber posts and the ring beam over the back wall
    for (k, &p) in POSTS.iter().enumerate() {
        let a = per_mille(p);
        let wood = vary(look.colour("wood_dark"), &mut r, ONE / 30);
        b.push(
            Mesh::Chunk,
            Pass::Solid,
            polar(a, WALL_R - cm(45), WALL_TOP / 2),
            &Frame3::new(a, 0, 0, v3(cm(55), WALL_TOP, cm(44))),
            wood,
            0,
        );
        b.push(
            Mesh::Cube,
            Pass::Solid,
            polar(a, WALL_R - cm(45), cm(28)),
            &Frame3::new(a, 0, 0, v3(cm(76), cm(56), cm(60))),
            look.colour("wood_darker"),
            0,
        );
        if k % 2 == 1 {
            // a candle in a sconce; its flame is drawn each frame
            b.push(
                Mesh::Cube,
                Pass::Solid,
                polar(a, WALL_R - cm(80), cm(400)),
                &Frame3::new(a, 0, 0, v3(cm(14), cm(14), cm(50))),
                look.colour("iron"),
                0,
            );
            b.push(
                Mesh::Cyl,
                Pass::Solid,
                polar(a, WALL_R - cm(102), cm(428)),
                &sz(cm(11), cm(34), cm(11)),
                look.colour("wax"),
                0,
            );
        }
    }
    for i in 0..26 {
        let a = per_mille(290) + i * per_mille(420) / 26;
        let mid = a + per_mille(420) / 52;
        let len = fx::mul(WALL_R, per_mille(420) / 26 * 6283 / 1000) + cm(10);
        b.push(
            Mesh::Chunk,
            Pass::Solid,
            polar(mid, WALL_R - cm(45), WALL_TOP - cm(60)),
            &Frame3::new(mid, 0, 0, v3(len, cm(42), cm(46))),
            look.colour("wood_dark"),
            0,
        );
    }
    // windows: a dusk sky, mullions, a sill, and a shaft of light that
    // lands on the planks in a warm pool
    for &w in WINDOWS.iter() {
        let a = per_mille(w);
        let pane = polar(a, WALL_R + cm(8), (WINDOW_LOW + WINDOW_HIGH) / 2 + cm(30));
        b.push(
            Mesh::Cube,
            Pass::Solid,
            pane,
            &Frame3::new(
                a,
                0,
                0,
                v3(
                    WINDOW_HALF * 2 + cm(30),
                    WINDOW_HIGH - WINDOW_LOW + cm(160),
                    cm(10),
                ),
            ),
            look.colour("window_sky"),
            ONE,
        );
        b.push(
            Mesh::Cube,
            Pass::Solid,
            polar(a, WALL_R - cm(20), (WINDOW_LOW + WINDOW_HIGH) / 2 + cm(40)),
            &Frame3::new(a, 0, 0, v3(cm(12), WINDOW_HIGH - WINDOW_LOW + cm(80), cm(14))),
            look.colour("wood_darker"),
            0,
        );
        for y in [cm(470), cm(650)] {
            b.push(
                Mesh::Cube,
                Pass::Solid,
                polar(a, WALL_R - cm(20), y),
                &Frame3::new(a, 0, 0, v3(WINDOW_HALF * 2, cm(10), cm(14))),
                look.colour("wood_darker"),
                0,
            );
        }
        b.push(
            Mesh::Chunk,
            Pass::Solid,
            polar(a, WALL_R - cm(40), WINDOW_LOW - cm(10)),
            &Frame3::new(a, 0, 0, v3(WINDOW_HALF * 2 + cm(60), cm(22), cm(80))),
            look.colour("wall_light"),
            0,
        );
        let from = polar(a, WALL_R, cm(560));
        let to = polar(a, WALL_R * 4 / 10, 0).add(v3(0, 0, cm(140)));
        let mid = from.lerp(to, ONE / 2);
        let d = to.sub(from);
        let yaw = crate::geom::yaw_of(d.x, d.z);
        let pitch = crate::geom::yaw_of(d.flat_len(), -d.y) - fx::QUARTER_TURN;
        let len = d.len();
        for (wid, thick) in [(cm(170), cm(4)), (cm(4), cm(260))] {
            b.push(
                Mesh::Cube,
                Pass::Glow,
                mid,
                &Frame3::new(yaw, pitch, 0, v3(wid, thick, len)),
                look.colour("window_sky"),
                ONE / 10,
            );
        }
        b.push(
            Mesh::Ground,
            Pass::Glow,
            v3(to.x, cm(2), to.z),
            &Frame3::new(yaw, 0, 0, v3(cm(220), cm(1), cm(460))),
            look.colour("window_sky"),
            ONE / 5,
        );
    }
    // a chandelier over the map table
    let ch = map.add(v3(0, cm(880), cm(100)));
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        ch,
        &sz(cm(280), cm(12), cm(280)),
        look.colour("iron"),
        0,
    );
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        ch.add(v3(0, cm(150), 0)),
        &sz(cm(5), cm(300), cm(5)),
        look.colour("iron"),
        0,
    );
    for k in 0..8 {
        b.push(
            Mesh::Cyl,
            Pass::Solid,
            ch.add(polar(k * fx::TURN / 8, cm(140), cm(18))),
            &sz(cm(10), cm(28), cm(10)),
            look.colour("wax"),
            0,
        );
    }
    library(&mut b, look, &mut r);
    invasion(&mut b, look);
    lab(&mut b, look, &mut r);
    map_table(&mut b, look, &mut r);
    podium(&mut b, look);
    recruit(&mut b, look);
    office(&mut b, look, &mut r);
    (b, near)
}

const BOOKS: [&str; 7] = [
    "book_red",
    "book_blue",
    "book_green",
    "book_purple",
    "book_gold",
    "book_pink",
    "book_grey",
];

fn library(b: &mut Batch, look: &Look, r: &mut Rng) {
    for &c in CASES.iter() {
        let a = per_mille(c);
        let at = polar(a, WALL_R - cm(95), 0);
        let yaw = a + fx::HALF_TURN;
        let (w, h, d) = (cm(280), cm(470), cm(66));
        for sx in [-1, 1] {
            b.solid(
                Mesh::Cube,
                place(at, yaw, v3(sx * w / 2, h / 2, 0)),
                &Frame3::new(yaw, 0, 0, v3(cm(16), h, d)),
                look.colour("wood_dark"),
            );
        }
        b.solid(
            Mesh::Chunk,
            place(at, yaw, v3(0, h + cm(8), 0)),
            &Frame3::new(yaw, 0, 0, v3(w + cm(34), cm(22), d + cm(14))),
            look.colour("wood_darker"),
        );
        b.solid(
            Mesh::Cube,
            place(at, yaw, v3(0, h / 2, -d / 2 + cm(3))),
            &Frame3::new(yaw, 0, 0, v3(w, h, cm(6))),
            look.colour("shelf_back"),
        );
        for shelf in 0..5 {
            let y = cm(20) + shelf * cm(90);
            b.solid(
                Mesh::Cube,
                place(at, yaw, v3(0, y, 0)),
                &Frame3::new(yaw, 0, 0, v3(w, cm(8), d)),
                look.colour("shelf"),
            );
            let mut x = -w / 2 + cm(12);
            while x < w / 2 - cm(16) {
                if r.below(16) == 0 {
                    x += r.between(cm(16), cm(36));
                    continue;
                }
                let bwid = r.between(cm(7), cm(16));
                let bh = r.between(cm(46), cm(74));
                let lean = if r.below(12) == 0 {
                    r.between(per_mille(25), per_mille(50))
                } else {
                    0
                };
                let colour = vary(look.colour(BOOKS[r.below(7) as usize]), r, ONE / 14);
                b.solid(
                    Mesh::Cube,
                    place(at, yaw, v3(x + bwid / 2, y + cm(4) + bh / 2, cm(2))),
                    &Frame3::new(yaw, 0, -lean, v3(bwid, bh, r.between(cm(40), cm(54)))),
                    colour,
                );
                x += bwid + cm(1) + lean / 3;
            }
        }
        shadow(b, look, at, yaw, w * 6 / 10, d);
    }
    // a ladder against the middle case
    let a = per_mille(CASES[1]) + per_mille(8);
    let at = polar(a, WALL_R - cm(165), 0);
    let yaw = a + fx::HALF_TURN;
    let lean = -per_mille(35);
    for sx in [-1, 1] {
        b.solid(
            Mesh::Cube,
            place(at, yaw, v3(sx * cm(38), cm(240), cm(-40))),
            &Frame3::new(yaw, lean, 0, v3(cm(9), cm(490), cm(9))),
            look.colour("wood_light"),
        );
    }
    for k in 0..10 {
        let y = cm(40) + k * cm(46);
        b.solid(
            Mesh::Cube,
            place(at, yaw, v3(0, y, cm(-40) + (y - cm(240)) * 22 / 100)),
            &Frame3::new(yaw, 0, 0, v3(cm(76), cm(7), cm(8))),
            look.colour("wood_light"),
        );
    }
    // a standing candle by the shelves; its flames are drawn each frame
    let c = candle_stand();
    b.solid(
        Mesh::Cyl,
        c.add(v3(0, cm(5), 0)),
        &sz(cm(60), cm(10), cm(60)),
        look.colour("iron"),
    );
    b.solid(
        Mesh::Cyl,
        c.add(v3(0, cm(95), 0)),
        &sz(cm(9), cm(180), cm(9)),
        look.colour("iron"),
    );
    b.solid(
        Mesh::Cyl,
        c.add(v3(0, cm(186), 0)),
        &sz(cm(44), cm(6), cm(44)),
        look.colour("iron"),
    );
    for (dx, dz, h) in [(0, 0, 30), (12, 6, 24), (-11, 7, 20)] {
        b.solid(
            Mesh::Cyl,
            c.add(v3(cm(dx), cm(190) + cm(h) / 2, cm(dz))),
            &sz(cm(10), cm(h), cm(10)),
            look.colour("wax"),
        );
    }
    shadow(b, look, c, 0, cm(40), cm(40));
}

fn candle_stand() -> V3 {
    prep::station_pos(Station::Library).add(v3(cm(260), 0, cm(-150)))
}

fn invasion(b: &mut Batch, look: &Look) {
    let at = prep::station_pos(Station::Book);
    b.solid(
        Mesh::Chunk,
        at.add(v3(0, cm(18), 0)),
        &sz(cm(220), cm(36), cm(220)),
        look.colour("stone"),
    );
    b.solid(
        Mesh::Frustum,
        at.add(v3(0, cm(100), 0)),
        &Frame3::new(0, fx::HALF_TURN, 0, v3(cm(110), cm(140), cm(110))),
        look.colour("stone_light"),
    );
    let top = at.add(v3(0, cm(180), cm(5)));
    let tilt = per_mille(55);
    b.solid(
        Mesh::Chunk,
        top,
        &Frame3::new(0, tilt, 0, v3(cm(140), cm(14), cm(100))),
        look.colour("stone"),
    );
    for sx in [-1, 1] {
        b.solid(
            Mesh::Cube,
            top.add(v3(sx * cm(34), cm(12), 0)),
            &Frame3::new(0, tilt, sx * per_mille(16), v3(cm(68), cm(7), cm(90))),
            look.colour("invasion_book"),
        );
        b.solid(
            Mesh::Cube,
            top.add(v3(sx * cm(32), cm(17), 0)),
            &Frame3::new(0, tilt, sx * per_mille(16), v3(cm(62), cm(5), cm(82))),
            look.colour("page"),
        );
    }
    shadow(b, look, at, 0, cm(130), cm(130));
    // two red banners on the wall behind
    for a in [475, 525] {
        let a = per_mille(a);
        let p = polar(a, WALL_R - cm(60), 0);
        b.solid(
            Mesh::Cube,
            p.add(v3(0, cm(880), 0)),
            &Frame3::new(a, 0, 0, v3(cm(190), cm(10), cm(10))),
            look.colour("wood_darker"),
        );
        b.solid(
            Mesh::Chunk,
            p.add(v3(0, cm(670), 0)),
            &Frame3::new(a, 0, 0, v3(cm(160), cm(420), cm(6))),
            look.colour("banner"),
        );
        b.solid(
            Mesh::Cone,
            p.add(v3(0, cm(430), 0)),
            &Frame3::new(a, fx::HALF_TURN, 0, v3(cm(160), cm(70), cm(6))),
            look.colour("banner"),
        );
        b.solid(
            Mesh::Gem,
            polar(a, WALL_R - cm(66), cm(700)),
            &Frame3::new(a, 0, 0, v3(cm(60), cm(80), cm(8))),
            look.colour("gold"),
        );
    }
}

fn cauldron() -> V3 {
    prep::station_pos(Station::Lab).add(v3(cm(240), 0, cm(250)))
}

fn lab(b: &mut Batch, look: &Look, r: &mut Rng) {
    let at = prep::station_pos(Station::Lab);
    let yaw = -per_mille(70);
    table(
        b,
        look,
        at,
        yaw,
        (cm(380), cm(150)),
        ("lab_table", "lab_table_dark"),
    );
    for k in 0..3 {
        let p = place(at, yaw, v3(cm(-115) + k * cm(115), cm(112), 0));
        b.solid(Mesh::Cyl, p, &sz(cm(30), cm(10), cm(30)), look.colour("iron"));
        b.push(
            Mesh::Cyl,
            Pass::Glow,
            p.add(v3(0, cm(190), 0)),
            &sz(cm(6), cm(320), cm(6)),
            look.colour(CRYSTALS[k as usize]),
            ONE / 3,
        );
    }
    // glassware
    b.push(
        Mesh::Sphere,
        Pass::Solid,
        place(at, yaw, v3(cm(-160), cm(135), cm(42))),
        &sz(cm(44), cm(44), cm(44)),
        look.colour("glass"),
        ONE / 4,
    );
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        place(at, yaw, v3(cm(-160), cm(170), cm(42))),
        &sz(cm(8), cm(40), cm(8)),
        look.colour("glass"),
        ONE / 4,
    );
    b.push(
        Mesh::Cone,
        Pass::Solid,
        place(at, yaw, v3(cm(165), cm(125), cm(45))),
        &sz(cm(30), cm(40), cm(30)),
        look.colour("charm"),
        ONE / 4,
    );
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        place(at, yaw, v3(cm(130), cm(125), cm(-45))),
        &sz(cm(20), cm(38), cm(20)),
        look.colour("type_acid"),
        ONE / 4,
    );
    // the cauldron on its legs; fire, brew and bubbles are drawn each frame
    let k = cauldron();
    b.solid(
        Mesh::Sphere,
        k.add(v3(0, cm(110), 0)),
        &sz(cm(210), cm(160), cm(210)),
        look.colour("cauldron"),
    );
    b.solid(
        Mesh::Cyl,
        k.add(v3(0, cm(182), 0)),
        &sz(cm(196), cm(14), cm(196)),
        look.colour("iron"),
    );
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        k.add(v3(0, cm(186), 0)),
        &sz(cm(176), cm(6), cm(176)),
        look.colour("brew"),
        ONE,
    );
    for i in 0..3 {
        let a = i * fx::TURN / 3;
        b.solid(
            Mesh::Cube,
            k.add(polar(a, cm(70), cm(26))),
            &Frame3::new(a, 0, 0, v3(cm(14), cm(56), cm(14))),
            look.colour("iron"),
        );
    }
    shadow(b, look, k, 0, cm(120), cm(120));
    // the wall of jars behind
    let a = per_mille(JARS);
    let wall = polar(a, WALL_R - cm(80), 0);
    let ty = a + fx::HALF_TURN;
    for y in [cm(260), cm(370), cm(480)] {
        b.solid(
            Mesh::Cube,
            place(wall, ty, v3(0, y, 0)),
            &Frame3::new(ty, 0, 0, v3(cm(290), cm(9), cm(56))),
            look.colour("wood_dark"),
        );
        let mut x = cm(-130);
        while x < cm(130) {
            let h = r.between(cm(26), cm(50));
            let role = JAR_COLOURS[r.below(JAR_COLOURS.len() as u32) as usize];
            b.push(
                Mesh::Cyl,
                Pass::Solid,
                place(wall, ty, v3(x, y + cm(5) + h / 2, 0)),
                &Frame3::new(ty, 0, 0, v3(r.between(cm(16), cm(26)), h, cm(22))),
                look.colour(role),
                ONE / 3,
            );
            b.solid(
                Mesh::Cyl,
                place(wall, ty, v3(x, y + cm(8) + h, 0)),
                &Frame3::new(ty, 0, 0, v3(cm(13), cm(6), cm(13))),
                look.colour("wood"),
            );
            x += r.between(cm(26), cm(40));
        }
    }
}

const CRYSTALS: [&str; 3] = ["type_fire", "type_acid", "type_frost"];
const JAR_COLOURS: [&str; 6] = [
    "type_acid",
    "type_fire",
    "type_frost",
    "charm",
    "gold",
    "type_spirit",
];

fn map_table(b: &mut Batch, look: &Look, r: &mut Rng) {
    let at = prep::station_pos(Station::Map);
    table(b, look, at, 0, (cm(380), cm(260)), ("wood", "wood_dark"));
    b.solid(
        Mesh::Cube,
        at.add(v3(0, cm(110), 0)),
        &sz(cm(350), cm(8), cm(230)),
        look.colour("shelf_back"),
    );
    // the battle ground in miniature: raised tiles, a path, small trees
    let (nx, nz, cell) = (18, 12, cm(18));
    for i in 0..nx {
        for j in 0..nz {
            let bend = fx::sin(i * fx::TURN / 14) * 2 / ONE;
            let path = (j - nz / 2 - bend).abs() < 1;
            let h = if path { cm(2) } else { r.between(cm(3), cm(9)) };
            let p = at.add(v3(
                -(nx - 1) * cell / 2 + i * cell,
                cm(115) + h / 2,
                -(nz - 1) * cell / 2 + j * cell,
            ));
            let c = if path {
                vary(look.colour("path"), r, ONE / 30)
            } else {
                vary(look.colour("grass"), r, ONE / 18)
            };
            b.push(
                Mesh::Cube,
                Pass::Solid,
                p,
                &sz(cell * 96 / 100, h, cell * 96 / 100),
                c,
                0,
            );
        }
    }
    for _ in 0..10 {
        let x = r.between(cm(-150), cm(150));
        let z = if r.below(2) == 0 {
            r.between(cm(-100), cm(-55))
        } else {
            r.between(cm(55), cm(100))
        };
        b.solid(
            Mesh::Cone,
            at.add(v3(x, cm(135), z)),
            &sz(cm(12), cm(28), cm(12)),
            look.colour("tree_dark"),
        );
    }
    let t = at.add(v3(cm(110), cm(115), cm(-40)));
    b.solid(
        Mesh::Cyl,
        t.add(v3(0, cm(30), 0)),
        &sz(cm(26), cm(60), cm(26)),
        look.colour("tower_stone"),
    );
    b.solid(
        Mesh::Cone,
        t.add(v3(0, cm(76), 0)),
        &sz(cm(34), cm(34), cm(34)),
        look.colour("tower_roof"),
    );
    // an enchanted rim of blue light
    for (w, d, x, z) in [
        (cm(366), cm(5), 0, cm(-118)),
        (cm(366), cm(5), 0, cm(118)),
        (cm(5), cm(240), cm(-180), 0),
        (cm(5), cm(240), cm(180), 0),
    ] {
        b.push(
            Mesh::Cube,
            Pass::Solid,
            at.add(v3(x, cm(116), z)),
            &sz(w, cm(5), d),
            look.colour("map_edge"),
            ONE,
        );
    }
    b.push(
        Mesh::Cube,
        Pass::Glow,
        at.add(v3(0, cm(140), 0)),
        &sz(cm(380), cm(50), cm(250)),
        look.colour("map_edge"),
        ONE / 16,
    );
}

fn podium(b: &mut Batch, look: &Look) {
    let (d, u) = dais();
    let side = v3(u.z, 0, -u.x);
    let face_students = crate::geom::yaw_of(u.x, u.z);
    let face_dais = crate::geom::yaw_of(-u.x, -u.z);
    b.solid(
        Mesh::Chunk,
        d.add(v3(0, cm(13), 0)),
        &Frame3::new(face_students, 0, 0, v3(cm(380), cm(26), cm(380))),
        look.colour("wood_dark"),
    );
    b.solid(
        Mesh::Cyl,
        d.add(v3(0, cm(38), 0)),
        &sz(cm(280), cm(24), cm(280)),
        look.colour("wood"),
    );
    shadow(b, look, d, 0, cm(220), cm(220));
    let l = d.add(u.scale(cm(80)));
    b.solid(
        Mesh::Frustum,
        l.add(v3(0, cm(110), 0)),
        &Frame3::new(face_students, fx::HALF_TURN, 0, v3(cm(60), cm(130), cm(50))),
        look.colour("wood_dark"),
    );
    b.solid(
        Mesh::Chunk,
        l.add(v3(0, cm(182), 0)),
        &Frame3::new(face_students, -per_mille(55), 0, v3(cm(100), cm(10), cm(70))),
        look.colour("wood"),
    );
    b.solid(
        Mesh::Cube,
        l.add(v3(0, cm(189), 0)),
        &Frame3::new(face_students, -per_mille(55), 0, v3(cm(64), cm(4), cm(50))),
        look.colour("page"),
    );
    // the chalkboard behind, with a rune that glows
    let cb = d.sub(u.scale(cm(160))).add(side.scale(cm(190)));
    for sx in [-1, 1] {
        b.solid(
            Mesh::Cube,
            place(cb, face_students, v3(sx * cm(115), cm(165), cm(-10))),
            &Frame3::new(face_students, per_mille(12), 0, v3(cm(10), cm(330), cm(10))),
            look.colour("wood_dark"),
        );
    }
    b.solid(
        Mesh::Cube,
        place(cb, face_students, v3(0, cm(240), 0)),
        &Frame3::new(face_students, 0, 0, v3(cm(250), cm(160), cm(8))),
        look.colour("chalkboard"),
    );
    b.solid(
        Mesh::Cube,
        place(cb, face_students, v3(0, cm(158), cm(5))),
        &Frame3::new(face_students, 0, 0, v3(cm(266), cm(10), cm(18))),
        look.colour("wood_dark"),
    );
    for (x, y, w, h) in [
        (0, 50, 120, 5),
        (0, -18, 120, 5),
        (-40, 16, 5, 70),
        (40, 16, 5, 70),
        (0, 16, 70, 5),
        (0, 16, 5, 40),
    ] {
        b.push(
            Mesh::Cube,
            Pass::Solid,
            place(cb, face_students, v3(cm(x), cm(240 + y), cm(6))),
            &Frame3::new(face_students, 0, 0, v3(cm(w), cm(h), cm(2))),
            look.colour("rune"),
            ONE,
        );
    }
    b.push(
        Mesh::Cube,
        Pass::Glow,
        place(cb, face_students, v3(0, cm(240), cm(20))),
        &Frame3::new(face_students, 0, 0, v3(cm(250), cm(160), cm(30))),
        look.colour("rune"),
        ONE / 10,
    );
    // two benches facing the dais
    for row in 0..2 {
        let p = d.add(u.scale(cm(330) + row * cm(170)));
        b.solid(
            Mesh::Chunk,
            p.add(v3(0, cm(58), 0)),
            &Frame3::new(face_dais, 0, 0, v3(cm(470), cm(12), cm(56))),
            look.colour("wood"),
        );
        for sx in [-1, 1] {
            b.solid(
                Mesh::Cube,
                place(p, face_dais, v3(sx * cm(200), cm(28), 0)),
                &Frame3::new(face_dais, 0, 0, v3(cm(12), cm(56), cm(50))),
                look.colour("wood_dark"),
            );
        }
        shadow(b, look, p, face_dais, cm(250), cm(40));
    }
}

fn recruit(b: &mut Batch, look: &Look) {
    let at = prep::station_pos(Station::Recruit);
    let q = queue_dir();
    let yaw = crate::geom::yaw_of(q.x, q.z);
    table(b, look, at, yaw, (cm(290), cm(130)), ("wood_light", "wood"));
    for (i, (x, z, t)) in [
        (-70, -10, 20),
        (-50, 15, -40),
        (-90, 5, 60),
        (-30, -20, 0),
        (-60, 0, -15),
    ]
    .iter()
    .enumerate()
    {
        b.solid(
            Mesh::Cube,
            place(at, yaw, v3(cm(*x), cm(108) + i as i32 * cm(1) / 2, cm(*z))),
            &Frame3::new(yaw + per_mille(*t), 0, 0, v3(cm(42), cm(1), cm(56))),
            look.colour("page"),
        );
    }
    b.solid(
        Mesh::Cyl,
        place(at, yaw, v3(cm(55), cm(114), cm(-25))),
        &sz(cm(18), cm(14), cm(18)),
        look.colour("eye"),
    );
    b.solid(
        Mesh::Cone,
        place(at, yaw, v3(cm(58), cm(140), cm(-25))),
        &Frame3::new(yaw, 0, -per_mille(60), v3(cm(8), cm(56), cm(4))),
        look.colour("white"),
    );
    b.solid(
        Mesh::Frustum,
        place(at, yaw, v3(cm(105), cm(114), cm(25))),
        &sz(cm(30), cm(16), cm(30)),
        look.colour("gold"),
    );
    // brass posts and a red rope along the queue
    let side = v3(q.z, 0, -q.x);
    let posts: Vec<V3> = (0..4)
        .map(|k| at.add(q.scale(cm(170) + k * cm(130))).add(side.scale(cm(95))))
        .collect();
    for (k, p) in posts.iter().enumerate() {
        b.solid(
            Mesh::Cyl,
            p.add(v3(0, cm(5), 0)),
            &sz(cm(44), cm(10), cm(44)),
            look.colour("gold"),
        );
        b.solid(
            Mesh::Cyl,
            p.add(v3(0, cm(62), 0)),
            &sz(cm(9), cm(115), cm(9)),
            look.colour("gold"),
        );
        b.solid(
            Mesh::Gem,
            p.add(v3(0, cm(124), 0)),
            &sz(cm(16), cm(16), cm(16)),
            look.colour("gold"),
        );
        if let Some(n) = posts.get(k + 1) {
            for (s, e, hs, he) in [
                (ONE / 10, ONE / 2, cm(115), cm(92)),
                (ONE / 2, ONE * 9 / 10, cm(92), cm(115)),
            ] {
                let (pa, pb) = (p.lerp(*n, s).add(v3(0, hs, 0)), p.lerp(*n, e).add(v3(0, he, 0)));
                let d = pb.sub(pa);
                let rope_yaw = crate::geom::yaw_of(d.x, d.z);
                let pitch = crate::geom::yaw_of(d.flat_len(), -d.y) - fx::QUARTER_TURN;
                b.solid(
                    Mesh::Cube,
                    pa.lerp(pb, ONE / 2),
                    &Frame3::new(rope_yaw, pitch, 0, v3(cm(5), cm(5), d.len())),
                    look.colour("banner"),
                );
            }
        }
    }
}

fn office(b: &mut Batch, look: &Look, r: &mut Rng) {
    let at = prep::station_pos(Station::Office);
    table(b, look, at, 0, (cm(300), cm(140)), ("wood", "wood_dark"));
    // the seating chart: a board with a peg for each tower seat
    b.solid(
        Mesh::Cube,
        at.add(v3(0, cm(110), cm(-10))),
        &sz(cm(150), cm(4), cm(90)),
        look.colour("map_green"),
    );
    for k in 0..12 {
        let p = at
            .add(v3(0, cm(115), cm(-10)))
            .add(polar(k * fx::TURN / 12, cm(32), 0));
        b.solid(Mesh::Cyl, p, &sz(cm(10), cm(10), cm(10)), look.colour("gold"));
    }
    // scrolls in a rack beside it
    let rack = at.add(v3(cm(220), 0, cm(-20)));
    b.solid(
        Mesh::Cube,
        rack.add(v3(0, cm(90), 0)),
        &sz(cm(80), cm(180), cm(50)),
        look.colour("wood_dark"),
    );
    for k in 0..9 {
        let p = rack.add(v3(cm(-26) + (k % 3) * cm(26), cm(40) + (k / 3) * cm(56), cm(20)));
        b.solid(
            Mesh::Cyl,
            p,
            &Frame3::new(0, fx::QUARTER_TURN, 0, v3(cm(18), cm(46), cm(18))),
            vary(look.colour("page"), r, ONE / 20),
        );
    }
    shadow(b, look, rack, 0, cm(50), cm(35));
}

/// Where the floor's piles of books stand: fifteen stacks of six, the
/// same each match.
/// A stack: where it stands, its turn, and each book's turn, thickness and colour.
type Stack = (V3, i32, Vec<(i32, Fx, usize)>);

fn pile_spots() -> Vec<Stack> {
    let mut r = Rng::new(7);
    let lib = prep::station_pos(Station::Library);
    (0..15)
        .map(|_| {
            let a = r.between(0, fx::TURN - 1);
            let d = r.between(cm(160), cm(420));
            let at = lib.add(v3(
                fx::mul(fx::sin(a), d),
                0,
                fx::mul(fx::cos(a), d) * 3 / 4 + cm(120),
            ));
            let books = (0..6)
                .map(|_| {
                    (
                        r.between(-per_mille(70), per_mille(70)),
                        r.between(cm(8), cm(13)),
                        r.below(7) as usize,
                    )
                })
                .collect();
            (at, r.between(0, fx::TURN - 1), books)
        })
        .collect()
}

/// How many books lie on the floor for a pile of `left`: ninety at the
/// start's forty, fewer as it empties, none once it is empty.
pub fn floor_books(left: usize) -> usize {
    (left * 90 / 40).min(90)
}

/// How near the camera has come to a station `d` away, 0 (the overview)
/// to ONE (down beside the archmage). It starts to come down four and a
/// half metres away and is all the way down at a little over the three
/// metres a station reaches (T35).
fn near(d: Fx) -> Fx {
    ramp(d, ONE * 45 / 10)
}

/// 0 at `from` metres or further, ONE at the station's 3.2 m, eased.
fn ramp(d: Fx, from: Fx) -> Fx {
    let to = ONE * 32 / 10;
    let t = fx::div((from - d).clamp(0, from - to), from - to);
    // eased, so it starts and stops gently
    fx::mul(fx::mul(t, t), 3 * ONE - 2 * t)
}

/// How near the camera has come to the nearest station, and which.
pub fn closeness(p: &Prep) -> (Fx, Station) {
    let (st, d) = prep::STATIONS
        .iter()
        .map(|t| (t.0, prep::station_pos(t.0).sub(p.pos).flat_len()))
        .min_by_key(|t| t.1)
        .unwrap_or((Station::Map, 99 * ONE));
    (near(d), st)
}

/// The room's camera: the overview from high above the archmage, coming
/// down to just behind them when they stand at a station, the way Ape Race
/// follows its ape (T35). It always looks the overview's way, north, so
/// the keys keep their directions on the screen; it never turns to face a
/// station (T46). It follows the archmage without turning; only how far
/// ahead or back it looks is pulled toward every station by how near that
/// station is, so it slides from one to the next and never jumps.
pub fn room_camera(p: &Prep) -> Camera {
    let at = v3(p.pos.x * 3 / 4, ONE, p.pos.z * 3 / 4);
    let over = Camera {
        eye: at.add(v3(0, 15 * ONE, 13 * ONE)),
        at: at.add(v3(0, 0, -ONE)),
        focal: ONE * 29 / 10,
    };
    // how far down it has come so far: `Prep::dip` eases toward
    // `closeness`
    let t = p.dip;
    if t == 0 {
        return over;
    }
    // behind and a little right of the archmage, so the station sits
    // left of the side panel that opens there
    let mut eye = p.pos.add(v3(cm(170), cm(600), cm(640)));
    let flat = eye.flat();
    if flat.flat_len() > WALL_R - 2 * ONE {
        let inside = flat.with_len(WALL_R - 2 * ONE);
        eye = v3(inside.x, eye.y, inside.z);
    }
    let pull = prep::STATIONS.iter().fold(V3::ZERO, |sum, s| {
        let to = prep::station_pos(s.0).sub(p.pos).flat();
        // over a longer way than the camera comes down, so the look
        // slides rather than tilts (T46)
        sum.add(to.scale(ramp(to.flat_len(), 7 * ONE)))
    });
    let close = Camera {
        eye,
        at: v3(eye.x, cm(150), p.pos.z + pull.z),
        focal: ONE * 19 / 10,
    };
    Camera {
        eye: over.eye.lerp(close.eye, t),
        at: over.at.lerp(close.at, t),
        focal: over.focal + fx::mul(close.focal - over.focal, t),
    }
}

/// The stations the title's archmage walks between, round the room (T47).
pub const TOUR: [Station; 6] = [
    Station::Library,
    Station::Book,
    Station::Lab,
    Station::Podium,
    Station::Office,
    Station::Recruit,
];

/// Where the archmage stands to use a station: on its side toward the
/// middle of the room.
pub fn stand_at(s: Station) -> V3 {
    let at = prep::station_pos(s);
    at.add(
        V3::ZERO
            .sub(at)
            .flat()
            .with_len(prep::PROP + prep::BODY + ONE / 10),
    )
}

/// The title's camera: high in the middle of the room, turning slowly so
/// every station passes, with the map table in front of it (T47). `turn`
/// runs 0 to a whole turn.
pub fn title_camera(turn: i32) -> Camera {
    let dir = v3(fx::sin(turn), 0, -fx::cos(turn));
    let mid = prep::station_pos(Station::Map);
    Camera {
        eye: mid.sub(dir.scale(cm(300))).add(v3(0, cm(620), 0)),
        at: mid.add(dir.scale(cm(850))).add(v3(0, cm(60), 0)),
        focal: ONE * 17 / 10,
    }
}

fn flame(b: &mut Batch, look: &Look, at: V3, size: Fx, tick: i32, phase: i32) {
    let k = ONE
        + fx::mul(fx::sin(tick * 1900 + phase * 7919), ONE / 8)
        + fx::mul(fx::sin(tick * 4700 + phase * 31), ONE / 16);
    b.push(
        Mesh::Cone,
        Pass::Solid,
        at,
        &sz(size / 2, fx::mul(size, k), size / 2),
        look.colour("flame"),
        ONE,
    );
    b.push(
        Mesh::Halo,
        Pass::Glow,
        at,
        &sz(size * 4, size * 4, size * 4),
        look.colour("ember"),
        ONE / 5,
    );
}

fn hair(look: &Look, id: u16) -> Rgb {
    look.colour(["hair_0", "hair_1", "hair_2", "hair_3"][id as usize % 4])
}

/// Everyone and everything that moves in the room.
pub fn room_dynamic(b: &mut Batch, look: &Look, c: &Content, p: &Prep, tick: i32) {
    // books still on the floor, in stacks
    let mut left = floor_books(p.pile.len());
    for (at, yaw, books) in pile_spots() {
        if left == 0 {
            break;
        }
        let mut y = 0;
        for (turn, h, col) in books.iter().take(left) {
            b.solid(
                Mesh::Chunk,
                at.add(v3(0, y + h / 2, 0)),
                &Frame3::new(yaw + turn, 0, 0, v3(cm(58), *h, cm(42))),
                look.colour(BOOKS[*col]),
            );
            y += h;
            left -= 1;
        }
        shadow(b, look, at, yaw, cm(42), cm(32));
    }
    // the archmage
    let moving = p.pos != p.prev;
    let reading = matches!(p.activity, Activity::Reading { .. });
    let pose = Pose {
        yaw: p.yaw,
        walk: if moving { tick * 900 } else { 0 },
        stride: cm(30),
        cast: if matches!(p.activity, Activity::Drawing { .. }) {
            ONE
        } else {
            0
        },
        arms: if reading {
            (-per_mille(160), -per_mille(160))
        } else {
            (0, 0)
        },
        nod: if reading { per_mille(50) } else { 0 },
        breath: tick * 300,
        ..Default::default()
    };
    look.draw(
        b,
        "archmage",
        p.pos,
        &pose,
        &[
            ("robe", look.colour("archmage_robe")),
            ("hat", look.colour("archmage_robe")),
        ],
        false,
    );
    shadow(b, look, p.pos, 0, cm(80), cm(80));
    if reading {
        let book = p.pos.add(v3(
            fx::mul(fx::sin(p.yaw), cm(60)),
            cm(150),
            fx::mul(fx::cos(p.yaw), cm(60)),
        ));
        for sx in [-1, 1] {
            b.solid(
                Mesh::Cube,
                place(book, p.yaw, v3(sx * cm(16), 0, 0)),
                &Frame3::new(p.yaw, 0, sx * per_mille(30), v3(cm(32), cm(3), cm(42))),
                look.colour("page"),
            );
        }
        for k in 0..6 {
            let a = (tick * 300 + k * fx::TURN / 6) % fx::TURN;
            let at = p.pos.add(v3(
                fx::mul(fx::sin(a), cm(80)),
                cm(190) + (k % 3) * cm(25),
                fx::mul(fx::cos(a), cm(80)),
            ));
            b.orb(at, cm(6), look.colour("type_acid"));
        }
    }
    // students on the benches, then standing in rows behind them; one at a
    // time puts a hand up
    let hand = (tick / 150) as usize % 8;
    for (i, s) in p.students.iter().enumerate().take(60) {
        let (at, yaw, seated) = listener(i);
        let robe = crate::scene::school_colour(look, c, s.school as usize);
        let up = seated && i == hand && (tick / 75) % 2 == 0;
        let pose = Pose {
            yaw,
            sit: if seated { cm(56) } else { 0 },
            arms: if up {
                (-per_mille(80), -per_mille(470))
            } else if seated {
                (-per_mille(70), -per_mille(70))
            } else {
                (0, 0)
            },
            breath: tick * 280 + i as i32 * 9000,
            ..Default::default()
        };
        let base = if seated { at.add(v3(0, cm(18), 0)) } else { at };
        look.draw(
            b,
            "student",
            base,
            &pose,
            &[("robe", robe), ("hat", robe), ("hair", hair(look, s.id))],
            i >= 26,
        );
        if i < 26 {
            shadow(b, look, at, 0, cm(45), cm(45));
        }
    }
    // applicants queueing at the recruitment table, the last one waving
    let rec = prep::station_pos(Station::Recruit);
    let q = queue_dir();
    let face = crate::geom::yaw_of(-q.x, -q.z);
    let n = p.applicants.len();
    for (i, a) in p.applicants.iter().enumerate() {
        let at = rec.add(q.scale(cm(160) + i as i32 * cm(125)));
        let robe = crate::scene::school_colour(look, c, a.school as usize);
        let wave = i + 1 == n && n > 1 && (tick / 90) % 3 == 0;
        let pose = Pose {
            yaw: face,
            arms: if wave {
                (0, -per_mille(420) + fx::mul(fx::sin(tick * 2600), per_mille(40)))
            } else {
                (0, 0)
            },
            breath: tick * 260 + i as i32 * 7000,
            ..Default::default()
        };
        look.draw(
            b,
            "student",
            at,
            &pose,
            &[
                ("robe", robe),
                ("hat", look.colour("eye")),
                ("hair", hair(look, i as u16 + 1)),
            ],
            false,
        );
        shadow(b, look, at, 0, cm(45), cm(45));
    }
    // helpers at work
    let (d, u) = dais();
    for j in &p.jobs {
        let (model, at, yaw) = match &j.work {
            Work::Teach(_) => ("penguin", d.add(v3(0, cm(50), 0)), crate::geom::yaw_of(u.x, u.z)),
            Work::Recruit => ("wisp_helper", rec.add(v3(cm(90), 0, cm(-100))), 0),
            // the librarian golem goes through the pile (T21)
            Work::Search => (
                "golem",
                prep::station_pos(Station::Library).add(v3(cm(160), 0, cm(220))),
                fx::TURN * 3 / 8,
            ),
            Work::Train => (
                "trainer",
                prep::station_pos(Station::Podium).add(v3(cm(-300), 0, cm(100))),
                fx::TURN / 4,
            ),
            Work::Raise(_) => continue,
            Work::Build(_) => {
                let a = (tick * 120) % fx::TURN;
                (
                    "golem",
                    prep::station_pos(Station::Map).add(polar(a, cm(330), 0)),
                    a + fx::QUARTER_TURN,
                )
            }
        };
        look.draw(
            b,
            model,
            at,
            &Pose {
                yaw,
                walk: tick * 700,
                stride: cm(20),
                ..Default::default()
            },
            &[],
            false,
        );
    }
    // flames: the sconces, the chandelier, the standing candle, the fire
    for (k, &w) in POSTS.iter().enumerate() {
        if k % 2 == 1 {
            flame(
                b,
                look,
                polar(per_mille(w), WALL_R - cm(102), cm(452)),
                cm(14),
                tick,
                k as i32,
            );
        }
    }
    let ch = prep::station_pos(Station::Map).add(v3(0, cm(880), cm(100)));
    for k in 0..8 {
        flame(
            b,
            look,
            ch.add(polar(k * fx::TURN / 8, cm(140), cm(40))),
            cm(12),
            tick,
            k + 10,
        );
    }
    let cs = candle_stand();
    for (k, (dx, dz, h)) in [(0, 0, 30), (12, 6, 24), (-11, 7, 20)].iter().enumerate() {
        flame(
            b,
            look,
            cs.add(v3(cm(*dx), cm(196 + h), cm(*dz))),
            cm(10),
            tick,
            k as i32 + 20,
        );
    }
    let k = cauldron();
    for i in 0..5 {
        flame(
            b,
            look,
            k.add(polar(i * fx::TURN / 5, cm(30), cm(22))),
            cm(34),
            tick,
            i + 30,
        );
    }
    // the brew bubbles and steams
    for i in 0..9 {
        let u = (tick * 500 + i * 7283) % fx::TURN;
        let at = k.add(polar(
            i * 7283,
            cm(10) + (i % 4) * cm(16),
            cm(186) + fx::mul(u, cm(40)),
        ));
        let s = if u < ONE * 4 / 5 {
            cm(8) + fx::mul(u, cm(8))
        } else {
            fx::mul(ONE - u, cm(60))
        };
        b.push(
            Mesh::Blob,
            Pass::Solid,
            at,
            &sz(s, s, s),
            look.colour("brew"),
            ONE * 3 / 4,
        );
    }
    for i in 0..5 {
        let u = (tick * 160 + i * fx::TURN / 5) % fx::TURN;
        let s = cm(40) + fx::mul(u, cm(150));
        b.push(
            Mesh::Halo,
            Pass::Glow,
            k.add(v3(
                fx::mul(fx::sin(tick * 200 + i * 900), cm(30)),
                cm(200) + fx::mul(u, cm(320)),
                0,
            )),
            &sz(s, s, s),
            look.colour("brew"),
            (ONE - u) / 14,
        );
    }
    // the laboratory's pieces float over the bench as crystals
    let lab = prep::station_pos(Station::Lab);
    for (i, role) in CRYSTALS.iter().enumerate() {
        let i = i as i32;
        let p_ = place(
            lab,
            -per_mille(70),
            v3(
                cm(-115) + i * cm(115),
                cm(190) + fx::mul(fx::sin(tick * 700 + i * 20000), cm(10)),
                0,
            ),
        );
        b.push(
            Mesh::Gem,
            Pass::Solid,
            p_,
            &Frame3::new(tick * 500 + i * 9000, 0, 0, v3(cm(36), cm(52), cm(36))),
            look.colour(role),
            ONE * 4 / 5,
        );
        b.push(
            Mesh::Halo,
            Pass::Glow,
            p_,
            &sz(cm(110), cm(110), cm(110)),
            look.colour(role),
            ONE / 4,
        );
    }
    // the invasion book's red beam pulses; loose pages drift up it
    let book = prep::station_pos(Station::Book);
    let pulse = ONE + fx::mul(fx::sin(tick * 900), ONE / 8);
    b.push(
        Mesh::Cyl,
        Pass::Glow,
        book.add(v3(0, cm(700), 0)),
        &sz(fx::mul(cm(50), pulse), cm(1100), fx::mul(cm(50), pulse)),
        look.colour("invasion_glow"),
        ONE / 3,
    );
    b.push(
        Mesh::Cyl,
        Pass::Glow,
        book.add(v3(0, cm(700), 0)),
        &sz(cm(14), cm(1100), cm(14)),
        look.colour("page"),
        ONE / 2,
    );
    b.push(
        Mesh::Halo,
        Pass::Glow,
        book.add(v3(0, cm(200), 0)),
        &sz(fx::mul(cm(180), pulse), cm(140), fx::mul(cm(180), pulse)),
        look.colour("invasion_glow"),
        ONE / 4,
    );
    for i in 0..6 {
        let u = (tick * 180 + i * fx::TURN / 6) % fx::TURN;
        let a = i * 7000 + tick * 300;
        let at = book.add(polar(
            a,
            cm(70) + fx::mul(u, cm(50)),
            cm(210) + fx::mul(u, cm(500)),
        ));
        b.push(
            Mesh::Cube,
            Pass::Solid,
            at,
            &Frame3::new(
                tick * 400 + i * 5000,
                tick * 300 + i * 3000,
                0,
                v3(cm(24), cm(1), cm(30)),
            ),
            look.colour("page"),
            ONE / 2,
        );
    }
    // dust turning in each window's light
    for (wi, &w) in WINDOWS.iter().enumerate() {
        let a = per_mille(w);
        let from = polar(a, WALL_R, cm(560));
        let to = polar(a, WALL_R * 4 / 10, 0).add(v3(0, 0, cm(140)));
        for i in 0..8 {
            let s = (i * fx::TURN / 8 + wi as i32 * 3000) % fx::TURN;
            let at = from.lerp(to, s).add(v3(
                fx::mul(fx::sin(tick * 60 + i * 9000), cm(40)),
                fx::mul(fx::sin(tick * 45 + i * 13000), cm(30)),
                fx::mul(fx::cos(tick * 52 + i * 7000), cm(40)),
            ));
            b.push(
                Mesh::Gem,
                Pass::Solid,
                at,
                &sz(cm(4), cm(4), cm(4)),
                look.colour("dust"),
                ONE,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content;

    fn prep_at(c: &Content, at: V3) -> Prep {
        let mut p = Prep::new(c, 0, 1, 1);
        p.start_day(c, 600);
        p.pos = at;
        p.prev = at;
        // the camera already settled there
        p.dip = closeness(&p).0;
        p
    }

    #[test]
    fn the_camera_stays_high_far_from_stations_and_comes_down_behind_the_archmage_at_one() {
        let c = content::shipped();
        // the room's middle-south, more than four and a half metres from
        // every station
        let away = prep_at(&c, v3(0, 0, 5 * ONE));
        assert_eq!(closeness(&away).0, 0);
        let over = room_camera(&away);
        assert_eq!(over.eye, v3(0, 16 * ONE, 13 * ONE + 5 * ONE * 3 / 4));
        // two metres south of the library: all the way down
        let lib = prep::station_pos(Station::Library);
        let near = prep_at(&c, lib.add(v3(0, 0, 2 * ONE)));
        let (t, st) = closeness(&near);
        assert_eq!((t, st), (ONE, Station::Library));
        let cam = room_camera(&near);
        // 6.4 m behind (further south), 1.7 m to the right (east, for a
        // camera looking north), 6 m up, looking north at the station's
        // row
        assert_eq!(cam.eye, near.pos.add(v3(cm(170), cm(600), cm(640))));
        assert_eq!(cam.at, lib.add(v3(cm(170), cm(150), 0)));
        // from the library's east side it still looks north, and does not
        // turn to face the library (T46)
        let east = prep_at(&c, lib.add(v3(2 * ONE, 0, 0)));
        let cam = room_camera(&east);
        assert_eq!(closeness(&east), (ONE, Station::Library));
        assert_eq!((cam.at.x - cam.eye.x, cam.at.z - cam.eye.z), (0, -cm(640)));
        assert!(cam.eye.y < over.eye.y && near_wall_shown(&cam) && !near_wall_shown(&over));
        // halfway in, the camera is between the two
        let mid = prep_at(&c, lib.add(v3(0, 0, ONE * 385 / 100)));
        let t = closeness(&mid).0;
        assert!(t > ONE / 4 && t < ONE * 3 / 4, "{t}");
    }

    #[test]
    fn walking_anywhere_in_the_room_never_turns_or_jumps_the_camera() {
        // the keys move the archmage along the room's axes, so the camera
        // must keep one heading for them to keep their directions on the
        // screen, and a step must never swing it (T46). Walk east across
        // the room in rows, a tick at a time, past every station.
        let c = content::shipped();
        let mut z = -12 * ONE;
        let mut dipped = 0;
        while z < 12 * ONE {
            let mut p = prep_at(&c, v3(-12 * ONE, 0, z));
            // the first step puts the archmage inside the wall
            p.step(&c, prep::RIGHT);
            p.dip = closeness(&p).0;
            let mut last = room_camera(&p);
            for _ in 0..400 {
                p.step(&c, prep::RIGHT);
                let cam = room_camera(&p);
                // looking north, never more than a few degrees aside
                let (dx, dz) = (cam.at.x - cam.eye.x, cam.at.z - cam.eye.z);
                assert!(dz < 0 && dx.abs() * 3 < -dz, "at {:?}: {dx} {dz}", p.pos);
                // walking is 7.5 cm a tick; coming down adds a forty-fifth
                // of the way from the overview, at most 15 m, so 33 cm. The
                // old camera, turning to face each station and coming all
                // the way down in 1.3 m of walking, moved over 60 cm.
                let moved = cam.eye.sub(last.eye).len().max(cam.at.sub(last.at).len());
                assert!(moved < cm(42), "at {:?} the camera moved {moved}", p.pos);
                if p.dip == ONE {
                    dipped += 1;
                }
                last = cam;
            }
            z += ONE / 2;
        }
        // the walk did pass stations with the camera all the way down
        assert!(dipped > 100, "{dipped}");
    }

    #[test]
    fn the_camera_comes_down_over_three_quarters_of_a_second() {
        let c = content::shipped();
        let lib = prep::station_pos(Station::Library);
        let mut p = prep_at(&c, v3(0, 0, 5 * ONE));
        p.pos = lib.add(v3(0, 0, 2 * ONE));
        let mut ticks = 0;
        while p.dip < ONE {
            p.step(&c, 0);
            ticks += 1;
        }
        // a forty-fifth rounds down a little, so one tick more
        assert_eq!(ticks, 46);
    }

    #[test]
    fn the_wall_steps_down_toward_the_camera_and_the_near_part_is_kept_apart() {
        assert_eq!(wall_rows(fx::HALF_TURN), WALL_ROWS);
        assert_eq!(wall_rows(0), 2);
        assert!(wall_rows(fx::QUARTER_TURN) > 2 && wall_rows(fx::QUARTER_TURN) < WALL_ROWS);
        let (room, near) = room_static(&crate::look::shipped());
        assert!(near.count() > 300, "{}", near.count());
        assert!(room.count() > near.count());
    }

    #[test]
    fn the_shelves_hold_books_and_the_floor_piles_shrink_as_the_pile_empties() {
        let look = crate::look::shipped();
        let (room, _) = room_static(&look);
        let books: usize = BOOKS
            .iter()
            .map(|r| {
                let want = look.colour(r);
                room.lists_for_test()
                    .iter()
                    .flat_map(|l| l.chunks(crate::render::INSTANCE_VALUES))
                    .filter(|i| {
                        let near = |a: i32, b: i32| (a - b).abs() <= ONE / 14 + 1;
                        near(i[12], want[0]) && near(i[13], want[1]) && near(i[14], want[2])
                    })
                    .count()
            })
            .sum();
        // three cases of five shelves, about eighteen books a shelf
        assert!(books > 150, "{books} books on the shelves");
        assert_eq!(floor_books(40), 90);
        assert_eq!(floor_books(20), 45);
        assert_eq!(floor_books(0), 0);
    }

    #[test]
    fn seated_students_face_the_dais() {
        let (d, _) = dais();
        for i in 0..12 {
            let (at, yaw, seated) = listener(i);
            assert_eq!(seated, i < 8);
            let to = d.sub(at);
            let want = crate::geom::yaw_of(to.x, to.z);
            let off = (yaw - want).rem_euclid(fx::TURN);
            // within a twelfth of a turn of looking straight at it
            assert!(off.min(fx::TURN - off) < fx::TURN / 12, "{i}: {yaw} vs {want}");
            assert!(at.flat_len() < prep::ROOM_RADIUS, "{i} stands outside the room");
        }
    }
}

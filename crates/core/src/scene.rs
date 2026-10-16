//! The two places the game draws in 3D: the tower's round room during
//! preparation, and each tower's field during the battle. The furniture of
//! the room and the scenery of a field go to the static buffer once; the
//! people, enemies, orbs and ragdolls are rebuilt every frame.

use crate::arena::{self, Arena, Cell};
use crate::battle::{self, Battle, PopKind, Team};
use crate::content::{self, Content, Field};
use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, V3};
use crate::look::{Look, Pose};
use crate::mesh::Mesh;
use crate::prep::{self, Activity, Prep, Station, Work};
use crate::render::{mix, Batch, Camera, Frame3, Pass, Rgb};
use crate::rng::Rng;

fn sz(x: Fx, y: Fx, z: Fx) -> Frame3 {
    Frame3::upright(v3(x, y, z))
}
fn cm(v: i32) -> Fx {
    v * ONE / 100
}

fn table(b: &mut Batch, look: &Look, at: V3, yaw: i32, w: Fx, d: Fx, top: &str) {
    b.inked(
        Mesh::Cube,
        at.add(v3(0, ONE, 0)),
        &Frame3::new(yaw, 0, 0, v3(w, cm(18), d)),
        look.colour(top),
    );
    for (sx, sz_) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let off = crate::render::rotate(
            v3(sx * (w / 2 - cm(20)), cm(50), sz_ * (d / 2 - cm(20))),
            yaw,
            0,
            0,
        );
        b.inked(
            Mesh::Cyl,
            at.add(off),
            &sz(cm(16), ONE, cm(16)),
            look.colour("wood_dark"),
        );
    }
}

/// The room's furniture: floor, rug, the back wall with its windows, the
/// shelves, and each station's furniture.
pub fn room_static(look: &Look) -> Batch {
    let mut b = Batch::new(look.colour("ink"));
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        v3(0, -cm(20), 0),
        &sz(30 * ONE, cm(40), 30 * ONE),
        look.colour("floor"),
        0,
    );
    for k in -10..=10 {
        b.push(
            Mesh::Cube,
            Pass::Solid,
            v3(k * cm(140), cm(1), 0),
            &sz(cm(5), cm(2), 28 * ONE),
            look.colour("floor_line"),
            0,
        );
    }
    for (r, role, y) in [(42, "rug", 3), (36, "rug_trim", 4), (33, "rug", 5)] {
        b.push(
            Mesh::Cyl,
            Pass::Solid,
            v3(0, cm(y), cm(50)),
            &sz(cm(r * 20), cm(5), cm(r * 16)),
            look.colour(role),
            0,
        );
    }
    // the back half of the round wall
    for k in 0..16 {
        let a = fx::QUARTER_TURN + fx::HALF_TURN * k / 15;
        let at = v3(
            fx::mul(fx::sin(a), 15 * ONE),
            5 * ONE,
            fx::mul(fx::cos(a), 15 * ONE),
        );
        b.push(
            Mesh::Cube,
            Pass::Solid,
            at,
            &Frame3::new(a, 0, 0, v3(cm(310), 10 * ONE, cm(40))),
            look.colour("wall"),
            0,
        );
        if k % 2 == 1 {
            let w = v3(
                fx::mul(fx::sin(a), cm(1470)),
                6 * ONE,
                fx::mul(fx::cos(a), cm(1470)),
            );
            b.push(
                Mesh::Cube,
                Pass::Solid,
                w,
                &Frame3::new(a, 0, 0, v3(cm(140), cm(360), cm(10))),
                look.colour("window"),
                ONE / 2,
            );
        } else {
            let p = v3(
                fx::mul(fx::sin(a), cm(1430)),
                5 * ONE,
                fx::mul(fx::cos(a), cm(1430)),
            );
            b.inked(Mesh::Cyl, p, &sz(ONE, 10 * ONE, ONE), look.colour("pillar"));
        }
    }
    // library shelves, nearly empty
    let lib = prep::station_pos(Station::Library);
    for (dx, dz, yaw) in [(-300, -150, 120), (-50, -450, 80), (-410, 190, 190)] {
        let at = lib.add(v3(cm(dx), 0, cm(dz)));
        let yaw = fx::TURN * yaw / 1000;
        b.inked(
            Mesh::Cube,
            at.add(v3(0, cm(170), 0)),
            &Frame3::new(yaw, 0, 0, v3(cm(320), cm(340), cm(70))),
            look.colour("shelf"),
        );
        for k in 0..4 {
            let off = crate::render::rotate(v3(0, cm(75 + k * 80), cm(14)), yaw, 0, 0);
            b.push(
                Mesh::Cube,
                Pass::Solid,
                at.add(off),
                &Frame3::new(yaw, 0, 0, v3(cm(290), cm(62), cm(50))),
                look.colour("shelf_back"),
                0,
            );
        }
    }
    // the laboratory
    let lab = prep::station_pos(Station::Lab);
    table(
        &mut b,
        look,
        lab,
        fx::TURN * -95 / 1000,
        cm(360),
        cm(180),
        "lab_table",
    );
    b.inked(
        Mesh::Cyl,
        lab.add(v3(cm(260), cm(50), cm(120))),
        &sz(cm(150), ONE, cm(150)),
        look.colour("cauldron"),
    );
    b.inked(
        Mesh::Sphere,
        lab.add(v3(cm(260), cm(150), cm(120))),
        &sz(cm(140), cm(110), cm(140)),
        look.colour("cauldron"),
    );
    b.push(
        Mesh::Cyl,
        Pass::Solid,
        lab.add(v3(cm(260), cm(205), cm(120))),
        &sz(cm(120), cm(5), cm(120)),
        look.colour("brew"),
        ONE,
    );
    for (k, role) in ["type_fire", "type_acid", "type_frost"].iter().enumerate() {
        let at = lab.add(v3(
            cm(-90 + 90 * k as i32),
            cm(150),
            cm(-20 + 30 * (k as i32 % 2)),
        ));
        b.push(
            Mesh::Ico,
            Pass::Solid,
            at,
            &Frame3::new(fx::TURN / 8, fx::TURN / 9, 0, v3(cm(50), cm(60), cm(50))),
            look.colour(role),
            ONE * 3 / 4,
        );
    }
    // the invasion book on its lectern
    let book = prep::station_pos(Station::Book);
    b.inked(
        Mesh::Cyl,
        book.add(v3(0, cm(70), 0)),
        &sz(cm(70), cm(140), cm(70)),
        look.colour("wood_dark"),
    );
    b.inked(
        Mesh::Cube,
        book.add(v3(0, cm(150), cm(10))),
        &Frame3::new(0, fx::TURN * 60 / 1000, 0, v3(cm(120), cm(12), cm(80))),
        look.colour("invasion_book"),
    );
    // the map table
    let map = prep::station_pos(Station::Map);
    table(&mut b, look, map, 0, cm(340), cm(240), "wood");
    b.push(
        Mesh::Cube,
        Pass::Solid,
        map.add(v3(0, cm(112), 0)),
        &sz(cm(300), cm(6), cm(200)),
        look.colour("map_green"),
        0,
    );
    b.inked(
        Mesh::Cyl,
        map.add(v3(cm(90), cm(140), cm(-40))),
        &sz(cm(30), cm(55), cm(30)),
        look.colour("tower_stone"),
    );
    b.inked(
        Mesh::Cone,
        map.add(v3(cm(90), cm(185), cm(-40))),
        &sz(cm(38), cm(35), cm(38)),
        look.colour("tower_roof"),
    );
    // the podium and its benches
    let pod = prep::station_pos(Station::Podium);
    b.inked(
        Mesh::Cyl,
        pod.add(v3(cm(220), cm(20), cm(-140))),
        &sz(cm(300), cm(40), cm(300)),
        look.colour("wood"),
    );
    b.inked(
        Mesh::Cyl,
        pod.add(v3(cm(220), cm(100), cm(-140))),
        &sz(cm(60), cm(130), cm(60)),
        look.colour("wood_light"),
    );
    b.inked(
        Mesh::Cube,
        pod.add(v3(cm(220), cm(170), cm(-135))),
        &Frame3::new(
            fx::TURN * -140 / 1000,
            fx::TURN * 55 / 1000,
            0,
            v3(cm(100), cm(10), cm(70)),
        ),
        look.colour("wood_dark"),
    );
    for (k, (dx, dz)) in [(-160, 140), (-70, 290), (20, 440)].iter().enumerate() {
        let _ = k;
        b.inked(
            Mesh::Cube,
            pod.add(v3(cm(*dx), cm(55), cm(*dz))),
            &Frame3::new(fx::TURN * -140 / 1000, 0, 0, v3(cm(420), cm(15), cm(60))),
            look.colour("wood_light"),
        );
    }
    // the recruitment table
    let rec = prep::station_pos(Station::Recruit);
    table(
        &mut b,
        look,
        rec,
        fx::TURN * 125 / 1000,
        cm(300),
        cm(160),
        "wood_light",
    );
    b
}

/// Where the scattered books lie, the same each match.
fn book_spots() -> Vec<(V3, i32, usize)> {
    let mut r = Rng::new(7);
    let lib = prep::station_pos(Station::Library);
    (0..90)
        .map(|_| {
            let a = r.between(0, fx::TURN - 1);
            let d = r.between(ONE / 2, cm(400));
            (
                lib.add(v3(fx::mul(fx::cos(a), d), cm(8), fx::mul(fx::sin(a), d) * 4 / 5)),
                r.between(0, fx::TURN - 1),
                r.below(7) as usize,
            )
        })
        .collect()
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

pub fn school_colour(look: &Look, c: &Content, school: usize) -> Rgb {
    look.colour(&format!(
        "school_{}",
        c.file.schools.get(school).map(|s| s.id.as_str()).unwrap_or("")
    ))
}

/// Everyone and everything that moves in the room.
pub fn room_dynamic(b: &mut Batch, look: &Look, c: &Content, p: &Prep, tick: i32) {
    // books still on the floor
    let left = p.pile.len();
    for (k, (at, yaw, col)) in book_spots().into_iter().enumerate() {
        if k * 40 >= left.max(1) * 90 {
            break;
        }
        b.inked(
            Mesh::Cube,
            at,
            &Frame3::new(yaw, 0, 0, v3(cm(62), cm(14), cm(46))),
            look.colour(BOOKS[col]),
        );
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
    if reading {
        for k in 0..6 {
            let a = (tick * 300 + k * fx::TURN / 6) % fx::TURN;
            let at = p.pos.add(v3(
                fx::mul(fx::sin(a), cm(70)),
                cm(170) + (k % 3) * cm(25),
                fx::mul(fx::cos(a), cm(70)),
            ));
            b.orb(at, cm(6), look.colour("type_acid"));
        }
    }
    // students on the benches, then standing in rows behind them
    let pod = prep::station_pos(Station::Podium);
    for (i, s) in p.students.iter().enumerate().take(60) {
        let (row, col) = (i / 6, i % 6);
        let along = crate::render::rotate(v3(cm(-170 + col as i32 * 68), 0, 0), fx::TURN * -140 / 1000, 0, 0);
        let base = pod
            .add(v3(
                cm(-160 + 90 * row as i32),
                if row < 3 { cm(62) } else { 0 },
                cm(140 + 150 * row as i32),
            ))
            .add(along);
        let robe = school_colour(look, c, s.school as usize);
        let pose = Pose {
            yaw: fx::TURN * 360 / 1000,
            scale: ONE,
            ..Default::default()
        };
        look.draw(
            b,
            "student",
            base,
            &pose,
            &[("robe", robe), ("hat", robe)],
            row > 4,
        );
    }
    // applicants waiting at the recruitment table
    let rec = prep::station_pos(Station::Recruit);
    for (i, a) in p.applicants.iter().enumerate() {
        let base = rec.add(v3(cm(-160 - 110 * i as i32), 0, cm(160 + 110 * i as i32)));
        let robe = school_colour(look, c, a.school as usize);
        look.draw(
            b,
            "student",
            base,
            &Pose {
                yaw: fx::TURN * 375 / 1000,
                ..Default::default()
            },
            &[("robe", robe), ("hat", look.colour("eye"))],
            false,
        );
    }
    // helpers at work
    for j in &p.jobs {
        let (model, at, yaw) = match &j.work {
            Work::Teach(_) => (
                "penguin",
                pod.add(v3(cm(220), cm(40), cm(-140))),
                fx::TURN * 360 / 1000,
            ),
            Work::Recruit => ("wisp_helper", rec.add(v3(cm(90), 0, cm(-100))), 0),
            Work::Train => ("trainer", pod.add(v3(cm(-300), 0, cm(100))), fx::TURN / 4),
            Work::Raise(_) => continue,
            Work::Build(_) => {
                let a = (tick * 120) % fx::TURN;
                (
                    "golem",
                    prep::station_pos(Station::Map).add(v3(
                        fx::mul(fx::sin(a), cm(330)),
                        0,
                        fx::mul(fx::cos(a), cm(330)),
                    )),
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
}

pub fn room_camera(p: &Prep) -> Camera {
    let at = v3(p.pos.x * 3 / 4, ONE, p.pos.z * 3 / 4);
    Camera {
        eye: at.add(v3(0, 15 * ONE, 13 * ONE)),
        at: at.add(v3(0, 0, -ONE)),
        focal: ONE * 29 / 10,
    }
}

/// A field's scenery, tower and placed features.
pub fn arena_static(look: &Look, c: &Content, a: &Arena, b_: Option<&Battle>, seats: usize) -> Batch {
    let mut b = Batch::new(look.colour("ink"));
    let half = arena::SIZE * arena::CELL / 2;
    // grass in patches of two shades
    let mut r = Rng::new(31);
    for z in 0..8 {
        for x in 0..8 {
            let at = v3(
                -half + (x * 2 + 1) * half / 8,
                -cm(5),
                -half + (z * 2 + 1) * half / 8,
            );
            let role = if r.below(3) == 0 { "grass_dark" } else { "grass" };
            b.push(
                Mesh::Cube,
                Pass::Solid,
                at,
                &sz(half / 4 + cm(2), cm(10), half / 4 + cm(2)),
                look.colour(role),
                0,
            );
        }
    }
    // the trail along today's route
    for (x, z) in a.route() {
        b.push(
            Mesh::Cyl,
            Pass::Solid,
            arena::centre(x, z).add(v3(0, cm(2), 0)),
            &sz(cm(190), cm(6), cm(190)),
            look.colour("path"),
            0,
        );
    }
    // trees and rocks
    for z in 0..arena::SIZE {
        for x in 0..arena::SIZE {
            if a.at(x, z) != Cell::Scenery {
                continue;
            }
            let at = arena::centre(x, z);
            if (x * 7 + z * 3) % 3 == 0 {
                b.inked(
                    Mesh::Ico,
                    at.add(v3(0, cm(40), 0)),
                    &Frame3::new(x * 1000, z * 700, 0, v3(cm(130), cm(90), cm(120))),
                    look.colour("rock"),
                );
            } else {
                b.inked(
                    Mesh::Cyl,
                    at.add(v3(0, cm(60), 0)),
                    &sz(cm(30), cm(120), cm(30)),
                    look.colour("trunk"),
                );
                let role = if (x + z) % 2 == 0 { "tree" } else { "tree_dark" };
                b.inked(
                    Mesh::Ico,
                    at.add(v3(0, cm(180), 0)),
                    &Frame3::new(x * 500, 0, 0, v3(cm(190), cm(180), cm(190))),
                    look.colour(role),
                );
                b.inked(
                    Mesh::Ico,
                    at.add(v3(cm(30), cm(250), cm(10))),
                    &Frame3::new(z * 900, 0, 0, v3(cm(140), cm(130), cm(140))),
                    look.colour(role),
                );
            }
        }
    }
    // the entry: two posts and a lintel
    let e = arena::centre(a.entry.0, a.entry.1);
    for dx in [-1, 1] {
        b.inked(
            Mesh::Cyl,
            e.add(v3(dx * cm(160), cm(110), 0)),
            &sz(cm(40), cm(220), cm(40)),
            look.colour("tower_ring"),
        );
    }
    b.inked(
        Mesh::Cube,
        e.add(v3(0, cm(230), 0)),
        &sz(cm(380), cm(30), cm(50)),
        look.colour("tower_ring"),
    );
    // the tower, its ring, its roof and its seats
    let t = battle::tower_centre(a);
    b.inked(
        Mesh::Cyl,
        t.add(v3(0, cm(350), 0)),
        &sz(cm(440), cm(700), cm(440)),
        look.colour("tower_stone"),
    );
    b.inked(
        Mesh::Cyl,
        t.add(v3(0, cm(720), 0)),
        &sz(cm(500), cm(60), cm(500)),
        look.colour("tower_ring"),
    );
    b.inked(
        Mesh::Cone,
        t.add(v3(0, cm(920), 0)),
        &sz(cm(540), cm(340), cm(540)),
        look.colour("tower_roof"),
    );
    b.orb(t.add(v3(0, cm(1110), 0)), cm(28), look.colour("gold"));
    for k in 0..4 {
        let ang = k * fx::QUARTER_TURN + fx::TURN / 8;
        let w = t.add(v3(
            fx::mul(fx::sin(ang), cm(222)),
            cm(420 + 80 * (k % 2)),
            fx::mul(fx::cos(ang), cm(222)),
        ));
        b.push(
            Mesh::Cube,
            Pass::Solid,
            w,
            &Frame3::new(ang, 0, 0, v3(cm(50), cm(80), cm(10))),
            look.colour("tower_window"),
            ONE / 2,
        );
    }
    for i in 0..seats.min(12) {
        let s = battle::seat_pos(t, i);
        let dir = s.sub(t).flat();
        b.inked(
            Mesh::Cyl,
            s.sub(v3(0, cm(20), 0)),
            &sz(cm(150), cm(30), cm(150)),
            look.colour("tower_ring"),
        );
        let arm_at = t.add(dir.ratio(55, 100)).add(v3(0, s.y - cm(40), 0));
        b.push(
            Mesh::Cube,
            Pass::Solid,
            arm_at,
            &Frame3::new(
                crate::geom::yaw_of(dir.x, dir.z),
                0,
                0,
                v3(cm(40), cm(30), cm(200)),
            ),
            look.colour("tower_stone"),
            0,
        );
    }
    // what the archmage placed before the battle
    if let Some(bt) = b_ {
        for s in bt.standing.iter().filter(|s| s.life < 0) {
            draw_feature(&mut b, look, c, s.feature, &s.cells);
        }
    }
    b
}

fn draw_feature(b: &mut Batch, look: &Look, c: &Content, feature: u16, cells: &[(i32, i32)]) {
    let f = &c.file.features[feature as usize];
    for &(x, z) in cells {
        let at = arena::centre(x, z);
        match f.id.as_str() {
            "glass_wall" => b.push(
                Mesh::Cube,
                Pass::Glow,
                at.add(v3(0, cm(80), 0)),
                &sz(arena::CELL, cm(160), cm(40)),
                look.colour("glass"),
                ONE / 3,
            ),
            "ivory_wall" => {
                b.inked(
                    Mesh::Cube,
                    at.add(v3(0, cm(70), 0)),
                    &sz(arena::CELL, cm(140), cm(55)),
                    look.colour("ivory"),
                );
                b.inked(
                    Mesh::Sphere,
                    at.add(v3(0, cm(145), 0)),
                    &sz(cm(64), cm(64), cm(64)),
                    look.colour("ivory"),
                );
            }
            "sub_tower" => {
                b.inked(
                    Mesh::Cyl,
                    at.add(v3(0, cm(150), 0)),
                    &sz(cm(200), cm(300), cm(200)),
                    look.colour("tower_stone"),
                );
                b.inked(
                    Mesh::Cyl,
                    at.add(v3(0, cm(315), 0)),
                    &sz(cm(240), cm(35), cm(240)),
                    look.colour("tower_ring"),
                );
            }
            _ => {}
        }
    }
}

fn enemy_model(c: &Content, kind: u16) -> &str {
    c.file
        .enemies
        .get(kind as usize)
        .map(|e| e.model.as_str())
        .unwrap_or("skeleton")
}

/// Everything that moves on a field. `far` draws the watch screen's
/// simpler figures.
pub fn arena_dynamic(b: &mut Batch, look: &Look, c: &Content, bt: &Battle, far: bool) {
    let t = bt.tower();
    for (i, k) in bt.casters.iter().enumerate() {
        let robe = school_colour(look, c, k.plan.school as usize);
        let face = k.aim.sub(k.pos);
        let pose = Pose {
            yaw: crate::geom::yaw_of(face.x, face.z),
            cast: if k.casting > 0 { ONE } else { 0 },
            ..Default::default()
        };
        let base = v3(
            k.pos.x,
            k.pos.y - if k.pos.y > ONE { 0 } else { ONE / 2 },
            k.pos.z,
        );
        if far && i >= 12 {
            continue;
        }
        look.draw(b, "student", base, &pose, &[("robe", robe), ("hat", robe)], far);
    }
    for u in &bt.units {
        let e = &c.file.enemies[u.kind as usize];
        let model = enemy_model(c, u.kind);
        let h = content::cm(e.height);
        let scale = (h as i64 * ONE as i64 / look.height(model).max(1) as i64) as Fx;
        let speed = u.body.vel.flat_len();
        let pose = Pose {
            yaw: u.body.yaw,
            lean: (u.body.lean.0, u.body.lean.1),
            walk: if speed > ONE / 400 && u.freeze == 0 {
                bt.tick * 1100 + u.id as i32 * 4000
            } else {
                0
            },
            stride: cm(25),
            scale,
            flash: u.flash > 3,
            frozen: u.freeze > 0,
            ..Default::default()
        };
        let base = v3(u.body.pos.x, u.body.pos.y - u.body.radius, u.body.pos.z);
        let swap: Vec<(&str, Rgb)> = if u.team == Team::Ally && u.home == Team::Ally {
            vec![(
                "bone",
                mix(look.colour("bone"), look.colour("golem_eye"), ONE / 3),
            )]
        } else {
            vec![]
        };
        look.draw(b, model, base, &pose, &swap, far);
        if u.freeze > 0 {
            b.push(
                Mesh::Cube,
                Pass::Glow,
                base.add(v3(0, h / 2, 0)),
                &sz(u.body.radius * 3, h + cm(30), u.body.radius * 3),
                look.colour("ice"),
                ONE / 3,
            );
        }
        if u.charm > 0 && u.home == Team::Foe {
            b.orb(base.add(v3(0, h + cm(40), 0)), cm(14), look.colour("charm"));
        }
    }
    for s in &bt.shots {
        let rgb = look.colour(&format!(
            "type_{}",
            c.file
                .types
                .get(s.dtype as usize)
                .map(|t| t.id.as_str())
                .unwrap_or("fire")
        ));
        b.orb(s.pos, s.radius.max(cm(12)), rgb);
    }
    for z in &bt.zones {
        let (role, glow) = match z.field {
            Field::Oil => ("oil", 0),
            Field::Lava => ("lava", ONE * 3 / 4),
            Field::Thorns => ("thorns", 0),
            Field::ManaSpring => ("mana_spring", ONE / 2),
        };
        b.push(
            Mesh::Cyl,
            Pass::Solid,
            z.pos.add(v3(0, cm(4), 0)),
            &sz(z.radius * 2, cm(6), z.radius * 2),
            look.colour(role),
            glow,
        );
    }
    for l in &bt.lures {
        b.inked(
            Mesh::Cyl,
            l.pos.add(v3(0, ONE, 0)),
            &sz(cm(20), 2 * ONE, cm(20)),
            look.colour("wood"),
        );
        b.inked(
            Mesh::Sphere,
            l.pos.add(v3(0, cm(160), 0)),
            &sz(cm(110), cm(130), cm(110)),
            look.colour("bait"),
        );
        b.push(
            Mesh::Cyl,
            Pass::Glow,
            l.pos.add(v3(0, cm(160), 0)),
            &sz(cm(180), cm(12), cm(180)),
            look.colour("lure"),
            ONE / 2,
        );
    }
    for s in bt.standing.iter().filter(|s| s.life >= 0) {
        draw_feature(b, look, c, s.feature, &s.cells);
    }
    for d in &bt.pile.dolls {
        let e = &c.file.enemies[d.kind as usize];
        let fade = d
            .fade
            .map(|f| (f as i32 * ONE / crate::ragdoll::FADE_TICKS as i32).clamp(0, ONE))
            .unwrap_or(ONE);
        look.draw_ragdoll(b, enemy_model(c, d.kind), d, content::cm(e.height), fade, far);
    }
    for p in &bt.pops {
        match p.kind {
            PopKind::Ring => {
                let rgb = look.colour(&format!(
                    "type_{}",
                    c.file
                        .types
                        .get(p.dtype as usize)
                        .map(|t| t.id.as_str())
                        .unwrap_or("fire")
                ));
                let grow = p.radius * (30 - p.life.min(24)) / 24;
                b.push(
                    Mesh::Sphere,
                    Pass::Glow,
                    p.pos.add(v3(0, ONE / 2, 0)),
                    &sz(grow * 2, grow, grow * 2),
                    rgb,
                    ONE / 2,
                );
            }
            PopKind::Freeze => b.orb(p.pos.add(v3(0, cm(250), 0)), cm(18), look.colour("ice")),
            PopKind::Charm => b.orb(p.pos.add(v3(0, cm(250), 0)), cm(18), look.colour("charm")),
            PopKind::Number => {}
        }
    }
    if bt.ward > 0 {
        let r = cm(500) + bt.ward.min(battle::WARD_CAP) * cm(1) / 10;
        b.push(
            Mesh::Sphere,
            Pass::Glow,
            t.add(v3(0, cm(300), 0)),
            &sz(r, r, r),
            look.colour("ward"),
            ONE / 5,
        );
    }
}

/// A battle view's own camera (T5): it circles a point on the field.
/// `yaw` and `pitch` are fx angles (`fx::TURN` a full turn), `dist` and
/// `at` are Q16 metres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Orbit {
    pub yaw: i32,
    pub pitch: i32,
    pub dist: Fx,
    pub at: V3,
}

/// How low and how high a view may tilt, and how near and far it may go.
pub const PITCH_MIN: i32 = fx::TURN * 10 / 360;
pub const PITCH_MAX: i32 = fx::TURN * 85 / 360;
pub const DIST_MIN: Fx = 8 * ONE;
pub const DIST_MAX: Fx = 120 * ONE;
/// How far from the field's middle the point a view circles may move.
pub const PAN_MAX: Fx = 30 * ONE;

impl Orbit {
    /// The two fixed views the game had before (`arena_camera`), as orbits:
    /// eye (-4, 46, 44) round (1, 0, 1) is 63.2 m away, 46.7° up, turned
    /// 6.6° left; eye (-20, 26, 26) round (-2, 0, 1) is 40.3 m, 40.2° up,
    /// turned 35.8° left.
    pub fn start(close: bool) -> Orbit {
        if close {
            Orbit {
                yaw: -(fx::TURN * 358 / 3600),
                pitch: fx::TURN * 402 / 3600,
                dist: ONE * 403 / 10,
                at: v3(-2 * ONE, 0, ONE),
            }
        } else {
            Orbit {
                yaw: -(fx::TURN * 66 / 3600),
                pitch: fx::TURN * 467 / 3600,
                dist: ONE * 632 / 10,
                at: v3(ONE, 0, ONE),
            }
        }
    }

    pub fn camera(&self) -> Camera {
        let flat = fx::mul(fx::cos(self.pitch), self.dist);
        let eye = self.at.add(v3(
            fx::mul(fx::sin(self.yaw), flat),
            fx::mul(fx::sin(self.pitch), self.dist),
            fx::mul(fx::cos(self.yaw), flat),
        ));
        Camera {
            eye,
            at: self.at,
            focal: ONE * 24 / 10,
        }
    }

    /// Turns and tilts by a drag of (dx, dy) pixels in a view `h` pixels
    /// tall: the view's height is half a turn across, a quarter turn up.
    pub fn orbit(&mut self, dx: i32, dy: i32, h: i32) {
        let h = h.max(1);
        self.yaw -= dx * (fx::TURN / 2) / h;
        self.pitch = (self.pitch + dy * (fx::TURN / 4) / h).clamp(PITCH_MIN, PITCH_MAX);
    }

    /// Slides the point it circles by a drag of (dx, dy) pixels, so the
    /// ground under the pointer follows it roughly.
    pub fn pan(&mut self, dx: i32, dy: i32, h: i32) {
        let per_px = self.dist / h.max(1);
        let (dx, dy) = (dx.clamp(-4000, 4000), dy.clamp(-4000, 4000));
        let (s, c) = (fx::sin(self.yaw), fx::cos(self.yaw));
        // right along the screen is (cos, 0, -sin); up the screen is away, (-sin, 0, -cos)
        let right = -dx * per_px;
        let away = dy * per_px;
        let x = self.at.x + fx::mul(c, right) - fx::mul(s, away);
        let z = self.at.z - fx::mul(s, right) - fx::mul(c, away);
        self.at = v3(x.clamp(-PAN_MAX, PAN_MAX), 0, z.clamp(-PAN_MAX, PAN_MAX));
    }

    /// Nearer for a wheel turned up (negative), further for down; one
    /// notch (100) is about a sixth.
    pub fn zoom(&mut self, delta: i32) {
        let d = delta.clamp(-400, 400);
        self.dist = (self.dist + fx::mul(self.dist, d * ONE / 600)).clamp(DIST_MIN, DIST_MAX);
    }
}

pub fn arena_camera(close: bool) -> Camera {
    if close {
        Camera {
            eye: v3(-20 * ONE, 26 * ONE, 26 * ONE),
            at: v3(-2 * ONE, 0, ONE),
            focal: ONE * 22 / 10,
        }
    } else {
        Camera {
            eye: v3(-4 * ONE, 46 * ONE, 44 * ONE),
            at: v3(ONE, 0, ONE),
            focal: ONE * 24 / 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::{Plan, StudentPlan};

    #[test]
    fn a_full_watch_screen_stays_under_its_instance_cap() {
        // four battles at their busiest: 120 enemies each, every student casting
        let c = content::shipped();
        let look = crate::look::shipped();
        let fire = vec![c.piece("fire_bolt").unwrap() as u16];
        let student = StudentPlan {
            mana_max: 200,
            regen: 20,
            spells: vec![(0, 100)],
            school: 0,
        };
        let plan = Plan {
            spells: vec![fire],
            students: vec![student; 20],
            placed: vec![],
        };
        let wild = [battle::Wild {
            enemy: c.enemy("bramble_elf").unwrap() as u16,
            count: 120,
            at_tick: 1,
        }];
        let mut total = 0;
        for seat in 0..4u8 {
            let mut bt = Battle::new(&c, seat, &plan, &[], &wild, seat as u64, seat as u64, 60 * 90, 24);
            for _ in 0..60 * 25 {
                bt.step(&c);
            }
            let mut b = Batch::new(look.colour("ink"));
            arena_dynamic(&mut b, &look, &c, &bt, true);
            total += b.count() + arena_static(&look, &c, &bt.arena, Some(&bt), 12).count();
        }
        // LTC G7: on a software renderer the cost is per instance
        assert!(total < 16_000, "{total} instances on the watch screen");
    }

    #[test]
    fn the_room_draws_its_furniture_once_and_its_people_each_frame() {
        let c = content::shipped();
        let look = crate::look::shipped();
        let s = room_static(&look);
        assert!(s.count() > 100);
        let mut p = Prep::new(&c, 0, 1, 1);
        p.start_day(&c, 600);
        let mut d = Batch::new(look.colour("ink"));
        room_dynamic(&mut d, &look, &c, &p, 1);
        assert!(d.count() > 30, "{}", d.count());
    }
}

//! Ragdolls: an enemy that dies or takes a heavy hit becomes a set of
//! Verlet points joined by sticks (Jakobsen's method, as Vagrancy's
//! `crates/sim` uses it). Velocity is the difference between a point and
//! where it was; each tick the points move, fall, and then the sticks, the
//! ground and the walls are satisfied over a few passes. Two of
//! Vagrancy's lessons hold here: every correction is shared by weight, so
//! a stick never moves the pair's centre of mass, and ground friction is
//! applied inside every pass, not once a tick, so limbs do not splay.
//!
//! A ragdoll that has barely moved for a while goes to sleep and costs
//! nothing until it fades.

use serde::{Deserialize, Serialize};

use crate::arena::{self, Arena, Cell};
use crate::body::GRAVITY;
use crate::fx::{Fx, ONE};
use crate::geom::{v3, V3};

pub const PASSES: usize = 4;
/// A point's collision radius against the ground and walls.
pub const POINT_RADIUS: Fx = ONE / 10;
/// Ticks of near stillness before sleep.
pub const SLEEP_AFTER: u16 = 30;
/// Motion under this (summed over the points, metres a tick) counts as still.
pub const STILL: Fx = ONE / 200;

/// The seven points every humanoid ragdoll has, by name.
pub const PELVIS: usize = 0;
pub const CHEST: usize = 1;
pub const HEAD: usize = 2;
pub const HAND_L: usize = 3;
pub const HAND_R: usize = 4;
pub const FOOT_L: usize = 5;
pub const FOOT_R: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub pos: V3,
    pub prev: V3,
    /// Relative weight; heavier points move less in a correction.
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stick {
    pub a: u8,
    pub b: u8,
    pub rest: Fx,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ragdoll {
    pub points: Vec<Point>,
    pub sticks: Vec<Stick>,
    /// The enemy kind it was, for drawing.
    pub kind: u16,
    pub age: u32,
    still: u16,
    pub asleep: bool,
    /// Ticks left before it is removed, once fading.
    pub fade: Option<u16>,
}

/// Rest offsets of the seven points for a body `height` tall, standing at
/// the origin facing +z.
fn skeleton(height: Fx) -> [V3; 7] {
    let h = |x: i32, y: i32| v3(height * x / 100, height * y / 100, 0);
    [
        h(0, 45),
        h(0, 70),
        h(0, 93),
        h(-30, 58),
        h(30, 58),
        h(-12, 4),
        h(12, 4),
    ]
}

const LINKS: [(usize, usize); 9] = [
    (PELVIS, CHEST),
    (CHEST, HEAD),
    (CHEST, HAND_L),
    (CHEST, HAND_R),
    (PELVIS, FOOT_L),
    (PELVIS, FOOT_R),
    // braces that keep the shape from folding flat
    (PELVIS, HEAD),
    (FOOT_L, FOOT_R),
    (HAND_L, HAND_R),
];

impl Ragdoll {
    /// A ragdoll standing where a body stood, moving with its velocity,
    /// with `kick` added to the upper points so it topples.
    pub fn from_body(at: V3, yaw: i32, height: Fx, vel: V3, kick: V3, kind: u16) -> Ragdoll {
        let base = v3(at.x, 0, at.z);
        let rest = skeleton(height);
        let points: Vec<Point> = rest
            .iter()
            .enumerate()
            .map(|(i, off)| {
                let pos = base.add(off.yaw(yaw));
                let extra = match i {
                    HEAD | CHEST | HAND_L | HAND_R => kick,
                    _ => kick.ratio(1, 3),
                };
                let weight = match i {
                    PELVIS | CHEST => 3,
                    HEAD => 2,
                    _ => 1,
                };
                Point {
                    pos,
                    prev: pos.sub(vel.add(extra)),
                    weight,
                }
            })
            .collect();
        let sticks = LINKS
            .iter()
            .map(|&(a, b)| Stick {
                a: a as u8,
                b: b as u8,
                rest: rest[b].sub(rest[a]).len(),
            })
            .collect();
        Ragdoll {
            points,
            sticks,
            kind,
            age: 0,
            still: 0,
            asleep: false,
            fade: None,
        }
    }

    pub fn centre(&self) -> V3 {
        let n = self.points.len().max(1) as i32;
        let s = self
            .points
            .iter()
            .fold(V3::ZERO, |acc, p| acc.add(p.pos.ratio(1, n)));
        s
    }

    pub fn step(&mut self, arena: &Arena) {
        self.age += 1;
        if let Some(f) = self.fade.as_mut() {
            *f = f.saturating_sub(1);
        }
        if self.asleep {
            return;
        }
        let mut motion: Fx = 0;
        for p in self.points.iter_mut() {
            let v = p.pos.sub(p.prev).ratio(99, 100);
            p.prev = p.pos;
            p.pos = p.pos.add(v);
            p.pos.y -= GRAVITY;
            motion = motion.saturating_add(v.len());
        }
        for _ in 0..PASSES {
            for s in &self.sticks {
                let (a, b) = (s.a as usize, s.b as usize);
                let d = self.points[b].pos.sub(self.points[a].pos);
                let l = d.len();
                if l == 0 {
                    continue;
                }
                let err = l - s.rest;
                let (wa, wb) = (self.points[a].weight, self.points[b].weight);
                // the lighter end moves more: each moves err × other / (wa + wb)
                let fix = d.ratio(err, l);
                let move_a = fix.ratio(wb, wa + wb);
                let move_b = fix.sub(move_a);
                self.points[a].pos = self.points[a].pos.add(move_a);
                self.points[b].pos = self.points[b].pos.sub(move_b);
            }
            for p in self.points.iter_mut() {
                if p.pos.y < POINT_RADIUS {
                    p.pos.y = POINT_RADIUS;
                    // friction inside every pass: give up most of the sideways motion
                    p.prev.x = p.pos.x - (p.pos.x - p.prev.x) * 3 / 5;
                    p.prev.z = p.pos.z - (p.pos.z - p.prev.z) * 3 / 5;
                    if p.prev.y < p.pos.y {
                        p.prev.y = p.pos.y;
                    }
                }
                keep_point_out(p, arena);
            }
        }
        if motion < STILL {
            self.still += 1;
            if self.still >= SLEEP_AFTER {
                self.asleep = true;
            }
        } else {
            self.still = 0;
        }
    }
}

fn keep_point_out(p: &mut Point, a: &Arena) {
    let half = arena::SIZE * arena::CELL / 2 - POINT_RADIUS;
    p.pos.x = p.pos.x.clamp(-half, half);
    p.pos.z = p.pos.z.clamp(-half, half);
    let (x, z) = arena::cell_of(p.pos);
    // only low points meet walls; walls are under 2 m tall
    if p.pos.y > 2 * ONE {
        return;
    }
    if matches!(a.at(x, z), Cell::Feature(_) | Cell::Tower) {
        // step back out the way it came in
        let back = p.prev;
        let (bx, bz) = arena::cell_of(back);
        if (bx, bz) != (x, z) {
            p.pos.x = back.x;
            p.pos.z = back.z;
        }
    }
}

/// The ragdolls of one arena, with the cap on how many move at once.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Pile {
    pub dolls: Vec<Ragdoll>,
}

/// Ticks a ragdoll fades for before it is removed.
pub const FADE_TICKS: u16 = 90;
/// A sleeping ragdoll starts to fade after this many ticks of age.
pub const LINGER: u32 = 60 * 6;

impl Pile {
    /// Adds a ragdoll. When more than `cap` are awake, the oldest awake
    /// one is put to sleep and starts to fade.
    pub fn add(&mut self, doll: Ragdoll, cap: usize) {
        self.dolls.push(doll);
        let awake = self.dolls.iter().filter(|d| !d.asleep).count();
        if awake > cap {
            if let Some(oldest) = self.dolls.iter_mut().filter(|d| !d.asleep).max_by_key(|d| d.age) {
                oldest.asleep = true;
                oldest.fade = Some(FADE_TICKS);
            }
        }
    }

    pub fn awake(&self) -> usize {
        self.dolls.iter().filter(|d| !d.asleep).count()
    }

    pub fn step(&mut self, arena: &Arena) {
        for d in self.dolls.iter_mut() {
            d.step(arena);
            if d.asleep && d.fade.is_none() && d.age > LINGER {
                d.fade = Some(FADE_TICKS);
            }
        }
        self.dolls.retain(|d| d.fade != Some(0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doll() -> Ragdoll {
        Ragdoll::from_body(v3(0, ONE, 0), 0, 2 * ONE, V3::ZERO, V3::ZERO, 0)
    }

    #[test]
    fn a_stick_correction_keeps_the_pairs_weighted_centre() {
        let mut d = doll();
        // stretch the pelvis–chest stick and satisfy it once by hand-run pass
        d.points[CHEST].pos.y += ONE / 2;
        let w = |d: &Ragdoll| {
            let (a, b) = (&d.points[PELVIS], &d.points[CHEST]);
            (a.pos.y as i64 * a.weight as i64 + b.pos.y as i64 * b.weight as i64)
                / (a.weight + b.weight) as i64
        };
        let before = w(&d);
        let s = d.sticks[0].clone();
        let (a, b) = (s.a as usize, s.b as usize);
        let dv = d.points[b].pos.sub(d.points[a].pos);
        let l = dv.len();
        let fix = dv.ratio(l - s.rest, l);
        let move_a = fix.ratio(d.points[b].weight, d.points[a].weight + d.points[b].weight);
        d.points[a].pos = d.points[a].pos.add(move_a);
        d.points[b].pos = d.points[b].pos.sub(fix.sub(move_a));
        assert!((w(&d) - before).abs() <= 2, "{} vs {before}", w(&d));
        assert!((d.points[CHEST].pos.sub(d.points[PELVIS].pos).len() - s.rest).abs() <= 2);
    }

    #[test]
    fn a_kicked_ragdoll_falls_settles_and_sleeps() {
        let a = Arena::new();
        let mut d = Ragdoll::from_body(v3(0, ONE, 0), 0, 2 * ONE, V3::ZERO, v3(0, ONE / 20, ONE / 10), 0);
        for _ in 0..600 {
            d.step(&a);
        }
        assert!(d.asleep, "still moving after ten seconds");
        assert!(d.points.iter().all(|p| p.pos.y >= POINT_RADIUS));
        // it toppled: the head is low now
        assert!(d.points[HEAD].pos.y < ONE, "head at {}", d.points[HEAD].pos.y);
        // and every stick is within 5 cm of its rest length
        for s in &d.sticks {
            let l = d.points[s.b as usize].pos.sub(d.points[s.a as usize].pos).len();
            assert!(
                (l - s.rest).abs() < ONE / 20,
                "stick {}-{} at {l} vs {}",
                s.a,
                s.b,
                s.rest
            );
        }
    }

    #[test]
    fn a_ragdoll_lying_flat_stays_put_while_awake() {
        // lay the skeleton on the ground: height becomes depth
        let a = Arena::new();
        let mut d = doll();
        for p in d.points.iter_mut() {
            p.pos = v3(p.pos.x, POINT_RADIUS, p.pos.y);
            p.prev = p.pos;
        }
        let at: Vec<V3> = d.points.iter().map(|p| p.pos).collect();
        for _ in 0..120 {
            d.asleep = false; // keep the solver running
            d.step(&a);
        }
        for (p, q) in d.points.iter().zip(at) {
            assert!(p.pos.sub(q).len() < ONE / 1000, "drifted {:?}", p.pos.sub(q));
        }
    }

    #[test]
    fn a_ragdoll_sliding_on_the_ground_stops_within_a_metre() {
        // lying flat, moving sideways at 6 m/s: friction must stop it fast.
        // Without friction the 1% damping a tick alone lets it run about 10 m.
        let a = Arena::new();
        let mut d = doll();
        for p in d.points.iter_mut() {
            p.pos = v3(p.pos.x, POINT_RADIUS, p.pos.y);
            p.prev = p.pos.sub(v3(ONE / 10, 0, 0));
        }
        let start = d.centre();
        for _ in 0..300 {
            d.step(&a);
        }
        let slid = d.centre().sub(start).flat_len();
        assert!(slid < ONE, "slid {slid}");
    }

    #[test]
    fn past_the_cap_the_oldest_awake_ragdoll_sleeps_and_fades() {
        let mut pile = Pile::default();
        for i in 0..5 {
            let mut d = doll();
            d.age = 100 - i;
            pile.add(d, 4);
        }
        assert_eq!(pile.awake(), 4);
        assert_eq!(
            pile.dolls[0].fade,
            Some(FADE_TICKS),
            "the oldest was put to sleep"
        );
        let a = Arena::new();
        for _ in 0..FADE_TICKS {
            pile.step(&a);
        }
        assert_eq!(pile.dolls.len(), 4, "the faded one is gone");
    }
}

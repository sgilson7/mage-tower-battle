//! Living bodies on a battlefield: each is a sphere with a velocity, a mass
//! and a lean. They walk toward a target, push each other apart, stay out
//! of blocked cells, fall under gravity, and take knockback as an impulse
//! over their mass. Lean and spin answer acceleration and hits through a
//! damped spring, after the drones in Raiders of the Lost Mart (drone.rs
//! `step_body`). A body that dies or takes a heavy hit becomes a ragdoll
//! (`ragdoll.rs`).
//!
//! Velocities are in metres a tick, so integration is one addition.

use serde::{Deserialize, Serialize};

use crate::arena::{self, Arena, Cell};
use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, V3};

/// Gravity in metres a tick a tick: 9.8 m/s² / 60².
pub const GRAVITY: Fx = ONE * 98 / 36000;
/// Neighbour grid cell, 2 m, coarser than the arena's cells.
pub const BUCKET: Fx = 2 * ONE;
pub const BUCKETS: i32 = 26;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Body {
    pub pos: V3,
    pub prev: V3,
    pub vel: V3,
    pub radius: Fx,
    /// Mass in kilograms, as a whole number.
    pub mass: i32,
    /// Heading in turns.
    pub yaw: i32,
    /// Lean forward and sideways in turns, and their speeds.
    pub lean: (i32, i32),
    pub lean_v: (i32, i32),
    /// Tumble from hits, in turns a tick; decays.
    pub spin: i32,
    pub grounded: bool,
}

impl Body {
    pub fn new(pos: V3, radius: Fx, mass: i32) -> Body {
        Body {
            pos,
            prev: pos,
            vel: V3::ZERO,
            radius,
            mass: mass.max(1),
            yaw: 0,
            lean: (0, 0),
            lean_v: (0, 0),
            spin: 0,
            grounded: true,
        }
    }

    /// Adds an impulse (kilogram metres a tick) at an offset from the
    /// centre: the velocity changes by impulse / mass, and the offset adds
    /// tumble.
    pub fn strike(&mut self, impulse: V3, offset: V3) {
        let dv = impulse.ratio(1, self.mass);
        self.vel = self.vel.add(dv);
        let torque = offset.cross(impulse);
        self.spin = (self.spin + (torque.y / self.mass.max(1)) / 4).clamp(-2000, 2000);
        self.lean_v.0 = (self.lean_v.0 + (torque.x / self.mass.max(1)) / 8).clamp(-3000, 3000);
        self.lean_v.1 = (self.lean_v.1 + (torque.z / self.mass.max(1)) / 8).clamp(-3000, 3000);
        if dv.y > 0 {
            self.grounded = false;
        }
    }

    /// The speed of the strike in metres a tick, used to decide whether it
    /// throws the body into a ragdoll.
    pub fn speed(&self) -> Fx {
        self.vel.len()
    }

    /// One tick of motion toward `want` (a velocity in metres a tick,
    /// horizontal), with `accel` the most it can change in a tick.
    pub fn step(&mut self, want: V3, accel: Fx) {
        self.prev = self.pos;
        let old = self.vel;
        if self.grounded {
            let d = want.flat().sub(self.vel.flat());
            let dl = d.flat_len();
            let push = if dl > accel { d.with_len(accel) } else { d };
            self.vel = self.vel.add(push.flat());
            if want.flat_len() > ONE / 400 {
                let target = crate::geom::yaw_of(want.x, want.z);
                let diff = wrap(target - self.yaw);
                self.yaw = wrap_turn(self.yaw + diff.clamp(-900, 900));
            }
        } else {
            self.vel.y -= GRAVITY;
        }
        self.pos = self.pos.add(self.vel);
        if self.pos.y <= self.radius {
            self.pos.y = self.radius;
            if self.vel.y < 0 {
                self.vel.y = -self.vel.y / 4;
                if self.vel.y < GRAVITY * 4 {
                    self.vel.y = 0;
                    self.grounded = true;
                }
            }
            // ground friction on what the walker is not driving
            if !self.grounded {
                self.vel.x = self.vel.x * 9 / 10;
                self.vel.z = self.vel.z * 9 / 10;
            }
        } else if self.pos.y > self.radius + ONE / 50 {
            self.grounded = false;
        }
        // lean toward the change in velocity, through a damped spring
        let acc = self.vel.sub(old).yaw(-self.yaw);
        let target = ((acc.z * 40).clamp(-6000, 6000), (-acc.x * 40).clamp(-6000, 6000));
        self.lean_v.0 = (self.lean_v.0 + (target.0 - self.lean.0) / 6) * 82 / 100;
        self.lean_v.1 = (self.lean_v.1 + (target.1 - self.lean.1) / 6) * 82 / 100;
        self.lean.0 += self.lean_v.0 / 4;
        self.lean.1 += self.lean_v.1 / 4;
        self.yaw = wrap_turn(self.yaw + self.spin);
        self.spin = self.spin * 93 / 100;
    }
}

pub fn wrap(a: i32) -> i32 {
    let t = fx::TURN;
    let m = a.rem_euclid(t);
    if m > t / 2 {
        m - t
    } else {
        m
    }
}
pub fn wrap_turn(a: i32) -> i32 {
    a.rem_euclid(fx::TURN)
}

/// Pushes a body out of blocked cells and off the grid's edges.
pub fn keep_out(b: &mut Body, a: &Arena) {
    let half = arena::SIZE * arena::CELL / 2;
    let lim = half - b.radius;
    b.pos.x = b.pos.x.clamp(-lim, lim);
    b.pos.z = b.pos.z.clamp(-lim, lim);
    let (cx, cz) = arena::cell_of(b.pos);
    for dz in -1..=1 {
        for dx in -1..=1 {
            let (x, z) = (cx + dx, cz + dz);
            let blocked = match a.at(x, z) {
                Cell::Open => false,
                Cell::Tower => true,
                _ => true,
            };
            if !blocked || !arena::inside(x, z) {
                continue;
            }
            let c = arena::centre(x, z);
            let h = arena::CELL / 2;
            // nearest point of the cell's square to the body
            let nx = b.pos.x.clamp(c.x - h, c.x + h);
            let nz = b.pos.z.clamp(c.z - h, c.z + h);
            let d = v3(b.pos.x - nx, 0, b.pos.z - nz);
            let l = d.flat_len();
            if l >= b.radius {
                continue;
            }
            if l == 0 {
                // centre inside the square: push out along the shorter side
                let (ex, ez) = (h - (b.pos.x - c.x).abs(), h - (b.pos.z - c.z).abs());
                if ex < ez {
                    b.pos.x += if b.pos.x >= c.x {
                        ex + b.radius
                    } else {
                        -(ex + b.radius)
                    };
                    b.vel.x = 0;
                } else {
                    b.pos.z += if b.pos.z >= c.z {
                        ez + b.radius
                    } else {
                        -(ez + b.radius)
                    };
                    b.vel.z = 0;
                }
                continue;
            }
            let out = d.with_len(b.radius - l);
            b.pos = b.pos.add(out);
            // remove the velocity into the wall
            let n = d.with_len(ONE);
            let into = n.dot(b.vel) as Fx;
            if into < 0 {
                b.vel = b.vel.sub(n.scale(into));
            }
        }
    }
    // a push out of a wall beside the edge stays on the grid (T58)
    b.pos.x = b.pos.x.clamp(-lim, lim);
    b.pos.z = b.pos.z.clamp(-lim, lim);
}

/// A uniform grid of body indices, rebuilt each tick (Galactic Survivors'
/// `rebuild_cells`), so separation looks only at near neighbours.
#[derive(Debug, Clone, Default)]
pub struct Buckets {
    pub cells: Vec<Vec<u16>>,
}

fn bucket_of(p: V3) -> (i32, i32) {
    let half = BUCKETS * BUCKET / 2;
    let c = |v: Fx| ((v + half).div_euclid(BUCKET)).clamp(0, BUCKETS - 1);
    (c(p.x), c(p.z))
}

impl Buckets {
    pub fn rebuild(&mut self, bodies: &[Body]) {
        if self.cells.len() != (BUCKETS * BUCKETS) as usize {
            self.cells = vec![Vec::new(); (BUCKETS * BUCKETS) as usize];
        }
        for c in self.cells.iter_mut() {
            c.clear();
        }
        for (i, b) in bodies.iter().enumerate() {
            let (x, z) = bucket_of(b.pos);
            self.cells[(z * BUCKETS + x) as usize].push(i as u16);
        }
    }

    /// Every index in the buckets around a point.
    pub fn near(&self, p: V3, out: &mut Vec<u16>) {
        out.clear();
        let (cx, cz) = bucket_of(p);
        for z in (cz - 1).max(0)..=(cz + 1).min(BUCKETS - 1) {
            for x in (cx - 1).max(0)..=(cx + 1).min(BUCKETS - 1) {
                out.extend_from_slice(&self.cells[(z * BUCKETS + x) as usize]);
            }
        }
    }
}

/// Pushes overlapping bodies apart, the lighter one further, in index
/// order so the result is the same on every machine.
pub fn separate(bodies: &mut [Body], buckets: &Buckets, scratch: &mut Vec<u16>) {
    for i in 0..bodies.len() {
        buckets.near(bodies[i].pos, scratch);
        for &j in scratch.iter() {
            let j = j as usize;
            if j <= i {
                continue;
            }
            let (a, b) = (&bodies[i], &bodies[j]);
            let d = b.pos.sub(a.pos).flat();
            let min = a.radius + b.radius;
            let l = d.flat_len();
            if l >= min {
                continue;
            }
            let overlap = min - l;
            let n = if l == 0 { v3(ONE, 0, 0) } else { d.with_len(ONE) };
            let (ma, mb) = (a.mass as i64, b.mass as i64);
            let share_a = (overlap as i64 * mb / (ma + mb)) as Fx;
            let share_b = overlap - share_a;
            bodies[i].pos = bodies[i].pos.sub(n.scale(share_a));
            bodies[j].pos = bodies[j].pos.add(n.scale(share_b));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_body_falls_and_comes_to_rest_on_the_ground() {
        let mut b = Body::new(v3(0, 3 * ONE, 0), ONE / 2, 50);
        b.grounded = false;
        for _ in 0..600 {
            b.step(V3::ZERO, 0);
        }
        assert_eq!(b.pos.y, ONE / 2);
        assert!(b.grounded);
        assert_eq!(b.vel.y, 0);
    }

    #[test]
    fn an_impulse_changes_velocity_by_impulse_over_mass() {
        let mut b = Body::new(v3(0, ONE / 2, 0), ONE / 2, 40);
        // 40 kg struck with 8 kg·m/tick: 0.2 m/tick
        b.strike(v3(8 * ONE, 0, 0), V3::ZERO);
        assert_eq!(b.vel.x, ONE / 5);
        let mut heavy = Body::new(v3(0, ONE / 2, 0), ONE / 2, 400);
        heavy.strike(v3(8 * ONE, 0, 0), V3::ZERO);
        assert_eq!(heavy.vel.x, ONE / 50, "ten times the mass, a tenth of the push");
    }

    #[test]
    fn a_walker_reaches_its_wanted_speed_at_its_acceleration() {
        let mut b = Body::new(v3(0, ONE / 2, 0), ONE / 2, 50);
        let want = v3(ONE / 20, 0, 0);
        b.step(want, ONE / 100);
        assert_eq!(b.vel.x, ONE / 100);
        for _ in 0..20 {
            b.step(want, ONE / 100);
        }
        assert_eq!(b.vel.x, ONE / 20);
    }

    #[test]
    fn a_body_pushed_out_of_a_wall_on_the_edge_stays_on_the_grid() {
        // a wall in the field's last row; a body in the outer half of its
        // cell is pushed out the short way, which is off the grid (T58)
        let mut a = Arena::new();
        a.block(&[(5, arena::SIZE - 1)], 0).unwrap();
        let c = arena::centre(5, arena::SIZE - 1);
        let r = ONE * 2 / 5;
        let mut b = Body::new(v3(c.x, r, c.z + arena::CELL * 3 / 10), r, 100);
        keep_out(&mut b, &a);
        let half = arena::SIZE * arena::CELL / 2;
        assert!(b.pos.z <= half - r, "{} past {}", b.pos.z, half - r);
    }

    #[test]
    fn a_body_at_top_speed_does_not_cross_a_wall() {
        let mut a = Arena::new();
        let wall: Vec<(i32, i32)> = (0..20).map(|z| (16, z)).collect();
        a.block(&wall, 0).unwrap();
        let start = arena::centre(14, 10);
        let mut b = Body::new(v3(start.x, ONE / 2, start.z), ONE / 2, 50);
        for _ in 0..240 {
            b.step(v3(ONE / 8, 0, 0), ONE);
            keep_out(&mut b, &a);
            assert!(
                arena::cell_of(b.pos).0 < 16,
                "crossed into the wall at {:?}",
                b.pos
            );
        }
    }

    #[test]
    fn two_overlapping_bodies_part_with_the_lighter_moving_further() {
        let mut bodies = vec![
            Body::new(v3(0, ONE / 2, 0), ONE / 2, 300),
            Body::new(v3(ONE / 2, ONE / 2, 0), ONE / 2, 100),
        ];
        let mut buckets = Buckets::default();
        buckets.rebuild(&bodies);
        let mut scratch = Vec::new();
        separate(&mut bodies, &buckets, &mut scratch);
        // overlap 0.5 m, split 1:3 by mass
        assert_eq!(bodies[0].pos.x, -(ONE / 8));
        assert_eq!(bodies[1].pos.x, ONE / 2 + ONE * 3 / 8);
    }
}

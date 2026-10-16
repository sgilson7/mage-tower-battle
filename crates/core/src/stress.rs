//! The physics probe's scenario (PLAN.md decision 2): four arenas, each
//! with a maze of walls, a crowd of walkers following the flow field,
//! regular knockback, and kills that turn walkers into ragdolls under the
//! cap. It measures what the battle's physics will cost before any content
//! exists. `tests/physics_probe.rs` runs it natively; the shim's `Stress`
//! runs it in the browser for `tools/probe_wasm.py`.

use crate::arena::{self, Arena};
use crate::body::{self, Body, Buckets};
use crate::fx::{Fx, ONE};
use crate::geom::{v3, V3};
use crate::ragdoll::{Pile, Ragdoll};
use crate::rng::Rng;

pub struct Field {
    pub arena: Arena,
    pub bodies: Vec<Body>,
    pub pile: Pile,
    buckets: Buckets,
    scratch: Vec<u16>,
}

pub struct Stress {
    pub fields: Vec<Field>,
    pub rng: Rng,
    pub tick: u32,
    pub crowd: usize,
    pub cap: usize,
    pub deaths: u32,
    /// Walkers turned into ragdolls by each third blast.
    pub burst: usize,
}

fn maze(a: &mut Arena, k: i32) {
    // three staggered walls with gaps, different in each arena
    for (row, gap) in [(8, 4 + k), (16, 26 - k), (24, 6 + 2 * k)] {
        let cells: Vec<(i32, i32)> = (0..arena::SIZE)
            .filter(|x| (x - gap).abs() > 1)
            .map(|x| (x, row))
            .collect();
        let _ = a.block(&cells, row as u16);
    }
}

fn spawn(a: &Arena, rng: &mut Rng) -> Body {
    let e = arena::centre(a.entry.0, a.entry.1);
    let jitter = |r: &mut Rng| r.between(-(ONE * 3 / 2), ONE * 3 / 2);
    let radius = ONE * 2 / 5 + rng.between(0, ONE / 5);
    let mut b = Body::new(
        v3(e.x + jitter(rng), radius, e.z + jitter(rng)),
        radius,
        40 + rng.between(0, 200),
    );
    b.yaw = rng.between(0, crate::fx::TURN - 1);
    b
}

impl Stress {
    pub fn new(seed: u64, crowd: usize, cap: usize) -> Stress {
        let mut rng = Rng::new(seed);
        let fields = (0..4)
            .map(|k| {
                let mut arena = Arena::new();
                maze(&mut arena, k);
                let bodies = (0..crowd).map(|_| spawn(&arena, &mut rng)).collect();
                Field {
                    arena,
                    bodies,
                    pile: Pile::default(),
                    buckets: Buckets::default(),
                    scratch: Vec::new(),
                }
            })
            .collect();
        Stress {
            fields,
            rng,
            tick: 0,
            crowd,
            cap,
            deaths: 0,
            burst: 1,
        }
    }

    /// A probe world with `burst` walkers made ragdolls by each third blast.
    pub fn with_burst(seed: u64, crowd: usize, cap: usize, burst: usize) -> Stress {
        let mut s = Stress::new(seed, crowd, cap);
        s.burst = burst;
        s
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        let speed: Fx = ONE / 30; // 2 m/s
        for f in self.fields.iter_mut() {
            for b in f.bodies.iter_mut() {
                let to = f.arena.next_step(b.pos).sub(b.pos).flat();
                let want = to.with_len(speed);
                b.step(want, ONE / 200);
            }
            // separation first, then walls: walls get the last word (R3.7)
            f.buckets.rebuild(&f.bodies);
            body::separate(&mut f.bodies, &f.buckets, &mut f.scratch);
            for b in f.bodies.iter_mut() {
                body::keep_out(b, &f.arena);
            }
            // a blast every 10 ticks knocks a few walkers; every third throws one into a ragdoll
            if self.tick.is_multiple_of(10) && !f.bodies.is_empty() {
                let i = self.rng.below(f.bodies.len() as u32) as usize;
                let at = f.bodies[i].pos;
                for b in f.bodies.iter_mut() {
                    let d = b.pos.sub(at).flat();
                    if d.flat_len() < 3 * ONE {
                        b.strike(d.with_len(ONE * 4).add(v3(0, ONE * 3, 0)), v3(0, ONE / 2, 0));
                    }
                }
                if self.tick.is_multiple_of(30) {
                    for _ in 0..self.burst.min(f.bodies.len()) {
                        let k = self.rng.below(f.bodies.len() as u32) as usize;
                        let b = f.bodies.swap_remove(k);
                        let kick = v3(0, ONE / 30, ONE / 15).yaw(b.yaw);
                        f.pile.add(
                            Ragdoll::from_body(b.pos, b.yaw, b.radius * 4, b.vel, kick, 0),
                            self.cap,
                        );
                        self.deaths += 1;
                    }
                }
            }
            // walkers that reach the tower start again at the entry
            for b in f.bodies.iter_mut() {
                let (x, z) = arena::cell_of(b.pos);
                if f.arena.dist[arena::idx(x, z)] <= 1 {
                    *b = spawn(&f.arena, &mut self.rng);
                }
            }
            while f.bodies.len() < self.crowd {
                let b = spawn(&f.arena, &mut self.rng);
                f.bodies.push(b);
            }
            f.pile.step(&f.arena);
        }
    }

    /// An FNV-1a hash of every position, so two runs can be compared.
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: i32| {
            for byte in v.to_le_bytes() {
                h ^= byte as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
        };
        for f in &self.fields {
            for b in &f.bodies {
                eat(b.pos.x);
                eat(b.pos.y);
                eat(b.pos.z);
            }
            for d in &f.pile.dolls {
                for p in &d.points {
                    eat(p.pos.x);
                    eat(p.pos.y);
                    eat(p.pos.z);
                }
            }
        }
        h
    }

    pub fn awake_ragdolls(&self) -> usize {
        self.fields.iter().map(|f| f.pile.awake()).sum()
    }
}

/// A point for tests that is not at the origin.
pub fn sample_point() -> V3 {
    v3(ONE, 0, ONE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_world_and_a_copy_continues_alike() {
        let mut a = Stress::new(9, 40, 8);
        let mut b = Stress::new(9, 40, 8);
        for _ in 0..600 {
            a.step();
            b.step();
        }
        assert_eq!(a.hash(), b.hash());
        let mut c = Stress::new(10, 40, 8);
        for _ in 0..600 {
            c.step();
        }
        assert_ne!(a.hash(), c.hash(), "a different seed should change the world");
    }

    #[test]
    fn ragdolls_stay_under_the_cap_and_walkers_stay_on_the_grid() {
        let mut s = Stress::new(3, 60, 6);
        for _ in 0..1200 {
            s.step();
            for f in &s.fields {
                assert!(f.pile.awake() <= 6);
                for b in &f.bodies {
                    let half = arena::SIZE * arena::CELL / 2;
                    assert!(b.pos.x.abs() <= half && b.pos.z.abs() <= half);
                    assert!(b.pos.y >= b.radius - 1);
                }
            }
        }
        assert!(s.deaths > 100);
    }
}

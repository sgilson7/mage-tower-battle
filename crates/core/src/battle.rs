//! One tower's battle: a pure function of the owner's plan, the invasions
//! sent at it, the day's wild enemies and a seed. Nobody gives input while
//! it runs, so every browser can run every battle and get the same result.
//!
//! Each tick: spawn; students cast; living units decide and move; bodies
//! are separated and then kept out of walls (walls get the last word,
//! SECOND-ORDER R3.7); shots fly and hit; fields and statuses tick; the
//! dead become ragdolls; temporary walls fall.
//!
//! Spells cast by the rig model (spell.rs): a payload makes shots, bursts,
//! fields, hexes, summons, walls or wards; a trigger after it hands the
//! rest of the chain to each thing it made, once per thing, never once per
//! hit (Galactic Survivors R3.14); one payload budget is shared by a whole
//! cast, nested triggers included (fixing GS R4.20); things that end are
//! marked and removed after the walk that ended them (GS R4.10).

use serde::{Deserialize, Serialize};

use crate::arena::{self, Arena, Cell};
use crate::body::{self, Body, Buckets};
use crate::content::{self, Arrange, Content, Effect, Field, On, PieceKind, Status};
use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, TPS, V3};
use crate::ragdoll::{Pile, Ragdoll};
use crate::rng::Rng;

pub const MAX_UNITS: usize = 400;
pub const MAX_POPS: usize = 48;
pub const WARD_CAP: i32 = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Team {
    Foe,
    Ally,
}

/// A student as the battle sees them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentPlan {
    pub mana_max: i32,
    /// Mana a second.
    pub regen: i32,
    /// (index into the plan's spells, effectiveness in percent)
    pub spells: Vec<(u16, i32)>,
    /// The school the student leans to, for the colour of their robe.
    #[serde(default)]
    pub school: u8,
}

/// A thing the archmage placed before the battle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placed {
    pub feature: u16,
    pub cells: Vec<(i32, i32)>,
}

/// Everything a player brings to their battle.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Plan {
    pub spells: Vec<Vec<u16>>,
    pub students: Vec<StudentPlan>,
    pub placed: Vec<Placed>,
}

/// Enemies sent at this tower by a rival.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incoming {
    pub from: u8,
    pub chain: Vec<u16>,
    /// Effectiveness of whoever sent it, percent: scales health.
    pub strength: i32,
    /// Percent of the chain's summon counts: 100 for one caster, more for
    /// a class casting together.
    pub count_pct: i32,
}

/// A group of the day's wild enemies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wild {
    pub enemy: u16,
    pub count: i32,
    pub at_tick: i32,
    /// Percent of the enemy's health, from the difficulty (T30).
    #[serde(default = "full")]
    pub strength: i32,
}

fn full() -> i32 {
    100
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Carry {
    pub on: On,
    pub rest: Vec<u16>,
    pub depth: u8,
    pub eff: i32,
    pub team: Team,
    pub cast: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub id: u32,
    pub kind: u16,
    pub body: Body,
    pub hp: i32,
    pub hp_max: i32,
    pub team: Team,
    /// The team it returns to when charm ends.
    pub home: Team,
    pub freeze: i32,
    pub charm: i32,
    pub slow: i32,
    pub fear: i32,
    pub last: [i32; 4],
    pub attack_cd: i32,
    pub flash: u8,
    pub stuck: u16,
    pub carry: Option<Carry>,
    /// Who sent it: a seat for an invasion, 255 for the day's wild enemies
    /// or the owner's own summons.
    pub from: u8,
    pub summoned_for_owner: bool,
    /// It reached the tower and was spent: no kill.
    pub leaked: bool,
    /// The student who summoned it (255 for none), and the ticks it has
    /// left (0 for no limit): a summon is held to its caster (T25).
    pub caster: u8,
    pub life: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shot {
    pub pos: V3,
    pub prev: V3,
    pub vel: V3,
    pub life: i32,
    pub damage: i32,
    pub dtype: u8,
    pub radius: Fx,
    pub pierce: i32,
    pub knock: i32,
    pub hit: Vec<u32>,
    pub team: Team,
    pub carry: Option<Carry>,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    pub pos: V3,
    pub radius: Fx,
    pub life: i32,
    pub age: i32,
    pub field: Field,
    pub dps: i32,
    pub dtype: u8,
    pub slow: i32,
    pub team: Team,
    pub carry: Option<Carry>,
    pub entered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lure {
    pub pos: V3,
    pub hp: i32,
    pub life: i32,
    pub radius: Fx,
}

/// A feature standing on the field, permanent or for a while.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Standing {
    pub feature: u16,
    pub cells: Vec<(i32, i32)>,
    pub hp: i32,
    /// Ticks left, or -1 for the whole battle.
    pub life: i32,
    pub id: u16,
}

/// A short-lived picture: a burst ring, a damage number, a beam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pop {
    pub pos: V3,
    pub value: i32,
    /// 50, 100 or 200: the type rate the hit used.
    pub rate: i32,
    pub life: i32,
    pub kind: PopKind,
    pub dtype: u8,
    pub radius: Fx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopKind {
    Number,
    Ring,
    Freeze,
    Charm,
}

/// What happened this tick, for sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ev {
    Cast(u8),
    Hit,
    Strong,
    Weak,
    Kill,
    Boom,
    TowerHit,
    Freeze,
    Charm,
    Summon,
    WallUp,
    WallDown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Tally {
    pub kills: i32,
    pub frozen: i32,
    pub charmed: i32,
    pub giants_charmed: i32,
    pub damage_taken: i32,
    pub warded: i32,
    pub biggest_hit: i32,
    /// Damage this tower took from each seat's invasions.
    pub taken_from: [i32; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Caster {
    pub plan: StudentPlan,
    pub pos: V3,
    pub range_pct: i32,
    /// Mana in hundredths.
    pub mana: i32,
    pub cd: Vec<i32>,
    pub aim: V3,
    pub casting: u8,
}

#[derive(Debug, Clone)]
pub struct Battle {
    pub owner: u8,
    pub tick: i32,
    pub ticks: i32,
    pub arena: Arena,
    pub units: Vec<Unit>,
    pub shots: Vec<Shot>,
    pub zones: Vec<Zone>,
    pub lures: Vec<Lure>,
    pub standing: Vec<Standing>,
    pub casters: Vec<Caster>,
    pub pile: Pile,
    pub pops: Vec<Pop>,
    pub spells: Vec<Vec<u16>>,
    spell_cost: Vec<i32>,
    spell_cd: Vec<i32>,
    queue: Vec<(i32, u16, Team, u8, Option<Carry>, i32)>,
    pub ward: i32,
    pub tally: Tally,
    pub events: Vec<(Ev, V3)>,
    budgets: Vec<i32>,
    /// The student who began each cast (255 for an invasion), and the cast
    /// being resolved now.
    cast_caster: Vec<u8>,
    cur_cast: u32,
    next_id: u32,
    rng: Rng,
    cap: usize,
    /// Percent applied to summon counts while an incoming chain is cast.
    count_pct: i32,
    buckets: Buckets,
    scratch: Vec<u16>,
}

fn pct(v: i32, p: i32) -> i32 {
    (v as i64 * p as i64 / 100) as i32
}

/// Where the tower's seats are: a ring of offshoots at three heights.
pub fn seat_pos(tower: V3, i: usize) -> V3 {
    let a = (i as i32 * fx::TURN / 12) + fx::TURN / 24;
    let r = ONE * 34 / 10;
    let y = 4 * ONE + (i as i32 % 3) * (ONE * 6 / 10);
    v3(
        tower.x + fx::mul(fx::sin(a), r),
        y,
        tower.z + fx::mul(fx::cos(a), r),
    )
}

/// Where students without a seat stand: a ring round the tower's foot.
pub fn ground_pos(tower: V3, i: usize) -> V3 {
    let a = (i as i32 * fx::TURN * 7 / 40) % fx::TURN;
    let r = ONE * 4 + (i as i32 / 20) * ONE;
    v3(
        tower.x + fx::mul(fx::sin(a), r),
        ONE / 2,
        tower.z + fx::mul(fx::cos(a), r),
    )
}

/// A tower's field: the open grid with trees and rocks the seed places,
/// never near the entry or the tower and never closing the route. The
/// layout editor and the battle both start from it.
pub fn field(seed: u64) -> Arena {
    let mut arena = Arena::new();
    let mut rng = Rng::new(seed);
    for _ in 0..18 {
        let (x, z) = (rng.between(0, arena::SIZE - 1), rng.between(0, arena::SIZE - 1));
        let near_ends = (x - arena.entry.0).abs() + (z - arena.entry.1).abs() < 5
            || (x - arena.tower.0).abs() + (z - arena.tower.1).abs() < 6;
        if !near_ends && arena.at(x, z) == Cell::Open {
            arena.cells[arena::idx(x, z)] = Cell::Scenery;
            arena.recompute();
            if !arena.reachable() {
                arena.cells[arena::idx(x, z)] = Cell::Open;
                arena.recompute();
            }
        }
    }
    arena
}

pub fn tower_centre(a: &Arena) -> V3 {
    arena::centre(a.tower.0, a.tower.1)
}

impl Battle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c: &Content,
        owner: u8,
        plan: &Plan,
        incoming: &[Incoming],
        wild: &[Wild],
        seed: u64,
        field_seed: u64,
        ticks: i32,
        cap: usize,
    ) -> Battle {
        let arena = field(field_seed);
        let rng = Rng::new(seed);
        let mut b = Battle {
            owner,
            tick: 0,
            ticks,
            arena,
            units: Vec::new(),
            shots: Vec::new(),
            zones: Vec::new(),
            lures: Vec::new(),
            standing: Vec::new(),
            casters: Vec::new(),
            pile: Pile::default(),
            pops: Vec::new(),
            spells: plan.spells.clone(),
            spell_cost: plan
                .spells
                .iter()
                .map(|s| crate::spell::stats(c, s).mana)
                .collect(),
            spell_cd: plan
                .spells
                .iter()
                .map(|s| content::ticks(crate::spell::stats(c, s).cooldown_ms))
                .collect(),
            queue: Vec::new(),
            ward: 0,
            tally: Tally::default(),
            events: Vec::new(),
            budgets: Vec::new(),
            cast_caster: Vec::new(),
            cur_cast: 0,
            next_id: 1,
            rng,
            cap,
            count_pct: 100,
            buckets: Buckets::default(),
            scratch: Vec::new(),
        };
        // the archmage's placements, in order; one that would close the path is skipped
        let seats = c.balance().tower_seats.max(0) as usize;
        let mut sub_seats: Vec<V3> = Vec::new();
        for (k, p) in plan.placed.iter().enumerate() {
            let f = &c.file.features[p.feature as usize];
            b.raise(c, p.feature, &p.cells, -1, k as u16);
            if f.seats > 0 && b.standing.iter().any(|s| s.id == k as u16) {
                let at = arena::centre(p.cells[0].0, p.cells[0].1);
                for _ in 0..f.seats {
                    sub_seats.push(v3(at.x, ONE * 33 / 10, at.z));
                }
            }
        }
        let tower = tower_centre(&b.arena);
        let bal = c.balance();
        for (i, s) in plan.students.iter().enumerate() {
            let (pos, range_pct) = if i < seats {
                (seat_pos(tower, i), bal.seat_range_pct)
            } else if i - seats < sub_seats.len() {
                (sub_seats[i - seats], bal.seat_range_pct)
            } else {
                (
                    ground_pos(tower, i - seats - sub_seats.len()),
                    bal.ground_range_pct,
                )
            };
            b.casters.push(Caster {
                plan: s.clone(),
                pos,
                range_pct,
                mana: s.mana_max * 100,
                cd: s
                    .spells
                    .iter()
                    .enumerate()
                    .map(|(k, _)| (k as i32 * 17 + i as i32 * 7) % 40)
                    .collect(),
                aim: pos,
                casting: 0,
            });
        }
        // the day's wild enemies, a unit every 12 ticks within a group
        for w in wild {
            for k in 0..w.count {
                b.queue
                    .push((w.at_tick + k * 12, w.enemy, Team::Foe, 255, None, w.strength));
            }
        }
        // invasions: each chain is cast from the entry when the battle starts
        b.queue.sort_by_key(|q| q.0);
        let entry = arena::centre(b.arena.entry.0, b.arena.entry.1);
        for (k, inv) in incoming.iter().enumerate() {
            let cast = b.new_budget(c, 255);
            let at = entry.add(v3(0, 0, -(k as i32 % 3) * ONE));
            b.count_pct = inv.count_pct.max(1);
            b.run(
                c,
                &inv.chain,
                0,
                at,
                at,
                inv.strength,
                Team::Foe,
                0,
                cast,
                inv.from,
            );
            b.count_pct = 100;
        }
        b
    }

    fn new_budget(&mut self, c: &Content, caster: u8) -> u32 {
        self.budgets.push(c.balance().cast_budget);
        self.cast_caster.push(caster);
        (self.budgets.len() - 1) as u32
    }

    pub fn tower(&self) -> V3 {
        tower_centre(&self.arena)
    }

    pub fn done(&self) -> bool {
        self.tick >= self.ticks
            || (self.tick > TPS * 5
                && self.queue.is_empty()
                && !self.units.iter().any(|u| u.team == Team::Foe))
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn(
        &mut self,
        c: &Content,
        kind: u16,
        at: V3,
        team: Team,
        from: u8,
        carry: Option<Carry>,
        strength: i32,
    ) {
        if self.units.len() >= MAX_UNITS {
            return;
        }
        let e = &c.file.enemies[kind as usize];
        let r = content::cm(e.radius);
        let jitter = v3(
            self.rng.between(-ONE / 2, ONE / 2),
            0,
            self.rng.between(-ONE / 2, ONE / 2),
        );
        let mut body = Body::new(v3(at.x + jitter.x, r, at.z + jitter.z), r, e.mass);
        body.yaw = self.rng.between(0, fx::TURN - 1);
        let hp = pct(e.health, strength.clamp(10, 150)).max(1);
        let id = self.next_id;
        self.next_id += 1;
        self.units.push(Unit {
            id,
            kind,
            body,
            hp,
            hp_max: hp,
            team,
            home: team,
            freeze: 0,
            charm: 0,
            slow: 0,
            fear: 0,
            last: [-10_000; 4],
            attack_cd: 30,
            flash: 0,
            stuck: 0,
            carry,
            from,
            summoned_for_owner: team == Team::Ally,
            leaked: false,
            caster: 255,
            life: 0,
        });
    }

    /// Casts a chain from `start`: origin is where it comes from, target
    /// where it is aimed.
    #[allow(clippy::too_many_arguments)]
    fn run(
        &mut self,
        c: &Content,
        chain: &[u16],
        start: usize,
        origin: V3,
        target: V3,
        eff: i32,
        team: Team,
        depth: u8,
        cast: u32,
        from: u8,
    ) {
        let bal = c.balance();
        self.cur_cast = cast;
        let mut dmg_pct = 0;
        let mut area_pct = 0;
        let mut speed_pct = 0;
        let mut pierce = 0;
        let mut form: Option<(Arrange, i32, i32)> = None;
        let mut k = start;
        while k < chain.len() {
            let piece = &c.file.pieces[chain[k] as usize];
            match &piece.kind {
                PieceKind::Modifier { mods, .. } => {
                    dmg_pct += mods.damage_pct;
                    area_pct += mods.area_pct;
                    speed_pct += mods.speed_pct;
                    pierce += mods.pierce;
                }
                PieceKind::Formation {
                    arrange,
                    copies,
                    share_pct,
                } => form = Some((*arrange, *copies, *share_pct)),
                PieceKind::Invade | PieceKind::Helper { .. } => {}
                PieceKind::Trigger { .. } => return,
                PieceKind::Payload { effect, .. } => {
                    let (arr, copies, share) = form.unwrap_or((Arrange::Twin, 1, 100));
                    let left = self.budgets.get(cast as usize).copied().unwrap_or(0);
                    let n = copies.min(left).max(0);
                    if n == 0 {
                        return;
                    }
                    self.budgets[cast as usize] -= n;
                    let carry = match chain.get(k + 1).map(|&p| &c.file.pieces[p as usize].kind) {
                        Some(PieceKind::Trigger { on }) if (depth as i32) < bal.max_depth => Some(Carry {
                            on: *on,
                            rest: chain[k + 2..].to_vec(),
                            depth: depth + 1,
                            eff,
                            team,
                            cast,
                        }),
                        _ => None,
                    };
                    let area_eff = bal.area_floor_pct + pct(100 - bal.area_floor_pct, eff);
                    for i in 0..n {
                        let (o, t) = arrange(arr, copies, i, origin, target);
                        self.payload(
                            c,
                            effect,
                            o,
                            t,
                            eff,
                            team,
                            dmg_pct,
                            area_pct + area_eff - 100,
                            speed_pct,
                            pierce,
                            share,
                            carry.clone(),
                            from,
                        );
                    }
                    if let Some(PieceKind::Trigger { .. }) =
                        chain.get(k + 1).map(|&p| &c.file.pieces[p as usize].kind)
                    {
                        return;
                    }
                    dmg_pct = 0;
                    area_pct = 0;
                    speed_pct = 0;
                    pierce = 0;
                    form = None;
                }
            }
            k += 1;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn payload(
        &mut self,
        c: &Content,
        effect: &Effect,
        origin: V3,
        target: V3,
        eff: i32,
        team: Team,
        dmg_pct: i32,
        area_pct: i32,
        speed_pct: i32,
        pierce: i32,
        share: i32,
        carry: Option<Carry>,
        from: u8,
    ) {
        let bal = c.balance();
        let dmg = |base: i32| pct(pct(pct(base, 100 + dmg_pct), share), eff).max(1);
        let area = |r: i32| content::cm(pct(r, (100 + area_pct).max(10)));
        match effect {
            Effect::Bolt {
                damage_type,
                damage,
                speed,
                range,
                radius,
                knock,
            } => {
                if self.shots.len() >= bal.max_shots as usize {
                    return;
                }
                let sp = content::cms(pct(*speed, (100 + speed_pct).max(20)));
                let dir = target.sub(origin);
                let life = (content::cm(*range) / sp.max(1)).max(1);
                self.shots.push(Shot {
                    pos: origin,
                    prev: origin,
                    vel: dir.with_len(sp),
                    life,
                    damage: dmg(*damage),
                    dtype: c.damage_type(damage_type) as u8,
                    radius: area(*radius),
                    pierce,
                    knock: *knock,
                    hit: Vec::new(),
                    team,
                    carry,
                    done: false,
                });
            }
            Effect::Burst {
                damage_type,
                damage,
                radius,
                knock,
                lift,
                ..
            } => {
                let r = area(*radius);
                let dt = c.damage_type(damage_type) as u8;
                self.pop(target, 0, 100, PopKind::Ring, dt, r);
                self.events.push((Ev::Boom, target));
                let mut hit_any: Option<V3> = None;
                let mut kills: Vec<V3> = Vec::new();
                let d = dmg(*damage);
                for i in 0..self.units.len() {
                    let u = &self.units[i];
                    if u.team == team || u.body.pos.sub(target).flat_len() > r + u.body.radius {
                        continue;
                    }
                    let away = u.body.pos.sub(target).flat().with_len(ONE);
                    // knock and lift are impulses in kg·cm a tick
                    let push = away.ratio(*knock, 100).add(v3(0, ONE / 100 * *lift, 0));
                    hit_any.get_or_insert(u.body.pos);
                    if self.hurt(c, i, d, dt, push) {
                        kills.push(self.units[i].body.pos);
                    }
                }
                self.after(c, carry, target, hit_any, &kills, from);
            }
            Effect::Ground {
                field,
                damage_type,
                dps,
                radius,
                life,
                slow_pct,
                ..
            } => {
                if self.zones.len() >= bal.max_effects as usize / 4 {
                    return;
                }
                let at = if *field == Field::ManaSpring {
                    self.tower()
                } else {
                    target.flat()
                };
                self.zones.push(Zone {
                    pos: at,
                    radius: area(*radius),
                    life: pct(content::ticks(*life), eff.max(20)),
                    age: 0,
                    field: *field,
                    dps: pct(pct(*dps, 100 + dmg_pct), eff),
                    dtype: c.damage_type(damage_type) as u8,
                    slow: *slow_pct,
                    team,
                    carry,
                    entered: false,
                });
            }
            Effect::Hex {
                status, radius, life, ..
            } => {
                let r = area(*radius);
                let mut first: Option<V3> = None;
                let ticks = pct(content::ticks(*life), eff).min(content::ticks(bal.status_cap_ms));
                for i in 0..self.units.len() {
                    let u = &self.units[i];
                    if u.home == team || u.body.pos.sub(target).flat_len() > r + u.body.radius {
                        continue;
                    }
                    first.get_or_insert(u.body.pos);
                    self.hex(c, i, *status, ticks);
                }
                self.after(c, carry, target, first, &[], from);
            }
            Effect::Summon { enemy, count } => {
                let kind = c.enemy(enemy).unwrap_or(0) as u16;
                let mut n = pct(pct(*count, eff), self.count_pct).max(1);
                // your own army is held in check: fewer a cast, a few alive
                // for each student, for a while each (T25); invasions keep
                // their numbers
                let caster = self
                    .cast_caster
                    .get(self.cur_cast as usize)
                    .copied()
                    .unwrap_or(255);
                let mut life = 0;
                if team == Team::Ally && caster != 255 {
                    n = pct(n, bal.ally_summon_pct).max(1);
                    let alive = self
                        .units
                        .iter()
                        .filter(|u| u.caster == caster && u.team == Team::Ally && u.hp > 0)
                        .count() as i32;
                    n = n.min(bal.summons_per_student - alive).max(0);
                    life = content::ticks(bal.summon_life_s * 1000);
                }
                self.events.push((Ev::Summon, origin));
                let at = if team == Team::Ally {
                    // in front of the tower, toward the entry
                    let t = self.tower();
                    let e = arena::centre(self.arena.entry.0, self.arena.entry.1);
                    t.add(e.sub(t).flat().with_len(4 * ONE))
                } else {
                    origin
                };
                for _ in 0..n {
                    let mine = carry.clone().filter(|k| k.on == On::Death);
                    self.spawn(c, kind, at, team, from, mine, eff);
                    if let Some(u) = self.units.last_mut() {
                        u.caster = caster;
                        u.life = life;
                    }
                }
            }
            Effect::Raise { feature } => {
                let f = c.feature(feature).unwrap_or(0) as u16;
                let ft = &c.file.features[f as usize];
                if ft.cast_life == 0 {
                    return;
                }
                let life = pct(content::ticks(ft.cast_life), eff.max(20));
                let cells = self.cells_across(target, ft.length);
                let id = 1000 + self.tick as u16;
                self.raise(c, f, &cells, life, id);
                self.events.push((Ev::WallUp, target));
            }
            Effect::Ward { absorb } => {
                self.ward = (self.ward + pct(*absorb, eff)).min(WARD_CAP);
            }
        }
    }

    /// Cells across the route at a point, `n` long, for a wall cast in battle.
    fn cells_across(&self, at: V3, n: i32) -> Vec<(i32, i32)> {
        let (x, z) = arena::cell_of(at);
        let next = arena::cell_of(self.arena.next_step(at));
        let (dx, dz) = (next.0 - x, next.1 - z);
        // across the walking direction
        let (ax, az) = if dx.abs() >= dz.abs() { (0, 1) } else { (1, 0) };
        (0..n)
            .map(|k| (x + ax * (k - n / 2), z + az * (k - n / 2)))
            .collect()
    }

    fn raise(&mut self, c: &Content, feature: u16, cells: &[(i32, i32)], life: i32, id: u16) {
        let f = &c.file.features[feature as usize];
        if f.blocks {
            // shorten a wall that would close the route rather than drop it
            let mut cells = cells.to_vec();
            while !cells.is_empty() {
                if self.arena.block(&cells, id).is_ok() {
                    break;
                }
                cells.pop();
            }
            if cells.is_empty() {
                return;
            }
            self.standing.push(Standing {
                feature,
                cells,
                hp: f.health,
                life,
                id,
            });
            return;
        }
        let at = cells
            .first()
            .map(|&(x, z)| arena::centre(x, z))
            .unwrap_or(V3::ZERO);
        if f.lure > 0 {
            self.lures.push(Lure {
                pos: at,
                hp: f.health.max(1),
                life,
                radius: content::cm(f.lure),
            });
        }
        if f.dps > 0 {
            self.zones.push(Zone {
                pos: at,
                radius: arena::CELL * f.length.max(1) * 2 / 3,
                life: if life < 0 { self.ticks + 1 } else { life },
                age: 0,
                field: Field::Lava,
                dps: f.dps,
                dtype: c.damage_type(&f.damage_type) as u8,
                slow: 0,
                team: Team::Ally,
                carry: None,
                entered: false,
            });
        }
        self.standing.push(Standing {
            feature,
            cells: cells.to_vec(),
            hp: f.health,
            life,
            id,
        });
    }

    fn pop(&mut self, pos: V3, value: i32, rate: i32, kind: PopKind, dtype: u8, radius: Fx) {
        if self.pops.len() >= MAX_POPS {
            self.pops.remove(0);
        }
        let life = if kind == PopKind::Number { 50 } else { 24 };
        self.pops.push(Pop {
            pos,
            value,
            rate,
            life,
            kind,
            dtype,
            radius,
        });
    }

    /// Damages a unit; returns true when it dies of it.
    fn hurt(&mut self, c: &Content, i: usize, damage: i32, dtype: u8, push: V3) -> bool {
        let kind = self.units[i].kind as usize;
        let defender = c.damage_type(&c.file.enemies[kind].damage_type);
        let rate = c.rate(dtype as usize, defender);
        let d = pct(damage, rate).max(1);
        let u = &mut self.units[i];
        if u.hp <= 0 {
            return false;
        }
        u.hp -= d;
        u.flash = 6;
        u.body.strike(push, v3(0, u.body.radius / 2, 0));
        self.tally.biggest_hit = self.tally.biggest_hit.max(d);
        let pos = u.body.pos.add(v3(0, content::cm(c.file.enemies[kind].height), 0));
        let ev = if rate > 100 {
            Ev::Strong
        } else if rate < 100 {
            Ev::Weak
        } else {
            Ev::Hit
        };
        self.events.push((ev, pos));
        if rate != 100 || d >= 40 || self.tick % 4 == 0 {
            self.pop(pos, d, rate, PopKind::Number, dtype, 0);
        }
        self.units[i].hp <= 0
    }

    fn hex(&mut self, c: &Content, i: usize, status: Status, ticks: i32) {
        let bal = c.balance();
        let slot = status as usize;
        let u = &mut self.units[i];
        // a repeat within five seconds lasts a fraction as long
        let t = if self.tick - u.last[slot] < TPS * 5 {
            pct(ticks, bal.status_repeat_pct)
        } else {
            ticks
        };
        u.last[slot] = self.tick;
        let is_giant = c.file.enemies[u.kind as usize].mass >= 300;
        match status {
            Status::Freeze => {
                u.freeze = u.freeze.max(t);
                self.tally.frozen += 1;
                self.events.push((Ev::Freeze, u.body.pos));
            }
            Status::Charm => {
                u.charm = u.charm.max(t);
                u.team = Team::Ally;
                self.tally.charmed += 1;
                if is_giant {
                    self.tally.giants_charmed += 1;
                }
                self.events.push((Ev::Charm, u.body.pos));
            }
            Status::Slow => u.slow = u.slow.max(t),
            Status::Fear => u.fear = u.fear.max(t),
        }
        let pos = self.units[i].body.pos;
        let kind = if status == Status::Charm {
            PopKind::Charm
        } else {
            PopKind::Freeze
        };
        if matches!(status, Status::Freeze | Status::Charm) {
            self.pop(pos, 0, 100, kind, 0, 0);
        }
    }

    /// Starts what a trigger carried: at the first hit, at each kill, or at
    /// the end.
    fn after(
        &mut self,
        c: &Content,
        carry: Option<Carry>,
        end: V3,
        first_hit: Option<V3>,
        kills: &[V3],
        from: u8,
    ) {
        let Some(k) = carry else { return };
        match k.on {
            On::Hit => {
                if let Some(p) = first_hit {
                    self.spring(c, &k, p, from);
                }
            }
            On::Death => {
                for &p in kills.iter().take(4) {
                    self.spring(c, &k, p, from);
                }
            }
            On::Expire => self.spring(c, &k, end, from),
        }
    }

    fn spring(&mut self, c: &Content, k: &Carry, at: V3, from: u8) {
        let target = self
            .nearest(at, k.team, 6 * ONE)
            .map(|i| self.units[i].body.pos)
            .unwrap_or(at);
        let origin = v3(at.x, at.y.max(ONE / 2), at.z);
        self.run(
            c, &k.rest, 0, origin, target, k.eff, k.team, k.depth, k.cast, from,
        );
    }

    /// The nearest unit opposing `team` within `range` of a point.
    fn nearest(&self, at: V3, team: Team, range: Fx) -> Option<usize> {
        let mut best: Option<(i64, usize)> = None;
        let r2 = range as i64 * range as i64;
        for (i, u) in self.units.iter().enumerate() {
            if u.team == team || u.hp <= 0 {
                continue;
            }
            let d = u.body.pos.sub(at).flat().len2();
            if d <= r2 && best.map(|b| d < b.0).unwrap_or(true) {
                best = Some((d, i));
            }
        }
        best.map(|b| b.1)
    }

    pub fn step(&mut self, c: &Content) {
        if self.done() {
            return;
        }
        self.tick += 1;
        self.events.clear();
        // spawns due now
        let entry = arena::centre(self.arena.entry.0, self.arena.entry.1);
        while self.queue.first().map(|q| q.0 <= self.tick).unwrap_or(false) {
            let (_, kind, team, from, carry, strength) = self.queue.remove(0);
            self.spawn(c, kind, entry, team, from, carry, strength);
        }
        self.cast_all(c);
        // a summon's time runs out: it fades, and counts as no kill
        for u in self.units.iter_mut() {
            if u.life > 0 {
                u.life -= 1;
                if u.life == 0 {
                    u.hp = 0;
                    u.leaked = true;
                }
            }
        }
        self.move_units(c);
        self.fly_shots(c);
        self.tick_zones(c);
        self.bury(c);
        self.pile.step(&self.arena);
        // temporary walls and lures
        let mut fell: Vec<u16> = Vec::new();
        for s in self.standing.iter_mut() {
            if s.life > 0 {
                s.life -= 1;
                if s.life == 0 {
                    fell.push(s.id);
                }
            }
        }
        for id in fell {
            self.arena.clear_feature(id);
            self.events.push((Ev::WallDown, V3::ZERO));
        }
        self.standing.retain(|s| s.life != 0);
        for l in self.lures.iter_mut() {
            if l.life > 0 {
                l.life -= 1;
            }
        }
        self.lures.retain(|l| l.life != 0 && l.hp > 0);
        for p in self.pops.iter_mut() {
            p.life -= 1;
        }
        self.pops.retain(|p| p.life > 0);
    }

    fn cast_all(&mut self, c: &Content) {
        for ci in 0..self.casters.len() {
            let regen = self.casters[ci].plan.regen * 100 / TPS;
            let spring_bonus = self.zones.iter().any(|z| {
                z.field == Field::ManaSpring && z.pos.sub(self.casters[ci].pos).flat_len() <= z.radius
            });
            let cap = self.casters[ci].plan.mana_max * 100;
            self.casters[ci].mana =
                (self.casters[ci].mana + regen * if spring_bonus { 2 } else { 1 }).min(cap);
            if self.casters[ci].casting > 0 {
                self.casters[ci].casting -= 1;
            }
            for k in 0..self.casters[ci].plan.spells.len() {
                if self.casters[ci].cd[k] > 0 {
                    self.casters[ci].cd[k] -= 1;
                    continue;
                }
                let (si, eff) = self.casters[ci].plan.spells[k];
                let Some(chain) = self.spells.get(si as usize).cloned() else {
                    continue;
                };
                if crate::spell::is_invasion(c, &chain) {
                    continue;
                }
                let cost = self.spell_cost[si as usize] * 100;
                if self.casters[ci].mana < cost {
                    continue;
                }
                let range = pct(spell_range(c, &chain), self.casters[ci].range_pct);
                let pos = self.casters[ci].pos;
                let needs_foe = !matches!(
                    first_effect(c, &chain),
                    Some(Effect::Ward { .. })
                        | Some(Effect::Ground {
                            field: Field::ManaSpring,
                            ..
                        })
                );
                let target = match self.nearest(pos, Team::Ally, range) {
                    Some(i) => self.units[i].body.pos.add(v3(0, ONE / 2, 0)),
                    None if !needs_foe && self.units.iter().any(|u| u.team == Team::Foe) => pos,
                    None => continue,
                };
                self.casters[ci].mana -= cost;
                self.casters[ci].cd[k] = pct(self.spell_cd[si as usize], 100).max(6);
                self.casters[ci].aim = target;
                self.casters[ci].casting = 10;
                let school = c.piece_school[chain[0] as usize] as u8;
                self.events.push((Ev::Cast(school), pos));
                let cast = self.new_budget(c, ci as u8);
                self.run(c, &chain, 0, pos, target, eff, Team::Ally, 0, cast, 255);
            }
        }
    }

    fn move_units(&mut self, c: &Content) {
        let tower = self.tower();
        let tower_reach = arena::CELL * 3 / 2;
        for i in 0..self.units.len() {
            let kind = self.units[i].kind as usize;
            let e = &c.file.enemies[kind];
            let reach = content::cm(e.reach);
            let mut speed = content::cms(e.speed);
            let u = &mut self.units[i];
            if u.flash > 0 {
                u.flash -= 1;
            }
            if u.charm > 0 {
                u.charm -= 1;
                if u.charm == 0 {
                    u.team = u.home;
                }
            }
            if u.slow > 0 {
                u.slow -= 1;
                speed /= 2;
            }
            if u.attack_cd > 0 {
                u.attack_cd -= 1;
            }
            if u.freeze > 0 {
                u.freeze -= 1;
                u.body.step(V3::ZERO, ONE / 50);
                continue;
            }
            let fear = if u.fear > 0 {
                u.fear -= 1;
                true
            } else {
                false
            };
            let pos = u.body.pos;
            let team = u.team;
            // in a slowing field
            let field_slow = self
                .zones
                .iter()
                .filter(|z| z.team != team && z.slow > 0 && z.pos.sub(pos).flat_len() <= z.radius)
                .map(|z| z.slow)
                .max()
                .unwrap_or(0);
            speed = pct(speed, 100 - field_slow);
            let mut want = V3::ZERO;
            let mut strike: Option<usize> = None;
            let mut at_tower = false;
            let mut at_lure: Option<usize> = None;
            if team == Team::Foe {
                if pos.sub(tower).flat_len() <= tower_reach + reach + self.units[i].body.radius {
                    at_tower = true;
                } else if let Some(a) = self.nearest(pos, Team::Foe, reach + ONE / 2) {
                    strike = Some(a);
                } else {
                    let lure = self
                        .lures
                        .iter()
                        .enumerate()
                        .filter(|(_, l)| l.pos.sub(pos).flat_len() <= l.radius)
                        .min_by_key(|(_, l)| l.pos.sub(pos).flat().len2())
                        .map(|(k, _)| k);
                    let goal = match lure {
                        Some(k) if self.lures[k].pos.sub(pos).flat_len() <= reach + ONE => {
                            at_lure = Some(k);
                            pos
                        }
                        Some(k) => self.lures[k].pos,
                        None => self.arena.next_step(pos),
                    };
                    let dir = goal.sub(pos).flat();
                    want = if fear {
                        dir.with_len(speed).scale(-ONE)
                    } else {
                        dir.with_len(speed)
                    };
                }
            } else {
                // allies hunt the nearest foe, or hold in front of the tower
                match self.nearest(pos, Team::Ally, 16 * ONE) {
                    Some(f)
                        if self.units[f].body.pos.sub(pos).flat_len()
                            <= reach + self.units[f].body.radius =>
                    {
                        strike = Some(f)
                    }
                    Some(f) => want = self.units[f].body.pos.sub(pos).flat().with_len(speed),
                    None => {}
                }
            }
            let u = &mut self.units[i];
            u.body.step(want, ONE / 120);
            if want != V3::ZERO && u.body.vel.flat_len() < speed / 8 {
                u.stuck = u.stuck.saturating_add(1);
            } else {
                u.stuck = 0;
            }
            if u.attack_cd == 0 && (strike.is_some() || at_tower || at_lure.is_some()) {
                u.attack_cd = content::ticks(e.attack_every);
                let dmg = e.attack;
                let from = u.from;
                let src = u.body.pos;
                if at_tower {
                    // an enemy that reaches the tower strikes it once, hard,
                    // and is spent (SECOND-ORDER R3.25)
                    let b = c.balance();
                    let dmg = dmg
                        * if (from as usize) < 4 {
                            b.invasion_leak_mult
                        } else {
                            b.leak_mult
                        };
                    self.units[i].hp = 0;
                    self.units[i].leaked = true;
                    let absorbed = dmg.min(self.ward);
                    self.ward -= absorbed;
                    self.tally.warded += absorbed;
                    let taken = dmg - absorbed;
                    self.tally.damage_taken += taken;
                    if (from as usize) < 4 {
                        self.tally.taken_from[from as usize] += taken;
                    }
                    self.events.push((Ev::TowerHit, src));
                } else if let Some(k) = at_lure {
                    self.lures[k].hp -= dmg;
                } else if let Some(t) = strike {
                    let push =
                        self.units[t]
                            .body
                            .pos
                            .sub(src)
                            .flat()
                            .with_len(ONE * 2)
                            .add(v3(0, ONE / 2, 0));
                    let dt = c.damage_type(&e.damage_type) as u8;
                    self.hurt(c, t, dmg, dt, push);
                }
            }
        }
        // separation first, then walls
        self.buckets
            .rebuild(&self.units.iter().map(|u| u.body.clone()).collect::<Vec<_>>());
        let mut bodies: Vec<Body> = self.units.iter().map(|u| u.body.clone()).collect();
        body::separate(&mut bodies, &self.buckets, &mut self.scratch);
        for (u, b) in self.units.iter_mut().zip(bodies) {
            u.body.pos = b.pos;
            body::keep_out(&mut u.body, &self.arena);
        }
    }

    fn fly_shots(&mut self, c: &Content) {
        let bodies: Vec<Body> = self.units.iter().map(|u| u.body.clone()).collect();
        self.buckets.rebuild(&bodies);
        let mut springs: Vec<(Carry, V3, Option<V3>, Vec<V3>)> = Vec::new();
        for s in 0..self.shots.len() {
            if self.shots[s].done {
                continue;
            }
            let sh = &mut self.shots[s];
            sh.prev = sh.pos;
            sh.pos = sh.pos.add(sh.vel);
            sh.life -= 1;
            let pos = sh.pos;
            self.buckets.near(pos, &mut self.scratch);
            let near: Vec<u16> = self.scratch.clone();
            let mut first_hit = None;
            let mut kills = Vec::new();
            for j in near {
                let j = j as usize;
                let (team, r, hit) = (
                    self.shots[s].team,
                    self.shots[s].radius,
                    self.shots[s].hit.clone(),
                );
                let u = &self.units[j];
                if u.team == team || u.hp <= 0 || hit.contains(&u.id) {
                    continue;
                }
                let centre = u.body.pos.add(v3(0, u.body.radius, 0));
                let d = pos.sub(centre);
                let reach = r + u.body.radius * 2;
                if d.len2() > reach as i64 * reach as i64 {
                    continue;
                }
                let sh = &self.shots[s];
                let push = sh.vel.with_len(ONE).ratio(sh.knock, 100).add(v3(0, ONE / 4, 0));
                let (dmg, dt) = (sh.damage, sh.dtype);
                let id = u.id;
                first_hit.get_or_insert(pos);
                if self.hurt(c, j, dmg, dt, push) {
                    kills.push(self.units[j].body.pos);
                }
                let sh = &mut self.shots[s];
                sh.hit.push(id);
                if sh.pierce > 0 {
                    sh.pierce -= 1;
                } else {
                    sh.done = true;
                    break;
                }
            }
            let sh = &mut self.shots[s];
            if sh.life <= 0 || sh.pos.y < 0 {
                sh.done = true;
            }
            // a trigger fires once per shot: take the carry when it springs
            let springs_now = match sh.carry.as_ref().map(|k| k.on) {
                Some(On::Hit) => first_hit.is_some(),
                Some(On::Death) => !kills.is_empty(),
                Some(On::Expire) => sh.done,
                None => false,
            };
            if springs_now {
                if let Some(k) = sh.carry.take() {
                    springs.push((k, pos, first_hit, kills));
                }
            }
        }
        self.shots.retain(|s| !s.done);
        for (k, end, hit, kills) in springs {
            self.after(c, Some(k), end, hit, &kills, 255);
        }
    }

    fn tick_zones(&mut self, c: &Content) {
        let mut springs: Vec<(Carry, V3, Option<V3>)> = Vec::new();
        for z in 0..self.zones.len() {
            self.zones[z].age += 1;
            self.zones[z].life -= 1;
            let zone = self.zones[z].clone();
            if zone.dps > 0 && zone.age % 15 == 0 {
                for i in 0..self.units.len() {
                    let u = &self.units[i];
                    if u.team == zone.team
                        || u.hp <= 0
                        || u.body.pos.sub(zone.pos).flat_len() > zone.radius + u.body.radius
                    {
                        continue;
                    }
                    if !self.zones[z].entered {
                        self.zones[z].entered = true;
                        if let Some(k) = self.zones[z].carry.take_if(|k| k.on == On::Hit) {
                            springs.push((k, u.body.pos, Some(u.body.pos)));
                        }
                    }
                    self.hurt(c, i, (zone.dps / 4).max(1), zone.dtype, V3::ZERO);
                }
            }
            if self.zones[z].life <= 0 {
                if let Some(k) = self.zones[z].carry.take() {
                    if k.on == On::Expire {
                        springs.push((k, zone.pos, None));
                    }
                }
            }
        }
        self.zones.retain(|z| z.life > 0);
        for (k, at, hit) in springs {
            self.after(c, Some(k), at, hit, &[], 255);
        }
    }

    fn bury(&mut self, c: &Content) {
        let mut deaths: Vec<(Carry, V3, u8)> = Vec::new();
        let mut i = 0;
        while i < self.units.len() {
            if self.units[i].hp > 0 {
                i += 1;
                continue;
            }
            let u = self.units.swap_remove(i);
            let e = &c.file.enemies[u.kind as usize];
            if u.home == Team::Foe && !u.leaked {
                self.tally.kills += 1;
            }
            self.events.push((Ev::Kill, u.body.pos));
            let kick = u.body.vel.add(v3(0, ONE / 40, 0));
            self.pile.add(
                Ragdoll::from_body(
                    u.body.pos,
                    u.body.yaw,
                    content::cm(e.height),
                    u.body.vel,
                    kick,
                    u.kind,
                ),
                self.cap,
            );
            if let Some(k) = u.carry {
                if k.on == On::Death {
                    deaths.push((k, u.body.pos, u.from));
                }
            }
        }
        for (k, at, from) in deaths {
            self.spring_from_summon(c, &k, at, from);
        }
    }

    fn spring_from_summon(&mut self, c: &Content, k: &Carry, at: V3, from: u8) {
        // a summon's death trigger casts from where it fell, aimed at the
        // nearest opponent, on the summon's own team
        let target = self
            .nearest(at, k.team, 6 * ONE)
            .map(|i| self.units[i].body.pos)
            .unwrap_or(at);
        self.run(c, &k.rest, 0, at, target, k.eff, k.team, k.depth, k.cast, from);
    }

    /// An FNV-1a hash of the battle's state, for comparing two runs.
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: i64| {
            for byte in v.to_le_bytes() {
                h ^= byte as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
        };
        eat(self.tick as i64);
        eat(self.tally.damage_taken as i64);
        eat(self.tally.kills as i64);
        for u in &self.units {
            eat(u.id as i64);
            eat(u.hp as i64);
            eat(u.body.pos.x as i64);
            eat(u.body.pos.z as i64);
        }
        for s in &self.shots {
            eat(s.pos.x as i64);
            eat(s.pos.z as i64);
        }
        h
    }
}

/// Where each copy of a formation starts and lands.
fn arrange(arr: Arrange, copies: i32, i: i32, origin: V3, target: V3) -> (V3, V3) {
    if copies <= 1 {
        return (origin, target);
    }
    match arr {
        Arrange::Ring => {
            let a = i * fx::TURN / copies;
            let r = 2 * ONE;
            (
                origin,
                target.add(v3(fx::mul(fx::sin(a), r), 0, fx::mul(fx::cos(a), r))),
            )
        }
        Arrange::Twin => {
            let d = target.sub(origin).flat();
            let side = v3(-d.z, 0, d.x).with_len(ONE * 4 / 5);
            let s = if i % 2 == 0 { side } else { side.scale(-ONE) };
            (origin.add(s), target.add(s))
        }
        Arrange::Fan => {
            let spread = fx::TURN / 30;
            let a = (i - (copies - 1) / 2) * spread;
            let d = target.sub(origin);
            (origin, origin.add(d.yaw(a)))
        }
    }
}

fn first_effect<'a>(c: &'a Content, chain: &[u16]) -> Option<&'a Effect> {
    chain.iter().find_map(|&p| match &c.file.pieces[p as usize].kind {
        PieceKind::Payload { effect, .. } => Some(effect),
        _ => None,
    })
}

/// How far a spell reaches: its first payload's range, in Q16 metres.
pub fn spell_range(c: &Content, chain: &[u16]) -> Fx {
    match first_effect(c, chain) {
        Some(Effect::Bolt { range, .. })
        | Some(Effect::Burst { range, .. })
        | Some(Effect::Hex { range, .. }) => content::cm(*range),
        Some(Effect::Ground { range, .. }) => content::cm((*range).max(400)),
        _ => 18 * ONE,
    }
}

#[cfg(test)]
mod tests;

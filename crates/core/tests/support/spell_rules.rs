//! The spell-space scene and the rules every spell keeps (TRIAGE T4), after
//! Galactic Survivors' rig rules. One fixed field, one student on the tower
//! with mana to spare casting the spell, and a fixed crowd of wild enemies
//! on the path; or, for an invasion, the spell cast at this tower from the
//! entry by seat 1. A few seconds are played and the battle is watched
//! tick by tick.
//!
//! A FAIL is a rule no spell may break whatever its pieces: a panic, too
//! many units, a score that rises, a unit off the world, a battle that
//! plays differently the second time, a mana cost that is not positive, an
//! invasion whose summon comes before any trigger yet makes no unit. A
//! note is about content: a payload that found nothing to do in this scene.
//!
//! Editing this file makes every logbook line stale (`spell_space.rs`).

#![allow(dead_code)]

use tool_core::battle::{Battle, Ev, Incoming, Plan, StudentPlan, Team, Wild, MAX_POPS, MAX_UNITS, WARD_CAP};
use tool_core::content::{Content, Effect, PieceKind};
use tool_core::fx::ONE;
use tool_core::geom::TPS;
use tool_core::spell;

pub const SCENE_SEED: u64 = 0x5eed_5ce7e;
pub const FIELD_SEED: u64 = 7;
/// Seconds played. Long enough for a slow bolt to land and a trigger to
/// fire; short enough that every two-piece spell plays in seconds.
pub const SCENE_S: i32 = 4;
pub const CAP: usize = 24;

pub fn content() -> Content {
    Content::parse(include_str!("../../../../data/content.json")).expect("content")
}

/// What a spell did in the scene.
#[derive(Debug, Default, Clone)]
pub struct Played {
    pub failures: Vec<String>,
    pub notes: Vec<String>,
    pub casts: i32,
    pub dealt: i64,
    pub kills: i32,
    pub statuses: i32,
    pub allies: i32,
    pub invaders: i32,
    pub raised: i32,
    pub zones: i32,
    pub ward: i32,
    pub most_units: usize,
    pub mana: i32,
    pub print: u64,
}

pub const FNV: u64 = 0xcbf2_9ce4_8422_2325;
pub fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// The scene's crowd: four of each of the first three wild enemies, all
/// within the first second. `play` moves each wild enemy, when it first
/// appears, onto the route a few cells from the tower, so a spell has
/// something in range at once.
pub fn crowd(c: &Content) -> Vec<Wild> {
    (0..c.file.enemies.len())
        .filter(|&i| !c.file.enemies[i].summoned_only)
        .take(3)
        .enumerate()
        .map(|(k, e)| Wild {
            enemy: e as u16,
            count: 4,
            at_tick: 1 + k as i32 * 4,
            strength: 100,
        })
        .collect()
}

/// The effect of the first payload before any trigger: the part of the
/// spell that runs at once, whatever the scene.
pub fn first_effect<'a>(c: &'a Content, chain: &[u16]) -> Option<&'a Effect> {
    for &p in chain {
        match &c.file.pieces[p as usize].kind {
            PieceKind::Trigger { .. } => return None,
            PieceKind::Payload { effect, .. } => return Some(effect),
            _ => {}
        }
    }
    None
}

/// A summon before any trigger: an invasion with one must make units.
fn summons_at_once(c: &Content, chain: &[u16]) -> bool {
    for &p in chain {
        match &c.file.pieces[p as usize].kind {
            PieceKind::Trigger { .. } => return false,
            PieceKind::Payload {
                effect: Effect::Summon { .. },
                ..
            } => return true,
            _ => {}
        }
    }
    false
}

fn battle(c: &Content, chain: &[u16]) -> Battle {
    let ticks = SCENE_S * TPS;
    if spell::is_invasion(c, chain) {
        let inv = Incoming {
            from: 1,
            chain: chain.to_vec(),
            strength: 100,
            count_pct: 100,
        };
        return Battle::new(
            c,
            0,
            &Plan::default(),
            &[inv],
            &crowd(c),
            SCENE_SEED,
            FIELD_SEED,
            ticks,
            CAP,
        );
    }
    let plan = Plan {
        spells: vec![chain.to_vec()],
        students: vec![StudentPlan {
            seat: tool_core::battle::AUTO_SEAT,
            mode: 0,
            rotation: Vec::new(),
            mana_max: 100_000,
            regen: 100_000,
            spells: vec![(0, 100)],
            school: 0,
        }],
        placed: Vec::new(),
    };
    Battle::new(c, 0, &plan, &[], &crowd(c), SCENE_SEED, FIELD_SEED, ticks, CAP)
}

/// Plays the scene once and watches every tick.
fn play(c: &Content, chain: &[u16]) -> Played {
    let mut p = Played {
        mana: spell::stats(c, chain).mana,
        ..Played::default()
    };
    let mut b = battle(c, chain);
    let edge = 40 * ONE;
    let mut taken = 0;
    let mut lowest: std::collections::BTreeMap<u32, (i32, i32)> = std::collections::BTreeMap::new();
    let mut seen_invaders: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut seen_allies: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut seen_raised: std::collections::BTreeSet<u16> = std::collections::BTreeSet::new();
    // the route's cells from 14 to 6 cells short of the tower
    let near: Vec<tool_core::geom::V3> = b
        .arena
        .route()
        .iter()
        .rev()
        .skip(6)
        .take(8)
        .map(|&(x, z)| tool_core::arena::centre(x, z))
        .collect();
    let mut moved: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut bring = |b: &mut Battle| {
        for u in b.units.iter_mut() {
            if u.from == 255 && u.home == Team::Foe && moved.insert(u.id) && !near.is_empty() {
                let at = near[u.id as usize % near.len()];
                u.body.pos.x = at.x;
                u.body.pos.z = at.z;
                u.body.prev.x = at.x;
                u.body.prev.z = at.z;
            }
        }
    };
    let fail = |p: &mut Played, f: String| {
        if !p.failures.contains(&f) {
            p.failures.push(f);
        }
    };
    let mut watch = |b: &Battle, p: &mut Played| {
        p.most_units = p.most_units.max(b.units.len());
        if b.units.len() > MAX_UNITS {
            fail(
                p,
                format!("units: {} on the field, more than {MAX_UNITS}", b.units.len()),
            );
        }
        if b.pops.len() > MAX_POPS {
            fail(p, format!("pops: {} at once, more than {MAX_POPS}", b.pops.len()));
        }
        if b.ward > WARD_CAP || b.ward < 0 {
            fail(p, format!("ward: {} outside 0..={WARD_CAP}", b.ward));
        }
        if b.tally.damage_taken < taken {
            fail(
                p,
                format!(
                    "score rose: damage taken fell from {taken} to {}",
                    b.tally.damage_taken
                ),
            );
        }
        taken = b.tally.damage_taken;
        for u in &b.units {
            if u.hp > u.hp_max {
                fail(p, format!("health: a unit has {} of {}", u.hp, u.hp_max));
            }
            let q = u.body.pos;
            if q.x.abs() > edge || q.z.abs() > edge || q.y < -5 * ONE || q.y > 80 * ONE {
                fail(p, "world: a unit left the world".to_string());
            }
            if u.home == Team::Foe {
                let e = lowest.entry(u.id).or_insert((u.hp_max, u.hp));
                e.1 = e.1.min(u.hp);
            }
            if u.from == 1 {
                seen_invaders.insert(u.id);
            }
            if u.summoned_for_owner {
                seen_allies.insert(u.id);
            }
            if u.freeze > 0 || u.charm > 0 || u.slow > 0 || u.fear > 0 {
                p.statuses += 1;
            }
        }
        for s in &b.standing {
            seen_raised.insert(s.id);
        }
        p.zones = p.zones.max(b.zones.len() as i32);
        p.ward = p.ward.max(b.ward);
        p.casts += b.events.iter().filter(|e| matches!(e.0, Ev::Cast(_))).count() as i32;
    };
    watch(&b, &mut p);
    while !b.done() {
        b.step(c);
        bring(&mut b);
        watch(&b, &mut p);
    }
    p.kills = b.tally.kills;
    p.dealt = lowest.values().map(|(max, low)| (max - low.max(&0)) as i64).sum();
    p.invaders = seen_invaders.len() as i32;
    p.allies = seen_allies.len() as i32;
    p.raised = seen_raised.len() as i32;
    let mut h = fnv(FNV, &b.tick.to_le_bytes());
    for u in &b.units {
        for v in [
            u.kind as i32,
            u.hp,
            u.body.pos.x,
            u.body.pos.y,
            u.body.pos.z,
            u.team as i32,
        ] {
            h = fnv(h, &v.to_le_bytes());
        }
    }
    for v in [
        b.tally.kills,
        b.tally.frozen,
        b.tally.charmed,
        b.tally.damage_taken,
        b.tally.warded,
        b.ward,
        b.standing.len() as i32,
        b.zones.len() as i32,
        b.shots.len() as i32,
    ] {
        h = fnv(h, &v.to_le_bytes());
    }
    p.print = h;
    p
}

/// Plays a chain, keeps every rule, and (when `deep`) plays it again to
/// check the second play is the same.
pub fn judge(c: &Content, chain: &[u16], deep: bool) -> Played {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| play(c, chain)));
    let mut p = match r {
        Ok(p) => p,
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            return Played {
                failures: vec![format!("panic: {}", msg.replace(['\t', '\n'], " "))],
                ..Played::default()
            };
        }
    };
    if deep {
        let again = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| play(c, chain)));
        match again {
            Ok(q) if q.print == p.print => {}
            Ok(_) => p
                .failures
                .push("determinism: a second play ended differently".into()),
            Err(_) => p.failures.push("determinism: a second play panicked".into()),
        }
    }
    if p.mana <= 0 {
        p.failures.push(format!("mana: costs {}", p.mana));
    }
    let invasion = spell::is_invasion(c, chain);
    if invasion {
        if summons_at_once(c, chain) && p.invaders == 0 {
            p.failures.push("invasion: its summon made no unit".into());
        }
        if !summons_at_once(c, chain) && p.invaders == 0 {
            p.notes
                .push("invasion: the summon behind a trigger never came".into());
        }
        return p;
    }
    if p.casts == 0 {
        p.notes.push("cast: the student never cast it".into());
        return p;
    }
    match first_effect(c, chain) {
        Some(Effect::Bolt { .. }) | Some(Effect::Burst { .. }) if p.dealt == 0 && p.kills == 0 => {
            p.notes.push("damage: it dealt no damage".into())
        }
        Some(Effect::Ground { dps, .. }) if *dps > 0 && p.dealt == 0 && p.kills == 0 => {
            p.notes.push("damage: its field dealt no damage".into())
        }
        Some(Effect::Ground { .. }) if p.zones == 0 => p.notes.push("field: it left no field".into()),
        Some(Effect::Hex { .. }) if p.statuses == 0 => {
            p.notes.push("status: no enemy took its status".into())
        }
        Some(Effect::Summon { .. }) if p.allies == 0 => p.notes.push("summon: it made no ally".into()),
        Some(Effect::Raise { .. }) if p.raised == 0 => p.notes.push("raise: nothing was raised".into()),
        Some(Effect::Ward { .. }) if p.ward == 0 => p.notes.push("ward: no ward was raised".into()),
        _ => {}
    }
    p
}

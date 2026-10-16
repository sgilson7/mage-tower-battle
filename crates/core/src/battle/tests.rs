//! Battle tests. Expected values are worked out by hand from
//! data/content.json in each test's comments.

use super::*;
use crate::content::shipped;

fn ids(c: &Content, names: &[&str]) -> Vec<u16> {
    names
        .iter()
        .map(|n| c.piece(n).unwrap_or_else(|| panic!("no piece {n}")) as u16)
        .collect()
}

fn empty(c: &Content) -> Battle {
    Battle::new(c, 0, &Plan::default(), &[], &[], 1, 1, TPS * 90, 24)
}

/// Puts one unit exactly at a point and returns its index.
fn put(b: &mut Battle, c: &Content, enemy: &str, at: V3, team: Team) -> usize {
    b.spawn(c, c.enemy(enemy).unwrap() as u16, at, team, 255, None, 100);
    let u = b.units.last_mut().unwrap();
    u.body.pos = v3(at.x, u.body.radius, at.z);
    u.body.prev = u.body.pos;
    b.units.len() - 1
}

#[test]
fn fire_does_double_to_leaf_half_to_earth_and_full_to_bone() {
    let c = shipped();
    let mut b = empty(&c);
    let fire = c.damage_type("fire") as u8;
    let elf = put(&mut b, &c, "bramble_elf", v3(0, 0, 0), Team::Foe);
    let giant = put(&mut b, &c, "hill_giant", v3(6 * ONE, 0, 0), Team::Foe);
    let skel = put(&mut b, &c, "skeleton", v3(-6 * ONE, 0, 0), Team::Foe);
    // a 30-damage fire hit: elf 90 − 60 = 30; giant 900 − 15 = 885; skeleton 120 − 30 = 90
    b.hurt(&c, elf, 30, fire, V3::ZERO);
    b.hurt(&c, giant, 30, fire, V3::ZERO);
    b.hurt(&c, skel, 30, fire, V3::ZERO);
    assert_eq!(
        (b.units[elf].hp, b.units[giant].hp, b.units[skel].hp),
        (30, 885, 90)
    );
    let rates: Vec<i32> = b
        .pops
        .iter()
        .filter(|p| p.kind == PopKind::Number)
        .map(|p| p.rate)
        .collect();
    assert_eq!(rates, vec![200, 50, 100], "each hit shows the rate it used");
}

#[test]
fn a_frozen_enemy_does_not_move_and_thaws() {
    let c = shipped();
    let mut b = empty(&c);
    let at = arena::centre(10, 20);
    let i = put(&mut b, &c, "skeleton", at, Team::Foe);
    b.hex(&c, i, Status::Freeze, 60);
    let start = b.units[i].body.pos;
    for _ in 0..60 {
        b.step(&c);
        assert_eq!(
            b.units[i].body.pos.flat(),
            start.flat(),
            "moved while frozen at tick {}",
            b.tick
        );
    }
    for _ in 0..30 {
        b.step(&c);
    }
    assert_ne!(b.units[i].body.pos.flat(), start.flat(), "never thawed");
}

#[test]
fn a_repeat_freeze_within_five_seconds_lasts_half_as_long() {
    let c = shipped();
    let mut b = empty(&c);
    let i = put(&mut b, &c, "skeleton", arena::centre(10, 20), Team::Foe);
    b.hex(&c, i, Status::Freeze, 100);
    b.units[i].freeze = 0;
    b.tick += 10;
    b.hex(&c, i, Status::Freeze, 100);
    // status_repeat_pct is 50
    assert_eq!(b.units[i].freeze, 50);
}

#[test]
fn a_charmed_enemy_fights_its_own_side_and_then_returns() {
    let c = shipped();
    let mut b = empty(&c);
    let a = arena::centre(12, 20);
    let giant = put(&mut b, &c, "hill_giant", a, Team::Foe);
    let other = put(
        &mut b,
        &c,
        "hill_giant",
        a.add(v3(ONE * 26 / 10, 0, 0)),
        Team::Foe,
    );
    b.hex(&c, giant, Status::Charm, 300);
    assert_eq!(b.units[giant].team, Team::Ally);
    assert_eq!(b.tally.giants_charmed, 1);
    let hp0 = b.units[other].hp;
    for _ in 0..240 {
        b.step(&c);
    }
    let other_hp = b.units.iter().find(|u| u.id == 2).map(|u| u.hp).unwrap_or(0);
    // giant on giant is earth on earth, ×1: at least two blows of 60 in four seconds
    assert!(
        other_hp <= hp0 - 120,
        "the charmed giant struck its own side: {other_hp}"
    );
    for _ in 0..120 {
        b.step(&c);
    }
    let g = b.units.iter().find(|u| u.id == 1).unwrap();
    assert_eq!(g.team, Team::Foe, "charm wore off");
}

#[test]
fn a_ward_takes_damage_before_the_score_and_the_score_never_rises() {
    let c = shipped();
    let mut b = empty(&c);
    b.ward = 50;
    let t = b.tower();
    // a hill giant reaching the tower strikes once: 60 × leak_mult 3 = 180,
    // of which the ward takes 50 and the score 130, and the giant is spent
    put(&mut b, &c, "hill_giant", t.add(v3(-(ONE * 4), 0, 0)), Team::Foe);
    let mut last = 0;
    let mut first = None;
    for _ in 0..TPS * 20 {
        b.step(&c);
        assert!(b.tally.damage_taken >= last, "score rose");
        last = b.tally.damage_taken;
        if first.is_none() && b.tally.warded > 0 {
            first = Some((b.tally.warded, b.tally.damage_taken));
        }
    }
    assert_eq!(first, Some((50, 130)));
    assert_eq!(b.tally.damage_taken, 130, "one strike, then spent");
    assert_eq!(b.tally.kills, 0, "a leak is not a kill");
}

#[test]
fn acid_halo_sends_eight_globs_and_each_hit_spills_oil_once() {
    let c = shipped();
    let halo = ids(&c, &["ring", "acid_glob", "on_hit", "oil_spill"]);
    let mut b = empty(&c);
    b.spells = vec![halo.clone()];
    // eight imps standing on the ring's points, 2 m round a target point
    let target = arena::centre(12, 18);
    for i in 0..8 {
        let a = i * fx::TURN / 8;
        let p = target.add(v3(fx::mul(fx::sin(a), 2 * ONE), 0, fx::mul(fx::cos(a), 2 * ONE)));
        let k = put(&mut b, &c, "ember_imp", p, Team::Foe);
        b.hex(&c, k, Status::Freeze, 600);
    }
    let cast = b.new_budget(&c);
    // cast from just above the target so each glob drops onto its own imp
    let origin = target.add(v3(0, ONE * 3, 0));
    b.run(
        &c,
        &halo,
        0,
        origin,
        target.add(v3(0, ONE / 2, 0)),
        100,
        Team::Ally,
        0,
        cast,
        255,
    );
    assert_eq!(b.shots.len(), 8, "a ring of eight");
    // budget 32 − 8 globs = 24 left
    assert_eq!(b.budgets[cast as usize], 24);
    for _ in 0..60 {
        b.step(&c);
    }
    let spills = b.zones.iter().filter(|z| z.field == Field::Oil).count();
    assert_eq!(spills, 8, "one spill per glob that hit");
    // each glob: 26 × 25% share = 6 (rounded down), acid on fire ×1: imps at 70 − 6 = 64
    for u in &b.units {
        assert_eq!(u.hp, 64);
    }
    assert_eq!(b.budgets[cast as usize], 16, "eight spills spent eight more");
}

#[test]
fn a_piercing_bolt_springs_its_trigger_once_not_once_per_hit() {
    let c = shipped();
    let chain = ids(&c, &["pierce", "ice_shard", "on_hit", "oil_spill"]);
    let mut b = empty(&c);
    let base = arena::centre(6, 18);
    for k in 0..3 {
        let i = put(&mut b, &c, "skeleton", base.add(v3(ONE * 2 * k, 0, 0)), Team::Foe);
        b.hex(&c, i, Status::Freeze, 600);
    }
    let cast = b.new_budget(&c);
    let from = base.add(v3(-(ONE * 3), ONE / 2, 0));
    b.run(
        &c,
        &chain,
        0,
        from,
        base.add(v3(ONE * 4, ONE / 2, 0)),
        100,
        Team::Ally,
        0,
        cast,
        255,
    );
    for _ in 0..60 {
        b.step(&c);
    }
    // pierce 2 lets one shard hit all three (frost on bone ×1: 22 each, 120 − 22 = 98)
    assert!(
        b.units.iter().all(|u| u.hp == 98),
        "{:?}",
        b.units.iter().map(|u| u.hp).collect::<Vec<_>>()
    );
    assert_eq!(b.zones.iter().filter(|z| z.field == Field::Oil).count(), 1);
}

#[test]
fn one_payload_budget_is_shared_by_a_whole_cast_with_nested_triggers() {
    let c = shipped();
    // each skeleton's death casts a ring of eight ghost summons
    let chain = ids(&c, &["invade", "ring", "skeleton", "on_death", "ring", "ghost"]);
    let mut b = empty(&c);
    let at = arena::centre(10, 20);
    let cast = b.new_budget(&c);
    b.run(&c, &chain, 0, at, at, 100, Team::Foe, 0, cast, 2);
    // ring of 8 copies, 3 skeletons each: 24; budget 32 − 8 = 24
    assert_eq!(b.units.len(), 24);
    assert!(b.units.iter().all(|u| u.from == 2 && u.team == Team::Foe));
    for u in b.units.iter_mut() {
        u.hp = 0;
    }
    b.bury(&c);
    // only three deaths find budget for their ring of 8 (24 → 16 → 8 → 0), each 2 ghosts: 48
    let ghost = c.enemy("ghost").unwrap() as u16;
    assert_eq!(b.units.iter().filter(|u| u.kind == ghost).count(), 48);
    assert_eq!(b.budgets[cast as usize], 0);
    assert!(b.units.iter().all(|u| u.from == 2), "ghosts keep the sender");
}

#[test]
fn an_invasion_arrives_at_the_entry_for_its_target() {
    let c = shipped();
    let parade = ids(&c, &["invade", "ring", "skeleton", "on_death", "ghost"]);
    let incoming = [Incoming {
        from: 3,
        chain: parade,
        strength: 50,
        count_pct: 100,
    }];
    let b = Battle::new(&c, 0, &Plan::default(), &incoming, &[], 1, 1, TPS * 90, 24);
    // ring 8 × skeleton count 3 at 50% → 1 (rounded down, at least 1): 8 skeletons
    assert_eq!(b.units.len(), 8);
    let entry = arena::centre(b.arena.entry.0, b.arena.entry.1);
    for u in &b.units {
        assert!(u.body.pos.sub(entry).flat_len() < 4 * ONE);
        assert_eq!(u.from, 3);
        // half strength: 120 × 50% = 60 health
        assert_eq!(u.hp, 60);
    }
}

#[test]
fn students_beyond_the_seats_stand_on_the_ground_with_less_range() {
    let c = shipped();
    let fire = ids(&c, &["fire_bolt"]);
    let student = StudentPlan {
        mana_max: 100,
        regen: 10,
        spells: vec![(0, 100)],
        school: 0,
    };
    let plan = Plan {
        spells: vec![fire],
        students: vec![student; 14],
        placed: vec![],
    };
    let b = Battle::new(&c, 0, &plan, &[], &[], 1, 1, TPS * 90, 24);
    // tower_seats is 12
    assert!(b.casters[..12]
        .iter()
        .all(|k| k.pos.y >= 4 * ONE && k.range_pct == 100));
    assert!(b.casters[12..].iter().all(|k| k.pos.y < ONE && k.range_pct == 70));
}

#[test]
fn a_placed_wall_bends_the_route_and_a_sub_tower_adds_a_seat() {
    let c = shipped();
    let glass = c.feature("glass_wall").unwrap() as u16;
    let sub = c.feature("sub_tower").unwrap() as u16;
    let fire = ids(&c, &["fire_bolt"]);
    let student = StudentPlan {
        mana_max: 100,
        regen: 10,
        spells: vec![(0, 100)],
        school: 0,
    };
    let plan = Plan {
        spells: vec![fire],
        students: vec![student; 13],
        placed: vec![
            Placed {
                feature: glass,
                cells: vec![(10, 20), (11, 20), (12, 20)],
            },
            Placed {
                feature: sub,
                cells: vec![(20, 12)],
            },
        ],
    };
    let b = Battle::new(&c, 0, &plan, &[], &[], 1, 1, TPS * 90, 24);
    assert!(matches!(b.arena.at(11, 20), Cell::Feature(_)));
    // the 13th student takes the sub-tower's seat at 3.3 m, at full range
    let s = &b.casters[12];
    assert_eq!(s.pos.y, ONE * 33 / 10);
    assert_eq!(s.range_pct, 100);
}

#[test]
fn a_battle_replays_to_the_same_hash_and_a_different_seed_differs() {
    let c = shipped();
    let fire = ids(&c, &["fire_bolt"]);
    let halo = ids(&c, &["ring", "acid_glob", "on_hit", "oil_spill"]);
    let student = StudentPlan {
        mana_max: 150,
        regen: 12,
        spells: vec![(0, 90), (1, 70)],
        school: 0,
    };
    let plan = Plan {
        spells: vec![fire, halo],
        students: vec![student; 6],
        placed: vec![],
    };
    let wild = [
        Wild {
            enemy: c.enemy("bramble_elf").unwrap() as u16,
            count: 30,
            at_tick: 30,
        },
        Wild {
            enemy: c.enemy("hill_giant").unwrap() as u16,
            count: 3,
            at_tick: 600,
        },
    ];
    let run = |seed: u64| {
        let mut b = Battle::new(&c, 0, &plan, &[], &wild, seed, seed, TPS * 60, 24);
        while !b.done() {
            b.step(&c);
        }
        (b.hash(), b.tally.kills, b.tally.damage_taken)
    };
    let a = run(7);
    assert_eq!(a, run(7));
    assert_ne!(a.0, run(8).0);
    assert!(a.1 > 10, "students killed {} of the elves", a.1);
}

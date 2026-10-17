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
    let cast = b.new_budget(&c, 255);
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
    let cast = b.new_budget(&c, 255);
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
    let cast = b.new_budget(&c, 255);
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
        seat: crate::battle::AUTO_SEAT,
        mode: 0,
        rotation: Vec::new(),
        mana_max: 100,
        regen: 10,
        spells: vec![(0, 100)],
        school: 0,
        ..Default::default()
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
        seat: crate::battle::AUTO_SEAT,
        mode: 0,
        rotation: Vec::new(),
        mana_max: 100,
        regen: 10,
        spells: vec![(0, 100)],
        school: 0,
        ..Default::default()
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
        seat: crate::battle::AUTO_SEAT,
        mode: 0,
        rotation: Vec::new(),
        mana_max: 150,
        regen: 12,
        spells: vec![(0, 90), (1, 70)],
        school: 0,
        ..Default::default()
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
            strength: 100,
        },
        Wild {
            enemy: c.enemy("hill_giant").unwrap() as u16,
            count: 3,
            at_tick: 600,
            strength: 100,
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

#[test]
fn a_student_keeps_at_most_four_summons_alive_and_each_lasts_twelve_seconds() {
    let c = shipped();
    let mut b = empty(&c);
    let chain = ids(&c, &["skeleton"]);
    let at = arena::centre(16, 16);
    let ally = |b: &Battle, who: u8| b.units.iter().filter(|u| u.caster == who && u.hp > 0).count();
    // a foe that outlasts the test keeps the battle going
    let far = put(&mut b, &c, "hill_giant", arena::centre(1, 1), Team::Foe);
    b.units[far].hp = 1_000_000;
    b.units[far].hp_max = 1_000_000;
    // skeleton count 3 at ally_summon_pct 67 → 2 a cast; summons_per_student 4
    for k in 0..3 {
        let cast = b.new_budget(&c, 0);
        b.run(&c, &chain, 0, at, at, 100, Team::Ally, 0, cast, 255);
        assert_eq!(ally(&b, 0), [2, 4, 4][k], "cast {k}");
    }
    // another student has their own four
    let cast = b.new_budget(&c, 1);
    b.run(&c, &chain, 0, at, at, 100, Team::Ally, 0, cast, 255);
    assert_eq!(ally(&b, 1), 2);
    // summon_life_s 12: 720 ticks, then they fade, and no kill is counted
    for _ in 0..719 {
        b.step(&c);
    }
    assert_eq!(ally(&b, 0), 4);
    b.step(&c);
    assert_eq!(ally(&b, 0), 0);
    assert_eq!(b.tally.kills, 0);
}

#[test]
fn an_invasion_keeps_its_numbers() {
    let c = shipped();
    let mut b = empty(&c);
    let chain = ids(&c, &["invade", "skeleton"]);
    let cast = b.new_budget(&c, 255);
    let at = arena::centre(1, 30);
    b.run(&c, &chain, 0, at, at, 100, Team::Foe, 0, cast, 2);
    // skeleton count 3, no limit for what a rival sends
    assert_eq!(b.units.iter().filter(|u| u.team == Team::Foe).count(), 3);
    assert!(b.units.iter().all(|u| u.life == 0));
}

#[test]
fn office_hours_seats_students_where_they_were_put_and_others_fill_in() {
    let c = shipped();
    let fire = ids(&c, &["fire_bolt"]);
    let mut a = StudentPlan {
        seat: AUTO_SEAT,
        mode: 0,
        rotation: Vec::new(),
        mana_max: 100,
        regen: 10,
        spells: vec![(0, 100)],
        school: 0,
        ..Default::default()
    };
    let tower = |b: &Battle| b.tower();
    let mut on_ground = a.clone();
    on_ground.seat = GROUND_SEAT;
    a.seat = 5;
    let auto = StudentPlan {
        seat: AUTO_SEAT,
        ..a.clone()
    };
    let plan = Plan {
        spells: vec![fire],
        students: vec![auto.clone(), on_ground, a, auto],
        placed: vec![],
    };
    let b = Battle::new(&c, 0, &plan, &[], &[], 1, 1, TPS * 90, 24);
    let t = tower(&b);
    // the first auto student takes seat 0, the chosen one seat 5, the next auto seat 1
    assert_eq!(b.casters[0].pos, seat_pos(t, 0));
    assert_eq!(b.casters[2].pos, seat_pos(t, 5));
    assert_eq!(b.casters[3].pos, seat_pos(t, 1));
    assert_eq!(b.casters[1].pos, ground_pos(t, 0));
    assert_eq!(
        b.casters[1].range_pct,
        c.balance().ground_range_pct,
        "the ground has less range"
    );
}

#[test]
fn a_rotation_casts_its_spells_in_order() {
    let c = shipped();
    let fire = ids(&c, &["fire_bolt"]);
    let ice = ids(&c, &["ice_shard"]);
    let student = StudentPlan {
        seat: AUTO_SEAT,
        mode: CAST_ROTATION,
        rotation: vec![1, 0],
        mana_max: 100_000,
        regen: 0,
        spells: vec![(0, 100), (1, 100)],
        school: 0,
        ..Default::default()
    };
    let plan = Plan {
        spells: vec![fire.clone(), ice.clone()],
        students: vec![student],
        placed: vec![],
    };
    let mut b = Battle::new(&c, 0, &plan, &[], &[], 1, 1, TPS * 90, 24);
    let near = b.tower().add(v3(4 * ONE, 0, 0));
    let foe = put(&mut b, &c, "hill_giant", near, Team::Foe);
    b.units[foe].hp = 1_000_000;
    b.units[foe].freeze = 1_000_000;
    let costs = [
        crate::spell::stats(&c, &fire).mana,
        crate::spell::stats(&c, &ice).mana,
    ];
    let mut spent = Vec::new();
    let mut last = b.casters[0].mana;
    for _ in 0..TPS * 10 {
        b.step(&c);
        let now = b.casters[0].mana;
        if now < last {
            spent.push((last - now) / 100);
        }
        last = now;
        if spent.len() == 3 {
            break;
        }
    }
    // ice (index 1) first, then fire, then ice again
    assert_eq!(spent, vec![costs[1], costs[0], costs[1]]);
}

#[test]
fn a_branch_starts_both_rows_when_its_trigger_fires() {
    let c = shipped();
    let mut b = empty(&c);
    let at = arena::centre(16, 16);
    let foe = put(&mut b, &c, "hill_giant", at, Team::Foe);
    b.units[foe].hp = 1_000_000;
    let one = ids(&c, &["fireball", "on_hit", "fire_bolt"]);
    let two = ids(&c, &["fireball", "on_hit", "fire_bolt", "branch", "ice_shard"]);
    let cast = b.new_budget(&c, 0);
    b.run(&c, &one, 0, at, at, 100, Team::Ally, 0, cast, 255);
    let after_one = b.shots.len();
    let cast = b.new_budget(&c, 0);
    b.run(&c, &two, 0, at, at, 100, Team::Ally, 0, cast, 255);
    // the burst hits at once; one row fires one bolt, two rows fire two
    assert_eq!(after_one, 1);
    assert_eq!(b.shots.len() - after_one, 2);
}

#[test]
fn seating_gives_choices_first_then_the_first_free_seat() {
    // three tower seats, one sub-tower seat; worked by hand: the second
    // asks for seat 1 after the first did, so is given the next free one
    let wants = [AUTO_SEAT, 1, 1, SUB_SEAT, GROUND_SEAT, AUTO_SEAT, AUTO_SEAT];
    assert_eq!(
        seating(3, 1, &wants),
        vec![0, 1, 2, SUB_SEAT, GROUND_SEAT, GROUND_SEAT, GROUND_SEAT]
    );
    // a sub-tower that is not there counts as no choice
    assert_eq!(seating(1, 0, &[SUB_SEAT, AUTO_SEAT]), vec![0, GROUND_SEAT]);
}

#[test]
fn a_student_put_on_a_sub_tower_stands_on_it_not_on_a_tower_seat() {
    // sub-tower seats were once numbered from 10, inside the tower's 12
    let c = shipped();
    assert!(c.balance().tower_seats <= SUB_SEAT);
    let sub = c.feature("sub_tower").unwrap() as u16;
    let st = StudentPlan {
        seat: SUB_SEAT,
        mode: 0,
        rotation: Vec::new(),
        mana_max: 100,
        regen: 10,
        spells: vec![(0, 100)],
        school: 0,
        ..Default::default()
    };
    let plan = Plan {
        spells: vec![ids(&c, &["fire_bolt"])],
        students: vec![st],
        placed: vec![Placed {
            feature: sub,
            cells: vec![(20, 12)],
        }],
    };
    let b = Battle::new(&c, 0, &plan, &[], &[], 1, 1, TPS * 90, 24);
    let at = crate::arena::centre(20, 12);
    assert_eq!(
        (b.casters[0].pos.x, b.casters[0].pos.y, b.casters[0].pos.z),
        (at.x, ONE * 33 / 10, at.z)
    );
}

// ---- round 8: mages who fight, and raiders (T40-T43) ----

fn temper(c: &Content, id: &str) -> u8 {
    c.file.temperaments.iter().position(|t| t.id == id).unwrap() as u8
}

/// A student who knows Fire Bolt at `eff` percent, with a full well of mana.
fn bolt_student(c: &Content, t: &str, eff: i32) -> StudentPlan {
    StudentPlan {
        mana_max: 10_000,
        regen: 10_000,
        spells: vec![(0, eff)],
        temper: temper(c, t),
        name: format!("{t} student"),
        ..Default::default()
    }
}

#[test]
fn a_mages_health_and_speed_come_from_effectiveness_summed_over_their_spells() {
    let c = shipped();
    // five spells at 100%: 500% summed. Health 40 + 20 × 5 = 140; speed
    // 200 + 40 × 5 = 400 cm/s, 4 m/s: 400 / 100 / 60 metres a tick
    let five = StudentPlan {
        spells: vec![(0, 100); 5],
        ..Default::default()
    };
    let (hp, speed) = mage_stats(&c, &five);
    assert_eq!(hp, 140);
    assert_eq!(speed, content::cms(400));
    // none known: the base, 40 and 2 m/s
    assert_eq!(mage_stats(&c, &StudentPlan::default()), (40, content::cms(200)));
    // half of one spell: 40 + 20 × 0.5 = 50
    let half = StudentPlan {
        spells: vec![(0, 50)],
        ..Default::default()
    };
    assert_eq!(mage_stats(&c, &half).0, 50);
}

#[test]
fn a_student_away_raiding_is_not_on_their_own_tower() {
    let c = shipped();
    let fire = ids(&c, &["fire_bolt"]);
    let away = StudentPlan {
        raid: 2,
        ..bolt_student(&c, "steadfast", 100)
    };
    let plan = Plan {
        spells: vec![fire],
        students: vec![bolt_student(&c, "steadfast", 100), away],
        placed: vec![],
    };
    let b = Battle::new(&c, 0, &plan, &[], &[], 1, 1, TPS * 90, 24);
    assert_eq!(b.casters.len(), 1, "one at home, one away");
    assert_eq!(b.casters[0].team, Team::Ally);
}

fn raided(c: &Content, defenders: Vec<StudentPlan>, raider: StudentPlan) -> Battle {
    let fire = ids(c, &["fire_bolt"]);
    let plan = Plan {
        spells: vec![fire.clone()],
        students: defenders,
        placed: vec![],
    };
    // one enemy late in the day, so the battle is not over before the
    // raider has done its work (T54)
    let late = [Wild {
        enemy: 0,
        count: 1,
        at_tick: TPS * 85,
        strength: 100,
    }];
    let mut b = Battle::new(c, 0, &plan, &[], &late, 1, 1, TPS * 90, 24);
    b.add_raider(
        c,
        &Raider {
            from: 2,
            plan: raider,
            spells: vec![fire],
        },
    );
    b
}

#[test]
fn a_raider_walks_in_late_and_a_reckless_one_strikes_the_tower_for_its_sender() {
    let c = shipped();
    // a raider a fall late: raid_arrive_s 4 s + 2 s = 6 s, 360 ticks
    let r = StudentPlan {
        late: 2 * TPS,
        ..bolt_student(&c, "reckless", 100)
    };
    let mut b = raided(&c, vec![], r);
    let k = b.casters.len() - 1;
    assert_eq!(
        (b.casters[k].team, b.casters[k].arrive, b.casters[k].from),
        (Team::Foe, 6 * TPS, 2)
    );
    let start = b.casters[k].pos;
    for _ in 0..6 * TPS - 1 {
        b.step(&c);
    }
    assert!(!b.standing_mage(k), "not on the field before it arrives");
    assert_eq!(b.casters[k].pos, start, "nor moving");
    assert!(!b.done(), "a raider still to come keeps the battle going");
    for _ in 0..TPS * 60 {
        b.step(&c);
        if b.tally.damage_taken > 0 {
            break;
        }
    }
    assert!(b.tally.damage_taken > 0, "it reached the tower and cast at it");
    assert_eq!(
        b.tally.taken_from[2], b.tally.damage_taken,
        "every point counted against its sender"
    );
    assert!(
        b.casters[k].pos.sub(b.tower()).flat_len() < start.sub(b.tower()).flat_len(),
        "it walked toward the tower"
    );
}

#[test]
fn a_hunter_leaves_the_tower_for_a_raider_and_climbs_back_once_it_falls() {
    let c = shipped();
    let hunter = StudentPlan {
        seat: 0,
        ..bolt_student(&c, "hunter", 100)
    };
    let steady = StudentPlan {
        seat: 1,
        ..bolt_student(&c, "steadfast", 100)
    };
    let mut b = raided(&c, vec![hunter, steady], bolt_student(&c, "steadfast", 100));
    assert!(
        b.casters[0].on_tower && b.casters[1].on_tower,
        "both start on their seats"
    );
    for _ in 0..4 * TPS + 2 {
        b.step(&c);
    }
    assert!(
        !b.casters[0].on_tower,
        "the hunter left the tower once a raider was on the field"
    );
    assert!(b.casters[1].on_tower, "the steadfast student stays on the seat");
    // the raider falls: the hunter goes back up
    let raider = b.casters.len() - 1;
    b.hurt_mage(raider, 10_000, 0);
    assert!(b.casters[raider].fell > 0 && !b.standing_mage(raider));
    for _ in 0..TPS * 20 {
        b.step(&c);
        if b.casters[0].on_tower {
            break;
        }
    }
    assert!(b.casters[0].on_tower, "back on the seat with no raider left");
    assert_eq!(b.casters[0].pos, b.casters[0].home);
}

#[test]
fn a_fallen_mage_casts_no_more_and_a_raiders_bolt_hurts_a_student() {
    let c = shipped();
    let steady = StudentPlan {
        seat: 0,
        ..bolt_student(&c, "steadfast", 100)
    };
    let mut b = raided(&c, vec![steady], bolt_student(&c, "steadfast", 100));
    let raider = b.casters.len() - 1;
    // a raider's bolt that reaches the student's seat: hurt, and the
    // shot is spent
    let at = b.casters[0].pos.add(v3(0, HAND, 0));
    b.shots.push(Shot {
        pos: at.sub(v3(0, 0, ONE / 10)),
        prev: at,
        vel: v3(0, 0, ONE / 20),
        life: 10,
        damage: 30,
        dtype: 0,
        radius: ONE / 4,
        pierce: 0,
        knock: 0,
        hit: vec![],
        team: Team::Foe,
        carry: None,
        done: false,
        from: 2,
    });
    let before = b.casters[0].hp;
    b.fly_shots(&c);
    assert_eq!(b.casters[0].hp, before - 30);
    assert!(b.shots.is_empty());
    // a skeleton beside the tower: a standing student casts at it, a
    // fallen one does not
    let casts = |fallen: bool| {
        let mut b = raided(
            &c,
            vec![StudentPlan {
                seat: 0,
                ..bolt_student(&c, "steadfast", 100)
            }],
            bolt_student(&c, "steadfast", 100),
        );
        if fallen {
            b.hurt_mage(0, 10_000, 0);
        }
        let near = b.tower().add(v3(ONE * 5, 0, 0));
        put(&mut b, &c, "skeleton", near, Team::Foe);
        let mut n = 0;
        for _ in 0..TPS * 3 {
            b.step(&c);
            if b.casters[0].casting > 0 {
                n += 1;
            }
        }
        n
    };
    assert!(casts(false) > 0, "standing, the student casts at the skeleton");
    assert_eq!(casts(true), 0, "fallen, never");
    let _ = raider;
}

#[test]
fn a_battle_lasts_until_its_enemies_are_gone_and_a_raider_alone_does_not_hold_it_open() {
    let c = shipped();
    // one slow walker that takes longer than battle_s to reach the tower,
    // in a battle allowed 600 s (T54)
    let wild = [Wild {
        enemy: 0,
        count: 1,
        at_tick: TPS,
        strength: 100,
    }];
    let mut b = Battle::new(&c, 0, &Plan::default(), &[], &wild, 1, 1, TPS * 600, 24);
    let mut t = 0;
    while !b.done() && t < TPS * 600 {
        b.step(&c);
        t += 1;
    }
    assert!(b.done());
    assert!(
        b.tally.damage_taken > 0,
        "it ended because the enemy reached the tower"
    );
    assert!(t < TPS * 600, "not at the time limit");
    // a rival's raider still standing once the enemies are gone goes home
    let r = bolt_student(&c, "steadfast", 100);
    let mut b = raided(&c, vec![], r);
    let k = b.casters.len() - 1;
    for _ in 0..TPS * 7 {
        b.step(&c);
    }
    // raid_arrive_s 4: arrived; the late enemy at 85 s is still to come
    assert!(b.standing_mage(k) && !b.done());
    b.queue.clear();
    assert!(b.done(), "the raider alone does not keep it going");
}

#[test]
fn a_battle_stops_at_its_time_limit() {
    let c = shipped();
    let wild = [Wild {
        enemy: 0,
        count: 1,
        at_tick: TPS * 30,
        strength: 100,
    }];
    let mut b = Battle::new(&c, 0, &Plan::default(), &[], &wild, 1, 1, TPS * 10, 24);
    for _ in 0..TPS * 11 {
        b.step(&c);
    }
    assert_eq!(b.tick, TPS * 10, "ten seconds and no more, enemy or not");
    assert!(b.done());
}

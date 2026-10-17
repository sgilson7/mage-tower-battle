//! Preparation tests, with expected values worked out from
//! data/content.json in the comments.

use super::*;
use crate::content::shipped;

fn prep(c: &Content) -> Prep {
    let mut p = Prep::new(c, 0, 11, 11);
    p.start_day(c, TPS * 240);
    p
}

fn go(p: &mut Prep, s: Station) {
    let at = station_pos(s);
    // stand just inside reach, on the side toward the room's centre
    let toward = v3(0, 0, 0).sub(at).flat().with_len(PROP + BODY + ONE / 10);
    p.pos = at.add(toward);
}

fn learn(p: &mut Prep, c: &Content, names: &[&str]) {
    for n in names {
        p.known[c.piece(n).unwrap()] = true;
    }
}

#[test]
fn walking_right_for_a_second_covers_the_walking_speed() {
    let c = shipped();
    let mut p = prep(&c);
    let start = p.pos;
    for _ in 0..TPS {
        p.step(&c, RIGHT);
    }
    // 450 cm/s: 60 ticks of cms(450) = 60 × 4915 Q16 = 4.5 m
    assert_eq!(p.pos.x - start.x, 60 * content::cms(450));
    assert_eq!(p.pos.z, start.z);
}

#[test]
fn the_room_wall_and_the_furniture_stop_the_archmage() {
    let c = shipped();
    let mut p = prep(&c);
    for _ in 0..TPS * 10 {
        p.step(&c, DOWN);
    }
    assert!(p.pos.flat_len() <= ROOM_RADIUS - BODY);
    let mut q = prep(&c);
    q.pos = v3(0, 0, ONE * 2);
    // walking straight at the map table (z −1.5 m): never inside its furniture
    let map = station_pos(Station::Map);
    for _ in 0..TPS * 3 {
        q.step(&c, UP);
        assert!(
            q.pos.sub(map).flat_len() >= PROP + BODY - 2,
            "walked into the table at {:?}",
            q.pos
        );
    }
    assert!(q.pos.z > map.z, "stopped on the near side of the table");
}

#[test]
fn an_action_away_from_its_station_is_refused() {
    let c = shipped();
    let mut p = prep(&c);
    assert_eq!(p.act(&c, "library.read", ""), Err("prep.not_here"));
    assert_eq!(p.act(&c, "podium.teach", ""), Err("prep.not_here"));
    go(&mut p, Station::Library);
    assert_eq!(p.station(), Some(Station::Library));
    assert_eq!(p.act(&c, "library.read", ""), Ok(()));
}

/// Shelves every falling book on its own school's shelf until the find.
pub fn sort(p: &mut Prep, c: &Content) {
    let mut guard = 0;
    while let Activity::Sorting { book, .. } = &p.activity {
        let shelf = book.to_string();
        p.act(c, "sort.shelf", &shelf).unwrap();
        guard += 1;
        assert!(guard < 100, "the sort ends");
    }
}

#[test]
fn a_book_is_found_by_shelving_ten_and_a_miss_makes_the_next_fall_faster() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Library);
    let before = p.known.iter().filter(|&&k| k).count();
    p.act(&c, "library.read", "").unwrap();
    let Activity::Sorting {
        book, fall, shelved, ..
    } = p.activity.clone()
    else {
        panic!("sorting")
    };
    // sort_fall_ms 1600: 96 ticks; none shelved yet
    assert_eq!((fall, shelved), (96, 0));
    // a wrong shelf: back on the pile, and 85% of the fall: 81 ticks
    let wrong = ((book as usize + 1) % c.file.schools.len()).to_string();
    p.act(&c, "sort.shelf", &wrong).unwrap();
    let Activity::Sorting { fall, shelved, .. } = p.activity.clone() else {
        panic!("still sorting")
    };
    assert_eq!((fall, shelved), (81, 0));
    // a book that lands counts as a miss too
    for _ in 0..81 {
        p.step(&c, 0);
    }
    let Activity::Sorting { fall, .. } = p.activity.clone() else {
        panic!("still sorting")
    };
    assert_eq!(fall, 68);
    // sort_books 10 shelved finds the next book on the pile: nine is not enough
    for _ in 0..9 {
        let Activity::Sorting { book, .. } = p.activity.clone() else {
            panic!("still sorting")
        };
        p.act(&c, "sort.shelf", &book.to_string()).unwrap();
    }
    assert!(
        matches!(p.activity, Activity::Sorting { shelved: 9, .. }),
        "nine shelved, still sorting"
    );
    sort(&mut p, &c);
    assert!(!p.busy());
    assert_eq!(
        p.known.iter().filter(|&&k| k).count() + usize::from(p.notes.iter().any(|n| n.key == "library.dud")),
        before + 1,
        "a new piece, or a book already known"
    );
}

#[test]
fn the_librarian_golem_finds_a_new_book_every_45_seconds() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Library);
    let before = p.known.iter().filter(|&&k| k).count();
    let mana = p.mana;
    p.act(&c, "library.golem", "").unwrap();
    // librarian_golem: 30 mana, 45 s (2,700 ticks); the golem passes known books by
    assert_eq!(mana - p.mana, 3000);
    go(&mut p, Station::Podium);
    for _ in 0..2699 {
        p.step(&c, 0);
    }
    assert_eq!(p.known.iter().filter(|&&k| k).count(), before);
    p.step(&c, 0);
    assert!(p.known.iter().filter(|&&k| k).count() > before);
    assert!(
        p.jobs.iter().any(|j| matches!(j.work, Work::Search)),
        "and it goes on looking"
    );
}

#[test]
fn the_lab_saves_a_spell_only_from_known_pieces_and_by_the_rules() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Lab);
    assert_eq!(p.act(&c, "lab.add", "ring"), Err("lab.unknown"));
    learn(&mut p, &c, &["ring", "acid_glob", "on_hit", "oil_spill"]);
    p.act(&c, "lab.add", "acid_glob").unwrap();
    p.act(&c, "lab.add", "on_hit").unwrap();
    assert_eq!(p.act(&c, "lab.save", ""), Err("lab.trigger_at_end"));
    p.act(&c, "lab.add", "oil_spill").unwrap();
    p.act(&c, "lab.save", "").unwrap();
    assert_eq!(p.spells.len(), 1, "not kept until it is sealed (T22)");
    seal(&mut p, &c);
    assert_eq!(p.spells.len(), 2, "Fire Bolt and the new spell");
    assert!(p.draft.is_empty());
    assert_eq!(p.chosen, 1);
}

/// Plays the seal as a careful person would: turns each ring back to the
/// mark one press at a time, then locks it.
pub fn seal(p: &mut Prep, c: &Content) {
    let mut guard = 0;
    while let Activity::Sealing { rings, at } = &p.activity {
        let bits = if rings[*at] == 0 {
            crate::prep::NOTE[3]
        } else {
            crate::prep::LEFT
        };
        p.step(c, bits);
        p.step(c, 0);
        guard += 1;
        assert!(guard < 1000, "the seal ends");
    }
}

fn play_lane(p: &mut Prep, c: &Content) {
    let Activity::Interview { lane, .. } = &p.activity else {
        panic!("no interview")
    };
    let hits: Vec<(i32, u8)> = lane.notes.iter().map(|n| (n.at, n.lane)).collect();
    let mut t = 0;
    while p.busy() {
        t += 1;
        let bits = hits
            .iter()
            .filter(|h| h.0 == t)
            .fold(0, |b, h| b | NOTE[h.1 as usize]);
        p.step(c, bits);
    }
}

#[test]
fn a_perfect_interview_enrols_the_applicant_near_the_average_of_score_and_aptitude() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Recruit);
    let a = p.applicants[0].clone();
    let n = p.students.len();
    p.act(&c, "recruit.one", "0").unwrap();
    play_lane(&mut p, &c);
    assert_eq!(p.students.len(), n + 1);
    let s = p.students.last().unwrap();
    // quality = (100 + base) / 2 ± variance 10
    let mid = (100 + a.base) / 2;
    assert!((s.quality - mid).abs() <= 10, "quality {} vs {mid}", s.quality);
    assert_eq!(s.name, a.name);
    assert!(p.record.perfect_interview);
    // mana from the quality: 80 + 120 × q / 100
    assert_eq!(s.mana_max, 80 + 120 * s.quality / 100);
}

#[test]
fn an_interview_that_scores_under_twenty_persuades_nobody() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Recruit);
    let n = p.students.len();
    p.act(&c, "recruit.group", "").unwrap();
    while p.busy() {
        p.step(&c, 0);
    }
    assert_eq!(p.students.len(), n);
    assert!(p.applicants.is_empty(), "the applicants left");
}

/// Draws the guide back through the overlay's CSS pixels at 1280 × 720.
fn draw_guide(p: &mut Prep, c: &Content) {
    let Activity::Drawing { guide, glyphs, .. } = &p.activity else {
        panic!("not drawing")
    };
    let (ox, oy) = draw_origin(*glyphs);
    let u = units_per_px(*glyphs);
    let pts: Vec<String> = guide
        .iter()
        .map(|&(x, y)| format!("{},{},1280,720", ox + x / u, oy + y / u))
        .collect();
    for v in pts {
        p.act(c, "draw.point", &v).unwrap();
    }
    p.act(c, "draw.end", "").unwrap();
}

#[test]
fn teaching_a_class_by_drawing_the_guide_gives_each_student_near_full_effectiveness() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Podium);
    p.act(&c, "podium.teach", "").unwrap();
    let Activity::Drawing { audience, band, .. } = &p.activity else {
        panic!()
    };
    assert_eq!(*audience, 2, "the two students the last archmage left");
    // a band for 2 students: 48 + (22 − 48) × 1 / 11 = 46 px
    assert_eq!(*band, 46 * units_per_px(1));
    draw_guide(&mut p, &c);
    assert!(p.record.best_teach >= 95, "score {}", p.record.best_teach);
    for s in &p.students {
        let fb = s.known.iter().find(|k| k.0 == 0).unwrap().1;
        // score + school modifier (−20 to +20) ± variance 10, capped at 100
        assert!(fb >= p.record.best_teach - 30 && fb <= 100, "{fb}");
    }
}

#[test]
fn one_on_one_teaching_has_a_wider_band_than_a_class() {
    let c = shipped();
    let mut p = prep(&c);
    for _ in 0..10 {
        let a = p.applicant(&c);
        p.enrol(&c, &a, 50);
    }
    go(&mut p, Station::Podium);
    p.act(&c, "podium.teach", "all").unwrap();
    let Activity::Drawing { band: class, .. } = p.activity.clone() else {
        panic!()
    };
    p.activity = Activity::Idle;
    let id = p.students[0].id.to_string();
    p.act(&c, "podium.teach", &id).unwrap();
    let Activity::Drawing {
        band: one, audience, ..
    } = p.activity.clone()
    else {
        panic!()
    };
    assert_eq!(audience, 1);
    assert!(one > class, "{one} vs {class}");
}

#[test]
fn the_invasion_book_casts_only_invasions_at_a_rival_for_mana() {
    let c = shipped();
    let mut p = prep(&c);
    learn(&mut p, &c, &["invade", "ring", "skeleton", "on_death", "ghost"]);
    let parade: Vec<u16> = ["invade", "ring", "skeleton", "on_death", "ghost"]
        .iter()
        .map(|n| c.piece(n).unwrap() as u16)
        .collect();
    p.spells.push(parade.clone());
    go(&mut p, Station::Book);
    assert_eq!(p.act(&c, "book.cast", "0"), Err("book.not_invasion"));
    assert_eq!(p.act(&c, "book.cast", ""), Err("book.bad"));
    let before = p.mana;
    p.act(&c, "book.cast", "1").unwrap();
    assert_eq!(
        p.sent,
        vec![(crate::prep::EVERY_RIVAL, 1)],
        "at every rival (T29)"
    );
    // invasion_mana_pct 150: one and a half times the spell's mana, rounded down
    assert_eq!(before - p.mana, spell::stats(&c, &parade).mana * 150 / 100 * 100);
}

#[test]
fn a_wall_needs_its_spell_and_mana_and_may_not_close_the_route() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Map);
    assert_eq!(
        p.act(&c, "layout.place", "ivory_wall,10,20,0"),
        Err("layout.unknown"),
        "ivory walls come from a book"
    );
    learn(&mut p, &c, &["ivory_wall"]);
    let before = p.mana;
    // a new archmage already knows glass walls (T1)
    p.act(&c, "layout.place", "glass_wall,10,20,0").unwrap();
    // glass wall: 2 mana a cell, at once, one cell (T23)
    assert_eq!(before - p.mana, 200);
    assert!(p.placed.is_empty(), "it has not risen yet");
    assert_eq!(
        p.act(&c, "layout.place", "glass_wall,10,20,0"),
        Err("layout.taken"),
        "a rising wall holds its cell"
    );
    p.finish_raises(&c);
    assert_eq!(p.placed[0].cells, vec![(10, 20)]);
    assert_eq!(
        p.act(&c, "layout.place", "ivory_wall,10,20,0"),
        Err("layout.taken")
    );
    // a row of ivory wall cells across the field must leave a gap
    let mut err = Ok(());
    for x in 0..arena::SIZE {
        p.mana = 100_000;
        err = p.act(&c, "layout.place", &format!("ivory_wall,{x},8,0"));
        if err.is_err() {
            break;
        }
    }
    assert!(
        matches!(
            err,
            Err("layout.closes_path") | Err("layout.off_grid") | Err("layout.taken")
        ),
        "{err:?}"
    );
    p.finish_raises(&c);
    assert!(
        p.field(&c).reachable(),
        "rising walls cannot close the route between them"
    );
}

#[test]
fn a_wall_placed_with_mana_rises_after_its_delay_and_not_before() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Map);
    p.act(&c, "layout.place", "sub_tower,10,20,0").unwrap();
    // sub_tower raise_ms 10,000: 600 ticks, and the archmage may walk away
    go(&mut p, Station::Library);
    for _ in 0..599 {
        p.step(&c, 0);
    }
    assert!(p.placed.is_empty());
    p.step(&c, 0);
    assert_eq!(p.placed.len(), 1);
    assert_eq!(p.placed[0].cells, vec![(10, 20)]);
}

#[test]
fn a_new_archmage_knows_the_starting_pieces_in_the_data() {
    let c = shipped();
    let p = prep(&c);
    let known: Vec<&str> = (0..c.file.pieces.len())
        .filter(|&i| p.known[i])
        .map(|i| c.file.pieces[i].id.as_str())
        .collect();
    assert_eq!(
        known,
        vec!["fire_bolt", "ice_shard", "glass_wall", "sub_tower", "branch"]
    );
}

#[test]
fn a_golem_builds_after_its_delay_and_not_before() {
    let c = shipped();
    let mut p = prep(&c);
    learn(&mut p, &c, &["glass_wall"]);
    go(&mut p, Station::Map);
    let before = p.mana;
    p.act(&c, "layout.golem", "glass_wall,10,20,0").unwrap();
    assert_eq!(p.mana, before, "a golem costs no mana");
    assert_eq!(
        p.act(&c, "layout.golem", "glass_wall,10,22,0"),
        Err("layout.golem_busy")
    );
    // golem_s 25: 1500 ticks
    for _ in 0..1499 {
        p.step(&c, 0);
    }
    assert!(p.placed.is_empty());
    p.step(&c, 0);
    assert_eq!(p.placed.len(), 1);
}

#[test]
fn a_teaching_elemental_teaches_the_class_when_its_job_ends() {
    let c = shipped();
    let mut p = prep(&c);
    learn(&mut p, &c, &["teaching_elemental", "ice_shard"]);
    p.spells.push(vec![c.piece("ice_shard").unwrap() as u16]);
    p.chosen = 1;
    go(&mut p, Station::Podium);
    p.act(&c, "podium.summon", "teaching_elemental").unwrap();
    draw_guide(&mut p, &c);
    assert_eq!(p.jobs.len(), 1);
    // job_s 20: 1200 ticks; nobody knows the spell until then
    for _ in 0..1199 {
        p.step(&c, 0);
    }
    assert!(p.students.iter().all(|s| !s.known.iter().any(|k| k.0 == 1)));
    p.step(&c, 0);
    assert!(p.students.iter().all(|s| s.known.iter().any(|k| k.0 == 1)));
}

#[test]
fn students_who_know_an_invasion_send_it_at_their_effectiveness() {
    let c = shipped();
    let mut p = prep(&c);
    let parade: Vec<u16> = ["invade", "ring", "skeleton", "on_death", "ghost"]
        .iter()
        .map(|n| c.piece(n).unwrap() as u16)
        .collect();
    p.spells.push(parade.clone());
    p.students[0].known.push((1, 73));
    assert_eq!(p.student_invasions(&c), vec![(parade, 73)]);
    // and the battle plan does not cast it: the battle skips invasion chains
    assert!(p.plan().students[0].spells.contains(&(1, 73)));
}

#[test]
fn a_rising_pool_holds_its_cells() {
    // lava does not block the route, so only the rising list keeps a
    // second thing off its cells
    let c = shipped();
    let mut p = prep(&c);
    learn(&mut p, &c, &["lava_pool"]);
    go(&mut p, Station::Map);
    p.act(&c, "layout.place", "lava_pool,10,20,0").unwrap();
    assert_eq!(
        p.act(&c, "layout.place", "lava_pool,10,20,0"),
        Err("layout.taken")
    );
    assert_eq!(
        p.act(&c, "layout.place", "glass_wall,10,20,0"),
        Err("layout.taken")
    );
}

#[test]
fn an_interview_played_with_held_keys_hires_the_applicant() {
    // a person's tap holds a key for about six ticks; each note key starts
    // down two ticks before its note and stays down six (T19)
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Recruit);
    let before = p.students.len();
    p.act(&c, "recruit.one", "0").unwrap();
    let mut guard = 0;
    while matches!(p.activity, Activity::Interview { .. }) && guard < 10_000 {
        let mut bits = 0;
        if let Activity::Interview { lane, .. } = &p.activity {
            for n in &lane.notes {
                let d = lane.tick + 1 - n.at;
                if (-2..4).contains(&d) {
                    bits |= crate::prep::NOTE[n.lane as usize];
                }
            }
        }
        p.step(&c, bits);
        guard += 1;
    }
    assert_eq!(p.students.len(), before + 1, "held keys still hire");
}

#[test]
fn a_cell_rubbed_out_or_undone_gives_its_mana_back() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Map);
    let full = p.mana;
    for x in 5..9 {
        p.act(&c, "layout.place", &format!("glass_wall,{x},20,0"))
            .unwrap();
    }
    // four glass cells at 2 mana each
    assert_eq!(full - p.mana, 800);
    p.act(&c, "layout.erase", "6,20").unwrap();
    assert_eq!(full - p.mana, 600, "a rising cell rubbed out");
    p.finish_raises(&c);
    assert_eq!(p.placed.len(), 3);
    p.act(&c, "layout.erase", "5,20").unwrap();
    assert_eq!(p.placed.len(), 2, "a standing cell rubbed out");
    p.act(&c, "layout.undo", "").unwrap();
    assert_eq!(
        p.placed.iter().map(|q| q.cells[0]).collect::<Vec<_>>(),
        vec![(7, 20)],
        "undo takes the last laid"
    );
    p.act(&c, "layout.clear", "").unwrap();
    assert!(p.placed.is_empty());
    assert_eq!(p.mana, full, "every cell's mana is back");
}

#[test]
fn a_seal_takes_longer_when_locked_off_the_mark_and_never_changes_the_spell() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Lab);
    p.act(&c, "lab.add", "fire_bolt").unwrap();
    p.act(&c, "lab.add", "ice_shard").unwrap();
    let draft = p.draft.clone();
    p.act(&c, "lab.save", "").unwrap();
    let Activity::Sealing { rings, at } = &p.activity else {
        panic!("sealing")
    };
    assert_eq!((rings.len(), *at), (2, 0), "one ring a piece");
    let start = rings[0];
    assert!(start > 0, "a ring starts off the mark");
    // a lock off the mark turns the ring a step further
    p.step(&c, crate::prep::NOTE[3]);
    p.step(&c, 0);
    let Activity::Sealing { rings, at } = &p.activity else {
        panic!("still sealing")
    };
    assert_eq!((rings[0], *at), ((start + 1) % crate::prep::SEAL_STEPS, 0));
    seal(&mut p, &c);
    assert_eq!(
        p.spells.last(),
        Some(&draft),
        "the spell is the draft, whatever the seal"
    );
    assert!(matches!(p.activity, Activity::Idle));
}

#[test]
fn spells_on_yourself_speed_mana_hold_more_and_cost_less() {
    let c = shipped();
    let mut p = prep(&c);
    assert_eq!(p.act(&c, "self.cast", ""), Err("self.unknown"), "none known yet");
    learn(&mut p, &c, &["rising_tide", "deep_well", "quiet_mind"]);
    let b = c.balance();
    // archmage_regen 2 a second: 200 hundredths a second, 3 a tick at 60 ticks
    p.mana = 0;
    p.step(&c, 0);
    let plain = p.mana;
    // Rising Tide (20 mana, regen +100% for 60 s) by Q: the first not working
    p.mana = 10_000;
    p.act(&c, "self.cast", "").unwrap();
    assert_eq!(p.mana, 10_000 - 2000);
    p.mana = 0;
    p.step(&c, 0);
    assert_eq!(p.mana, plain * 2, "twice as fast");
    // Deep Well (30 mana, +50% held all day)
    p.mana = 10_000;
    p.act(&c, "self.cast", "").unwrap();
    assert_eq!(p.mana_cap(&c), b.archmage_mana * 100 * 150 / 100);
    // Quiet Mind (15 mana, 30% off for 30 s): a glass wall cell's 2 mana costs 1.4
    p.mana = 10_000;
    p.act(&c, "self.cast", "quiet_mind").unwrap();
    p.step(&c, 0);
    go(&mut p, Station::Map);
    let before = p.mana;
    p.act(&c, "layout.place", "glass_wall,3,3,0").unwrap();
    assert_eq!(before - p.mana, 140);
    // Rising Tide runs out after 60 s (3,600 ticks)
    for _ in 0..3600 {
        p.step(&c, 0);
    }
    assert!(!p
        .boons
        .iter()
        .any(|bn| c.file.pieces[bn.0 as usize].id == "rising_tide"));
    assert!(
        p.boons
            .iter()
            .any(|bn| c.file.pieces[bn.0 as usize].id == "deep_well"),
        "the well lasts the day"
    );
    let well = [
        c.piece("deep_well").unwrap() as u16,
        c.piece("fire_bolt").unwrap() as u16,
    ];
    assert_eq!(
        spell::check(&c, &well),
        Err(spell::ChainError::SelfOnly),
        "not a piece of a spell"
    );
}

#[test]
fn a_seat_holds_one_student_and_two_trade_when_one_takes_the_others() {
    let c = shipped();
    let mut p = prep(&c);
    let (a, b) = (p.students[0].id, p.students[1].id);
    go(&mut p, Station::Office);
    p.act(&c, "office.seat", &format!("{a},3")).unwrap();
    p.act(&c, "office.seat", &format!("{b},5")).unwrap();
    p.act(&c, "office.seat", &format!("{b},3")).unwrap();
    assert_eq!((p.students[0].seat, p.students[1].seat), (5, 3), "they trade");
    // the ground holds any number, so no one moves
    let g = crate::battle::GROUND_SEAT;
    p.act(&c, "office.seat", &format!("{a},{g}")).unwrap();
    p.act(&c, "office.seat", &format!("{b},{g}")).unwrap();
    assert_eq!((p.students[0].seat, p.students[1].seat), (g, g));
}

#[test]
fn a_seat_can_be_chosen_at_the_map_table_and_nothing_else_of_office_hours() {
    let c = shipped();
    let mut p = prep(&c);
    let a = p.students[0].id;
    go(&mut p, Station::Map);
    p.act(&c, "office.seat", &format!("{a},2")).unwrap();
    assert_eq!(p.students[0].seat, 2);
    assert_eq!(p.act(&c, "office.mode", &format!("{a},1")), Err("prep.not_here"));
    go(&mut p, Station::Library);
    assert_eq!(p.act(&c, "office.seat", &format!("{a},4")), Err("prep.not_here"));
}

#[test]
fn a_step_of_a_rotation_can_be_taken_out() {
    let c = shipped();
    let mut p = prep(&c);
    let a = p.students[0].id;
    p.spells = vec![vec![0], vec![1], vec![2]];
    go(&mut p, Station::Office);
    for k in [0, 1, 2] {
        p.act(&c, "office.rot_add", &format!("{a},{k}")).unwrap();
    }
    p.act(&c, "office.rot_del", &format!("{a},1")).unwrap();
    assert_eq!(p.students[0].rotation, vec![0, 2]);
    assert_eq!(p.act(&c, "office.rot_del", &format!("{a},2")), Err("office.bad"));
}

#[test]
fn sorting_counts_a_streak_and_the_books_on_each_shelf() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Library);
    p.act(&c, "library.read", "").unwrap();
    let book = |p: &Prep| match &p.activity {
        Activity::Sorting { book, .. } => *book,
        _ => panic!("sorting"),
    };
    let first = book(&p);
    p.act(&c, "sort.shelf", &first.to_string()).unwrap();
    let second = book(&p);
    p.act(&c, "sort.shelf", &second.to_string()).unwrap();
    let wrong = (book(&p) + 1) % c.file.schools.len() as u8;
    let Activity::Sorting { streak, per, .. } = p.activity.clone() else {
        panic!("sorting")
    };
    assert_eq!(streak, 2);
    let mut want = vec![0u8; c.file.schools.len()];
    want[first as usize] += 1;
    want[second as usize] += 1;
    assert_eq!(per, want, "one on each right shelf");
    p.act(&c, "sort.shelf", &wrong.to_string()).unwrap();
    let Activity::Sorting {
        streak, per: after, ..
    } = p.activity.clone()
    else {
        panic!("sorting")
    };
    assert_eq!(
        (streak, after),
        (0, want),
        "a miss ends the streak and shelves nothing"
    );
}

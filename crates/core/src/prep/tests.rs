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

#[test]
fn reading_takes_its_time_then_teaches_a_new_piece() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Library);
    let before = p.known.iter().filter(|&&k| k).count();
    p.act(&c, "library.read", "").unwrap();
    // read_ms 2500 is 150 ticks
    for _ in 0..149 {
        p.step(&c, 0);
    }
    assert!(p.busy());
    assert_eq!(p.known.iter().filter(|&&k| k).count(), before);
    p.step(&c, 0);
    assert!(!p.busy());
    assert!(p.known.iter().filter(|&&k| k).count() > before);
    assert_eq!(p.record.books, 1);
    // the clock ran the whole time
    assert_eq!(p.clock, TPS * 240 - 150);
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
    assert_eq!(p.spells.len(), 2, "Fire Bolt and the new spell");
    assert!(p.draft.is_empty());
    assert_eq!(p.chosen, 1);
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
    assert_eq!(p.act(&c, "book.cast", "0,1"), Err("book.not_invasion"));
    assert_eq!(p.act(&c, "book.cast", "1,0"), Err("book.bad"), "not at yourself");
    let before = p.mana;
    p.act(&c, "book.cast", "1,2").unwrap();
    assert_eq!(p.sent, vec![(2, 1)]);
    // invasion_mana_pct 150: one and a half times the spell's mana, rounded down
    assert_eq!(before - p.mana, spell::stats(&c, &parade).mana * 150 / 100 * 100);
}

#[test]
fn a_wall_needs_its_spell_and_mana_and_may_not_close_the_route() {
    let c = shipped();
    let mut p = prep(&c);
    go(&mut p, Station::Map);
    assert_eq!(
        p.act(&c, "layout.place", "glass_wall,10,20,0"),
        Err("layout.unknown")
    );
    learn(&mut p, &c, &["glass_wall", "ivory_wall"]);
    let before = p.mana;
    p.act(&c, "layout.place", "glass_wall,10,20,0").unwrap();
    // glass wall: 20 mana, 3 cells along x
    assert_eq!(before - p.mana, 2000);
    assert_eq!(p.placed[0].cells, vec![(10, 20), (11, 20), (12, 20)]);
    assert_eq!(
        p.act(&c, "layout.place", "glass_wall,11,20,0"),
        Err("layout.taken")
    );
    // a row of ivory walls across the field must leave a gap
    let mut err = Ok(());
    for x in (0..arena::SIZE).step_by(3) {
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
    assert!(p.field(&c).reachable());
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

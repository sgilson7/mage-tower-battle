//! Match tests.

use super::*;
use crate::content::shipped;
use crate::prep::{station_pos, Station, BODY, PROP};

fn quick(c: &Content) -> Settings {
    let mut s = Settings::from_content(c);
    s.days = 2;
    s.prep_s = 20;
    s.battle_s = 30;
    s
}

fn versus(c: &Content) -> Match {
    let kinds = [
        SeatKind::Local,
        SeatKind::Bot(Style::Spellcrafter),
        SeatKind::Bot(Style::AttackMage),
        SeatKind::Bot(Style::MassTeacher),
    ];
    let names = ["You", "A", "B", "C"].map(String::from);
    Match::new(c, 42, Mode::Versus, quick(c), kinds, names, 0)
}

fn go(p: &mut Prep, s: Station) {
    let at = station_pos(s);
    p.pos = at.add(
        crate::geom::V3::ZERO
            .sub(at)
            .flat()
            .with_len(PROP + BODY + crate::fx::ONE / 10),
    );
}

#[test]
fn a_versus_match_runs_its_days_and_scores_only_fall() {
    let c = shipped();
    let mut m = versus(&c);
    let start = c.balance().score_start;
    let mut last = [start; 4];
    let mut days_seen = 0;
    for _ in 0..TPS * 400 {
        m.step(&c, 0);
        for (i, s) in m.seats.iter().enumerate() {
            assert!(s.score <= last[i], "seat {i} gained score");
            last[i] = s.score;
        }
        if m.phase == Phase::Summary {
            days_seen += 1;
            m.next_day(&c);
        }
        if m.phase == Phase::Results {
            break;
        }
    }
    assert_eq!(m.phase, Phase::Results);
    assert_eq!(days_seen, 2);
    assert_eq!(m.results.len(), 8, "four towers, two days");
    for s in &m.seats {
        assert_eq!(s.score, start - s.taken.iter().sum::<i32>());
    }
}

#[test]
fn an_archmages_invasion_shows_at_once_and_students_only_when_the_battle_starts() {
    let c = shipped();
    let kinds = [SeatKind::Local, SeatKind::Local, SeatKind::Empty, SeatKind::Empty];
    let names = ["You", "Rival", "", ""].map(String::from);
    let mut m = Match::new(&c, 3, Mode::Practice, quick(&c), kinds, names, 0);
    let parade: Vec<u16> = ["invade", "ring", "skeleton", "on_death", "ghost"]
        .iter()
        .map(|n| c.piece(n).unwrap() as u16)
        .collect();
    // seat 1's students know the parade; seat 0's archmage casts it at seat 1
    {
        let p1 = m.seats[1].prep.as_mut().unwrap();
        p1.spells.push(parade.clone());
        p1.students[0].known.push((1, 60));
    }
    let p0 = m.seats[0].prep.as_mut().unwrap();
    p0.spells.push(parade.clone());
    for &piece in &parade {
        p0.known[piece as usize] = true;
    }
    go(p0, Station::Book);
    p0.act(&c, "book.cast", "1,1").unwrap();
    m.step(&c, 0);
    // seat 1's book shows seat 0's archmage invasion now, mid-preparation
    assert_eq!(m.phase, Phase::Prep);
    let theirs = m.book_of(1);
    assert_eq!(theirs.len(), 1);
    assert_eq!(theirs[0].from, 0);
    // seat 0 sees nothing yet from seat 1's students
    assert!(m.book().is_empty());
    m.begin_battle(&c);
    let to_me: Vec<&Incoming> = m.book();
    assert_eq!(to_me.len(), 1, "seat 1's student invasion arrives at the start");
    assert_eq!((to_me[0].from, to_me[0].strength), (1, 60));
}

#[test]
fn students_invasions_go_to_the_rival_leading_on_score() {
    let c = shipped();
    let mut m = versus(&c);
    m.seats[1].score = 9000;
    m.seats[2].score = 9500;
    m.seats[3].score = 8000;
    m.seats[0].score = 9500;
    // from seat 3: seats 0 and 2 tie at 9500; the lower seat leads
    assert_eq!(m.leader(3), 0);
    assert_eq!(m.leader(0), 2);
}

#[test]
fn the_ranking_orders_by_score_and_a_tie_is_shared() {
    let c = shipped();
    let mut m = versus(&c);
    m.seats[0].score = 7000;
    m.seats[1].score = 9000;
    m.seats[2].score = 9000;
    m.seats[3].score = 8000;
    assert_eq!(m.ranking(), vec![1, 2, 3, 0]);
    assert!(m.won(1) && m.won(2) && !m.won(3));
}

#[test]
fn no_two_days_share_a_main_enemy() {
    let c = shipped();
    let mut m = versus(&c);
    // eight wild kinds: eight days must use each once
    m.settings.days = 8;
    let mut mains = vec![m.wild[0].enemy];
    for _ in 0..7 {
        m.day += 1;
        let w = m.draw_wild(&c);
        mains.push(w[0].enemy);
    }
    let mut sorted = mains.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 8, "{mains:?}");
}

#[test]
fn medals_follow_each_trials_ladder() {
    let c = shipped();
    // errand run, time: 150 / 120 / 95 / 80 s
    assert_eq!(medal(&c, 0, 200), 0);
    assert_eq!(medal(&c, 0, 150), 1);
    assert_eq!(medal(&c, 0, 100), 2);
    assert_eq!(medal(&c, 0, 81), 3);
    assert_eq!(medal(&c, 0, 80), 4);
    // necromancer's exam, damage dealt: 300 / 700 / 1200 / 1800
    assert_eq!(medal(&c, 3, 299), 0);
    assert_eq!(medal(&c, 3, 1200), 3);
    assert!(better(&c, 0, 90, Some(100)) && !better(&c, 3, 90, Some(100)));
}

#[test]
fn the_crash_course_ends_when_the_class_averages_eighty() {
    let c = shipped();
    let mut m = Match::trial(&c, 1);
    assert_eq!(m.seats[0].prep.as_ref().unwrap().students.len(), 8);
    m.step(&c, 0);
    assert_eq!(m.phase, Phase::Prep);
    let p = m.seats[0].prep.as_mut().unwrap();
    for s in p.students.iter_mut() {
        s.known.push((1, 80));
    }
    m.elapsed = TPS * 33;
    m.step(&c, 0);
    assert_eq!(m.phase, Phase::Results);
    assert_eq!(m.trial_done, Some(33));
}

#[test]
fn invasions_reach_a_battle_in_sender_order_whatever_order_they_arrived_in() {
    let c = shipped();
    // an invasion chain a bot would cast
    let mut p = Prep::new(&c, 1, 5, 5);
    p.start_day(&c, 600 * TPS);
    let mut rng = Rng::new(3);
    bot::day(&c, &mut p, Style::AttackMage, 90, 0, true, &mut rng);
    let chain = p
        .spells
        .iter()
        .find(|s| spell::is_invasion(&c, s))
        .cloned()
        .expect("an invasion");
    let kinds = [
        SeatKind::Local,
        SeatKind::Remote,
        SeatKind::Remote,
        SeatKind::Empty,
    ];
    let names = ["You", "A", "B", ""].map(String::from);
    let froms = |order: [u8; 2]| {
        let mut m = Match::new(&c, 9, Mode::Online, quick(&c), kinds, names.clone(), 0);
        for from in order {
            m.take_invasion(&c, from, 0, chain.clone());
        }
        m.take_plan(1, Plan::default(), vec![]);
        m.take_plan(2, Plan::default(), vec![]);
        m.begin_battle(&c);
        m.seats[0]
            .incoming
            .iter()
            .map(|(i, _)| i.from)
            .collect::<Vec<u8>>()
    };
    assert_eq!(froms([2, 1]), vec![1, 2]);
    assert_eq!(froms([1, 2]), vec![1, 2]);
}

#[test]
fn an_archmage_s_invasion_reaches_every_rival() {
    let c = shipped();
    let mut m = versus(&c);
    let parade: Vec<u16> = ["invade", "skeleton"]
        .iter()
        .map(|n| c.piece(n).unwrap() as u16)
        .collect();
    let p = m.local_prep_mut().unwrap();
    p.spells.push(parade.clone());
    let k = p.spells.len() - 1;
    p.sent.push((crate::prep::EVERY_RIVAL, k as u16));
    m.step(&c, 0);
    for seat in 1..4 {
        assert!(
            m.seats[seat]
                .incoming
                .iter()
                .any(|(i, _)| i.from == 0 && i.chain == parade),
            "seat {seat} receives it"
        );
    }
    assert!(
        m.seats[0].incoming.iter().all(|(i, _)| i.from != 0),
        "not yourself"
    );
    // each rival gets invasion_spread_pct 35 of archmage_count_pct 150: 52
    let got = m.seats[1]
        .incoming
        .iter()
        .find(|(i, _)| i.from == 0)
        .unwrap()
        .0
        .count_pct;
    assert_eq!(got, 150 * 35 / 100);
}

#[test]
fn a_harder_difficulty_brings_more_and_tougher_wild_enemies() {
    let c = shipped();
    let at = |d: i32| {
        let mut s = quick(&c);
        s.difficulty = d;
        let kinds = [SeatKind::Local, SeatKind::Empty, SeatKind::Empty, SeatKind::Empty];
        let m = Match::new(&c, 42, Mode::Practice, s, kinds, Default::default(), 0);
        (m.wild.iter().map(|w| w.count).sum::<i32>(), m.wild[0].strength)
    };
    let (easy, normal, brutal) = (at(0), at(1), at(3));
    assert!(
        easy.0 < normal.0 && normal.0 < brutal.0,
        "{easy:?} {normal:?} {brutal:?}"
    );
    // health_pct from content.json: easy 85, normal 105, brutal 150
    assert_eq!((easy.1, normal.1, brutal.1), (85, 105, 150));
    assert_eq!(Settings::from_content(&c).difficulty, 1, "normal by default");
}

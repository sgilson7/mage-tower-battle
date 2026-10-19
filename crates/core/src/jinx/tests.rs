//! Jinx and walkway tests (T62), with expected values worked out from
//! data/content.json in the comments: walking 450 cm/s, a walkway 150%, a
//! bog 33% within 220 cm, ice 3% grip, a barrier 400 by 80 cm, a jinx 30
//! to 60 s, two of yours in a rival's tower at once, 25% of a jinx's
//! seconds from a computer rival's next day.

use super::*;
use crate::content::shipped;
use crate::prep::{Prep, LEFT, RIGHT};

fn prep(c: &Content) -> Prep {
    let mut p = Prep::new(c, 0, 11, 11);
    p.start_day(c, TPS * 240);
    p
}

/// Standing at the invasion book, on the side toward the room's centre.
fn at_book() -> V3 {
    let at = station_pos(Station::Book);
    at.add(V3::ZERO.sub(at).flat().with_len(PROP + BODY + ONE / 10))
}

fn piece(c: &Content, id: &str) -> u16 {
    c.piece(id).unwrap() as u16
}

fn jinx(c: &Content, id: &str, x: i32, z: i32, turn: u8) -> Jinx {
    let shape = shape(c, piece(c, id)).unwrap();
    let way = fits(c, shape, x, z, turn).unwrap();
    Jinx {
        piece: piece(c, id),
        from: 1,
        x,
        z,
        turn,
        way,
        left: 40 * TPS,
    }
}

fn cm_of(v: Fx) -> i32 {
    (v as i64 * 100 / ONE as i64) as i32
}

#[test]
fn walking_on_a_walkway_is_half_again_as_fast() {
    let c = shipped();
    let mut p = prep(&c);
    // the middle of the walkway from the invasion book to the laboratory
    let (a, b) = walkway_ends(5);
    p.pos = a.add(b.sub(a).ratio(1, 2));
    let start = p.pos;
    p.step(&c, RIGHT);
    // 150% of 450 cm/s is 675 cm/s: one tick of cms(675)
    assert_eq!(p.pos.x - start.x, content::cms(675));
}

#[test]
fn a_bog_slows_walking_to_a_third() {
    let c = shipped();
    let mut p = prep(&c);
    // the archmage starts at (0, 300) cm, on no walkway
    let start = p.pos;
    p.jinxes.push(jinx(&c, "bog", 0, 300, 0));
    p.step(&c, RIGHT);
    // 33% of a tick at 450 cm/s: 4915 Q16 × 33 / 100 = 1621
    assert_eq!(p.pos.x - start.x, content::cms(450) * 33 / 100);
}

#[test]
fn on_black_ice_the_archmage_slides_on_after_the_keys_are_let_go() {
    let c = shipped();
    let (a, b) = walkway_ends(5);
    let mid = a.add(b.sub(a).ratio(1, 2));
    // the same walk with ice and without: a second along the walkway, then
    // a tick with no key
    let run = |ice: bool| {
        let mut p = prep(&c);
        p.pos = mid;
        if ice {
            let (x, z) = (cm_of(mid.x), cm_of(mid.z));
            p.jinxes.push(jinx(&c, "black_ice", x, z, 0));
        }
        for _ in 0..TPS / 2 {
            p.step(&c, RIGHT);
        }
        let before = p.pos;
        p.step(&c, 0);
        p.pos.sub(before).flat_len()
    };
    assert_eq!(run(false), 0, "off the ice the archmage stops at once");
    assert!(run(true) > 0, "on the ice the archmage slides on");
}

#[test]
fn black_ice_is_slow_to_answer_the_keys() {
    let c = shipped();
    let (a, b) = walkway_ends(5);
    let mid = a.add(b.sub(a).ratio(1, 2));
    let mut p = prep(&c);
    p.pos = mid;
    p.jinxes
        .push(jinx(&c, "black_ice", cm_of(mid.x), cm_of(mid.z), 0));
    let start = p.pos;
    p.step(&c, RIGHT);
    // the first tick closes 3% of the gap to 675 cm/s: 20 cm/s (20.25)
    assert_eq!(p.pos.x - start.x, content::cms(675) * 3 / 100);
}

#[test]
fn a_barrier_stops_the_archmage_at_its_face() {
    let c = shipped();
    let mut p = prep(&c);
    // the archmage at (0, 300) cm walks left into a barrier at (-200, 300)
    // turned a quarter, so it runs along z: its near face is 40 cm from
    // its middle, and the archmage's body 40 cm more, so they stop at -120
    p.jinxes.push(jinx(&c, "barrier", -200, 300, 2));
    for _ in 0..TPS * 2 {
        p.step(&c, LEFT);
    }
    assert!((p.pos.x - content::cm(-120)).abs() <= 2, "{}", cm_of(p.pos.x));
}

#[test]
fn a_barrier_that_lands_on_the_archmage_puts_them_beside_it() {
    let c = shipped();
    let mut p = prep(&c);
    // standing at (0, 300); a barrier along x through (0, 290) is 80 cm
    // deep, so the nearest way out is across it: to z = 290 + 40 + 40
    p.jinxes.push(jinx(&c, "barrier", 0, 290, 0));
    p.step(&c, 0);
    assert!((p.pos.z - content::cm(370)).abs() <= 2, "{}", cm_of(p.pos.z));
}

#[test]
fn a_barrier_cannot_cover_a_station_or_leave_the_room() {
    let c = shipped();
    let (lab_x, lab_z) = (1015, 96);
    assert_eq!(
        fits(&c, JinxShape::Barrier, lab_x - 150, lab_z, 2),
        Err("jinx.on_station")
    );
    assert_eq!(fits(&c, JinxShape::Barrier, 1350, 0, 2), Err("jinx.outside"));
    assert_eq!(fits(&c, JinxShape::Ice, 0, 1300, 0), Err("jinx.no_walkway"));
}

#[test]
fn a_computer_rival_s_jinx_fits_on_every_walkway() {
    let c = shipped();
    for way in 0..WALKWAYS.len() {
        for sh in [JinxShape::Barrier, JinxShape::Bog, JinxShape::Ice] {
            let (x, z, turn) = spot(sh, way);
            assert!(fits(&c, sh, x, z, turn).is_ok(), "{sh:?} on walkway {way}");
        }
    }
}

#[test]
fn a_jinx_lasts_thirty_to_sixty_seconds_by_the_drawing() {
    let c = shipped();
    assert_eq!(seconds(&c, 0), 30);
    assert_eq!(seconds(&c, 50), 45);
    assert_eq!(seconds(&c, 100), 60);
}

#[test]
fn a_jinx_drawn_at_the_book_is_paid_for_and_sent_to_its_rival() {
    let c = shipped();
    let mut p = prep(&c);
    p.pos = at_book();
    let mana = p.mana;
    // a bog on the walkway from the podium to the book, at the Attack
    // Mage's seat 2; a drawing of nothing scores 0, so 30 s
    p.act(&c, "book.jinx", &format!("{},2,-477,-30,0", piece(&c, "bog")))
        .unwrap();
    assert!(p.busy(), "the jinx's drawing begins");
    p.act(&c, "draw.end", "").unwrap();
    assert_eq!(p.mana, mana - 20 * 100, "a bog costs 20 mana");
    assert_eq!(p.jinx_out.len(), 1);
    let (to, j) = p.jinx_out[0];
    assert_eq!((to, j.left), (2, 30 * TPS));
    assert_eq!(p.cast.len(), 1, "it counts toward that tower's jinxes");
}

#[test]
fn two_of_your_jinxes_stand_in_a_rival_s_tower_at_once() {
    let c = shipped();
    let mut p = prep(&c);
    p.pos = at_book();
    let bog = piece(&c, "bog");
    for _ in 0..2 {
        p.act(&c, "book.jinx", &format!("{bog},1,0,600,0")).unwrap();
        p.act(&c, "draw.end", "").unwrap();
    }
    assert_eq!(
        p.act(&c, "book.jinx", &format!("{bog},1,0,600,0")),
        Err("jinx.full")
    );
    // another rival's tower still has room, and your own is not a target
    assert!(p.act(&c, "book.jinx", &format!("{bog},3,0,600,0")).is_ok());
    p.act(&c, "prep.cancel", "").unwrap();
    assert_eq!(
        p.act(&c, "book.jinx", &format!("{bog},0,0,600,0")),
        Err("jinx.bad")
    );
}

#[test]
fn a_jinx_runs_out_and_the_next_day_clears_the_rest() {
    let c = shipped();
    let mut p = prep(&c);
    let mut j = jinx(&c, "bog", 0, 600, 0);
    j.left = 2;
    p.jinxes.push(j);
    p.jinxes.push(jinx(&c, "barrier", 0, -600, 0));
    p.step(&c, 0);
    p.step(&c, 0);
    assert_eq!(p.jinxes.len(), 1, "the bog with two ticks left is gone");
    p.start_day(&c, TPS * 240);
    assert!(p.jinxes.is_empty());
}

#[test]
fn a_jinx_s_landing_says_who_cast_it() {
    let c = shipped();
    let mut p = prep(&c);
    p.take_jinx(&c, jinx(&c, "bog", 0, 600, 0), "The Attack Mage");
    let n = p.notes.last().unwrap();
    assert_eq!(
        (n.key.as_str(), n.value.as_str(), n.who.as_str()),
        ("jinx.landed", "Bog", "The Attack Mage")
    );
}

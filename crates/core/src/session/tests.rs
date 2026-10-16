//! Session tests: every screen draws with every string found, and a short
//! match can be played through the one door from the title to the results.

use super::*;

fn session() -> Session {
    Session::new(
        include_str!("../../../../data/strings.json"),
        include_str!("../../../../data/content.json"),
        include_str!("../../../../data/palette.json"),
        include_str!("../../../../data/models.json"),
        include_str!("../../../../data/sounds.json"),
        include_str!("../../../../GLOSSARY.md"),
        "",
        42,
    )
    .expect("session")
}

/// Every string in a view model, with none missing from strings.json.
fn no_missing(v: &Value) {
    let text = v.to_string();
    let re_like = text.match_indices('[').filter(|(i, _)| {
        text[i + 1..]
            .chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false)
    });
    for (i, _) in re_like {
        let end = text[i..].find(']').map(|e| i + e).unwrap_or(i);
        let inner = &text[i + 1..end];
        assert!(
            !(inner.contains('.') && !inner.contains(' ')),
            "missing string [{inner}]"
        );
    }
    assert!(
        !text.contains("{n}") && !text.contains("{value}") && !text.contains("{name}"),
        "an unfilled placeholder: {text}"
    );
}

fn run(s: &mut Session, ticks: i32) {
    for _ in 0..ticks {
        let now = s.tick as i64 * 16_667 + 16_667;
        s.frame(now, 0);
    }
}

#[test]
fn every_screen_draws_with_its_strings() {
    let mut s = session();
    for screen in ["title", "setup", "trials", "achievements", "help"] {
        s.act("screen.show", screen).unwrap();
        no_missing(&s.ui());
    }
    assert!(s.act("screen.show", "nowhere").is_err());
    assert!(s.act("no.such", "").is_err());
}

#[test]
fn a_practice_day_runs_from_the_title_to_its_results() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    assert_eq!(s.ui()["phase"], "prep");
    // the library, then the laboratory
    s.teleport(Station::Library);
    run(&mut s, 2);
    no_missing(&s.ui());
    s.act("library.read", "").unwrap();
    run(&mut s, 160);
    s.teleport(Station::Lab);
    run(&mut s, 2);
    no_missing(&s.ui());
    s.act("lab.add", "fire_bolt").unwrap();
    s.act("lab.add", "fire_bolt").unwrap();
    s.act("lab.save", "").unwrap();
    // the podium: draw a straight line across the board through the pointer
    s.teleport(Station::Podium);
    run(&mut s, 2);
    s.act("podium.teach", "all").unwrap();
    assert!(!s.overlay().is_empty());
    no_missing(&s.ui());
    s.act("pointer.down", "300,380,1280,720").unwrap();
    for x in (300..1000).step_by(20) {
        s.act("pointer.move", &format!("{x},380,1280,720")).unwrap();
    }
    s.act("pointer.up", "1000,380,1280,720").unwrap();
    assert!(s.overlay().is_empty(), "the board closes when the stroke ends");
    // the map: open it and place nothing known; then start the battle
    s.teleport(Station::Map);
    run(&mut s, 2);
    s.act("layout.open", "").unwrap();
    no_missing(&s.ui());
    s.act("layout.close", "").unwrap();
    s.act("day.ready", "").unwrap();
    run(&mut s, 3);
    assert_eq!(s.ui()["phase"], "battle");
    no_missing(&s.ui());
    run(&mut s, TPS * 95);
    assert_eq!(s.ui()["phase"], "summary");
    no_missing(&s.ui());
    s.act("day.next", "").unwrap();
    let ui = s.ui();
    assert_eq!(ui["phase"], "results", "a practice match is one day");
    no_missing(&ui);
    // the score fell by what the tower took, and only by that
    let b = s.local_battle().unwrap();
    let taken = b.tally.damage_taken;
    assert_eq!(ui["data"]["score"], json!(s.c.balance().score_start - taken));
}

#[test]
fn a_versus_day_shows_four_towers_and_the_summary() {
    let mut s = session();
    s.settings.prep_s = 60;
    s.settings.battle_s = 30;
    s.act("match.start", "versus").unwrap();
    s.act("day.ready", "").unwrap();
    run(&mut s, 3);
    assert_eq!(s.ui()["phase"], "battle");
    let views = s.frame_out().commands.chunks(6).filter(|c| c[0] == 0).count();
    assert_eq!(views, 4, "the watch screen draws four viewports");
    s.act("view.focus", "2").unwrap();
    run(&mut s, 1);
    assert_eq!(s.frame_out().commands.chunks(6).filter(|c| c[0] == 0).count(), 1);
    run(&mut s, TPS * 35);
    let ui = s.ui();
    assert_eq!(ui["phase"], "summary");
    no_missing(&ui);
    s.act("day.next", "").unwrap();
    assert_eq!(s.ui()["phase"], "prep");
}

#[test]
fn a_trial_result_is_kept_as_a_best_and_saved() {
    let mut s = session();
    s.act("trial.start", "1").unwrap();
    // finish the crash course at once by hand
    if let Some(p) = s.m.as_mut().and_then(|m| m.local_prep_mut()) {
        for st in p.students.iter_mut() {
            st.known.push((1, 90));
        }
    }
    run(&mut s, 2);
    let ui = s.ui();
    assert_eq!(ui["phase"], "results");
    no_missing(&ui);
    assert!(s.save.bests.contains_key("crash_course"));
    let saved = s.take_save();
    assert!(saved.contains("crash_course"));
    assert_eq!(s.take_save(), "", "saved once until something changes");
}

#[test]
fn every_sound_renders_and_a_muted_game_plays_none() {
    let mut s = session();
    for i in 0..s.sound_count() {
        assert!(s.sound_wav(i).len() > 44);
    }
    s.act("sound.mute", "").unwrap();
    s.act("match.start", "practice").unwrap();
    run(&mut s, 2);
    assert!(s.sounds().is_empty());
}

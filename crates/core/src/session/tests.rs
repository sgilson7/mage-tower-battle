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
        include_str!("../../../../data/news.json"),
        "",
        42,
    )
    .expect("session")
}

/// Every string in a view model, with none missing from strings.json.
/// A panel's blocks of one type (T32), in order.
fn blocks(panel: &Value, t: &str) -> Vec<Value> {
    panel["blocks"]
        .as_array()
        .map(|b| b.iter().filter(|x| x["t"] == t).cloned().collect())
        .unwrap_or_default()
}

fn find_panel(s: &Session, id: &str) -> Option<Value> {
    s.ui()["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .cloned()
}

/// Every button a panel holds, wherever it sits.
fn buttons_in(v: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    match v {
        Value::Object(m) => {
            if m.contains_key("name") && m.contains_key("enabled") {
                out.push(v.clone());
            }
            for x in m.values() {
                out.extend(buttons_in(x));
            }
        }
        Value::Array(a) => {
            for x in a {
                out.extend(buttons_in(x));
            }
        }
        _ => {}
    }
    out
}

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

/// Plays the laboratory's seal through the session (T22).
fn seal(s: &mut Session) {
    let mut guard = 0;
    loop {
        let bits = match s.m.as_ref().and_then(|m| m.local_prep()).map(|p| &p.activity) {
            Some(Activity::Sealing { rings, at }) if rings[*at] == 0 => prep::NOTE[3],
            Some(Activity::Sealing { .. }) => prep::LEFT,
            _ => break,
        };
        for b in [bits, 0] {
            let now = s.tick as i64 * 16_667 + 16_667;
            s.frame(now, b);
        }
        guard += 1;
        assert!(guard < 1000, "the seal ends");
    }
}

/// Plays the library's sorting game through the session (T20).
fn sort(s: &mut Session) {
    let mut guard = 0;
    while let Some(Activity::Sorting { book, .. }) =
        s.m.as_ref()
            .and_then(|m| m.local_prep())
            .map(|p| p.activity.clone())
    {
        s.act("sort.shelf", &book.to_string()).unwrap();
        guard += 1;
        assert!(guard < 100, "the sort ends");
    }
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
    for screen in ["title", "setup", "trials", "achievements", "help", "online"] {
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
    assert!(!s.overlay().is_empty(), "the sorting game is drawn");
    no_missing(&s.ui());
    sort(&mut s);
    run(&mut s, 2);
    s.teleport(Station::Lab);
    run(&mut s, 2);
    no_missing(&s.ui());
    s.act("lab.add", "fire_bolt").unwrap();
    s.act("lab.add", "fire_bolt").unwrap();
    s.act("lab.save", "").unwrap();
    assert!(!s.overlay().is_empty(), "the seal is drawn");
    no_missing(&s.ui());
    seal(&mut s);
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

// ---- online play ----

/// A wire between a host session and joiner sessions (peer k+1 at the host,
/// peer 100 at each joiner): delivers every outbox, in order.
fn wire(host: &mut Session, joiners: &mut [Session]) {
    for _ in 0..4 {
        let out: Vec<(u32, String, bool)> = serde_json::from_str(&host.net_outbox()).unwrap();
        for (peer, text, _) in out {
            joiners[(peer - 1) as usize].net_message(100, &text);
        }
        for (k, j) in joiners.iter_mut().enumerate() {
            let out: Vec<(u32, String, bool)> = serde_json::from_str(&j.net_outbox()).unwrap();
            for (_, text, _) in out {
                host.net_message(k as u32 + 1, &text);
            }
        }
    }
}

fn online_room(n: usize) -> (Session, Vec<Session>) {
    let mut host = session();
    host.settings.days = 2;
    host.settings.prep_s = 60;
    host.settings.battle_s = 30;
    let room = host.host(7, 9, "app");
    let code = room.rsplit('-').next().unwrap().to_string();
    let mut js: Vec<Session> = (0..n).map(|_| session()).collect();
    for (k, j) in js.iter_mut().enumerate() {
        j.act("online.name", &format!("Wren {k}")).unwrap();
        assert_eq!(j.join(&code, "app"), room, "the same room on the relays");
        host.net_peer(k as u32 + 1);
        j.net_peer(100);
    }
    wire(&mut host, &mut js);
    (host, js)
}

/// Runs every session a tick at a time, wired, with the joiners' archmages
/// played by the Attack Mage bot so invasions cross the wire.
fn play_online(host: &mut Session, js: &mut [Session], ticks: i32) {
    for _ in 0..ticks {
        for j in js.iter_mut() {
            let c = j.c.clone();
            if let Some(m) = j.m.as_mut() {
                let (seat, day, phase) = (m.local, m.day, m.phase);
                let target = m.leader(seat).max(0) as u8;
                if phase == Phase::Prep {
                    if let Some(p) = m.local_prep_mut() {
                        if !p.ready {
                            let mut rng = crate::rng::Rng::new(day as u64 * 31 + seat as u64);
                            // what the pile hands out depends on the seed; an
                            // invasion must cross the wire, so they know one
                            for piece in ["invade", "skeleton"] {
                                let k = c.piece(piece).unwrap();
                                p.known[k] = true;
                            }
                            crate::bot::day(&c, p, Style::AttackMage, 80, target, true, &mut rng);
                            p.ready = true;
                        }
                    }
                }
            }
        }
        host.act("day.ready", "").ok();
        run(host, 1);
        for j in js.iter_mut() {
            run(j, 1);
        }
        wire(host, js);
        let summary = |s: &Session| s.m.as_ref().map(|m| m.phase == Phase::Summary).unwrap_or(false);
        if summary(host) && js.iter().all(summary) {
            host.act("day.next", "").unwrap();
            for j in js.iter_mut() {
                j.act("day.next", "").unwrap();
            }
        }
    }
}

#[test]
fn the_lobby_shows_every_seat_and_starts_only_when_everyone_is_ready() {
    let (mut host, mut js) = online_room(2);
    let ui = host.ui();
    no_missing(&ui);
    assert_eq!(ui["screen"], "online");
    let rows = ui["panels"][0]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    assert!(rows[1]["label"].as_str().unwrap().starts_with("Wren 0"));
    assert!(host.act("online.start", "").is_err(), "the joiners are not ready");
    for j in js.iter_mut() {
        no_missing(&j.ui());
        j.act("online.ready", "").unwrap();
    }
    wire(&mut host, &mut js);
    host.act("online.start", "").unwrap();
    wire(&mut host, &mut js);
    run(&mut host, 1);
    for j in js.iter_mut() {
        run(j, 1);
        let m = j.m.as_ref().expect("the start reached the joiner");
        assert_eq!(m.mode, Mode::Online);
        assert_eq!(m.seats[0].kind, SeatKind::Remote);
        assert_eq!(m.settings.days, 2, "the host's settings");
    }
    let m = host.m.as_ref().unwrap();
    assert_eq!(m.seats[1].kind, SeatKind::Remote);
    assert_eq!(m.seats[2].kind, SeatKind::Remote);
    assert!(
        matches!(m.seats[3].kind, SeatKind::Bot(_)),
        "the open seat is a computer rival"
    );
    assert_eq!(m.seats[1].name, "Wren 0");
}

#[test]
fn every_browser_of_an_online_match_ends_with_the_same_scores() {
    let (mut host, mut js) = online_room(2);
    for j in js.iter_mut() {
        j.act("online.ready", "").unwrap();
    }
    wire(&mut host, &mut js);
    host.act("online.start", "").unwrap();
    wire(&mut host, &mut js);
    play_online(&mut host, &mut js, TPS * 200);
    let scores = |s: &Session| -> Vec<i32> { s.m.as_ref().unwrap().seats.iter().map(|x| x.score).collect() };
    let hm = host.m.as_ref().unwrap();
    assert_eq!(hm.phase, Phase::Results, "two days ran to the results");
    let from_joiners: i32 = hm
        .results
        .iter()
        .filter(|r| r.seat != 1 && r.seat != 2)
        .map(|r| r.from_invasions)
        .sum();
    assert!(
        from_joiners > 0,
        "the joiners' invasions crossed the wire and did damage"
    );
    for j in &js {
        assert_eq!(scores(j), scores(&host), "the same scores in every browser");
        assert_eq!(j.m.as_ref().unwrap().checksum(), hm.checksum());
        assert!(!j.desync_told);
    }
    assert!(!host.desync_told);
    assert!(
        hm.seats.iter().all(|s| s.score < 10_000),
        "every tower took damage"
    );
}

#[test]
fn a_joiner_who_leaves_mid_match_does_not_stop_the_others() {
    let (mut host, mut js) = online_room(2);
    for j in js.iter_mut() {
        j.act("online.ready", "").unwrap();
    }
    wire(&mut host, &mut js);
    host.act("online.start", "").unwrap();
    wire(&mut host, &mut js);
    run(&mut host, 2);
    // seat 2 closes the page before sending a plan
    js.pop();
    host.net_left(2);
    let mut rest = js;
    play_online(&mut host, &mut rest, TPS * 200);
    let hm = host.m.as_ref().unwrap();
    assert_eq!(hm.phase, Phase::Results);
    assert!(hm.seats[2].gone);
    assert_eq!(
        hm.seats.iter().map(|s| s.score).collect::<Vec<_>>(),
        rest[0]
            .m
            .as_ref()
            .unwrap()
            .seats
            .iter()
            .map(|s| s.score)
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_different_checksum_from_another_browser_is_told_once() {
    let (mut host, mut js) = online_room(1);
    js[0].act("online.ready", "").unwrap();
    wire(&mut host, &mut js);
    host.act("online.start", "").unwrap();
    wire(&mut host, &mut js);
    run(&mut host, 1);
    host.sums.push((0, 0, 1));
    host.sums.push((0, 1, 2));
    host.check_sums();
    assert!(host.desync_told);
    let n = host.toasts.len();
    host.check_sums();
    assert_eq!(host.toasts.len(), n, "told once");
}

#[test]
fn e_opens_the_map_at_the_map_table_and_esc_closes_it() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Map);
    let tick = |s: &mut Session, bits: u32| {
        let now = s.tick as i64 * 16_667 + 16_667;
        s.frame(now, bits);
    };
    tick(&mut s, prep::USE);
    assert!(s.layout.open, "E at the map table opens the map");
    let ui = s.ui();
    let panel = ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "layout")
        .cloned()
        .unwrap();
    assert_eq!(
        panel["kind"], "map",
        "the map's panel is not a side panel the overlay hides"
    );
    assert!(buttons_in(&panel).iter().any(|a| a["name"] == "layout.close"));
    tick(&mut s, prep::BACK | prep::USE);
    assert!(!s.layout.open, "Esc closes it, even with E held");
    tick(&mut s, prep::USE);
    assert!(!s.layout.open, "an E held through the Esc does not open it again");
    tick(&mut s, 0);
    tick(&mut s, prep::USE);
    assert!(s.layout.open, "a fresh E opens it again");
    s.act("layout.close", "").unwrap();
    assert!(!s.layout.open, "the button closes it");
    // walking off closes it too
    s.act("layout.open", "").unwrap();
    s.teleport(Station::Library);
    tick(&mut s, 0);
    assert!(!s.layout.open, "leaving the map table closes the map");
}

#[test]
fn the_glossary_opens_over_any_screen_lists_every_piece_and_pauses_a_solo_day() {
    let mut s = session();
    s.act("glossary.toggle", "").unwrap();
    let ui = s.ui();
    no_missing(&ui);
    let g = ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "glossary")
        .cloned()
        .expect("over the title");
    let cards = g["cards"].as_array().unwrap();
    assert_eq!(
        cards.len(),
        s.c.file.pieces.len(),
        "one card for every spell piece"
    );
    assert_eq!(cards[0]["number"], "No. 1");
    assert_eq!(cards[0]["name"], s.c.file.pieces[0].name.as_str());
    assert!(
        cards
            .iter()
            .all(|c| c["icon"].as_str().unwrap().starts_with('M') && !c["text"].as_str().unwrap().is_empty()),
        "each has an icon and an explanation"
    );
    for tab in ["spells", "terms", "how"] {
        s.act("glossary.tab", tab).unwrap();
        no_missing(&s.ui());
    }
    assert!(s.act("glossary.tab", "nowhere").is_err());
    s.act("glossary.tab", "how").unwrap();
    let how = s.ui()["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "glossary")
        .cloned()
        .unwrap();
    let text = how.to_string();
    assert!(
        text.contains("Invade") && text.contains("Summon"),
        "how to send an invasion and summon a helper"
    );
    s.act("glossary.close", "").unwrap();
    s.act("match.start", "practice").unwrap();
    run(&mut s, 2);
    let clock = s.m.as_ref().unwrap().local_prep().unwrap().clock;
    s.act("glossary.toggle", "").unwrap();
    run(&mut s, 60);
    assert_eq!(
        s.m.as_ref().unwrap().local_prep().unwrap().clock,
        clock,
        "the day waits"
    );
    let ui = s.ui();
    no_missing(&ui);
    let g = ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "glossary")
        .cloned()
        .unwrap();
    assert!(
        g["cards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["chip"] == "known"),
        "known pieces are marked"
    );
    s.act("glossary.toggle", "").unwrap();
    run(&mut s, 60);
    assert!(
        s.m.as_ref().unwrap().local_prep().unwrap().clock < clock,
        "and goes on once it closes"
    );
}

#[test]
fn an_online_day_does_not_wait_for_the_glossary() {
    let (mut host, mut js) = online_room(1);
    js[0].act("online.ready", "").unwrap();
    wire(&mut host, &mut js);
    host.act("online.start", "").unwrap();
    run(&mut host, 2);
    let clock = host.m.as_ref().unwrap().local_prep().unwrap().clock;
    host.act("glossary.toggle", "").unwrap();
    run(&mut host, 30);
    assert!(host.m.as_ref().unwrap().local_prep().unwrap().clock < clock);
}

#[test]
fn each_battle_view_turns_moves_and_zooms_on_its_own() {
    let mut s = session();
    s.resize(1280, 720, 100);
    s.settings.prep_s = 60;
    s.settings.battle_s = 30;
    s.act("match.start", "versus").unwrap();
    s.act("day.ready", "").unwrap();
    run(&mut s, 3);
    let start = s.cams;
    // the top-left view is seat 0, the bottom-right seat 3 (CSS y runs down)
    s.act("cam.down", "100,100,1280,720,1,0").unwrap();
    s.act("cam.move", "200,140,1280,720,1,0").unwrap();
    s.act("cam.up", "200,140,1280,720,0,0").unwrap();
    assert_ne!(s.cams[0].yaw, start[0].yaw, "a drag turns the view");
    assert_ne!(s.cams[0].pitch, start[0].pitch);
    assert_eq!(s.cams[0].at, start[0].at, "a plain drag does not move it");
    assert_eq!(&s.cams[1..], &start[1..], "the other views stay as they were");
    s.act("cam.down", "1000,600,1280,720,2,0").unwrap();
    s.act("cam.move", "1050,600,1280,720,2,0").unwrap();
    s.act("cam.up", "1050,600,1280,720,0,0").unwrap();
    assert_ne!(s.cams[3].at, start[3].at, "a right drag moves the view");
    assert_eq!(s.cams[3].yaw, start[3].yaw);
    s.act("cam.wheel", "1000,100,1280,720,-100").unwrap();
    assert!(s.cams[1].dist < start[1].dist, "the wheel up zooms in");
    s.act("cam.move", "0,0,1280,720,0,0").unwrap();
    assert_ne!(s.cams[0], start[0], "a move with no drag changes nothing");
    let ui = s.ui();
    no_missing(&ui);
    assert!(ui.to_string().contains("cam.reset"));
    s.act("cam.reset", "").unwrap();
    assert_eq!(s.cams, start);
}

#[test]
fn the_starting_orbits_are_the_old_fixed_cameras() {
    for close in [false, true] {
        let a = scene::arena_camera(close);
        let b = scene::Orbit::start(close).camera();
        let near = |p: i32, q: i32| (p - q).abs() < crate::fx::ONE / 2;
        assert!(
            near(a.eye.x, b.eye.x) && near(a.eye.y, b.eye.y) && near(a.eye.z, b.eye.z),
            "{close}: {:?} vs {:?}",
            a.eye,
            b.eye
        );
        assert_eq!(a.at, b.at);
    }
}

#[test]
fn a_view_stays_above_the_field_and_within_its_limits() {
    let mut o = scene::Orbit::start(false);
    o.orbit(0, -100_000, 500);
    assert_eq!(o.pitch, scene::PITCH_MIN);
    o.orbit(0, 100_000, 500);
    assert_eq!(o.pitch, scene::PITCH_MAX);
    for _ in 0..200 {
        o.zoom(-400);
    }
    assert_eq!(o.dist, scene::DIST_MIN);
    for _ in 0..200 {
        o.zoom(400);
    }
    assert_eq!(o.dist, scene::DIST_MAX);
    o.pan(-1_000_000, 1_000_000, 100);
    assert!(o.at.x.abs() <= scene::PAN_MAX && o.at.z.abs() <= scene::PAN_MAX && o.at.y == 0);
    // dragging right moves the point the view circles left, so the ground follows the pointer
    let mut o = scene::Orbit::start(false);
    let x0 = o.at.x;
    o.pan(50, 0, 500);
    assert!(o.at.x < x0);
}

#[test]
fn the_map_panel_does_not_change_while_something_rises() {
    // a panel that changes every tick is redrawn under the pointer and its
    // buttons cannot be clicked (T9)
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Map);
    s.act("layout.open", "").unwrap();
    s.act("layout.feature", "glass_wall").unwrap();
    s.act("pointer.move", "500,200,1280,720").unwrap();
    s.act("pointer.down", "500,200,1280,720").unwrap();
    let panel = |s: &Session, id: &str| {
        s.ui()["panels"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .cloned()
    };
    let glass = s.c.file.features[s.c.feature("glass_wall").unwrap()].name.clone();
    assert!(
        panel(&s, "route")
            .unwrap()
            .to_string()
            .contains(&format!("1 × {glass}")),
        "the glass wall is rising, beside the map"
    );
    let before = panel(&s, "layout").unwrap();
    // glass rises in 500 ms (30 ticks); look while it is still rising
    run(&mut s, 20);
    assert_eq!(panel(&s, "layout").unwrap(), before, "the map's buttons stay put");
    assert!(panel(&s, "route").unwrap().to_string().contains(&glass));
}

#[test]
fn a_book_read_shows_its_card_for_three_seconds_or_until_the_next_book() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Library);
    let read = |s: &mut Session| {
        let before = s.card.clone().map(|c| c.0);
        // a book already known shows no card: sort again until a new one
        let mut tries = 0;
        while s.card.clone().map(|c| c.0) == before && tries < 20 {
            s.act("library.read", "").unwrap();
            sort(s);
            run(s, 1);
            tries += 1;
        }
        s.card.clone().expect("a card for the book")
    };
    let (card, _) = read(&mut s);
    assert_eq!(s.ui()["card"], card, "the page is sent the card");
    assert!(!card["name"].as_str().unwrap().is_empty() && card["icon"].as_str().unwrap().starts_with('M'));
    no_missing(&s.ui());
    // card_s 3: still there just before three seconds, gone after
    run(&mut s, TPS * 3 - 2);
    assert!(s.card.is_some());
    run(&mut s, 3);
    assert!(s.card.is_none(), "it goes after three seconds");
    let (a, _) = read(&mut s);
    let (b, left) = read(&mut s);
    assert_ne!(a["name"], b["name"], "the next book replaces the card");
    assert!(left > TPS * 3 - 5, "and starts its own three seconds");
}

#[test]
fn a_battle_plays_behind_the_title_while_the_camera_circles_it() {
    let mut s = session();
    run(&mut s, 2);
    let demo = s.demo.as_ref().expect("a battle behind the menu");
    assert_eq!(demo.phase, Phase::Battle);
    assert!(s.frame_out().commands.chunks(6).any(|c| c[0] == 0), "it is drawn");
    let (yaw, tick) = (
        s.cams[0].yaw,
        s.demo.as_ref().unwrap().battles[0].as_ref().unwrap().tick,
    );
    run(&mut s, 60);
    assert_ne!(s.cams[0].yaw, yaw, "the camera moves");
    assert!(
        s.demo.as_ref().unwrap().battles[0].as_ref().unwrap().tick > tick,
        "the battle goes on"
    );
    assert!(s.sounds().is_empty(), "the title's battle makes no sound");
    s.act("match.start", "practice").unwrap();
    assert!(s.demo.is_none(), "a match replaces it");
}

#[test]
fn whats_new_is_marked_new_until_its_notes_are_opened() {
    let mut s = session();
    let title = s.ui();
    no_missing(&title);
    let menu = title["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "title")
        .cloned()
        .unwrap();
    let news = menu["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["value"] == "news")
        .cloned()
        .unwrap();
    assert_eq!(news["badge"], "New", "a first visit has not seen the notes");
    let latest = s.news.entries[0].version.clone();
    assert!(
        title.to_string().contains(&latest),
        "the version shows on the title"
    );
    s.act("screen.show", "news").unwrap();
    let ui = s.ui();
    no_missing(&ui);
    let shown = ui["panels"][0]["rows"].as_array().unwrap().len();
    assert_eq!(
        shown,
        s.news.entries.iter().filter(|e| !e.minor).count(),
        "minor patches are hidden"
    );
    s.act("news.minor", "").unwrap();
    assert_eq!(
        s.ui()["panels"][0]["rows"].as_array().unwrap().len(),
        s.news.entries.len()
    );
    let saved = s.take_save();
    assert!(saved.contains(&format!("\"news_seen\":\"{latest}\"")), "{saved}");
    s.act("screen.show", "title").unwrap();
    let menu = s.ui()["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "title")
        .cloned()
        .unwrap();
    assert!(!menu.to_string().contains("\"badge\""), "seen, so no badge");
    // a later visit with an older note seen shows the badge again
    let mut s2 = Session::new(
        include_str!("../../../../data/strings.json"),
        include_str!("../../../../data/content.json"),
        include_str!("../../../../data/palette.json"),
        include_str!("../../../../data/models.json"),
        include_str!("../../../../data/sounds.json"),
        include_str!("../../../../GLOSSARY.md"),
        include_str!("../../../../data/news.json"),
        &saved.replace(&latest, "0.0.1"),
        1,
    )
    .unwrap();
    assert!(s2.ui().to_string().contains("\"badge\""));
    let _ = s2.take_save();
}

#[test]
fn mana_costs_are_sent_for_the_mana_symbol() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Lab);
    run(&mut s, 1);
    s.act("lab.add", "fire_bolt").unwrap();
    let c = s.c.clone();
    let bolt = spell::stats(&c, &[c.piece("fire_bolt").unwrap() as u16]).mana;
    let ui = s.ui();
    let lab = ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "station")
        .cloned()
        .unwrap();
    let cost = &blocks(&lab, "mana")[0]["mana"];
    assert_eq!(cost["n"], bolt.to_string(), "the draft's cost");
    assert_eq!(cost["label"], format!("{bolt} mana"));
    s.act("lab.add", "ice_shard").unwrap();
    s.act("lab.save", "").unwrap();
    seal(&mut s);
    let ui = s.ui();
    let lab = ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "station")
        .cloned()
        .unwrap();
    let saved: Vec<Value> = buttons_in(&lab)
        .into_iter()
        .filter(|a| a["name"] == "spell.choose")
        .collect();
    assert!(!saved.is_empty());
    assert!(
        saved.iter().all(|r| r["mana"]["n"].is_string()),
        "each saved spell's cost"
    );
    s.act("glossary.toggle", "").unwrap();
    let g = s.ui()["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "glossary")
        .cloned()
        .unwrap();
    let card = g["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Fire Bolt")
        .cloned()
        .unwrap();
    assert_eq!(card["mana"]["n"], bolt.to_string());
}

#[test]
fn the_host_s_lobby_offers_the_room_code_and_a_link_to_copy() {
    let (host, _) = online_room(1);
    let ui = host.ui();
    no_missing(&ui);
    let lobby = ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "lobby")
        .cloned()
        .unwrap();
    let code = host.online.as_ref().unwrap().room.clone();
    let has = |name: &str| {
        lobby["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["name"] == name && a["value"] == code.as_str())
    };
    assert!(has("page.copy"), "the code");
    assert!(has("page.copy_link"), "the link, which carries the code");
}

#[test]
fn each_trial_has_its_own_difficulty_and_keeps_a_best_at_each() {
    let mut s = session();
    s.act("screen.show", "trials").unwrap();
    no_missing(&s.ui());
    assert!(s.ui().to_string().contains("Difficulty: Normal"));
    s.act("trial.difficulty", "2").unwrap();
    assert!(s.ui().to_string().contains("Difficulty: Hard"));
    s.act("trial.start", "2").unwrap();
    assert_eq!(s.m.as_ref().unwrap().settings.difficulty, 2);
    assert_eq!(s.best_key(2, 2), "siege_drill@hard");
    assert_eq!(s.best_key(2, 1), "siege_drill", "normal keeps the old key");
    s.act("screen.show", "setup").unwrap();
    s.act("setup.difficulty", "up").unwrap();
    assert_eq!(s.settings.difficulty, 2);
    no_missing(&s.ui());
}

#[test]
fn office_hours_shows_a_student_and_sets_their_seat_and_rotation() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Office);
    run(&mut s, 1);
    let ids: Vec<u16> =
        s.m.as_ref()
            .unwrap()
            .local_prep()
            .unwrap()
            .students
            .iter()
            .map(|x| x.id)
            .collect();
    assert!(!ids.is_empty(), "a new archmage has students");
    let ui = s.ui();
    no_missing(&ui);
    let ps = ui["panels"].as_array().unwrap();
    let office = ps.iter().find(|p| p["id"] == "station").unwrap();
    assert_eq!(office["kind"], "office", "one panel, after the mockup");
    assert_eq!(
        blocks(office, "meters").len(),
        2,
        "the student's stats and their leanings"
    );
    let seat_buttons = buttons_in(office)
        .iter()
        .filter(|a| a["name"] == "office.seat")
        .count() as i32;
    // the tower's 12 seats on the board, no sub-tower yet, then first free and the ground
    assert_eq!(seat_buttons, 12 + 2);
    let id = ids[0];
    s.act("office.pick", &id.to_string()).unwrap();
    s.act("office.seat", &format!("{id},{}", battle::GROUND_SEAT))
        .unwrap();
    s.act("office.mode", &format!("{id},{}", battle::CAST_ROTATION))
        .unwrap();
    s.act("office.rot_add", &format!("{id},0")).unwrap();
    assert!(s.act("office.seat", &format!("{id},500")).is_err());
    let plan = s.m.as_ref().unwrap().local_prep().unwrap().plan();
    let st = &plan.students[0];
    assert_eq!(
        (st.seat, st.mode, st.rotation.clone()),
        (battle::GROUND_SEAT, battle::CAST_ROTATION, vec![0])
    );
    no_missing(&s.ui());
    s.act("glossary.toggle", "").unwrap();
    s.act("glossary.tab", "students").unwrap();
    let ui = s.ui();
    no_missing(&ui);
    assert!(
        ui.to_string().contains("Stands: Ground"),
        "the students tab shows where they stand"
    );
}

#[test]
fn on_the_map_a_students_disc_is_dragged_to_another_seat() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Map);
    s.act("layout.open", "").unwrap();
    s.act("layout.seats", "on").unwrap();
    let ids: Vec<u16> =
        s.m.as_ref()
            .unwrap()
            .local_prep()
            .unwrap()
            .students
            .iter()
            .map(|x| x.id)
            .collect();
    // nobody seated yet: the first student is given seat 0, at the ring's top.
    // The tower's middle is cell (27.5, 4.5) and the ring 3.6 cells round
    // it; a cell is 18 px from (352, 72): seat 0 at (847, 88), seat 3 a
    // quarter turn on at (911, 153)
    let err = s.act("pointer.up", "911,153,1280,720").unwrap_err();
    assert_eq!(
        err, "Pick a student first.",
        "an empty seat needs a student picked"
    );
    s.act("pointer.down", "847,88,1280,720").unwrap();
    s.act("pointer.up", "911,153,1280,720").unwrap();
    let seats: Vec<i32> =
        s.m.as_ref()
            .unwrap()
            .local_prep()
            .unwrap()
            .students
            .iter()
            .map(|x| x.seat)
            .collect();
    assert_eq!(seats[0], 3, "the first student now chose seat 3");
    assert_eq!(seats[1], battle::AUTO_SEAT, "the other was not moved");
    // the map shows the seats only while seating
    s.act("layout.seats", "off").unwrap();
    s.act("pointer.down", "847,88,1280,720").unwrap();
    s.act("pointer.up", "847,88,1280,720").unwrap();
    assert_eq!(s.m.as_ref().unwrap().local_prep().unwrap().students[0].seat, 3);
    let _ = ids;
}

#[test]
fn the_builder_shows_one_kind_of_tile_and_a_dropped_tile_must_fit_its_slot() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Lab);
    run(&mut s, 1);
    let tray = |s: &Session| blocks(&find_panel(s, "station").unwrap(), "tray")[0]["tray"].clone();
    let kinds = |t: &Value| -> Vec<String> {
        t["tiles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["kind"].as_str().unwrap().to_string())
            .collect()
    };
    let t = tray(&s);
    assert!(
        !kinds(&t).is_empty() && kinds(&t).iter().all(|k| k == "pay"),
        "payloads first"
    );
    s.act("lab.tab", "trig").unwrap();
    assert!(kinds(&tray(&s)).iter().all(|k| k == "trig"));
    assert!(s.act("lab.tab", "spells").is_err());
    s.act("lab.hold", "fire_bolt").unwrap();
    assert!(
        s.act("lab.drop", "trig").is_err(),
        "a payload does not go in a trigger's slot"
    );
    s.act("lab.hold", "fire_bolt").unwrap();
    s.act("lab.drop", "pay").unwrap();
    let c = s.c.clone();
    assert_eq!(
        s.m.as_ref().unwrap().local_prep().unwrap().draft,
        vec![c.piece("fire_bolt").unwrap() as u16]
    );
    assert!(s.act("lab.drop", "pay").is_err(), "nothing held after a drop");
    no_missing(&s.ui());
}

#[test]
fn the_seal_lists_its_rings_and_the_sorting_game_has_a_shelf_button_each() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Lab);
    run(&mut s, 1);
    s.act("lab.add", "fire_bolt").unwrap();
    s.act("lab.add", "ice_shard").unwrap();
    s.act("lab.save", "").unwrap();
    let seal = find_panel(&s, "busy").unwrap();
    assert_eq!(seal["kind"], "seal");
    let rings = blocks(&seal, "rows")[0]["rows"].as_array().unwrap().clone();
    assert_eq!(rings.len(), 2, "a ring for each piece");
    assert_eq!(
        (rings[0]["chip"].as_str(), rings[1]["chip"].as_str()),
        (Some("turning"), Some("next"))
    );
    no_missing(&s.ui());
    s.act("prep.cancel", "").unwrap();
    s.teleport(Station::Library);
    s.act("library.read", "").unwrap();
    let sort = find_panel(&s, "busy").unwrap();
    assert_eq!(sort["kind"], "sort");
    let shelves: Vec<Value> = buttons_in(&sort)
        .into_iter()
        .filter(|a| a["name"] == "sort.shelf")
        .collect();
    assert_eq!(shelves.len(), s.c.file.schools.len());
    assert!(
        find_panel(&s, "golem").is_some(),
        "the golem is offered beside it"
    );
    no_missing(&s.ui());
}

#[test]
fn office_hours_draws_the_seats_and_a_held_seat_can_be_picked_up() {
    let mut s = session();
    s.act("match.start", "practice").unwrap();
    s.teleport(Station::Office);
    run(&mut s, 1);
    let office = find_panel(&s, "station").unwrap();
    let board = &blocks(&office, "board")[0];
    let seats: Vec<&Value> = board["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["kind"] == "seat")
        .collect();
    assert_eq!(seats.len(), 12, "the tower's twelve seats");
    // two students, given seats 0 and 1: those discs can be picked up
    let held: Vec<&str> = seats
        .iter()
        .filter_map(|i| i["text"].as_str())
        .filter(|t| !t.is_empty())
        .collect();
    assert_eq!(held.len(), 2);
    assert_eq!(
        seats.iter().filter(|i| i["action"]["grab"].is_object()).count(),
        2
    );
    assert!(
        seats.iter().all(|i| i["action"]["drop"] == true),
        "each seat takes a drop"
    );
}

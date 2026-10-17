//! Probe, not a check: the words each station's panel shows on the first
//! day of a versus match, not counting hover text, action names and values
//! or style keys (TRIAGE T49). `WORDS=lab` also prints that panel.
//!
//!     cargo test -p tool_core --test words_probe -- --ignored --nocapture

use serde_json::Value;
use tool_core::prep::Station;
use tool_core::session::Session;

fn shown_words(v: &Value) -> usize {
    const HIDDEN: [&str; 17] = [
        "hint", "help", "name", "value", "id", "kind", "school", "tone", "t", "col", "icon", "view", "style",
        "left", "top", "w", "h",
    ];
    match v {
        Value::String(s) => s
            .split_whitespace()
            .filter(|w| w.chars().any(|c| c.is_alphabetic()))
            .count(),
        Value::Array(a) => a.iter().map(shown_words).sum(),
        Value::Object(o) => o
            .iter()
            .filter(|(k, _)| !HIDDEN.contains(&k.as_str()) && !k.ends_with("box"))
            .map(|(_, v)| shown_words(v))
            .sum(),
        _ => 0,
    }
}

fn run(s: &mut Session, ticks: i32) {
    for k in 0..ticks {
        s.frame(k as i64 * 16_667 + 16_667, 0);
    }
}

#[test]
#[ignore]
fn words_per_station() {
    let mut s = Session::new(
        include_str!("../../../data/strings.json"),
        include_str!("../../../data/content.json"),
        include_str!("../../../data/palette.json"),
        include_str!("../../../data/models.json"),
        include_str!("../../../data/sounds.json"),
        include_str!("../../../GLOSSARY.md"),
        include_str!("../../../data/news.json"),
        "",
        42,
    )
    .unwrap();
    s.rehearse = false;
    s.act("match.start", "versus").unwrap();
    let show = std::env::var("WORDS").ok();
    let mut total = 0;
    for st in [
        Station::Library,
        Station::Lab,
        Station::Book,
        Station::Map,
        Station::Podium,
        Station::Recruit,
        Station::Office,
    ] {
        s.teleport(st);
        run(&mut s, 1);
        let ui = s.ui();
        let panel = ui["panels"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == "station")
            .cloned()
            .unwrap_or(Value::Null);
        let n = shown_words(&panel);
        total += n;
        println!("{:<10} {n}", st.key());
        if show.as_deref() == Some(st.key()) {
            println!("{}", serde_json::to_string_pretty(&panel).unwrap());
        }
    }
    s.teleport(Station::Map);
    s.act("layout.open", "").unwrap();
    run(&mut s, 1);
    let ui = s.ui();
    let n: usize = ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["id"] == "layout" || p["id"] == "route")
        .map(shown_words)
        .sum();
    total += n;
    println!("{:<10} {n}", "map open");
    if show.as_deref() == Some("layout") {
        println!("{}", serde_json::to_string_pretty(&ui["panels"]).unwrap());
    }
    println!("{:<10} {total}", "all");
}

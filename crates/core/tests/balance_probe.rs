//! Probe, not a check: many four-bot matches, one of each plan, in
//! rotating seats. Prints each plan's share of wins (a tie counts for
//! everyone in it) and the most damage any tower took in a match, which
//! must stay far below the starting score (the person: reaching 0 should
//! be "rather difficult if not borderline impossible").
//!
//!     cargo test --release -p tool_core --test balance_probe -- --ignored --nocapture
//!     MATCHES=200 cargo test --release ...

use tool_core::bot::{Style, STYLES};
use tool_core::content::Content;
use tool_core::game::{Match, Mode, Phase, SeatKind, Settings};

#[test]
#[ignore]
fn win_shares_and_worst_damage_over_many_matches() {
    let c = Content::parse(include_str!("../../../data/content.json")).unwrap();
    let n: u64 = std::env::var("MATCHES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(40);
    let mut wins = [0u32; 4];
    let mut worst = 0;
    let mut worst_style = Style::Balanced;
    let mut total_taken = [0i64; 4];
    let mut from_invasions = [0i64; 4];
    for k in 0..n {
        let rot = (k % 4) as usize;
        let styles: Vec<Style> = (0..4).map(|i| STYLES[(i + rot) % 4]).collect();
        let kinds = [
            SeatKind::Bot(styles[0]),
            SeatKind::Bot(styles[1]),
            SeatKind::Bot(styles[2]),
            SeatKind::Bot(styles[3]),
        ];
        let names = ["a", "b", "c", "d"].map(String::from);
        let mut m = Match::new(
            &c,
            1000 + k,
            Mode::Versus,
            Settings::from_content(&c),
            kinds,
            names,
            9,
        );
        while m.phase != Phase::Results {
            m.step(&c, 0);
            if m.phase == Phase::Summary {
                m.next_day(&c);
            }
        }
        for r in &m.results {
            let style_idx = STYLES.iter().position(|s| *s == styles[r.seat as usize]).unwrap();
            from_invasions[style_idx] += r.from_invasions as i64;
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            let style_idx = STYLES.iter().position(|s| *s == styles[i]).unwrap();
            let taken: i32 = m.seats[i].taken.iter().sum();
            total_taken[style_idx] += taken as i64;
            if taken > worst {
                worst = taken;
                worst_style = styles[i];
            }
            if m.won(i) {
                wins[style_idx] += 1;
            }
        }
    }
    for (i, s) in STYLES.iter().enumerate() {
        let (w, t, f) = (wins[i], total_taken[i] / n as i64, from_invasions[i] / n as i64);
        println!("{s:?}: won {w} of {n}, mean damage taken {t}, of it from invasions {f}");
    }
    println!(
        "worst damage taken in one match: {worst} ({worst_style:?}), score_start {}",
        c.balance().score_start
    );
}

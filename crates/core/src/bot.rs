//! Computer-played rivals, one for each plan in entry 1: the spellcrafter,
//! the mass teacher, the attack mage and the balanced archmage. A bot does
//! not walk or draw. It spends a day's time from a budget that charges
//! each action about what it costs a person (walking included), and its
//! mini-game scores come from a seeded skill. What it saves still passes
//! the same chain rules a player's spell does.

use serde::{Deserialize, Serialize};

use crate::content::{Content, Effect, PieceKind};
use crate::geom::TPS;
use crate::prep::{Prep, Station};
use crate::rng::Rng;
use crate::spell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Style {
    Spellcrafter,
    MassTeacher,
    AttackMage,
    Balanced,
}

pub const STYLES: [Style; 4] = [
    Style::Spellcrafter,
    Style::MassTeacher,
    Style::AttackMage,
    Style::Balanced,
];

impl Style {
    pub fn key(self) -> &'static str {
        match self {
            Style::Spellcrafter => "bot.spellcrafter",
            Style::MassTeacher => "bot.mass_teacher",
            Style::AttackMage => "bot.attack_mage",
            Style::Balanced => "bot.balanced",
        }
    }
}

/// Seconds each action costs a bot, walking to the station included.
const READ_S: i32 = 5;
const INTERVIEW_S: i32 = 9;
const GROUP_S: i32 = 11;
const TEACH_S_PER_GLYPH: i32 = 3;
const CRAFT_S: i32 = 6;
const PLACE_S: i32 = 4;

fn put(p: &mut Prep, s: Station) {
    let at = crate::prep::station_pos(s);
    let toward = crate::geom::V3::ZERO
        .sub(at)
        .flat()
        .with_len(crate::prep::PROP + crate::prep::BODY + crate::fx::ONE / 10);
    p.pos = at.add(toward);
}

fn payloads(c: &Content, p: &Prep, pred: impl Fn(&Effect) -> bool) -> Vec<u16> {
    (0..c.file.pieces.len())
        .filter(|&i| p.known[i])
        .filter(|&i| matches!(&c.file.pieces[i].kind, PieceKind::Payload { effect, .. } if pred(effect)))
        .map(|i| i as u16)
        .collect()
}

fn known(c: &Content, p: &Prep, id: &str) -> Option<u16> {
    c.piece(id).filter(|&i| p.known[i]).map(|i| i as u16)
}

/// The spells a bot would like, strongest first, from what it knows.
fn wanted(c: &Content, p: &Prep, style: Style) -> Vec<Vec<u16>> {
    let mut out: Vec<Vec<u16>> = Vec::new();
    let damage = payloads(c, p, |e| matches!(e, Effect::Bolt { .. } | Effect::Burst { .. }));
    let forms: Vec<u16> = ["ring", "fan", "twin"]
        .iter()
        .filter_map(|n| known(c, p, n))
        .collect();
    let mods: Vec<u16> = ["heavy", "pierce"]
        .iter()
        .filter_map(|n| known(c, p, n))
        .collect();
    let on_hit = known(c, p, "on_hit");
    let grounds = payloads(c, p, |e| matches!(e, Effect::Ground { .. } | Effect::Hex { .. }));
    for &d in damage.iter().rev() {
        if let Some(&f) = forms.first() {
            if let (Some(t), Some(&g)) = (on_hit, grounds.first()) {
                out.push(vec![f, d, t, g]);
            }
            out.push(vec![f, d]);
        }
        if let Some(&m) = mods.first() {
            out.push(vec![m, d]);
        }
        out.push(vec![d]);
    }
    for &g in &grounds {
        out.push(vec![g]);
    }
    if let Some(w) = known(c, p, "ward") {
        out.push(vec![w]);
    }
    let summons = payloads(c, p, |e| matches!(e, Effect::Summon { .. }));
    if let Some(inv) = known(c, p, "invade") {
        for &s in summons.iter().rev() {
            if let (Some(&f), Some(death), Some(&s2)) =
                (forms.first(), known(c, p, "on_death"), summons.first())
            {
                out.push(vec![inv, f, s, death, s2]);
            }
            out.push(vec![inv, s]);
        }
    }
    for &s in &summons {
        out.push(vec![s]);
    }
    out.retain(|ch| spell::check(c, ch).is_ok());
    if style == Style::AttackMage {
        out.sort_by_key(|ch| !spell::is_invasion(c, ch));
    }
    out
}

/// One bot's day: spends the preparation's time and leaves the Prep ready
/// for its battle. `leader` is the rival an attack mage aims at.
/// `invade` is false on the first day: computer rivals give a new player
/// one quiet day (SECOND-ORDER R3.30).
#[allow(clippy::too_many_arguments)]
pub fn day(c: &Content, p: &mut Prep, style: Style, skill: i32, leader: u8, invade: bool, rng: &mut Rng) {
    let mut left = p.day_ticks / TPS;
    let spend = |s: i32, left: &mut i32| -> bool {
        if *left < s {
            return false;
        }
        *left -= s;
        true
    };
    let roll = |rng: &mut Rng| (skill + rng.between(-12, 12)).clamp(5, 100);
    let reads = match style {
        Style::Spellcrafter => left * 55 / 100 / READ_S,
        Style::MassTeacher => left * 15 / 100 / READ_S,
        Style::AttackMage => left * 45 / 100 / READ_S,
        Style::Balanced => left * 30 / 100 / READ_S,
    };
    put(p, Station::Library);
    for _ in 0..reads {
        if !spend(READ_S, &mut left) || p.read_now(c).is_err() {
            break;
        }
        while p.busy() {
            p.step(c, 0);
        }
    }
    // spells
    put(p, Station::Lab);
    for ch in wanted(c, p, style).into_iter().take(4) {
        if !spend(CRAFT_S, &mut left) {
            break;
        }
        if !p.spells.contains(&ch) && p.spells.len() < crate::prep::MAX_SPELLS {
            p.draft = ch;
            let _ = p.save_now(c);
            p.draft.clear();
        }
    }
    // recruiting
    let interviews = match style {
        Style::AttackMage => 0,
        Style::Spellcrafter => 2,
        Style::Balanced => 1,
        Style::MassTeacher => 9,
    };
    put(p, Station::Recruit);
    for _ in 0..interviews {
        if p.applicants.is_empty() {
            break;
        }
        let group = matches!(style, Style::MassTeacher | Style::Balanced) || p.students.len() < 10;
        if !spend(if group { GROUP_S } else { INTERVIEW_S }, &mut left) {
            break;
        }
        let score = roll(rng);
        p.bot_hire(c, score, if group { None } else { Some(0) });
    }
    // walls: lengthen the route with whatever walls are known
    put(p, Station::Map);
    let walls: Vec<&str> = ["ivory_wall", "glass_wall"]
        .into_iter()
        .filter(|w| known(c, p, w).is_some())
        .collect();
    for w in walls {
        for _ in 0..3 {
            if !spend(PLACE_S, &mut left) {
                break;
            }
            let (x, z, d) = (rng.between(2, 28), rng.between(6, 26), rng.below(2));
            // a wall of three cells, laid a cell at a time (T23)
            for k in 0..3 {
                let (cx, cz) = if d == 1 { (x, z + k) } else { (x + k, z) };
                let _ = p.act(c, "layout.place", &format!("{w},{cx},{cz},0"));
            }
        }
    }
    p.finish_raises(c);
    // teaching: every spell to everyone, best first, while time lasts
    put(p, Station::Podium);
    let order: Vec<usize> = (0..p.spells.len()).rev().collect();
    for i in order {
        let glyphs = p.spells[i].len() as i32;
        if !spend(TEACH_S_PER_GLYPH * glyphs + 2, &mut left) {
            break;
        }
        let audience = p.students.len() as i32;
        // a bigger class means a narrower band: skill falls a little with size
        let score = (roll(rng) - audience / 3).clamp(5, 100);
        p.bot_teach(c, i as u16, score);
    }
    // invasions last, so the time left can wait for mana to come back:
    // the attack mage sends many, the leader first, then the others in
    // turn; the balanced archmage sends one
    if invade && (style == Style::AttackMage || style == Style::Balanced) {
        put(p, Station::Book);
        let inv: Vec<usize> = (0..p.spells.len())
            .filter(|&i| spell::is_invasion(c, &p.spells[i]))
            .collect();
        let targets: Vec<u8> = std::iter::once(leader)
            .chain((0..4u8).filter(|&t| t != leader && t != p.seat))
            .collect();
        // each cast reaches every rival now (T29), so fewer casts
        let casts = if style == Style::AttackMage { 4 } else { 1 };
        let regen = c.balance().archmage_regen.max(1);
        let mut k = 0;
        'cast: for _ in 0..casts {
            for &i in inv.iter().rev() {
                let t = targets[k % targets.len()];
                if p.act(c, "book.cast", &format!("{i},{t}")).is_err() {
                    // wait for the mana, if the day has the time
                    let cost = spell::stats(c, &p.spells[i]).mana * c.balance().invasion_mana_pct / 100;
                    let wait = (cost * 100 - p.mana).max(0) / (regen * 100) + 1;
                    if !spend(wait, &mut left) {
                        break 'cast;
                    }
                    p.mana += wait * regen * 100;
                    if p.act(c, "book.cast", &format!("{i},{t}")).is_err() {
                        break 'cast;
                    }
                }
                k += 1;
            }
        }
    }
    p.ready = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::shipped;

    fn bot_prep(c: &Content, seat: u8) -> Prep {
        let mut p = Prep::new(c, seat, 5, 5);
        p.start_day(c, TPS * 240);
        p
    }

    #[test]
    fn each_bot_uses_only_legal_spells_and_ends_ready() {
        let c = shipped();
        for (k, style) in STYLES.into_iter().enumerate() {
            let mut p = bot_prep(&c, k as u8);
            let mut rng = Rng::new(k as u64);
            for day_no in 0..4 {
                p.start_day(&c, TPS * 240);
                day(&c, &mut p, style, 70, ((k + 1) % 4) as u8, true, &mut rng);
                assert!(p.ready, "{style:?} day {day_no}");
            }
            for s in &p.spells {
                assert_eq!(
                    spell::check(&c, s),
                    Ok(()),
                    "{style:?} saved an illegal spell {s:?}"
                );
            }
            assert!(p.field(&c).reachable(), "{style:?} closed its own route");
        }
    }

    #[test]
    fn the_four_plans_come_out_different() {
        let c = shipped();
        let mut out = Vec::new();
        for (k, style) in STYLES.into_iter().enumerate() {
            let mut p = bot_prep(&c, k as u8);
            let mut rng = Rng::new(9);
            for _ in 0..3 {
                p.start_day(&c, TPS * 240);
                day(&c, &mut p, style, 70, 0, true, &mut rng);
            }
            let invasions = p.spells.iter().filter(|s| spell::is_invasion(&c, s)).count();
            out.push((style, p.students.len(), p.record.books, invasions));
        }
        let (spell_k, mass, attack) = (out[0], out[1], out[2]);
        assert!(
            mass.1 > spell_k.1 + 4,
            "the mass teacher recruits far more: {out:?}"
        );
        assert!(spell_k.2 > mass.2, "the spellcrafter reads more: {out:?}");
        assert_eq!(
            attack.1,
            c.balance().start_students as usize,
            "the attack mage recruits nobody: {out:?}"
        );
    }
}

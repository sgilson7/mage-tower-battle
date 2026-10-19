//! Spells: a chain of spell pieces read left to right (after Galactic
//! Survivors' rig model). Modifiers and formations change the next
//! payload; a trigger hands the rest of the chain to what the payload made,
//! to start when it hits, dies or ends; Invade sends what follows to a
//! rival. This module checks a chain and works out its power, cooldown and
//! mana cost; `battle.rs` casts it.

use serde::{Deserialize, Serialize};

use crate::content::{Content, Effect, PieceKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spell {
    pub pieces: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainError {
    Empty,
    TooLong,
    NoPayloadFirst,
    TriggerAtEnd,
    ModifierAtEnd,
    InvadeWithoutSummon,
    InvadeNotFirst,
    BranchFirst,
    SelfOnly,
    BranchTwice,
    TooDeep,
    Helper,
    PlaceOnly,
    /// A jinx is cast at a rival's tower, never put in a spell (T62).
    JinxOnly,
}

impl ChainError {
    pub fn key(self) -> &'static str {
        match self {
            ChainError::Empty => "lab.empty",
            ChainError::TooLong => "lab.too_long",
            ChainError::NoPayloadFirst => "lab.no_payload_first",
            ChainError::TriggerAtEnd => "lab.trigger_at_end",
            ChainError::ModifierAtEnd => "lab.modifier_at_end",
            ChainError::InvadeWithoutSummon => "lab.invade_without_summon",
            ChainError::InvadeNotFirst => "lab.invade_first",
            ChainError::BranchFirst => "lab.branch_first",
            ChainError::SelfOnly => "lab.self_only",
            ChainError::BranchTwice => "lab.branch_twice",
            ChainError::TooDeep => "lab.too_deep",
            ChainError::Helper => "lab.helper",
            ChainError::PlaceOnly => "lab.place_only",
            ChainError::JinxOnly => "lab.jinx_only",
        }
    }
}

fn is_payload(k: &PieceKind) -> bool {
    matches!(k, PieceKind::Payload { .. })
}

/// Whether a piece may be placed by hand from the map table: a Raise payload.
pub fn raises(c: &Content, piece: u16) -> Option<usize> {
    match &c.file.pieces.get(piece as usize)?.kind {
        PieceKind::Payload {
            effect: Effect::Raise { feature },
            ..
        } => c.feature(feature),
        _ => None,
    }
}

/// Checks a chain against the rules in the order a player meets them.
pub fn check(c: &Content, pieces: &[u16]) -> Result<(), ChainError> {
    let b = c.balance();
    if pieces.is_empty() {
        return Err(ChainError::Empty);
    }
    if pieces.len() > b.max_chain as usize {
        return Err(ChainError::TooLong);
    }
    let kinds: Vec<&PieceKind> = pieces.iter().map(|&p| &c.file.pieces[p as usize].kind).collect();
    // a branch (T17): one, after the first trigger; each row with the part
    // before it must be a spell on its own
    let branches: Vec<usize> = (0..kinds.len())
        .filter(|&i| matches!(kinds[i], PieceKind::Branch))
        .collect();
    if branches.len() > 1 {
        return Err(ChainError::BranchTwice);
    }
    if let Some(&bi) = branches.first() {
        let t = kinds.iter().position(|k| matches!(k, PieceKind::Trigger { .. }));
        let Some(t) = t.filter(|&t| t < bi) else {
            return Err(ChainError::BranchFirst);
        };
        let row_a: Vec<u16> = pieces[..bi].to_vec();
        let row_b: Vec<u16> = pieces[..=t].iter().chain(&pieces[bi + 1..]).copied().collect();
        check(c, &row_a)?;
        return check(c, &row_b);
    }
    if kinds.iter().any(|k| matches!(k, PieceKind::SelfCast { .. })) {
        return Err(ChainError::SelfOnly);
    }
    if kinds.iter().any(|k| matches!(k, PieceKind::Jinx { .. })) {
        return Err(ChainError::JinxOnly);
    }
    if kinds.iter().any(|k| matches!(k, PieceKind::Helper { .. })) {
        return Err(ChainError::Helper);
    }
    for &p in pieces {
        if let Some(f) = raises(c, p) {
            if c.file.features[f].cast_life == 0 {
                return Err(ChainError::PlaceOnly);
            }
        }
    }
    let first_trigger = kinds
        .iter()
        .position(|k| matches!(k, PieceKind::Trigger { .. }))
        .unwrap_or(kinds.len());
    if !kinds[..first_trigger].iter().any(|k| is_payload(k)) {
        return Err(ChainError::NoPayloadFirst);
    }
    let triggers = kinds
        .iter()
        .filter(|k| matches!(k, PieceKind::Trigger { .. }))
        .count();
    if triggers > b.max_depth as usize {
        return Err(ChainError::TooDeep);
    }
    for (i, k) in kinds.iter().enumerate() {
        match k {
            PieceKind::Trigger { .. } => {
                if !matches!(
                    kinds.get(i + 1),
                    Some(PieceKind::Payload { .. })
                        | Some(PieceKind::Modifier { .. })
                        | Some(PieceKind::Formation { .. })
                        | Some(PieceKind::Invade)
                ) || !kinds[i + 1..].iter().any(|k| is_payload(k))
                {
                    return Err(ChainError::TriggerAtEnd);
                }
            }
            PieceKind::Modifier { .. } | PieceKind::Formation { .. } => {
                // a modifier needs a payload after it before any trigger
                let rest = &kinds[i + 1..];
                let until = rest
                    .iter()
                    .position(|k| matches!(k, PieceKind::Trigger { .. }))
                    .unwrap_or(rest.len());
                if !rest[..until].iter().any(|k| is_payload(k)) {
                    return Err(ChainError::ModifierAtEnd);
                }
            }
            PieceKind::Invade if i > 0 => return Err(ChainError::InvadeNotFirst),
            PieceKind::Invade => {
                // the summon must come before the next trigger: an invasion
                // is cast at the entry, where a payload before a trigger
                // reaches nothing, so a summon behind one never comes
                // (SECOND-ORDER R4.21, found by the spell space)
                let rest = &pieces[i + 1..];
                let until = rest
                    .iter()
                    .position(|&p| matches!(c.file.pieces[p as usize].kind, PieceKind::Trigger { .. }))
                    .unwrap_or(rest.len());
                let summon = rest[..until].iter().any(|&p| {
                    matches!(
                        &c.file.pieces[p as usize].kind,
                        PieceKind::Payload {
                            effect: Effect::Summon { .. },
                            ..
                        }
                    )
                });
                if !summon {
                    return Err(ChainError::InvadeWithoutSummon);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Whether a spell sends its summons to a rival.
pub fn is_invasion(c: &Content, pieces: &[u16]) -> bool {
    pieces
        .iter()
        .any(|&p| matches!(c.file.pieces[p as usize].kind, PieceKind::Invade))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub power: i32,
    pub cooldown_ms: i32,
    pub mana: i32,
}

/// Power, cooldown and mana cost. Power adds each payload's power, raised
/// by the modifiers before it and multiplied by its formation; what comes
/// after a trigger counts at the trigger weight, once per trigger depth.
/// The cooldown is the first payload's, changed by the modifiers before it.
/// Mana = power² × k / (cooldown + c), rounded up: it grows faster than
/// power, and a longer cooldown makes the same power cheaper.
pub fn stats(c: &Content, pieces: &[u16]) -> Stats {
    let b = c.balance();
    let mut power: i64 = 0;
    let mut weight: i64 = 100;
    let mut pp: i64 = 0;
    let mut cd_pct: i64 = 0;
    let mut form: i64 = 100;
    let mut cooldown: Option<i64> = None;
    // a branch's row counts at the weight the first trigger set (T17)
    let mut after_first: Option<i64> = None;
    for &p in pieces {
        match &c.file.pieces[p as usize].kind {
            PieceKind::Modifier { mods, power_pct } => {
                pp += *power_pct as i64;
                cd_pct += mods.cooldown_pct as i64;
            }
            PieceKind::Formation {
                copies, share_pct, ..
            } => form = *copies as i64 * *share_pct as i64,
            PieceKind::Payload {
                power: base,
                cooldown: cd,
                ..
            } => {
                power += *base as i64 * (100 + pp) * form * weight / 1_000_000;
                if cooldown.is_none() {
                    cooldown = Some(*cd as i64 * (100 + cd_pct) / 100);
                }
                pp = 0;
                cd_pct = 0;
                form = 100;
            }
            PieceKind::Trigger { .. } => {
                weight = weight * b.trigger_weight_pct as i64 / 100;
                after_first.get_or_insert(weight);
            }
            PieceKind::Branch => {
                weight = after_first.unwrap_or(weight);
                pp = 0;
                cd_pct = 0;
                form = 100;
            }
            PieceKind::Invade
            | PieceKind::Helper { .. }
            | PieceKind::SelfCast { .. }
            | PieceKind::Jinx { .. } => {}
        }
    }
    let cd = cooldown.unwrap_or(1000).max(300);
    let num = power * power * b.mana_k as i64;
    let den = cd + b.mana_c as i64;
    let mut mana = (num + den - 1) / den.max(1);
    // a spell that summons your own army costs more (T25); an invasion keeps its price
    let summons = pieces.iter().any(|&p| {
        matches!(
            c.file.pieces[p as usize].kind,
            PieceKind::Payload {
                effect: Effect::Summon { .. },
                ..
            }
        )
    });
    if summons && !is_invasion(c, pieces) {
        mana = mana * b.summon_mana_pct as i64 / 100;
    }
    Stats {
        power: power as i32,
        cooldown_ms: cd as i32,
        mana: mana as i32,
    }
}

/// A name from the pieces: the formation and the payloads, joined.
pub fn name(c: &Content, pieces: &[u16]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut prefix = String::new();
    for &p in pieces {
        let piece = &c.file.pieces[p as usize];
        match piece.kind {
            PieceKind::Formation { .. } | PieceKind::Invade => prefix = format!("{} ", piece.name),
            PieceKind::Payload { .. } => {
                parts.push(format!("{prefix}{}", piece.name));
                prefix.clear();
            }
            _ => {}
        }
    }
    parts.join(" + ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::shipped;

    fn ids(c: &Content, names: &[&str]) -> Vec<u16> {
        names
            .iter()
            .map(|n| c.piece(n).unwrap_or_else(|| panic!("no piece {n}")) as u16)
            .collect()
    }

    #[test]
    fn fire_bolt_alone_and_heavy_cost_what_the_formula_gives() {
        let c = shipped();
        // 30² × 40 / (900 + 2000) = 12.41 → 13
        assert_eq!(
            stats(&c, &ids(&c, &["fire_bolt"])),
            Stats {
                power: 30,
                cooldown_ms: 900,
                mana: 13
            }
        );
        // Heavy: power 30 × 160% = 48, cooldown 900 × 150% = 1350; 48² × 40 / 3350 = 27.5 → 28
        assert_eq!(
            stats(&c, &ids(&c, &["heavy", "fire_bolt"])),
            Stats {
                power: 48,
                cooldown_ms: 1350,
                mana: 28
            }
        );
    }

    #[test]
    fn acid_halo_costs_what_entry_one_worked_out() {
        let c = shipped();
        // Ring 8 × 25% = 200%: acid glob 28 → 56; oil spill 15 at 50% → 7; power 63
        // 63² × 40 / (1000 + 2000) = 52.92 → 53
        let halo = ids(&c, &["ring", "acid_glob", "on_hit", "oil_spill"]);
        assert_eq!(check(&c, &halo), Ok(()));
        assert_eq!(
            stats(&c, &halo),
            Stats {
                power: 63,
                cooldown_ms: 1000,
                mana: 53
            }
        );
        assert_eq!(name(&c, &halo), "Ring Acid Glob + Oil Spill");
    }

    #[test]
    fn a_longer_cooldown_never_costs_more_and_more_power_never_less() {
        let c = shipped();
        let b = c.balance();
        let cost =
            |p: i64, cd: i64| (p * p * b.mana_k as i64 + cd + b.mana_c as i64 - 1) / (cd + b.mana_c as i64);
        // the formula used by stats(), checked over a grid of values
        for p in [10, 30, 63, 120, 400] {
            for cd in (300..20_000).step_by(700) {
                assert!(cost(p, cd + 700) <= cost(p, cd));
                assert!(cost(p + 10, cd) >= cost(p, cd));
            }
        }
        // and through real chains: Heavy raises power and cooldown; Swift lowers both
        let plain = stats(&c, &ids(&c, &["fire_bolt"]));
        let swift = stats(&c, &ids(&c, &["swift", "fire_bolt"]));
        assert!(swift.cooldown_ms < plain.cooldown_ms);
    }

    #[test]
    fn a_branch_comes_once_after_a_trigger_and_each_row_must_be_a_spell() {
        let c = shipped();
        let r = |n: &[&str]| check(&c, &ids(&c, n));
        assert_eq!(
            r(&["fireball", "on_hit", "fire_bolt", "branch", "ice_shard"]),
            Ok(())
        );
        assert_eq!(
            r(&["fire_bolt", "branch", "ice_shard"]),
            Err(ChainError::BranchFirst)
        );
        assert_eq!(
            r(&[
                "fireball",
                "on_hit",
                "fire_bolt",
                "branch",
                "ice_shard",
                "branch",
                "acid_glob"
            ]),
            Err(ChainError::BranchTwice)
        );
        assert_eq!(
            r(&["fireball", "on_hit", "fire_bolt", "branch", "on_hit"]),
            Err(ChainError::TriggerAtEnd),
            "the second row needs its own payload"
        );
        assert_eq!(
            r(&["fireball", "on_hit", "branch", "ice_shard"]),
            Err(ChainError::TriggerAtEnd),
            "so does the first"
        );
    }

    #[test]
    fn a_branch_row_counts_at_the_first_trigger_s_weight() {
        let c = shipped();
        let one = stats(&c, &ids(&c, &["fireball", "on_hit", "fire_bolt"]));
        let two = stats(
            &c,
            &ids(&c, &["fireball", "on_hit", "fire_bolt", "branch", "fire_bolt"]),
        );
        let head = stats(&c, &ids(&c, &["fireball"]));
        // the second row adds what the first row's fire bolt added
        assert_eq!(two.power - one.power, one.power - head.power);
        assert_eq!(two.cooldown_ms, one.cooldown_ms, "the first payload's cooldown");
    }

    #[test]
    fn an_invasion_summons_before_any_trigger() {
        let c = shipped();
        let r = |n: &[&str]| check(&c, &ids(&c, n));
        assert_eq!(
            r(&["invade", "fire_bolt", "on_expire", "skeleton"]),
            Err(ChainError::InvadeWithoutSummon),
            "a summon behind a trigger never comes (R4.21)"
        );
        assert_eq!(r(&["invade", "skeleton", "on_death", "fire_bolt"]), Ok(()));
        assert_eq!(r(&["invade", "skeleton"]), Ok(()));
        assert_eq!(
            r(&["fire_bolt", "on_expire", "invade", "skeleton"]),
            Err(ChainError::InvadeNotFirst),
            "an invasion is a spell that starts with Invade"
        );
    }

    #[test]
    fn chains_that_cannot_cast_are_refused_with_their_reason() {
        let c = shipped();
        let r = |n: &[&str]| check(&c, &ids(&c, n));
        assert_eq!(check(&c, &[]), Err(ChainError::Empty));
        assert_eq!(r(&["on_hit", "fire_bolt"]), Err(ChainError::NoPayloadFirst));
        assert_eq!(r(&["fire_bolt", "on_hit"]), Err(ChainError::TriggerAtEnd));
        assert_eq!(r(&["fire_bolt", "ring"]), Err(ChainError::ModifierAtEnd));
        assert_eq!(
            r(&["ring", "on_hit", "fire_bolt"]),
            Err(ChainError::NoPayloadFirst)
        );
        assert_eq!(r(&["invade", "fire_bolt"]), Err(ChainError::InvadeWithoutSummon));
        assert_eq!(r(&["teaching_elemental"]), Err(ChainError::Helper));
        assert_eq!(r(&["sub_tower"]), Err(ChainError::PlaceOnly));
        assert_eq!(
            r(&[
                "fire_bolt",
                "on_hit",
                "ice_shard",
                "on_hit",
                "acid_glob",
                "on_hit"
            ]),
            Err(ChainError::TriggerAtEnd)
        );
        assert_eq!(
            r(&[
                "fire_bolt",
                "on_hit",
                "ice_shard",
                "on_hit",
                "acid_glob",
                "on_hit",
                "fire_bolt",
                "on_hit",
                "ice_shard"
            ]),
            Err(ChainError::TooLong),
            "max_chain 8: nine pieces is one too many"
        );
        assert_eq!(
            r(&[
                "fire_bolt",
                "fire_bolt",
                "fire_bolt",
                "fire_bolt",
                "fire_bolt",
                "fire_bolt"
            ]),
            Ok(())
        );
        assert_eq!(ChainError::TriggerAtEnd.key(), "lab.trigger_at_end");
    }

    #[test]
    fn bone_parade_is_an_invasion_and_a_fire_bolt_is_not() {
        let c = shipped();
        let parade = ids(&c, &["invade", "ring", "skeleton", "on_death", "ghost"]);
        assert_eq!(check(&c, &parade), Ok(()));
        assert!(is_invasion(&c, &parade));
        assert!(!is_invasion(&c, &ids(&c, &["fire_bolt"])));
    }
}

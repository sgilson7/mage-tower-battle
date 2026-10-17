//! The game's content: schools, damage types and their chart, spell
//! pieces, enemies, things to place, helpers and every balance number.
//! It lives in `data/content.json`; the core parses it and refuses a file
//! that would break a rule, naming the entry. Ids in the file are strings;
//! after parsing, everything refers to entries by index.

use serde::{Deserialize, Serialize};

use crate::fx::{Fx, ONE};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct School {
    pub id: String,
    pub name: String,
    /// The icon's shape, so a school is never told by colour alone.
    pub shape: String,
    pub colour: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageType {
    pub id: String,
    pub name: String,
    pub colour: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Oil,
    Lava,
    Thorns,
    ManaSpring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Freeze,
    Charm,
    Slow,
    Fear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arrange {
    Ring,
    Twin,
    Fan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum On {
    Hit,
    Death,
    Expire,
}

/// What a payload does where it lands. Distances in cm, times in ms,
/// speeds in cm/s, as written in the file; the core converts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "does", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    /// A projectile toward the target.
    Bolt {
        damage_type: String,
        damage: i32,
        speed: i32,
        range: i32,
        radius: i32,
        knock: i32,
    },
    /// A burst at the target point.
    Burst {
        damage_type: String,
        damage: i32,
        range: i32,
        radius: i32,
        knock: i32,
        lift: i32,
    },
    /// A lasting zone on the ground.
    Ground {
        field: Field,
        damage_type: String,
        dps: i32,
        range: i32,
        radius: i32,
        life: i32,
        slow_pct: i32,
    },
    /// A status on enemies around the target.
    Hex {
        status: Status,
        range: i32,
        radius: i32,
        life: i32,
    },
    /// Bodies that fight for the caster, or, behind Invade, for the
    /// caster against a rival.
    Summon { enemy: String, count: i32 },
    /// A thing to place on the grid; a student casts it for a while.
    Raise { feature: String },
    /// A shield on the tower that takes damage before the score does.
    Ward { absorb: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Mods {
    pub damage_pct: i32,
    pub area_pct: i32,
    pub speed_pct: i32,
    pub cooldown_pct: i32,
    pub pierce: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PieceKind {
    Payload {
        effect: Effect,
        power: i32,
        cooldown: i32,
    },
    Modifier {
        mods: Mods,
        power_pct: i32,
    },
    Formation {
        arrange: Arrange,
        copies: i32,
        share_pct: i32,
    },
    Invade,
    Trigger {
        on: On,
    },
    /// A whole spell the archmage casts at a station: a helper.
    Helper {
        helper: String,
    },
    /// A trigger's second row (T17): the sets after it start when the same
    /// trigger fires, beside the first row's.
    Branch,
    /// A spell the archmage casts on themself (T24): for `life_ms` (0 for
    /// the rest of the day), mana returns `regen_pct` faster, holds
    /// `max_pct` more, and costs `discount_pct` less.
    SelfCast {
        mana: i32,
        #[serde(default)]
        regen_pct: i32,
        #[serde(default)]
        max_pct: i32,
        #[serde(default)]
        discount_pct: i32,
        life_ms: i32,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Piece {
    pub id: String,
    pub name: String,
    pub school: String,
    pub text: String,
    /// The glyph's strokes, from `glyph::STROKES`.
    pub glyph: Vec<String>,
    #[serde(flatten)]
    pub kind: PieceKind,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enemy {
    pub id: String,
    pub name: String,
    pub damage_type: String,
    pub model: String,
    pub health: i32,
    /// cm/s
    pub speed: i32,
    /// cm
    pub radius: i32,
    pub height: i32,
    pub mass: i32,
    pub attack: i32,
    /// ms between attacks
    pub attack_every: i32,
    /// cm
    pub reach: i32,
    /// Only invasions and summons bring it.
    #[serde(default)]
    pub summoned_only: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    pub id: String,
    pub name: String,
    /// Cells it covers, as offsets in a line along its facing.
    pub length: i32,
    pub blocks: bool,
    /// 0 for a feature that never breaks.
    pub health: i32,
    pub mana: i32,
    /// Seconds a builder golem takes instead, or 0 if golems cannot build it.
    pub golem_s: i32,
    /// Milliseconds a cell takes to rise once the archmage places it with
    /// mana (T1, T23); `mana` is per cell too.
    pub raise_ms: i32,
    /// How long it stands when a student casts it in a battle, ms; 0 if
    /// only the archmage can place it.
    pub cast_life: i32,
    /// Damage a second to enemies standing on it, and its type.
    #[serde(default)]
    pub dps: i32,
    #[serde(default)]
    pub damage_type: String,
    /// Enemies within this many cm turn to attack it.
    #[serde(default)]
    pub lure: i32,
    /// Student seats it adds.
    #[serde(default)]
    pub seats: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Helper {
    pub id: String,
    pub name: String,
    pub model: String,
    pub mana: i32,
    /// Seconds it spends on one job.
    pub job_s: i32,
}

/// Every tuning number, in the file's units.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Balance {
    pub score_start: i32,
    /// A mage on the field (T42): health is `mage_hp` plus `mage_hp_per_100`
    /// for every 100% of effectiveness summed over the spells they know;
    /// speed likewise, in cm/s.
    pub mage_hp: i32,
    pub mage_hp_per_100: i32,
    pub mage_speed_cms: i32,
    pub mage_speed_per_100_cms: i32,
    /// Below this percent of health a mage is hurt (T41).
    pub hurt_pct: i32,
    /// An enemy this close to the tower is near it (T41), in cm.
    pub near_tower_cm: i32,
    /// Raiders join a rival's battle this long after it starts (T40), and
    /// each fall on the journey makes them this much later (T45).
    pub raid_arrive_s: i32,
    pub journey_late_s: i32,
    /// Percent of a raider's spell damage that reaches the tower's score
    /// (T40); students and summons take it whole.
    pub raid_tower_pct: i32,
    /// The pieces a new archmage knows before reading a book (T1).
    pub start_pieces: Vec<String>,
    /// Your own summons (T25): percent of a summon's count, at most this
    /// many alive for each student, each lasting this long; and percent of
    /// a summoning spell's mana.
    pub ally_summon_pct: i32,
    pub summons_per_student: i32,
    pub summon_life_s: i32,
    pub summon_mana_pct: i32,
    /// The book pile (T39): books to shelve for one find; and copies of
    /// known books in the pile.
    pub sort_books: i32,
    pub dud_books: i32,
    /// Seconds a book's card stays on screen (T10).
    pub card_s: i32,
    /// Seconds the title's camera takes to turn once round the tower (T11,
    /// T47).
    pub demo_turn_s: i32,
    /// Seconds the title's archmage stands at each station (T47).
    pub demo_stay_s: i32,
    pub days: i32,
    pub prep_s: i32,
    pub battle_s: i32,
    /// A battle ends when every enemy has fallen or reached the tower, or
    /// after this many seconds (T54); `battle_s` is how long the waves take
    /// to arrive.
    pub battle_max_s: i32,
    pub practice_prep_s: i32,
    pub ragdoll_cap: i32,
    pub mana_k: i32,
    pub mana_c: i32,
    pub trigger_weight_pct: i32,
    pub cast_budget: i32,
    pub max_shots: i32,
    pub max_effects: i32,
    pub max_chain: i32,
    pub max_depth: i32,
    pub effect_min_pct: i32,
    pub effect_max_pct: i32,
    pub area_floor_pct: i32,
    pub status_cap_ms: i32,
    pub status_repeat_pct: i32,
    /// A body thrown faster than this (cm/s) becomes a ragdoll.
    pub ragdoll_speed: i32,
    pub walk_cms: i32,
    pub read_ms: i32,
    pub archmage_mana: i32,
    pub archmage_regen: i32,
    pub tower_seats: i32,
    pub seat_range_pct: i32,
    pub ground_range_pct: i32,
    pub student_mana: [i32; 2],
    pub student_regen: [i32; 2],
    pub school_mod: [i32; 2],
    pub variance: i32,
    pub applicants: i32,
    pub start_students: i32,
    /// The most students a college holds (T56).
    pub max_students: i32,
    /// Spells a student can hold (T50), as (quality from, slots) pairs.
    pub spell_slots: Vec<[i32; 2]>,
    /// A class lesson (T52): each student after the first takes this
    /// percent off the effectiveness a drawing earns, and adds a stroke from
    /// `class_strokes` to the drawing.
    pub class_pct: i32,
    pub class_strokes: Vec<String>,
    /// Percent of a piece's mana it holds when the builder golem raised it
    /// (T61).
    pub golem_hold_pct: i32,
    /// Ticks of the rehearsal played each frame (T48).
    pub rehearsal_ticks: i32,
    pub books: i32,
    pub band_px: Vec<[i32; 2]>,
    pub perfect_ticks: i32,
    pub good_ticks: i32,
    pub wild_waves: i32,
    /// An enemy that reaches the tower strikes it once for this many times its attack.
    pub leak_mult: i32,
    /// The same for enemies a rival's invasion brought.
    pub invasion_leak_mult: i32,
    /// Percent of a spell's mana an archmage pays to cast it as an invasion from the book.
    pub invasion_mana_pct: i32,
    /// Percent of an invasion's numbers each rival gets, now that it goes to
    /// them all (T29).
    pub invasion_spread_pct: i32,
    /// Percent of summon numbers in an invasion the archmage casts.
    pub archmage_count_pct: i32,
    /// Percent of the wild enemies' base numbers.
    pub wild_pct: i32,
    /// The difficulties a match or a trial may be played at (T30): each
    /// scales the wild enemies' numbers (on top of `wild_pct`) and health.
    pub difficulties: Vec<Difficulty>,
    pub default_difficulty: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentFile {
    /// A note for people; ignored.
    #[serde(rename = "_about", default)]
    pub about: String,
    pub schools: Vec<School>,
    pub types: Vec<DamageType>,
    /// chart[attacker][defender] = percent; missing pairs are 100.
    pub chart: Vec<(String, String, i32)>,
    pub pieces: Vec<Piece>,
    pub enemies: Vec<Enemy>,
    pub features: Vec<Feature>,
    pub helpers: Vec<Helper>,
    pub balance: Balance,
    /// Parts of applicants' names.
    pub names: Names,
    /// Books that hold a whole spell.
    pub tomes: Vec<Tome>,
    pub trials: Vec<Trial>,
    pub achievements: Vec<Achievement>,
    /// What a student's temperament is, and the behaviour tree it gives
    /// them on the field (T41).
    pub temperaments: Vec<Temperament>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Temperament {
    pub id: String,
    pub name: String,
    pub text: String,
    pub tree: crate::mind::RawNode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Difficulty {
    pub id: String,
    pub count_pct: i32,
    pub health_pct: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    pub id: String,
    pub name: String,
    pub text: String,
    /// "time" (seconds, lower is better), "damage" (lower is better) or
    /// "dealt" (higher is better).
    pub measure: String,
    /// Bronze, Silver, Gold and Archmage, in the measure's units.
    pub medals: [i32; 4],
    /// A trial's own goal where it has one: the crash course's class average.
    #[serde(default)]
    pub target: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Names {
    pub first: Vec<String>,
    pub last: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tome {
    pub name: String,
    pub pieces: Vec<String>,
}

/// Parsed and checked content, with indices resolved.
#[derive(Debug, Clone)]
pub struct Content {
    pub file: ContentFile,
    /// rate[attacker][defender] in percent.
    pub rate: Vec<Vec<i32>>,
    /// For each piece: its school's index.
    pub piece_school: Vec<usize>,
    /// Each temperament's behaviour tree, flattened (T41).
    pub minds: Vec<crate::mind::Tree>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    NotJson(String),
    Unknown {
        what: &'static str,
        id: String,
        in_entry: String,
    },
    Duplicate {
        what: &'static str,
        id: String,
    },
    BelowHalf {
        attacker: String,
        defender: String,
        pct: i32,
    },
    OutOfRange {
        field: String,
        value: i32,
    },
    BadGlyph {
        piece: String,
        stroke: String,
    },
    BadTree {
        temperament: String,
        why: String,
    },
}

impl ContentError {
    pub fn message(&self) -> String {
        match self {
            ContentError::NotJson(e) => format!("The content file could not be read: {e}"),
            ContentError::Unknown { what, id, in_entry } => format!("\"{in_entry}\" names a {what} \"{id}\" that the content file does not define."),
            ContentError::Duplicate { what, id } => format!("The content file defines the {what} \"{id}\" twice."),
            ContentError::BelowHalf { attacker, defender, pct } => format!("The type chart gives {attacker} on {defender} {pct}%; nothing may go below 50%, so no enemy is immune."),
            ContentError::OutOfRange { field, value } => format!("The balance value \"{field}\" is {value}, which is out of range."),
            ContentError::BadGlyph { piece, stroke } => format!("The glyph of \"{piece}\" uses a stroke \"{stroke}\" the game cannot draw."),
            ContentError::BadTree { temperament, why } => format!("The tree of the temperament \"{temperament}\" cannot be read: {why}."),
        }
    }
}

fn find(ids: &[&str], what: &'static str, id: &str, in_entry: &str) -> Result<usize, ContentError> {
    ids.iter()
        .position(|x| *x == id)
        .ok_or_else(|| ContentError::Unknown {
            what,
            id: id.to_string(),
            in_entry: in_entry.to_string(),
        })
}

fn unique(ids: &[&str], what: &'static str) -> Result<(), ContentError> {
    for (i, a) in ids.iter().enumerate() {
        if ids[..i].contains(a) {
            return Err(ContentError::Duplicate {
                what,
                id: a.to_string(),
            });
        }
    }
    Ok(())
}

impl Content {
    pub fn parse(json: &str) -> Result<Content, ContentError> {
        let file: ContentFile =
            serde_json::from_str(json).map_err(|e| ContentError::NotJson(e.to_string()))?;
        let schools: Vec<&str> = file.schools.iter().map(|s| s.id.as_str()).collect();
        let types: Vec<&str> = file.types.iter().map(|t| t.id.as_str()).collect();
        let pieces: Vec<&str> = file.pieces.iter().map(|p| p.id.as_str()).collect();
        let enemies: Vec<&str> = file.enemies.iter().map(|e| e.id.as_str()).collect();
        let features: Vec<&str> = file.features.iter().map(|f| f.id.as_str()).collect();
        let helpers: Vec<&str> = file.helpers.iter().map(|h| h.id.as_str()).collect();
        unique(&schools, "school")?;
        unique(&types, "damage type")?;
        unique(&pieces, "spell piece")?;
        unique(&enemies, "enemy")?;
        unique(&features, "feature")?;
        unique(&helpers, "helper")?;
        let mut rate = vec![vec![100; types.len()]; types.len()];
        for (a, d, pct) in &file.chart {
            let ai = find(&types, "damage type", a, "chart")?;
            let di = find(&types, "damage type", d, "chart")?;
            if *pct < 50 {
                return Err(ContentError::BelowHalf {
                    attacker: a.clone(),
                    defender: d.clone(),
                    pct: *pct,
                });
            }
            rate[ai][di] = *pct;
        }
        for s in &file.balance.class_strokes {
            if !crate::glyph::STROKES.contains(&s.as_str()) {
                return Err(ContentError::BadGlyph {
                    piece: "class_strokes".into(),
                    stroke: s.clone(),
                });
            }
        }
        let mut piece_school = Vec::new();
        for p in &file.pieces {
            piece_school.push(find(&schools, "school", &p.school, &p.id)?);
            for s in &p.glyph {
                if !crate::glyph::STROKES.contains(&s.as_str()) {
                    return Err(ContentError::BadGlyph {
                        piece: p.id.clone(),
                        stroke: s.clone(),
                    });
                }
            }
            if let PieceKind::Payload { effect, .. } = &p.kind {
                match effect {
                    Effect::Bolt { damage_type, .. }
                    | Effect::Burst { damage_type, .. }
                    | Effect::Ground { damage_type, .. } => {
                        find(&types, "damage type", damage_type, &p.id)?;
                    }
                    Effect::Summon { enemy, .. } => {
                        find(&enemies, "enemy", enemy, &p.id)?;
                    }
                    Effect::Raise { feature } => {
                        find(&features, "feature", feature, &p.id)?;
                    }
                    Effect::Hex { .. } | Effect::Ward { .. } => {}
                }
            }
            if let PieceKind::Helper { helper } = &p.kind {
                find(&helpers, "helper", helper, &p.id)?;
            }
        }
        for e in &file.enemies {
            find(&types, "damage type", &e.damage_type, &e.id)?;
        }
        for t in &file.tomes {
            for p in &t.pieces {
                find(&pieces, "spell piece", p, &t.name)?;
            }
        }
        if file.names.first.is_empty() || file.names.last.is_empty() {
            return Err(ContentError::OutOfRange {
                field: "names".into(),
                value: 0,
            });
        }
        let b = &file.balance;
        for (field, value, lo, hi) in [
            ("score_start", b.score_start, 1, 1_000_000),
            ("days", b.days, 1, 10),
            ("prep_s", b.prep_s, 10, 900),
            ("battle_s", b.battle_s, 10, 600),
            ("battle_max_s", b.battle_max_s, b.battle_s, 1200),
            ("max_students", b.max_students, 1, 12),
            ("class_pct", b.class_pct, 0, 14),
            ("golem_hold_pct", b.golem_hold_pct, 0, 100),
            ("rehearsal_ticks", b.rehearsal_ticks, 1, 600),
            ("ragdoll_cap", b.ragdoll_cap, 0, 400),
            ("max_depth", b.max_depth, 1, 4),
            ("max_chain", b.max_chain, 2, 8),
            ("effect_min_pct", b.effect_min_pct, 1, 100),
            ("effect_max_pct", b.effect_max_pct, 50, 150),
            // a tower seat's number stays below the sub-towers' (T33)
            ("tower_seats", b.tower_seats, 0, crate::battle::SUB_SEAT),
        ] {
            if !(lo..=hi).contains(&value) {
                return Err(ContentError::OutOfRange {
                    field: field.to_string(),
                    value,
                });
            }
        }
        let ids: Vec<&str> = file.temperaments.iter().map(|t| t.id.as_str()).collect();
        unique(&ids, "temperament")?;
        if file.temperaments.is_empty() {
            return Err(ContentError::OutOfRange {
                field: "temperaments".into(),
                value: 0,
            });
        }
        let mut minds = Vec::new();
        for t in &file.temperaments {
            minds.push(
                crate::mind::Tree::build(&t.tree).map_err(|e| ContentError::BadTree {
                    temperament: t.id.clone(),
                    why: e,
                })?,
            );
        }
        Ok(Content {
            file,
            rate,
            piece_school,
            minds,
        })
    }

    pub fn piece(&self, id: &str) -> Option<usize> {
        self.file.pieces.iter().position(|p| p.id == id)
    }
    pub fn enemy(&self, id: &str) -> Option<usize> {
        self.file.enemies.iter().position(|e| e.id == id)
    }
    pub fn feature(&self, id: &str) -> Option<usize> {
        self.file.features.iter().position(|f| f.id == id)
    }
    pub fn helper(&self, id: &str) -> Option<usize> {
        self.file.helpers.iter().position(|h| h.id == id)
    }
    pub fn damage_type(&self, id: &str) -> usize {
        self.file.types.iter().position(|t| t.id == id).unwrap_or(0)
    }
    /// A tome's pieces as indices.
    pub fn tome(&self, i: usize) -> Vec<u16> {
        self.file.tomes[i]
            .pieces
            .iter()
            .filter_map(|p| self.piece(p))
            .map(|p| p as u16)
            .collect()
    }
    /// A difficulty by index, the default one when out of range.
    pub fn difficulty(&self, i: i32) -> &Difficulty {
        let d = &self.balance().difficulties;
        d.get(i.max(0) as usize)
            .or_else(|| d.get(self.balance().default_difficulty as usize))
            .unwrap_or(&d[0])
    }

    pub fn balance(&self) -> &Balance {
        &self.file.balance
    }
    /// The damage rate in percent for one type striking another.
    pub fn rate(&self, attacker: usize, defender: usize) -> i32 {
        self.rate
            .get(attacker)
            .and_then(|r| r.get(defender))
            .copied()
            .unwrap_or(100)
    }
}

/// Centimetres to Q16 metres.
pub fn cm(v: i32) -> Fx {
    (v as i64 * ONE as i64 / 100) as Fx
}

/// cm/s to Q16 metres a tick.
pub fn cms(v: i32) -> Fx {
    (v as i64 * ONE as i64 / 100 / crate::geom::TPS as i64) as Fx
}

/// Milliseconds to ticks, rounded down, at least 1 for a positive time.
pub fn ticks(ms: i32) -> i32 {
    if ms <= 0 {
        return 0;
    }
    (ms * crate::geom::TPS / 1000).max(1)
}

/// The shipped content, compiled in for tests; the page sends the same file.
#[cfg(test)]
pub fn shipped() -> Content {
    Content::parse(include_str!("../../../data/content.json")).expect("data/content.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_content_loads_and_every_type_pair_is_at_least_half() {
        let c = shipped();
        assert_eq!(c.file.schools.len(), 5);
        assert_eq!(c.file.types.len(), 7);
        for a in 0..7 {
            for d in 0..7 {
                assert!(c.rate(a, d) >= 50, "{a} on {d}");
                assert!([50, 100, 200].contains(&c.rate(a, d)), "only ×½, ×1 and ×2");
            }
        }
        let fire = c.damage_type("fire");
        assert_eq!(c.rate(fire, c.damage_type("leaf")), 200);
        assert_eq!(c.rate(fire, c.damage_type("earth")), 50);
    }

    #[test]
    fn each_school_has_at_least_four_pieces_and_its_own_shape() {
        let c = shipped();
        for (i, s) in c.file.schools.iter().enumerate() {
            let n = c.piece_school.iter().filter(|&&k| k == i).count();
            assert!(n >= 4, "{} has {n} pieces", s.id);
            assert_eq!(
                c.file.schools.iter().filter(|o| o.shape == s.shape).count(),
                1,
                "{} shares a shape",
                s.id
            );
        }
    }

    fn edited(f: impl Fn(&mut serde_json::Value)) -> Result<Content, ContentError> {
        let mut v: serde_json::Value =
            serde_json::from_str(include_str!("../../../data/content.json")).unwrap();
        f(&mut v);
        Content::parse(&v.to_string())
    }

    fn piece_mut<'a>(v: &'a mut serde_json::Value, id: &str) -> &'a mut serde_json::Value {
        v["pieces"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|p| p["id"] == id)
            .expect("piece in content.json")
    }

    #[test]
    fn a_chart_entry_below_half_is_refused_by_name() {
        let e = edited(|v| {
            let row = v["chart"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|r| r[0] == "fire" && r[1] == "earth")
                .unwrap();
            row[2] = 0.into();
        })
        .unwrap_err();
        assert_eq!(
            e,
            ContentError::BelowHalf {
                attacker: "fire".into(),
                defender: "earth".into(),
                pct: 0
            }
        );
        assert!(e.message().contains("no enemy is immune"));
    }

    #[test]
    fn an_unknown_school_and_a_duplicate_piece_are_refused() {
        let e = edited(|v| piece_mut(v, "fire_bolt")["school"] = "necromancy".into()).unwrap_err();
        assert!(matches!(e, ContentError::Unknown { what: "school", .. }), "{e:?}");
        let e = edited(|v| piece_mut(v, "ice_shard")["id"] = "fire_bolt".into()).unwrap_err();
        assert_eq!(
            e,
            ContentError::Duplicate {
                what: "spell piece",
                id: "fire_bolt".into()
            }
        );
    }

    #[test]
    fn a_glyph_stroke_the_game_cannot_draw_is_refused() {
        let e = edited(|v| piece_mut(v, "fire_bolt")["glyph"] = serde_json::json!(["teapot"])).unwrap_err();
        assert!(matches!(e, ContentError::BadGlyph { .. }), "{e:?}");
    }

    #[test]
    fn unit_conversions_match_hand_values() {
        assert_eq!(cm(150), ONE * 3 / 2);
        assert_eq!(cms(600), ONE / 10);
        assert_eq!(ticks(1000), 60);
        assert_eq!(ticks(10), 1);
        assert_eq!(ticks(0), 0);
    }
}

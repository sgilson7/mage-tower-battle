//! A match: up to four archmages over a number of days, each day a
//! preparation and then a battle at every tower at once. Scores only fall,
//! by the damage each tower takes; the highest score after the last day
//! wins (the person's rule, DECISIONS.md 2026-10-16).
//!
//! Invasions reach the target's invasion book at two times: an archmage's
//! own on the tick it is cast, students' when the battle starts. Students'
//! invasions go to the rival leading on score (SECOND-ORDER R3.21).

use serde::{Deserialize, Serialize};

use crate::battle::{Battle, Incoming, Plan, Wild};
use crate::bot::{self, Style};
use crate::content::Content;
use crate::geom::TPS;
use crate::prep::Prep;
use crate::rng::Rng;
use crate::spell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Practice,
    Versus,
    Online,
    Trial(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub days: i32,
    pub prep_s: i32,
    pub battle_s: i32,
    /// Battle ticks run per display tick: 1 to 4.
    pub speed: i32,
    pub ragdoll_cap: i32,
    /// The computer rivals' skill, 30 to 95.
    pub skill: i32,
}

impl Settings {
    pub fn from_content(c: &Content) -> Settings {
        let b = c.balance();
        Settings {
            days: b.days,
            prep_s: b.prep_s,
            battle_s: b.battle_s,
            speed: 1,
            ragdoll_cap: b.ragdoll_cap,
            skill: 65,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeatKind {
    Local,
    Remote,
    Bot(Style),
    Empty,
}

#[derive(Debug, Clone)]
pub struct Seat {
    pub kind: SeatKind,
    pub name: String,
    pub score: i32,
    pub prep: Option<Prep>,
    /// A remote player's plan for this day's battle, once it has arrived.
    pub plan: Option<Plan>,
    /// Invasions aimed at this seat today, with the prep tick from which
    /// the book shows them.
    pub incoming: Vec<(Incoming, i32)>,
    /// A remote player's students' invasions, sent with their plan.
    pub student_invasions: Vec<(Vec<u16>, i32)>,
    pub taken: Vec<i32>,
    pub recruited_any: bool,
}

impl Seat {
    pub fn active(&self) -> bool {
        self.kind != SeatKind::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Prep,
    /// Waiting for remote plans.
    Waiting,
    Battle,
    Summary,
    Results,
}

/// What one tower's battle came to, for the day summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayResult {
    pub seat: u8,
    pub taken: i32,
    /// Of `taken`, how much rivals' invasions did.
    pub from_invasions: i32,
    pub kills: i32,
    pub frozen: i32,
    pub charmed: i32,
    pub warded: i32,
    pub dealt: i32,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub seed: u64,
    pub mode: Mode,
    pub settings: Settings,
    pub day: i32,
    pub phase: Phase,
    pub seats: Vec<Seat>,
    pub battles: Vec<Option<Battle>>,
    pub wild: Vec<Wild>,
    pub local: u8,
    pub prep_tick: i32,
    pub results: Vec<DayResult>,
    /// For trials: ticks since the trial began.
    pub elapsed: i32,
    pub trial_done: Option<i32>,
    rng: Rng,
    used_main: Vec<u16>,
}

pub fn field_seed(seed: u64, seat: u8) -> u64 {
    seed ^ (seat as u64 + 1).wrapping_mul(0x5bd1_e995)
}

impl Match {
    /// A match with `kinds` in the four seats; `local` is this browser's.
    pub fn new(
        c: &Content,
        seed: u64,
        mode: Mode,
        settings: Settings,
        kinds: [SeatKind; 4],
        names: [String; 4],
        local: u8,
    ) -> Match {
        let seats = kinds
            .iter()
            .enumerate()
            .map(|(i, &kind)| {
                let prep = match kind {
                    SeatKind::Local | SeatKind::Bot(_) => {
                        Some(Prep::new(c, i as u8, seed, field_seed(seed, i as u8)))
                    }
                    _ => None,
                };
                Seat {
                    kind,
                    name: names[i].clone(),
                    score: c.balance().score_start,
                    prep,
                    plan: None,
                    incoming: Vec::new(),
                    student_invasions: Vec::new(),
                    taken: Vec::new(),
                    recruited_any: false,
                }
            })
            .collect();
        let mut m = Match {
            seed,
            mode,
            settings,
            day: 0,
            phase: Phase::Prep,
            seats,
            battles: vec![None, None, None, None],
            wild: Vec::new(),
            local,
            prep_tick: 0,
            results: Vec::new(),
            elapsed: 0,
            trial_done: None,
            rng: Rng::new(seed ^ 0xa11ce),
            used_main: Vec::new(),
        };
        m.start_day(c);
        m
    }

    pub fn prep_ticks(&self) -> i32 {
        self.settings.prep_s * TPS
    }

    /// The day's wild enemies, the same for every tower: a main enemy no
    /// earlier day had, and two smaller groups, more each day.
    fn draw_wild(&mut self, c: &Content) -> Vec<Wild> {
        let wild: Vec<u16> = (0..c.file.enemies.len())
            .filter(|&i| !c.file.enemies[i].summoned_only)
            .map(|i| i as u16)
            .collect();
        let fresh: Vec<u16> = wild
            .iter()
            .copied()
            .filter(|e| !self.used_main.contains(e))
            .collect();
        let pool = if fresh.is_empty() { &wild } else { &fresh };
        let main = pool[self.rng.below(pool.len() as u32) as usize];
        self.used_main.push(main);
        let ramp = (100 + 35 * self.day) * c.balance().wild_pct / 100;
        let ticks = self.settings.battle_s * TPS;
        let count = |e: u16, base: i32| {
            let hp = c.file.enemies[e as usize].health.max(1);
            // about base × 300 health a group, more for later days
            (base * 300 / hp).clamp(1, 60) * ramp / 100
        };
        let mut out = vec![Wild {
            enemy: main,
            count: count(main, 6).max(2),
            // the main group arrives at once; the others a quarter and half way in
            at_tick: TPS,
        }];
        for k in 0..c.balance().wild_waves - 1 {
            let e = wild[self.rng.below(wild.len() as u32) as usize];
            out.push(Wild {
                enemy: e,
                count: count(e, 3),
                at_tick: ticks * (k + 1) / 4,
            });
        }
        out
    }

    pub fn start_day(&mut self, c: &Content) {
        self.phase = Phase::Prep;
        self.prep_tick = 0;
        self.wild = self.draw_wild(c);
        self.battles = vec![None, None, None, None];
        let ticks = self.prep_ticks();
        let leader = self.leader(self.local);
        let day = self.day;
        for i in 0..self.seats.len() {
            self.seats[i].incoming.clear();
            self.seats[i].plan = None;
            self.seats[i].student_invasions.clear();
            if let Some(p) = self.seats[i].prep.as_mut() {
                p.start_day(c, ticks);
            }
        }
        // bots plan their whole day now; their archmage's invasions appear
        // in a target's book partway through the preparation
        for i in 0..self.seats.len() {
            if let SeatKind::Bot(style) = self.seats[i].kind {
                let target = if style == Style::AttackMage {
                    leader.max(0) as u8
                } else {
                    self.leader(i as u8) as u8
                };
                let mut rng = Rng::new(self.seed ^ (self.day as u64 * 977) ^ i as u64);
                let skill = self.settings.skill;
                if let Some(p) = self.seats[i].prep.as_mut() {
                    let target = if target as usize == i {
                        ((i + 1) % 4) as u8
                    } else {
                        target
                    };
                    bot::day(c, p, style, skill, target, day > 0, &mut rng);
                }
                let sent = self.seats[i]
                    .prep
                    .as_ref()
                    .map(|p| p.sent.clone())
                    .unwrap_or_default();
                for (k, (t, s)) in sent.into_iter().enumerate() {
                    let chain = self.seats[i]
                        .prep
                        .as_ref()
                        .map(|p| p.spells[s as usize].clone())
                        .unwrap_or_default();
                    let at = ticks * (3 + k as i32) / 10;
                    self.deliver(
                        t,
                        Incoming {
                            from: i as u8,
                            chain,
                            strength: 100,
                            count_pct: c.balance().archmage_count_pct,
                        },
                        at,
                    );
                }
            }
        }
    }

    fn deliver(&mut self, target: u8, inv: Incoming, visible_at: i32) {
        if let Some(s) = self.seats.get_mut(target as usize) {
            if s.active() {
                s.incoming.push((inv, visible_at));
            }
        }
    }

    /// The rival leading on score, other than `not`; the lowest seat on a tie.
    pub fn leader(&self, not: u8) -> i32 {
        let mut best: Option<(i32, usize)> = None;
        for (i, s) in self.seats.iter().enumerate() {
            if i as u8 == not || !s.active() {
                continue;
            }
            if best.map(|b| s.score > b.0).unwrap_or(true) {
                best = Some((s.score, i));
            }
        }
        best.map(|b| b.1 as i32).unwrap_or(-1)
    }

    pub fn local_prep(&self) -> Option<&Prep> {
        self.seats.get(self.local as usize).and_then(|s| s.prep.as_ref())
    }

    pub fn local_prep_mut(&mut self) -> Option<&mut Prep> {
        let l = self.local as usize;
        self.seats.get_mut(l).and_then(|s| s.prep.as_mut())
    }

    /// The local archmage's invasions not yet delivered, for the online
    /// layer to send; delivered here at once for the other seats.
    fn route_local_invasions(&mut self, c: &Content) -> Vec<(u8, Vec<u16>)> {
        let l = self.local as usize;
        let mut out = Vec::new();
        let Some(p) = self.seats.get_mut(l).and_then(|s| s.prep.as_mut()) else {
            return out;
        };
        let sent: Vec<(u8, u16)> = std::mem::take(&mut p.sent);
        let chains: Vec<(u8, Vec<u16>)> = sent
            .into_iter()
            .map(|(t, s)| (t, p.spells[s as usize].clone()))
            .collect();
        let now = self.prep_tick;
        for (t, chain) in chains {
            self.deliver(
                t,
                Incoming {
                    from: l as u8,
                    chain: chain.clone(),
                    strength: 100,
                    count_pct: c.balance().archmage_count_pct,
                },
                now,
            );
            out.push((t, chain));
        }
        out
    }

    /// One display tick. Returns the local invasions cast this tick, for
    /// the online layer.
    pub fn step(&mut self, c: &Content, bits: u32) -> Vec<(u8, Vec<u16>)> {
        let mut sent = Vec::new();
        match self.phase {
            Phase::Prep => {
                self.prep_tick += 1;
                self.elapsed += 1;
                let l = self.local as usize;
                if let Some(seat) = self.seats.get_mut(l) {
                    if let Some(p) = seat.prep.as_mut() {
                        p.step(c, bits);
                        if p.record.recruited > 0 {
                            seat.recruited_any = true;
                        }
                    }
                }
                sent = self.route_local_invasions(c);
                if let Mode::Trial(k) = self.mode {
                    if let Some(t) = self.trial_time_done(c, k) {
                        self.trial_done = Some(t);
                        self.phase = Phase::Results;
                        return sent;
                    }
                }
                let done = self.local_prep().map(|p| p.clock == 0 || p.ready).unwrap_or(true);
                if done {
                    if self.mode == Mode::Online {
                        self.phase = Phase::Waiting;
                    } else {
                        self.begin_battle(c);
                    }
                }
            }
            Phase::Waiting => {
                if self
                    .seats
                    .iter()
                    .all(|s| s.kind != SeatKind::Remote || s.plan.is_some())
                {
                    self.begin_battle(c);
                }
            }
            Phase::Battle => {
                for _ in 0..self.settings.speed.clamp(1, 4) {
                    for b in self.battles.iter_mut().flatten() {
                        b.step(c);
                    }
                }
                if self.battles.iter().flatten().all(|b| b.done()) {
                    self.finish_battle(c);
                }
            }
            Phase::Summary | Phase::Results => {}
        }
        sent
    }

    /// The plan a seat brings: its own preparation's, or the remote one.
    pub fn plan_of(&self, i: usize) -> Plan {
        match (&self.seats[i].prep, &self.seats[i].plan) {
            (_, Some(p)) => p.clone(),
            (Some(p), None) => p.plan(),
            _ => Plan::default(),
        }
    }

    pub fn begin_battle(&mut self, c: &Content) {
        // students' invasions, to the leader
        for i in 0..self.seats.len() {
            let inv = match &self.seats[i].prep {
                Some(p) if self.seats[i].kind != SeatKind::Remote => p.student_invasions(c),
                _ => self.seats[i].student_invasions.clone(),
            };
            let target = self.leader(i as u8);
            if target < 0 {
                continue;
            }
            // pooled by spell: one send each, at the class's mean
            // effectiveness, numbers growing with the square root of the
            // class (SECOND-ORDER R3.26)
            let mut pooled: Vec<(Vec<u16>, i32, i32)> = Vec::new();
            for (chain, eff) in inv {
                match pooled.iter_mut().find(|p| p.0 == chain) {
                    Some(p) => {
                        p.1 += eff;
                        p.2 += 1;
                    }
                    None => pooled.push((chain, eff, 1)),
                }
            }
            for (chain, sum, n) in pooled {
                let at = self.prep_tick;
                let count_pct = crate::geom::isqrt(n as u64 * 10_000) as i32;
                let inv = Incoming {
                    from: i as u8,
                    chain,
                    strength: sum / n,
                    count_pct,
                };
                self.deliver(target as u8, inv, at);
            }
        }
        let ticks = self.settings.battle_s * TPS;
        let cap = self.settings.ragdoll_cap.max(0) as usize;
        for i in 0..self.seats.len() {
            if !self.seats[i].active() {
                continue;
            }
            let plan = self.plan_of(i);
            let incoming: Vec<Incoming> = self.seats[i]
                .incoming
                .iter()
                .map(|(inv, _)| inv.clone())
                .collect();
            let seed = self.seed ^ (self.day as u64 * 7919) ^ (i as u64 * 104_729);
            self.battles[i] = Some(Battle::new(
                c,
                i as u8,
                &plan,
                &incoming,
                &self.wild,
                seed,
                field_seed(self.seed, i as u8),
                ticks,
                cap,
            ));
        }
        self.phase = Phase::Battle;
    }

    fn finish_battle(&mut self, _c: &Content) {
        let mut dealt = [0i32; 4];
        for b in self.battles.iter().flatten() {
            for (from, d) in b.tally.taken_from.iter().enumerate() {
                dealt[from] += d;
            }
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.seats.len() {
            if let Some(b) = &self.battles[i] {
                let t = b.tally.damage_taken;
                self.seats[i].score -= t;
                self.seats[i].taken.push(t);
                self.results.push(DayResult {
                    seat: i as u8,
                    taken: t,
                    from_invasions: b.tally.taken_from.iter().sum(),
                    kills: b.tally.kills,
                    frozen: b.tally.frozen,
                    charmed: b.tally.charmed,
                    warded: b.tally.warded,
                    dealt: dealt[i],
                });
            }
        }
        if let Mode::Trial(k) = self.mode {
            let b = self.battles[self.local as usize].as_ref();
            let v = match k {
                2 => b.map(|b| b.tally.damage_taken).unwrap_or(0),
                _ => self
                    .battles
                    .get(1)
                    .and_then(|b| b.as_ref())
                    .map(|b| b.tally.taken_from[self.local as usize])
                    .unwrap_or(0),
            };
            self.trial_done = Some(v);
            self.phase = Phase::Results;
            return;
        }
        self.phase = Phase::Summary;
    }

    /// Moves from the day summary to the next day or the results.
    pub fn next_day(&mut self, c: &Content) {
        if self.phase != Phase::Summary {
            return;
        }
        self.day += 1;
        if self.day >= self.settings.days {
            self.phase = Phase::Results;
        } else {
            self.start_day(c);
        }
    }

    /// Seats in order of score, highest first; a tie keeps seat order.
    pub fn ranking(&self) -> Vec<usize> {
        let mut r: Vec<usize> = (0..self.seats.len())
            .filter(|&i| self.seats[i].active())
            .collect();
        r.sort_by(|&a, &b| self.seats[b].score.cmp(&self.seats[a].score).then(a.cmp(&b)));
        r
    }

    /// Whether `seat` won outright or shares first place.
    pub fn won(&self, seat: usize) -> bool {
        let top = self.ranking().first().map(|&i| self.seats[i].score).unwrap_or(0);
        self.seats[seat].score == top
    }

    /// What the local archmage's invasion book shows now: invasions whose
    /// time has come. Students' invasions only exist once the battle starts.
    pub fn book(&self) -> Vec<&Incoming> {
        self.book_of(self.local)
    }

    /// What a seat's invasion book shows now.
    pub fn book_of(&self, seat: u8) -> Vec<&Incoming> {
        let Some(s) = self.seats.get(seat as usize) else {
            return Vec::new();
        };
        s.incoming
            .iter()
            .filter(|(_, at)| *at <= self.prep_tick || self.phase != Phase::Prep)
            .map(|(i, _)| i)
            .collect()
    }

    // ---- trials ----

    /// A trial: one day, the local archmage at seat 0, a fixed kit.
    pub fn trial(c: &Content, k: u8) -> Match {
        let b = c.balance();
        let mut settings = Settings::from_content(c);
        settings.days = 1;
        let name = |s: &str| s.to_string();
        let (kinds, prep_s) = match k {
            0 | 1 => (
                [SeatKind::Local, SeatKind::Empty, SeatKind::Empty, SeatKind::Empty],
                600,
            ),
            2 => (
                [SeatKind::Local, SeatKind::Empty, SeatKind::Empty, SeatKind::Empty],
                120,
            ),
            _ => (
                [
                    SeatKind::Local,
                    SeatKind::Bot(Style::Balanced),
                    SeatKind::Empty,
                    SeatKind::Empty,
                ],
                120,
            ),
        };
        settings.prep_s = prep_s;
        let seed = 0x7121_0000 + k as u64;
        let mut m = Match::new(
            c,
            seed,
            Mode::Trial(k),
            settings,
            kinds,
            [name("You"), name("Examiner"), name(""), name("")],
            0,
        );
        let ids = |names: &[&str]| -> Vec<usize> { names.iter().filter_map(|n| c.piece(n)).collect() };
        let p = m.seats[0].prep.as_mut().expect("local prep");
        let kit: Vec<usize> = match k {
            0 => ids(&["glass_wall", "fan", "heavy"]),
            1 => ids(&["fan", "fire_bolt"]),
            2 => ids(&[
                "ring",
                "fan",
                "heavy",
                "fireball",
                "frost_ring",
                "charm",
                "glass_wall",
                "ivory_wall",
                "ward",
                "stone_spike",
                "on_hit",
                "oil_spill",
            ]),
            _ => ids(&[
                "invade",
                "ring",
                "twin",
                "skeleton",
                "ghost",
                "lava_elemental",
                "on_death",
            ]),
        };
        for i in kit {
            p.known[i] = true;
        }
        match k {
            1 => {
                p.students.clear();
                for _ in 0..8 {
                    p.add_plain_student(c, 50);
                }
                p.spells
                    .push(ids(&["fan", "fire_bolt"]).into_iter().map(|i| i as u16).collect());
                p.chosen = 1;
            }
            2 => {
                for _ in 0..10 {
                    p.add_plain_student(c, 60);
                }
            }
            3 => {
                p.students.clear();
                p.mana = b.archmage_mana * 200;
                p.applicants.clear();
            }
            _ => {}
        }
        if k == 2 {
            // the hardest wave there is: day four's ramp, three groups
            m.day = 3;
            m.wild = m.draw_wild(c);
            m.day = 0;
        }
        m
    }

    /// For a time trial, the seconds taken once its goal is met.
    fn trial_time_done(&self, c: &Content, k: u8) -> Option<i32> {
        let p = self.local_prep()?;
        let met = match k {
            0 => {
                let r = &p.record;
                r.books >= 4 && r.spells_made >= 2 && r.recruited >= 2 && r.best_teach > 0 && r.placed >= 1
            }
            1 => {
                let fan = p
                    .spells
                    .iter()
                    .position(|s| s.len() == 2 && spell::name(c, s).starts_with("Fan"))?;
                let n = p.students.len().max(1) as i32;
                let sum: i32 = p
                    .students
                    .iter()
                    .map(|s| {
                        s.known
                            .iter()
                            .find(|k| k.0 as usize == fan)
                            .map(|k| k.1)
                            .unwrap_or(0)
                    })
                    .sum();
                sum / n >= 80
            }
            _ => false,
        };
        met.then_some(self.elapsed / TPS)
    }
}

/// The medal a trial result earns: 0 none, 1 bronze, 2 silver, 3 gold, 4 archmage.
pub fn medal(c: &Content, trial: usize, value: i32) -> i32 {
    let t = &c.file.trials[trial];
    if t.measure == "dealt" {
        t.medals.iter().filter(|&&m| value >= m).count() as i32
    } else {
        t.medals.iter().filter(|&&m| value <= m).count() as i32
    }
}

/// Whether one trial result beats another.
pub fn better(c: &Content, trial: usize, value: i32, than: Option<i32>) -> bool {
    match than {
        None => true,
        Some(b) if c.file.trials[trial].measure == "dealt" => value > b,
        Some(b) => value < b,
    }
}

#[cfg(test)]
mod tests;

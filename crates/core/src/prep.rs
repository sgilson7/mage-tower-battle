//! One archmage's preparation: the round room, the six stations, and
//! everything done at them. Time is the resource (entry 1, decision 5):
//! the day clock runs every tick, walking between stations costs time, and
//! reading, interviewing and drawing take the time they take.
//!
//! The archmage acts at a station only when standing at it. Helpers work
//! on their own while the archmage is elsewhere.

use serde::{Deserialize, Serialize};

use crate::arena::{self, Arena};
use crate::battle::{self, Placed, Plan, StudentPlan};
use crate::content::{self, Content, PieceKind};
use crate::fx::{Fx, ONE};
use crate::geom::{v3, TPS, V3};
use crate::glyph;
use crate::rhythm::Lane;
use crate::rng::Rng;
use crate::spell;

pub const ROOM_RADIUS: Fx = 14 * ONE;
/// A station acts when the archmage is this close to it.
pub const REACH: Fx = ONE * 3;
/// Its furniture keeps the archmage this far from its centre.
pub const PROP: Fx = ONE * 13 / 10;
pub const BODY: Fx = ONE * 2 / 5;
pub const MAX_SPELLS: usize = 12;
/// The target an invasion is sent at: every rival (T29).
pub const EVERY_RIVAL: u8 = 255;

/// Input bits from `web/input.js`, by the actions in `data/controls.json`.
pub const UP: u32 = 1;
pub const DOWN: u32 = 2;
pub const LEFT: u32 = 4;
pub const RIGHT: u32 = 8;
pub const USE: u32 = 16;
pub const BACK: u32 = 32;
pub const NOTE: [u32; 4] = [64, 128, 256, 512];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Station {
    Library,
    Lab,
    Book,
    Map,
    Podium,
    Recruit,
}

pub const STATIONS: [(Station, i32, i32); 6] = [
    (Station::Library, -850, -550),
    (Station::Lab, 850, -550),
    (Station::Book, 0, -950),
    (Station::Map, 0, -150),
    (Station::Podium, 800, 500),
    (Station::Recruit, -800, 500),
];

pub fn station_pos(s: Station) -> V3 {
    let (_, x, z) = STATIONS.iter().find(|t| t.0 == s).copied().unwrap_or((s, 0, 0));
    v3(content::cm(x), 0, content::cm(z))
}

impl Station {
    pub fn key(self) -> &'static str {
        match self {
            Station::Library => "library",
            Station::Lab => "lab",
            Station::Book => "book",
            Station::Map => "map",
            Station::Podium => "podium",
            Station::Recruit => "recruit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Student {
    pub id: u16,
    pub name: String,
    pub school: u8,
    pub weak: u8,
    pub quality: i32,
    pub mana_max: i32,
    pub regen: i32,
    /// Per-school modifier to effectiveness, by school index.
    pub mods: Vec<i32>,
    pub variance: i32,
    /// (spell index, effectiveness)
    pub known: Vec<(u16, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Applicant {
    pub name: String,
    pub school: u8,
    pub weak: u8,
    /// Aptitude the interview is averaged with.
    pub base: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Work {
    Teach(u16),
    Recruit,
    Train,
    Build(Placed),
    /// A wall or tower the archmage placed with mana, rising (T1).
    Raise(Placed),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub helper: u16,
    pub score: i32,
    pub left: i32,
    pub total: i32,
    pub work: Work,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Draw {
    /// Teach a spell to everyone (`None`) or one student.
    Teach {
        spell: u16,
        one: Option<u16>,
    },
    Summon {
        helper: u16,
        piece: u16,
        work: Work,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Idle,
    Reading {
        left: i32,
    },
    Interview {
        lane: Lane,
        one: Option<usize>,
        by_helper: bool,
    },
    Drawing {
        why: Draw,
        guide: Vec<(i32, i32)>,
        drawn: Vec<(i32, i32)>,
        band: i32,
        audience: i32,
        glyphs: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Book {
    Piece(u16),
    Tome(u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Record {
    pub books: i32,
    pub spells_made: i32,
    pub longest_chain: i32,
    pub best_teach: i32,
    pub biggest_class: i32,
    pub recruited: i32,
    pub invasions: i32,
    pub placed: i32,
    pub summoned: i32,
    pub perfect_interview: bool,
    pub golem_built: bool,
    /// One bit per helper kind summoned this match.
    pub summoned_kinds: u32,
}

/// What the page shows after an action: a string key and a value for its
/// placeholder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Prep {
    /// The keys held last tick, to find the ones pressed this tick.
    pub held: u32,
    /// Cells laid on the map today, in order, for undo (T23).
    pub laid: Vec<(i32, i32)>,
    pub seat: u8,
    pub pos: V3,
    pub prev: V3,
    pub yaw: i32,
    pub clock: i32,
    pub day_ticks: i32,
    /// Archmage mana in hundredths.
    pub mana: i32,
    pub pile: Vec<Book>,
    pub known: Vec<bool>,
    pub spells: Vec<Vec<u16>>,
    pub draft: Vec<u16>,
    pub students: Vec<Student>,
    pub applicants: Vec<Applicant>,
    pub activity: Activity,
    pub jobs: Vec<Job>,
    pub placed: Vec<Placed>,
    pub field_seed: u64,
    /// Invasions the archmage cast this day: (target seat, spell index).
    pub sent: Vec<(u8, u16)>,
    pub chosen: u16,
    pub record: Record,
    pub notes: Vec<Note>,
    pub ready: bool,
    pub last_read: Option<Book>,
    next_id: u16,
    rng: Rng,
}

fn lerp(range: [i32; 2], q: i32) -> i32 {
    range[0] + (range[1] - range[0]) * q.clamp(0, 100) / 100
}

/// The overlay the mini-games draw on, in logical pixels (`draw.rs`).
pub const OVER_W: i32 = 1280;
pub const OVER_H: i32 = 720;
/// The drawing area: glyph boxes fit inside this many pixels.
pub const DRAW_W: i32 = 1100;
pub const DRAW_H: i32 = 300;
pub const DRAW_Y: i32 = 230;

/// Glyph units per overlay pixel for an incantation of `n` glyphs.
pub fn units_per_px(n: usize) -> i32 {
    (glyph::width(n) / DRAW_W).max(glyph::BOX / DRAW_H) + 1
}

/// The overlay pixel where the incantation's glyph-space origin sits.
pub fn draw_origin(n: usize) -> (i32, i32) {
    let u = units_per_px(n);
    let w = glyph::width(n) / u;
    ((OVER_W - w) / 2, DRAW_Y)
}

impl Prep {
    pub fn new(c: &Content, seat: u8, seed: u64, field_seed: u64) -> Prep {
        let mut rng = Rng::new(seed ^ ((seat as u64 + 1) * 0x9e37_79b9));
        let mut known = vec![false; c.file.pieces.len()];
        for s in &c.balance().start_pieces {
            if let Some(i) = c.piece(s) {
                known[i] = true;
            }
        }
        let mut pile: Vec<Book> = (0..c.file.pieces.len())
            .filter(|&i| !known[i])
            .map(|i| Book::Piece(i as u16))
            .collect();
        pile.extend((0..c.file.tomes.len()).map(|t| Book::Tome(t as u16)));
        // shuffle by the seed (Fisher–Yates)
        for i in (1..pile.len()).rev() {
            let j = rng.below(i as u32 + 1) as usize;
            pile.swap(i, j);
        }
        let mut p = Prep {
            held: 0,
            laid: Vec::new(),
            seat,
            pos: v3(0, 0, 3 * ONE),
            prev: v3(0, 0, 3 * ONE),
            yaw: crate::fx::HALF_TURN,
            clock: 0,
            day_ticks: 0,
            mana: c.balance().archmage_mana * 100,
            pile,
            known,
            spells: Vec::new(),
            draft: Vec::new(),
            students: Vec::new(),
            applicants: Vec::new(),
            activity: Activity::Idle,
            jobs: Vec::new(),
            placed: Vec::new(),
            field_seed,
            sent: Vec::new(),
            chosen: 0,
            record: Record::default(),
            notes: Vec::new(),
            ready: false,
            last_read: None,
            next_id: 1,
            rng,
        };
        if let Some(fb) = c.piece("fire_bolt") {
            p.spells.push(vec![fb as u16]);
        }
        // the poor students the last archmage left behind
        for _ in 0..c.balance().start_students {
            let a = p.applicant(c);
            let q = p.rng.between(10, 30);
            p.enrol(c, &a, q);
            if let Some(s) = p.students.last_mut() {
                s.known.push((0, 40 + q / 2));
            }
        }
        // the students the last archmage left were not recruited
        p.record.recruited = 0;
        p
    }

    fn name(&mut self, c: &Content) -> String {
        let n = &c.file.names;
        let f = &n.first[self.rng.below(n.first.len() as u32) as usize];
        let l = &n.last[self.rng.below(n.last.len() as u32) as usize];
        format!("{f} {l}")
    }

    fn applicant(&mut self, c: &Content) -> Applicant {
        let k = c.file.schools.len() as u32;
        let school = self.rng.below(k) as u8;
        let weak = ((school as u32 + 1 + self.rng.below(k - 1)) % k) as u8;
        Applicant {
            name: self.name(c),
            school,
            weak,
            base: self.rng.between(25, 85),
        }
    }

    fn enrol(&mut self, c: &Content, a: &Applicant, quality: i32) {
        let b = c.balance();
        let q = quality.clamp(1, 100);
        let mut mods = vec![0; c.file.schools.len()];
        mods[a.school as usize] = lerp([b.school_mod[1] / 2, b.school_mod[1]], q);
        mods[a.weak as usize] = b.school_mod[0];
        let id = self.next_id;
        self.next_id += 1;
        self.students.push(Student {
            id,
            name: a.name.clone(),
            school: a.school,
            weak: a.weak,
            quality: q,
            mana_max: lerp(b.student_mana, q),
            regen: lerp(b.student_regen, q),
            mods,
            variance: b.variance,
            known: Vec::new(),
        });
        self.record.recruited += 1;
    }

    /// A new day: the clock, full mana, fresh applicants.
    pub fn start_day(&mut self, c: &Content, prep_ticks: i32) {
        self.clock = prep_ticks;
        self.day_ticks = prep_ticks;
        self.mana = c.balance().archmage_mana * 100;
        self.applicants = (0..c.balance().applicants).map(|_| self.applicant(c)).collect();
        self.activity = Activity::Idle;
        self.sent.clear();
        self.ready = false;
        self.pos = v3(0, 0, 3 * ONE);
        self.prev = self.pos;
    }

    pub fn station(&self) -> Option<Station> {
        STATIONS
            .iter()
            .map(|t| t.0)
            .find(|&s| station_pos(s).sub(self.pos).flat_len() <= REACH)
    }

    fn note(&mut self, key: &str, value: impl Into<String>) {
        self.notes.push(Note {
            key: key.to_string(),
            value: value.into(),
        });
        if self.notes.len() > 6 {
            self.notes.remove(0);
        }
    }

    pub fn busy(&self) -> bool {
        !matches!(self.activity, Activity::Idle)
    }

    pub fn step(&mut self, c: &Content, bits: u32) {
        // keys that went down this tick: a note or a use counts once a press,
        // however long the key is held (T19, SECOND-ORDER R5.1)
        let down = bits & !self.held;
        self.held = bits;
        if self.clock > 0 {
            self.clock -= 1;
        }
        let b = c.balance();
        self.mana = (self.mana + b.archmage_regen * 100 / TPS).min(b.archmage_mana * 100);
        self.work_jobs(c);
        self.prev = self.pos;
        let mut finished_reading = false;
        let mut finished_lane: Option<(i32, Option<usize>, bool)> = None;
        match &mut self.activity {
            Activity::Reading { left } => {
                *left -= 1;
                finished_reading = *left <= 0;
            }
            Activity::Interview { lane, one, by_helper } => {
                lane.step();
                for (k, bit) in NOTE.iter().enumerate() {
                    if down & bit != 0 {
                        lane.press(k as u8);
                    }
                }
                if lane.done() {
                    finished_lane = Some((lane.score(), *one, *by_helper));
                }
            }
            _ => {}
        }
        if finished_reading {
            self.activity = Activity::Idle;
            self.read_book(c);
        }
        if let Some((score, one, _)) = finished_lane {
            self.activity = Activity::Idle;
            if score == 100 {
                self.record.perfect_interview = true;
            }
            self.hire(c, score, one);
        }
        if bits & BACK != 0 && self.busy() {
            self.activity = Activity::Idle;
            self.note("prep.cancelled", "");
        }
        if self.busy() {
            return;
        }
        // walking
        let mut d = V3::ZERO;
        if bits & UP != 0 {
            d.z -= ONE;
        }
        if bits & DOWN != 0 {
            d.z += ONE;
        }
        if bits & LEFT != 0 {
            d.x -= ONE;
        }
        if bits & RIGHT != 0 {
            d.x += ONE;
        }
        if d != V3::ZERO {
            let step = content::cms(b.walk_cms);
            self.pos = self.pos.add(d.with_len(step));
            self.yaw = crate::geom::yaw_of(d.x, d.z);
            let r = self.pos.flat_len();
            let lim = ROOM_RADIUS - BODY;
            if r > lim {
                self.pos = self.pos.with_len(lim);
            }
            for (s, _, _) in STATIONS {
                let at = station_pos(s);
                let off = self.pos.sub(at).flat();
                let l = off.flat_len();
                if l < PROP + BODY {
                    self.pos = at.add(if l == 0 {
                        v3(0, 0, PROP + BODY)
                    } else {
                        off.with_len(PROP + BODY)
                    });
                }
            }
        }
        if down & USE != 0 {
            let _ = self.primary(c);
        }
    }

    /// What E does at the station the archmage stands at.
    fn primary(&mut self, c: &Content) -> Result<(), &'static str> {
        match self.station() {
            Some(Station::Library) => self.act(c, "library.read", ""),
            Some(Station::Lab) => self.act(c, "lab.save", ""),
            Some(Station::Recruit) => self.act(c, "recruit.one", "0"),
            Some(Station::Podium) => self.act(c, "podium.teach", ""),
            _ => Ok(()),
        }
    }

    fn read_book(&mut self, c: &Content) {
        // skip books that teach only what is already known
        while let Some(book) = self.pile.pop() {
            match book {
                Book::Piece(p) if self.known[p as usize] => continue,
                Book::Piece(p) => {
                    self.known[p as usize] = true;
                    self.record.books += 1;
                    self.last_read = Some(book.clone());
                    self.note("library.learned", c.file.pieces[p as usize].name.clone());
                    return;
                }
                Book::Tome(t) => {
                    let pieces = c.tome(t as usize);
                    for &p in &pieces {
                        self.known[p as usize] = true;
                    }
                    self.record.books += 1;
                    if self.spells.len() < MAX_SPELLS
                        && spell::check(c, &pieces).is_ok()
                        && !self.spells.contains(&pieces)
                    {
                        self.spells.push(pieces);
                    }
                    self.last_read = Some(book.clone());
                    self.note("library.tome", c.file.tomes[t as usize].name.clone());
                    return;
                }
            }
        }
        self.note("library.empty", "");
    }

    fn hire(&mut self, c: &Content, score: i32, one: Option<usize>) {
        let who: Vec<usize> = match one {
            Some(i) if i < self.applicants.len() => vec![i],
            Some(_) => vec![],
            None => (0..self.applicants.len()).collect(),
        };
        let v = c.balance().variance;
        let mut joined = 0;
        for &i in who.iter().rev() {
            let a = self.applicants.remove(i);
            let q = (score + a.base) / 2 + self.rng.between(-v, v);
            // an interview scoring under 20 does not persuade anyone
            if score >= 20 {
                self.enrol(c, &a, q);
                joined += 1;
            }
        }
        self.note("recruit.joined", joined.to_string());
    }

    fn teach(&mut self, c: &Content, spell: u16, one: Option<u16>, score: i32) {
        let Some(chain) = self.spells.get(spell as usize) else {
            return;
        };
        let school = chain
            .iter()
            .find(|&&p| matches!(c.file.pieces[p as usize].kind, PieceKind::Payload { .. }))
            .map(|&p| c.piece_school[p as usize])
            .unwrap_or(0);
        let b = c.balance();
        let mut taught = 0;
        for s in self.students.iter_mut() {
            if one.is_some() && one != Some(s.id) {
                continue;
            }
            let roll = self.rng.between(-s.variance, s.variance);
            // the drawing's score through a steep curve (T26); the school
            // and the roll move it a little, and only a flawless drawing
            // reaches the top
            let base = glyph::effectiveness(score, b.effect_min_pct, b.effect_max_pct);
            let top = if score >= 100 {
                b.effect_max_pct
            } else {
                b.effect_max_pct - 1
            };
            let eff = (base + s.mods[school] + roll).clamp(b.effect_min_pct, top);
            match s.known.iter_mut().find(|k| k.0 == spell) {
                Some(k) => k.1 = k.1.max(eff),
                None => s.known.push((spell, eff)),
            }
            taught += 1;
        }
        self.record.best_teach = self.record.best_teach.max(score);
        self.record.biggest_class = self.record.biggest_class.max(taught);
        self.note("podium.taught", format!("{taught}"));
    }

    fn work_jobs(&mut self, c: &Content) {
        let mut done: Vec<Job> = Vec::new();
        for j in self.jobs.iter_mut() {
            j.left -= 1;
            if j.left <= 0 {
                done.push(j.clone());
            }
        }
        self.jobs.retain(|j| j.left > 0);
        for j in done {
            match j.work {
                Work::Teach(spell) => self.teach(c, spell, None, j.score),
                Work::Recruit => self.hire(c, j.score * 4 / 5, None),
                Work::Train => {
                    let trained = self.students.iter_mut().min_by_key(|s| s.mana_max).map(|s| {
                        s.mana_max += s.mana_max * j.score / 400;
                        s.regen += s.regen * j.score / 400;
                        s.name.clone()
                    });
                    if let Some(name) = trained {
                        self.note("helper.trained", name);
                    }
                }
                Work::Raise(p) => {
                    if self.can_place(c, &p).is_ok() {
                        let name = c.file.features[p.feature as usize].name.clone();
                        self.placed.push(p);
                        self.record.placed += 1;
                        self.note("layout.raised", name);
                    } else {
                        self.note("layout.closes_path", "");
                    }
                }
                Work::Build(p) => {
                    if self.can_place(c, &p).is_ok() {
                        self.placed.push(p);
                        self.record.placed += 1;
                        self.record.golem_built = true;
                        self.note("helper.built", "");
                    } else {
                        self.note("layout.closes_path", "");
                    }
                }
            }
        }
    }

    /// Takes away what stands or rises in a cell, giving its mana back.
    fn erase(&mut self, c: &Content, cell: (i32, i32)) {
        let before = self.jobs.len();
        let mut refund = 0;
        self.jobs.retain(|j| match &j.work {
            Work::Raise(p) if p.cells.contains(&cell) => {
                refund += c.file.features[p.feature as usize].mana;
                false
            }
            _ => true,
        });
        if self.jobs.len() == before {
            if let Some(i) = self.placed.iter().position(|p| p.cells.contains(&cell)) {
                refund += c.file.features[self.placed[i].feature as usize].mana;
                self.placed.remove(i);
            }
        }
        self.laid.retain(|&l| l != cell);
        let cap = c.balance().archmage_mana * 100;
        self.mana = (self.mana + refund * 100).min(cap);
    }

    /// Walls and towers placed with mana that have not risen yet.
    pub fn rising(&self) -> impl Iterator<Item = &Placed> {
        self.jobs.iter().filter_map(|j| match &j.work {
            Work::Raise(p) | Work::Build(p) => Some(p),
            _ => None,
        })
    }

    /// Finishes every rising wall at once: a computer rival plans its whole
    /// day in one call, so its walls rise within the day it budgeted.
    pub fn finish_raises(&mut self, c: &Content) {
        for j in self.jobs.iter_mut() {
            if matches!(j.work, Work::Raise(_) | Work::Build(_)) {
                j.left = 1;
            }
        }
        self.work_jobs(c);
    }

    /// The field with everything placed so far.
    pub fn field(&self, c: &Content) -> Arena {
        let mut a = battle::field(self.field_seed);
        for (k, p) in self.placed.iter().enumerate() {
            if c.file.features[p.feature as usize].blocks {
                let _ = a.block(&p.cells, k as u16);
            }
        }
        a
    }

    fn can_place(&self, c: &Content, p: &Placed) -> Result<(), &'static str> {
        let f = &c.file.features[p.feature as usize];
        // what is still rising counts as standing, so a row of rising walls
        // cannot close the route between them
        let mut a = self.field(c);
        for (k, q) in self.rising().enumerate() {
            if c.file.features[q.feature as usize].blocks {
                let _ = a.block(&q.cells, 500 + k as u16);
            }
        }
        for &(x, z) in &p.cells {
            if !arena::inside(x, z) {
                return Err("layout.off_grid");
            }
            if self
                .placed
                .iter()
                .chain(self.rising())
                .any(|q| q.cells.contains(&(x, z)))
            {
                return Err("layout.taken");
            }
        }
        if f.blocks {
            a.block(&p.cells, 999).map_err(|e| e.key())?;
        } else if p.cells.iter().any(|&(x, z)| a.at(x, z) != arena::Cell::Open) {
            return Err("layout.taken");
        }
        Ok(())
    }

    /// The placement a "feature,x,z,dir" value describes, if this archmage
    /// knows how to raise it.
    fn placement(&self, c: &Content, value: &str) -> Result<Placed, &'static str> {
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() != 4 {
            return Err("layout.bad");
        }
        let f = c.feature(parts[0]).ok_or("layout.bad")?;
        let known = (0..c.file.pieces.len()).any(|p| self.known[p] && spell::raises(c, p as u16) == Some(f));
        if !known {
            return Err("layout.unknown");
        }
        let x: i32 = parts[1].parse().map_err(|_| "layout.bad")?;
        let z: i32 = parts[2].parse().map_err(|_| "layout.bad")?;
        // one cell at a time: a wall is laid by dragging along it (T23)
        Ok(Placed {
            feature: f as u16,
            cells: vec![(x, z)],
        })
    }

    fn at(&self, s: Station) -> Result<(), &'static str> {
        if self.station() == Some(s) {
            Ok(())
        } else {
            Err("prep.not_here")
        }
    }

    fn spend(&mut self, mana: i32) -> Result<(), &'static str> {
        if self.mana < mana * 100 {
            return Err("prep.no_mana");
        }
        self.mana -= mana * 100;
        Ok(())
    }

    /// Starts drawing a spell's or a helper's glyphs at the podium.
    fn begin_drawing(&mut self, c: &Content, why: Draw, audience: i32) {
        let glyphs: Vec<Vec<String>> = match &why {
            Draw::Teach { spell, .. } => self.spells[*spell as usize]
                .iter()
                .map(|&p| c.file.pieces[p as usize].glyph.clone())
                .collect(),
            Draw::Summon { piece, .. } => vec![c.file.pieces[*piece as usize].glyph.clone()],
        };
        let guide = glyph::incantation(&glyphs);
        let band = glyph::band(&c.balance().band_px, audience, units_per_px(glyphs.len()));
        self.activity = Activity::Drawing {
            why,
            guide,
            drawn: Vec::new(),
            band,
            audience,
            glyphs: glyphs.len(),
        };
    }

    pub fn act(&mut self, c: &Content, name: &str, value: &str) -> Result<(), &'static str> {
        let b = c.balance();
        match name {
            "library.read" => {
                self.at(Station::Library)?;
                if self.busy() {
                    return Err("prep.busy");
                }
                if self
                    .pile
                    .iter()
                    .all(|bk| matches!(bk, Book::Piece(p) if self.known[*p as usize]))
                {
                    return Err("library.empty");
                }
                self.activity = Activity::Reading {
                    left: content::ticks(b.read_ms),
                };
            }
            "lab.add" => {
                self.at(Station::Lab)?;
                let p = c.piece(value).ok_or("lab.unknown")?;
                if !self.known[p] {
                    return Err("lab.unknown");
                }
                if self.draft.len() >= b.max_chain as usize {
                    return Err(spell::ChainError::TooLong.key());
                }
                self.draft.push(p as u16);
            }
            "lab.remove" => {
                self.at(Station::Lab)?;
                let i: usize = value.parse().map_err(|_| "lab.bad")?;
                if i < self.draft.len() {
                    self.draft.remove(i);
                }
            }
            "lab.clear" => self.draft.clear(),
            "lab.save" => {
                self.at(Station::Lab)?;
                spell::check(c, &self.draft).map_err(|e| e.key())?;
                if self.spells.contains(&self.draft) {
                    return Err("lab.already");
                }
                if self.spells.len() >= MAX_SPELLS {
                    return Err("lab.full");
                }
                self.record.spells_made += 1;
                self.record.longest_chain = self.record.longest_chain.max(self.draft.len() as i32);
                self.spells.push(std::mem::take(&mut self.draft));
                self.chosen = (self.spells.len() - 1) as u16;
                self.note("lab.saved", spell::name(c, &self.spells[self.chosen as usize]));
            }
            "spell.choose" => {
                let i: u16 = value.parse().map_err(|_| "lab.bad")?;
                if (i as usize) < self.spells.len() {
                    self.chosen = i;
                }
            }
            "recruit.one" | "recruit.group" => {
                self.at(Station::Recruit)?;
                if self.busy() {
                    return Err("prep.busy");
                }
                if self.applicants.is_empty() {
                    return Err("recruit.none");
                }
                let group = name == "recruit.group";
                let one = if group {
                    None
                } else {
                    Some(value.parse::<usize>().unwrap_or(0).min(self.applicants.len() - 1))
                };
                let lane = Lane::new(&mut self.rng, group, b.perfect_ticks, b.good_ticks);
                self.activity = Activity::Interview {
                    lane,
                    one,
                    by_helper: false,
                };
            }
            "podium.teach" => {
                self.at(Station::Podium)?;
                if self.busy() {
                    return Err("prep.busy");
                }
                if self.students.is_empty() {
                    return Err("podium.no_students");
                }
                let spell = self.chosen.min(self.spells.len().saturating_sub(1) as u16);
                if self.spells.is_empty() {
                    return Err("podium.no_spell");
                }
                let one: Option<u16> = if value.is_empty() || value == "all" {
                    None
                } else {
                    value.parse().ok()
                };
                let audience = if one.is_some() {
                    1
                } else {
                    self.students.len() as i32
                };
                self.begin_drawing(c, Draw::Teach { spell, one }, audience);
            }
            "podium.summon" => {
                // value: the helper piece id
                self.at(Station::Podium).or_else(|_| self.at(Station::Recruit))?;
                if self.busy() {
                    return Err("prep.busy");
                }
                let piece = c.piece(value).ok_or("helper.unknown")?;
                if !self.known[piece] {
                    return Err("helper.unknown");
                }
                let PieceKind::Helper { helper } = &c.file.pieces[piece].kind else {
                    return Err("helper.unknown");
                };
                let h = c.helper(helper).ok_or("helper.unknown")?;
                let work = match helper.as_str() {
                    "teaching_elemental" => {
                        if self.spells.is_empty() {
                            return Err("podium.no_spell");
                        }
                        Work::Teach(self.chosen)
                    }
                    "recruitment_elemental" => Work::Recruit,
                    _ => Work::Train,
                };
                self.spend(c.file.helpers[h].mana)?;
                self.begin_drawing(
                    c,
                    Draw::Summon {
                        helper: h as u16,
                        piece: piece as u16,
                        work,
                    },
                    1,
                );
            }
            "draw.point" => {
                // value: "x,y,w,h" in CSS pixels on the overlay
                let Activity::Drawing { drawn, glyphs, .. } = &mut self.activity else {
                    return Err("draw.none");
                };
                let n = crate::strings::ints(value);
                if n.len() < 4 || n[2] <= 0 || n[3] <= 0 {
                    return Err("draw.bad");
                }
                let (ox, oy) = draw_origin(*glyphs);
                let u = units_per_px(*glyphs);
                let (lx, ly) = (n[0] * OVER_W / n[2], n[1] * OVER_H / n[3]);
                drawn.push(((lx - ox) * u, (ly - oy) * u));
            }
            "draw.end" => {
                let Activity::Drawing {
                    why,
                    guide,
                    drawn,
                    band,
                    ..
                } = std::mem::replace(&mut self.activity, Activity::Idle)
                else {
                    return Err("draw.none");
                };
                let score = glyph::score(&guide, &drawn, band);
                match why {
                    Draw::Teach { spell, one } => self.teach(c, spell, one, score),
                    Draw::Summon { helper, work, .. } => {
                        let total = content::ticks(c.file.helpers[helper as usize].job_s * 1000);
                        self.jobs.push(Job {
                            helper,
                            score,
                            left: total,
                            total,
                            work,
                        });
                        self.record.summoned += 1;
                        self.record.summoned_kinds |= 1 << helper.min(31);
                        self.note("helper.summoned", score.to_string());
                    }
                }
                self.note("draw.score", score.to_string());
            }
            "layout.place" => {
                // costs its mana now and rises raise_s later, while the
                // archmage does other things (T1)
                self.at(Station::Map)?;
                let p = self.placement(c, value)?;
                self.can_place(c, &p)?;
                let f = &c.file.features[p.feature as usize];
                self.spend(f.mana)?;
                self.laid.push(p.cells[0]);
                let total = content::ticks(f.raise_ms.max(0)).max(1);
                let golem = c.helper("builder_golem").unwrap_or(0) as u16;
                self.jobs.push(Job {
                    helper: golem,
                    score: 100,
                    left: total,
                    total,
                    work: Work::Raise(p),
                });
            }
            "layout.golem" => {
                self.at(Station::Map)?;
                let p = self.placement(c, value)?;
                let f = &c.file.features[p.feature as usize];
                if f.golem_s == 0 {
                    return Err("layout.no_golem");
                }
                if self.jobs.iter().any(|j| matches!(j.work, Work::Build(_))) {
                    return Err("layout.golem_busy");
                }
                self.can_place(c, &p)?;
                let golem = c.helper("builder_golem").unwrap_or(0) as u16;
                let total = content::ticks(f.golem_s * 1000);
                self.jobs.push(Job {
                    helper: golem,
                    score: 100,
                    left: total,
                    total,
                    work: Work::Build(p),
                });
            }
            "layout.remove" => {
                self.at(Station::Map)?;
                let i: usize = value.parse().map_err(|_| "layout.bad")?;
                if i < self.placed.len() {
                    self.placed.remove(i);
                }
            }
            "layout.erase" => {
                // value "x,z": rubs out what stands or rises in that cell, and
                // gives its mana back (T23)
                self.at(Station::Map)?;
                let n = crate::strings::ints(value);
                if n.len() != 2 {
                    return Err("layout.bad");
                }
                self.erase(c, (n[0], n[1]));
            }
            "layout.clear" => {
                self.at(Station::Map)?;
                for cell in std::mem::take(&mut self.laid) {
                    self.erase(c, cell);
                }
            }
            "layout.undo" => {
                self.at(Station::Map)?;
                if let Some(cell) = self.laid.pop() {
                    self.erase(c, cell);
                }
            }
            "book.cast" => {
                // value: the spell; an invasion goes to every rival (T29)
                self.at(Station::Book)?;
                let mut n: Vec<u16> = value.split(',').filter_map(|s| s.parse().ok()).collect();
                if n.is_empty() || n.len() > 2 {
                    return Err("book.bad");
                }
                n.truncate(1);
                n.push(EVERY_RIVAL as u16);
                let chain = self.spells.get(n[0] as usize).ok_or("book.bad")?;
                if !spell::is_invasion(c, chain) {
                    return Err("book.not_invasion");
                }
                let cost = spell::stats(c, chain).mana * b.invasion_mana_pct / 100;
                self.spend(cost)?;
                self.sent.push((n[1] as u8, n[0]));
                self.record.invasions += 1;
            }
            "day.ready" => self.ready = true,
            "prep.cancel" => {
                if self.busy() {
                    self.activity = Activity::Idle;
                    self.note("prep.cancelled", "");
                }
            }
            _ => return Err("prep.unknown"),
        }
        Ok(())
    }

    /// A student of a set quality with no school leanings, for trials.
    pub fn add_plain_student(&mut self, c: &Content, quality: i32) {
        let a = self.applicant(c);
        self.enrol(c, &a, quality);
        if let Some(s) = self.students.last_mut() {
            s.mods.iter_mut().for_each(|m| *m = 0);
            s.variance = 0;
        }
        self.record.recruited -= 1;
    }

    /// A bot's interview, at a score its skill drew.
    pub fn bot_hire(&mut self, c: &Content, score: i32, one: Option<usize>) {
        self.hire(c, score, one);
    }

    /// A bot's lesson to the whole class, at a score its skill drew.
    pub fn bot_teach(&mut self, c: &Content, spell: u16, score: i32) {
        if !self.students.is_empty() {
            self.teach(c, spell, None, score);
        }
    }

    /// What this archmage brings to the battle.
    pub fn plan(&self) -> Plan {
        Plan {
            spells: self.spells.clone(),
            students: self
                .students
                .iter()
                .map(|s| StudentPlan {
                    mana_max: s.mana_max,
                    regen: s.regen,
                    spells: s.known.clone(),
                    school: s.school,
                })
                .collect(),
            placed: self.placed.clone(),
        }
    }

    /// Invasions this archmage's students send when the battle starts:
    /// (chain, effectiveness), one per student who knows one.
    pub fn student_invasions(&self, c: &Content) -> Vec<(Vec<u16>, i32)> {
        let mut out = Vec::new();
        for s in &self.students {
            for &(spell, eff) in &s.known {
                if let Some(chain) = self.spells.get(spell as usize) {
                    if spell::is_invasion(c, chain) {
                        out.push((chain.clone(), eff));
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests;

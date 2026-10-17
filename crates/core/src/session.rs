//! The game as the page sees it: screens, the one door for every action
//! (`act`), fixed ticks from display frames, and what to draw and play
//! each frame. The page builds its panels from `ui()` without knowing what
//! any of them means (after Peter's Mandible).

use serde_json::{json, Value};

use crate::arena;
use crate::battle::{self, Battle};
use crate::bot::{Style, STYLES};
use crate::content::{Content, PieceKind};
use crate::game::{self, Match, Mode, Phase, SeatKind, Settings};
use crate::geom::{self, v3, TPS};
use crate::glossary;
use crate::glyph;
use crate::look::Look;
use crate::online::{self, Event, Msg, Online, Role};
use crate::overlay;
use crate::prep::{self, Activity, Draw, Station};
use crate::render::{Batch, Camera, FrameOut, StaticScene};
use crate::room;
use crate::save::Save;
use crate::scene;
use crate::sound::{self, Recipe};
use crate::spell;
use crate::strings::{self, Strings};
use crate::ticks::Clock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    Setup,
    Trials,
    Achievements,
    Help,
    Online,
    News,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    All,
    Seat(u8),
}

#[derive(Debug, Clone, Default)]
pub struct Layout {
    pub open: bool,
    pub feature: Option<usize>,
    pub along_z: bool,
    pub hover: Option<(i32, i32)>,
    pub golem: bool,
    just_closed: bool,
    /// While the pointer is down on the map: whether it rubs out, and the
    /// last cell it touched (T23).
    drag: Option<bool>,
    last: Option<(i32, i32)>,
    /// Seating students on the map instead of building (T33).
    pub seats: bool,
}

pub struct Session {
    pub c: Content,
    look: Look,
    s: Strings,
    hints: Vec<(String, String)>,
    pub screen: Screen,
    pub settings: Settings,
    pub rivals: [Style; 3],
    pub m: Option<Match>,
    clock: Clock,
    pub tick: i32,
    pub save: Save,
    save_dirty: bool,
    seed: u64,
    matches: u64,
    pub view: View,
    pub close: bool,
    pub layout: Layout,
    pointer_down: bool,
    recipes: Vec<Recipe>,
    sounds: Vec<u8>,
    sound_cool: Vec<i32>,
    toasts: Vec<(String, i32)>,
    statics: StaticScene,
    static_key: (i32, i32, i32),
    frame: FrameOut,
    canvas: (i32, i32),
    lane_hits: usize,
    trial_result: Option<(u8, i32, i32, bool)>,
    earned_this_match: Vec<String>,
    results_seen: bool,
    /// Online play (PLAN.md decision 15): the room, once hosted or joined.
    pub online: Option<Online>,
    online_name: String,
    online_code: String,
    /// The room was opened with pasted codes, so it has no room code to show.
    pasted: bool,
    /// The glossary's open tab, while it is open (T8).
    pub glossary: Option<String>,
    /// The tower behind the menus (T11, T47), and how many have been made.
    demo: Option<Match>,
    demos: u64,
    /// The title's archmage: which station of `room::TOUR` they are
    /// walking to, and the ticks left to stand there once arrived (T47).
    tour: (usize, i32),
    /// What's new (T12): each update's notes, newest first.
    news: News,
    all_news: bool,
    /// The student office hours has open (T15).
    office_pick: Option<u16>,
    /// The builder's open tab, and the tile being dragged (T32).
    lab_tab: String,
    held: Option<usize>,
    /// The student picked up at the invasion book (T40), office hours'
    /// open tab (T41), and which of your students the battle's tree shows.
    raid_held: Option<u16>,
    office_tab: String,
    watch: usize,
    /// The difficulty chosen for each trial (T30).
    trial_diff: Vec<i32>,
    /// The card for the book just read, and the ticks it stays (T10).
    pub card: Option<(Value, i32)>,
    /// Each battle view's own camera (T5), and the drag moving one.
    pub cams: [scene::Orbit; 4],
    drag: Option<Drag>,
    build: String,
    /// Messages held for a later day or phase.
    net_queue: Vec<Event>,
    sums: Vec<(i32, u8, u64)>,
    desync_told: bool,
    /// Whether the rehearsal runs (T48); the tests that do not look at it
    /// turn it off to keep their time.
    pub rehearse: bool,
    pub rehearsal: Rehearsal,
}

/// Today's battle played quietly against the plan as it stands (T48): what
/// it was played for, the battle while it runs, the score it took when it
/// finished, and the one before the last change.
#[derive(Default)]
pub struct Rehearsal {
    pub key: u64,
    pub battle: Option<Box<Battle>>,
    pub result: Option<i32>,
    pub before: Option<i32>,
}

/// What's new, as data/news.json holds it (T12).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct News {
    pub entries: Vec<NewsEntry>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NewsEntry {
    pub version: String,
    pub date: String,
    pub minor: bool,
    pub items: Vec<String>,
}

/// An SVG path and view box drawing glyphs side by side, in glyph units,
/// for a card's icon (T10).
fn icon(glyphs: &[Vec<String>]) -> (String, String) {
    let pts = glyph::incantation(glyphs);
    let mut d = String::new();
    for (k, (x, y)) in pts.iter().enumerate() {
        d.push_str(&format!("{}{x} {y} ", if k == 0 { "M" } else { "L" }));
    }
    let w = glyph::width(glyphs.len().max(1));
    let pad = glyph::BOX / 8;
    (
        d.trim_end().to_string(),
        format!("{} {} {} {}", -pad, -pad, w + 2 * pad, glyph::BOX + 2 * pad),
    )
}

/// A drag in a battle view: which seat's camera, where the pointer was, and
/// whether it moves the view rather than turning it.
#[derive(Debug, Clone, Copy)]
struct Drag {
    seat: usize,
    x: i32,
    y: i32,
    /// The view's height in CSS pixels.
    h: i32,
    pan: bool,
}

/// A mana cost the page draws as a mana symbol with its number (T13).
fn mana(s: &Strings, n: i32) -> Value {
    json!({"n": n.to_string(), "label": s.f("lab.spell_mana", &[("mana", n.to_string())])})
}

fn act_json(name: &str, value: &str, label: String, enabled: bool) -> Value {
    json!({"name": name, "value": value, "label": label, "enabled": enabled})
}

/// A name's initials, for a seat's disc: the first letter of its first two words.
/// Moves a panel's instruction paragraphs (blocks of tone "help") onto
/// its title as hover text, so the panel shows what to do, not how (T49).
fn tuck_help(panel: &mut Value) {
    let Some(bl) = panel["blocks"].as_array_mut() else {
        return;
    };
    let help: Vec<String> = bl
        .iter()
        .filter(|b| b["tone"] == "help")
        .filter_map(|b| b["text"].as_str().map(str::to_string))
        .collect();
    if help.is_empty() {
        return;
    }
    bl.retain(|b| b["tone"] != "help");
    let text = help.join(" ");
    match bl.iter_mut().find(|b| b["t"] == "title" || b["t"] == "h") {
        Some(t) => t["help"] = json!(text),
        None => panel["help"] = json!(text),
    }
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .flat_map(|ch| ch.to_uppercase())
        .collect()
}

fn row(label: String, detail: String) -> Value {
    json!({"label": label, "detail": detail})
}

impl Session {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        strings_json: &str,
        content_json: &str,
        palette_json: &str,
        models_json: &str,
        sounds_json: &str,
        glossary_md: &str,
        news_json: &str,
        save_text: &str,
        seed: u64,
    ) -> Result<Session, String> {
        let build = online::build_of(&[content_json, strings_json]);
        let s = Strings::parse(strings_json)?;
        let c = Content::parse(content_json).map_err(|e| e.message())?;
        let look = Look::parse(palette_json, models_json)?;
        let recipes = sound::parse(sounds_json)?;
        let news: News = serde_json::from_str(news_json).map_err(|e| format!("news.json: {e}"))?;
        if news.entries.is_empty() {
            return Err("news.json: no entries".into());
        }
        let entries = glossary::parse(glossary_md).map_err(|e| e.message())?;
        let b = c.balance();
        let fill = [
            ("battle.length_s", b.battle_s.to_string()),
            ("battle.max_min", (b.battle_max_s / 60).to_string()),
            ("golem.hold_pct", b.golem_hold_pct.to_string()),
            ("students.max", b.max_students.to_string()),
            ("match.days", b.days.to_string()),
            ("effectiveness.min", b.effect_min_pct.to_string()),
            ("effectiveness.max", b.effect_max_pct.to_string()),
            ("day.prep_s", b.prep_s.to_string()),
            ("schools.count", c.file.schools.len().to_string()),
            ("score.start", strings::number(b.score_start)),
        ];
        let hints = entries
            .into_iter()
            .map(|e| {
                let mut d = e.definition;
                for (k, v) in &fill {
                    d = d.replace(&format!("{{{k}}}"), v);
                }
                (e.term, d)
            })
            .collect();
        let (save, broken) = Save::read(save_text);
        let settings = save
            .settings
            .clone()
            .unwrap_or_else(|| Settings::from_content(&c));
        let mut session = Session {
            sound_cool: vec![0; recipes.len()],
            c,
            look,
            s,
            hints,
            screen: Screen::Title,
            settings,
            rivals: [Style::Spellcrafter, Style::AttackMage, Style::MassTeacher],
            m: None,
            clock: Clock::default(),
            tick: 0,
            save,
            save_dirty: false,
            seed,
            matches: 0,
            view: View::All,
            close: false,
            layout: Layout::default(),
            pointer_down: false,
            recipes,
            sounds: Vec::new(),
            toasts: Vec::new(),
            statics: StaticScene::default(),
            static_key: (-1, -1, -1),
            frame: FrameOut::default(),
            canvas: (1280, 720),
            lane_hits: 0,
            trial_result: None,
            earned_this_match: Vec::new(),
            results_seen: false,
            online: None,
            online_name: String::new(),
            online_code: String::new(),
            pasted: false,
            glossary: None,
            news,
            all_news: false,
            demo: None,
            demos: 0,
            tour: (0, 0),
            card: None,
            trial_diff: Vec::new(),
            office_pick: None,
            lab_tab: "pay".into(),
            held: None,
            raid_held: None,
            office_tab: "seats".into(),
            watch: 0,
            cams: [scene::Orbit::start(false); 4],
            drag: None,
            build,
            net_queue: Vec::new(),
            sums: Vec::new(),
            desync_told: false,
            rehearse: true,
            rehearsal: Rehearsal::default(),
        };
        session.online_name = session.s.t("online.default_name");
        session.trial_diff = vec![session.c.balance().default_difficulty; session.c.file.trials.len()];
        if broken {
            session.toast(session.s.t("save.broken"));
        }
        Ok(session)
    }

    fn toast(&mut self, text: String) {
        self.toasts.push((text, TPS * 4));
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    fn play(&mut self, id: &str) {
        if self.save.muted {
            return;
        }
        if let Some(i) = self.recipes.iter().position(|r| r.id == id) {
            if self.sound_cool[i] == 0 && self.sounds.len() < 6 {
                self.sounds.push(i as u8);
                self.sound_cool[i] = 4;
            }
        }
    }

    // ---- the one door ----

    pub fn act(&mut self, name: &str, value: &str) -> Result<(), String> {
        let r = self.act_inner(name, value);
        match &r {
            Err(e) => {
                self.play("error");
                let _ = e;
            }
            Ok(())
                if name.starts_with("screen.") || name.starts_with("setup.") || name.starts_with("view.") =>
            {
                self.play("click")
            }
            _ => {}
        }
        r
    }

    fn err(&self, key: &str) -> String {
        if self.s.has(key) {
            self.s.t(key)
        } else {
            self.s.f("error.unknown", &[("key", key.to_string())])
        }
    }

    fn act_inner(&mut self, name: &str, value: &str) -> Result<(), String> {
        match name {
            "screen.show" => {
                self.screen = match value {
                    "title" => Screen::Title,
                    "setup" => Screen::Setup,
                    "trials" => Screen::Trials,
                    "achievements" => Screen::Achievements,
                    "help" => Screen::Help,
                    "online" => Screen::Online,
                    "news" => {
                        // opening the notes marks this version as seen
                        self.save.news_seen = self.news.entries[0].version.clone();
                        self.save_dirty = true;
                        Screen::News
                    }
                    other => return Err(self.s.f("error.no_screen", &[("name", other.to_string())])),
                };
                if self.screen != Screen::Play {
                    self.m = None;
                }
            }
            "setup.difficulty" => {
                let n = self.c.balance().difficulties.len() as i32;
                let d = self.settings.difficulty + if value == "up" { 1 } else { -1 };
                self.settings.difficulty = d.clamp(0, n - 1);
                self.save.settings = Some(self.settings.clone());
                self.save_dirty = true;
            }
            "trial.difficulty" => {
                let k: usize = value.parse().map_err(|_| self.err("error.bad"))?;
                let n = self.c.balance().difficulties.len() as i32;
                if k >= self.trial_diff.len() {
                    return Err(self.err("error.bad"));
                }
                self.trial_diff[k] = (self.trial_diff[k] + 1) % n;
            }
            "setup.days" | "setup.prep" | "setup.battle" | "setup.speed" | "setup.cap" | "setup.skill" => {
                let up = value == "up";
                let st = &mut self.settings;
                let step = |v: i32, by: i32, lo: i32, hi: i32| (v + if up { by } else { -by }).clamp(lo, hi);
                match name {
                    "setup.days" => st.days = step(st.days, 1, 1, 8),
                    "setup.prep" => st.prep_s = step(st.prep_s, 30, 60, 480),
                    "setup.battle" => st.battle_s = step(st.battle_s, 15, 30, 180),
                    "setup.speed" => {
                        st.speed = if up {
                            (st.speed * 2).min(4)
                        } else {
                            (st.speed / 2).max(1)
                        }
                    }
                    "setup.cap" => st.ragdoll_cap = step(st.ragdoll_cap, 8, 0, 200),
                    _ => st.skill = step(st.skill, 5, 30, 95),
                }
                self.save.settings = Some(self.settings.clone());
                self.save_dirty = true;
            }
            "setup.rival" => {
                let i: usize = value.parse().map_err(|_| self.err("error.bad"))?;
                if i < 3 {
                    let k = STYLES.iter().position(|s| *s == self.rivals[i]).unwrap_or(0);
                    self.rivals[i] = STYLES[(k + 1) % STYLES.len()];
                }
            }
            "match.start" => self.start_match(value)?,
            "trial.start" => {
                let k: u8 = value.parse().map_err(|_| self.err("error.bad"))?;
                if k as usize >= self.c.file.trials.len() {
                    return Err(self.err("error.bad"));
                }
                let d = self
                    .trial_diff
                    .get(k as usize)
                    .copied()
                    .unwrap_or(self.c.balance().default_difficulty);
                self.m = Some(Match::trial_at(&self.c, k, d));
                self.begin_play();
            }
            // kept as typed (a name may end in a space while it is typed);
            // cleaned when the room opens
            "online.name" => self.online_name = value.chars().filter(|c| !c.is_control()).take(24).collect(),
            "online.code" => {
                self.online_code = value
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .map(|c| c.to_ascii_uppercase())
                    .take(8)
                    .collect()
            }
            "online.pasted" => self.pasted = true,
            "news.minor" => self.all_news = !self.all_news,
            "office.pick" => self.office_pick = value.parse().ok(),
            "lab.tab" => {
                if !["pay", "mod", "trig", "branch"].contains(&value) {
                    return Err(self.err("lab.unknown"));
                }
                self.lab_tab = value.to_string();
            }
            "lab.hold" => self.held = self.c.piece(value),
            // the pointer let go: a laboratory tile is no longer in hand, so a
            // later click on a slot opens its tab instead of dropping it (T57)
            "drag.end" => self.held = None,
            "raid.hold" => self.raid_held = value.parse().ok(),
            "raid.drop" => {
                // a student let go over a rival, or a rival clicked with a
                // student picked: the journey starts (T40, T45)
                let id = self.raid_held.take().ok_or_else(|| self.err("raid.pick_first"))?;
                self.forward("raid.send", &format!("{id}:{value}"))?;
            }
            "office.tab" => {
                if !["seats", "mind"].contains(&value) {
                    return Err(self.err("error.bad"));
                }
                self.office_tab = value.to_string();
            }
            "watch.next" => self.watch = self.watch.wrapping_add(1),
            "watch.prev" => self.watch = self.watch.wrapping_add(usize::MAX),
            "lab.drop" => {
                // an empty slot clicked with no tile in hand opens the tray's
                // tab of its kind (T57)
                let Some(k) = self.held.take() else {
                    return self.act_inner("lab.tab", value);
                };
                // a tile let go over an open slot: added if the slot takes its kind
                let kind = self.tile(k as u16)["kind"].as_str().unwrap_or("").to_string();
                if kind != value && kind != "branch" {
                    return Err(self.err("builder.wrong_slot"));
                }
                let id = self.c.file.pieces[k].id.clone();
                self.forward("lab.add", &id)?;
            }
            "glossary.toggle" => {
                self.glossary = match self.glossary {
                    Some(_) => None,
                    None => Some("pieces".into()),
                }
            }
            "glossary.close" => self.glossary = None,
            "glossary.tab" => {
                if !["pieces", "spells", "students", "terms", "how"].contains(&value) {
                    return Err(self.err("error.bad"));
                }
                self.glossary = Some(value.to_string());
            }
            "online.ready" => {
                let o = self.online.as_mut().ok_or_else(|| self.s.t("online.no_room"))?;
                let r = !o.ready;
                o.set_ready(r);
            }
            "online.start" => {
                self.matches += 1;
                let seed = self.seed ^ self.matches.wrapping_mul(0x9e37_79b9_7f4a_7c15);
                let bots = self.rivals.map(|r| r.key().to_string());
                let st = self.settings.clone();
                let o = self.online.as_mut().ok_or_else(|| self.s.t("online.no_room"))?;
                if !o.start(seed, st, &bots) {
                    return Err(self.s.t("online.not_everyone_ready"));
                }
            }
            "online.leave" => self.leave_online(),
            "match.leave" => {
                if self.m.as_ref().map(|m| m.mode == Mode::Online).unwrap_or(false) {
                    self.leave_online();
                }
                self.m = None;
                self.screen = Screen::Title;
                self.layout = Layout::default();
            }
            "day.next" => {
                if let Some(m) = self.m.as_mut() {
                    m.next_day(&self.c);
                    if m.phase == Phase::Prep {
                        self.play("day_start");
                    }
                }
                self.view = View::All;
            }
            "view.focus" => {
                self.view = match value {
                    "all" => View::All,
                    v => View::Seat(v.parse().map_err(|_| self.err("error.bad"))?),
                };
            }
            "view.close" => {
                self.close = !self.close;
                self.cams = [scene::Orbit::start(self.close); 4];
            }
            "cam.reset" => self.cams = [scene::Orbit::start(self.close); 4],
            "cam.down" | "cam.move" | "cam.up" | "cam.wheel" => self.cam(name, value)?,
            "battle.speed" => {
                let v: i32 = value.parse().map_err(|_| self.err("error.bad"))?;
                self.settings.speed = v.clamp(1, 4);
                if let Some(m) = self.m.as_mut() {
                    m.settings.speed = self.settings.speed;
                }
            }
            "sound.mute" => {
                self.save.muted = !self.save.muted;
                self.save_dirty = true;
            }
            "layout.open" => {
                self.prep_at(Station::Map)?;
                self.layout.open = true;
                if self.layout.feature.is_none() {
                    self.layout.feature = self.placeable().first().copied();
                }
            }
            "layout.close" => self.layout.open = false,
            "layout.feature" => {
                let f = self.c.feature(value).ok_or_else(|| self.err("layout.bad"))?;
                self.layout.feature = Some(f);
            }
            "layout.turn" => self.layout.along_z = !self.layout.along_z,
            "layout.mode" => self.layout.golem = value == "golem",
            "layout.seats" => self.layout.seats = value == "on",
            "pointer.down" | "pointer.move" | "pointer.up" => self.pointer(name, value)?,
            _ => {
                // everything else belongs to the local archmage's preparation
                let c = &self.c;
                let m = self
                    .m
                    .as_mut()
                    .ok_or_else(|| format!("Unknown action \"{name}\"."))?;
                if m.phase != Phase::Prep {
                    return Err(self.s.t("prep.over"));
                }
                if let Mode::Trial(3) = m.mode {
                    if name.starts_with("recruit.") {
                        return Err(self.s.t("trial.no_recruiting"));
                    }
                }
                let p = m
                    .local_prep_mut()
                    .ok_or_else(|| format!("Unknown action \"{name}\"."))?;
                p.act(c, name, value).map_err(|k| self.err(k))?;
                if name == "library.read" || name == "draw.end" {
                    self.layout.open = false;
                }
            }
        }
        Ok(())
    }

    /// The seat whose battle view holds a pointer at CSS (x, y) on a
    /// canvas of CSS size (w, h), and that view's height in canvas pixels.
    fn view_at(&self, x: i32, y: i32, w: i32, h: i32) -> Option<(usize, i32)> {
        let m = self
            .m
            .as_ref()
            .filter(|m| m.battles.iter().any(|b| b.is_some()))?;
        let (cw, ch) = self.canvas;
        let (px, py) = (x * cw / w.max(1), ch - y * ch / h.max(1));
        let shown = self.shown(m);
        self.viewports(shown.len())
            .into_iter()
            .zip(shown)
            .find(|(v, _)| px >= v.0 && px < v.0 + v.2 && py >= v.1 && py < v.1 + v.3)
            .map(|(v, seat)| (seat, v.3 * h.max(1) / ch.max(1)))
    }

    /// The battle views' mouse (T5): drag turns a view, a right drag or a
    /// Shift drag moves it, the wheel zooms. Each view keeps its own camera.
    /// value: "x,y,w,h,buttons,shift" in CSS pixels; for the wheel,
    /// "x,y,w,h,delta".
    fn cam(&mut self, name: &str, value: &str) -> Result<(), String> {
        let n = strings::ints(value);
        if n.len() < 5 {
            return Err(self.err("error.bad"));
        }
        let (x, y, w, h) = (n[0], n[1], n[2], n[3]);
        match name {
            "cam.down" => {
                self.drag = self.view_at(x, y, w, h).map(|(seat, vh)| Drag {
                    seat,
                    x,
                    y,
                    h: vh,
                    pan: n[4] & 2 != 0 || n[4] & 4 != 0 || n.get(5) == Some(&1),
                });
            }
            "cam.move" => {
                let Some(d) = self.drag else { return Ok(()) };
                let o = &mut self.cams[d.seat.min(3)];
                if d.pan {
                    o.pan(x - d.x, y - d.y, d.h);
                } else {
                    o.orbit(x - d.x, y - d.y, d.h);
                }
                self.drag = Some(Drag { x, y, ..d });
            }
            "cam.up" => self.drag = None,
            _ => {
                if let Some((seat, _)) = self.view_at(x, y, w, h) {
                    self.cams[seat.min(3)].zoom(n[4]);
                }
            }
        }
        Ok(())
    }

    fn prep_at(&self, s: Station) -> Result<(), String> {
        let p = self
            .m
            .as_ref()
            .and_then(|m| m.local_prep())
            .ok_or_else(|| self.s.t("prep.over"))?;
        if p.station() != Some(s) {
            return Err(self.s.t("prep.not_here"));
        }
        Ok(())
    }

    /// Features the local archmage knows how to raise.
    fn placeable(&self) -> Vec<usize> {
        let Some(p) = self.m.as_ref().and_then(|m| m.local_prep()) else {
            return Vec::new();
        };
        let mut out: Vec<usize> = (0..self.c.file.pieces.len())
            .filter(|&i| p.known[i])
            .filter_map(|i| spell::raises(&self.c, i as u16))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    fn pointer(&mut self, name: &str, value: &str) -> Result<(), String> {
        let n = strings::ints(value);
        if n.len() < 4 {
            return Err(self.err("draw.bad"));
        }
        let drawing = self
            .m
            .as_ref()
            .and_then(|m| m.local_prep())
            .map(|p| matches!(p.activity, Activity::Drawing { .. }))
            .unwrap_or(false);
        if drawing {
            match name {
                "pointer.down" => {
                    self.pointer_down = true;
                    self.forward("draw.point", value)?;
                }
                "pointer.move" if self.pointer_down => self.forward("draw.point", value)?,
                "pointer.up" if self.pointer_down => {
                    self.pointer_down = false;
                    self.forward("draw.point", value)?;
                    self.forward("draw.end", "")?;
                    self.play("draw_done");
                }
                _ => {}
            }
            return Ok(());
        }
        if self.layout.open && self.layout.seats {
            // seating (T33): press a student's disc to pick them up, let go
            // over a seat to put the one picked there
            let (lx, ly) = overlay::logical(n[0], n[1], n[2], n[3]);
            let Some(p) = self.m.as_ref().and_then(|m| m.local_prep()) else {
                return Ok(());
            };
            let Some(hit) = overlay::spot_at(&self.seat_spots(p), lx, ly).cloned() else {
                return Ok(());
            };
            match name {
                "pointer.down" if hit.who.is_some() => self.office_pick = hit.who,
                "pointer.up" => {
                    let id = self.office_pick.ok_or_else(|| self.err("map.pick_student"))?;
                    if hit.who != Some(id) {
                        self.forward("office.seat", &format!("{id},{}", hit.seat))?;
                        self.play("place");
                    }
                }
                _ => {}
            }
            return Ok(());
        }
        if self.layout.open {
            let (lx, ly) = overlay::logical(n[0], n[1], n[2], n[3]);
            self.layout.hover = overlay::map_cell(lx, ly);
            // drag to lay a cell at a time; a right button rubs out (T23).
            // value's fifth number is the pointer's buttons, when sent.
            match name {
                "pointer.down" => {
                    let erase = n.get(4).map(|b| b & 2 != 0).unwrap_or(false);
                    self.layout.drag = Some(erase);
                    self.layout.last = None;
                }
                "pointer.up" => {
                    self.layout.drag = None;
                    self.layout.last = None;
                    return Ok(());
                }
                _ => {}
            }
            let (Some(erase), Some((x, z))) = (self.layout.drag, self.layout.hover) else {
                return Ok(());
            };
            if self.layout.last == Some((x, z)) {
                return Ok(());
            }
            self.layout.last = Some((x, z));
            if erase {
                self.forward("layout.erase", &format!("{x},{z}"))?;
                return Ok(());
            }
            if let Some(f) = self.layout.feature {
                let id = self.c.file.features[f].id.clone();
                let v = format!("{id},{x},{z},{}", if self.layout.along_z { 1 } else { 0 });
                let act = if self.layout.golem {
                    "layout.golem"
                } else {
                    "layout.place"
                };
                // along a drag a refused cell is skipped, not an error
                if self.forward(act, &v).is_ok() {
                    self.play("place");
                } else if name == "pointer.down" {
                    self.forward(act, &v)?;
                }
            }
        }
        Ok(())
    }

    fn forward(&mut self, name: &str, value: &str) -> Result<(), String> {
        let c = &self.c;
        let m = self.m.as_mut().ok_or_else(|| self.s.t("prep.over"))?;
        let p = m.local_prep_mut().ok_or_else(|| self.s.t("prep.over"))?;
        p.act(c, name, value)
            .map_err(|k| if self.s.has(k) { self.s.t(k) } else { k.to_string() })
    }

    fn start_match(&mut self, value: &str) -> Result<(), String> {
        self.matches += 1;
        let seed = self.seed ^ self.matches.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        let you = self.s.t("seat.you");
        let mut st = self.settings.clone();
        let (mode, kinds, names) = match value {
            "practice" => {
                st.days = 1;
                st.prep_s = self.c.balance().practice_prep_s;
                (
                    Mode::Practice,
                    [SeatKind::Local, SeatKind::Empty, SeatKind::Empty, SeatKind::Empty],
                    [you, String::new(), String::new(), String::new()],
                )
            }
            "versus" => {
                let r = self.rivals;
                let name = |s: Style| self.s.t(s.key());
                (
                    Mode::Versus,
                    [
                        SeatKind::Local,
                        SeatKind::Bot(r[0]),
                        SeatKind::Bot(r[1]),
                        SeatKind::Bot(r[2]),
                    ],
                    [you, name(r[0]), name(r[1]), name(r[2])],
                )
            }
            other => return Err(self.s.f("error.no_mode", &[("name", other.to_string())])),
        };
        self.m = Some(Match::new(&self.c, seed, mode, st, kinds, names, 0));
        self.begin_play();
        Ok(())
    }

    // ---- online ----

    fn leave_online(&mut self) {
        self.online = None;
        self.pasted = false;
        self.net_queue.clear();
        self.sums.clear();
        self.screen = Screen::Title;
        self.m = None;
    }

    /// Opens a room as host from two random words the page drew; returns
    /// the room's name on the relays.
    fn settle_name(&mut self) {
        self.online_name = online::clean_name(&self.online_name);
        if self.online_name.is_empty() {
            self.online_name = self.s.t("online.default_name");
        }
    }

    pub fn host(&mut self, a: u32, b: u32, app: &str) -> String {
        self.settle_name();
        let code = online::make_code(a, b);
        let room = online::room_name(app, &code);
        self.online = Some(Online::host(&self.online_name, &self.build, code));
        self.pasted = false;
        self.screen = Screen::Online;
        room
    }

    /// Joins by a typed code (empty for pasted codes); returns the room's
    /// name, or nothing when the code cannot be one.
    pub fn join(&mut self, code: &str, app: &str) -> String {
        let clean = online::clean_code(code);
        if clean.is_none() && !code.is_empty() {
            return String::new();
        }
        let code = clean.unwrap_or_default();
        self.settle_name();
        self.pasted = code.is_empty();
        let room = online::room_name(app, &code);
        self.online = Some(Online::join(&self.online_name, &self.build, code));
        self.screen = Screen::Online;
        room
    }

    pub fn net_peer(&mut self, peer: u32) {
        if let Some(o) = self.online.as_mut() {
            o.connected(peer);
        }
    }

    pub fn net_left(&mut self, peer: u32) {
        if let Some(o) = self.online.as_mut() {
            o.left(peer);
        }
    }

    pub fn net_message(&mut self, peer: u32, text: &str) {
        if let Some(o) = self.online.as_mut() {
            o.message(peer, text);
        }
    }

    pub fn net_error(&mut self, key: &str) {
        let key = if self.s.has(key) && key.starts_with("online.") {
            key
        } else {
            "online.garbled"
        };
        if let Some(o) = self.online.as_mut() {
            o.status = key.to_string();
        }
    }

    /// What to send, as JSON `[[peer, text, false], …]` (nothing goes on the
    /// fast channel).
    pub fn net_outbox(&mut self) -> String {
        let out: Vec<Value> = self
            .online
            .as_mut()
            .map(|o| o.take_outbox())
            .unwrap_or_default()
            .into_iter()
            .map(|(p, t)| json!([p, t, false]))
            .collect();
        json!(out).to_string()
    }

    pub fn build(&self) -> String {
        self.build.clone()
    }

    pub fn net_active(&self) -> bool {
        self.online.is_some()
    }

    pub fn net_status(&self) -> String {
        self.online.as_ref().map(|o| o.status.clone()).unwrap_or_default()
    }

    /// Takes what the room heard, and acts on what is due: a start now, an
    /// invasion or a plan in its own day's preparation, a checksum whenever.
    fn pump_online(&mut self) {
        let Some(o) = self.online.as_mut() else { return };
        let local_is_host = o.role == Role::Host;
        let seat = o.seat;
        let mut queue = std::mem::take(&mut self.net_queue);
        queue.append(&mut o.events);
        let mut keep = Vec::new();
        for e in queue {
            if let Event::Start {
                seed,
                settings,
                kinds,
                names,
            } = e
            {
                self.start_online(seed, settings, &kinds, &names, seat);
                continue;
            }
            let c = &self.c;
            let Some(m) = self.m.as_mut().filter(|m| m.mode == Mode::Online) else {
                continue;
            };
            let open = matches!(m.phase, Phase::Prep | Phase::Waiting);
            let due = |day: i32| {
                if day < m.day {
                    Some(false)
                } else if day == m.day && open {
                    Some(true)
                } else {
                    None
                }
            };
            match e {
                Event::Invade {
                    day,
                    from,
                    target,
                    chain,
                    ..
                } => match due(day) {
                    Some(true) if online::sane_chain(c, &chain) => m.take_invasion(c, from, target, chain),
                    None => keep.push(Event::Invade {
                        day,
                        from,
                        seq: 0,
                        target,
                        chain,
                    }),
                    _ => {}
                },
                Event::Plan {
                    day,
                    seat: s,
                    plan,
                    students,
                } => match due(day) {
                    Some(true) if online::sane_plan(c, &plan, &students) => m.take_plan(s, plan, students),
                    Some(true) => m.depart(s),
                    None => keep.push(Event::Plan {
                        day,
                        seat: s,
                        plan,
                        students,
                    }),
                    _ => {}
                },
                Event::Sum { day, seat: s, hash } => self.sums.push((day, s, hash)),
                Event::Left { seat: s } => {
                    if s == 0 && !local_is_host {
                        // the host carried every message: everyone remote is gone
                        for k in 0..4 {
                            m.depart(k);
                        }
                    } else {
                        m.depart(s);
                    }
                }
                Event::Start { .. } => {}
            }
        }
        self.net_queue = keep;
        self.check_sums();
    }

    fn start_online(&mut self, seed: u64, settings: Settings, kinds: &[String], names: &[String], local: u8) {
        let you = self.online_name.clone();
        let mut k = [SeatKind::Empty; 4];
        let mut n: [String; 4] = Default::default();
        for i in 0..4 {
            let kind = kinds.get(i).map(String::as_str).unwrap_or("");
            let style = STYLES.iter().copied().find(|s| s.key() == kind);
            (k[i], n[i]) = match (i as u8 == local, kind, style) {
                (true, _, _) => (SeatKind::Local, you.clone()),
                (false, "person", _) => (SeatKind::Remote, names.get(i).cloned().unwrap_or_default()),
                (false, _, Some(st)) => (SeatKind::Bot(st), self.s.t(st.key())),
                _ => (SeatKind::Empty, String::new()),
            };
        }
        self.sums.clear();
        self.desync_told = false;
        self.m = Some(Match::new(
            &self.c,
            seed,
            Mode::Online,
            settings,
            k,
            n,
            local.min(3),
        ));
        self.begin_play();
    }

    /// The local archmage's half of the match: each invasion as it is
    /// cast, the plan when the preparation ends, a checksum after a battle.
    fn send_online(&mut self, cast: Vec<(u8, Vec<u16>)>, before: Phase) {
        let Some(m) = self.m.as_ref().filter(|m| m.mode == Mode::Online) else {
            return;
        };
        let Some(o) = self.online.as_mut() else { return };
        let (day, local) = (m.day, m.local);
        for (target, chain) in cast {
            o.seq += 1;
            let seq = o.seq;
            o.say(Msg::Invade {
                day,
                from: local,
                seq,
                target,
                chain,
            });
        }
        if before == Phase::Prep && m.phase != Phase::Prep {
            let (plan, students) = m.local_plan(&self.c);
            o.say(Msg::Plan {
                day,
                seat: local,
                plan,
                students,
            });
        }
        if before == Phase::Battle && m.phase != Phase::Battle {
            let hash = m.checksum();
            o.say(Msg::Sum {
                day,
                seat: local,
                hash,
            });
            self.sums.push((day, local, hash));
            self.check_sums();
        }
    }

    /// Every browser ran every battle; a checksum that differs from this
    /// browser's for the same day means the games have parted.
    fn check_sums(&mut self) {
        let Some(m) = self.m.as_ref() else { return };
        let local = m.local;
        let differs = self.sums.iter().any(|&(d, s, h)| {
            s != local
                && self
                    .sums
                    .iter()
                    .any(|&(d2, s2, h2)| s2 == local && d2 == d && h2 != h)
        });
        if differs && !self.desync_told {
            self.desync_told = true;
            let t = self.s.t("online.desync");
            self.toast(t);
        }
    }

    // ---- the title's tower (T11, T47) ----

    /// Behind every menu the camera turns slowly in the middle of the
    /// tower, and an archmage walks from station to station with their
    /// class on the benches (T11, T47).
    /// Plays a little more of the rehearsal (T48), starting it again when
    /// the plan, the book or the day has changed since it began.
    fn rehearse_step(&mut self) {
        let Some(m) = self.m.as_ref() else { return };
        if m.phase != Phase::Prep {
            self.rehearsal = Rehearsal::default();
            return;
        }
        let Some(p) = m.local_prep() else { return };
        let shown = m.book().len();
        let key = {
            // FNV-1a over what the battle reads from the plan and the book
            let text = format!("{}|{}|{:?}", m.day, shown, p.plan());
            text.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
                (h ^ b as u64).wrapping_mul(0x100_0000_01b3)
            })
        };
        let r = &mut self.rehearsal;
        if key != r.key {
            r.key = key;
            if r.result.is_some() {
                r.before = r.result;
            }
            r.battle = m.rehearsal(&self.c).map(Box::new);
        }
        if let Some(b) = r.battle.as_mut() {
            for _ in 0..self.c.balance().rehearsal_ticks {
                if b.done() {
                    break;
                }
                b.step(&self.c);
            }
            if b.done() {
                r.result = Some(b.tally.damage_taken);
                r.battle = None;
            }
        }
    }

    fn step_demo(&mut self) {
        if self.demo.is_none() {
            self.demos += 1;
            let seed = self.seed ^ self.demos.wrapping_mul(0x2545_f491_4f6c_dd1d);
            let kinds = [SeatKind::Local, SeatKind::Empty, SeatKind::Empty, SeatKind::Empty];
            let st = Settings::from_content(&self.c);
            let m = Match::new(&self.c, seed, Mode::Versus, st, kinds, Default::default(), 0);
            self.demo = Some(m);
            self.tour = (0, 0);
            self.static_key = (-1, -1, -1);
        }
        let walk = crate::content::cms(self.c.balance().walk_cms);
        let stay = self.c.balance().demo_stay_s * TPS;
        let (mut next, mut left) = self.tour;
        if let Some(p) = self.demo.as_mut().and_then(|m| m.local_prep_mut()) {
            p.prev = p.pos;
            let st = room::TOUR[next % room::TOUR.len()];
            let to = room::stand_at(st).sub(p.pos).flat();
            if left > 0 {
                // standing at the station, facing it
                left -= 1;
                let face = prep::station_pos(st).sub(p.pos);
                p.yaw = crate::geom::yaw_of(face.x, face.z);
                if left == 0 {
                    next = (next + 1) % room::TOUR.len();
                }
            } else if to.flat_len() <= walk {
                p.pos = p.pos.add(to);
                left = stay;
            } else {
                p.pos = p.pos.add(to.with_len(walk));
                p.yaw = crate::geom::yaw_of(to.x, to.z);
            }
        }
        self.tour = (next, left);
    }

    /// The title's camera now: one turn in `demo_turn_s` (T47).
    fn title_view(&self) -> Camera {
        let turn_ticks = (self.c.balance().demo_turn_s.max(1) * TPS) as i64;
        let turn = (self.tick as i64 % turn_ticks) * crate::fx::TURN as i64 / turn_ticks;
        room::title_camera(turn as i32)
    }

    fn begin_play(&mut self) {
        self.screen = Screen::Play;
        self.demo = None;
        self.cams = [scene::Orbit::start(self.close); 4];
        self.drag = None;
        self.view = View::All;
        self.layout = Layout::default();
        self.trial_result = None;
        self.earned_this_match.clear();
        self.results_seen = false;
        self.static_key = (-1, -1, -1);
        self.play("day_start");
    }

    // ---- time ----

    pub fn resize(&mut self, css_w: i32, css_h: i32, dpr_100: i32) -> Vec<i32> {
        let w = (css_w.max(1) * dpr_100.clamp(50, 300) / 100).min(2560);
        let h = (css_h.max(1) * dpr_100.clamp(50, 300) / 100).min(1440);
        self.canvas = (w, h);
        vec![w, h]
    }

    pub fn frame(&mut self, now_us: i64, bits: u32) {
        self.sounds.clear();
        for t in self.clock.frame(now_us, bits) {
            self.step(t);
        }
        // once a drawn frame, not once a tick: a slow machine that runs
        // several ticks a frame to catch up must not also rehearse several
        // times over, or it falls further behind (T48)
        if self.rehearse {
            self.rehearse_step();
        }
        self.build_frame();
    }

    fn step(&mut self, bits: u32) {
        self.tick += 1;
        for c in self.sound_cool.iter_mut() {
            *c = (*c - 1).max(0);
        }
        for t in self.toasts.iter_mut() {
            t.1 -= 1;
        }
        self.toasts.retain(|t| t.1 > 0);
        if let Some(card) = self.card.as_mut() {
            card.1 -= 1;
        }
        self.card = self.card.take().filter(|c| c.1 > 0);
        self.pump_online();
        if self.glossary_pauses() {
            return;
        }
        if self.m.is_none() {
            self.step_demo();
            return;
        }
        // the map table: E opens the map, Esc closes it (T3)
        let at_map = self
            .m
            .as_ref()
            .filter(|m| m.phase == Phase::Prep)
            .and_then(|m| m.local_prep())
            .map(|p| p.station() == Some(Station::Map) && !p.busy())
            .unwrap_or(false);
        if self.layout.open && (bits & prep::BACK != 0 || !at_map) {
            self.layout.open = false;
        } else if !self.layout.open && at_map && bits & prep::USE != 0 && !self.layout.just_closed {
            let _ = self.act_inner("layout.open", "");
        }
        // a held E does not open the map again the tick after Esc closes it
        self.layout.just_closed = bits & prep::USE != 0 && !self.layout.open;
        let Some(m) = self.m.as_mut() else { return };
        let before = m.phase;
        let bits = if self.layout.open {
            bits & !(prep::UP | prep::DOWN | prep::LEFT | prep::RIGHT | prep::USE)
        } else {
            bits
        };
        let cast = m.step(&self.c, bits);
        let after = m.phase;
        self.send_online(cast, before);
        let Some(m) = self.m.as_mut() else { return };
        // the archmage's notes become toasts and sounds
        let mut notes = Vec::new();
        let mut lane_hits = None;
        let mut last_read = None;
        if let Some(p) = m.local_prep_mut() {
            notes = std::mem::take(&mut p.notes);
            last_read = p.last_read.clone();
            if let Activity::Interview { lane, .. } = &p.activity {
                lane_hits = Some(
                    lane.notes
                        .iter()
                        .filter(|n| n.got.map(|g| g > 0).unwrap_or(false))
                        .count(),
                );
            }
        }
        if let Some(h) = lane_hits {
            if h > self.lane_hits {
                self.play("note_hit");
            }
            self.lane_hits = h;
        } else {
            self.lane_hits = 0;
        }
        for n in notes {
            let sound = match n.key.as_str() {
                "library.learned" => "learn",
                "library.tome" => "whole_spell",
                "lab.saved" => "save_spell",
                "recruit.joined" if n.value != "0" => "interview_win",
                "recruit.joined" => "interview_fail",
                "helper.summoned" => "summon",
                "helper.built" => "golem_done",
                _ => "",
            };
            if !sound.is_empty() {
                self.play(sound);
            }
            // a book read shows its card for a while, or until the next one (T10)
            let card = match (n.key.as_str(), &last_read) {
                ("library.learned", Some(prep::Book::Piece(k))) => {
                    Some(self.piece_card(*k as usize, self.s.t("card.learned")))
                }
                ("library.tome", Some(prep::Book::Tome(t))) => Some(self.tome_card(*t as usize)),
                _ => None,
            };
            if let Some(card) = card {
                self.card = Some((card, self.c.balance().card_s.max(1) * TPS));
            } else if n.key != "draw.score" {
                let text = self.s.f(&n.key, &[("value", n.value.clone())]);
                self.toast(text);
            }
        }
        // battle sounds from the battle in view
        if after == Phase::Battle {
            if before != Phase::Battle {
                self.play("battle_start");
            }
            let seat = match self.view {
                View::Seat(k) => k,
                View::All => self.m.as_ref().map(|m| m.local).unwrap_or(0),
            };
            let evs: Vec<crate::battle::Ev> = self
                .m
                .as_ref()
                .and_then(|m| m.battles.get(seat as usize))
                .and_then(|b| b.as_ref())
                .map(|b| b.events.iter().map(|e| e.0).collect())
                .unwrap_or_default();
            for e in evs {
                use crate::battle::Ev;
                let id = match e {
                    Ev::Cast(k) => [
                        "cast_destruction",
                        "cast_alteration",
                        "cast_conjuration",
                        "cast_illusion",
                        "cast_restoration",
                    ]
                    .get(k as usize)
                    .copied()
                    .unwrap_or("cast_destruction"),
                    Ev::Hit => "hit",
                    Ev::Strong => "strong",
                    Ev::Weak => "weak",
                    Ev::Kill => "kill",
                    Ev::Boom => "boom",
                    Ev::TowerHit => "tower_hit",
                    Ev::Freeze => "freeze",
                    Ev::Charm => "charm",
                    Ev::Summon => "summon_battle",
                    Ev::WallUp => "wall_up",
                    Ev::WallDown => "wall_down",
                };
                self.play(id);
            }
        }
        if before != after {
            match after {
                Phase::Summary => self.play("summary"),
                Phase::Results => self.finish(),
                _ => {}
            }
        }
        if self.tick % 30 == 0 || before != after {
            self.check_achievements();
        }
    }

    fn finish(&mut self) {
        let Some(m) = self.m.as_ref() else { return };
        if let Mode::Trial(k) = m.mode {
            let value = m.trial_done.unwrap_or(0);
            let id = self.best_key(k as usize, m.settings.difficulty);
            let before = self.save.bests.get(&id).copied();
            let new_best = game::better(&self.c, k as usize, value, before);
            if new_best {
                self.save.bests.insert(id, value);
                self.save_dirty = true;
            }
            let medal = game::medal(&self.c, k as usize, value);
            self.trial_result = Some((k, value, medal, new_best));
            self.play(if medal > 0 { "medal" } else { "summary" });
        } else {
            let won = m.won(m.local as usize);
            self.play(if won { "victory" } else { "defeat" });
        }
    }

    fn earn(&mut self, id: &str) {
        if self.save.earn(id) {
            self.save_dirty = true;
            self.earned_this_match.push(id.to_string());
            let name = self
                .c
                .file
                .achievements
                .iter()
                .find(|a| a.id == id)
                .map(|a| a.name.clone())
                .unwrap_or_default();
            self.toast(self.s.f("achievement.earned", &[("name", name)]));
            self.play("achievement");
        }
    }

    fn check_achievements(&mut self) {
        let Some(m) = self.m.as_ref() else { return };
        let mut ids: Vec<&str> = Vec::new();
        if let Some(p) = m.local_prep() {
            let r = &p.record;
            let books = self.c.file.pieces.len() - 2 + self.c.file.tomes.len();
            if r.spells_made >= 1 {
                ids.push("first_spell");
            }
            if r.longest_chain >= 6 {
                ids.push("long_chain");
            }
            if r.best_teach >= 95 {
                ids.push("perfect_lesson");
            }
            if r.biggest_class >= 20 {
                ids.push("big_class");
            }
            if r.perfect_interview {
                ids.push("perfect_interview");
            }
            if p.students.len() >= 30 {
                ids.push("full_house");
            }
            if r.books as usize >= books {
                ids.push("bookworm");
            }
            if r.golem_built {
                ids.push("golem_work");
            }
            if r.summoned_kinds.count_ones() >= 3 {
                ids.push("helper_all");
            }
        }
        let local = m.local as usize;
        if let Some(Some(b)) = m.battles.get(local) {
            if b.tally.frozen >= 25 {
                ids.push("freezer");
            }
            if b.tally.giants_charmed > 0 {
                ids.push("charm_giant");
            }
            if b.done() && b.tally.damage_taken == 0 && b.tally.kills > 0 {
                ids.push("untouched");
            }
        }
        let dealt: i32 = m
            .battles
            .iter()
            .flatten()
            .map(|b| b.tally.taken_from[local.min(3)])
            .max()
            .unwrap_or(0);
        if dealt >= 500 {
            ids.push("invader");
        }
        if m.phase == Phase::Results && m.mode == Mode::Versus && m.won(local) {
            ids.push("win_versus");
            if !m.seats[local].recruited_any {
                ids.push("no_recruits_win");
            }
        }
        let golds = self.c.file.trials.iter().enumerate().all(|(k, t)| {
            self.save
                .bests
                .get(&t.id)
                .map(|&v| game::medal(&self.c, k, v) >= 3)
                .unwrap_or(false)
        });
        if golds {
            ids.push("gold_trials");
        }
        let ids: Vec<String> = ids.into_iter().map(String::from).collect();
        for id in ids {
            self.earn(&id);
        }
    }

    /// `act` as the page sees it: `{"ok": true}` or `{"error": "..."}`.
    pub fn act_json(&mut self, name: &str, value: &str) -> String {
        match self.act(name, value) {
            Ok(()) => json!({"ok": true}).to_string(),
            Err(e) => json!({"error": e}).to_string(),
        }
    }

    pub fn take_save(&mut self) -> String {
        if !self.save_dirty {
            return String::new();
        }
        self.save_dirty = false;
        self.save.text()
    }

    pub fn sounds(&self) -> Vec<u8> {
        self.sounds.clone()
    }

    pub fn sound_count(&self) -> u32 {
        self.recipes.len() as u32
    }

    pub fn sound_wav(&self, i: u32) -> Vec<u8> {
        self.recipes.get(i as usize).map(sound::wav).unwrap_or_default()
    }

    // ---- what to draw ----

    pub fn frame_out(&self) -> &FrameOut {
        &self.frame
    }

    pub fn statics(&self) -> &StaticScene {
        &self.statics
    }

    fn viewports(&self, n: usize) -> Vec<(i32, i32, i32, i32)> {
        let (w, h) = self.canvas;
        if n <= 1 {
            return vec![(0, 0, w, h)];
        }
        let (hw, hh) = (w / 2, h / 2);
        // drawing-buffer y runs from the bottom: the top row first
        vec![
            (0, hh, hw, h - hh),
            (hw, hh, w - hw, h - hh),
            (0, 0, hw, hh),
            (hw, 0, w - hw, hh),
        ]
    }

    /// The seats whose battles are drawn, in viewport order.
    fn shown(&self, m: &Match) -> Vec<usize> {
        match self.view {
            View::Seat(k) => vec![k as usize],
            View::All => {
                let active: Vec<usize> = (0..m.seats.len()).filter(|&i| m.battles[i].is_some()).collect();
                if active.len() <= 1 {
                    active
                } else {
                    (0..4).collect()
                }
            }
        }
    }

    fn build_frame(&mut self) {
        let mut f = FrameOut::default();
        // a match, or behind the menus the title's tower
        let Some(m) = self.m.as_ref().or(self.demo.as_ref()) else {
            self.frame = f;
            return;
        };
        match m.phase {
            Phase::Prep | Phase::Waiting => {
                let key = (0, m.day, 0);
                if self.static_key != key {
                    self.statics = StaticScene::default();
                    let (room, near) = scene::room_static(&self.look);
                    self.statics.add(&room);
                    self.statics.add(&near);
                    self.statics.version += 1 + self.tick as u32;
                    self.static_key = key;
                }
                if let Some(p) = m.local_prep() {
                    // behind the menus the camera turns in the middle of
                    // the room (T47)
                    let cam = if self.m.is_none() {
                        self.title_view()
                    } else {
                        scene::room_camera(p)
                    };
                    let (x, y, w, h) = (0, 0, self.canvas.0, self.canvas.1);
                    f.viewport(x, y, w, h, &cam, &scene::room_mood(&self.look));
                    f.static_draws(&self.statics.parts[0]);
                    if scene::near_wall_shown(&cam) {
                        f.static_draws(&self.statics.parts[1]);
                    }
                    let mut b = Batch::new();
                    scene::room_dynamic(&mut b, &self.look, &self.c, p, self.tick);
                    f.batch(&b);
                }
            }
            Phase::Battle | Phase::Summary | Phase::Results => {
                let key = (1, m.day, m.battles.iter().filter(|b| b.is_some()).count() as i32);
                if self.static_key != key {
                    self.statics = StaticScene::default();
                    for b in m.battles.iter() {
                        match b {
                            Some(bt) => {
                                let seats = bt.casters.len();
                                let s = scene::arena_static(&self.look, &self.c, &bt.arena, Some(bt), seats);
                                self.statics.add(&s);
                            }
                            None => {
                                self.statics.add(&Batch::new());
                            }
                        }
                    }
                    self.statics.version += 1 + self.tick as u32;
                    self.static_key = key;
                }
                let shown = self.shown(m);
                let far = shown.len() > 1;
                for (vp, &seat) in self.viewports(shown.len()).into_iter().zip(shown.iter()) {
                    let cam = self.cams[seat.min(3)].camera();
                    f.viewport(vp.0, vp.1, vp.2, vp.3, &cam, &scene::field_mood(&self.look));
                    if let Some(Some(bt)) = m.battles.get(seat) {
                        if let Some(part) = self.statics.parts.get(seat) {
                            f.static_draws(part);
                        }
                        let mut b = Batch::new();
                        scene::arena_dynamic(&mut b, &self.look, &self.c, bt, far);
                        f.batch(&b);
                    }
                }
            }
        }
        self.frame = f;
    }

    /// The 2D overlay, or nothing when no mini-game or editor is open.
    pub fn overlay(&self) -> Vec<i32> {
        let Some(m) = self.m.as_ref() else {
            return Vec::new();
        };
        if m.phase != Phase::Prep {
            return Vec::new();
        }
        let Some(p) = m.local_prep() else {
            return Vec::new();
        };
        match &p.activity {
            Activity::Drawing {
                why,
                guide,
                drawn,
                band,
                ..
            } => {
                let pieces: Vec<u16> = match why {
                    Draw::Teach { spell, .. } => p.spells.get(*spell as usize).cloned().unwrap_or_default(),
                    Draw::Summon { piece, .. } => vec![*piece],
                };
                let names: Vec<String> = pieces
                    .iter()
                    .map(|&k| self.c.file.pieces[k as usize].name.clone())
                    .collect();
                let heading = match why {
                    Draw::Teach { spell, one } => {
                        let name = spell::name(&self.c, &p.spells[*spell as usize]);
                        let who = match one {
                            None => self
                                .s
                                .f("podium.to_class", &[("n", p.students.len().to_string())]),
                            Some(id) => p
                                .students
                                .iter()
                                .find(|s| s.id == *id)
                                .map(|s| s.name.clone())
                                .unwrap_or_default(),
                        };
                        self.s.f("podium.drawing", &[("spell", name), ("who", who)])
                    }
                    Draw::Summon { helper, .. } => self.s.f(
                        "podium.summoning",
                        &[("name", self.c.file.helpers[*helper as usize].name.clone())],
                    ),
                };
                let live = crate::glyph::score(guide, drawn, *band);
                overlay::drawing(&p.activity, &names, &heading, &format!("{live}%"))
            }
            Activity::Interview { lane, one, .. } => {
                let who = match one {
                    Some(i) => p.applicants.get(*i).map(|a| a.name.clone()).unwrap_or_default(),
                    None => self.s.t("recruit.group_heading"),
                };
                let keys = [
                    self.s.t("key.note0"),
                    self.s.t("key.note1"),
                    self.s.t("key.note2"),
                    self.s.t("key.note3"),
                ];
                overlay::interview(
                    lane,
                    &self.s.f("recruit.interviewing", &[("who", who)]),
                    &keys,
                    &format!("{}%", lane.score()),
                )
            }
            Activity::Journey {
                x, y, falls, student, ..
            } => {
                let school = p
                    .students
                    .iter()
                    .find(|st| st.id == *student)
                    .map(|st| st.school)
                    .unwrap_or(0);
                overlay::journey(
                    *x * 100 / crate::fx::ONE,
                    *y * 100 / crate::fx::ONE,
                    *falls,
                    school,
                )
            }
            Activity::Sealing { rings, at } => overlay::seal(&self.c, &p.draft, rings, *at),
            _ if self.layout.open => {
                let a = p.field(&self.c);
                let ghost = match (self.layout.hover, self.layout.feature) {
                    (Some((x, z)), Some(f)) if !self.layout.seats => {
                        let cells = vec![(x, z)];
                        let placed = crate::battle::Placed {
                            feature: f as u16,
                            cells,
                        };
                        let ok = p
                            .clone()
                            .act(
                                &self.c,
                                if self.layout.golem {
                                    "layout.golem"
                                } else {
                                    "layout.place"
                                },
                                &format!(
                                    "{},{x},{z},{}",
                                    self.c.file.features[f].id,
                                    if self.layout.along_z { 1 } else { 0 }
                                ),
                            )
                            .is_ok();
                        Some((placed, ok))
                    }
                    _ => None,
                };
                let rising: Vec<crate::battle::Placed> = p.rising().cloned().collect();
                overlay::layout(
                    &self.c,
                    &a,
                    &p.placed,
                    &rising,
                    ghost.as_ref().map(|(g, ok)| (g, *ok)),
                    &if self.layout.seats {
                        self.seat_spots(p)
                    } else {
                        Vec::new()
                    },
                )
            }
            _ => Vec::new(),
        }
    }

    // ---- the view model ----

    pub fn ui(&self) -> Value {
        let hints: Vec<Value> = self
            .hints
            .iter()
            .map(|(t, d)| json!({"term": t, "definition": d}))
            .collect();
        let toasts: Vec<Value> = self.toasts.iter().map(|t| json!(t.0)).collect();
        let mut out = match self.screen {
            Screen::Title => self.ui_title(),
            Screen::Setup => self.ui_setup(),
            Screen::Trials => self.ui_trials(),
            Screen::Achievements => self.ui_achievements(),
            Screen::Help => self.ui_help(),
            Screen::Online => self.ui_online(),
            Screen::News => self.ui_news(),
            Screen::Play => self.ui_play(),
        };
        if let Some(tab) = &self.glossary {
            let g = self.ui_glossary(tab);
            if let Some(panels) = out["panels"].as_array_mut() {
                panels.push(g);
            }
            out["glossary"] = json!(true);
        }
        out["glossary_label"] = json!(self.s.t("gloss.button"));
        out["card"] = self.card.as_ref().map(|c| c.0.clone()).unwrap_or(Value::Null);
        out["hints"] = json!(hints);
        out["toasts"] = json!(toasts);
        out["muted"] = json!(self.save.muted);
        out["mute_label"] = json!(self.s.t(if self.save.muted {
            "sound.unmute"
        } else {
            "sound.mute"
        }));
        out
    }

    fn base(&self, screen: &str, title: String, panels: Vec<Value>) -> Value {
        json!({"screen": screen, "title": title, "panels": panels, "labels": [], "phase": "", "station": null, "busy": null, "overlay": false})
    }

    fn ui_title(&self) -> Value {
        let s = &self.s;
        let actions = vec![
            act_json("screen.show", "setup", s.t("title.versus"), true),
            act_json("match.start", "practice", s.t("title.practice"), true),
            act_json("screen.show", "online", s.t("title.online"), true),
            act_json("screen.show", "trials", s.t("title.trials"), true),
            act_json("screen.show", "achievements", s.t("title.achievements"), true),
            act_json("screen.show", "help", s.t("title.help"), true),
        ];
        // after Ape Race's: the name over a live battle, the menu, the
        // controls, and what is new (T11, T12)
        let (version, unseen) = self.version();
        let mut actions = actions;
        let mut news = act_json("screen.show", "news", s.t("news.button"), true);
        if unseen {
            news["badge"] = json!(s.t("news.new"));
        }
        actions.push(news);
        let controls: Vec<Value> = [
            "walk", "use", "back", "notes", "glossary", "turn", "camera", "mute",
        ]
        .iter()
        .map(|k| row(s.t(&format!("controls.{k}")), s.t(&format!("controls.{k}.keys"))))
        .collect();
        let logo =
            json!({"id": "logo", "kind": "logo", "heading": s.t("game.name"), "text": s.t("title.tagline")});
        let menu = json!({"id": "title", "kind": "menu", "actions": actions});
        let keys = json!({"id": "controls", "kind": "controls", "heading": s.t("controls.heading"), "rows": controls});
        let corner = json!({"id": "version", "kind": "version", "actions": [act_json("screen.show", "news", s.f("title.version", &[("version", version.to_string())]), true)]});
        self.base("title", s.t("game.name"), vec![logo, menu, keys, corner])
    }

    fn ui_setup(&self) -> Value {
        let s = &self.s;
        let st = &self.settings;
        let setting = |key: &str, label: &str, value: String| json!({"label": s.t(label), "detail": value, "actions": [act_json(key, "down", s.t("setup.less"), true), act_json(key, "up", s.t("setup.more"), true)]});
        let rows = vec![
            setting("setup.days", "setup.days", st.days.to_string()),
            setting("setup.prep", "setup.prep", strings::clock(st.prep_s * TPS)),
            setting("setup.battle", "setup.battle", strings::clock(st.battle_s * TPS)),
            setting("setup.speed", "setup.speed", format!("{}×", st.speed)),
            setting("setup.cap", "setup.cap", st.ragdoll_cap.to_string()),
            setting("setup.skill", "setup.skill", st.skill.to_string()),
            setting(
                "setup.difficulty",
                "setup.difficulty",
                s.t(&format!("difficulty.{}", self.c.difficulty(st.difficulty).id)),
            ),
        ];
        let rivals: Vec<Value> = (0..3)
            .map(|i| json!({"label": s.f("setup.rival", &[("n", (i + 1).to_string())]), "detail": s.t(self.rivals[i].key()), "actions": [act_json("setup.rival", &i.to_string(), s.t("setup.change"), true)]}))
            .collect();
        let p = json!({"id": "setup", "kind": "center", "heading": s.t("setup.heading"), "text": s.t("setup.text"), "rows": rows});
        let r = json!({"id": "rivals", "kind": "center", "heading": s.t("setup.rivals"), "rows": rivals, "actions": [act_json("match.start", "versus", s.t("setup.start"), true), act_json("screen.show", "title", s.t("back"), true)]});
        self.base("setup", s.t("setup.heading"), vec![p, r])
    }

    fn ui_online(&self) -> Value {
        let s = &self.s;
        let Some(o) = self.online.as_ref() else {
            let inputs = vec![
                json!({"id": "online-name", "label": s.t("online.name"), "value": self.online_name, "action": "online.name"}),
                json!({"id": "online-code", "label": s.t("online.code"), "value": self.online_code, "action": "online.code"}),
            ];
            let actions = vec![
                act_json("page.host", "", s.t("online.host"), true),
                act_json(
                    "page.join",
                    &self.online_code,
                    s.t("online.join"),
                    online::clean_code(&self.online_code).is_some(),
                ),
                act_json("page.invite", "", s.t("online.invite"), true),
                act_json("page.reply", "", s.t("online.reply"), true),
                act_json("screen.show", "title", s.t("back"), true),
            ];
            let p = json!({"id": "online", "kind": "center", "heading": s.t("online.heading"), "text": s.t("online.text"), "inputs": inputs, "actions": actions});
            return self.base("online", s.t("online.heading"), vec![p]);
        };
        let rows: Vec<Value> = o
            .lobby
            .iter()
            .enumerate()
            .map(|(k, seat)| {
                // open seats take the computer rivals in order, as start() fills them
                let bot = o.lobby[..k].iter().filter(|x| x.kind == "open").count();
                let name = match seat.kind.as_str() {
                    "open" => s.t(self.rivals[bot.min(2)].key()),
                    _ => seat.name.clone(),
                };
                let you = if k as u8 == o.seat { s.t("online.you") } else { String::new() };
                let chip = match seat.kind.as_str() {
                    "person" if seat.ready => s.t("online.ready_chip"),
                    "person" => s.t("online.not_ready_chip"),
                    _ => String::new(),
                };
                json!({"label": format!("{name}{you}"), "detail": s.t(&format!("online.seat_{}", seat.kind)), "chip": chip})
            })
            .collect();
        let code = if self.pasted || o.room.is_empty() {
            s.t("online.by_paste")
        } else {
            s.f("online.code_is", &[("code", o.room.clone())])
        };
        let mut actions = Vec::new();
        if !self.pasted && !o.room.is_empty() {
            actions.push(act_json("page.copy", &o.room, s.t("online.copy_code"), true));
            // a link that joins this room when opened (T14)
            actions.push(act_json("page.copy_link", &o.room, s.t("online.copy_link"), true));
        }
        if self.pasted && o.role == Role::Host {
            actions.push(act_json("page.invite", "", s.t("online.invite_more"), true));
        }
        match o.role {
            Role::Host => actions.push(act_json("online.start", "", s.t("online.start"), o.can_start())),
            Role::Joiner => actions.push(act_json(
                "online.ready",
                "",
                s.t(if o.ready { "online.unready" } else { "online.ready" }),
                o.seat < 4,
            )),
        }
        actions.push(act_json("online.leave", "", s.t("online.leave"), true));
        let st = &self.settings;
        let text = format!(
            "{} {}",
            code,
            s.f(
                "online.settings",
                &[
                    ("days", st.days.to_string()),
                    ("prep", strings::clock(st.prep_s * TPS)),
                    ("battle", strings::clock(st.battle_s * TPS))
                ]
            )
        );
        let p = json!({"id": "lobby", "kind": "center", "heading": s.t("online.lobby"), "text": text, "rows": rows, "actions": actions});
        let status = json!({"id": "net-status", "kind": "bottom", "text": s.t(&o.status)});
        let mut v = self.base("online", s.t("online.lobby"), vec![p, status]);
        v["data"] = json!({"net": o.status, "seat": o.seat.to_string()});
        v
    }

    /// The latest version, and whether this player has not opened its notes.
    pub fn version(&self) -> (&str, bool) {
        let v = self.news.entries[0].version.as_str();
        (v, self.save.news_seen != v)
    }

    fn ui_news(&self) -> Value {
        let s = &self.s;
        let rows: Vec<Value> = self
            .news
            .entries
            .iter()
            .filter(|e| self.all_news || !e.minor)
            .map(|e| json!({"label": s.f("news.version", &[("version", e.version.clone()), ("date", e.date.clone())]), "list": e.items}))
            .collect();
        let toggle = s.t(if self.all_news {
            "news.hide_minor"
        } else {
            "news.show_minor"
        });
        let p = json!({"id": "news", "kind": "center", "heading": s.t("news.heading"), "rows": rows, "actions": [act_json("news.minor", "", toggle, true), act_json("screen.show", "title", s.t("back"), true)]});
        self.base("news", s.t("news.heading"), vec![p])
    }

    /// Where a trial's best at a difficulty is kept: the trial's id at the
    /// default difficulty (as saves before T30 kept it), else id@difficulty.
    fn best_key(&self, k: usize, difficulty: i32) -> String {
        let id = &self.c.file.trials[k].id;
        if difficulty == self.c.balance().default_difficulty {
            id.clone()
        } else {
            format!("{id}@{}", self.c.difficulty(difficulty).id)
        }
    }

    fn ui_trials(&self) -> Value {
        let s = &self.s;
        let medals = ["medal.bronze", "medal.silver", "medal.gold", "medal.archmage"];
        let mut panels = Vec::new();
        for (k, t) in self.c.file.trials.iter().enumerate() {
            let d = self
                .trial_diff
                .get(k)
                .copied()
                .unwrap_or(self.c.balance().default_difficulty);
            let best = self.save.bests.get(&self.best_key(k, d)).copied();
            let earned = best.map(|v| game::medal(&self.c, k, v)).unwrap_or(0);
            let unit = |v: i32| {
                if t.measure == "time" {
                    s.f("trial.seconds", &[("n", v.to_string())])
                } else {
                    strings::number(v)
                }
            };
            let rows: Vec<Value> = medals
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    // the top medal stays hidden until the one below is earned (Ape Race)
                    let shown = i < 3 || earned >= 3;
                    let target = if shown { unit(t.medals[i]) } else { s.t("trial.hidden") };
                    json!({"label": s.t(m), "detail": target, "chip": if earned > i as i32 { s.t("trial.got") } else { String::new() }})
                })
                .collect();
            let best_text = match best {
                Some(v) => s.f("trial.best", &[("value", unit(v))]),
                None => s.t("trial.no_best"),
            };
            panels.push(json!({"id": format!("trial-{k}"), "kind": "card", "heading": t.name, "text": format!("{} {}", t.text, best_text), "rows": rows, "actions": [act_json("trial.start", &k.to_string(), s.t("trial.start"), true), act_json("trial.difficulty", &k.to_string(), s.f("trial.difficulty", &[("name", s.t(&format!("difficulty.{}", self.c.difficulty(d).id)))]), true)]}));
        }
        panels.push(json!({"id": "trials-back", "kind": "bottom", "actions": [act_json("screen.show", "title", s.t("back"), true)]}));
        self.base("trials", s.t("title.trials"), panels)
    }

    fn ui_achievements(&self) -> Value {
        let s = &self.s;
        let rows: Vec<Value> = self
            .c
            .file
            .achievements
            .iter()
            .map(|a| {
                let got = self.save.achievements.contains(&a.id);
                json!({"label": a.name, "detail": a.text, "chip": if got { s.t("achievement.got") } else { s.t("achievement.not_yet") }})
            })
            .collect();
        let n = self.save.achievements.len();
        let p = json!({"id": "achievements", "kind": "center", "heading": s.t("title.achievements"), "text": s.f("achievement.count", &[("n", n.to_string()), ("of", self.c.file.achievements.len().to_string())]), "rows": rows, "actions": [act_json("screen.show", "title", s.t("back"), true)]});
        self.base("achievements", s.t("title.achievements"), vec![p])
    }

    /// The how-to topics, in order, with their numbers filled in.
    fn help_rows(&self) -> Vec<Value> {
        let s = &self.s;
        let b = self.c.balance();
        let fill = [
            ("score", strings::number(b.score_start)),
            ("pct", b.invasion_mana_pct.to_string()),
            ("students_max", b.max_students.to_string()),
            (
                "raise_max",
                self.c
                    .file
                    .features
                    .iter()
                    .map(|f| f.raise_ms / 1000)
                    .max()
                    .unwrap_or(0)
                    .to_string(),
            ),
        ];
        (1..)
            .take_while(|k| s.has(&format!("help.{k}.head")))
            .map(|k| {
                row(
                    s.t(&format!("help.{k}.head")),
                    s.f(&format!("help.{k}.text"), &fill),
                )
            })
            .collect()
    }

    /// The glossary (T8): every spell piece by number, the saved spells,
    /// every glossary word and the how-to topics, over any screen.
    fn ui_glossary(&self, tab: &str) -> Value {
        let s = &self.s;
        let c = &self.c;
        let prep = self.m.as_ref().and_then(|m| m.local_prep());
        let tabs: Vec<Value> = ["pieces", "spells", "students", "terms", "how"]
            .iter()
            .map(|t| act_json("glossary.tab", t, s.t(&format!("gloss.tab.{t}")), *t != tab))
            .collect();
        let secs = |ms: i32| format!("{}.{}", ms / 1000, ms % 1000 / 100);
        let (text, rows, cards): (String, Vec<Value>, Vec<Value>) = match tab {
            "spells" => {
                let rows: Vec<Value> = prep
                    .map(|p| {
                        p.spells
                            .iter()
                            .map(|ch| {
                                let st = spell::stats(c, ch);
                                let pieces: Vec<String> = ch
                                    .iter()
                                    .map(|&k| c.file.pieces[k as usize].name.clone())
                                    .collect();
                                let mut r = row(
                                    spell::name(c, ch),
                                    s.f(
                                        "gloss.spell_detail",
                                        &[
                                            ("pieces", pieces.join(" → ")),
                                            ("power", st.power.to_string()),
                                            ("cooldown", secs(st.cooldown_ms)),
                                            ("mana", st.mana.to_string()),
                                        ],
                                    ),
                                );
                                r["mana"] = mana(s, st.mana);
                                r
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let text = match (prep, rows.is_empty()) {
                    (None, _) => s.t("gloss.no_match"),
                    (Some(_), true) => s.t("gloss.no_spells"),
                    _ => s.t("gloss.spells_text"),
                };
                (text, rows, Vec::new())
            }
            "students" => {
                // every student's numbers, readable at any time (T16)
                let rows: Vec<Value> = prep
                    .map(|p| {
                        p.students
                            .iter()
                            .map(|st| {
                                let taught: Vec<String> = st
                                    .known
                                    .iter()
                                    .filter_map(|&(k, e)| p.spells.get(k as usize).map(|ch| s.f("gloss.taught", &[("spell", spell::name(c, ch)), ("n", e.to_string())])))
                                    .collect();
                                let leans: Vec<String> = c
                                    .file
                                    .schools
                                    .iter()
                                    .enumerate()
                                    .filter(|(k, _)| st.mods.get(*k).copied().unwrap_or(0) != 0)
                                    .map(|(k, sc)| {
                                        let m = st.mods[k];
                                        format!("{} {}{m}%", sc.name, if m > 0 { "+" } else { "" })
                                    })
                                    .collect();
                                json!({"label": st.name, "detail": s.f("gloss.student", &[("mana", st.mana_max.to_string()), ("regen", st.regen.to_string()), ("seat", self.seat_name(st.seat)), ("leans", leans.join(", ")), ("taught", if taught.is_empty() { s.t("gloss.untaught") } else { taught.join(", ") })]), "school": c.file.schools[st.school as usize].id})
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let text = match (prep, rows.is_empty()) {
                    (None, _) => s.t("gloss.no_match"),
                    (Some(_), true) => s.t("office.no_students"),
                    _ => s.t("gloss.students_text"),
                };
                (text, rows, Vec::new())
            }
            "terms" => (
                s.t("gloss.terms_text"),
                self.hints
                    .iter()
                    .map(|(t, d)| row(t.clone(), d.clone()))
                    .collect(),
                Vec::new(),
            ),
            "how" => (s.t("gloss.how_text"), self.help_rows(), Vec::new()),
            _ => {
                let cards = (0..c.file.pieces.len())
                    .map(|i| {
                        let chip = match prep {
                            Some(p) if p.known[i] => s.t("gloss.known"),
                            Some(_) => s.t("gloss.unknown"),
                            None => String::new(),
                        };
                        self.piece_card(i, chip)
                    })
                    .collect();
                (s.t("gloss.pieces_text"), Vec::new(), cards)
            }
        };
        let paused = self.glossary_pauses();
        let text = if paused {
            format!("{text} {}", s.t("gloss.paused"))
        } else {
            text
        };
        json!({"id": "glossary", "kind": "glossary", "heading": s.t("gloss.heading"), "text": text, "saved_actions": tabs, "rows": rows, "cards": cards, "actions": [act_json("glossary.close", "", s.t("gloss.close"), true)]})
    }

    /// A game against the computer waits while the glossary is open; an
    /// online one cannot, since the other players' days go on.
    fn glossary_pauses(&self) -> bool {
        self.glossary.is_some() && self.m.as_ref().map(|m| m.mode != Mode::Online).unwrap_or(false)
    }

    /// A piece as a builder tile: its glyph, name and school (T17).
    fn tile(&self, k: u16) -> Value {
        let c = &self.c;
        let piece = &c.file.pieces[k as usize];
        let (d, view) = icon(std::slice::from_ref(&piece.glyph));
        let kind = match piece.kind {
            PieceKind::Payload { .. } => "pay",
            PieceKind::Trigger { .. } => "trig",
            PieceKind::Branch => "branch",
            _ => "mod",
        };
        // what the piece does, in its own words and its numbers (T38)
        let facts = self.piece_card(k as usize, String::new())["facts"]
            .as_str()
            .unwrap_or("")
            .to_string();
        json!({"id": piece.id, "name": piece.name, "icon": d, "view": view, "school": c.file.schools[c.piece_school[k as usize]].id, "kind": kind, "text": piece.text, "facts": facts})
    }

    /// The draft as the builder shows it (T17): rows of sets, each an
    /// optional run of modifiers, a payload and a trigger; a branch begins
    /// the second row. The last set is open, with slots for what may come.
    fn lane(&self, p: &prep::Prep) -> Value {
        let c = &self.c;
        let s = &self.s;
        let mut rows: Vec<Vec<Value>> = vec![Vec::new()];
        let blank = || json!({"mods": [], "payload": null, "trigger": null, "open": false});
        let mut cur = blank();
        let filled = |v: &Value| {
            !v["mods"].as_array().map(|m| m.is_empty()).unwrap_or(true) || !v["payload"].is_null()
        };
        for (i, &k) in p.draft.iter().enumerate() {
            let mut t = self.tile(k);
            t["i"] = json!(i);
            match c.file.pieces[k as usize].kind {
                PieceKind::Payload { .. } => {
                    t["mana"] = mana(s, spell::stats(c, &[k]).mana);
                    if !cur["payload"].is_null() {
                        rows.last_mut()
                            .unwrap()
                            .push(std::mem::replace(&mut cur, blank()));
                    }
                    cur["payload"] = t;
                }
                PieceKind::Trigger { .. } => {
                    t["does"] = json!(c.file.pieces[k as usize].text);
                    cur["trigger"] = t;
                    rows.last_mut()
                        .unwrap()
                        .push(std::mem::replace(&mut cur, blank()));
                }
                PieceKind::Branch => {
                    if filled(&cur) {
                        rows.last_mut()
                            .unwrap()
                            .push(std::mem::replace(&mut cur, blank()));
                    }
                    rows.push(vec![json!({"branch": t})]);
                }
                _ => {
                    if !cur["payload"].is_null() {
                        rows.last_mut()
                            .unwrap()
                            .push(std::mem::replace(&mut cur, blank()));
                    }
                    cur["mods"].as_array_mut().unwrap().push(t);
                }
            }
        }
        cur["open"] = json!(true);
        rows.last_mut().unwrap().push(cur);
        // each set's heading: its number, and the trigger that starts it
        let mut n = 0;
        for row in rows.iter_mut() {
            let mut after: Option<String> = None;
            for set in row.iter_mut().filter(|v| v.get("branch").is_none()) {
                n += 1;
                set["label"] = json!(match &after {
                    Some(t) => s.f(
                        "builder.set_after",
                        &[("n", n.to_string()), ("trigger", t.clone())]
                    ),
                    None => s.f("builder.set", &[("n", n.to_string())]),
                });
                after = set["trigger"]["name"].as_str().map(|x| x.to_string());
            }
        }
        let lane_mana = (!p.draft.is_empty()).then(|| mana(s, spell::stats(c, &p.draft).mana));
        json!({"rows": rows, "mana": lane_mana, "spell": if p.draft.is_empty() { s.t("builder.new") } else { spell::name(c, &p.draft) }, "labels": {"mod": s.t("builder.slot_mod"), "pay": s.t("builder.slot_pay"), "trig": s.t("builder.slot_trig"), "branch": s.t("builder.branch_row"), "mods": s.t("kind.modifier"), "payload": s.t("kind.payload"), "trigger": s.t("kind.trigger"), "your": s.t("builder.your_spell")}})
    }

    /// The pieces to add (T17, T32): a tab for each kind, the open tab's
    /// tiles, and how many of that kind are not read yet.
    fn tray(&self, p: &prep::Prep) -> Value {
        let c = &self.c;
        let s = &self.s;
        let groups = [
            ("pay", "builder.tray_pay"),
            ("mod", "builder.tray_mod"),
            ("trig", "builder.tray_trig"),
            ("branch", "builder.tray_branch"),
        ];
        // a spell's pieces: not helpers, not spells on yourself, not what
        // only the map table places
        let spell_piece = |i: usize| {
            !matches!(
                c.file.pieces[i].kind,
                PieceKind::Helper { .. } | PieceKind::SelfCast { .. }
            ) && spell::raises(c, i as u16)
                .map(|f| c.file.features[f].cast_life > 0)
                .unwrap_or(true)
        };
        let kind_of = |i: usize| self.tile(i as u16)["kind"].as_str().unwrap_or("").to_string();
        let open = groups
            .iter()
            .find(|g| g.0 == self.lab_tab)
            .map(|g| g.0)
            .unwrap_or("pay");
        let tabs: Vec<Value> = groups
            .iter()
            .map(|(kind, label)| {
                let mut a = act_json("lab.tab", kind, s.t(label), true);
                a["on"] = json!(*kind == open);
                a
            })
            .collect();
        let tiles: Vec<Value> = (0..c.file.pieces.len())
            .filter(|&i| p.known[i] && spell_piece(i) && kind_of(i) == open)
            .map(|i| {
                let mut t = self.tile(i as u16);
                if matches!(c.file.pieces[i].kind, PieceKind::Payload { .. }) {
                    t["mana"] = mana(s, spell::stats(c, &[i as u16]).mana);
                }
                t
            })
            .collect();
        let unread = (0..c.file.pieces.len())
            .filter(|&i| !p.known[i] && spell_piece(i) && kind_of(i) == open)
            .count();
        let unread_label = if unread > 0 {
            s.f("builder.unread", &[("n", unread.to_string())])
        } else {
            String::new()
        };
        json!({"tabs": tabs, "tiles": tiles, "kind": open, "unread_label": unread_label})
    }

    /// A spell piece as a card (T10): its glyph as an icon, its name, number,
    /// school and kind, what it does, and its numbers. The glossary shows
    /// every piece this way, and reading a book shows the one it taught.
    fn piece_card(&self, i: usize, chip: String) -> Value {
        let s = &self.s;
        let c = &self.c;
        let piece = &c.file.pieces[i];
        let secs = |ms: i32| format!("{}.{}", ms / 1000, ms % 1000 / 100);
        let kind = match piece.kind {
            PieceKind::Payload { .. } => "kind.payload",
            PieceKind::Modifier { .. } => "kind.modifier",
            PieceKind::Formation { .. } => "kind.formation",
            PieceKind::Invade => "kind.invade",
            PieceKind::Trigger { .. } => "kind.trigger",
            PieceKind::Helper { .. } => "kind.helper",
            PieceKind::Branch => "kind.branch",
            PieceKind::SelfCast { .. } => "kind.self",
        };
        let school = &c.file.schools[c.piece_school[i]];
        let mut lines: Vec<String> = Vec::new();
        match &piece.kind {
            PieceKind::Payload { .. } => match spell::raises(c, i as u16) {
                Some(f) => {
                    let f = &c.file.features[f];
                    lines.push(s.f(
                        "gloss.raises",
                        &[("mana", f.mana.to_string()), ("s", strings::seconds(f.raise_ms))],
                    ));
                }
                None => {
                    let st = spell::stats(c, &[i as u16]);
                    lines.push(s.f(
                        "gloss.payload",
                        &[
                            ("power", st.power.to_string()),
                            ("cooldown", secs(st.cooldown_ms)),
                            ("mana", st.mana.to_string()),
                        ],
                    ));
                }
            },
            PieceKind::Modifier { mods, .. } => {
                let list: Vec<String> = [
                    ("gloss.mod.damage", mods.damage_pct),
                    ("gloss.mod.area", mods.area_pct),
                    ("gloss.mod.speed", mods.speed_pct),
                    ("gloss.mod.cooldown", mods.cooldown_pct),
                    ("gloss.mod.pierce", mods.pierce),
                ]
                .iter()
                .filter(|(_, v)| *v != 0)
                .map(|(k, v)| {
                    s.f(
                        k,
                        &[(
                            "v",
                            if *v > 0 && *k != "gloss.mod.pierce" {
                                format!("+{v}")
                            } else {
                                v.to_string()
                            },
                        )],
                    )
                })
                .collect();
                lines.push(s.f("gloss.mods", &[("list", list.join(", "))]));
            }
            PieceKind::Formation {
                copies, share_pct, ..
            } => {
                lines.push(s.f(
                    "gloss.formation",
                    &[("copies", copies.to_string()), ("share", share_pct.to_string())],
                ));
            }
            PieceKind::Helper { helper } => {
                if let Some(h) = c.helper(helper) {
                    let h = &c.file.helpers[h];
                    lines.push(s.f(
                        "gloss.helper",
                        &[("mana", h.mana.to_string()), ("s", h.job_s.to_string())],
                    ));
                }
            }
            PieceKind::SelfCast { mana: m, life_ms, .. } => {
                let how_long = if *life_ms > 0 {
                    s.f("self.left", &[("s", strings::seconds(*life_ms))])
                } else {
                    s.t("gloss.self_day")
                };
                lines.push(s.f("gloss.self", &[("mana", m.to_string()), ("s", how_long)]));
            }
            PieceKind::Invade | PieceKind::Trigger { .. } | PieceKind::Branch => {}
        }
        let (d, view) = icon(std::slice::from_ref(&piece.glyph));
        // what it costs: a payload's mana on its own, a placement's or a helper's
        let cost = match &piece.kind {
            PieceKind::Payload { .. } => Some(match spell::raises(c, i as u16) {
                Some(f) => c.file.features[f].mana,
                None => spell::stats(c, &[i as u16]).mana,
            }),
            PieceKind::Helper { helper } => c.helper(helper).map(|h| c.file.helpers[h].mana),
            PieceKind::SelfCast { mana: m, .. } => Some(*m),
            _ => None,
        };
        json!({"number": s.f("gloss.number", &[("n", (i + 1).to_string())]), "name": piece.name, "kind": s.f("card.kind", &[("school", school.name.clone()), ("kind", s.t(kind))]), "school": school.id, "text": piece.text, "facts": lines.join(" "), "chip": chip, "icon": d, "view": view, "mana": cost.map(|n| mana(s, n))})
    }

    /// A whole spell a tome taught, as a card.
    fn tome_card(&self, t: usize) -> Value {
        let s = &self.s;
        let c = &self.c;
        let pieces = c.tome(t);
        let glyphs: Vec<Vec<String>> = pieces
            .iter()
            .map(|&k| c.file.pieces[k as usize].glyph.clone())
            .collect();
        let (d, view) = icon(&glyphs);
        let school = pieces
            .first()
            .map(|&k| c.file.schools[c.piece_school[k as usize]].id.clone())
            .unwrap_or_default();
        let names: Vec<String> = pieces
            .iter()
            .map(|&k| c.file.pieces[k as usize].name.clone())
            .collect();
        let st = spell::stats(c, &pieces);
        let facts = s.f(
            "gloss.spell_detail",
            &[
                ("pieces", names.join(" → ")),
                ("power", st.power.to_string()),
                (
                    "cooldown",
                    format!("{}.{}", st.cooldown_ms / 1000, st.cooldown_ms % 1000 / 100),
                ),
                ("mana", st.mana.to_string()),
            ],
        );
        json!({"number": "", "name": c.file.tomes[t].name, "kind": s.t("gloss.tome_kind"), "school": school, "text": s.t("gloss.tome_text"), "facts": facts, "chip": "", "icon": d, "view": view, "mana": mana(s, st.mana)})
    }

    fn ui_help(&self) -> Value {
        let s = &self.s;
        let rows = self.help_rows();
        let p = json!({"id": "help", "kind": "center", "heading": s.t("title.help"), "rows": rows, "actions": [act_json("screen.show", "title", s.t("back"), true)]});
        self.base("help", s.t("title.help"), vec![p])
    }

    fn ui_play(&self) -> Value {
        let Some(m) = self.m.as_ref() else {
            return self.ui_title();
        };
        match m.phase {
            Phase::Prep => self.ui_prep(m),
            Phase::Waiting => {
                let p = json!({"id": "waiting", "kind": "center", "heading": self.s.t("waiting.heading"), "text": self.s.t("waiting.text")});
                let mut v = self.base("play", self.s.t("game.name"), vec![p]);
                v["phase"] = json!("waiting");
                v
            }
            Phase::Battle => self.ui_battle(m),
            Phase::Summary => self.ui_summary(m),
            Phase::Results => self.ui_results(m),
        }
    }

    /// The rehearsal's chip in the HUD (T48): what today's battle would
    /// take off the score if it began now, and how the last change moved
    /// it.
    fn rehearsal_item(&self) -> Value {
        let s = &self.s;
        let r = &self.rehearsal;
        let running = r.battle.is_some();
        let value = match r.result {
            Some(d) => s.f("rehearsal.loss", &[("n", strings::number(d))]),
            None => s.t("rehearsal.first"),
        };
        let note = match (r.result, r.before, running) {
            (_, _, true) if r.result.is_some() => s.t("rehearsal.updating"),
            (Some(now), Some(was), false) if now < was => {
                s.f("rehearsal.better", &[("n", strings::number(was - now))])
            }
            (Some(now), Some(was), false) if now > was => {
                s.f("rehearsal.worse", &[("n", strings::number(now - was))])
            }
            (Some(_), Some(_), false) => s.t("rehearsal.same"),
            _ => String::new(),
        };
        let tone = match (r.result, r.before, running) {
            (Some(now), Some(was), false) if now < was => "good",
            (Some(now), Some(was), false) if now > was => "bad",
            _ => "",
        };
        json!({"label": s.t("rehearsal.label"), "value": value, "note": note, "tone": tone, "kind": if running { "rehearsal stale" } else { "rehearsal" }, "hint": s.t("rehearsal.hint")})
    }

    fn hud(&self, m: &Match) -> Value {
        let s = &self.s;
        let p = m.local_prep();
        // the bar is your maximum: what you have, and at its right end the
        // part the field holds (T61)
        let max = p.map(|p| p.mana_max(&self.c)).unwrap_or(1).max(1);
        let mana = p.map(|p| p.mana * 1000 / max).unwrap_or(0);
        let held = p.map(|p| p.held(&self.c)).unwrap_or(0);
        let clock = match m.mode {
            Mode::Trial(0) | Mode::Trial(1) => strings::clock(m.elapsed),
            _ => strings::clock(p.map(|p| p.clock).unwrap_or(0)),
        };
        let items = vec![
            json!({"label": s.f("hud.day", &[("day", (m.day + 1).to_string()), ("days", m.settings.days.to_string())]), "value": clock, "kind": "timer"}),
            json!({"label": s.t("hud.score"), "value": strings::number(m.seats[m.local as usize].score)}),
            json!({"label": s.t("hud.students"), "value": p.map(|p| p.students.len()).unwrap_or(0).to_string()}),
            json!({"label": s.t("hud.pages"), "value": p.map(|p| p.known.iter().filter(|&&k| k).count()).unwrap_or(0).to_string()}),
            self.rehearsal_item(),
            json!({"label": s.t("hud.mana"), "value": if held > 0 { s.f("hud.mana_held", &[("n", strings::number(p.map(|p| p.mana / 100).unwrap_or(0))), ("held", strings::number(held))]) } else { strings::number(p.map(|p| p.mana / 100).unwrap_or(0)) }, "bar": strings::css_pct(mana), "held": (held > 0).then(|| strings::css_pct((held * 100 * 1000 / max).min(1000))), "hint": s.t("hud.held_hint")}),
        ];
        json!({"id": "hud", "kind": "hud", "items": items})
    }

    fn ui_prep(&self, m: &Match) -> Value {
        let s = &self.s;
        let Some(p) = m.local_prep() else {
            return self.base("play", s.t("game.name"), vec![]);
        };
        let mut panels = vec![self.hud(m)];
        let station = p.station();
        let busy = match &p.activity {
            Activity::Idle => None,
            Activity::Reading { .. } => Some("reading"),
            Activity::Interview { .. } => Some("interview"),
            Activity::Drawing { .. } => Some("drawing"),
            Activity::Sealing { .. } => Some("sealing"),
            Activity::Sorting { .. } => Some("sorting"),
            Activity::Journey { .. } => Some("journey"),
        };
        if let Some(b) = busy {
            let text = match b {
                "reading" => s.t("busy.reading"),
                "interview" => s.t("busy.interview"),
                "sealing" => s.t("busy.sealing"),
                "sorting" => s.t("busy.sorting"),
                _ => s.t("busy.drawing"),
            };
            match &p.activity {
                Activity::Sealing { rings, at } => panels.push(self.ui_seal(p, rings, *at)),
                Activity::Sorting { books, held, per, misses } => {
                    panels.push(self.ui_sort(books, *held, per, *misses));
                    panels.push(self.ui_golem_card(p));
                }
                Activity::Journey { student, target, ledge, falls, .. } => {
                    panels.push(self.ui_journey(m, p, *student, *target, *ledge, *falls));
                }
                _ => panels.push(json!({"id": "busy", "kind": "bottom", "text": text, "actions": [act_json("prep.cancel", "", s.t("busy.cancel"), true)]})),
            }
        } else if self.layout.open {
            panels.push(self.ui_layout(p));
            // what is rising changes every second, so the route has a
            // panel of its own and the map's buttons stay put (T9)
            panels.push(self.ui_route(p));
        } else if let Some(st) = station {
            panels.push(self.ui_station(m, p, st));
        } else {
            panels.push(json!({"id": "walk", "kind": "bottom", "text": s.t("prep.walk_hint")}));
        }
        // helpers at work
        if !p.jobs.is_empty() {
            let rows: Vec<Value> = p
                .jobs
                .iter()
                .enumerate()
                .map(|(k, j)| {
                    // a wall placed with mana rises by itself, with no helper (T1)
                    let label = match &j.work {
                        prep::Work::Raise(pl) => s.f("map.rising_short", &[("name", self.c.file.features[pl.feature as usize].name.clone())]),
                        _ => self.c.file.helpers[j.helper as usize].name.clone(),
                    };
                    {
                        // in whole seconds, so the panel changes once a second, not every tick
                        let (left_s, total_s) = ((j.left + TPS - 1) / TPS, ((j.total + TPS - 1) / TPS).max(1));
                        let detail = if j.waiting {
                            s.f("jobs.waiting", &[("mana", self.c.file.helpers[j.helper as usize].mana.to_string())])
                        } else if j.work.repeats() {
                            s.f("jobs.again", &[("t", strings::clock(j.left)), ("mana", self.c.file.helpers[j.helper as usize].mana.to_string())])
                        } else {
                            strings::clock(j.left)
                        };
                        let mut r = json!({"label": label, "detail": detail, "bar": strings::css_pct((total_s - left_s) * 1000 / total_s)});
                        // a helper works on until it is sent away (T55)
                        if j.work.repeats() {
                            r["actions"] = json!([act_json("helper.dismiss", &k.to_string(), s.t("jobs.dismiss"), true)]);
                        }
                        r
                    }
                })
                .collect();
            // beside the station's panel, not on top of it
            panels.push(json!({"id": "jobs", "kind": "jobs", "heading": s.t("jobs.heading"), "rows": rows}));
        }
        // spells you can cast on yourself: small icons, their text on hover (T24, T28)
        let selves: Vec<Value> = (0..self.c.file.pieces.len())
            .filter(|&i| p.known[i] && matches!(self.c.file.pieces[i].kind, PieceKind::SelfCast { .. }))
            .map(|i| {
                let piece = &self.c.file.pieces[i];
                let left = p.boons.iter().find(|bn| bn.0 as usize == i).map(|bn| bn.1);
                let label = left
                    .map(|t| s.f("self.left", &[("s", ((t + TPS - 1) / TPS).to_string())]))
                    .unwrap_or_default();
                let card = self.piece_card(i, String::new());
                let mut a = act_json("self.cast", &piece.id, label, true);
                a["icon"] = card["icon"].clone();
                a["view"] = card["view"].clone();
                a["hint"] = json!(format!(
                    "{}: {} {}",
                    piece.name,
                    piece.text,
                    card["facts"].as_str().unwrap_or("")
                ));
                a
            })
            .collect();
        if !selves.is_empty() {
            panels.push(json!({"id": "self", "kind": "self", "actions": selves}));
        }
        panels.push(json!({"id": "ready", "kind": "corner", "actions": [act_json("day.ready", "", s.t("prep.ready"), true), act_json("match.leave", "", s.t("leave"), true)]}));
        let labels: Vec<Value> = if busy.is_none() && !self.layout.open {
            self.station_labels(p)
        } else {
            vec![]
        };
        let mut v = self.base("play", s.t("game.name"), panels);
        v["phase"] = json!("prep");
        v["station"] = json!(station.map(|s| s.key()));
        v["busy"] = json!(busy);
        v["labels"] = json!(labels);
        v["overlay"] = json!(busy == Some("drawing") || busy == Some("interview") || self.layout.open);
        // the archmage's place in decimetres, for the browser check to walk by
        let dm = |v: i32| v * 10 / crate::fx::ONE;
        // the sorting game's book and the seal's ring, for a browser check
        // that plays them by keys
        let (book, ring) = match &p.activity {
            Activity::Sorting { books, .. } => (
                books.first().map(|b| b.to_string()).unwrap_or_default(),
                String::new(),
            ),
            Activity::Sealing { rings, at } => (
                String::new(),
                rings.get(*at).map(|r| r.to_string()).unwrap_or_default(),
            ),
            _ => (String::new(), String::new()),
        };
        // a panel's instructions go behind a ? on its title (T49)
        if let Some(ps) = v["panels"].as_array_mut() {
            for panel in ps.iter_mut() {
                tuck_help(panel);
            }
        }
        v["data"] = json!({"students": p.students.len(), "spells": p.spells.len(), "known": p.known.iter().filter(|&&k| k).count(), "placed": p.placed.len(), "x": dm(p.pos.x), "z": dm(p.pos.z), "book": book, "ring": ring});
        v
    }

    fn station_labels(&self, p: &prep::Prep) -> Vec<Value> {
        // once the camera has come down behind the archmage, the station
        // in front of them speaks for itself (T35)
        if p.dip > crate::fx::ONE / 2 {
            return vec![];
        }
        let cam = scene::room_camera(p);
        let (w, h) = self.canvas;
        let aspect = (w.max(1) as i64 * crate::fx::ONE as i64 / h.max(1) as i64) as i32;
        let mtx = cam.matrix(aspect);
        prep::STATIONS
            .iter()
            .filter_map(|(st, _, _)| {
                let at = prep::station_pos(*st).add(v3(0, crate::fx::ONE * 7 / 2, 0));
                geom::project(&mtx, at).map(|(x, y)| json!({"text": self.s.t(&format!("station.{}", st.key())), "left": strings::css_pct(x), "top": strings::css_pct(y), "kind": "station"}))
            })
            .collect()
    }

    fn ui_station(&self, m: &Match, p: &prep::Prep, st: Station) -> Value {
        let s = &self.s;
        let c = &self.c;
        match st {
            Station::Library => {
                let left = p
                    .pile
                    .iter()
                    .filter(|b| match b {
                        prep::Book::Piece(k) => !p.known[*k as usize],
                        prep::Book::Tome(_) => true,
                    })
                    .count();
                let last = match &p.last_read {
                    Some(prep::Book::Piece(k)) => {
                        let piece = &c.file.pieces[*k as usize];
                        s.f(
                            "library.last",
                            &[("name", piece.name.clone()), ("text", piece.text.clone())],
                        )
                    }
                    Some(prep::Book::Tome(t)) => s.f(
                        "library.last_tome",
                        &[("name", c.file.tomes[*t as usize].name.clone())],
                    ),
                    None => s.t("library.none_yet"),
                };
                let g = c.helper("librarian_golem").map(|h| c.file.helpers[h].clone());
                let searching = p.jobs.iter().any(|j| matches!(j.work, prep::Work::Search));
                let mut golem = act_json(
                    "library.golem",
                    "",
                    s.f(
                        "library.golem",
                        &[("s", g.as_ref().map(|h| h.job_s).unwrap_or(0).to_string())],
                    ),
                    !searching && left > 0,
                );
                golem["mana"] = mana(s, g.map(|h| h.mana).unwrap_or(0));
                json!({"id": "station", "kind": "side", "heading": s.t("station.library"), "text": format!("{} {}", s.f("library.left", &[("n", left.to_string())]), last), "actions": [act_json("library.read", "", s.t("library.read"), left > 0), golem]})
            }
            Station::Lab => {
                let mut rows = Vec::new();
                for (i, piece) in c.file.pieces.iter().enumerate() {
                    if !p.known[i] || matches!(piece.kind, PieceKind::Helper { .. }) {
                        continue;
                    }
                    let kind = match piece.kind {
                        PieceKind::Payload { .. } => "kind.payload",
                        PieceKind::Modifier { .. } | PieceKind::Formation { .. } | PieceKind::Invade => {
                            "kind.modifier"
                        }
                        PieceKind::Trigger { .. } => "kind.trigger",
                        PieceKind::Helper { .. } => "kind.helper",
                        PieceKind::Branch => "kind.branch",
                        PieceKind::SelfCast { .. } => "kind.self",
                    };
                    // a payload's own mana; a modifier's depends on the spell it is in
                    let cost = matches!(piece.kind, PieceKind::Payload { .. })
                        .then(|| mana(s, spell::stats(c, &[i as u16]).mana));
                    rows.push(json!({"label": piece.name, "detail": piece.text, "chip": s.t(kind), "mana": cost, "school": c.file.schools[c.piece_school[i]].id, "actions": [act_json("lab.add", &piece.id, s.t("lab.add"), true)]}));
                }
                let draft: Vec<Value> = p
                    .draft
                    .iter()
                    .enumerate()
                    .map(|(k, &d)| {
                        act_json(
                            "lab.remove",
                            &k.to_string(),
                            c.file.pieces[d as usize].name.clone(),
                            true,
                        )
                    })
                    .collect();
                let stats = if p.draft.is_empty() {
                    s.t("builder.empty")
                } else {
                    let st = spell::stats(c, &p.draft);
                    let check = spell::check(c, &p.draft)
                        .err()
                        .map(|e| s.t(e.key()))
                        .unwrap_or_default();
                    format!(
                        "{} {check}",
                        s.f(
                            "lab.stats",
                            &[
                                ("power", st.power.to_string()),
                                (
                                    "cooldown",
                                    format!("{}.{}", st.cooldown_ms / 1000, st.cooldown_ms % 1000 / 100)
                                ),
                                ("mana", st.mana.to_string())
                            ]
                        )
                    )
                };
                // the builder (T17, T32), after its mockup: the lane of sets, the
                // tray of tiles by kind, and this spell's numbers to the right
                let chosen: Vec<Value> = p
                    .spells
                    .iter()
                    .enumerate()
                    .map(|(k, ch)| {
                        let mut a = act_json("spell.choose", &k.to_string(), spell::name(c, ch), true);
                        a["mana"] = mana(s, spell::stats(c, ch).mana);
                        a["on"] = json!(k as u16 == p.chosen);
                        a
                    })
                    .collect();
                let draft_mana = (!p.draft.is_empty()).then(|| mana(s, spell::stats(c, &p.draft).mana));
                let _ = (rows, draft);
                let mut bl = vec![
                    json!({"t": "title", "col": "head", "text": s.t("station.lab")}),
                    json!({"t": "p", "col": "head", "tone": "help", "text": s.t("builder.lead")}),
                    json!({"t": "lane", "col": "lane", "lane": self.lane(p)}),
                    json!({"t": "tray", "col": "tray", "tray": self.tray(p)}),
                    json!({"t": "p", "col": "tray", "tone": "help", "text": s.t("builder.drag")}),
                    json!({"t": "h", "col": "sum", "text": s.t("builder.this_spell")}),
                    json!({"t": "title", "col": "sum", "text": if p.draft.is_empty() { s.t("builder.new") } else { spell::name(c, &p.draft) }}),
                    json!({"t": "p", "col": "sum", "text": stats}),
                ];
                // what the spell does, piece by piece, in order (T38)
                if !p.draft.is_empty() {
                    bl.push(json!({"t": "h", "col": "sum", "text": s.t("builder.in_order")}));
                    let items: Vec<Value> = p
                        .draft
                        .iter()
                        .map(|&k| {
                            let piece = &c.file.pieces[k as usize];
                            let facts = self.piece_card(k as usize, String::new())["facts"].as_str().unwrap_or("").to_string();
                            json!({"lead": piece.name, "text": if facts.is_empty() { piece.text.clone() } else { format!("{} {}", piece.text, facts) }})
                        })
                        .collect();
                    bl.push(json!({"t": "ol", "col": "sum", "items": items}));
                }
                if let Some(m) = draft_mana {
                    bl.push(json!({"t": "mana", "col": "sum", "mana": m}));
                }
                bl.push(json!({"t": "actions", "col": "sum", "actions": [act_json("lab.save", "", s.t("lab.save"), !p.draft.is_empty() && spell::check(c, &p.draft).is_ok()), act_json("lab.clear", "", s.t("lab.clear"), !p.draft.is_empty())]}));
                bl.push(json!({"t": "p", "col": "sum", "tone": "help", "text": s.t("builder.seal_note")}));
                if !chosen.is_empty() {
                    bl.push(json!({"t": "h", "col": "tray", "text": s.t("builder.your_spells")}));
                    bl.push(json!({"t": "actions", "col": "tray", "style": "chips", "actions": chosen}));
                }
                json!({"id": "station", "kind": "builder", "blocks": bl})
            }
            Station::Recruit => {
                let rows: Vec<Value> = p
                    .applicants
                    .iter()
                    .enumerate()
                    .map(|(k, a)| json!({"label": a.name, "detail": s.f("recruit.leans", &[("good", c.file.schools[a.school as usize].name.clone()), ("bad", c.file.schools[a.weak as usize].name.clone())]), "school": c.file.schools[a.school as usize].id, "actions": [act_json("recruit.one", &k.to_string(), s.t("recruit.interview"), true)]}))
                    .collect();
                // one applicant at a time (T51), up to the college's limit (T56)
                let full = p.class_full(c);
                let rows: Vec<Value> = rows
                    .into_iter()
                    .map(|mut r| {
                        r["actions"][0]["enabled"] = json!(!full);
                        r
                    })
                    .collect();
                let mut actions = Vec::new();
                if let Some(h) = c.piece("recruitment_elemental").filter(|&h| p.known[h]) {
                    actions.push(act_json(
                        "podium.summon",
                        &c.file.pieces[h].id,
                        s.t("recruit.summon"),
                        true,
                    ));
                }
                let text = if full {
                    s.f(
                        "recruit.full_text",
                        &[("n", c.balance().max_students.to_string())],
                    )
                } else {
                    s.f(
                        "recruit.text",
                        &[
                            ("n", p.applicants.len().to_string()),
                            ("have", p.students.len().to_string()),
                            ("max", c.balance().max_students.to_string()),
                        ],
                    )
                };
                json!({"id": "station", "kind": "side", "heading": s.t("station.recruit"), "text": text, "rows": rows, "actions": actions})
            }
            Station::Podium => {
                let spell = p
                    .spells
                    .get(p.chosen as usize)
                    .map(|ch| spell::name(c, ch))
                    .unwrap_or_default();
                // a class is everyone with room for the spell (T50), and earns
                // less for the drawing the bigger it is (T52)
                let class = p.class_for(c, p.chosen, None).len() as i32;
                let mut actions = vec![act_json(
                    "podium.teach",
                    "all",
                    s.f(
                        "podium.teach_all",
                        &[
                            ("n", class.to_string()),
                            ("pct", prep::Prep::class_pct(c, class).to_string()),
                        ],
                    ),
                    class > 1 && !p.spells.is_empty(),
                )];
                for h in ["teaching_elemental", "mana_trainer"] {
                    if let Some(k) = c.piece(h).filter(|&k| p.known[k]) {
                        actions.push(act_json(
                            "podium.summon",
                            h,
                            s.f("podium.summon", &[("name", c.file.pieces[k].name.clone())]),
                            true,
                        ));
                    }
                }
                let rows: Vec<Value> = p
                    .students
                    .iter()
                    .map(|st| {
                        let slots = prep::slots(c, st);
                        let knows = st.known.iter().find(|k| k.0 == p.chosen).map(|k| k.1);
                        let eff = knows.map(|e| format!("{e}%")).unwrap_or_else(|| s.t("podium.not_taught"));
                        // what a full student would forget for this spell (T50)
                        let forgets = (knows.is_none() && st.known.len() as i32 >= slots)
                            .then(|| st.known.iter().enumerate().min_by_key(|(i, k)| (k.1, *i)).map(|(_, k)| k.0))
                            .flatten()
                            .and_then(|k| p.spells.get(k as usize))
                            .map(|ch| s.f("podium.forgets", &[("spell", spell::name(c, ch))]));
                        let detail = s.f("podium.row", &[("eff", eff), ("n", st.known.len().to_string()), ("slots", slots.to_string())]);
                        let detail = match forgets {
                            Some(f) => format!("{detail} · {f}"),
                            None => detail,
                        };
                        json!({"label": st.name, "detail": detail, "school": c.file.schools[st.school as usize].id, "actions": [act_json("podium.teach", &st.id.to_string(), s.t("podium.teach_one"), true)]})
                    })
                    .collect();
                let saved: Vec<Value> = p
                    .spells
                    .iter()
                    .enumerate()
                    .map(|(k, ch)| {
                        act_json(
                            "spell.choose",
                            &k.to_string(),
                            spell::name(c, ch),
                            k as u16 != p.chosen,
                        )
                    })
                    .collect();
                // the last lesson's card: what each student got (T48)
                let mut v = json!({"id": "station", "kind": "side", "heading": s.t("station.podium"), "text": s.f("podium.text", &[("spell", spell)]), "saved_actions": saved, "rows": rows, "actions": actions});
                if let Some(l) = &p.lesson {
                    let lines: Vec<Value> = l
                        .learned
                        .iter()
                        .filter_map(|x| {
                            let st = p.students.iter().find(|st| st.id == x.id)?;
                            let change = match x.before {
                                Some(b) if x.after > b => s.f(
                                    "lesson.up",
                                    &[("from", b.to_string()), ("to", x.after.to_string())],
                                ),
                                Some(b) => s.f("lesson.same", &[("to", b.to_string())]),
                                None => s.f("lesson.new", &[("to", x.after.to_string())]),
                            };
                            Some(json!({"label": st.name, "detail": change}))
                        })
                        .collect();
                    let name = p
                        .spells
                        .get(l.spell as usize)
                        .map(|ch| spell::name(c, ch))
                        .unwrap_or_default();
                    v["lesson"] = json!({"heading": s.f("lesson.heading", &[("spell", name), ("score", l.score.to_string()), ("pct", l.class_pct.to_string())]), "rows": lines});
                }
                v
            }
            Station::Book => {
                let mut rows: Vec<Value> = m
                    .wild
                    .iter()
                    .map(|w| {
                        row(
                            s.t("book.wild"),
                            s.f(
                                "book.group",
                                &[
                                    ("n", w.count.to_string()),
                                    ("name", c.file.enemies[w.enemy as usize].name.clone()),
                                    (
                                        "type",
                                        c.file.types
                                            [c.damage_type(&c.file.enemies[w.enemy as usize].damage_type)]
                                        .name
                                        .clone(),
                                    ),
                                ],
                            ),
                        )
                    })
                    .collect();
                for inv in m.book() {
                    rows.push(row(
                        m.seats[inv.from as usize].name.clone(),
                        s.f("book.invasion", &[("spell", spell::name(c, &inv.chain))]),
                    ));
                }
                rows.push(row(s.t("book.students_heading"), s.t("book.students_hidden")));
                let mut actions = Vec::new();
                // an invasion goes to every rival at once (T29)
                let rivals = m
                    .seats
                    .iter()
                    .enumerate()
                    .filter(|(t, seat)| *t != m.local as usize && seat.active())
                    .count();
                for (k, ch) in p.spells.iter().enumerate() {
                    if !spell::is_invasion(c, ch) || rivals == 0 {
                        continue;
                    }
                    let cost = spell::stats(c, ch).mana * c.balance().invasion_mana_pct / 100;
                    let mut a = act_json(
                        "book.cast",
                        &k.to_string(),
                        s.f(
                            "book.cast_all",
                            &[("spell", spell::name(c, ch)), ("n", rivals.to_string())],
                        ),
                        true,
                    );
                    a["mana"] = mana(s, cost);
                    actions.push(a);
                }
                self.ui_book(m, p, rows, actions)
            }
            Station::Office => self.ui_office(p),
            Station::Map => {
                json!({"id": "station", "kind": "side", "heading": s.t("station.map"), "text": s.f("map.text", &[("n", p.placed.len().to_string())]), "actions": [act_json("layout.open", "", s.t("map.open"), true)]})
            }
        }
    }

    /// The student office hours shows: the one picked, else the first.
    fn office_student<'a>(&self, p: &'a prep::Prep) -> Option<&'a prep::Student> {
        p.students
            .iter()
            .find(|s| Some(s.id) == self.office_pick)
            .or_else(|| p.students.first())
    }

    /// A seat's name for a reader (T15).
    fn seat_name(&self, seat: i32) -> String {
        let s = &self.s;
        match seat {
            battle::AUTO_SEAT => s.t("office.seat_auto"),
            battle::GROUND_SEAT => s.t("office.seat_ground"),
            k if k >= battle::SUB_SEAT => s.f(
                "office.seat_sub",
                &[("n", (k - battle::SUB_SEAT + 1).to_string())],
            ),
            k => s.f("office.seat_tower", &[("n", (k + 1).to_string())]),
        }
    }

    /// The seats on the field's picture, with who holds each (T32, T33):
    /// the battle's own seating rule, so the picture is where they stand.
    fn seat_spots(&self, p: &prep::Prep) -> Vec<overlay::Spot> {
        let c = &self.c;
        let subs = self.sub_cells(p);
        let seats = c.balance().tower_seats;
        let got = self.seated(p);
        overlay::spots(&p.field(c), seats, &subs)
            .into_iter()
            .map(|(seat, x, z)| {
                let who = got.iter().position(|&g| g == seat).map(|i| &p.students[i]);
                overlay::Spot {
                    seat,
                    x,
                    z,
                    who: who.map(|st| st.id),
                    text: who.map(|st| initials(&st.name)).unwrap_or_default(),
                    on: who.is_some() && who.map(|st| st.id) == self.office_pick,
                    chose: who.map(|st| st.seat == seat).unwrap_or(false),
                }
            })
            .collect()
    }

    /// A cell for each sub-tower seat, in the order the battle numbers them.
    fn sub_cells(&self, p: &prep::Prep) -> Vec<(i32, i32)> {
        let c = &self.c;
        p.placed
            .iter()
            .flat_map(|q| {
                let n = c.file.features[q.feature as usize].seats.max(0) as usize;
                std::iter::repeat_n(q.cells.first().copied().unwrap_or((0, 0)), n)
            })
            .collect()
    }

    /// The seat each student will have when the battle starts.
    fn seated(&self, p: &prep::Prep) -> Vec<i32> {
        let wants: Vec<i32> = p.students.iter().map(|s| s.seat).collect();
        battle::seating(
            self.c.balance().tower_seats.max(0) as usize,
            self.sub_cells(p).len(),
            &wants,
        )
    }

    /// The field as boxes the page places (T32): the route, walls, the
    /// tower, and a button for each seat that takes a student dropped on it.
    fn seat_board(&self, p: &prep::Prep, pick: Option<u16>) -> Value {
        let c = &self.c;
        let a = p.field(c);
        let spots = self.seat_spots(p);
        // the board shows the part of the field round the seats, 8 wide to
        // 5 high and at least 18 cells across, as the mockup does
        let d = overlay::SPOT_SIZE;
        let (mut x0, mut z0, mut x1, mut z1) = (
            (a.tower.0 - 1) * 1000,
            (a.tower.1 - 1) * 1000,
            (a.tower.0 + 2) * 1000,
            (a.tower.1 + 2) * 1000,
        );
        for sp in &spots {
            x0 = x0.min(sp.x - d);
            z0 = z0.min(sp.z - d);
            x1 = x1.max(sp.x + d);
            z1 = z1.max(sp.z + d);
        }
        let n = arena::SIZE * 1000;
        let w = (x1 - x0).max((z1 - z0) * 8 / 5).max(18_000).min(n);
        let h = (w * 5 / 8).min(n);
        let left = ((x0 + x1) / 2 - w / 2).clamp(0, n - w);
        let top = ((z0 + z1) / 2 - h / 2).clamp(0, n - h);
        // thousandths of a cell to tenths of a percent of the board
        let px = |v: i32| strings::css_pct((v - left) * 1000 / w.max(1));
        let pz = |v: i32| strings::css_pct((v - top) * 1000 / h.max(1));
        let sx = |v: i32| strings::css_pct(v * 1000 / w.max(1));
        let sz = |v: i32| strings::css_pct(v * 1000 / h.max(1));
        let shown = |x: i32, z: i32, cw: i32, ch: i32| {
            x * 1000 < left + w && (x + cw) * 1000 > left && z * 1000 < top + h && (z + ch) * 1000 > top
        };
        let cell = |x: i32, z: i32, cw: i32, ch: i32, kind: &str| json!({"kind": kind, "left": px(x * 1000), "top": pz(z * 1000), "w": sx(cw * 1000), "h": sz(ch * 1000)});
        let mut items = Vec::new();
        // the route in straight runs
        let route = a.route();
        let mut k = 0;
        while k < route.len() {
            let (rx, rz) = route[k];
            let mut j = k;
            let along_x = route.get(k + 1).map(|q| q.1 == rz).unwrap_or(true);
            while let Some(&q) = route.get(j + 1) {
                if (along_x && q.1 == rz && (q.0 - route[j].0).abs() == 1)
                    || (!along_x && q.0 == rx && (q.1 - route[j].1).abs() == 1)
                {
                    j += 1;
                } else {
                    break;
                }
            }
            let (ex, ez) = route[j];
            let (cx, cz, cw, ch) = (rx.min(ex), rz.min(ez), (ex - rx).abs() + 1, (ez - rz).abs() + 1);
            if shown(cx, cz, cw, ch) {
                items.push(cell(cx, cz, cw, ch, "route"));
            }
            k = j + 1;
        }
        for z in 0..arena::SIZE {
            for x in 0..arena::SIZE {
                if !shown(x, z, 1, 1) {
                    continue;
                }
                match a.at(x, z) {
                    arena::Cell::Scenery => items.push(cell(x, z, 1, 1, "scenery")),
                    _ if (x, z) == a.entry => items.push(cell(x, z, 1, 1, "entry")),
                    _ => {}
                }
            }
        }
        for q in p.placed.iter().chain(p.rising()) {
            let kind = format!("sw-{}", overlay::swatch(&c.file.features[q.feature as usize].id));
            for &(x, z) in q.cells.iter().filter(|&&(x, z)| shown(x, z, 1, 1)) {
                items.push(cell(x, z, 1, 1, &kind));
            }
        }
        items.push(cell(a.tower.0 - 1, a.tower.1 - 1, 3, 3, "tower"));
        for sp in spots {
            let mut b = json!({"kind": "seat", "tone": if sp.text.is_empty() { "free" } else if sp.chose { "taken" } else { "given" }, "text": sp.text, "on": sp.on, "left": px(sp.x - d / 2), "top": pz(sp.z - d / 2), "w": sx(d), "h": sz(d), "hint": self.seat_name(sp.seat)});
            let mut a = act_json(
                "office.seat",
                &format!("{},{}", pick.unwrap_or(0), sp.seat),
                String::new(),
                pick.is_some(),
            );
            a["drop"] = json!(true);
            if let Some(who) = sp.who {
                a["grab"] = act_json("office.pick", &who.to_string(), String::new(), true);
                a["enabled"] = json!(true);
            }
            b["action"] = a;
            items.push(b);
        }
        json!({"t": "board", "col": "deploy", "aspect": "8 / 5", "items": items})
    }

    /// A student in the class list, as one button: press to pick them,
    /// drag them onto a seat (T32).
    fn student_row(&self, st: &prep::Student, pick: Option<u16>) -> Value {
        let c = &self.c;
        let mut press = act_json("office.pick", &st.id.to_string(), st.name.clone(), true);
        press["grab"] = act_json("office.pick", &st.id.to_string(), String::new(), true);
        json!({"label": st.name, "chip": self.seat_name(st.seat), "school": c.file.schools[st.school as usize].id, "on": pick == Some(st.id), "press": press})
    }

    /// A student's health and speed for the spells they know (T42), as
    /// shown: health, and speed in metres a second to a tenth.
    fn mage_numbers(&self, st: &prep::Student) -> (i32, String) {
        let b = self.c.balance();
        let plan = battle::StudentPlan {
            spells: st.known.clone(),
            ..Default::default()
        };
        let (hp, _) = battle::mage_stats(&self.c, &plan);
        let sum: i32 = st.known.iter().map(|k| k.1.max(0)).sum();
        let cms = b.mage_speed_cms + sum * b.mage_speed_per_100_cms / 100;
        (hp, format!("{}.{}", cms / 100, cms % 100 / 10))
    }

    /// The invasion book (T40, T44), after its mockup: your students to
    /// send, a road to each rival with your raiders on it, who stays on
    /// your towers, and the invasion spells as before.
    fn ui_book(&self, m: &Match, p: &prep::Prep, wild_rows: Vec<Value>, spell_actions: Vec<Value>) -> Value {
        let s = &self.s;
        let c = &self.c;
        let rivals: Vec<usize> = (0..m.seats.len())
            .filter(|&t| t != m.local as usize && m.seats[t].active())
            .collect();
        let temper = |t: u8| {
            c.file
                .temperaments
                .get(t as usize)
                .map(|x| x.name.clone())
                .unwrap_or_default()
        };
        let mut bl = vec![
            json!({"t": "title", "col": "list", "text": s.t("station.book")}),
            json!({"t": "p", "col": "list", "tone": "help", "text": s.t("raid.lead")}),
            json!({"t": "h", "col": "list", "text": s.t("raid.students")}),
        ];
        let rows: Vec<Value> = p
            .students
            .iter()
            .map(|st| {
                let (hp, speed) = self.mage_numbers(st);
                let detail = match st.raid {
                    Some((t, _)) => s.f("raid.away", &[("rival", m.seats.get(t as usize).map(|x| x.name.clone()).unwrap_or_default())]),
                    None => s.f("raid.stats", &[("hp", hp.to_string()), ("speed", speed), ("n", st.known.len().to_string())]),
                };
                let mut press = act_json("raid.hold", &st.id.to_string(), st.name.clone(), st.raid.is_none());
                press["grab"] = act_json("raid.hold", &st.id.to_string(), String::new(), true);
                let actions = if st.raid.is_some() { vec![act_json("raid.recall", &st.id.to_string(), s.t("raid.recall"), true)] } else { vec![] };
                json!({"label": st.name, "detail": detail, "chip": temper(st.temper), "school": c.file.schools[st.school as usize].id, "on": self.raid_held == Some(st.id), "dim": st.raid.is_some(), "press": press, "actions": actions})
            })
            .collect();
        if rows.is_empty() {
            bl.push(json!({"t": "p", "col": "list", "text": s.t("office.no_students")}));
        } else {
            bl.push(json!({"t": "rows", "col": "list", "rows": rows}));
        }
        bl.push(json!({"t": "p", "col": "list", "tone": "help", "text": s.t("raid.how")}));
        // the road: your tower, then each rival, with your raiders on the way
        let mut items = vec![
            json!({"kind": "roof", "left": "0%", "top": "14%", "w": "11%", "h": "17%"}),
            json!({"kind": "tower", "text": s.t("raid.you"), "left": "2%", "top": "30%", "w": "7%", "h": "62%"}),
        ];
        // a dotted road from your tower to each rival's card (T44)
        let n = rivals.len().max(1) as i32;
        let along = |k: usize, u: i32| {
            let card = 40 + k as i32 * 1000 / n + (1000 / n - 60) / 2;
            let (x0, y0, x1) = (100, 700, 610);
            // level at first, then up or down to the card
            let e = u * u / 1000;
            (x0 + (x1 - x0) * u / 1000, y0 + (card - y0) * e / 1000)
        };
        for k in 0..rivals.len() {
            for d in 1..18 {
                let (x, y) = along(k, d * 1000 / 18);
                items.push(json!({"kind": "dot", "left": strings::css_pct(x - 6), "top": strings::css_pct(y - 12), "w": "1.2%", "h": "3.4%"}));
            }
        }
        for (k, &t) in rivals.iter().enumerate() {
            let top = 40 + k as i32 * 1000 / rivals.len().max(1) as i32;
            let mine: Vec<&prep::Student> = p
                .students
                .iter()
                .filter(|st| st.raid.map(|r| r.0 as usize) == Some(t))
                .collect();
            let names: Vec<String> = mine.iter().map(|st| st.name.clone()).collect();
            let text = if names.is_empty() {
                s.f(
                    "raid.rival_none",
                    &[
                        ("rival", m.seats[t].name.clone()),
                        ("score", strings::number(m.seats[t].score)),
                    ],
                )
            } else {
                s.f(
                    "raid.rival",
                    &[
                        ("rival", m.seats[t].name.clone()),
                        ("score", strings::number(m.seats[t].score)),
                        ("names", names.join(", ")),
                    ],
                )
            };
            let mut a = act_json(
                "raid.drop",
                &t.to_string(),
                String::new(),
                self.raid_held.is_some(),
            );
            a["drop"] = json!(true);
            items.push(json!({"kind": "rival", "text": text, "action": a, "on": self.raid_held.is_some(), "left": "62%", "top": strings::css_pct(top), "w": "37%", "h": strings::css_pct(1000 / rivals.len().max(1) as i32 - 60)}));
            for (j, st) in mine.iter().enumerate() {
                let (x, y) = along(k, 820 - j as i32 * 90);
                items.push(json!({"kind": "walker", "tone": format!("school-{}", c.file.schools[st.school as usize].id), "text": initials(&st.name),
                    "left": strings::css_pct(x - 20), "top": strings::css_pct(y - 80), "w": "4%", "h": "16%"}));
            }
        }
        if rivals.is_empty() {
            items.push(json!({"kind": "note", "text": s.t("raid.no_rivals"), "left": "20%", "top": "38%", "w": "60%", "h": "24%"}));
        }
        bl.push(json!({"t": "board", "col": "road", "aspect": "26 / 9", "items": items}));
        let home = p.students.iter().filter(|st| st.raid.is_none()).count();
        bl.push(json!({"t": "h", "col": "home", "text": s.t("raid.home_heading")}));
        bl.push(json!({"t": "p", "col": "home", "text": s.f("raid.home", &[("n", home.to_string()), ("of", p.students.len().to_string())])}));
        let staying: Vec<Value> = p
            .students
            .iter()
            .filter(|st| st.raid.is_none())
            .map(|st| json!({"text": st.name, "tone": ""}))
            .collect();
        bl.push(json!({"t": "chips", "col": "home", "chips": staying}));
        bl.push(json!({"t": "h", "col": "spells", "text": s.t("raid.spells_heading")}));
        bl.push(json!({"t": "p", "col": "spells", "tone": "help", "text": s.t("book.text")}));
        bl.push(json!({"t": "rows", "col": "spells", "rows": wild_rows}));
        if !spell_actions.is_empty() {
            bl.push(json!({"t": "actions", "col": "spells", "actions": spell_actions}));
        }
        json!({"id": "station", "kind": "raid", "blocks": bl})
    }

    /// A temperament's tree as a board (T41), after the BT lab: a choice
    /// spreads its options across, a sequence stacks its steps down, and
    /// each node is lit by what it answered last (`lit`, empty for none).
    fn tree_board(&self, temper: u8, lit: &[u8]) -> Value {
        use crate::mind::Kind;
        let s = &self.s;
        let Some(tree) = self.c.minds.get(temper as usize) else {
            return json!({"t": "p", "text": ""});
        };
        let (at, cols) = tree.layout();
        let rows = at.iter().map(|a| a.1).max().unwrap_or(0) + 1;
        let (cw, rh) = (1000 / cols.max(1), 1000 / rows.max(1));
        let label = |k: &Kind| match k {
            Kind::First => s.t("bt.first"),
            Kind::Seq => s.t("bt.seq"),
            Kind::If(q) => s.t(&format!(
                "bt.if.{}",
                serde_json::to_value(q)
                    .ok()
                    .and_then(|v| v.as_str().map(|x| x.to_string()))
                    .unwrap_or_default()
            )),
            Kind::Do(a) => s.t(&format!(
                "bt.do.{}",
                serde_json::to_value(a)
                    .ok()
                    .and_then(|v| v.as_str().map(|x| x.to_string()))
                    .unwrap_or_default()
            )),
        };
        let mut items = Vec::new();
        let indent = |n: usize| -> i32 {
            // a sequence's steps sit a little to the right of it
            let deep = tree
                .nodes
                .iter()
                .enumerate()
                .any(|(p, (k, kids))| *k == Kind::Seq && kids.contains(&n) && at[p].0 == at[n].0);
            if deep {
                cw / 10
            } else {
                0
            }
        };
        for (n, (kind, kids)) in tree.nodes.iter().enumerate() {
            let (c0, r0) = at[n];
            let left = c0 * cw + cw / 20 + indent(n);
            let top = r0 * rh + rh / 8;
            let (w, h) = (cw * 8 / 10, rh * 7 / 10);
            let tone = match lit.get(n).copied().unwrap_or(0) {
                crate::mind::RAN_DONE => "ran",
                crate::mind::RAN_RUNNING => "running",
                crate::mind::RAN_FAILED => "failed",
                _ => "",
            };
            let shape = match kind {
                Kind::First => "node-first",
                Kind::Seq => "node-seq",
                Kind::If(_) => "node-ask",
                Kind::Do(_) => "node-step",
            };
            items.push(json!({"kind": shape, "tone": tone, "text": label(kind), "left": strings::css_pct(left), "top": strings::css_pct(top), "w": strings::css_pct(w), "h": strings::css_pct(h)}));
            // lines: a choice drops to a bar over its options; a sequence
            // runs a line down its left side to each step
            let line = |l: i32, t: i32, w: i32, h: i32| json!({"kind": "edge", "left": strings::css_pct(l), "top": strings::css_pct(t), "w": strings::css_pct(w.max(3)), "h": strings::css_pct(h.max(4))});
            if kids.is_empty() {
                continue;
            }
            if *kind == Kind::First {
                let bar = top + h + (rh - h) / 2 - rh / 16;
                let xs: Vec<i32> = kids
                    .iter()
                    .map(|&k| at[k].0 * cw + cw / 20 + indent(k) + cw * 4 / 10)
                    .collect();
                let (lo, hi) = (
                    xs.iter().copied().min().unwrap_or(0),
                    xs.iter().copied().max().unwrap_or(0),
                );
                items.push(line(left + w / 2, top + h, 3, bar - top - h));
                items.push(line(lo, bar, hi - lo + 3, 4));
                for x in xs {
                    items.push(line(x, bar, 3, rh / 4));
                }
            } else {
                let x = left + cw / 30;
                let last = kids.iter().map(|&k| at[k].1).max().unwrap_or(r0);
                items.push(line(x, top + h, 3, (last - r0) * rh - h / 2 + rh / 8));
                for &k in kids {
                    let ky = at[k].1 * rh + rh / 8 + h / 2;
                    items.push(line(x, ky, cw / 10 - cw / 30, 4));
                }
            }
        }
        json!({"t": "board", "aspect": format!("{} / {}", cols * 16, rows * 6), "items": items})
    }

    /// Office hours (T15, T32), after its mockup: the class on the left,
    /// the student in the middle, where they stand and what they cast on
    /// the right.
    fn ui_office(&self, p: &prep::Prep) -> Value {
        let s = &self.s;
        let c = &self.c;
        let b = c.balance();
        let pick = self.office_student(p);
        let pid = pick.map(|x| x.id);
        let mut bl = vec![
            json!({"t": "title", "col": "roster", "text": s.t("station.office")}),
            json!({"t": "p", "col": "roster", "tone": "help", "text": s.t("office.lead")}),
        ];
        if p.students.is_empty() {
            bl.push(json!({"t": "p", "col": "roster", "text": s.t("office.no_students")}));
            return json!({"id": "station", "kind": "office", "blocks": bl});
        }
        bl.push(json!({"t": "h", "col": "roster", "text": s.f("office.class", &[("n", p.students.len().to_string())])}));
        let rows: Vec<Value> = p.students.iter().map(|st| self.student_row(st, pid)).collect();
        bl.push(json!({"t": "rows", "col": "roster", "rows": rows}));
        let Some(st) = pick else {
            return json!({"id": "station", "kind": "office", "blocks": bl});
        };
        let id = st.id.to_string();
        // the student
        let full = |v: i32, hi: i32| strings::css_pct((v * 1000 / hi.max(1)).clamp(0, 1000));
        let ground = self
            .seated(p)
            .get(p.students.iter().position(|x| x.id == st.id).unwrap_or(0))
            .copied()
            == Some(battle::GROUND_SEAT);
        let range = if ground {
            b.ground_range_pct
        } else {
            b.seat_range_pct
        };
        bl.push(json!({"t": "title", "col": "who", "text": st.name}));
        bl.push(json!({"t": "p", "col": "who", "tone": "muted", "text": s.f("office.about", &[("good", c.file.schools[st.school as usize].name.clone()), ("bad", c.file.schools[st.weak as usize].name.clone()), ("q", st.quality.to_string()), ("slots", prep::slots(c, st).to_string())])}));
        let temper = c.file.temperaments.get(st.temper as usize);
        bl.push(json!({"t": "chips", "col": "who", "chips": [{"text": temper.map(|t| t.name.clone()).unwrap_or_default(), "tone": "gold"}]}));
        // health and speed from summed effectiveness (T42)
        let sum: i32 = st.known.iter().map(|k| k.1.max(0)).sum();
        let (hp, speed) = self.mage_numbers(st);
        let most_hp = b.mage_hp + b.mage_hp_per_100 * 5;
        bl.push(json!({"t": "h", "col": "who", "text": s.f("office.summed", &[("n", sum.to_string())])}));
        bl.push(json!({"t": "meters", "col": "who", "rows": [
            {"label": s.t("office.health"), "value": hp.to_string(), "left": "0%", "width": full(hp, most_hp), "tone": "good"},
            {"label": s.t("office.speed"), "value": s.f("office.speed_value", &[("v", speed)]), "left": "0%", "width": full(b.mage_speed_cms + sum * b.mage_speed_per_100_cms / 100, b.mage_speed_cms + b.mage_speed_per_100_cms * 5), "tone": "mana"},
        ]}));
        bl.push(json!({"t": "p", "col": "who", "tone": "help", "text": s.f("office.summed_explain", &[("hp", b.mage_hp.to_string()), ("per", b.mage_hp_per_100.to_string()), ("speed", (b.mage_speed_cms / 100).to_string()), ("per_speed", format!("{}.{}", b.mage_speed_per_100_cms / 100, b.mage_speed_per_100_cms % 100 / 10))])}));
        bl.push(json!({"t": "h", "col": "who", "text": s.t("office.stats")}));
        bl.push(json!({"t": "meters", "col": "who", "rows": [
            {"label": s.t("office.mana"), "value": st.mana_max.to_string(), "left": "0%", "width": full(st.mana_max, b.student_mana[1]), "tone": "mana"},
            {"label": s.t("office.regen"), "value": s.f("office.per_s_short", &[("n", st.regen.to_string())]), "left": "0%", "width": full(st.regen, b.student_regen[1]), "tone": "good"},
            {"label": s.t("office.range"), "value": format!("{range}%"), "left": "0%", "width": full(range, 100), "tone": "gold"},
        ]}));
        bl.push(json!({"t": "h", "col": "who", "text": s.t("office.proficiency")}));
        // a leaning fills from the middle: right for more, left for less
        let most = b.school_mod[0].abs().max(b.school_mod[1].abs()).max(1);
        let leans: Vec<Value> = c
            .file
            .schools
            .iter()
            .enumerate()
            .map(|(k, school)| {
                let m = st.mods.get(k).copied().unwrap_or(0);
                let w = m.abs() * 500 / most;
                let left = if m < 0 { 500 - w } else { 500 };
                json!({"label": school.name, "value": format!("{}{m}%", if m > 0 { "+" } else { "" }), "left": strings::css_pct(left), "width": strings::css_pct(w.max(8)), "tone": school.id, "centre": true})
            })
            .collect();
        bl.push(json!({"t": "meters", "col": "who", "rows": leans}));
        bl.push(json!({"t": "h", "col": "who", "text": s.t("office.spells_heading")}));
        let spells: Vec<Value> = st
            .known
            .iter()
            .filter_map(|&(k, eff)| {
                let ch = p.spells.get(k as usize)?;
                Some(json!([{"b": spell::name(c, ch)}, {"text": format!("{eff}%")}, {"mana": mana(s, spell::stats(c, ch).mana)}]))
            })
            .collect();
        if spells.is_empty() {
            bl.push(json!({"t": "p", "col": "who", "tone": "muted", "text": s.t("office.no_spells")}));
        } else {
            bl.push(json!({"t": "table", "col": "who", "head": [s.t("office.col_spell"), s.t("office.col_eff"), s.t("office.col_cast")], "rows": spells}));
        }
        // the right column: seats and casting, or how they decide (T41)
        let tabs: Vec<Value> = [("seats", "office.tab_seats"), ("mind", "office.tab_mind")]
            .iter()
            .map(|(k, label)| {
                let mut a = act_json("office.tab", k, s.t(label), true);
                a["on"] = json!(self.office_tab == *k);
                a
            })
            .collect();
        bl.push(json!({"t": "actions", "col": "deploy", "style": "tabs", "actions": tabs}));
        if self.office_tab == "mind" {
            if let Some(t) = temper {
                bl.push(json!({"t": "p", "col": "deploy", "text": format!("{}: {}", t.name, t.text)}));
            }
            bl.push(json!({"t": "chips", "col": "deploy", "chips": [{"text": s.t("bt.legend_ran"), "tone": "good"}, {"text": s.t("bt.legend_running"), "tone": "gold"}, {"text": s.t("bt.legend_failed"), "tone": "bad"}]}));
            let mut tree = self.tree_board(st.temper, &[]);
            tree["col"] = json!("deploy");
            bl.push(tree);
            bl.push(json!({"t": "p", "col": "deploy", "tone": "help", "text": s.t("bt.read")}));
            return json!({"id": "station", "kind": "office", "blocks": bl});
        }
        // where they stand
        bl.push(json!({"t": "h", "col": "deploy", "text": s.f("office.where_short", &[("name", st.name.clone())])}));
        bl.push(self.seat_board(p, pid));
        let elsewhere: Vec<Value> = [battle::AUTO_SEAT, battle::GROUND_SEAT]
            .iter()
            .map(|&v| {
                let mut a = act_json("office.seat", &format!("{id},{v}"), self.seat_name(v), true);
                a["on"] = json!(st.seat == v);
                a["drop"] = json!(true);
                a
            })
            .collect();
        bl.push(json!({"t": "actions", "col": "deploy", "style": "chips", "actions": elsewhere}));
        bl.push(json!({"t": "p", "col": "deploy", "tone": "help", "text": s.f("office.board_text", &[("n", b.tower_seats.to_string())])}));
        // what they cast
        bl.push(
            json!({"t": "h", "col": "deploy", "text": s.f("office.casts", &[("name", st.name.clone())])}),
        );
        let modes: Vec<Value> = [
            ("office.mode_rotation", battle::CAST_ROTATION),
            ("office.mode_best", battle::CAST_BEST),
            ("office.mode_one", battle::CAST_ONE),
        ]
        .iter()
        .map(|(k, v)| {
            let mut a = act_json("office.mode", &format!("{id},{v}"), s.t(k), true);
            a["on"] = json!(st.mode == *v);
            a
        })
        .collect();
        bl.push(json!({"t": "actions", "col": "deploy", "style": "seg", "actions": modes}));
        let steps: Vec<Value> = st
            .rotation
            .iter()
            .enumerate()
            .filter_map(|(i, &k)| {
                let ch = p.spells.get(k as usize)?;
                let mut a = act_json("office.rot_del", &format!("{id},{i}"), spell::name(c, ch), true);
                a["num"] = json!((i + 1).to_string());
                a["mana"] = mana(s, spell::stats(c, ch).mana);
                a["hint"] = json!(s.t("office.step_hint"));
                Some(a)
            })
            .collect();
        if st.mode != battle::CAST_BEST {
            bl.push(json!({"t": "actions", "col": "deploy", "style": "steps", "actions": steps}));
            let mut adds: Vec<Value> = st
                .known
                .iter()
                .filter_map(|&(k, _)| {
                    p.spells
                        .get(k as usize)
                        .filter(|ch| !spell::is_invasion(c, ch))
                        .map(|ch| {
                            act_json(
                                "office.rot_add",
                                &format!("{id},{k}"),
                                s.f("office.add", &[("spell", spell::name(c, ch))]),
                                st.rotation.len() < 8,
                            )
                        })
                })
                .collect();
            if !st.rotation.is_empty() {
                adds.push(act_json("office.rot_clear", &id, s.t("office.clear"), true));
            }
            bl.push(json!({"t": "actions", "col": "deploy", "style": "adds", "actions": adds}));
        }
        bl.push(
            json!({"t": "p", "col": "deploy", "tone": "small", "text": match st.mode {
                battle::CAST_ROTATION => s.t("office.mode_rotation_text"),
                battle::CAST_ONE => s.t("office.mode_one_text"),
                _ => s.t("office.mode_best_text"),
            }}),
        );
        json!({"id": "station", "kind": "office", "blocks": bl})
    }

    /// The map table's own panel (T23, T32, T33), after its mockup: a row
    /// for each thing to place, with its colour, mana and time to rise; or,
    /// seating, the class to drag onto the map's seats.
    fn ui_layout(&self, p: &prep::Prep) -> Value {
        let s = &self.s;
        let mut tabs = vec![
            act_json("layout.seats", "off", s.t("map.tab_build"), true),
            act_json("layout.seats", "on", s.t("map.tab_seats"), true),
        ];
        tabs[if self.layout.seats { 1 } else { 0 }]["on"] = json!(true);
        let mut bl = vec![
            json!({"t": "title", "col": "map", "text": s.t("station.map")}),
            json!({"t": "actions", "col": "map", "style": "seg", "actions": tabs}),
        ];
        if self.layout.seats {
            bl.push(json!({"t": "p", "col": "map", "tone": "help", "text": s.t("map.seats_how")}));
            let pick = self.office_pick;
            let rows: Vec<Value> = p.students.iter().map(|st| self.student_row(st, pick)).collect();
            if rows.is_empty() {
                bl.push(json!({"t": "p", "col": "map", "tone": "muted", "text": s.t("office.no_students")}));
            } else {
                bl.push(json!({"t": "rows", "col": "map", "rows": rows}));
            }
            let id = pick.map(|x| x.to_string()).unwrap_or_default();
            let elsewhere: Vec<Value> = [battle::AUTO_SEAT, battle::GROUND_SEAT]
                .iter()
                .map(|&v| {
                    let mut a = act_json(
                        "office.seat",
                        &format!("{id},{v}"),
                        self.seat_name(v),
                        pick.is_some(),
                    );
                    a["drop"] = json!(true);
                    a
                })
                .collect();
            bl.push(json!({"t": "actions", "col": "map", "style": "chips", "actions": elsewhere}));
        } else {
            let rows: Vec<Value> = self
                .placeable()
                .into_iter()
                .map(|f| {
                    let ft = &self.c.file.features[f];
                    let chip = if self.layout.golem { s.f("map.golem_chip", &[("s", ft.golem_s.to_string())]) } else { s.f("map.rise_chip", &[("s", strings::seconds(ft.raise_ms))]) };
                    let cost = (!self.layout.golem).then(|| mana(s, ft.mana));
                    let on = self.layout.feature == Some(f);
                    json!({"label": ft.name, "swatch": overlay::swatch(&ft.id), "mana": cost, "chip": chip, "on": on, "press": act_json("layout.feature", &ft.id, ft.name.clone(), true)})
                })
                .collect();
            if rows.is_empty() {
                bl.push(json!({"t": "p", "col": "map", "text": s.t("map.none_known")}));
            } else {
                bl.push(json!({"t": "p", "col": "map", "tone": "help", "text": s.t("map.how")}));
                bl.push(json!({"t": "rows", "col": "map", "rows": rows}));
            }
            bl.push(json!({"t": "actions", "col": "map", "actions": [
                act_json("layout.undo", "", s.t("map.undo"), !p.laid.is_empty()),
                act_json("layout.clear", "", s.t("map.clear"), !p.laid.is_empty()),
                act_json("layout.mode", if self.layout.golem { "mana" } else { "golem" }, s.t(if self.layout.golem { "map.use_mana" } else { "map.use_golem" }), true),
            ]}));
        }
        bl.push(json!({"t": "actions", "col": "map", "style": "soft", "actions": [act_json("layout.close", "", s.t("map.close"), true)]}));
        json!({"id": "layout", "kind": "map", "blocks": bl})
    }

    /// Sealing (T22, T32), after its mockup: the words and the list of
    /// rings in the panel, the dial drawn over it by the overlay.
    fn ui_seal(&self, p: &prep::Prep, rings: &[i32], at: usize) -> Value {
        let s = &self.s;
        let c = &self.c;
        let rows: Vec<Value> = rings
            .iter()
            .enumerate()
            .map(|(k, _)| {
                let piece = p.draft.get(k).copied().unwrap_or(0) as usize;
                let chip = if k < at { "seal.sealed" } else if k == at { "seal.turning" } else { "seal.next" };
                json!({"label": s.f("seal.ring", &[("n", (k + 1).to_string()), ("piece", c.file.pieces[piece].name.clone())]), "chip": s.t(chip), "school": c.file.schools[c.piece_school[piece]].id, "on": k == at, "chip_tone": if k < at { "good" } else { "" }, "dim": k > at})
            })
            .collect();
        let bl = vec![
            json!({"t": "title", "col": "head", "text": s.t("seal.title")}),
            json!({"t": "p", "col": "head", "tone": "help", "text": s.t("seal.text")}),
            json!({"t": "rows", "col": "side", "rows": rows}),
            json!({"t": "keys", "col": "side", "parts": [
                {"key": s.t("key.left")}, {"text": " "}, {"key": s.t("key.right")}, {"text": s.t("seal.keys_turn")},
                {"key": s.t("key.space")}, {"text": s.t("seal.keys_lock")}, {"key": s.t("key.esc")}, {"text": s.t("seal.keys_stop")},
            ]}),
            json!({"t": "p", "col": "side", "text": s.t("seal.explain")}),
            json!({"t": "note", "col": "side", "lead": s.t("seal.never_lead"), "text": s.t("seal.never")}),
            json!({"t": "actions", "col": "side", "style": "soft", "actions": [act_json("prep.cancel", "", s.t("busy.cancel"), true)]}),
        ];
        json!({"id": "busy", "kind": "seal", "box": overlay::css_box(overlay::SEAL_BOX), "blocks": bl})
    }

    /// The book pile (T39), after its mockup: books on the floor to drag
    /// to their school's shelf, the count, and whether the sort is still
    /// perfect. Nothing falls.
    fn ui_sort(&self, books: &[u8], held: Option<usize>, per: &[u8], misses: i32) -> Value {
        let s = &self.s;
        let c = &self.c;
        let b = c.balance();
        let total = b.sort_books.max(1);
        let shelved = total - books.len() as i32;
        // the floor: a book's place comes from its number, so it does not jump
        let items: Vec<Value> = books
            .iter()
            .enumerate()
            .map(|(i, &k)| {
                let school = &c.file.schools[(k as usize).min(c.file.schools.len() - 1)];
                let (col, row) = (i as i32 % 4, i as i32 / 4);
                let mut a = act_json("sort.hold", &i.to_string(), String::new(), true);
                a["grab"] = act_json("sort.hold", &i.to_string(), String::new(), true);
                json!({"kind": "book", "tone": format!("school-{}", school.id), "text": school.name, "action": a, "on": held == Some(i),
                    "left": strings::css_pct(20 + col * 245 + (i as i32 * 37) % 40), "top": strings::css_pct(30 + row * 320 + (i as i32 * 53) % 40),
                    "w": strings::css_pct(205), "h": strings::css_pct(250)})
            })
            .collect();
        let shelves: Vec<Value> = c
            .file
            .schools
            .iter()
            .enumerate()
            .map(|(k, school)| {
                let spines: Vec<String> = (0..per.get(k).copied().unwrap_or(0) as i32)
                    .map(|j| strings::css_pct(600 + (j * 7 + k as i32 * 3) % 4 * 100))
                    .collect();
                let mut a = act_json("sort.shelf", &k.to_string(), String::new(), true);
                a["drop"] = json!(true);
                json!({"action": a, "school": school.id, "label": school.name, "key": (k + 1).to_string(), "spines": spines, "on": false})
            })
            .collect();
        let (chip, tone) = if misses == 0 {
            (s.t("sort.perfect_yet"), "good")
        } else {
            (s.f("sort.missed", &[("n", misses.to_string())]), "bad")
        };
        let bl = vec![
            json!({"t": "title", "col": "head", "text": s.t("sort.title")}),
            json!({"t": "p", "col": "head", "tone": "help", "text": s.t("sort.text")}),
            json!({"t": "board", "col": "pile", "aspect": "14 / 5", "items": items}),
            json!({"t": "progress", "col": "found", "label": s.t("sort.shelving"), "value": s.f("sort.count", &[("n", shelved.to_string()), ("of", total.to_string())]), "width": strings::css_pct((shelved * 1000 / total).clamp(0, 1000))}),
            json!({"t": "chips", "col": "found", "chips": [{"text": chip, "tone": tone}]}),
            json!({"t": "p", "col": "found", "tone": "help", "text": s.t("sort.explain")}),
            json!({"t": "actions", "col": "found", "style": "soft", "actions": [act_json("prep.cancel", "", s.t("busy.cancel"), true)]}),
            json!({"t": "shelves", "col": "shelves", "shelves": shelves}),
        ];
        json!({"id": "busy", "kind": "sort", "box": overlay::css_box(overlay::SORT_BOX), "blocks": bl})
    }

    /// A raider's journey (T45): who is going where, how far they are, and
    /// how late the falls have made them. The run is the overlay's.
    fn ui_journey(
        &self,
        m: &Match,
        p: &prep::Prep,
        student: u16,
        target: u8,
        ledge: usize,
        falls: i32,
    ) -> Value {
        let s = &self.s;
        let name = p
            .students
            .iter()
            .find(|st| st.id == student)
            .map(|st| st.name.clone())
            .unwrap_or_default();
        let rival = m
            .seats
            .get(target as usize)
            .map(|x| x.name.clone())
            .unwrap_or_default();
        let late = falls.min(prep::LATE_FALLS) * self.c.balance().journey_late_s;
        let gaps = (prep::LEDGES.len() - 1) as i32;
        let bl = vec![
            json!({"t": "title", "col": "head", "text": s.f("journey.title", &[("name", name), ("rival", rival)])}),
            json!({"t": "p", "col": "head", "tone": "help", "text": s.t("journey.text")}),
            json!({"t": "chips", "col": "head", "chips": [
                {"text": s.f("journey.jumps", &[("n", (ledge as i32).min(gaps).to_string()), ("of", gaps.to_string())]), "tone": ""},
                {"text": s.f("journey.falls", &[("n", falls.to_string()), ("s", late.to_string())]), "tone": if falls > 0 { "bad" } else { "good" }},
            ]}),
            json!({"t": "keys", "col": "head", "parts": [{"key": s.t("key.space")}, {"text": s.t("journey.keys")}]}),
        ];
        json!({"id": "busy", "kind": "journey", "box": overlay::css_box(overlay::JOURNEY_BOX), "blocks": bl})
    }

    /// Beside the sorting game: the golem that searches instead (T21).
    fn ui_golem_card(&self, p: &prep::Prep) -> Value {
        let s = &self.s;
        let c = &self.c;
        let Some(h) = c.helper("librarian_golem") else {
            return json!({"id": "golem", "kind": "golem-card"});
        };
        let helper = &c.file.helpers[h];
        let busy = p.jobs.iter().any(|j| matches!(j.work, prep::Work::Search));
        json!({"id": "golem", "kind": "golem-card", "blocks": [
            {"t": "title", "col": "golem", "text": s.t("sort.golem_heading")},
            {"t": "p", "col": "golem", "text": s.f("sort.golem_text", &[("s", helper.job_s.to_string())])},
            {"t": "mana", "col": "golem", "mana": mana(s, helper.mana)},
            {"t": "actions", "col": "golem", "actions": [act_json("library.golem", "", s.t(if busy { "library.golem_busy" } else { "sort.golem_wake" }), !busy)]},
        ]})
    }

    /// The route beside the map (T23, T32): how long it is against the bare
    /// field's, and what is still rising. It changes as things rise, so it
    /// holds no buttons (T9).
    fn ui_route(&self, p: &prep::Prep) -> Value {
        let s = &self.s;
        let route = p.field(&self.c).route().len() as i32;
        let bare = (crate::battle::field(p.field_seed).route().len() as i32).max(1);
        let times = route * 10 / bare;
        let cells: usize = p.placed.iter().chain(p.rising()).map(|q| q.cells.len()).sum();
        let mut bl = vec![
            json!({"t": "title", "col": "route", "text": s.t("map.route_heading")}),
            json!({"t": "big", "col": "route", "text": s.f("map.route_cells", &[("n", route.to_string())]), "small": s.f("map.route_was", &[("n", bare.to_string())])}),
            json!({"t": "p", "col": "route", "text": s.f("map.route_far", &[("x", format!("{}.{}", times / 10, times % 10))])}),
            json!({"t": "p", "col": "route", "text": s.f("map.route_placed", &[("n", cells.to_string())])}),
        ];
        // rising cells by feature: how many, and the seconds until the last stands
        let mut groups: Vec<(u16, i32, i32)> = Vec::new();
        for j in &p.jobs {
            if let prep::Work::Raise(pl) | prep::Work::Build(pl) = &j.work {
                let left_s = (j.left + TPS - 1) / TPS;
                match groups.iter_mut().find(|g| g.0 == pl.feature) {
                    Some(g) => {
                        g.1 += 1;
                        g.2 = g.2.max(left_s);
                    }
                    None => groups.push((pl.feature, 1, left_s)),
                }
            }
        }
        if !groups.is_empty() {
            bl.push(json!({"t": "h", "col": "route", "text": s.t("jobs.heading")}));
            let rows: Vec<Value> = groups
                .iter()
                .map(|&(f, n, left)| json!({"label": s.f("map.rising_cells", &[("n", n.to_string()), ("name", self.c.file.features[f as usize].name.clone()), ("s", left.to_string())]), "swatch": overlay::swatch(&self.c.file.features[f as usize].id)}))
                .collect();
            bl.push(json!({"t": "rows", "col": "route", "rows": rows}));
        }
        bl.push(json!({"t": "p", "col": "route", "tone": "help", "text": s.t(if self.layout.seats { "map.seats_note" } else { "map.route_note" })}));
        json!({"id": "route", "kind": "route", "blocks": bl})
    }

    /// The raiders on your field and your raiders away (T44).
    fn ui_raiders(&self, m: &Match, b: &crate::battle::Battle) -> Value {
        let s = &self.s;
        let local = m.local as usize;
        let mut rows = Vec::new();
        for (k, mage) in b.casters.iter().enumerate() {
            if mage.team != crate::battle::Team::Foe {
                continue;
            }
            let rival = m
                .seats
                .get(mage.from as usize)
                .map(|x| x.name.clone())
                .unwrap_or_default();
            let detail = if mage.fell > 0 {
                s.t("raid.fell")
            } else if b.tick < mage.arrive {
                s.t("raid.coming")
            } else {
                format!("{} / {}", mage.hp, mage.hp_max)
            };
            let _ = k;
            rows.push(json!({"label": s.f("raid.raider_row", &[("name", mage.plan.name.clone()), ("rival", rival)]), "detail": detail, "bar": strings::css_pct(mage.hp * 1000 / mage.hp_max.max(1)), "school": "foe"}));
        }
        let mut away = Vec::new();
        for (t, other) in m.battles.iter().enumerate() {
            let Some(other) = other else { continue };
            if t == local {
                continue;
            }
            for mage in other
                .casters
                .iter()
                .filter(|x| x.team == crate::battle::Team::Foe && x.from as usize == local)
            {
                let state = if mage.fell > 0 {
                    s.t("raid.fell")
                } else {
                    format!("{} / {}", mage.hp, mage.hp_max)
                };
                away.push(json!({"label": s.f("raid.away_row", &[("name", mage.plan.name.clone()), ("rival", m.seats[t].name.clone())]), "detail": state}));
            }
        }
        let mut bl = vec![json!({"t": "h", "col": "raiders", "text": s.t("raid.on_field")})];
        if rows.is_empty() {
            bl.push(json!({"t": "p", "col": "raiders", "tone": "small", "text": s.t("raid.none_here")}));
        } else {
            bl.push(json!({"t": "rows", "col": "raiders", "rows": rows}));
        }
        if !away.is_empty() {
            bl.push(json!({"t": "h", "col": "raiders", "text": s.t("raid.yours_away")}));
            bl.push(json!({"t": "rows", "col": "raiders", "rows": away}));
        }
        json!({"id": "raiders", "kind": "raiders", "blocks": bl})
    }

    /// One of your students' trees, lit as it decides (T41).
    fn ui_mind(&self, b: &crate::battle::Battle) -> Option<Value> {
        use crate::mind::Kind;
        let s = &self.s;
        let mine: Vec<usize> = (0..b.casters.len())
            .filter(|&k| b.casters[k].team == crate::battle::Team::Ally)
            .collect();
        if mine.is_empty() {
            return None;
        }
        let k = mine[self.watch % mine.len()];
        let mage = &b.casters[k];
        let t = mage.plan.temper as usize % self.c.minds.len().max(1);
        let tree = self.c.minds.get(t)?;
        let temper = self
            .c
            .file
            .temperaments
            .get(t)
            .map(|x| x.name.clone())
            .unwrap_or_default();
        // the tree read top to bottom, each node with its depth and answer
        let mut depth = vec![0usize; tree.nodes.len()];
        for (n, (_, kids)) in tree.nodes.iter().enumerate() {
            for &c in kids {
                depth[c] = depth[n] + 1;
            }
        }
        let lines: Vec<Value> = tree
            .nodes
            .iter()
            .enumerate()
            .map(|(n, (kind, _))| {
                let text = match kind {
                    Kind::First => s.t("bt.first"),
                    Kind::Seq => s.t("bt.seq"),
                    Kind::If(q) => s.t(&format!(
                        "bt.if.{}",
                        serde_json::to_value(q)
                            .ok()
                            .and_then(|v| v.as_str().map(|x| x.to_string()))
                            .unwrap_or_default()
                    )),
                    Kind::Do(a) => s.t(&format!(
                        "bt.do.{}",
                        serde_json::to_value(a)
                            .ok()
                            .and_then(|v| v.as_str().map(|x| x.to_string()))
                            .unwrap_or_default()
                    )),
                };
                let (mark, tone) = match mage.lit.get(n).copied().unwrap_or(0) {
                    crate::mind::RAN_DONE => (s.t("bt.mark_ran"), "ran"),
                    crate::mind::RAN_RUNNING => (s.t("bt.mark_running"), "running"),
                    crate::mind::RAN_FAILED => (s.t("bt.mark_failed"), "failed"),
                    _ => (String::new(), ""),
                };
                json!({"text": text, "mark": mark, "tone": tone, "pad": format!("{}em", depth[n])})
            })
            .collect();
        let state = if mage.fell > 0 {
            s.t("raid.fell")
        } else {
            s.f(
                "bt.watch_stats",
                &[("hp", mage.hp.to_string()), ("of", mage.hp_max.to_string())],
            )
        };
        Some(json!({"id": "mind", "kind": "mind", "blocks": [
            {"t": "h", "col": "mind", "text": s.f("bt.watching", &[("name", mage.plan.name.clone()), ("temper", temper)])},
            {"t": "actions", "col": "mind", "style": "soft", "actions": [act_json("watch.prev", "", s.t("bt.prev"), mine.len() > 1), act_json("watch.next", "", s.t("bt.next"), mine.len() > 1)]},
            {"t": "lines", "col": "mind", "lines": lines},
            {"t": "p", "col": "mind", "tone": "small", "text": state},
        ]}))
    }

    fn ui_battle(&self, m: &Match) -> Value {
        let s = &self.s;
        let shown = self.shown(m);
        let mut panels = Vec::new();
        let local = m.local as usize;
        let tick = m.battles.iter().flatten().map(|b| b.tick).max().unwrap_or(0);
        // the clock counts up: a battle lasts until its enemies are gone (T54)
        panels.push(json!({"id": "hud", "kind": "hud", "items": [json!({"label": s.f("hud.battle", &[("day", (m.day + 1).to_string())]), "value": strings::clock(tick), "kind": "timer"})]}));
        let vps = self.viewports(shown.len());
        let (w, h) = (self.canvas.0.max(1), self.canvas.1.max(1));
        let mut labels = Vec::new();
        for (vp, &seat) in vps.iter().zip(shown.iter()) {
            let Some(Some(b)) = m.battles.get(seat) else {
                continue;
            };
            let left = vp.0 * 1000 / w;
            let top = (h - vp.1 - vp.3) * 1000 / h;
            let score = m.seats[seat].score - b.tally.damage_taken;
            let bottom = top + vp.3 * 1000 / h - 70;
            labels.push(json!({"text": format!("{} · {}", m.seats[seat].name, strings::number(score)), "left": strings::css_pct(left + 10), "top": strings::css_pct(bottom), "kind": "seat"}));
            // damage numbers over the battle in view
            let cam = self.cams[seat.min(3)].camera();
            let aspect = (vp.2.max(1) as i64 * crate::fx::ONE as i64 / vp.3.max(1) as i64) as i32;
            let mtx = cam.matrix(aspect);
            for pop in b
                .pops
                .iter()
                .filter(|p| p.kind == crate::battle::PopKind::Number)
                .take(if shown.len() > 1 { 6 } else { 24 })
            {
                if let Some((x, y)) = geom::project(&mtx, pop.pos.add(v3(0, crate::fx::ONE, 0))) {
                    let gx = left + x * vp.2 / w;
                    let gy = top + y * vp.3 / h;
                    let kind = if pop.rate > 100 {
                        "dmg strong"
                    } else if pop.rate < 100 {
                        "dmg weak"
                    } else {
                        "dmg"
                    };
                    labels.push(json!({"text": pop.value.to_string(), "left": strings::css_pct(gx), "top": strings::css_pct(gy), "kind": kind}));
                }
            }
            // raiders on this field, named (T44), each a line above the last
            let mut named = 0;
            for (k, mage) in b.casters.iter().enumerate() {
                if mage.team != crate::battle::Team::Foe || !b.standing_mage(k) {
                    continue;
                }
                let lift = crate::fx::ONE * 4 + named * crate::fx::ONE * 3 / 2;
                named += 1;
                if let Some((x, y)) = geom::project(&mtx, mage.pos.add(v3(0, lift, 0))) {
                    let rival = m
                        .seats
                        .get(mage.from as usize)
                        .map(|x| x.name.clone())
                        .unwrap_or_default();
                    labels.push(json!({"text": s.f("raid.label", &[("name", mage.plan.name.clone()), ("rival", rival)]), "left": strings::css_pct(left + x * vp.2 / w), "top": strings::css_pct(top + y * vp.3 / h), "kind": "foe"}));
                }
            }
        }
        if let Some(Some(b)) = m.battles.get(local) {
            panels.push(self.ui_raiders(m, b));
            if let Some(mind) = self.ui_mind(b) {
                panels.push(mind);
            }
        }
        let mut actions = vec![act_json(
            "view.focus",
            "all",
            s.t("view.all"),
            self.view != View::All,
        )];
        for (i, seat) in m.seats.iter().enumerate() {
            if seat.active() && m.battles[i].is_some() {
                actions.push(act_json(
                    "view.focus",
                    &i.to_string(),
                    seat.name.clone(),
                    self.view != View::Seat(i as u8),
                ));
            }
        }
        actions.push(act_json(
            "view.close",
            "",
            s.t(if self.close { "view.far" } else { "view.near" }),
            true,
        ));
        actions.push(act_json(
            "cam.reset",
            "",
            s.t("view.reset"),
            self.cams.iter().any(|o| *o != scene::Orbit::start(self.close)),
        ));
        for sp in [1, 2, 4] {
            actions.push(act_json(
                "battle.speed",
                &sp.to_string(),
                format!("{sp}×"),
                m.settings.speed != sp,
            ));
        }
        panels.push(json!({"id": "watch", "kind": "corner", "actions": actions}));
        panels.push(json!({"id": "view-hint", "kind": "hint", "text": s.t("view.hint")}));
        let _ = local;
        let mut v = self.base("play", s.t("game.name"), panels);
        v["phase"] = json!("battle");
        v["labels"] = json!(labels);
        v["data"] = json!({"tick": tick, "taken": m.battles.get(local).and_then(|b| b.as_ref()).map(|b| b.tally.damage_taken).unwrap_or(0)});
        v
    }

    fn day_rows(&self, m: &Match, day: i32) -> Vec<Value> {
        let s = &self.s;
        let n = m.seats.iter().filter(|x| x.active()).count();
        let from = (day as usize) * n;
        m.results
            .iter()
            .skip(from)
            .take(n)
            .map(|r| {
                json!({"label": m.seats[r.seat as usize].name, "detail": s.f("summary.row", &[("taken", strings::number(r.taken)), ("kills", r.kills.to_string()), ("frozen", r.frozen.to_string()), ("charmed", r.charmed.to_string()), ("warded", strings::number(r.warded)), ("dealt", strings::number(r.dealt))]), "chip": strings::number(m.seats[r.seat as usize].score)})
            })
            .collect()
    }

    fn ui_summary(&self, m: &Match) -> Value {
        let s = &self.s;
        let rows = self.day_rows(m, m.day);
        let last = m.day + 1 >= m.settings.days;
        let p = json!({"id": "summary", "kind": "center", "heading": s.f("summary.heading", &[("day", (m.day + 1).to_string())]), "text": s.t("summary.text"), "rows": rows, "actions": [act_json("day.next", "", s.t(if last { "summary.results" } else { "summary.next" }), true)]});
        let mut v = self.base("play", s.t("game.name"), vec![p, json!({"id": "watch", "kind": "corner", "actions": [act_json("match.leave", "", s.t("leave"), true)]})]);
        v["phase"] = json!("summary");
        v
    }

    fn ui_results(&self, m: &Match) -> Value {
        let s = &self.s;
        if let Some((k, value, medal, new_best)) = self.trial_result {
            let t = &self.c.file.trials[k as usize];
            let unit = if t.measure == "time" {
                s.f("trial.seconds", &[("n", value.to_string())])
            } else {
                strings::number(value)
            };
            let medal_name = if medal == 0 {
                s.t("medal.none")
            } else {
                s.t(["medal.bronze", "medal.silver", "medal.gold", "medal.archmage"][(medal - 1) as usize])
            };
            let text = format!(
                "{} {}",
                s.f("trial.result", &[("value", unit), ("medal", medal_name)]),
                if new_best {
                    s.t("trial.new_best")
                } else {
                    String::new()
                }
            );
            let p = json!({"id": "results", "kind": "center", "heading": t.name, "text": text, "actions": [act_json("trial.start", &k.to_string(), s.t("trial.again"), true), act_json("screen.show", "trials", s.t("title.trials"), true)]});
            let mut v = self.base("play", s.t("game.name"), vec![p]);
            v["phase"] = json!("results");
            v["data"] = json!({"trial": value, "medal": medal});
            return v;
        }
        let rows: Vec<Value> = m
            .ranking()
            .into_iter()
            .enumerate()
            .map(|(place, i)| json!({"label": s.f("results.place", &[("n", (place + 1).to_string()), ("name", m.seats[i].name.clone())]), "detail": s.f("results.taken", &[("taken", strings::number(m.seats[i].taken.iter().sum()))]), "chip": strings::number(m.seats[i].score)}))
            .collect();
        let won = m.won(m.local as usize);
        let earned: Vec<Value> = self
            .earned_this_match
            .iter()
            .filter_map(|id| self.c.file.achievements.iter().find(|a| &a.id == id))
            .map(|a| row(a.name.clone(), a.text.clone()))
            .collect();
        let actions = if m.mode == Mode::Online {
            vec![act_json("match.leave", "", s.t("results.title"), true)]
        } else {
            vec![
                act_json("screen.show", "setup", s.t("results.again"), true),
                act_json("match.leave", "", s.t("results.title"), true),
            ]
        };
        let mut panels = vec![
            json!({"id": "results", "kind": "center", "heading": s.t(if won { "results.won" } else { "results.lost" }), "text": s.t("results.text"), "rows": rows, "actions": actions}),
        ];
        if !earned.is_empty() {
            panels.push(
                json!({"id": "earned", "kind": "side", "heading": s.t("results.earned"), "rows": earned}),
            );
        }
        let mut v = self.base("play", s.t("game.name"), panels);
        v["phase"] = json!("results");
        v["data"] = json!({"won": won, "score": m.seats[m.local as usize].score, "taken": m.seats[m.local as usize].taken.iter().sum::<i32>()});
        v
    }

    /// The battle the local seat fought today, for tests.
    pub fn local_battle(&self) -> Option<&Battle> {
        self.m
            .as_ref()
            .and_then(|m| m.battles.get(m.local as usize))
            .and_then(|b| b.as_ref())
    }

    /// The camera a single viewport would use, for tests and probes.
    pub fn camera(&self) -> Camera {
        let seat = match self.view {
            View::Seat(k) => k as usize,
            View::All => self.m.as_ref().map(|m| m.local as usize).unwrap_or(0),
        };
        self.cams[seat.min(3)].camera()
    }

    /// Moves the local archmage straight to a station: a probe and test aid,
    /// not reachable from the page (the shim does not export it).
    pub fn teleport(&mut self, st: Station) {
        if let Some(p) = self.m.as_mut().and_then(|m| m.local_prep_mut()) {
            let at = prep::station_pos(st);
            p.pos = at.add(
                crate::geom::V3::ZERO
                    .sub(at)
                    .flat()
                    .with_len(prep::PROP + prep::BODY + crate::fx::ONE / 10),
            );
        }
    }
}

#[cfg(test)]
mod tests;

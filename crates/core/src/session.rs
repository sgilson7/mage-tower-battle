//! The game as the page sees it: screens, the one door for every action
//! (`act`), fixed ticks from display frames, and what to draw and play
//! each frame. The page builds its panels from `ui()` without knowing what
//! any of them means (after Peter's Mandible).

use serde_json::{json, Value};

use crate::battle::Battle;
use crate::bot::{Style, STYLES};
use crate::content::{Content, PieceKind};
use crate::game::{self, Match, Mode, Phase, SeatKind, Settings};
use crate::geom::{self, v3, TPS};
use crate::glossary;
use crate::look::Look;
use crate::online::{self, Event, Msg, Online, Role};
use crate::overlay;
use crate::prep::{self, Activity, Draw, Station};
use crate::render::{Batch, Camera, FrameOut, StaticScene};
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
    /// Each battle view's own camera (T5), and the drag moving one.
    pub cams: [scene::Orbit; 4],
    drag: Option<Drag>,
    build: String,
    /// Messages held for a later day or phase.
    net_queue: Vec<Event>,
    sums: Vec<(i32, u8, u64)>,
    desync_told: bool,
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

fn act_json(name: &str, value: &str, label: String, enabled: bool) -> Value {
    json!({"name": name, "value": value, "label": label, "enabled": enabled})
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
        save_text: &str,
        seed: u64,
    ) -> Result<Session, String> {
        let build = online::build_of(&[content_json, strings_json]);
        let s = Strings::parse(strings_json)?;
        let c = Content::parse(content_json).map_err(|e| e.message())?;
        let look = Look::parse(palette_json, models_json)?;
        let recipes = sound::parse(sounds_json)?;
        let entries = glossary::parse(glossary_md).map_err(|e| e.message())?;
        let b = c.balance();
        let fill = [
            ("battle.length_s", b.battle_s.to_string()),
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
            cams: [scene::Orbit::start(false); 4],
            drag: None,
            build,
            net_queue: Vec::new(),
            sums: Vec::new(),
            desync_told: false,
        };
        session.online_name = session.s.t("online.default_name");
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
                    other => return Err(self.s.f("error.no_screen", &[("name", other.to_string())])),
                };
                if self.screen != Screen::Play {
                    self.m = None;
                }
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
                self.m = Some(Match::trial(&self.c, k));
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
            "glossary.toggle" => {
                self.glossary = match self.glossary {
                    Some(_) => None,
                    None => Some("pieces".into()),
                }
            }
            "glossary.close" => self.glossary = None,
            "glossary.tab" => {
                if !["pieces", "spells", "terms", "how"].contains(&value) {
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
        if n.len() != 4 {
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
        if self.layout.open {
            let (lx, ly) = overlay::logical(n[0], n[1], n[2], n[3]);
            self.layout.hover = overlay::map_cell(lx, ly);
            if name == "pointer.down" {
                if let (Some((x, z)), Some(f)) = (self.layout.hover, self.layout.feature) {
                    let id = self.c.file.features[f].id.clone();
                    let v = format!("{id},{x},{z},{}", if self.layout.along_z { 1 } else { 0 });
                    let act = if self.layout.golem {
                        "layout.golem"
                    } else {
                        "layout.place"
                    };
                    self.forward(act, &v)?;
                    self.play("place");
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

    fn begin_play(&mut self) {
        self.screen = Screen::Play;
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
        self.pump_online();
        if self.glossary_pauses() {
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
        if let Some(p) = m.local_prep_mut() {
            notes = std::mem::take(&mut p.notes);
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
            if n.key != "draw.score" {
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
            let id = self.c.file.trials[k as usize].id.clone();
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
        let Some(m) = self.m.as_ref() else {
            self.frame = f;
            return;
        };
        let ink = self.look.colour("ink");
        match m.phase {
            Phase::Prep | Phase::Waiting => {
                let key = (0, m.day, 0);
                if self.static_key != key {
                    self.statics = StaticScene::default();
                    let room = scene::room_static(&self.look);
                    self.statics.add(&room);
                    self.statics.version += 1 + self.tick as u32;
                    self.static_key = key;
                }
                if let Some(p) = m.local_prep() {
                    let cam = scene::room_camera(p);
                    let (x, y, w, h) = (0, 0, self.canvas.0, self.canvas.1);
                    f.viewport(x, y, w, h, &cam, self.look.colour("sky_room"));
                    f.static_draws(&self.statics.parts[0]);
                    let mut b = Batch::new(ink);
                    b.eye = Some((cam.eye, ONE_M));
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
                                self.statics.add(&Batch::new(ink));
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
                    f.viewport(vp.0, vp.1, vp.2, vp.3, &cam, self.look.colour("sky_field"));
                    if let Some(Some(bt)) = m.battles.get(seat) {
                        if let Some(part) = self.statics.parts.get(seat) {
                            f.static_draws(part);
                        }
                        let mut b = Batch::new(ink);
                        b.eye = Some((cam.eye, ONE_M));
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
            _ if self.layout.open => {
                let a = p.field(&self.c);
                let ghost = match (self.layout.hover, self.layout.feature) {
                    (Some((x, z)), Some(f)) => {
                        let n = self.c.file.features[f].length.max(1);
                        let cells = (0..n)
                            .map(|k| {
                                if self.layout.along_z {
                                    (x, z + k)
                                } else {
                                    (x + k, z)
                                }
                            })
                            .collect();
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
        let p = json!({"id": "title", "kind": "center", "heading": s.t("game.name"), "text": s.t("title.blurb"), "actions": actions});
        self.base("title", s.t("game.name"), vec![p])
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

    fn ui_trials(&self) -> Value {
        let s = &self.s;
        let medals = ["medal.bronze", "medal.silver", "medal.gold", "medal.archmage"];
        let mut panels = Vec::new();
        for (k, t) in self.c.file.trials.iter().enumerate() {
            let best = self.save.bests.get(&t.id).copied();
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
            panels.push(json!({"id": format!("trial-{k}"), "kind": "card", "heading": t.name, "text": format!("{} {}", t.text, best_text), "rows": rows, "actions": [act_json("trial.start", &k.to_string(), s.t("trial.start"), true)]}));
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
            (
                "raise_max",
                self.c
                    .file
                    .features
                    .iter()
                    .map(|f| f.raise_s)
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
        let tabs: Vec<Value> = ["pieces", "spells", "terms", "how"]
            .iter()
            .map(|t| act_json("glossary.tab", t, s.t(&format!("gloss.tab.{t}")), *t != tab))
            .collect();
        let secs = |ms: i32| format!("{}.{}", ms / 1000, ms % 1000 / 100);
        let (text, rows): (String, Vec<Value>) = match tab {
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
                                row(
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
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let text = match (prep, rows.is_empty()) {
                    (None, _) => s.t("gloss.no_match"),
                    (Some(_), true) => s.t("gloss.no_spells"),
                    _ => s.t("gloss.spells_text"),
                };
                (text, rows)
            }
            "terms" => (
                s.t("gloss.terms_text"),
                self.hints
                    .iter()
                    .map(|(t, d)| row(t.clone(), d.clone()))
                    .collect(),
            ),
            "how" => (s.t("gloss.how_text"), self.help_rows()),
            _ => {
                let rows = c
                    .file
                    .pieces
                    .iter()
                    .enumerate()
                    .map(|(i, piece)| {
                        let kind = match piece.kind {
                            PieceKind::Payload { .. } => "kind.payload",
                            PieceKind::Modifier { .. } => "kind.modifier",
                            PieceKind::Formation { .. } => "kind.formation",
                            PieceKind::Invade => "kind.invade",
                            PieceKind::Trigger { .. } => "kind.trigger",
                            PieceKind::Helper { .. } => "kind.helper",
                        };
                        let school = &c.file.schools[c.piece_school[i]];
                        let mut lines = vec![s.f("gloss.kind_line", &[("school", school.name.clone()), ("kind", s.t(kind))]), piece.text.clone()];
                        match &piece.kind {
                            PieceKind::Payload { .. } => match spell::raises(c, i as u16) {
                                Some(f) => {
                                    let f = &c.file.features[f];
                                    lines.push(s.f("gloss.raises", &[("mana", f.mana.to_string()), ("s", f.raise_s.to_string())]));
                                }
                                None => {
                                    let st = spell::stats(c, &[i as u16]);
                                    lines.push(s.f("gloss.payload", &[("power", st.power.to_string()), ("cooldown", secs(st.cooldown_ms)), ("mana", st.mana.to_string())]));
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
                                .map(|(k, v)| s.f(k, &[("v", if *v > 0 && *k != "gloss.mod.pierce" { format!("+{v}") } else { v.to_string() })]))
                                .collect();
                                lines.push(s.f("gloss.mods", &[("list", list.join(", "))]));
                            }
                            PieceKind::Formation { copies, share_pct, .. } => {
                                lines.push(s.f("gloss.formation", &[("copies", copies.to_string()), ("share", share_pct.to_string())]));
                            }
                            PieceKind::Helper { helper } => {
                                if let Some(h) = c.helper(helper) {
                                    let h = &c.file.helpers[h];
                                    lines.push(s.f("gloss.helper", &[("mana", h.mana.to_string()), ("s", h.job_s.to_string())]));
                                }
                            }
                            PieceKind::Invade | PieceKind::Trigger { .. } => {}
                        }
                        let chip = match prep {
                            Some(p) if p.known[i] => s.t("gloss.known"),
                            Some(_) => s.t("gloss.unknown"),
                            None => String::new(),
                        };
                        json!({"label": s.f("gloss.entry", &[("n", (i + 1).to_string()), ("name", piece.name.clone())]), "detail": lines.join(" "), "chip": chip, "school": school.id})
                    })
                    .collect();
                (s.t("gloss.pieces_text"), rows)
            }
        };
        let paused = self.glossary_pauses();
        let text = if paused {
            format!("{text} {}", s.t("gloss.paused"))
        } else {
            text
        };
        json!({"id": "glossary", "kind": "glossary", "heading": s.t("gloss.heading"), "text": text, "saved_actions": tabs, "rows": rows, "actions": [act_json("glossary.close", "", s.t("gloss.close"), true)]})
    }

    /// A game against the computer waits while the glossary is open; an
    /// online one cannot, since the other players' days go on.
    fn glossary_pauses(&self) -> bool {
        self.glossary.is_some() && self.m.as_ref().map(|m| m.mode != Mode::Online).unwrap_or(false)
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

    fn hud(&self, m: &Match) -> Value {
        let s = &self.s;
        let p = m.local_prep();
        let mana = p
            .map(|p| p.mana * 1000 / (self.c.balance().archmage_mana * 100).max(1))
            .unwrap_or(0);
        let clock = match m.mode {
            Mode::Trial(0) | Mode::Trial(1) => strings::clock(m.elapsed),
            _ => strings::clock(p.map(|p| p.clock).unwrap_or(0)),
        };
        let items = vec![
            json!({"label": s.f("hud.day", &[("day", (m.day + 1).to_string()), ("days", m.settings.days.to_string())]), "value": clock, "kind": "timer"}),
            json!({"label": s.t("hud.score"), "value": strings::number(m.seats[m.local as usize].score)}),
            json!({"label": s.t("hud.students"), "value": p.map(|p| p.students.len()).unwrap_or(0).to_string()}),
            json!({"label": s.t("hud.pages"), "value": p.map(|p| p.known.iter().filter(|&&k| k).count()).unwrap_or(0).to_string()}),
            json!({"label": s.t("hud.mana"), "value": strings::number(p.map(|p| p.mana / 100).unwrap_or(0)), "bar": strings::css_pct(mana)}),
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
        };
        if let Some(b) = busy {
            let text = match b {
                "reading" => s.t("busy.reading"),
                "interview" => s.t("busy.interview"),
                _ => s.t("busy.drawing"),
            };
            panels.push(json!({"id": "busy", "kind": "bottom", "text": text, "actions": [act_json("prep.cancel", "", s.t("busy.cancel"), true)]}));
        } else if self.layout.open {
            panels.push(self.ui_layout(p));
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
                .map(|j| {
                    // a wall placed with mana rises by itself, with no helper (T1)
                    let label = match &j.work {
                        prep::Work::Raise(pl) => s.f("map.rising_short", &[("name", self.c.file.features[pl.feature as usize].name.clone())]),
                        _ => self.c.file.helpers[j.helper as usize].name.clone(),
                    };
                    json!({"label": label, "detail": strings::clock(j.left), "bar": strings::css_pct(1000 - j.left * 1000 / j.total.max(1))})
                })
                .collect();
            // beside the station's panel, not on top of it
            panels.push(json!({"id": "jobs", "kind": "jobs", "heading": s.t("jobs.heading"), "rows": rows}));
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
        v["data"] = json!({"students": p.students.len(), "spells": p.spells.len(), "known": p.known.iter().filter(|&&k| k).count(), "placed": p.placed.len(), "x": dm(p.pos.x), "z": dm(p.pos.z)});
        v
    }

    fn station_labels(&self, p: &prep::Prep) -> Vec<Value> {
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
                json!({"id": "station", "kind": "side", "heading": s.t("station.library"), "text": format!("{} {}", s.f("library.left", &[("n", left.to_string())]), last), "actions": [act_json("library.read", "", s.t("library.read"), left > 0)]})
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
                    };
                    rows.push(json!({"label": piece.name, "detail": piece.text, "chip": s.t(kind), "school": c.file.schools[c.piece_school[i]].id, "actions": [act_json("lab.add", &piece.id, s.t("lab.add"), true)]}));
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
                    s.t("lab.draft_empty")
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
                let saved: Vec<Value> = p.spells.iter().enumerate().map(|(k, ch)| json!({"label": spell::name(c, ch), "detail": s.f("lab.spell_mana", &[("mana", spell::stats(c, ch).mana.to_string())]), "chip": if k as u16 == p.chosen { s.t("lab.chosen") } else { String::new() }, "actions": [act_json("spell.choose", &k.to_string(), s.t("lab.choose"), true)]})).collect();
                json!({"id": "station", "kind": "side", "heading": s.t("station.lab"), "text": stats, "draft": draft, "rows": rows, "saved": saved, "actions": [act_json("lab.save", "", s.t("lab.save"), !p.draft.is_empty()), act_json("lab.clear", "", s.t("lab.clear"), !p.draft.is_empty())]})
            }
            Station::Recruit => {
                let rows: Vec<Value> = p
                    .applicants
                    .iter()
                    .enumerate()
                    .map(|(k, a)| json!({"label": a.name, "detail": s.f("recruit.leans", &[("good", c.file.schools[a.school as usize].name.clone()), ("bad", c.file.schools[a.weak as usize].name.clone())]), "school": c.file.schools[a.school as usize].id, "actions": [act_json("recruit.one", &k.to_string(), s.t("recruit.interview"), true)]}))
                    .collect();
                let mut actions = vec![act_json(
                    "recruit.group",
                    "",
                    s.t("recruit.group"),
                    p.applicants.len() > 1,
                )];
                if let Some(h) = c.piece("recruitment_elemental").filter(|&h| p.known[h]) {
                    actions.push(act_json(
                        "podium.summon",
                        &c.file.pieces[h].id,
                        s.t("recruit.summon"),
                        true,
                    ));
                }
                json!({"id": "station", "kind": "side", "heading": s.t("station.recruit"), "text": s.f("recruit.text", &[("n", p.applicants.len().to_string())]), "rows": rows, "actions": actions})
            }
            Station::Podium => {
                let spell = p
                    .spells
                    .get(p.chosen as usize)
                    .map(|ch| spell::name(c, ch))
                    .unwrap_or_default();
                let mut actions = vec![act_json(
                    "podium.teach",
                    "all",
                    s.f("podium.teach_all", &[("n", p.students.len().to_string())]),
                    !p.students.is_empty() && !p.spells.is_empty(),
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
                        let eff = st.known.iter().find(|k| k.0 == p.chosen).map(|k| format!("{}%", k.1)).unwrap_or_else(|| s.t("podium.not_taught"));
                        json!({"label": st.name, "detail": eff, "school": c.file.schools[st.school as usize].id, "actions": [act_json("podium.teach", &st.id.to_string(), s.t("podium.teach_one"), true)]})
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
                json!({"id": "station", "kind": "side", "heading": s.t("station.podium"), "text": s.f("podium.text", &[("spell", spell)]), "saved_actions": saved, "rows": rows, "actions": actions})
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
                for (k, ch) in p.spells.iter().enumerate() {
                    if !spell::is_invasion(c, ch) {
                        continue;
                    }
                    for (t, seat) in m.seats.iter().enumerate() {
                        if t == m.local as usize || !seat.active() {
                            continue;
                        }
                        actions.push(act_json(
                            "book.cast",
                            &format!("{k},{t}"),
                            s.f(
                                "book.cast",
                                &[("spell", spell::name(c, ch)), ("who", seat.name.clone())],
                            ),
                            true,
                        ));
                    }
                }
                json!({"id": "station", "kind": "side", "heading": s.t("station.book"), "text": s.t("book.text"), "rows": rows, "actions": actions})
            }
            Station::Map => {
                json!({"id": "station", "kind": "side", "heading": s.t("station.map"), "text": s.f("map.text", &[("n", p.placed.len().to_string())]), "actions": [act_json("layout.open", "", s.t("map.open"), true)]})
            }
        }
    }

    fn ui_layout(&self, p: &prep::Prep) -> Value {
        let s = &self.s;
        let rows: Vec<Value> = self
            .placeable()
            .into_iter()
            .map(|f| {
                let ft = &self.c.file.features[f];
                let detail = if self.layout.golem { s.f("map.golem_cost", &[("s", ft.golem_s.to_string())]) } else { s.f("map.mana_cost", &[("mana", ft.mana.to_string()), ("s", ft.raise_s.to_string())]) };
                json!({"label": ft.name, "detail": detail, "chip": if self.layout.feature == Some(f) { s.t("map.selected") } else { String::new() }, "actions": [act_json("layout.feature", &ft.id, s.t("map.pick"), true)]})
            })
            .collect();
        let text = if rows.is_empty() {
            s.t("map.none_known")
        } else {
            s.t("map.how")
        };
        let placed: Vec<Value> = p
            .placed
            .iter()
            .enumerate()
            .map(|(k, pl)| {
                act_json(
                    "layout.remove",
                    &k.to_string(),
                    s.f(
                        "map.remove",
                        &[("name", self.c.file.features[pl.feature as usize].name.clone())],
                    ),
                    true,
                )
            })
            .collect();
        let rising: Vec<Value> = p
            .jobs
            .iter()
            .filter_map(|j| match &j.work {
                prep::Work::Raise(pl) | prep::Work::Build(pl) => Some(json!({"label": s.f("map.rising", &[("name", self.c.file.features[pl.feature as usize].name.clone()), ("s", ((j.left + TPS - 1) / TPS).to_string())]), "bar": strings::css_pct((j.total - j.left) * 1000 / j.total.max(1))})),
                _ => None,
            })
            .collect();
        json!({"id": "layout", "kind": "map", "heading": s.t("station.map"), "text": text, "rows": rows, "saved": rising, "saved_actions": placed, "actions": [act_json("layout.turn", "", s.t("map.turn"), true), act_json("layout.mode", if self.layout.golem { "mana" } else { "golem" }, s.t(if self.layout.golem { "map.use_mana" } else { "map.use_golem" }), true), act_json("layout.close", "", s.t("map.close"), true)]})
    }

    fn ui_battle(&self, m: &Match) -> Value {
        let s = &self.s;
        let shown = self.shown(m);
        let mut panels = Vec::new();
        let local = m.local as usize;
        let tick = m.battles.iter().flatten().map(|b| b.tick).max().unwrap_or(0);
        let ticks = m.settings.battle_s * TPS;
        panels.push(json!({"id": "hud", "kind": "hud", "items": [json!({"label": s.f("hud.battle", &[("day", (m.day + 1).to_string())]), "value": strings::clock(ticks - tick), "kind": "timer"})]}));
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
    /// not reachable from the page.
    #[cfg(test)]
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

/// Within a metre of the eye, shapes get no ink.
const ONE_M: crate::fx::Fx = crate::fx::ONE;

#[cfg(test)]
mod tests;

//! Online play for four (PLAN.md decision 15), on the online kit's
//! pattern (kits/online): the page's transport connects browsers and
//! carries text; this module decides everything about the room.
//!
//! The host is the hub of a star: joiners talk only to the host, and the
//! host passes on what each sends. A battle takes no input, so there is no
//! rollback and no lockstep: each player prepares alone, an archmage's
//! invasion is passed on the moment it is cast, and when a preparation
//! ends its plan is sent. Every browser then runs all four battles from
//! the same seed and plans, and compares a checksum when they end.
//!
//! Rules kept from the kit's games:
//! - versions meet, then part with a reason: the room is named by the
//!   protocol, and the hello turns a different build away (`online.reload`);
//! - garbage changes the status and nothing else (`online.garbled`);
//! - a message from a browser that never said hello is ignored.

use serde::{Deserialize, Serialize};

use crate::battle::Plan;
use crate::content::Content;
use crate::game::Settings;
use crate::spell;

pub const PROTOCOL: u32 = 1;
pub const SEATS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum Msg {
    Hello {
        protocol: u32,
        build: String,
        name: String,
    },
    Welcome {
        seat: u8,
    },
    Turned {
        reason: String,
    },
    Lobby {
        seats: Vec<LobbySeat>,
    },
    Ready {
        ready: bool,
        name: String,
    },
    Start {
        seed: u64,
        settings: Settings,
        kinds: Vec<String>,
        names: Vec<String>,
    },
    Invade {
        day: i32,
        from: u8,
        seq: u32,
        target: u8,
        chain: Vec<u16>,
    },
    Plan {
        day: i32,
        seat: u8,
        plan: Plan,
        students: Vec<(Vec<u16>, i32)>,
    },
    Sum {
        day: i32,
        seat: u8,
        hash: u64,
    },
    Left {
        seat: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LobbySeat {
    /// "host", "person" or "open" (a computer rival when the match starts).
    pub kind: String,
    pub name: String,
    pub ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Host,
    Joiner,
}

/// Messages the match must act on, in the order they arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Start {
        seed: u64,
        settings: Settings,
        kinds: Vec<String>,
        names: Vec<String>,
    },
    Invade {
        day: i32,
        from: u8,
        seq: u32,
        target: u8,
        chain: Vec<u16>,
    },
    Plan {
        day: i32,
        seat: u8,
        plan: Plan,
        students: Vec<(Vec<u16>, i32)>,
    },
    Sum {
        day: i32,
        seat: u8,
        hash: u64,
    },
    Left {
        seat: u8,
    },
}

#[derive(Debug, Clone)]
pub struct Online {
    pub role: Role,
    pub room: String,
    pub build: String,
    pub name: String,
    pub seat: u8,
    /// (transport peer id, seat); a joiner's one peer is the host, seat 0.
    pub peers: Vec<(u32, u8)>,
    /// Peers connected but not yet said hello.
    pub waiting: Vec<u32>,
    pub lobby: Vec<LobbySeat>,
    pub ready: bool,
    pub started: bool,
    pub status: String,
    pub outbox: Vec<(u32, String)>,
    pub events: Vec<Event>,
    pub seq: u32,
}

/// Five letters with no I, O, 0 or 1, from two random words the page drew.
pub fn make_code(a: u32, b: u32) -> String {
    const LETTERS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut v = (a as u64) << 32 | b as u64;
    (0..5)
        .map(|_| {
            let c = LETTERS[(v % LETTERS.len() as u64) as usize] as char;
            v /= LETTERS.len() as u64;
            c
        })
        .collect()
}

/// A typed code, cleaned: letters and digits only, upper case, 4 to 8 long.
pub fn clean_code(s: &str) -> Option<String> {
    let c: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    (4..=8).contains(&c.len()).then_some(c)
}

/// A typed name: printable, trimmed, at most 16 characters.
pub fn clean_name(s: &str) -> String {
    let n: String = s.chars().filter(|c| !c.is_control()).collect();
    let n: String = n.trim().chars().take(16).collect();
    n.trim_end().to_string()
}

/// The room's name on the relays: the app, the protocol and the code, never
/// the build, so deploys do not split rooms (Ribbit King's lesson).
pub fn room_name(app: &str, code: &str) -> String {
    format!("{app}-p{PROTOCOL}-{code}")
}

/// A build's name for the hello: a checksum of the rules and words it ships
/// with, so two browsers that would play different games never meet.
pub fn build_of(texts: &[&str]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for t in texts {
        for b in t.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

/// A chain from another browser: a spell this build could have made.
pub fn sane_chain(c: &Content, chain: &[u16]) -> bool {
    chain.len() <= 24
        && chain.iter().all(|&k| (k as usize) < c.file.pieces.len())
        && spell::check(c, chain).is_ok()
}

/// A plan from another browser: every index in range, every chain a spell.
pub fn sane_plan(c: &Content, plan: &Plan, students: &[(Vec<u16>, i32)]) -> bool {
    let n = plan.spells.len();
    n <= 64
        && plan.spells.iter().all(|ch| sane_chain(c, ch))
        && plan.students.len() <= 256
        && plan.students.iter().all(|s| {
            s.spells
                .iter()
                .all(|&(k, e)| (k as usize) < n && (0..=400).contains(&e))
                && s.rotation.len() <= 8
                && s.rotation.iter().all(|&k| (k as usize) < n)
                && (0..=1_000_000).contains(&s.mana_max)
                && (0..=1_000_000).contains(&s.regen)
        })
        && plan.placed.len() <= 1024
        && plan.placed.iter().all(|p| {
            (p.feature as usize) < c.file.features.len()
                && !p.cells.is_empty()
                && p.cells.len() <= 16
                && p.cells.iter().all(|&(x, z)| crate::arena::inside(x, z))
        })
        && students.len() <= 256
        && students
            .iter()
            .all(|(ch, e)| sane_chain(c, ch) && (0..=400).contains(e))
}

impl Online {
    pub fn host(name: &str, build: &str, code: String) -> Online {
        let mut lobby = vec![
            LobbySeat {
                kind: "open".into(),
                name: String::new(),
                ready: false
            };
            SEATS
        ];
        lobby[0] = LobbySeat {
            kind: "host".into(),
            name: clean_name(name),
            ready: true,
        };
        Online {
            role: Role::Host,
            room: code,
            build: build.into(),
            name: clean_name(name),
            seat: 0,
            peers: Vec::new(),
            waiting: Vec::new(),
            lobby,
            ready: true,
            started: false,
            status: "online.waiting".into(),
            outbox: Vec::new(),
            events: Vec::new(),
            seq: 0,
        }
    }

    pub fn join(name: &str, build: &str, code: String) -> Online {
        let mut o = Online::host(name, build, code);
        o.role = Role::Joiner;
        o.seat = 255;
        o.ready = false;
        o.status = "online.joining".into();
        o
    }

    fn send(&mut self, peer: u32, m: &Msg) {
        if let Ok(t) = serde_json::to_string(m) {
            self.outbox.push((peer, t));
        }
    }

    /// To every joiner (host) or to the host (joiner).
    pub fn broadcast(&mut self, m: &Msg) {
        let peers: Vec<u32> = self.peers.iter().map(|p| p.0).collect();
        for p in peers {
            self.send(p, m);
        }
    }

    fn broadcast_except(&mut self, m: &Msg, not: u32) {
        let peers: Vec<u32> = self.peers.iter().map(|p| p.0).filter(|&p| p != not).collect();
        for p in peers {
            self.send(p, m);
        }
    }

    fn seat_of(&self, peer: u32) -> Option<u8> {
        self.peers.iter().find(|p| p.0 == peer).map(|p| p.1)
    }

    pub fn connected(&mut self, peer: u32) {
        match self.role {
            Role::Host => self.waiting.push(peer),
            Role::Joiner => {
                self.peers.push((peer, 0));
                let hello = Msg::Hello {
                    protocol: PROTOCOL,
                    build: self.build.clone(),
                    name: self.name.clone(),
                };
                self.send(peer, &hello);
                self.status = "online.waiting".into();
            }
        }
    }

    pub fn left(&mut self, peer: u32) {
        self.waiting.retain(|&p| p != peer);
        let Some(seat) = self.seat_of(peer) else { return };
        self.peers.retain(|p| p.0 != peer);
        match self.role {
            Role::Joiner => {
                self.status = "online.host_left".into();
                if self.started {
                    self.events.push(Event::Left { seat: 0 });
                }
            }
            Role::Host => {
                if self.started {
                    let m = Msg::Left { seat };
                    self.broadcast(&m);
                    self.events.push(Event::Left { seat });
                } else {
                    self.lobby[seat as usize] = LobbySeat {
                        kind: "open".into(),
                        name: String::new(),
                        ready: false,
                    };
                    self.share_lobby();
                }
                self.status = "online.left".into();
            }
        }
    }

    fn share_lobby(&mut self) {
        let m = Msg::Lobby {
            seats: self.lobby.clone(),
        };
        self.broadcast(&m);
    }

    pub fn message(&mut self, peer: u32, text: &str) {
        let Ok(m) = serde_json::from_str::<Msg>(text) else {
            self.status = "online.garbled".into();
            return;
        };
        let known = self.seat_of(peer);
        match (self.role, m) {
            (
                Role::Host,
                Msg::Hello {
                    protocol,
                    build,
                    name,
                },
            ) => {
                if !self.waiting.contains(&peer) {
                    return;
                }
                self.waiting.retain(|&p| p != peer);
                if protocol != PROTOCOL || build != self.build {
                    self.send(
                        peer,
                        &Msg::Turned {
                            reason: "online.reload".into(),
                        },
                    );
                    return;
                }
                let free = if self.started {
                    None
                } else {
                    self.lobby.iter().position(|s| s.kind == "open")
                };
                let Some(seat) = free else {
                    self.send(
                        peer,
                        &Msg::Turned {
                            reason: "online.full".into(),
                        },
                    );
                    return;
                };
                self.peers.push((peer, seat as u8));
                self.lobby[seat] = LobbySeat {
                    kind: "person".into(),
                    name: clean_name(&name),
                    ready: false,
                };
                self.send(peer, &Msg::Welcome { seat: seat as u8 });
                self.share_lobby();
                self.status = "online.waiting".into();
            }
            (_, _) if known.is_none() => {}
            (Role::Host, Msg::Ready { ready, name }) => {
                let s = known.unwrap_or(0) as usize;
                if !self.started {
                    self.lobby[s].ready = ready;
                    self.lobby[s].name = clean_name(&name);
                    self.share_lobby();
                }
            }
            (Role::Host, m @ (Msg::Invade { .. } | Msg::Plan { .. } | Msg::Sum { .. })) => {
                // a joiner speaks only for its own seat
                let seat = known.unwrap_or(255);
                let honest = match &m {
                    Msg::Invade { from, .. } => *from == seat,
                    Msg::Plan { seat: s, .. } | Msg::Sum { seat: s, .. } => *s == seat,
                    _ => false,
                };
                if !honest || !self.started {
                    self.status = "online.garbled".into();
                    return;
                }
                self.broadcast_except(&m, peer);
                self.take(m);
            }
            (Role::Joiner, Msg::Welcome { seat }) => {
                self.seat = seat;
                self.status = "online.in_lobby".into();
            }
            (Role::Joiner, Msg::Turned { reason }) => {
                self.status = if reason == "online.full" {
                    "online.full".into()
                } else {
                    "online.reload".into()
                };
            }
            (Role::Joiner, Msg::Lobby { seats }) => {
                if seats.len() == SEATS {
                    self.lobby = seats;
                }
            }
            (
                Role::Joiner,
                Msg::Start {
                    seed,
                    settings,
                    kinds,
                    names,
                },
            ) => {
                self.started = true;
                self.status = "online.playing".into();
                self.events.push(Event::Start {
                    seed,
                    settings,
                    kinds,
                    names,
                });
            }
            (
                Role::Joiner,
                m @ (Msg::Invade { .. } | Msg::Plan { .. } | Msg::Sum { .. } | Msg::Left { .. }),
            ) => self.take(m),
            _ => self.status = "online.garbled".into(),
        }
    }

    fn take(&mut self, m: Msg) {
        let e = match m {
            Msg::Invade {
                day,
                from,
                seq,
                target,
                chain,
            } => Event::Invade {
                day,
                from,
                seq,
                target,
                chain,
            },
            Msg::Plan {
                day,
                seat,
                plan,
                students,
            } => Event::Plan {
                day,
                seat,
                plan,
                students,
            },
            Msg::Sum { day, seat, hash } => Event::Sum { day, seat, hash },
            Msg::Left { seat } => Event::Left { seat },
            _ => return,
        };
        self.events.push(e);
    }

    pub fn set_ready(&mut self, ready: bool) {
        self.ready = ready;
        if self.role == Role::Joiner {
            let m = Msg::Ready {
                ready,
                name: self.name.clone(),
            };
            self.broadcast(&m);
        }
    }

    pub fn can_start(&self) -> bool {
        self.role == Role::Host && !self.started && self.lobby.iter().all(|s| s.kind != "person" || s.ready)
    }

    /// The host starts the match for everyone; the start reaches its own
    /// events too, so every browser starts the same way.
    pub fn start(&mut self, seed: u64, settings: Settings, bots: &[String; 3]) -> bool {
        if !self.can_start() {
            return false;
        }
        let mut b = 0;
        let mut kinds = Vec::new();
        let mut names = Vec::new();
        for s in &self.lobby {
            if s.kind == "open" {
                kinds.push(bots[b.min(2)].clone());
                names.push(String::new());
                b += 1;
            } else {
                kinds.push("person".to_string());
                names.push(s.name.clone());
            }
        }
        let m = Msg::Start {
            seed,
            settings: settings.clone(),
            kinds: kinds.clone(),
            names: names.clone(),
        };
        self.broadcast(&m);
        self.started = true;
        self.status = "online.playing".into();
        self.events.push(Event::Start {
            seed,
            settings,
            kinds,
            names,
        });
        true
    }

    /// Sends a message the match made (an invasion, a plan, a checksum).
    pub fn say(&mut self, m: Msg) {
        self.broadcast(&m);
    }

    pub fn take_outbox(&mut self) -> Vec<(u32, String)> {
        std::mem::take(&mut self.outbox)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Settings {
        Settings {
            days: 2,
            prep_s: 60,
            battle_s: 30,
            speed: 1,
            ragdoll_cap: 24,
            skill: 60,
            difficulty: 1,
        }
    }

    /// A wire between a host and joiners: delivers every outbox in order.
    fn pump(host: &mut Online, joiners: &mut [Online]) {
        for _ in 0..8 {
            for (peer, text) in host.take_outbox() {
                let j = &mut joiners[(peer - 1) as usize];
                j.message(100, &text);
            }
            for (k, j) in joiners.iter_mut().enumerate() {
                for (_, text) in j.take_outbox() {
                    host.message(k as u32 + 1, &text);
                }
            }
        }
    }

    fn room(n: usize) -> (Online, Vec<Online>) {
        let mut host = Online::host("Host", "b1", "ABCDE".into());
        let mut js: Vec<Online> = (0..n)
            .map(|k| Online::join(&format!("J{k}"), "b1", "ABCDE".into()))
            .collect();
        for (k, j) in js.iter_mut().enumerate() {
            host.connected(k as u32 + 1);
            j.connected(100);
        }
        pump(&mut host, &mut js);
        (host, js)
    }

    #[test]
    fn every_message_round_trips() {
        let msgs = vec![
            Msg::Hello {
                protocol: 1,
                build: "b".into(),
                name: "n".into(),
            },
            Msg::Welcome { seat: 2 },
            Msg::Turned {
                reason: "online.full".into(),
            },
            Msg::Ready {
                ready: true,
                name: "n".into(),
            },
            Msg::Invade {
                day: 1,
                from: 2,
                seq: 3,
                target: 0,
                chain: vec![1, 2],
            },
            Msg::Plan {
                day: 0,
                seat: 1,
                plan: Plan::default(),
                students: vec![(vec![4], 50)],
            },
            Msg::Sum {
                day: 0,
                seat: 3,
                hash: u64::MAX,
            },
            Msg::Left { seat: 2 },
        ];
        for m in msgs {
            let t = serde_json::to_string(&m).unwrap();
            assert_eq!(serde_json::from_str::<Msg>(&t).unwrap(), m);
        }
    }

    #[test]
    fn three_joiners_take_seats_one_to_three_and_a_fourth_is_turned_away() {
        let (mut host, js) = room(3);
        assert_eq!(js.iter().map(|j| j.seat).collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!(host.lobby.iter().filter(|s| s.kind == "person").count(), 3);
        assert!(
            js.iter().all(|j| j.lobby == host.lobby),
            "everyone sees the same lobby"
        );
        let mut late = Online::join("Late", "b1", "ABCDE".into());
        host.connected(9);
        late.connected(100);
        for (_, t) in late.take_outbox() {
            host.message(9, &t);
        }
        let turned: Vec<String> = host
            .take_outbox()
            .into_iter()
            .filter(|(p, _)| *p == 9)
            .map(|(_, t)| t)
            .collect();
        for t in turned {
            late.message(100, &t);
        }
        assert_eq!(late.status, "online.full");
    }

    #[test]
    fn another_build_is_turned_away_with_a_reason() {
        let mut host = Online::host("Host", "b1", "ABCDE".into());
        let mut j = Online::join("Old", "b0", "ABCDE".into());
        host.connected(1);
        j.connected(100);
        pump(&mut host, std::slice::from_mut(&mut j));
        assert_eq!(j.status, "online.reload");
        assert_eq!(j.seat, 255);
        assert!(host.lobby.iter().all(|s| s.kind != "person"));
    }

    #[test]
    fn the_host_starts_only_when_every_person_is_ready_and_everyone_gets_the_same_start() {
        let (mut host, mut js) = room(2);
        let bots = [
            "balanced".to_string(),
            "attack_mage".to_string(),
            "spellcrafter".to_string(),
        ];
        assert!(!host.start(7, settings(), &bots), "joiners are not ready");
        for j in js.iter_mut() {
            j.set_ready(true);
        }
        pump(&mut host, &mut js);
        assert!(host.start(7, settings(), &bots));
        pump(&mut host, &mut js);
        let start = host.events[0].clone();
        assert!(
            matches!(&start, Event::Start { kinds, .. } if kinds == &vec!["person", "person", "person", "balanced"])
        );
        for j in &js {
            assert_eq!(j.events[0], start);
        }
    }

    #[test]
    fn the_host_passes_an_invasion_on_to_every_other_joiner_in_order() {
        let (mut host, mut js) = room(3);
        for j in js.iter_mut() {
            j.set_ready(true);
        }
        pump(&mut host, &mut js);
        host.start(
            1,
            settings(),
            &["balanced".into(), "balanced".into(), "balanced".into()],
        );
        pump(&mut host, &mut js);
        let inv = Msg::Invade {
            day: 0,
            from: 2,
            seq: 1,
            target: 1,
            chain: vec![5, 6],
        };
        let plan = Msg::Plan {
            day: 0,
            seat: 2,
            plan: Plan::default(),
            students: vec![],
        };
        js[1].say(inv);
        js[1].say(plan);
        pump(&mut host, &mut js);
        for o in [&host, &js[0], &js[2]] {
            let tail: Vec<&Event> = o.events.iter().skip(1).collect();
            assert!(
                matches!(tail[0], Event::Invade { from: 2, seq: 1, .. }),
                "{tail:?}"
            );
            assert!(
                matches!(tail[1], Event::Plan { seat: 2, .. }),
                "the plan follows its invasion"
            );
        }
        assert_eq!(js[1].events.len(), 1, "the sender is not sent its own message");
    }

    #[test]
    fn garbage_and_a_joiner_speaking_for_another_seat_change_only_the_status() {
        let (mut host, mut js) = room(1);
        js[0].set_ready(true);
        pump(&mut host, &mut js);
        host.start(
            1,
            settings(),
            &["balanced".into(), "balanced".into(), "balanced".into()],
        );
        pump(&mut host, &mut js);
        let before = host.events.len();
        host.message(1, "not json");
        assert_eq!(host.status, "online.garbled");
        host.message(
            1,
            &serde_json::to_string(&Msg::Plan {
                day: 0,
                seat: 3,
                plan: Plan::default(),
                students: vec![],
            })
            .unwrap(),
        );
        assert_eq!(
            host.events.len(),
            before,
            "a plan for someone else's seat is refused"
        );
        host.message(
            77,
            &serde_json::to_string(&Msg::Sum {
                day: 0,
                seat: 1,
                hash: 1,
            })
            .unwrap(),
        );
        assert_eq!(host.events.len(), before, "a stranger is ignored");
    }

    #[test]
    fn a_joiner_leaving_frees_its_seat_in_the_lobby_and_is_reported_in_a_match() {
        let (mut host, mut js) = room(2);
        host.left(1);
        assert_eq!(host.lobby[1].kind, "open");
        pump(&mut host, &mut js);
        assert_eq!(js[1].lobby[1].kind, "open");
        js[1].set_ready(true);
        pump(&mut host, &mut js);
        host.start(
            3,
            settings(),
            &["balanced".into(), "balanced".into(), "balanced".into()],
        );
        host.left(2);
        assert!(matches!(host.events.last(), Some(Event::Left { seat: 2 })));
    }

    #[test]
    fn a_plan_from_another_browser_is_refused_when_an_index_is_out_of_range() {
        let c = crate::content::shipped();
        let mut p = crate::prep::Prep::new(&c, 0, 1, 1);
        p.start_day(&c, 600);
        let good = p.plan();
        assert!(sane_plan(&c, &good, &[]));
        let mut bad = good.clone();
        bad.students.push(crate::battle::StudentPlan {
            seat: crate::battle::AUTO_SEAT,
            mode: 0,
            rotation: Vec::new(),
            mana_max: 10,
            regen: 1,
            spells: vec![(9, 50)],
            school: 0,
        });
        assert!(
            !sane_plan(&c, &bad, &[]),
            "a student knows a spell the plan does not hold"
        );
        let mut bad = good.clone();
        bad.placed.push(crate::battle::Placed {
            feature: 999,
            cells: vec![(1, 1)],
        });
        assert!(!sane_plan(&c, &bad, &[]), "a feature this build does not have");
        let mut bad = good;
        bad.placed.push(crate::battle::Placed {
            feature: 0,
            cells: vec![(40, 1)],
        });
        assert!(!sane_plan(&c, &bad, &[]), "a cell off the field");
        assert!(!sane_chain(&c, &[9999]));
    }

    #[test]
    fn the_build_name_changes_with_any_text() {
        assert_eq!(build_of(&["a", "b"]), build_of(&["a", "b"]));
        assert_ne!(build_of(&["a", "b"]), build_of(&["a", "c"]));
        assert_eq!(build_of(&[""]).len(), 16);
    }

    #[test]
    fn codes_and_names_are_cleaned_and_the_room_carries_the_protocol() {
        assert_eq!(make_code(1, 2).len(), 5);
        for k in 0..400u32 {
            let code = make_code(k.wrapping_mul(2_654_435_761), k);
            assert!(!code.contains(['I', 'O', '0', '1']), "{code}");
        }
        assert_eq!(clean_code(" ab-c d1 "), Some("ABCD1".into()));
        assert_eq!(clean_code("ab"), None);
        assert_eq!(
            clean_name("  Wren\u{7}  Ashby the very long name  "),
            "Wren  Ashby the"
        );
        assert_eq!(room_name("app", "ABCDE"), "app-p1-ABCDE");
    }
}

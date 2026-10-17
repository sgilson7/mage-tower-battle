//! Behaviour trees: how a mage on the field decides what to do (T41). A
//! student's temperament names a tree in `data/content.json`; the battle
//! ticks it once a tick for every mage, at home or raiding. The trees are
//! read the way Vagrancy's BT lab reads them: a choice ("first that
//! works") tries its options in turn, a sequence ("in order") runs its
//! steps until one fails, a question checks the world, and a step acts.
//!
//! A step answers `Done`, `Running` (still under way, as a walk or a cast
//! waiting on its cooldown) or `Failed`. A choice stops at the first option
//! that does not fail; a sequence stops at the first step that does not
//! succeed. What each node answered this tick is kept, so the page can
//! light the tree as it runs.

use serde::{Deserialize, Serialize};

/// What a question asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cond {
    /// Health below two fifths.
    Hurt,
    /// A mage of the other side is on this field and standing.
    FoeMage,
    /// Something of the other side is within this mage's reach.
    FoeInRange,
    /// This mage is raiding a rival's field.
    Raiding,
    /// An enemy is within eight metres of the tower.
    FoeNearTower,
    /// This mage is on a seat of the tower.
    OnTower,
}

/// What a step does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Act {
    /// Back to the tower and up onto the seat; a raider backs off toward
    /// the entry.
    FallBack,
    /// Down from the seat to the ground beside it.
    LeaveTower,
    /// Toward the nearest mage of the other side, to casting range.
    ApproachMage,
    /// Cast at the nearest mage of the other side.
    CastMage,
    /// Cast at whatever of the other side is nearest.
    CastNearest,
    /// Cast at the enemy nearest the tower.
    CastNearTower,
    /// A raider casts at the tower itself.
    CastTower,
    /// A raider walks the route toward the tower.
    Advance,
    /// Stay put; a student off the tower goes back up to their seat.
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    First,
    Seq,
    If,
    Do,
}

/// A node as written in the content file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawNode {
    pub k: NodeKind,
    /// The question or the step, for `if` and `do`.
    #[serde(default)]
    pub cond: Option<Cond>,
    #[serde(default, rename = "do")]
    pub act: Option<Act>,
    #[serde(default)]
    pub c: Vec<RawNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    First,
    Seq,
    If(Cond),
    Do(Act),
}

/// A tree flattened in reading order: node 0 is the root, and each node
/// lists its children by index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub nodes: Vec<(Kind, Vec<usize>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Done,
    Running,
    Failed,
}

/// What a node answered on the last tick, for lighting the tree.
pub const NOT_RUN: u8 = 0;
pub const RAN_DONE: u8 = 1;
pub const RAN_RUNNING: u8 = 2;
pub const RAN_FAILED: u8 = 3;

impl Tree {
    /// Flattens a written tree, refusing a question without its question,
    /// a step without its step, a leaf with children or a branch without.
    pub fn build(raw: &RawNode) -> Result<Tree, String> {
        let mut t = Tree { nodes: Vec::new() };
        t.add(raw)?;
        Ok(t)
    }

    fn add(&mut self, raw: &RawNode) -> Result<usize, String> {
        let kind = match (raw.k, raw.cond, raw.act) {
            (NodeKind::First, None, None) => Kind::First,
            (NodeKind::Seq, None, None) => Kind::Seq,
            (NodeKind::If, Some(c), None) => Kind::If(c),
            (NodeKind::Do, None, Some(a)) => Kind::Do(a),
            _ => return Err("a node names the wrong thing for its kind".into()),
        };
        let leaf = matches!(kind, Kind::If(_) | Kind::Do(_));
        if leaf != raw.c.is_empty() {
            return Err("a question or a step has children, or a choice or a sequence has none".into());
        }
        let at = self.nodes.len();
        self.nodes.push((kind, Vec::new()));
        for k in &raw.c {
            let i = self.add(k)?;
            self.nodes[at].1.push(i);
        }
        Ok(at)
    }

    /// Ticks the tree once. `lit` gets what each node answered.
    pub fn tick(&self, w: &mut impl World, lit: &mut Vec<u8>) -> Status {
        lit.clear();
        lit.resize(self.nodes.len(), NOT_RUN);
        self.run(0, w, lit)
    }

    fn run(&self, n: usize, w: &mut impl World, lit: &mut [u8]) -> Status {
        let (kind, kids) = &self.nodes[n];
        let s = match kind {
            Kind::If(c) => {
                if w.ask(*c) {
                    Status::Done
                } else {
                    Status::Failed
                }
            }
            Kind::Do(a) => w.act(*a),
            Kind::First => {
                let mut s = Status::Failed;
                for &k in kids {
                    s = self.run(k, w, lit);
                    if s != Status::Failed {
                        break;
                    }
                }
                s
            }
            Kind::Seq => {
                let mut s = Status::Done;
                for &k in kids {
                    s = self.run(k, w, lit);
                    if s != Status::Done {
                        break;
                    }
                }
                s
            }
        };
        lit[n] = match s {
            Status::Done => RAN_DONE,
            Status::Running => RAN_RUNNING,
            Status::Failed => RAN_FAILED,
        };
        s
    }

    /// Where each node sits for drawing, in columns and rows, after the BT
    /// lab: a choice spreads its options across; a sequence stacks its
    /// steps under itself. Returns (column, row) by node, and the width in
    /// columns.
    pub fn layout(&self) -> (Vec<(i32, i32)>, i32) {
        let mut at = vec![(0, 0); self.nodes.len()];
        let w = self.lay(0, 0, 0, &mut at);
        (at, w)
    }

    fn lay(&self, n: usize, col: i32, row: i32, at: &mut [(i32, i32)]) -> i32 {
        let (kind, kids) = &self.nodes[n];
        at[n] = (col, row);
        if kids.is_empty() {
            return 1;
        }
        if *kind == Kind::First {
            let mut c = col;
            for &k in kids {
                c += self.lay(k, c, row + 1, at);
            }
            return (c - col).max(1);
        }
        let mut r = row + 1;
        let mut w = 1;
        for &k in kids {
            w = w.max(self.lay(k, col, r, at));
            r = self.deepest(k, at) + 1;
        }
        w
    }

    fn deepest(&self, n: usize, at: &[(i32, i32)]) -> i32 {
        let mut d = at[n].1;
        for &k in &self.nodes[n].1 {
            d = d.max(self.deepest(k, at));
        }
        d
    }
}

/// What a tree asks and does: the battle answers for one mage.
pub trait World {
    fn ask(&mut self, c: Cond) -> bool;
    fn act(&mut self, a: Act) -> Status;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        hurt: bool,
        walking: bool,
        did: Vec<Act>,
    }

    impl World for Fake {
        fn ask(&mut self, c: Cond) -> bool {
            c == Cond::Hurt && self.hurt
        }
        fn act(&mut self, a: Act) -> Status {
            self.did.push(a);
            if a == Act::FallBack && self.walking {
                Status::Running
            } else {
                Status::Done
            }
        }
    }

    fn tree(json: &str) -> Tree {
        Tree::build(&serde_json::from_str(json).unwrap()).unwrap()
    }

    const CAUTIOUS: &str = r#"{"k":"first","c":[
        {"k":"seq","c":[{"k":"if","cond":"hurt"},{"k":"do","do":"fall_back"}]},
        {"k":"do","do":"hold"}]}"#;

    #[test]
    fn a_choice_takes_its_first_option_that_does_not_fail_and_a_running_step_stops_it() {
        let t = tree(CAUTIOUS);
        let mut lit = Vec::new();
        // not hurt: the question fails, so the sequence fails, so hold runs
        let mut w = Fake {
            hurt: false,
            walking: false,
            did: vec![],
        };
        assert_eq!(t.tick(&mut w, &mut lit), Status::Done);
        assert_eq!(w.did, vec![Act::Hold]);
        // nodes in reading order: first, seq, if hurt, do fall back, do hold
        assert_eq!(lit, vec![RAN_DONE, RAN_FAILED, RAN_FAILED, NOT_RUN, RAN_DONE]);
        // hurt and still walking back: the sequence is running, hold never runs
        let mut w = Fake {
            hurt: true,
            walking: true,
            did: vec![],
        };
        assert_eq!(t.tick(&mut w, &mut lit), Status::Running);
        assert_eq!(w.did, vec![Act::FallBack]);
        assert_eq!(
            lit,
            vec![RAN_RUNNING, RAN_RUNNING, RAN_DONE, RAN_RUNNING, NOT_RUN]
        );
    }

    #[test]
    fn a_choice_spreads_its_options_across_and_a_sequence_stacks_its_steps() {
        let (at, w) = tree(CAUTIOUS).layout();
        // the choice on top; the sequence in column 0 with its two steps
        // under it; hold in column 1
        assert_eq!(at, vec![(0, 0), (0, 1), (0, 2), (0, 3), (1, 1)]);
        assert_eq!(w, 2);
    }

    #[test]
    fn a_node_that_names_the_wrong_thing_is_refused() {
        for bad in [
            r#"{"k":"if"}"#,
            r#"{"k":"do","do":"hold","c":[{"k":"do","do":"hold"}]}"#,
            r#"{"k":"seq"}"#,
            r#"{"k":"first","cond":"hurt","c":[{"k":"do","do":"hold"}]}"#,
        ] {
            assert!(Tree::build(&serde_json::from_str(bad).unwrap()).is_err(), "{bad}");
        }
    }
}

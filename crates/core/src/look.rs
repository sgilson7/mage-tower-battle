//! How things look: the colour palette (`data/palette.json`) and the
//! characters (`data/models.json`, written by `tools/models.py`), parsed
//! and checked by the core, and the posing that draws a character standing,
//! walking, casting or fallen as a ragdoll.

use serde::Deserialize;
use std::collections::BTreeMap;

use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, V3};
use crate::mesh::Mesh;
use crate::ragdoll::{self, Ragdoll};
use crate::render::{hex, Batch, Frame3, Pass, Rgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Node {
    Body,
    Head,
    HandL,
    HandR,
    FootL,
    FootR,
    /// An arm, turned about its shoulder (`Part::pivot`): it swings when
    /// walking and lifts to cast or read (T34).
    ArmL,
    ArmR,
    /// Held in the right hand: carried along as the arm turns, but kept
    /// upright, so a raised staff stands up rather than pointing back.
    GripR,
}

impl Node {
    fn from_name(s: &str) -> Option<Node> {
        Some(match s {
            "body" => Node::Body,
            "head" => Node::Head,
            "hand_l" => Node::HandL,
            "hand_r" => Node::HandR,
            "foot_l" => Node::FootL,
            "foot_r" => Node::FootR,
            "arm_l" => Node::ArmL,
            "arm_r" => Node::ArmR,
            "grip_r" => Node::GripR,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawPart {
    m: String,
    p: [i32; 3],
    s: [i32; 3],
    c: String,
    n: String,
    t: [i32; 2],
    far: bool,
    /// An arm part's shoulder, in cm.
    #[serde(default)]
    q: Option<[i32; 3]>,
    /// How much the part glows, in thousandths.
    #[serde(default)]
    g: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct RawModels {
    models: BTreeMap<String, Vec<RawPart>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub mesh: Mesh,
    pub at: V3,
    pub size: V3,
    pub role: String,
    pub node: Node,
    pub tilt: (i32, i32),
    pub far: bool,
    pub pivot: V3,
    pub glow: Fx,
}

#[derive(Debug, Clone, Default)]
pub struct Look {
    pub palette: BTreeMap<String, Rgb>,
    pub models: BTreeMap<String, Vec<Part>>,
}

/// A character's pose for one frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pose {
    pub yaw: i32,
    pub lean: (i32, i32),
    /// Walking phase in turns; 0 stands still.
    pub walk: i32,
    pub stride: Fx,
    /// 0..=ONE: how far the right hand is raised to cast.
    pub cast: Fx,
    pub scale: Fx,
    /// White-hot when struck.
    pub flash: bool,
    /// Drawn in ice.
    pub frozen: bool,
    /// Extra turn of each arm about its shoulder, in turns; negative lifts
    /// it forward (left, right).
    pub arms: (i32, i32),
    /// The head's nod: positive looks down.
    pub nod: i32,
    /// How far the body has sunk to sit, Q16 metres before scaling.
    pub sit: Fx,
    /// A slow rise and fall while standing, by phase; 0 is still.
    pub breath: i32,
}

/// How far an arm swings each way when walking, in turns.
pub const ARM_SWING: i32 = fx::TURN / 14;
/// How far the right arm lifts to cast: past level, toward overhead.
pub const ARM_CAST: i32 = fx::TURN * 3 / 8;

const MAGENTA: Rgb = [ONE, 0, ONE];

impl Look {
    pub fn parse(palette_json: &str, models_json: &str) -> Result<Look, String> {
        let raw: BTreeMap<String, serde_json::Value> =
            serde_json::from_str(palette_json).map_err(|e| format!("The palette could not be read: {e}"))?;
        let mut palette = BTreeMap::new();
        for (k, v) in raw {
            if k.starts_with('_') {
                continue;
            }
            let s = v
                .as_str()
                .ok_or_else(|| format!("The palette role \"{k}\" is not a colour."))?;
            palette.insert(k, hex(s));
        }
        let m: RawModels =
            serde_json::from_str(models_json).map_err(|e| format!("The models could not be read: {e}"))?;
        let mut models = BTreeMap::new();
        for (name, parts) in m.models {
            let mut out = Vec::new();
            for p in parts {
                let mesh = Mesh::from_name(&p.m).ok_or_else(|| {
                    format!("The model \"{name}\" uses a mesh \"{}\" there is none of.", p.m)
                })?;
                let node = Node::from_name(&p.n)
                    .ok_or_else(|| format!("The model \"{name}\" names a node \"{}\".", p.n))?;
                if !palette.contains_key(&p.c) {
                    return Err(format!(
                        "The model \"{name}\" uses a colour \"{}\" the palette does not have.",
                        p.c
                    ));
                }
                let cm = |v: i32| v * ONE / 100;
                out.push(Part {
                    mesh,
                    at: v3(cm(p.p[0]), cm(p.p[1]), cm(p.p[2])),
                    size: v3(cm(p.s[0]), cm(p.s[1]), cm(p.s[2])),
                    role: p.c,
                    node,
                    tilt: (fx::TURN * p.t[0] / 1000, fx::TURN * p.t[1] / 1000),
                    far: p.far,
                    pivot: p.q.map(|q| v3(cm(q[0]), cm(q[1]), cm(q[2]))).unwrap_or(V3::ZERO),
                    glow: p.g * ONE / 1000,
                });
            }
            models.insert(name, out);
        }
        Ok(Look { palette, models })
    }

    pub fn colour(&self, role: &str) -> Rgb {
        self.palette.get(role).copied().unwrap_or(MAGENTA)
    }

    /// The tallest point of a model, for scaling it to a height.
    pub fn height(&self, model: &str) -> Fx {
        self.models
            .get(model)
            .map(|ps| ps.iter().map(|p| p.at.y + p.size.y / 2).max().unwrap_or(ONE))
            .unwrap_or(ONE)
    }

    /// Draws a character standing at `base` (its feet) in a pose.
    /// `swap` replaces colour roles (a student's robe in its school's colour).
    pub fn draw(&self, b: &mut Batch, model: &str, base: V3, pose: &Pose, swap: &[(&str, Rgb)], far: bool) {
        let Some(parts) = self.models.get(model) else {
            return;
        };
        let k = if pose.scale == 0 { ONE } else { pose.scale };
        let swing = fx::sin(pose.walk);
        let bob = if pose.walk == 0 {
            fx::mul(fx::sin(pose.breath), ONE / 80)
        } else {
            fx::mul(fx::sin(pose.walk * 2).abs(), pose.stride / 6)
        };
        // how far the whole body rises with breath or step, less how far it
        // has sunk to sit
        let lift = bob - pose.sit;
        // arms: swing against the feet, the right lifts to cast
        let arm_l = -fx::mul(swing, ARM_SWING) + pose.arms.0;
        let arm_r = fx::mul(swing, ARM_SWING) - fx::mul(pose.cast, ARM_CAST) + pose.arms.1;
        let head_y = parts
            .iter()
            .filter(|p| p.node == Node::Head)
            .map(|p| p.at.y)
            .min()
            .unwrap_or(0);
        // where the right hand rests: the lowest part of the right arm
        let hand_r = parts
            .iter()
            .filter(|p| p.node == Node::ArmR)
            .min_by_key(|p| p.at.y)
            .map(|p| (p.at, p.pivot));
        for p in parts {
            if far && !p.far {
                continue;
            }
            let mut at = p.at;
            let mut turn = 0;
            match p.node {
                Node::FootL => at.z += fx::mul(swing, pose.stride) + pose.sit,
                Node::FootR => at.z -= fx::mul(swing, pose.stride) - pose.sit,
                Node::HandL => at.z -= fx::mul(swing, pose.stride / 2),
                Node::HandR => {
                    at.z += fx::mul(swing, pose.stride / 2);
                    at.y += fx::mul(pose.cast, ONE * 3 / 5);
                    at.z += fx::mul(pose.cast, ONE / 4);
                }
                Node::ArmL | Node::ArmR => {
                    turn = if p.node == Node::ArmL { arm_l } else { arm_r };
                    at = p.pivot.add(crate::render::rotate(at.sub(p.pivot), 0, turn, 0));
                    at.y += lift;
                }
                Node::Head => {
                    // a nod turns the head about its neck
                    let neck = v3(0, head_y - ONE * 15 / 100, 0);
                    at = neck.add(crate::render::rotate(at.sub(neck), 0, pose.nod, 0));
                    turn = pose.nod;
                    at.y += lift;
                }
                Node::GripR => {
                    if let Some((hand, pivot)) = hand_r {
                        let arm = hand.sub(pivot);
                        at = at.add(crate::render::rotate(arm, 0, arm_r, 0).sub(arm));
                    }
                    at.y += lift;
                }
                Node::Body => at.y += lift,
            }
            let at = crate::render::rotate(at.scale(k), pose.yaw, pose.lean.0, pose.lean.1);
            let f = Frame3::new(
                pose.yaw,
                pose.lean.0 + p.tilt.0 + turn,
                pose.lean.1 + p.tilt.1,
                p.size.scale(k),
            );
            let mut rgb = swap
                .iter()
                .find(|(r, _)| *r == p.role)
                .map(|(_, c)| *c)
                .unwrap_or_else(|| self.colour(&p.role));
            if pose.frozen {
                rgb = crate::render::mix(rgb, self.colour("ice"), ONE * 3 / 5);
            }
            if pose.flash {
                rgb = crate::render::mix(rgb, [ONE, ONE, ONE], ONE * 3 / 5);
            }
            let mesh = if far && p.mesh == Mesh::Sphere {
                Mesh::Blob
            } else {
                p.mesh
            };
            b.push(mesh, Pass::Solid, base.add(at), &f, rgb, p.glow);
            if p.glow > 0 && !far {
                let halo = Frame3::new(pose.yaw, 0, 0, p.size.scale(k).scale(ONE * 3));
                b.push(Mesh::Halo, Pass::Glow, base.add(at), &halo, rgb, p.glow / 3);
            }
        }
    }

    /// Draws a fallen character from its ragdoll's points: each part keeps
    /// its offset from its node's resting place.
    pub fn draw_ragdoll(&self, b: &mut Batch, model: &str, d: &Ragdoll, height: Fx, fade: Fx, far: bool) {
        let Some(parts) = self.models.get(model) else {
            return;
        };
        let mh = self.height(model).max(1);
        let k = (height as i64 * ONE as i64 / mh as i64) as Fx;
        let pt = |i: usize| d.points.get(i).map(|p| p.pos).unwrap_or(V3::ZERO);
        let rest = |n: Node| -> (V3, V3) {
            let h = |x: i32, y: i32| v3(fx::mul(mh, x * ONE / 100), fx::mul(mh, y * ONE / 100), 0);
            match n {
                Node::Body => (h(0, 57), pt(ragdoll::PELVIS).lerp(pt(ragdoll::CHEST), ONE / 2)),
                Node::Head => (h(0, 93), pt(ragdoll::HEAD)),
                Node::HandL | Node::ArmL => (h(-30, 58), pt(ragdoll::HAND_L)),
                Node::HandR | Node::ArmR | Node::GripR => (h(30, 58), pt(ragdoll::HAND_R)),
                Node::FootL => (h(-12, 4), pt(ragdoll::FOOT_L)),
                Node::FootR => (h(12, 4), pt(ragdoll::FOOT_R)),
            }
        };
        // the fallen body's facing: from pelvis toward head, flattened
        let up = pt(ragdoll::HEAD).sub(pt(ragdoll::PELVIS));
        let yaw = crate::geom::yaw_of(up.x, up.z);
        let tip = fx::QUARTER_TURN * 9 / 10;
        for p in parts {
            if far && !p.far {
                continue;
            }
            let (node_rest, node_at) = rest(p.node);
            let off = crate::render::rotate(p.at.sub(node_rest).scale(k), yaw, tip, 0);
            let f = Frame3::new(
                yaw,
                tip + p.tilt.0,
                p.tilt.1,
                p.size.scale(k).scale(fade.max(ONE / 10)),
            );
            let rgb = crate::render::mix(self.colour(&p.role), self.colour("ink"), (ONE - fade) / 2);
            b.push(
                if far { Mesh::Blob } else { p.mesh },
                Pass::Solid,
                node_at.add(off),
                &f,
                rgb,
                0,
            );
        }
    }
}

#[cfg(test)]
pub fn shipped() -> Look {
    Look::parse(
        include_str!("../../../data/palette.json"),
        include_str!("../../../data/models.json"),
    )
    .expect("palette and models")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content;

    fn cm(v: i32) -> Fx {
        v * ONE / 100
    }

    #[test]
    fn every_enemy_helper_and_mage_has_a_model_and_every_role_a_colour() {
        let look = shipped();
        let c = content::shipped();
        for e in &c.file.enemies {
            assert!(
                look.models.contains_key(&e.model),
                "{} has no model {}",
                e.id,
                e.model
            );
        }
        for h in &c.file.helpers {
            assert!(
                look.models.contains_key(&h.model),
                "{} has no model {}",
                h.id,
                h.model
            );
        }
        for m in ["archmage", "student"] {
            assert!(look.models.contains_key(m));
        }
        for s in &c.file.schools {
            assert!(look.palette.contains_key(&format!("school_{}", s.id)));
        }
        for t in &c.file.types {
            assert!(look.palette.contains_key(&format!("type_{}", t.id)));
        }
    }

    #[test]
    fn a_model_naming_a_missing_colour_is_refused() {
        let bad = include_str!("../../../data/models.json").replacen("\"robe\"", "\"plaid\"", 1);
        let e = Look::parse(include_str!("../../../data/palette.json"), &bad).unwrap_err();
        assert!(e.contains("plaid"), "{e}");
    }

    #[test]
    fn the_far_view_draws_fewer_parts_and_no_ink() {
        let look = shipped();
        let mut near = Batch::new();
        let mut far = Batch::new();
        look.draw(&mut near, "giant", V3::ZERO, &Pose::default(), &[], false);
        look.draw(&mut far, "giant", V3::ZERO, &Pose::default(), &[], true);
        assert!(
            far.count() * 2 < near.count(),
            "{} vs {}",
            far.count(),
            near.count()
        );
    }

    /// Where the archmage's parts of one colour end up, highest first.
    fn tops(look: &Look, pose: &Pose, role: &str) -> Vec<(Fx, Fx)> {
        let mut b = Batch::new();
        let swap: Vec<(&str, Rgb)> = vec![];
        look.draw(&mut b, "archmage", V3::ZERO, pose, &swap, false);
        let mut v = Vec::new();
        let want = look.colour(role);
        for l in b.lists_for_test() {
            for i in l.chunks(crate::render::INSTANCE_VALUES) {
                if [i[12], i[13], i[14]] == want {
                    v.push((i[7], i[11]));
                }
            }
        }
        v.sort_by_key(|a| std::cmp::Reverse(a.0));
        v
    }

    #[test]
    fn casting_lifts_the_right_arm_about_its_shoulder_above_the_head() {
        let look = shipped();
        let stand = tops(&look, &Pose::default(), "staff_orb");
        let cast = tops(
            &look,
            &Pose {
                cast: ONE,
                ..Default::default()
            },
            "staff_orb",
        );
        let head = tops(&look, &Pose::default(), "skin")[0].0;
        assert!(!stand.is_empty() && !cast.is_empty());
        // the staff's stone rises and swings forward (+z) when the arm turns
        assert!(cast[0].0 > stand[0].0 + cm(40), "{:?} {:?}", cast[0], stand[0]);
        assert!(cast[0].1 > stand[0].1 + cm(40), "{:?} {:?}", cast[0], stand[0]);
        // and the lifted hand (the highest skin part) is above the head's centre
        let hand = tops(
            &look,
            &Pose {
                cast: ONE,
                ..Default::default()
            },
            "skin",
        )[0]
        .0;
        assert!(hand > head, "{hand} {head}");
    }

    #[test]
    fn walking_swings_the_arms_against_each_other_and_sitting_lowers_the_body() {
        let look = shipped();
        let step = Pose {
            walk: fx::QUARTER_TURN,
            stride: cm(30),
            ..Default::default()
        };
        // with the left foot forward the right hand is forward: the two
        // hands (the lowest skin parts) differ in the sign of their z
        let hands = tops(&look, &step, "skin");
        let zs: Vec<Fx> = hands.iter().rev().take(2).map(|h| h.1).collect();
        assert!(
            zs.len() == 2 && zs[0].signum() != zs[1].signum() && zs[0] != 0,
            "{zs:?}"
        );
        let sit = Pose {
            sit: cm(50),
            ..Default::default()
        };
        let (up, down) = (
            tops(&look, &Pose::default(), "skin")[0].0,
            tops(&look, &sit, "skin")[0].0,
        );
        // the sinking is in the model's own metres: 50 cm lower
        assert!((up - down - cm(50)).abs() < cm(2), "{up} {down}");
    }
}

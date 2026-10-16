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
}

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
            0
        } else {
            fx::mul(fx::sin(pose.walk * 2).abs(), pose.stride / 6)
        };
        for p in parts {
            if far && !p.far {
                continue;
            }
            let mut at = p.at;
            match p.node {
                Node::FootL => at.z += fx::mul(swing, pose.stride),
                Node::FootR => at.z -= fx::mul(swing, pose.stride),
                Node::HandL => at.z -= fx::mul(swing, pose.stride / 2),
                Node::HandR => {
                    at.z += fx::mul(swing, pose.stride / 2);
                    at.y += fx::mul(pose.cast, ONE * 3 / 5);
                    at.z += fx::mul(pose.cast, ONE / 4);
                }
                Node::Head | Node::Body => at.y += bob,
            }
            let at = crate::render::rotate(at.scale(k), pose.yaw, pose.lean.0, pose.lean.1);
            let f = Frame3::new(
                pose.yaw,
                pose.lean.0 + p.tilt.0,
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
            if far {
                b.push(mesh, Pass::Solid, base.add(at), &f, rgb, 0);
            } else {
                b.inked(mesh, base.add(at), &f, rgb);
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
                Node::HandL => (h(-30, 58), pt(ragdoll::HAND_L)),
                Node::HandR => (h(30, 58), pt(ragdoll::HAND_R)),
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
        let mut near = Batch::new(look.colour("ink"));
        let mut far = Batch::new(look.colour("ink"));
        look.draw(&mut near, "giant", V3::ZERO, &Pose::default(), &[], false);
        look.draw(&mut far, "giant", V3::ZERO, &Pose::default(), &[], true);
        assert!(
            far.count() * 4 < near.count(),
            "{} vs {}",
            far.count(),
            near.count()
        );
    }
}

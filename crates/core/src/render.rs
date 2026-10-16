//! What the WebGL page draws, as numbers the core builds (Builder D3).
//! After Long Time Comin's `render.rs`, with Long Time Comin's ways of
//! drawing a great many orbs (SECOND-ORDER R3.3):
//!
//! - one instanced draw per mesh and pass, 16 values an instance: three
//!   columns of the shape's transform with the translation in their w,
//!   then rgb and glow;
//! - a static buffer for what does not move, uploaded once a scene, and a
//!   dynamic buffer re-sent each frame;
//! - an ink pass: the same shape a little larger, drawn with its front
//!   faces culled, which leaves the outline;
//! - a glow pass, added with no depth writes, for orbs and fire.
//!
//! A frame is a list of commands: a viewport (with the view it uses) and
//! draws. The watch screen is four viewports in one frame.

use crate::fx::{self, Fx, ONE};
use crate::geom::{self, v3, M4, V3};
use crate::mesh::Mesh;

pub const INSTANCE_VALUES: usize = 16;
pub const COMMAND_VALUES: usize = 6;
/// Per view: the 16-value matrix, the eye and a pad, the clear colour.
pub const VIEW_VALUES: usize = 24;
pub const MESHES: usize = Mesh::ALL.len();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pass {
    Solid = 0,
    Ink = 1,
    Glow = 2,
}

pub const PASSES: [Pass; 3] = [Pass::Solid, Pass::Ink, Pass::Glow];
pub const STATIC: u32 = 0;
pub const DYNAMIC: u32 = 1;

/// A colour in Q16 thirds, 0..=ONE each.
pub type Rgb = [Fx; 3];

pub fn hex(s: &str) -> Rgb {
    let h = s.trim_start_matches('#');
    let p = |i: usize| i32::from_str_radix(h.get(i..i + 2).unwrap_or("80"), 16).unwrap_or(128);
    [p(0) * ONE / 255, p(2) * ONE / 255, p(4) * ONE / 255]
}

pub fn mix(a: Rgb, b: Rgb, t: Fx) -> Rgb {
    [
        a[0] + fx::mul(b[0] - a[0], t),
        a[1] + fx::mul(b[1] - a[1], t),
        a[2] + fx::mul(b[2] - a[2], t),
    ]
}

/// Rotates a vector by roll (about z), then pitch (about x), then yaw
/// (about y), angles in turns.
pub fn rotate(v: V3, yaw: i32, pitch: i32, roll: i32) -> V3 {
    let (sr, cr) = (fx::sin(roll), fx::cos(roll));
    let v = v3(
        fx::mul(v.x, cr) - fx::mul(v.y, sr),
        fx::mul(v.x, sr) + fx::mul(v.y, cr),
        v.z,
    );
    let (sp, cp) = (fx::sin(pitch), fx::cos(pitch));
    let v = v3(
        v.x,
        fx::mul(v.y, cp) - fx::mul(v.z, sp),
        fx::mul(v.y, sp) + fx::mul(v.z, cp),
    );
    v.yaw(yaw)
}

/// An orientation and size: the columns of a shape's transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame3 {
    pub x: V3,
    pub y: V3,
    pub z: V3,
}

impl Frame3 {
    pub fn new(yaw: i32, pitch: i32, roll: i32, size: V3) -> Frame3 {
        Frame3 {
            x: rotate(v3(size.x, 0, 0), yaw, pitch, roll),
            y: rotate(v3(0, size.y, 0), yaw, pitch, roll),
            z: rotate(v3(0, 0, size.z), yaw, pitch, roll),
        }
    }
    pub fn upright(size: V3) -> Frame3 {
        Frame3 {
            x: v3(size.x, 0, 0),
            y: v3(0, size.y, 0),
            z: v3(0, 0, size.z),
        }
    }
    fn grown(&self, by: Fx) -> Frame3 {
        let g = |c: V3| {
            let l = c.len().max(1);
            c.ratio(l + by, l)
        };
        Frame3 {
            x: g(self.x),
            y: g(self.y),
            z: g(self.z),
        }
    }
}

/// Ink outline thickness for a shape of this size: 3 cm, plus a little
/// more for big things, capped at 25 cm.
pub fn ink_width(f: &Frame3) -> Fx {
    let size = f.x.len().max(f.y.len()).max(f.z.len());
    (ONE * 3 / 100 + size / 220).min(ONE / 4)
}

/// Instances grouped by pass and mesh.
#[derive(Debug, Clone, Default)]
pub struct Batch {
    lists: Vec<Vec<i32>>,
    pub ink: Rgb,
    /// Shapes nearer the eye than this get no ink (Raiders row 6).
    pub eye: Option<(V3, Fx)>,
}

impl Batch {
    pub fn new(ink: Rgb) -> Batch {
        Batch {
            lists: vec![Vec::new(); PASSES.len() * MESHES],
            ink,
            eye: None,
        }
    }

    pub fn push(&mut self, mesh: Mesh, pass: Pass, at: V3, f: &Frame3, rgb: Rgb, glow: Fx) {
        let l = &mut self.lists[pass as usize * MESHES + mesh as usize];
        l.extend_from_slice(&[
            f.x.x, f.x.y, f.x.z, at.x, f.y.x, f.y.y, f.y.z, at.y, f.z.x, f.z.y, f.z.z, at.z, rgb[0], rgb[1],
            rgb[2], glow,
        ]);
    }

    /// A solid shape with its ink outline.
    pub fn inked(&mut self, mesh: Mesh, at: V3, f: &Frame3, rgb: Rgb) {
        self.push(mesh, Pass::Solid, at, f, rgb, 0);
        let near = self.eye.map(|(e, d)| e.sub(at).len() < d).unwrap_or(false);
        if !near {
            let g = f.grown(ink_width(f) * 2);
            let ink = self.ink;
            self.push(mesh, Pass::Ink, at, &g, ink, 0);
        }
    }

    /// A glowing shape: a solid core and a soft halo twice its size.
    pub fn orb(&mut self, at: V3, r: Fx, rgb: Rgb) {
        let f = Frame3::upright(v3(r * 2, r * 2, r * 2));
        self.push(Mesh::Blob, Pass::Solid, at, &f, rgb, ONE);
        let h = Frame3::upright(v3(r * 4, r * 4, r * 4));
        self.push(Mesh::Blob, Pass::Glow, at, &h, rgb, ONE / 2);
    }

    pub fn count(&self) -> usize {
        self.lists.iter().map(|l| l.len() / INSTANCE_VALUES).sum()
    }

    /// Appends the batch's values to `values` and its draws to `commands`,
    /// in pass order, for buffer slot `buffer`.
    pub fn emit(&self, buffer: u32, values: &mut Vec<i32>, commands: &mut Vec<u32>) {
        for pass in PASSES {
            for mesh in Mesh::ALL {
                let l = &self.lists[pass as usize * MESHES + mesh as usize];
                if l.is_empty() {
                    continue;
                }
                let offset = (values.len() * 4) as u32;
                values.extend_from_slice(l);
                commands.extend_from_slice(&[
                    1,
                    buffer,
                    mesh as u32,
                    offset,
                    (l.len() / INSTANCE_VALUES) as u32,
                    pass as u32,
                ]);
            }
        }
    }
}

/// A camera: where it is, where it looks, and its lens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Camera {
    pub eye: V3,
    pub at: V3,
    /// 1 / tan(half the vertical field of view), Q16.
    pub focal: Fx,
}

impl Camera {
    pub fn matrix(&self, aspect: Fx) -> M4 {
        let proj = geom::perspective(self.focal, aspect.max(1), ONE / 5, 300 * ONE);
        geom::mat_mul(&proj, &geom::look_at(self.eye, self.at))
    }
}

/// One frame's commands and values.
#[derive(Debug, Clone, Default)]
pub struct FrameOut {
    pub commands: Vec<u32>,
    pub values: Vec<i32>,
    pub views: Vec<i32>,
}

impl FrameOut {
    /// Starts a viewport (pixels from the bottom left) drawn with `cam`;
    /// `clear` gives its sky colour.
    pub fn viewport(&mut self, x: i32, y: i32, w: i32, h: i32, cam: &Camera, clear: Rgb) {
        let aspect = (w.max(1) as i64 * ONE as i64 / h.max(1) as i64) as Fx;
        let m = cam.matrix(aspect);
        let at = self.views.len() as u32;
        self.views.extend_from_slice(&m);
        self.views
            .extend_from_slice(&[cam.eye.x, cam.eye.y, cam.eye.z, 0]);
        self.views.extend_from_slice(&[clear[0], clear[1], clear[2], ONE]);
        self.commands.extend_from_slice(&[
            0,
            x.max(0) as u32,
            y.max(0) as u32,
            w.max(1) as u32,
            h.max(1) as u32,
            at,
        ]);
    }

    pub fn batch(&mut self, b: &Batch) {
        b.emit(DYNAMIC, &mut self.values, &mut self.commands);
    }

    /// Replays a static scene's draws (already offset into the static buffer).
    pub fn static_draws(&mut self, draws: &[u32]) {
        self.commands.extend_from_slice(draws);
    }
}

/// A scene part that does not move, uploaded once.
#[derive(Debug, Clone, Default)]
pub struct StaticScene {
    pub values: Vec<i32>,
    /// Draw commands for each part, offset into `values`.
    pub parts: Vec<Vec<u32>>,
    pub version: u32,
}

impl StaticScene {
    pub fn add(&mut self, b: &Batch) -> usize {
        let mut cmds = Vec::new();
        b.emit(STATIC, &mut self.values, &mut cmds);
        self.parts.push(cmds);
        self.parts.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_batch_emits_one_draw_per_mesh_and_pass_with_byte_offsets() {
        let mut b = Batch::new([0, 0, 0]);
        let f = Frame3::upright(v3(ONE, ONE, ONE));
        b.inked(Mesh::Sphere, V3::ZERO, &f, [ONE, 0, 0]);
        b.inked(Mesh::Sphere, v3(ONE, 0, 0), &f, [ONE, 0, 0]);
        b.push(Mesh::Cube, Pass::Solid, V3::ZERO, &f, [0, ONE, 0], 0);
        assert_eq!(b.count(), 5);
        let (mut values, mut cmds) = (vec![0; 16], Vec::new());
        b.emit(DYNAMIC, &mut values, &mut cmds);
        // solid cube, solid spheres, ink spheres: three draws
        assert_eq!(cmds.len(), 3 * COMMAND_VALUES);
        // the first starts after the 16 values already there: byte 64
        assert_eq!(
            &cmds[..6],
            &[1, DYNAMIC, Mesh::Cube as u32, 64, 1, Pass::Solid as u32]
        );
        assert_eq!(
            &cmds[6..12],
            &[1, DYNAMIC, Mesh::Sphere as u32, 128, 2, Pass::Solid as u32]
        );
        assert_eq!(cmds[16], 2, "two inked spheres");
        assert_eq!(values.len(), 16 + 5 * INSTANCE_VALUES);
    }

    #[test]
    fn ink_is_a_little_larger_and_dropped_near_the_eye() {
        let mut b = Batch::new([0, 0, 0]);
        let f = Frame3::upright(v3(ONE, ONE, ONE));
        b.inked(Mesh::Sphere, V3::ZERO, &f, [ONE, ONE, ONE]);
        let ink = &b.lists[Pass::Ink as usize * MESHES + Mesh::Sphere as usize];
        // 1 m sphere: 3 cm + 1/220 m ≈ 3.45 cm a side, grown by twice that
        assert!(ink[0] > ONE && ink[0] < ONE + ONE / 10, "{}", ink[0]);
        let mut near = Batch::new([0, 0, 0]);
        near.eye = Some((v3(0, 0, ONE / 2), ONE));
        near.inked(Mesh::Sphere, V3::ZERO, &f, [ONE, ONE, ONE]);
        assert_eq!(near.count(), 1, "no ink within a metre of the eye");
    }

    #[test]
    fn a_quarter_turn_of_yaw_turns_x_into_minus_z() {
        let v = rotate(v3(ONE, 0, 0), fx::QUARTER_TURN, 0, 0);
        assert!(v.x.abs() < 200 && (v.z + ONE).abs() < 200, "{v:?}");
        let h = hex("#ff8000");
        assert_eq!(h, [ONE, 128 * ONE / 255, 0]);
    }
}

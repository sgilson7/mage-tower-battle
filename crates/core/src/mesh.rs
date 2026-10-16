//! The unit meshes everything is drawn from, after Long Time Comin's
//! `mesh.rs`. Each is a triangle list of (position, normal) pairs in Q16,
//! centred on the origin and one unit across, so an instance's transform
//! carries its size. Characters are spheres (entry 1, decision 37); the
//! field is cubes, cylinders and cones, flat-shaded by the shader.

use crate::fx::{self, Fx, ONE};
use crate::geom::{v3, V3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mesh {
    Cube = 0,
    /// a smooth globe: heads, bodies, orbs
    Sphere = 1,
    /// a low globe for crowds and scenery
    Blob = 2,
    Cyl = 3,
    Cone = 4,
    /// a flat square facing +y: the ground, rugs, decals
    Ground = 5,
    /// a twenty-sided low-poly crown for trees and rocks
    Ico = 6,
}

impl Mesh {
    pub const ALL: [Mesh; 7] = [
        Mesh::Cube,
        Mesh::Sphere,
        Mesh::Blob,
        Mesh::Cyl,
        Mesh::Cone,
        Mesh::Ground,
        Mesh::Ico,
    ];

    pub fn from_name(s: &str) -> Option<Mesh> {
        match s {
            "cube" => Some(Mesh::Cube),
            "sphere" => Some(Mesh::Sphere),
            "blob" => Some(Mesh::Blob),
            "cyl" => Some(Mesh::Cyl),
            "cone" => Some(Mesh::Cone),
            "ground" => Some(Mesh::Ground),
            "ico" => Some(Mesh::Ico),
            _ => None,
        }
    }
}

/// Values per vertex: position xyz, normal xyz.
pub const VERTEX_VALUES: usize = 6;

const H: Fx = ONE / 2;

fn norm(p: V3) -> V3 {
    p.with_len(ONE)
}

fn push_tri(out: &mut Vec<i32>, a: V3, b: V3, c: V3) {
    let n = norm(b.sub(a).cross(c.sub(a)));
    for p in [a, b, c] {
        out.extend_from_slice(&[p.x, p.y, p.z, n.x, n.y, n.z]);
    }
}

fn push_quad(out: &mut Vec<i32>, a: V3, b: V3, c: V3, d: V3) {
    push_tri(out, a, b, c);
    push_tri(out, a, c, d);
}

/// Vertex data of a mesh, counter-clockwise from outside.
pub fn vertices(mesh: Mesh) -> Vec<i32> {
    let mut o = Vec::new();
    match mesh {
        Mesh::Cube => {
            let p = |x: i32, y: i32, z: i32| v3(x * H, y * H, z * H);
            push_quad(&mut o, p(1, -1, 1), p(1, -1, -1), p(1, 1, -1), p(1, 1, 1));
            push_quad(&mut o, p(-1, -1, -1), p(-1, -1, 1), p(-1, 1, 1), p(-1, 1, -1));
            push_quad(&mut o, p(-1, 1, 1), p(1, 1, 1), p(1, 1, -1), p(-1, 1, -1));
            push_quad(&mut o, p(-1, -1, -1), p(1, -1, -1), p(1, -1, 1), p(-1, -1, 1));
            push_quad(&mut o, p(-1, -1, 1), p(1, -1, 1), p(1, 1, 1), p(-1, 1, 1));
            push_quad(&mut o, p(1, -1, -1), p(-1, -1, -1), p(-1, 1, -1), p(1, 1, -1));
        }
        Mesh::Ground => {
            let p = |x: i32, z: i32| v3(x * H, 0, z * H);
            push_quad(&mut o, p(-1, 1), p(1, 1), p(1, -1), p(-1, -1));
        }
        Mesh::Sphere | Mesh::Blob => {
            // latitude and longitude bands; each vertex's normal is itself
            let (lat, lon) = if mesh == Mesh::Sphere { (10, 16) } else { (5, 8) };
            let at = |i: i32, j: i32| {
                let a = fx::HALF_TURN * i / lat - fx::QUARTER_TURN;
                let b = fx::TURN * j / lon;
                let r = fx::cos(a);
                v3(
                    fx::mul(fx::mul(r, fx::cos(b)), H),
                    fx::mul(fx::sin(a), H),
                    fx::mul(fx::mul(r, -fx::sin(b)), H),
                )
            };
            let mut tri = |a: V3, b: V3, c: V3| {
                for p in [a, b, c] {
                    let n = norm(p);
                    o.extend_from_slice(&[p.x, p.y, p.z, n.x, n.y, n.z]);
                }
            };
            for i in 0..lat {
                for j in 0..lon {
                    let (a, b, c, d) = (at(i, j), at(i, j + 1), at(i + 1, j + 1), at(i + 1, j));
                    if i > 0 {
                        tri(a, b, c);
                    }
                    if i < lat - 1 {
                        tri(a, c, d);
                    }
                }
            }
        }
        Mesh::Ico => {
            // an icosahedron: twelve corners from three golden rectangles
            let g: Fx = ONE * 1618 / 1000;
            let s = |v: V3| v.with_len(H);
            let c = [
                v3(-ONE, g, 0),
                v3(ONE, g, 0),
                v3(-ONE, -g, 0),
                v3(ONE, -g, 0),
                v3(0, -ONE, g),
                v3(0, ONE, g),
                v3(0, -ONE, -g),
                v3(0, ONE, -g),
                v3(g, 0, -ONE),
                v3(g, 0, ONE),
                v3(-g, 0, -ONE),
                v3(-g, 0, ONE),
            ]
            .map(s);
            let faces = [
                (0, 11, 5),
                (0, 5, 1),
                (0, 1, 7),
                (0, 7, 10),
                (0, 10, 11),
                (1, 5, 9),
                (5, 11, 4),
                (11, 10, 2),
                (10, 7, 6),
                (7, 1, 8),
                (3, 9, 4),
                (3, 4, 2),
                (3, 2, 6),
                (3, 6, 8),
                (3, 8, 9),
                (4, 9, 5),
                (2, 4, 11),
                (6, 2, 10),
                (8, 6, 7),
                (9, 8, 1),
            ];
            for (a, b, d) in faces {
                push_tri(&mut o, c[a], c[b], c[d]);
            }
        }
        Mesh::Cyl | Mesh::Cone => {
            const SIDES: i32 = 14;
            let cone = mesh == Mesh::Cone;
            let rim = |j: i32, y: i32| {
                let a = fx::TURN * j / SIDES;
                v3(fx::mul(fx::cos(a), H), y, fx::mul(-fx::sin(a), H))
            };
            let apex = v3(0, H, 0);
            for j in 0..SIDES {
                let (a, b) = (rim(j, -H), rim(j + 1, -H));
                if cone {
                    push_tri(&mut o, a, b, apex);
                } else {
                    let (c, d) = (rim(j + 1, H), rim(j, H));
                    push_quad(&mut o, a, b, c, d);
                    push_tri(&mut o, d, c, apex);
                }
                push_tri(&mut o, b, a, v3(0, -H, 0));
            }
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tris(m: Mesh) -> Vec<[V3; 4]> {
        vertices(m)
            .chunks(3 * VERTEX_VALUES)
            .map(|t| {
                let v = |i: usize| v3(t[i], t[i + 1], t[i + 2]);
                [v(0), v(6), v(12), v(3)]
            })
            .collect()
    }

    #[test]
    fn every_face_of_a_closed_mesh_points_away_from_its_centre() {
        for (m, centre) in [
            (Mesh::Cube, V3::ZERO),
            (Mesh::Sphere, V3::ZERO),
            (Mesh::Blob, V3::ZERO),
            (Mesh::Cyl, V3::ZERO),
            (Mesh::Cone, v3(0, -H / 2, 0)),
            (Mesh::Ico, V3::ZERO),
        ] {
            for [a, b, c, n] in tris(m) {
                let centroid = v3(
                    (a.x + b.x + c.x) / 3,
                    (a.y + b.y + c.y) / 3,
                    (a.z + b.z + c.z) / 3,
                );
                assert!(
                    centroid.sub(centre).dot(n) > 0,
                    "{m:?}: face at {centroid:?} has normal {n:?}"
                );
            }
        }
    }

    #[test]
    fn the_cube_has_twelve_triangles_the_ico_twenty_and_the_ground_faces_up() {
        assert_eq!(tris(Mesh::Cube).len(), 12);
        assert_eq!(tris(Mesh::Ico).len(), 20);
        for [_, _, _, n] in tris(Mesh::Ground) {
            assert_eq!(n, v3(0, ONE, 0));
        }
    }
}

//! What the page needs to know once: every size, stride and offset, the
//! passes, the light, and which bit each action sets. The page holds none
//! of these numbers itself (Builder D3).

use serde_json::json;

use crate::fx::ONE;
use crate::mesh::{self, Mesh, VERTEX_VALUES};
use crate::prep;
use crate::render::{COMMAND_VALUES, INSTANCE_VALUES, VIEW_UNIFORMS, VIEW_VALUES};

/// The light's weights, after Ape Race's shader (T34): how much of the sky
/// and ground light a shape takes, how much of the sun on the faces turned
/// to it, how much of the sun wrapped round the rest, and a half for the
/// wrapping. Then the glow pass's strength and how bright a glowing solid
/// is. The colours and the haze come with each view (`render::Mood`).
/// (name, size, values in Q16.)
const UNIFORMS: [(&str, usize, [i32; 4]); 3] = [
    ("uEps", 1, [7, 0, 0, 0]),
    (
        "uMix",
        4,
        [ONE * 64 / 100, ONE * 46 / 100, ONE * 24 / 100, ONE / 2],
    ),
    ("uGlow", 2, [ONE * 16 / 10, ONE * 108 / 100, 0, 0]),
];

/// The uniforms' values in order, Q16; the shim turns them into floats.
pub fn uniform_values() -> Vec<i32> {
    UNIFORMS.iter().flat_map(|(_, n, v)| v[..*n].to_vec()).collect()
}

pub fn numbers() -> String {
    let meshes: Vec<serde_json::Value> = Mesh::ALL
        .iter()
        .map(|&m| json!({"id": m as u32, "vertices": mesh::vertices(m).len() / VERTEX_VALUES}))
        .collect();
    json!({
        "meshes": meshes,
        "vertex": {"stride": VERTEX_VALUES * 4, "attributes": [["aPos", 3, 0], ["aNormal", 3, 12]]},
        "instance": {"stride": INSTANCE_VALUES * 4, "attributes": [["iA", 4, 0], ["iB", 4, 16], ["iC", 4, 32], ["iD", 4, 48]]},
        "command_values": COMMAND_VALUES,
        "ops": {"viewport": 0, "draw": 1},
        "view": {"size": VIEW_VALUES, "matrix": [0, 16], "clear": [20, 24], "uniforms": VIEW_UNIFORMS.iter().map(|(name, at, n)| json!([name, at, n])).collect::<Vec<_>>()},
        "passes": {"solid": 0, "glow": 1},
        "buffers": {"static": 0, "dynamic": 1},
        "uniforms": UNIFORMS.iter().map(|(name, n, _)| json!([name, n])).collect::<Vec<_>>(),
        "actions": {"up": prep::UP, "down": prep::DOWN, "left": prep::LEFT, "right": prep::RIGHT, "use": prep::USE, "back": prep::BACK, "note0": prep::NOTE[0], "note1": prep::NOTE[1], "note2": prep::NOTE[2], "note3": prep::NOTE[3]},
    })
    .to_string()
}

pub fn mesh_values(id: u32) -> Vec<i32> {
    Mesh::ALL
        .iter()
        .find(|&&m| m as u32 == id)
        .map(|&m| mesh::vertices(m))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_numbers_name_every_mesh_and_the_view_layout_matches_the_frame() {
        let n: serde_json::Value = serde_json::from_str(&numbers()).unwrap();
        assert_eq!(n["meshes"].as_array().unwrap().len(), Mesh::ALL.len());
        assert_eq!(n["view"]["clear"][1], json!(24));
        assert_eq!(
            n["view"]["uniforms"].as_array().unwrap().len(),
            VIEW_UNIFORMS.len()
        );
        let mut f = crate::render::FrameOut::default();
        let cam = crate::render::Camera {
            eye: crate::geom::v3(0, ONE, ONE),
            at: crate::geom::V3::ZERO,
            focal: ONE,
        };
        f.viewport(
            0,
            0,
            10,
            10,
            &cam,
            &crate::scene::room_mood(&crate::look::shipped()),
        );
        assert_eq!(f.views.len(), VIEW_VALUES);
        assert_eq!(f.commands.len(), COMMAND_VALUES);
        assert_eq!(mesh_values(Mesh::Cube as u32).len(), 36 * VERTEX_VALUES);
    }
}

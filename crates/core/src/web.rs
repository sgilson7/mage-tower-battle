//! What the page needs to know once: every size, stride and offset, the
//! passes, the light, and which bit each action sets. The page holds none
//! of these numbers itself (Builder D3).

use serde_json::json;

use crate::fx::ONE;
use crate::mesh::{self, Mesh, VERTEX_VALUES};
use crate::prep;
use crate::render::{COMMAND_VALUES, INSTANCE_VALUES, VIEW_VALUES};

/// The light: from above and in front, three flat bands like the
/// journal's mockups, the room's sky tint, and the glow pass's strength.
/// (name, size, values in Q16.)
const UNIFORMS: [(&str, usize, [i32; 3]); 6] = [
    ("uEps", 1, [7, 0, 0]),
    ("uLight", 3, [ONE * 35 / 100, ONE * 85 / 100, ONE * 40 / 100]),
    ("uBands", 3, [ONE * 62 / 100, ONE * 86 / 100, ONE * 104 / 100]),
    ("uCuts", 2, [ONE / 10, ONE * 55 / 100, 0]),
    ("uSky", 3, [ONE * 12 / 100, ONE * 10 / 100, ONE * 18 / 100]),
    ("uGlow", 2, [ONE * 2, ONE * 2, 0]),
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
        "view": {"size": VIEW_VALUES, "matrix": [0, 16], "eye": [16, 19], "clear": [20, 24]},
        "passes": {"solid": 0, "ink": 1, "glow": 2},
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
        assert_eq!(n["view"]["clear"][1], json!(VIEW_VALUES));
        let mut f = crate::render::FrameOut::default();
        let cam = crate::render::Camera {
            eye: crate::geom::v3(0, ONE, ONE),
            at: crate::geom::V3::ZERO,
            focal: ONE,
        };
        f.viewport(0, 0, 10, 10, &cam, [0, 0, 0]);
        assert_eq!(f.views.len(), VIEW_VALUES);
        assert_eq!(f.commands.len(), COMMAND_VALUES);
        assert_eq!(mesh_values(Mesh::Cube as u32).len(), 36 * VERTEX_VALUES);
    }
}

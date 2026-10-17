//! tool_core: every application rule of the tool.
//!
//! Builder requirement: this crate performs no input or output. It takes
//! values and returns values, so `cargo test` reaches every rule without a
//! browser. `tests/boundary.rs` enforces that, along with the dependency
//! allowlist and the reference ban on floats, hash maps and clocks.
//!
//! The door the shim calls through is `session::Session`: screens, `act`,
//! fixed ticks, and what to draw and play each frame.

pub mod arena;
pub mod battle;
pub mod body;
pub mod bot;
pub mod content;
pub mod draw;
pub mod fx;
pub mod game;
pub mod geom;
pub mod glossary;
pub mod glyph;
pub mod look;
pub mod mesh;
pub mod mind;
pub mod online;
pub mod overlay;
pub mod prep;
pub mod ragdoll;
pub mod render;
pub mod rhythm;
pub mod rng;
pub mod room;
pub mod save;
pub mod scene;
pub mod session;
pub mod sound;
pub mod spell;
pub mod stress;
pub mod strings;
pub mod ticks;
pub mod web;

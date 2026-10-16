//! tool_shim: forwards values between the page and `tool_core::session`.
//!
//! Builder requirement: the shim decides nothing. Each function is one call
//! into the core and a conversion: Q16 integers to floats for WebGL, the
//! page's milliseconds to microseconds. `crates/core/tests/boundary.rs`
//! fails if a branch, a loop or the `?` operator appears here.

use tool_core::session::Session;
use wasm_bindgen::prelude::*;

fn floats(v: &[i32]) -> Vec<f32> {
    v.iter().map(|&x| x as f32 / 65536.0).collect()
}

#[wasm_bindgen]
pub struct Game {
    inner: Session,
}

#[wasm_bindgen]
impl Game {
    #[allow(clippy::too_many_arguments)]
    #[wasm_bindgen(constructor)]
    pub fn new(
        strings: &str,
        content: &str,
        palette: &str,
        models: &str,
        sounds: &str,
        glossary: &str,
        save: &str,
        seed: u32,
    ) -> Result<Game, JsError> {
        Session::new(
            strings,
            content,
            palette,
            models,
            sounds,
            glossary,
            save,
            seed as u64,
        )
        .map(|inner| Game { inner })
        .map_err(|e| JsError::new(&e))
    }
    pub fn frame(&mut self, now_ms: f64, bits: u32) {
        self.inner.frame((now_ms * 1000.0) as i64, bits)
    }
    pub fn resize(&mut self, css_w: u32, css_h: u32, dpr: f64) -> Vec<i32> {
        self.inner
            .resize(css_w as i32, css_h as i32, (dpr * 100.0) as i32)
    }
    pub fn act(&mut self, name: &str, value: &str) -> String {
        self.inner.act_json(name, value)
    }
    pub fn ui(&self) -> String {
        self.inner.ui().to_string()
    }
    pub fn commands(&self) -> Vec<u32> {
        self.inner.frame_out().commands.clone()
    }
    pub fn values(&self) -> Vec<f32> {
        floats(&self.inner.frame_out().values)
    }
    pub fn views(&self) -> Vec<f32> {
        floats(&self.inner.frame_out().views)
    }
    pub fn static_values(&self) -> Vec<f32> {
        floats(&self.inner.statics().values)
    }
    pub fn static_version(&self) -> u32 {
        self.inner.statics().version
    }
    pub fn overlay(&self) -> Vec<i32> {
        self.inner.overlay()
    }
    pub fn sounds(&self) -> Vec<u8> {
        self.inner.sounds()
    }
    pub fn sound_count(&self) -> u32 {
        self.inner.sound_count()
    }
    pub fn sound_wav(&self, i: u32) -> Vec<u8> {
        self.inner.sound_wav(i)
    }
    pub fn take_save(&mut self) -> String {
        self.inner.take_save()
    }
    pub fn build(&self) -> String {
        self.inner.build()
    }
    pub fn host(&mut self, a: u32, b: u32, app_id: &str) -> String {
        self.inner.host(a, b, app_id)
    }
    pub fn join(&mut self, code: &str, app_id: &str) -> String {
        self.inner.join(code, app_id)
    }
    pub fn net_peer(&mut self, peer: u32) {
        self.inner.net_peer(peer)
    }
    pub fn net_left(&mut self, peer: u32) {
        self.inner.net_left(peer)
    }
    pub fn net_message(&mut self, peer: u32, text: &str) {
        self.inner.net_message(peer, text)
    }
    pub fn net_error(&mut self, key: &str) {
        self.inner.net_error(key)
    }
    pub fn net_outbox(&mut self) -> String {
        self.inner.net_outbox()
    }
    pub fn net_active(&self) -> bool {
        self.inner.net_active()
    }
    pub fn net_status(&self) -> String {
        self.inner.net_status()
    }
}

#[wasm_bindgen]
pub fn numbers() -> String {
    tool_core::web::numbers()
}

#[wasm_bindgen]
pub fn uniforms() -> Vec<f32> {
    floats(&tool_core::web::uniform_values())
}

#[wasm_bindgen]
pub fn mesh(id: u32) -> Vec<f32> {
    floats(&tool_core::web::mesh_values(id))
}

#[wasm_bindgen]
pub fn draw_assets() -> String {
    tool_core::draw::assets()
}

/// The physics probe (PLAN.md decision 2), for tools/probe_wasm.py.
#[wasm_bindgen]
pub struct StressProbe {
    inner: tool_core::stress::Stress,
}

#[wasm_bindgen]
impl StressProbe {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, crowd: u32, cap: u32, burst: u32) -> StressProbe {
        StressProbe {
            inner: tool_core::stress::Stress::with_burst(
                seed as u64,
                crowd as usize,
                cap as usize,
                burst as usize,
            ),
        }
    }
    pub fn run(&mut self, ticks: u32) {
        self.inner.run(ticks)
    }
    pub fn awake(&self) -> u32 {
        self.inner.awake_ragdolls() as u32
    }
}

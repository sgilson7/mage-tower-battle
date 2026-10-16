//! Probe, not a check (docs/checks.md): prints what four arenas of physics
//! cost a tick natively. Run:
//!
//!     cargo test --release -p tool_core --test physics_probe -- --ignored --nocapture
//!
//! The wasm cost is measured in Chromium by `tools/probe_wasm.py`. The
//! test harness's own clock is used here; the core never reads one.

use std::time::Instant;

use tool_core::stress::Stress;

#[test]
#[ignore]
fn physics_cost_of_four_arenas() {
    for (crowd, cap, burst) in [
        (120, 24, 1),
        (120, 24, 12),
        (120, 48, 24),
        (200, 96, 48),
        (300, 160, 80),
    ] {
        let mut s = Stress::new(1, crowd, cap);
        s.burst = burst;
        for _ in 0..120 {
            s.step();
        }
        let ticks = 1800;
        let start = Instant::now();
        let mut most = 0;
        for _ in 0..ticks {
            s.step();
            most = most.max(s.awake_ragdolls());
        }
        let us = start.elapsed().as_micros() as f64 / ticks as f64;
        println!(
            "crowd {crowd} x4, ragdoll cap {cap}/arena, burst {burst}: {us:.1} us/tick native (most awake ragdolls {most}, deaths {})",
            s.deaths
        );
    }
}

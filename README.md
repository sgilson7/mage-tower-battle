# Mage Tower Battle

You are the new archmage of a mage college. The last one was fired, and on his way out he pulled every book off the library shelves. Read what he left on the floor to learn spell pieces, join them into spells at the laboratory, interview applicants, and teach your students by drawing each spell's glyphs at the podium. Then watch: every tower is attacked at once, and your students cast what you taught them, as well as you taught it.

**Play it:** https://sgilson7.github.io/mage-tower-battle/

## Playing

- Walk with WASD or the arrow keys. Stand at a station to use it; E does its main thing and Esc cancels.
- A day is a few minutes of preparation and then a battle you watch. Walking, reading, interviewing and teaching all take time, so the order you visit stations in matters.
- Spells read left to right: modifiers change the next payload, and a trigger starts the rest of the spell when the payload hits, dies or ends. A strong spell with a short cooldown costs much more mana.
- Teaching one student forgives a shaky line; a whole class needs a careful one. How well you draw is how well they learn.
- Spells that start with Invade send summons at a rival. Yours show in their invasion book at once; your students' only when the battle starts.
- Damage to your tower lowers your score, and nothing raises it. The highest score after the last day wins.
- Modes: versus three computer rivals (the Spellcrafter, the Mass Teacher, the Attack Mage and the Balanced Archmage), a short practice day, and four trials with Bronze, Silver, Gold and Archmage medals. Online play for four is planned for the next update.

## How it is built

The game is a [Builder](https://github.com/sgilson7/builder-setting) project. Every rule lives in a Rust core with no input or output, no floats and no clocks: battles, spells, the room, the mini-games' scoring, the bots, the drawing lists and even the chip sounds. A WebAssembly shim moves values to the page and decides nothing, and the page draws what the core returns. Tests read the repository's own source and fail if a rule leaks out of the core.

The design, the plan and the record of every decision made along the way are in `BRIEF.md`, `PLAN.md`, `SECOND-ORDER.md`, `DECISIONS.md` and `HANDOFF.md`.

## Build, test and serve

```bash
./scripts/setup.sh              # once: Python venv with Playwright and three browsers
cargo test --workspace          # the core, boundary, copy and storage tests
./scripts/build.sh              # release WebAssembly build into dist/
python3 scripts/serve.py        # http://127.0.0.1:8000/
./scripts/check.sh              # everything, before publishing
python3 scripts/break_tests.py  # each row of tests/breaks.json must make its test fail
./scripts/count-tests.sh        # the test count to quote in a document
```

Probes, outside the check: `tools/probe_wasm.py` (physics in the browser), `crates/core/tests/physics_probe.rs` and `crates/core/tests/balance_probe.rs` (run with `--ignored --nocapture`).

Needs Rust (pinned in `rust-toolchain.toml`), `wasm-bindgen-cli` 0.2.127 and Python 3.

## Layout

| Path | Holds |
| --- | --- |
| `crates/core` | every rule: `battle.rs`, `prep.rs`, `spell.rs`, `game.rs`, `bot.rs`, the physics in `body.rs` and `ragdoll.rs`, drawing in `scene.rs`, `look.rs` and `render.rs`, chip sounds in `sound.rs`; `session.rs` is the door the shim calls |
| `crates/shim` | the `wasm-bindgen` functions the page calls; each is one call into the core |
| `web/` | the page: `app.js`, the WebGL renderer `gl.js`, the overlay `render2d.js`, panels `ui.js`, `keys.js`, `sound.js`, and `store.js`, the only file that touches browser storage |
| `data/` | every word (`strings.json`), the content and balance (`content.json`), models, palette, sounds, controls and shaders |
| `GLOSSARY.md`, `TONE.md` | what each term means; how every string is written |
| `tests/browser/` | the browser check and the practice day it plays |

## Licence

MIT. See `LICENSE` and `LICENSES.md`.

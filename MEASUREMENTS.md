# Measurements

Stage 4, recorded by the agent before building and again at handoff. `B2 Require step-by-step justification`.

The state measured rather than recalled, so later changes can be compared with it. A probe prints numbers and asserts loose bounds at most; record its output here with the command that produced it.

| What | Command | Result | When |
| --- | --- | --- | --- |
| Toolchain | `rustc --version; wasm-bindgen --version` | rustc 1.99.0; wasm-bindgen 0.2.127 | 2026-10-16, before the build |
| Core tests | `./scripts/count-tests.sh` | 38 passed, 0 failed, 0 ignored (template) | 2026-10-16, before the build |
| Full check | `./scripts/check.sh` | BUILDER CHECK PASSED, Chromium, Firefox, WebKit, 43 s | 2026-10-16, before the build |
| WebAssembly size | printed by `./scripts/build.sh` | 170 KB | 2026-10-16, before the build |
| Breaks caught | `python3 scripts/break_tests.py` | 2 of 2 | 2026-10-16, before the build |

## Physics probe (PLAN.md decision 2), 2026-10-16

Four arenas, each with three staggered walls, a crowd walking the flow field, a blast every 10 ticks and ragdolls in bursts every 30. Native: `cargo test --release -p tool_core --test physics_probe -- --ignored --nocapture`. Wasm: `./scripts/build.sh && .venv/bin/python tools/probe_wasm.py` (Chromium, headless).

| Crowd per arena | Ragdoll cap per arena | Awake ragdolls, all four arenas | Native µs/tick | Wasm µs/tick |
| --- | --- | --- | --- | --- |
| 120 | 24 | 15 (burst 1) | 188 | 236 |
| 120 | 24 | 96 (burst 12) | 305 | 365 |
| 120 | 48 | 192 | 427 | 519 |
| 200 | 96 | 384 | 890 | 1,082 |
| 300 | 160 | 640 | 1,586 | 1,911 |

Budget was 4 ms of wasm a tick for all four arenas: met at every row. The battle stays at 60 ticks a second; the ragdoll cap starts at 24 as the person chose and can rise (R3.8).

## Balance probe (PLAN.md decision 10), 2026-10-16

`MATCHES=40 cargo test --release -p tool_core --test balance_probe -- --ignored --nocapture`: four-bot matches, one of each plan, seats rotating, default settings (4 days, 240 s preparation, 90 s battles).

| Plan | Wins of 40 | Mean damage taken | Of it from invasions |
| --- | --- | --- | --- |
| Spellcrafter | 2 | 3,263 | 2,497 |
| Mass Teacher | 13 | 2,129 | 1,706 |
| Attack Mage | 0 | 3,419 | 1,348 |
| Balanced | 25 | 1,546 | 1,071 |

Worst damage taken in one match: 6,165 (score starts at 10,000). Ties count as a win for each tied seat. Earlier runs on the way here are in SECOND-ORDER R3.25 to R3.28.

Machine: Apple silicon Mac (aarch64-apple-darwin), the person's laptop.

## Online play (PLAN.md decision 15), 2026-10-16

- `tests/browser/online.py`: two browsers on one machine met by pasted codes with no relay and no STUN, and played a one-day match to its summary; the two summaries were the same text. Passed in Chromium, Firefox and WebKit.
- `every_browser_of_an_online_match_ends_with_the_same_scores`: a host and two joiners, wired in the test, played two days with invasions crossing; all three ended with the same scores and checksum.
- Live, by hand: two Chromium browsers on one machine met by room code over the public Nostr relays in `web/net/config.js` and started a match.
- The WebAssembly module is 1,336,496 bytes (`dist/pkg/tool_shim_bg.wasm`, this build).

## Spell space (TRIAGE T4), 2026-10-16

`crates/core/tests/spell_space.rs`, scene in `support/spell_rules.rs` (README in `tests/spell-log/`). 33 pieces may go in a spell (the 36 less the 3 helpers).

| Length | Rows | Spells the laboratory accepts | Logged |
| --- | --- | --- | --- |
| 1 | 33 | 22 | all |
| 2 | 1,089 | 619 | all |
| 3 | 35,937 | 18,928 | all |
| 4 | 1,185,921 | 577,006 | all |

- Everyday test (every row of one and two pieces, each spell played twice): 1,122 rows, 641 spells, 0.21 s in a debug build on this Mac. Of the 481 refused: no payload first 176, modifier at the end 132, trigger at the end 66, place-only piece 66, Invade without a summon 41.
- Logbook, release build: lengths 1 to 3 in 3.8 s; length 4 in 101 s (about 5,700 spells a second on all cores). The full local log for lengths 1 to 4 is 186 MB.
- Result: no spell of up to four pieces broke a rule. 473 were noted: 471 invasions whose summon sits behind a trigger and never came, and 2 (`heavy fire_bolt ring glass_wall`, `heavy fire_bolt ring ivory_wall`) whose bolt dealt no damage in the scene. The most units on the field at once was 36.

## Not measured

Devices, browsers and conditions no measurement here covers.

- Online play between two real networks, behind strict NATs, or with four people at once.
- Any phone or tablet; any computer but the author's Mac.

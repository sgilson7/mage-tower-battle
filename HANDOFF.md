# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`. A fresh session starts from this file, not from memory.

## What this is

Mage Tower Battle is a browser game for one to four players. Each player is an archmage rebuilding a wrecked college. They read books, join spell pieces into spells, recruit and teach students, and lay out the field; then four towers fight the day's enemies at once while rivals' invasions arrive. Score only falls, and the highest score after the last day wins. It is a Rust core compiled to WebAssembly with a WebGL page. It lives at https://sgilson7.github.io/mage-tower-battle/ (repository `sgilson7/mage-tower-battle`).

## Rules that hold it up

| Rule | Guarded by |
| --- | --- |
| Every rule lives in `crates/core`: no floats, hash maps, clocks, randomness or I/O there | `crates/core/tests/boundary.rs` |
| The shim decides nothing; the page holds no number but 0 and 1 and no `Math` | `boundary.rs` page and shim scans |
| Shaders hold only 0 and 1; every rule value arrives as a uniform | `boundary.rs` shader scan |
| Every string is in `data/strings.json`; glossary `Not:` words and TONE.md phrases fail | `crates/core/tests/copy.rs` |
| Score never rises; a ward takes damage before the score | `a_ward_takes_damage_before_the_score_and_the_score_never_rises` |
| Only `web/store.js` names browser storage | `no_storage.rs` (BUILDER-SPEC change, R3.6) |
| Online: every browser runs all four battles from the same seed and plans. Bots aim by their own seat, invasions are sorted by sender, remote plans are checked before use | `every_browser_of_an_online_match_ends_with_the_same_scores`, `invasions_reach_a_battle_in_sender_order_whatever_order_they_arrived_in`, `a_plan_from_another_browser_is_refused_when_an_index_is_out_of_range` |
| Only `web/online.js` imports the transport; only `web/net/config.js` names a relay, and it matches the CSP | `crates/core/tests/online_fence.rs` |

## Shape of the code

| Where | What |
| --- | --- |
| `crates/core/src/arena.rs`, `body.rs`, `ragdoll.rs`, `battle.rs` | the field, the flow field, bodies, Verlet ragdolls, a battle |
| `spell.rs`, `glyph.rs`, `rhythm.rs`, `prep.rs`, `bot.rs` | spells and mana, the drawing and the interview, the preparation, computer rivals |
| `game.rs` | a match: days, scores, invasions, trials and medals |
| `online.rs` | the room protocol: hello, lobby, start, invasions, plans, checksums |
| `session.rs` | screens, the `act` door, the `ui` view model, the frame |
| `mesh.rs`, `render.rs`, `look.rs`, `scene.rs`, `overlay.rs`, `sound.rs` | what is drawn and heard, as numbers the page uploads |
| `crates/shim` | wasm-bindgen exports only |
| `web/` | `app.js` (frame loop), `gl.js`, `ui.js`, `keys.js`, `sound.js`, `store.js`, `online.js`, `net/` (transport, vendored Trystero) |
| `data/` | content, strings, palette, models, sounds, controls, display, shaders |

## Commands

| Command | Prints when it passes |
| --- | --- |
| `cargo test` | every `test result: ok` |
| `./scripts/build.sh` | the file list and `wasm: … KB` |
| `./scripts/check.sh` | the eight steps, the last ending in the browser check passing in Chromium, Firefox and WebKit |
| `python3 scripts/break_tests.py` | `N of N breaks caught` |
| `.venv/bin/python tests/browser/online.py chromium firefox webkit` | `ONLINE CHECK PASSED in …` |
| `.venv/bin/python tests/browser/interactions.py` | a practice day played by keys |
| `.venv/bin/python tests/browser/contrast.py chromium firefox webkit` | `CONTRAST CHECK PASSED …`: text stands out in the light and dark schemes (T6) |
| `SPELL_LENGTHS=1,2,3,4 SPELL_BUDGET_S=600 cargo test --release -p tool_core --test spell_space -- --ignored --nocapture logbook` | the spell-space logbook (T4): every spell of up to four pieces played and checked; README in `tests/spell-log/` |
| `.venv/bin/python tools/shots.py OUTDIR` | screenshots of the running game |
| `.venv/bin/python tools/probe_wasm.py` | the physics timing (MEASUREMENTS.md) |

Publishing: a push to `main` on GitHub runs `.github/workflows/deploy.yml` (check, the online check, deploy to Pages, then the browser check against the live address). The local branch `public` is pushed as `main` (R3.36); local `main` holds the private history and is never pushed.

## What will bite within the hour

- Push `public`, not `main`: `git push origin public:main`.
- A test that needs a workaround is a finding; record it in SECOND-ORDER.md, do not loosen the check.
- A new behaviour test needs a row in `tests/breaks.json`: write it with `tools/add_breaks.py` from a fragment, since `cargo fmt` rots exact lines (R3.23).
- An online change must keep the browsers equal: anything a battle reads must come from the seed, the plans or the start message, never from the local seat.
- R3.28: the bots' win shares miss the plan's one-in-ten target for the spellcrafter and attack mage; judge the plans by playing. Bots now also know Glass Wall from the start (T1), so the old balance numbers are stale.
- A change to `content.json` or the rule code makes the spell log stale; re-run the logbook and commit `tests/spell-log/`.
- Round 4 answered TRIAGE T1–T13 (rows R4.1–R4.18 and R4.20–R4.22). The game's version is the first entry of `data/news.json`: add an entry there with every update.

## Mistakes so far

- Checks that could not fail (R3.9, R3.22, and in this round the room-code alphabet and the invasion order, both caught by `break_tests.py` and strengthened). Rule: every behaviour test has a break row.
- Physics order (R3.7): separate before keep-out, or walkers leave the grid. Rule: the order in `stress.rs` and `battle.rs`.
- Breaks rotted after formatting (R3.19, R3.23). Rule: `tools/add_breaks.py`.
- Reversions to the textbook rule: R2.1 (a tower at zero is out; the person's rule is score only falls) and R3.25 (enemies keep attacking; the plan's rule is one leak strike). Rule: glossary entries and the score test.
- Untrusted input (R3.38): `spell::check` indexed without a bound. Rule: `online::sane_plan`.
- A bot aimed by the local seat (R3.37). Rule: the online equality test.
- Checks that ran in one colour scheme only (R4.4) and drove the map through `act` where the page hid its panel (R4.3). Rule: `tests/browser/contrast.py`; the map test checks the panel's kind.
- A spell the laboratory accepted that could never work: an invasion whose summon came after a trigger or after a payload (R4.21), found by the spell space.

## Open for the person

Every `SECOND-ORDER.md` row with status *person*, by number.

- R1.12: names the agent made up for the mockups.
- R3.10: every name in `data/content.json` (pieces, enemies, features, helpers) and the type chart.
- R3.28: the bots' win shares; decide whether to tune the attack mage and spellcrafter.
- R3.40: try a room code with a friend on another network; the machine checks cover pasted codes on one machine only.
- R5.3: what the wireframe's far-left red box and its two rows mean (PLAN.md round 5, question 1).
- R5.4: round 5's names and numbers (PLAN.md round 5, questions 2–8). Round 5 waits for the person to read its plan before any code.

R2.2, R2.5, R2.6, R2.7 and R2.8 were answered with the plan and are closed.

## What passed

16 October 2026, round 4 (TRIAGE T1–T13), on the commit that last changed this section:

- `./scripts/check.sh`: `BROWSER CHECK PASSED in chromium, firefox, webkit` and `BUILDER CHECK PASSED`.
- `./scripts/count-tests.sh`: `192 passed, 0 failed, 3 ignored`.
- `python3 scripts/break_tests.py`: every row added this round caught when run alone (T9–T13: 9 of 9); the last full run was 89 of 89 before them.
- `tests/browser/contrast.py`: `CONTRAST CHECK PASSED in chromium, firefox, webkit`.
- `tests/browser/online.py`: `ONLINE CHECK PASSED in chromium`.
- The spell-space logbook, lengths 1–4: 589,309 spells, none broke a rule, 2 notes.

## The next action

Play a whole Versus match and an online match with a friend, then write what felt wrong as TRIAGE rows (T1, T2 …).

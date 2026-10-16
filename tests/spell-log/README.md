# The spell logbook (T4)

The spell space test (`crates/core/tests/spell_space.rs`) plays a spell for four seconds in one fixed scene and checks it against the rules in `crates/core/tests/support/spell_rules.rs`, after Galactic Survivors' rig log. The logbook keeps a line for every row of pieces it has looked at, so each run plays only rows that are new or out of date.

    cargo test --release -p tool_core --test spell_space -- --ignored --nocapture logbook

`SPELL_BUDGET_S` sets how long a run may play (120 seconds unless set). `SPELL_LENGTHS` sets which lengths it covers (1,2,3 unless set). A row is any 1 to 6 of the 33 pieces a spell may hold, numbered in base 33; the laboratory refuses most rows (`spell::check`), and a refused row is logged with its reason and not played. Rows of up to three pieces are logged in number order. From four pieces on they are logged in a fixed shuffled order, so a run that stops early has logged an even sample and later runs fill in the rest. A run stops after the block it is playing when the budget is spent. The test fails when any spell in the log breaks a rule.

`cargo test` runs the everyday test instead: every row of one and two pieces, each spell played twice, in under a second.

## The scene

One field (seed 7), one student on the tower with mana to spare who knows only the spell, and twelve wild enemies (four each of the first three) moved onto the route six to fourteen cells from the tower as they appear. An invasion is cast at this tower from the entry by seat 1 instead, with no student. Four seconds are played and every tick is watched.

## The rules

A **FAIL** breaks a rule no spell may break: a panic; more units than `MAX_UNITS` or pops than `MAX_POPS`; a ward outside 0 to `WARD_CAP`; damage taken that falls (a score that rises); a unit above its health; a unit off the world; a second play that ends differently; a mana cost of zero or less; an invasion with a summon before any trigger that makes no unit.

A **note** is about content: a spell the student never cast, or whose first payload found nothing to do in this scene (no damage, no field, no status, no ally, nothing raised, no ward), or an invasion whose summon sits behind a trigger that never fired.

## Where it lives

The full log stays on the computer that runs it, in `spell-log/` at the top of the repository, which git ignores (about 190 MB for every row of four pieces or fewer). This folder holds the small version:

| File | Holds |
| --- | --- |
| `summary.tsv` | One line for each block of 4,096 row numbers (64 blocks a line from four pieces on): rows, spells, failures, notes, a fingerprint of their behaviour, its key, and when it was last tested |
| `notable.tsv` | Every spell whose latest result is a failure or a note |
| `changes.tsv` | The spells whose behaviour changed in the latest run, at most 200 |
| `runs.tsv` | One line for each run: when, from which commit, how many spells it played, how many changed, and how much of each length is logged |

A line in the full log is: number, row, result, behaviour, fingerprint, key, last tested. A line is played again when its key no longer matches. The key covers the rule code (`battle.rs`, `body.rs`, `ragdoll.rs`, `arena.rs`, `spell.rs`, `content.rs`, `geom.rs`, `fx.rs`, `rng.rs`), the scene and rules file, the list of pieces, the content that is not a piece, and the data of each piece in the row, so a change to one piece replays only the rows that hold it. Adding, removing or reordering a piece renumbers every row, and the log starts again.

A spell that behaves differently when played again is not a failure. It is listed in `changes.tsv`, so a balance change shows which spells it moved.

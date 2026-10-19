# Second-order notebook

Stage 6, kept by the agent as the work happens, not afterwards. `B3 Build in self-questioning`.

The code and tests are the work; this file is the record about the work. It is where a decision the agent made alone, an assumption that turned out wrong, or something only the person can do becomes visible to the person. With the plan approved in advance, it is the main place those decisions surface before the person uses the tool. [`docs/procedure.md`](https://github.com/sgilson7/builder-setting/blob/main/docs/procedure.md#the-second-order-notebook) says why.

One row per assumption, surprise or open item. **Kind** is *divergence* (what was assumed, what turned out to be true, what changed), *finding*, or *worklist* (the person's to do). Number rows by round (R1.1, R1.2, then R2.1 after the first triage) so a later round does not renumber an earlier one. **Rule** is the rule the row produced for later sessions, and where it now lives (`CLAUDE.md`, a test, `tests/breaks.json`), or blank. **Status** is *done*, *open*, or *person*; every *person* row is named in `HANDOFF.md`. `scripts/notebook_check.py` checks the form of each row and that the notebook is not empty once `PLAN.md` holds a decision, not what the rows say.

| # | Kind | Assumed or found | What is true | What changed | Rule | Status |
| --- | --- | --- | --- | --- | --- | --- |
| R1.1 | finding | Transcription errors in the description. | "Noida" (2:02) is Noita; "Yap Yap" (18:37) is YAPYAP; "Diplomacy is not an option" (15:38) is the game of that name; "spawn base" (12:11) is spawn bait; "4v4 … 1v1, 1v1, 1v1" (7:24) is a four-player free-for-all. | Read this way in the brief and the journal; the quotes stay unchanged. |  | done |
| R1.2 | worklist | "Roughly one-fifth as strong as the Dukat Dock" (6:32). | The words are garbled. Read as: one-fifth as strong as the spell at full effectiveness. | Journal question 7 asks the person to confirm. Answered: "it is garbled but yeah thats the interpretation". |  | done |
| R1.3 | finding | "Everyone hates me, apologize, everyone says something" (7:16). | Reads as speech picked up from outside the description; no rule taken from it. | Left out of the design. |  | done |
| R1.4 | worklist | "5 different … colleges … like destruction, illusion" (3:08–3:24). | Skyrim has exactly five schools: Alteration, Conjuration, Destruction, Illusion, Restoration. Every example spell fits one of them (walls: Alteration; skeletons and elementals: Conjuration; charm: Illusion; healing: Restoration). | Proposed as the five schools; journal question 2. Answered yes: the five Skyrim schools. |  | done |
| R1.5 | finding | "College" names both the institution and a kind of magic (3:24). | Two meanings for one word breaks the glossary rule. | The kind of magic is a school; GLOSSARY.md rules out "college" for it. | GLOSSARY.md, School, Not: line | done |
| R1.6 | worklist | Online play for four players (7:24, 14:27). | Online play is a declared off-origin feature (R7) and needs kits/online. The battle takes no input, so each browser could run all four battles from the plans exchanged when the battle starts, and only the preparation needs a channel. | Journal question 1; no online code until the person confirms. Answered yes: online for four players, through kits/online. |  | done |
| R1.7 | worklist | Solo play is not described. | A four-player game with fewer people online needs rivals the computer plays, and the build's own tests need them too. | Journal question 3 proposes computer rivals. Answered yes: computer-played rivals. |  | done |
| R1.8 | worklist | How a match is won is not described. | Proposed: a tower at zero health is out; after the last day the tower with the most health left wins. | Journal question 4. Answered with a different rule; see R2.1. |  | done |
| R1.9 | worklist | Day length "3 to 5 minutes" and "3 to 5 days" (8:02, 12:55). | Proposed: 4-minute preparation, 4 days, both in data. | Journal question 5. Answered yes: 4-minute preparations, 4 days. |  | done |
| R1.10 | finding | Full 3D physics with ragdolls in four battles at once (21:05), in a core with no floats. | Q16.16 rigid bodies and ragdolls for four arenas at 60 ticks a second is the largest cost in the design. Vagrancy and Raiders of the Lost Mart did ragdolls; neither ran four arenas at once. | PLAN.md will put a tick-cost probe first, before content, with a fallback: simpler bodies for crowds, full ragdolls for what dies near the camera. |  | open |
| R1.11 | worklist | Teaching numbers in the mockup. | The tolerance band of 48, 22 and 12 pixels for 1, 12 and 25 students, and a mana cost of 38 against 90, are the agent's guesses to make the pictures concrete. | Shown as proposals; journal question 6. Answered yes: starting values. |  | done |
| R1.12 | worklist | Names in the mockups. | Applicant names, spell names (Acid Halo, Bone Parade, Frost Lance, Lava Imp Swarm) and enemy names (Hill Giants, Bramble Elves) were made up by the agent. | Placeholders a reader would see; the person accepts or replaces them. |  | person |
| R1.13 | finding | The mockups needed 3D pictures before any game exists. | They are drawn with three.js r160 in ../journal/mockups, outside this project, vendored with a checksum. The game will not use three.js: its page draws from the core's numbers (D3). | Nothing in this project changed. |  | done |
| R1.14 | divergence | Students cast invasion spells "at the beginning of the battle" (14:08). | Read as: each student who knows an invasion spell adds its enemies, scaled by that student's effectiveness, to the rival's battle when it starts, and casts nothing during it. | Journal question 8 asks the person to confirm the scaling. Answered yes. |  | done |
| R2.1 | divergence | Proposed in entry 1: a tower at zero health is out; the healthiest tower after the last day wins. | The person's rule: "the more damage you take, the lower your score goes … you cannot gain score, so it serves as essentially an infinite health pool". Towers never fall, and every player plays every day. | Score replaces tower health in the glossary, the brief and the plan. Score has no floor: it may go below zero. | GLOSSARY.md, Score and Tower | done |
| R2.2 | worklist | The match length. | The same answer says both "at the end of 3 days" and "at the end of 4 days", and answer 5 accepted 4 days. | The plan uses 4 days, as one value in data, and asks the person to confirm. | Answered: "lets try 4 days 4 minutes first", adjustable in Setup. | done |
| R2.3 | finding | How the game is deployed. | The person asked for "github pages as the deployment ci". The template publishes with scripts/publish.sh to a gh-pages branch; the person's choice is a GitHub Actions workflow that runs ./scripts/check.sh and deploys dist/ to Pages, followed by the browser check against the live address. Long Time Comin' and Raiders of the Lost Mart already deploy this way (`deploy.yml`: Chromium-only check.sh, upload-pages-artifact, deploy-pages), so the workflow is copied from theirs. | Recorded as a change to the template's deploy route in BUILDER-SPEC.md (R8 and R9 still hold: the same check.sh gates it, and the grant is in BRIEF.md). | BUILDER-SPEC.md change table | done |
| R2.4 | finding | "Ragdoll physics like vagrancy and the arc from raiders of the lost mart" (21:05) named two physics systems to copy. | Raiders has no ragdoll and no code called Arc: "the arc" is Arc Raiders, the game Raiders' brief names, and its drones are rigid bodies with spring-driven lean and spin. Vagrancy has the ragdoll: a Rust Verlet solver, 15 points and 8 passes, about 20 µs a fighter a tick. | PLAN.md decision 2: living enemies are sphere bodies with lean and spin after the drones; dead or heavily hit enemies become Verlet ragdolls after Vagrancy, capped per arena. | PLAN.md decision 2 | done |
| R2.5 | worklist | How many full ragdolls an arena can hold. | Not measured yet. The plan starts at 24 and lets the oldest settle and fade past it. | Plan question 5; the probe sets the final cap. | Answered: start at 24 and push it; the probe measured 96 within the frame budget (MEASUREMENTS.md). | done |
| R2.6 | worklist | Where the score starts. | The person's rule fixes only that score falls by damage and never rises. | Plan question 3 proposes 10,000, with no floor. | Answered: 10,000, "rather difficult if not borderline impossible" to reach 0. | done |
| R2.7 | worklist | The MVP's size, its 7 damage types and its battle length. | The agent chose the counts, the type names (Fire, Frost, Acid, Leaf, Earth, Bone, Spirit) and 90-second battles, and left sound out. | Plan questions 2, 4 and 6. The type names are a generated list a reader will see. | Answered: yes to the size and types, battles at most 90 s, NES chip sounds for everything. | done |
| R2.8 | worklist | How the browser check reaches a battle inside its time. | A 4-minute preparation is too long for CI on a software renderer. A Practice mode (one day, one-minute preparation) is a real option any reader can pick, so the check takes no path a reader cannot. | Plan decision 17; plan question 7. | Answered: yes, with a versus mode and trials besides. | done |
| R3.1 | finding | Match settings. | The person asked that days, preparation length and battle length be settings in the game, with a simulation speed for the battle. | Setup carries days, preparation, battle length and battle speed, each with the plan's values as defaults. |  | open |
| R3.2 | finding | The score's starting value. | 10,000, and the person wants reaching 0 to be close to impossible. | A balance probe reports the most damage any tower took in 200 bot matches; the start must stay well above it. |  | open |
| R3.3 | finding | "Use all the optimizations long time coming uses to get like a ton of orbs". | LTC's methods: one instanced draw per mesh and pass, 16 floats an instance, a static buffer for the scene uploaded once, a dynamic buffer re-sent each frame, a glow pass with additive blending and no depth writes, near-eye orbs drawn as glow only, and caps on live orbs. LTC never measured its orb load. | All of these are taken; a draw-cost test caps instances per scene and the probe measures frames with the battle at its caps. |  | open |
| R3.4 | finding | "Basic satisfying chip noises based on early nes games for all sounds". | Builder lets sounds arrive as bytes from the core. NES voices are two pulse waves (duty 12.5, 25, 50%), a triangle and a noise channel from a shift register. | The core synthesises every sound as 8-bit WAV bytes from recipes in data/sounds.json; the page decodes and plays them and computes nothing. |  | open |
| R3.5 | finding | "Some time trial / achievement seeking modes … like ape race has". | Ape Race has Time Trial with a four-tier medal ladder per map (Bronze, Silver, Gold, Author, the top hidden until the one below is beaten), personal bests with a New Best moment, and unlocks for the top tier. It has no achievement list; Galactic Survivors has one. | Trials with Ape Race's medal ladder, plus an achievements list after Galactic Survivors. |  | open |
| R3.6 | divergence | D17: the page keeps nothing between visits. | Trials' personal bests, medals and achievements mean nothing if they vanish when the tab closes. | D17 changed for one save key holding bests, medals, achievements and settings; web/store.js is the only file that may name storage, and the core treats an unreadable save as a new one (after Galactic Survivors R4.22). | BUILDER-SPEC.md change table; no_storage.rs | open |
| R3.7 | finding | The probe's first run pushed walkers off the grid. | Separation ran after the wall check, so a crowd could shove a walker through a wall or past the edge. | Each tick is now: move, separate, then keep out of walls, so walls get the last word. ragdolls_stay_under_the_cap_and_walkers_stay_on_the_grid caught it. | stress.rs order; the same order in battle.rs | done |
| R3.8 | finding | Whether four arenas of physics fit in a browser. | Measured: 365 µs a tick in wasm for four arenas of 120 walkers with 96 ragdolls awake, 1.9 ms with 640 awake. The 4 ms budget holds with room to spare; a ragdoll costs about 2 µs a tick. | No change to 60 ticks a second. The ragdoll cap starts at 24 and is a setting the person can raise. | MEASUREMENTS.md | done |
| R3.9 | finding | Two physics tests passed with their behaviour broken (checks that cannot fail). | The corner test asserted one forbidden diagonal while the walker took the other, equally near one; the ragdoll settle test passed without friction because 1% damping a tick settles it anyway. | The corner test now asserts the rule (a diagonal needs both side cells open); friction has its own test, a ragdoll sliding at 6 m/s stops within a metre. | tests/breaks.json rows "a route cuts a corner" and "ground friction is left out" | done |
| R3.10 | worklist | The content list. | The agent wrote every name a player will read in data/content.json: 36 spell pieces in five schools, 12 enemies (8 wild, 4 only from invasions), 5 things to place, 4 helpers, and the type chart. | The person accepts or replaces them; the type chart gives each type two doubles and two halves, and fire on leaf and frost on earth are double as entry 1 showed. | | person |
| R3.11 | divergence | Entry 1 decision 17: effectiveness scales damage, area and duration and divides cooldown. | Scaling everything multiplies to far less than one-fifth at 20%, and very small areas would read as misses. | Effectiveness scales damage and duration fully and area by half again from a floor (area_floor_pct); cooldown is not scaled. 20% effectiveness is then about one-fifth as strong. | content.json balance | open |
| R3.12 | divergence | Plan decision 2: a body becomes a ragdoll on death or a heavy hit. | A living enemy turned ragdoll would need a way back up. | Only death makes a ragdoll; a heavy hit while alive throws the body into the air with tumble and lean (the drone model), and it lands and walks on. | | done |
| R3.13 | finding | Invasion spells and summons. | One Skeleton piece serves both: a student casting it raises skeletons for the tower; behind the Invade modifier the chain sends them to a rival. | Invade is a Conjuration modifier; a chain with Invade is never cast in the battle by students, and students who know one send it when the battle starts. | spell.rs check and is_invasion | done |
| R3.14 | finding | Glyph shapes. | Thirteen named strokes (loop, wave, zig, hook, spiral, vee, peak, line, arch, bolt, eight, square, tri) cover 36 pieces in one or two strokes each. | Glyphs are recipes the core expands, so each is the same on every machine and content.json refuses an unknown stroke. | glyph.rs STROKES | done |
| R3.15 | finding | Two of my hand-worked glyph scores were wrong at first. | The band also covers guide points within 50 units beyond the end of a drawn dash, so a dash of 100 covers 7 of 41 guide points (17), not a tenth; and a drop to y 550 is still inside a band of 50. | Expected values corrected after working them again by hand, with the working written in each test. | glyph.rs tests | done |
| R3.16 | finding | How students who do not fit on the tower cast. | Your mass-teacher plan ("a hundred mage students") cannot work if only the tower's seats cast. | Twelve seats on the tower and one per sub-tower cast at full range; every other student stands round the tower's foot and casts at 70% range. | battle.rs; students_beyond_the_seats_stand_on_the_ground_with_less_range | done |
| R3.17 | finding | How a glass wall breaks when the route never needs to go through it. | Enemies always walk around walls, so nothing would ever strike one. | Glass walls cast by students stand for a while and fall; placed glass and ivory walls last the battle; bait takes blows and breaks. Glass is the cheap, short wall and ivory the dear, lasting one. | battle.rs raise | open |
| R3.18 | finding | A cast's payload budget, counted. | With one budget of 32 shared by a cast and its nested triggers, an attack mage's chain of skeletons that each die into a ring of ghosts gets three rings of ghosts and then stops: 24 skeletons and 48 ghosts. | Budget per cast stays 32 in content.json; the person may raise it if invasions feel small. | one_payload_budget_is_shared_by_a_whole_cast_with_nested_triggers | open |
| R3.19 | finding | A break rotted after formatting. | cargo fmt split the formation line in spell.rs across three lines, so its break no longer matched. break_tests.py reported it as ROTTED rather than passing. | The break is re-aimed at the formatted line; breaks are run after fmt from now on. | tests/breaks.json | done |
| R3.20 | divergence | Plan: applicants are drawn each day (balance.applicants was 3). | Three a day caps a match at about 14 students, and the mass-teacher plan needs many. | 8 applicants a day, and a group interview takes them all at once on a harder lane (12 notes, windows a quarter tighter). A recruitment elemental interviews whoever is left at 80% of its summoning score. | content.json applicants; rhythm.rs group | open |
| R3.21 | finding | Which rival a student-taught invasion goes to. | The description says students' invasions are "added to the enemy's book"; with three rivals, which one is not said. | Each goes to the rival leading on score when the battle starts (the lowest seat on a tie), so invasions press the leader. | game.rs | open |
| R3.22 | finding | The furniture test passed with furniture gone (a check that cannot fail). | It looked only at where the walk ended, which was far past the table either way. | It now checks every tick that the archmage is never inside the table. | tests/breaks.json "furniture does not stop the archmage" | done |
| R3.23 | finding | Breaks written as exact text rotted after cargo fmt reflowed long lines. | Four breaks so far matched nothing after formatting. | tools/add_breaks.py writes a break from a fragment found on exactly one line of the formatted file, and asserts every match. | tools/add_breaks.py | done |
| R3.24 | finding | What a reader finds in the library. | Every piece but the two starters is in the pile once, with six tomes that teach a whole spell; a book whose piece is already known is skipped. | 34 books and 6 tomes a match, in an order each archmage's seed shuffles. | prep.rs Prep::new | done |
| R3.25 | divergence | Plan decision 6: enemies at the tower attack it on their attack timer until killed. | The balance probe's first run: a tower took 65,000 to 88,000 over a match against a 10,000 start, because enemies that reached it pounded it for the rest of a 90-second battle. | An enemy that reaches the tower strikes once, for its attack × leak_mult (3), and is spent, as in most tower defence; invasion enemies strike at × invasion_leak_mult (1) because they come in numbers. | content.json; a_ward_takes_damage_before_the_score_and_the_score_never_rises | open |
| R3.26 | divergence | Students who know an invasion each send it. | A class of 30 sent hundreds of skeletons, up to the 400-unit cap. | Students' invasions are pooled per spell: one send at the class's mean effectiveness, numbers × the square root of the class size (4 students 2×, 25 students 5×). | game.rs begin_battle | open |
| R3.27 | finding | Invasions cast from the book. | At the spell's mana an attack mage could send seven a day; and a bot casting all at once never earned the mana a person regenerates over four minutes. | The book charges 150% of the spell's mana; the archmage's own invasions send 150% of the numbers; bots cast invasions last and wait for mana in time they have left. | content.json invasion_mana_pct, archmage_count_pct; bot.rs | open |
| R3.28 | worklist | Plan decision 10's probe target: every plan wins at least one match in ten. | 40 four-bot matches (2026-10-16): Balanced 25, Mass Teacher 13, Spellcrafter 2, Attack Mage 0. Mean damage taken per match 1,546 to 3,419, worst 6,165 against a 10,000 start, so reaching 0 stays out of reach. | Not met for the spellcrafter and attack-mage bots. Wild numbers are at 60% (wild_pct) so invasions are the main threat. The bots are rough players; the person judges the plans by playing them. | MEASUREMENTS.md | person |
| R3.29 | divergence | R1.5: GLOSSARY.md rules out "college" for a school of magic. | The copy test reads words, not senses: it also refused "joined the college", the institution the person named. It refused "Turn" (rotate) for Day and "The Attack Mage" (the person's own name for the plan) for Invasion. | College has its own entry and School's Not: line drops it; Day keeps only "round"; Invasion keeps only "raid". The sense of "college" is now held by the glossary's definitions, not by the test. | GLOSSARY.md | done |
| R3.30 | divergence | Computer rivals invade from the first day. | In the first screenshots of a versus match, the attack-mage bot sent a large invasion at a new player on day 1, before the player could prepare anything; on day 1 every score ties and the leader rule picks the lowest seat, the player's. | Computer rivals send no invasions on the first day. People playing each other online may. | bot.rs day(invade) | open |
| R3.31 | finding | The realtime kit's web/input.js. | It reads keys only while pointer lock (or its fallback) is engaged; this game never locks the mouse. | web/keys.js does the kit's job without the lock: keys to action bits, taps between frames kept, typing in a text box ignored. input.js is not copied. | web/keys.js | done |
| R3.32 | finding | The browser check expected a book to teach exactly one piece. | Each browser draws its own seed; WebKit's first book was a tome, which teaches a whole spell's pieces at once. | The check asks for at least one more piece. | tests/browser/interactions.py | done |
| R3.33 | finding | Kits taken: realtime (fx.rs, rng.rs, ticks.rs) and canvas (draw.rs, render2d.js). | Neither changes a default. draw.rs gained the overlay's colour roles and shapes (one per school) and a 1280 × 720 canvas; render2d.js lets the stylesheet size the canvas. | Recorded here; the spec table lists the defaults that did change (deploy route, D17, sound bytes). | BUILDER-SPEC.md | done |
| R3.34 | worklist | PLAN.md decision 15: online play for four. | Not built in this pass; solo, versus, practice and trials come first so the deploy has a whole game. The title says online play is planned. | Built with kits/online: a host and up to three joiners by room code or pasted codes; plans and invasions cross the wire and every browser runs all four battles. | `crates/core/src/online.rs`; `session/tests.rs` online tests; `tests/browser/online.py` | done |
| R3.35 | finding | The description names three people (8:43, 9:05) in its examples of a battle. | The repository is public, and Builder's procedure leaves people's names out of a public repository. | BRIEF.md quotes those cues with [a friend] in place of each name and says so; everything else stays word for word. | BRIEF.md | done |
| R3.36 | finding | Publishing the history. | Early commits' BRIEF.md held the three names R3.35 removed; rewriting the history was refused by the session's safety check, so the published branch starts from one commit of the redacted tree. | `public` is the published branch (pushed as main); the full history stays on the author's machine on `main`. Every later change is a commit on `public`. | | done |
| R3.37 | divergence | Online play keeps every browser's match the same. | The Attack Mage bot aimed at the rival leading *the local seat's* view (`leader(local)`), so two browsers gave the same bot different targets; and a player who left mid-match would have stalled everyone waiting for a plan. | Every bot aims at `leader(own seat)`; invasions are ordered by sender before a battle; a departed seat is marked gone and brings only what it had sent. | `every_browser_of_an_online_match_ends_with_the_same_scores`; `a_joiner_who_leaves_mid_match_does_not_stop_the_others` | done |
| R3.38 | finding | A plan from another browser is untrusted input. | `spell::check` indexed pieces without a bound, so a chain with an unknown piece id panicked (the new test found it). | `online::sane_chain` and `sane_plan` check every index before a remote plan or invasion is used; a bad plan counts as the seat leaving. | `a_plan_from_another_browser_is_refused_when_an_index_is_out_of_range` | done |
| R3.39 | finding | Messages for a later day. | A player who reaches the next day's preparation first sends invasions while another is still reading the summary; applied at once, `start_day` would have cleared them. | Online events wait in a queue until their own day's preparation. | `pump_online` in `session.rs` | done |
| R3.40 | worklist | The room-code path over public relays, STUN and two real networks are not tested by a machine. | `tests/browser/online.py` covers pasted codes on one machine. By hand on 16 October 2026, two Chromium browsers on one machine met by room code KNHDB over the public relays on the live site and started a match; two real networks and strict NATs remain untried. | Try a room code with a friend on another network before relying on it, and write what happened in TRIAGE.md. | | person |
| R4.1 | divergence | T1: "a spell by default that creates walls and towers … they just take up to a minute to spawn and consume mana". | Walls and sub-towers had to be read from a book, and with mana they stood at once. | A new archmage knows Glass Wall and Raise Sub-tower (`balance.start_pieces`). A placement costs its mana at once and rises `raise_s` later (glass 15 s, bait 15 s, lava 20 s, ivory 30 s, sub-tower 60 s), while the archmage does other things; several may rise at once. A rising wall holds its cells and counts against closing the route. The golem stays as the no-mana choice. The raise times are the agent's; the person may change them in content.json. | `a_wall_placed_with_mana_rises_after_its_delay_and_not_before`; `a_new_archmage_knows_the_starting_pieces_in_the_data` | done |
| R4.2 | finding | A wall still rising when the preparation ends. | It is not raised, and the day's mana it cost is gone (mana refills each day). A 60 s sub-tower placed late in a one-minute practice day never stands. | Kept simple; the map lists what is rising with the seconds left. | | done |
| R4.3 | finding | T3: the map could not be left. | CSS hid every side panel while an overlay was open (`body[data-overlay="on"] .panel-side`), and the map's own panel, with its Close button, was a side panel; Esc did nothing for the map and walking was masked. A check that drove the map through `act` could not see it. | The map's panel has its own kind and sits left of the map; E at the map table opens it, Esc closes it, walking off closes it. | `e_opens_the_map_at_the_map_table_and_esc_closes_it` | done |
| R4.4 | finding | T6: white text on white chips. | In the dark colour scheme `--ink` turns near white, but chip buttons kept a fixed pale background (`#e9e0ff`); gold buttons also took the light ink. Every check ran in the light scheme. | Chips and rows take their background from theme tokens; button text stays dark on gold. A browser check measures contrast in both schemes, and fails at 1.1:1 with the old style. | `tests/browser/contrast.py` | done |
| R4.5 | finding | T2 and T7: how to summon a helper and send an invasion. | Both worked, but nothing on screen said how. | Two how-to topics (help.10, help.11), also in the glossary's How to tab. | `the_glossary_opens_over_any_screen_lists_every_piece_and_pauses_a_solo_day` | done |
| R4.6 | divergence | T8: a glossary to open at any time. | Hover hints only. | A Glossary button and the G key open it over any screen: every spell piece by number (its school, kind, text and numbers, and whether you know it), your saved spells, every GLOSSARY.md word and the how-to topics. A game against the computer waits while it is open; an online one cannot. | `the_glossary_opens_over_any_screen_…`; `an_online_day_does_not_wait_for_the_glossary` | done |
| R4.7 | divergence | T5: "make the battle window independently draggable, zoomable, move withinable". | Read as the camera inside each battle view, not moving the view's frame on screen. | Each of the four views has its own orbit camera: drag turns it, right-drag or Shift-drag moves it, the wheel zooms; Reset the views puts them back. The starting orbits are the two old fixed cameras. If the person meant moving and resizing the view frames themselves, that is a new row. | `each_battle_view_turns_moves_and_zooms_on_its_own`; `the_starting_orbits_are_the_old_fixed_cameras` | done |
| R4.8 | finding | My hand-worked angles for the close camera. | I wrote 40.9 m, 39.5° and 33.7°; the test that compares the orbit with the old camera found the eye 1.2 m off. The right values are 40.3 m, 40.2° and 35.8°. | A miscount, caught by the test. | `the_starting_orbits_are_the_old_fixed_cameras` | done |
| R4.9 | finding | The glossary button and the glossary panel shared the id `glossary`, so setting the button's label emptied the panel. | Found by looking at the page, not by a test. | The button is `glossary-button`; the contrast check opens the glossary from it and needs its rows. | `tests/browser/contrast.py` | done |
| R4.10 | finding | Helpers at work and the station panel were both side panels in the same place, one over the other. | A rising wall was also listed as a Builder Golem. | The work list has its own place on the left and names a rising wall as rising. | | done |
| R4.11 | finding | The browser check failed now and then, waiting for two saved spells, and passed when run again. | First blamed on two quick clicks, wrongly: it failed again after that fix. The page's seed is random, and when the first book read is a book of a whole spell, a second spell is saved before the laboratory, so saving one makes three and never two. A check that hard-coded a count the seed can change. | The check waits for one more spell than it saw before saving. (The click waits stay.) | `tests/browser/interactions.py` | done |
| R4.12 | finding | T9: map buttons dead after a placement. | The rising list I added in T1 put a per-tick progress bar in the map panel, so its JSON changed every frame and ui.js replaced the panel under the pointer; the browser only fires a click when press and release hit the same element. My T1 browser look clicked with no pause between press and release, so it never saw it. | The rising list has its own panel and its bar moves in whole seconds; ui.js now acts on a press and release of the same action even across a redraw; the browser check presses and holds a map button for 150 ms after placing a wall, in three engines. | `the_map_panel_does_not_change_while_something_rises`; `tests/browser/interactions.py` | done |
| R4.13 | divergence | T10: "the chip for the spell name should be way bigger". | Read as the message shown when a book teaches a piece. | A card with the piece's glyph drawn as its icon in its school's colour, its number, school and kind, its text and its numbers, for `balance.card_s` (3) seconds or until the next book; a tome shows its whole spell. The glossary's spell pieces are the same cards. | `a_book_read_shows_its_card_for_three_seconds_or_until_the_next_book` | done |
| R4.14 | finding | WebKit refused the browser check's `wait_for_function`. | Playwright runs a page function by eval, and the page's CSP has no 'unsafe-eval'. | The check polls from Python. | `tests/browser/interactions.py` | done |
| R4.15 | finding | Commit 'Map buttons work while something rises…' was pushed with clippy and the copy test failing. | My command ran each check with `;` instead of `&&`, so the commit and push ran after the failures; the string said "tome", which GLOSSARY.md rules out for Book. CI's check job would refuse the deploy. | Fixed in the next commit; the chain is `&&` again (the rule from R3.36's round). | | done |
| R4.16 | divergence | T11: "an animated landing page like ape race has". | Ape Race's title, looked at on its live page: a live 3D world behind the menu with a drifting camera, a big slanted gold title and a tagline, the menu and a Controls panel side by side, What's New with a New badge, the version in the corner. | The same layout over a real battle: a computer archmage's tower holds a day while the camera circles once in `balance.demo_turn_s` (90) seconds, silent; a new day starts when one ends. The title and panels animate in, and stand still for a reader who asks for reduced motion. The tagline "Read the floor. Teach the class. Hold the tower." is mine. | `a_battle_plays_behind_the_title_while_the_camera_circles_it` | done |
| R4.17 | divergence | T12: "a whats new tracker". | After Ape Race's: version notes on the title, a New badge until they are opened, minor patches hidden behind a toggle. | `data/news.json` (newest first; its first version is the game's version, shown in the corner). Opening the notes keeps that version in the save. The notes are mine, written from this session's TRIAGE rows; they are checked by copy.rs like every data file. | `whats_new_is_marked_new_until_its_notes_are_opened` | done |
| R4.18 | divergence | T13: "little mana symbols to show the mana cost of spells when created". | Mana was a number inside a sentence. | The core sends each cost as its own value and the page draws a mana drop with the number: the draft spell, each saved spell, each payload in the laboratory, the map's placements, every card and the glossary. A modifier shows none, since what it costs depends on the spell it is in. | `mana_costs_are_sent_for_the_mana_symbol` | done |
| R4.19 | divergence | T14: "gotta add the copy link". | Read as Galactic Survivors' room link. | The host's lobby has Copy the link: this page's address with `#room=CODE`. Opening it takes the room out of the address, opens the Online screen and joins with the player's default name. A joiner can change their name only before joining; the link skips that step. | `the_host_s_lobby_offers_the_room_code_and_a_link_to_copy` | done |
| R4.20 | finding | T4: the spell-space scene. | With the crowd spawned at the entry, 580 of the 641 short spells were never cast: in four seconds no enemy came within a student's range, so every rule passed on spells that did nothing (a check that cannot fail). | The scene moves each wild enemy, when it first appears, onto the route six to fourteen cells from the tower; now every short spell is cast, and the README says so. | `crates/core/tests/support/spell_rules.rs` `play`; `tests/spell-log/README.md` | done |
| R4.21 | finding | T4: invasions whose summon is behind a trigger. | The laboratory accepts a chain such as `invade fire_bolt on_expire skeleton`, but an invasion is cast from the entry at the entry, so the payload before the trigger has nothing to reach and 471 such four-piece invasions never summoned anything: the player pays mana for nothing. | The laboratory now refuses it: Invade goes first (`lab.invade_first`, as the book and help already said) and needs a summon before the next trigger (`lab.invade_without_summon`). The re-run logbook then found 150 more of the same kind, with Invade after a trigger, which led to the first rule. | `spell::check`; `an_invasion_summons_before_any_trigger` | done |
| R4.22 | finding | T4: what the spell space covers. | Every row of up to four pieces was logged (1,222,980 rows, 596,575 spells): no panic, no unit count past the cap, no rising score, no second play that differed. Five and six pieces (about 40 million and 1.3 billion rows) are sampled by the logbook's shuffle when a run is given the budget. | MEASUREMENTS.md "Spell space". The everyday test plays every row of one and two pieces in `cargo test`. | `every_chain_of_one_and_two_pieces_keeps_every_rule`; tests/breaks.json rows "an invasion's summon makes no unit", "a unit starts above its health", "a spell costs no mana", "a summon reads past the enemy list" | done |
| R5.1 | finding | T19: recruitment never hires. | `Prep::step` passes the held bits to the interview, and `Lane::press` runs for each tick a note key is down; a tap lasts about six ticks, so each hit also scores about five strays at −10. The browser check never plays an interview, and the rhythm tests press a key for one tick, so no check could see it: a check that cannot fail, again. | Plan decision 20: presses on the down edge, and a test that holds keys as a person does. | PLAN.md 20; `an_interview_played_with_held_keys_hires_the_applicant` | done |
| R5.2 | finding | T26: why 100% was easy. | `glyph::score` counts the share of the line inside the band and the share of the guide covered; any steady trace inside the band scores 100, and the score is the effectiveness. | Plan decision 25. | PLAN.md 25; `effectiveness_climbs_steeply_and_only_a_flawless_drawing_reaches_the_top` | done |
| R5.3 | divergence | T17's wireframe (`MageTowerBattle/image.png`) was not attached to the message; found in the project's parent folder. | Read as columns blue (modifier), red (payload), yellow (trigger), a red tile on the left for the spell, the small red square as the trigger's link to the next set. Two rows unexplained. | Asked as round 5 question 1; answered "yes, rows are branches if that is possible in the game" (R5.5). | DECISIONS.md 2026-10-16 (round 5) | done |
| R5.4 | worklist | Round 5's names and numbers are mine. | The self spells (Mana Spring, Deep Well, Quiet Mind), the sorting game's ten books, the golem's 45 s, the per-cell prices, the summon limits, the curve's power 4, the 2:30 reading. | Answered by number; the summon limit moved to each student (R5.6). The names Mana Spring was taken by an older piece, so the first self spell is Rising Tide. | DECISIONS.md 2026-10-16 (round 5) | done |
| R5.5 | divergence | Rows are branches. | A spell was one line. | A Branch piece (known from the start) splits what the first trigger starts into two rows; a spell has one branch at most, after its first trigger, and each row with what comes before it must be a spell on its own. The second row counts at the first trigger's weight. Spells may hold 8 pieces, up from 6, so a branch has room. | `a_branch_starts_both_rows_when_its_trigger_fires`; `a_branch_comes_once_after_a_trigger_and_each_row_must_be_a_spell` | done |
| R5.6 | divergence | "the limit should be per student, not per tower". | Planned as 8 a tower. | At most 4 summons alive for each student, 2 skeletons a cast (67% of the count), each lasting 12 s, summoning spells at 150% mana; invasions keep their numbers. The spell space's most units at once fell from 36 to 16. | `a_student_keeps_at_most_four_summons_alive_and_each_lasts_twelve_seconds`; `an_invasion_keeps_its_numbers` | done |
| R5.7 | finding | The new rules together made the game far harder than asked. | The balance probe after T26, T29 and T30: mean damage a match 5,206 to 9,299 and a worst of 14,928, past the 10,000 start. Effectiveness now climbs steeply, so the bots' skill (65) drew at 65 and taught at about 17%; and every invasion now hits three rivals. | Three changes: an invasion sent to every rival arrives at 35% of its numbers at each (`invasion_spread_pct`); a bot's skill reads as a careful player's drawing (65 draws about 84); Normal is 110% of the wild count and 105% of health. Result: mean 1,788 to 4,095 (before round 5: 1,546 to 3,419), worst 7,602 (before: 6,165): somewhat harder, and 0 stays out of reach. | MEASUREMENTS.md; `tests/balance_probe.rs` | done |
| R5.8 | finding | Two of my tests leaned on luck. | The online equality test needed a joiner to read the Invade piece; once the pile held known books its order changed and the test failed. The reading test assumed a read takes 2.5 s. | The online test gives its joiners Invade; reading tests play the sorting game and accept a book already known. | `session/tests.rs`, `prep/tests.rs` | done |
| R5.9 | finding | A check that could not fail, written this round. | The first golem test ended with an assertion that accepted any result. | Removed before commit. | | done |
| R5.10 | worklist | Round 5's names and numbers are mine. | Rising Tide, Deep Well and Quiet Mind and their numbers; the librarian golem's 30 mana and 45 s; the sorting game's 10 books, 1.6 s fall and 85% speed-up; the seal's 8 turns; per-cell prices; difficulty levels (Easy 80/85, Normal 110/105, Hard 140/125, Brutal 180/150 percent of count and health); invasion spread 35%; the Crash Course trial's target, now 60%. | All in `data/content.json` for the person to change. | | person |
| R5.11 | finding | The Crash Course trial asked for an 80% class average. | Under the steep curve, 80% needs a near-flawless drawing for eight students. | The target moved to the trial's data at 60%; the text says why. | `data/content.json` trials | done |
| R5.12 | finding | The builder's tiles drew their names in the school's colour; Alteration green on paper measured 2.6:1. | Caught by `tests/browser/contrast.py` (T6's check) before the push. | The icon keeps the school's colour; the name stays ink. | `tests/browser/contrast.py` | done |
| R5.13 | finding | Ten break rows rotted this round, because their code was rewritten (reading, saving, seating, raising, the map panel, mana). | `break_tests.py` reported each as ROTTED, none as missed. | Each was rewritten against today's line; the sorting test gained a nine-is-not-enough step so a break that ends sorting early is caught. | `tests/breaks.json` | done |
| R6.1 | divergence | Round 5's screens were built to their approved mockups. | They were drawn by the generic panel renderer (rows, chips, buttons) and lost the mockups' columns, bars, field, dial and shelves; the person saw it at once (T32). I checked each screen worked, not that it looked like its mockup. | The page gained a few generic blocks the core lays out (columns, meters, a table, a board of placed boxes, button groups with one chosen, keys, a note, shelves) and drag and drop. Office hours, the builder, the seal, the sorting game and the map table were rebuilt to their mockups and compared side by side. | `CLAUDE.md` rule 13 | done |
| R6.2 | finding | Sub-tower seats were numbered from `SUB_SEAT` 10. | The tower has 12 seats, so "Sub-tower 1" and "Sub-tower 2" were tower seats 11 and 12, and office hours offered only Towers 1–10. No test seated a student on a sub-tower by choice. | `SUB_SEAT` is 32; content refuses `tower_seats` above it; a test puts a student on a sub-tower and finds them there. | `a_student_put_on_a_sub_tower_stands_on_it_not_on_a_tower_seat`; content check | done |
| R6.3 | divergence | Choosing a seat at office hours put the student there. | Two students could choose one seat; the second was quietly given the first free seat. | A seat holds one student: whoever had it takes the mover's old place, so dragging one student onto another trades them. The ground holds any number. The map table can choose a seat (T33), nothing else of office hours. | `a_seat_holds_one_student_and_two_trade_when_one_takes_the_others` | done |
| R6.4 | divergence | The pictures of the field would compute who sits where for themselves. | That would be a second copy of the battle's seating rule. | The rule moved out of `Battle::new` into `battle::seating`, which the battle, office hours' board and the map's discs all use, so a picture shows where a student will stand, including those given the first free seat (pale). | `seating_gives_choices_first_then_the_first_free_seat` | done |
| R6.5 | finding | The seats would be drawn where they are in 3D. | In 3D the 12 seats stand at three heights on a ring 2.3 cells round the tower; seen from above they overlap. | The map and office hours draw them on a flat ring 3.6 cells round, from the top, clockwise: a diagram, not to scale. Office hours' board shows the part of the field round the seats and any sub-towers, 8 wide to 5 high and at least 18 cells across, as the mockup crops. | `overlay::spots` | done |
| R6.6 | finding | The overlay canvas cleared itself each frame. | It never cleared; every overlay began with a full-screen shade, which hid that. The seal and sorting game now draw over a panel, without a shade, and left trails. | The clear op wipes the canvas before it fills, and a see-through `clear` colour exists. | `web/render2d.js` | done |
| R6.7 | divergence | The sorting game's shelves were drawn on the canvas, and clicks mapped to shelves by x. | The mockup's shelves hold the books shelved and a key cap; that is easier as page buttons. | The shelves are buttons (`sort.shelf`), the canvas draws only the falling book over the panel and takes no pointer, and `shelf_at` is gone. The seal works the same way: the dial on the canvas, the words beside it. | `the_seal_lists_its_rings_and_the_sorting_game_has_a_shelf_button_each` | done |
| R6.8 | divergence | Each mockup's every detail would be built. | Left out: the sorting game's clock chip (the game keeps no time for it) and office hours' "hired on day 1 from an interview of 82" (not recorded). The builder's tray shows one blank tile with a count, not one blank per unread piece: twenty blanks pushed the tray off the panel. | Recorded here. | | done |
| R6.9 | worklist | The new words are mine. | Office hours, map seating, seal, sorting game and builder labels (listed at the foot). "Seal and keep the spell" became "Seal the spell", as in the mockup. | For the person to read. | | person |
| R6.10 | finding | The contrast check covered the new screens. | It measured the title, the glossary and the laboratory only. Measuring the laboratory again in the dark theme found two mana numbers drawn in the theme's pale ink on a light card and on a gold button (1.0:1 and 1.7:1). | A mana number takes its button's colour, the spell's card keeps dark words, and the check now also measures office hours and the map's seating, failing if the walk does not reach them. | `tests/browser/contrast.py` | done |
| R7.1 | divergence | The mockup's look, built in the game. | The mockup was drawn with three.js, which casts real shadows; the game's renderer has none. | Each thing stands on a soft dark disc instead, as in Ape Race. | `room.rs` `shadow` | done |
| R7.2 | finding | The ink outline was part of the look. | Ape Race has none, and the outline pass drew every shape twice. With it gone, light on the facets tells shapes apart, and a room with sixty students fits under the watch screen's 16,000-instance cap. | The ink pass is removed from the core, the page and the shader; the battle field takes the same light and facets, with an earth slab under the grass (T36). | `render.rs` tests, `the_room_draws_its_furniture_once_and_its_people_each_frame` | done |
| R7.3 | worklist | Enemies and helpers would be rebuilt too. | Only the mages were rebuilt with jointed arms; the enemies and helpers keep their sphere-built models (entry 1, decision 37), now faceted and newly lit. | For the person to judge by playing: rebuild them the same way, or keep them round. | | person |
| R7.4 | finding | The camera would come down within six metres of a station. | The seven stations' six-metre circles cover most of the room, so the camera would have dipped almost everywhere the archmage walked. | It starts to come down at 4.5 m and is all the way down at 3.2 m, a little past a station's 3 m reach. | `the_camera_stays_high_far_from_stations_and_comes_down_behind_the_archmage_at_one` | done |
| R7.5 | finding | Glow halos could be faceted like everything else. | A faceted halo fades in flat patches, which shows its edges. | A smooth `Halo` mesh, for the glow pass only, is the one exception to flat faces. | `every_face_is_flat_so_shapes_read_as_facets` | done |
| R7.6 | finding | Nudging each block's corners would make the wall look hand-cut. | Every stone shares one mesh, so every stone was the same wedge and the wall read as a saw-tooth. | The nudge is halved, and half the stones are turned end for end. | `room.rs` wall loop | done |
| R7.7 | finding | A staff could turn with the arm that holds it. | Turned about the shoulder, a staff held in the hand points backward as the arm lifts. | Parts held in the hand (`grip_r`) move with the hand but stay upright. | `casting_lifts_the_right_arm_about_its_shoulder_above_the_head` | done |
| R7.8 | divergence | The map table would show the player's own field. | The room's furniture is built once a day, and the layout changes during it. | The table shows a fixed field in miniature; the map it opens is the real one. | | open |
| R7.9 | finding | My screenshot walk reached each station. | Holding two keys ran the archmage into the dais, so one picture I first took of the podium was the overview. | The walk now steers by the archmage's position until the station reports it is reached. | | done |
| R7.10 | divergence | The cut-away wall serves every camera. | With the camera low inside the room, the missing near wall showed the dark beyond. | The near wall is a second static part, drawn only when the eye is inside the room and below the wall's top. | `the_wall_steps_down_toward_the_camera_and_the_near_part_is_kept_apart` | done |
| R7.11 | worklist | The look's colours and distances are mine. | The dusk and daylight colours, the haze's distances, the sun's direction, the light weights, the HUD's dark glass, and the camera's 5.6 m behind and 5.2 m up; and the What's new text for 1.6.0, which is mine (listed at the foot). | For the person to judge by playing and to read. | | person |
| R7.12 | finding | Online still works with the new look (T37). | The drawing reads nothing an online match shares. | `ONLINE CHECK PASSED in chromium, firefox, webkit`. | `tests/browser/online.py` | done |
| R7.13 | divergence | Presentation numbers live in `data/*.json` (`CLAUDE.md` rule 4). | The haze's distances, the sun's direction and the light weights are constants in `room.rs`, `scene.rs` and `web.rs`, following the light uniforms already in `web.rs`. | To move to `data/display.json` in a later round. | | open |
| R7.14 | finding | The browser check's waits poll until their limit. | One wait read the route panel with Playwright's `inner_text`, which itself waits 30 s for the element and throws; one WebKit run in four ended there, the other three passed. | The wait reads the panel only if it is there now and keeps polling to its own limit. | `tests/browser/interactions.py` `text_now` | done |
| R8.1 | finding | Tiles named their pieces well enough (T38). | A playtester could not tell what a spell did; the pieces' own sentences and numbers lived only in the glossary's cards. | Each tile carries its sentence and the card's line of numbers; the spell's box reads the draft out in order. | `every_tile_says_what_it_does_and_the_spell_reads_out_in_order` | done |
| R8.2 | divergence | The sorting game stays a falling-book game (T20, T39). | It was too fast. | A pile on the floor is dragged to the shelves, untimed. A perfect sort reads a second book, skipping known ones, so it teaches something new while one is left. Keys 1 to 5 still shelve the first book, which keeps the browser check playing. | `a_book_put_on_a_wrong_shelf_goes_back_to_the_pile_and_a_perfect_sort_teaches_two` | done |
| R8.3 | divergence | A raid reaches every rival, like T29's invasions. | A student cannot be on three fields at once. | A raider goes to the one rival you pick at the invasion book; they come home the next day. The attack mage sends its Reckless and Hunter students at the leader from the second day (R8.12). | `a_student_sent_to_raid_fights_on_that_rivals_field_and_not_on_their_own`, `a_rival_sends_its_reckless_students_to_raid_the_leader_but_not_on_the_quiet_first_day` | done |
| R8.4 | finding | Raiders could break online matches. | A battle is run in every browser from the plans. Who raids where, how late, and their temperament and name travel in each student's plan, and a remote plan is checked for them. The online equality test now sends a raider from each joiner. | `ONLINE CHECK` and the equality test with raids. | `every_browser_of_an_online_match_ends_with_the_same_scores`, `online::sane_plan` | done |
| R8.5 | divergence | Students' trees would run only for raiders. | Every mage on a field ticks their temperament's tree, at home too. A student's temperament comes from a hash of their name and school, so it costs the match's chance nothing; the trees are data in `content.json`. A Steadfast student behaves as before, but casts a ward or a mana spring only once something is in reach. | Trees for every mage, at home and raiding. | `mind.rs`, `content.json` temperaments | done |
| R8.6 | worklist | The numbers for health and speed. | Health 40 plus 20 for every 100% summed; speed 2 m/s plus 0.4 m/s for every 100%. Raiders join 4 s into a battle, 2 s later for each fall, up to three falls. Mine, in `content.json`. | For the person to judge by playing. | | person |
| R8.7 | finding | A journey could fail a send. | Holding to the brief: it never fails, it only delays arrival. My first run cost 14 s for seven falls, so the jump was lengthened and only three falls count. | `prep::LATE_FALLS` | `a_raider_sent_from_the_invasion_book_jumps_three_gaps_and_a_fall_makes_them_later` | done |
| R8.8 | divergence | Glossary words stay out of every string. | `raid` is ruled out for Invasion and `health` for Score, and both are now right for something else: a raid sends a student (T40), a mage has health (T42). Following `copy.rs`'s own rule, the glossary keeps its Not lines and `copy.rs` allows the two words in `data/` with the reasons. New entries: Health, Journey, Raider, Temperament. Battle's `wave` is avoided. | For the person to confirm. | `crates/core/tests/copy.rs` ALLOWED | person |
| R8.9 | finding | My first test of a fallen mage could fail. | It checked a fallen student's mana stayed put, but nothing was in reach and a full well stays full, so it passed with the rule broken; `break_tests.py` found it. It now compares a fallen student with a standing one who has a skeleton in reach. | | `a_fallen_mage_casts_no_more_and_a_raiders_bolt_hurts_a_student` | done |
| R8.10 | worklist | The new words are mine. | Temperament names and what they do, the tree's labels, the raid, journey and sorting texts, and What's new 1.7.0 (listed at the foot). | For the person to read. | | person |
| R8.11 | divergence | Wild enemies fight every mage. | They strike a student on the ground within reach; ground fields and lingering zones still hurt only units, not mages. | Left for a later round. | | open |
| R8.12 | finding | Raids would leave the game about as hard as round 7. | The balance probe, 40 matches each: round 7 took 1,788 to 4,095 a match (worst 7,602). With every computer rival raiding, 3,266 to 5,075 (worst 8,366). A raider's hits on the tower barely mattered (they fall fast); raiding by all four rivals made the difference, with about a thousand more points off every tower a match. | Only the attack mage raids; a raider's hits reach the tower at 15% (`raid_tower_pct`) and its summons are held like a student's. Result: 2,200 to 4,224 (worst 7,749): a little harder, and 0 stays out of reach. On the way I also found that a student had stopped casting more than one ready spell a tick, which I had changed by accident; it casts every ready spell that reaches again, as before. | `tests/balance_probe.rs`, MEASUREMENTS.md | done |
| R8.13 | finding | The new screens kept their contrast. | Measured for the first time, office hours' gold chips (a temperament, "still running") drew the dark theme's pale words on gold, 1.7:1. | A gold chip keeps dark words; the contrast check now also measures the tree, the book pile and the invasion book. | `tests/browser/contrast.py` | done |
| R8.14 | finding | The old break rows still pointed at real code. | The full run caught 171 of 177. Four rows broke the falling-book game, whose tests went with it, and so could not fail any more (one still "passed" by running no test). Two rows' lines had moved. | The four were taken out, since the pile's rows (`a wrong shelf takes the book`, `a perfect sort teaches one`) cover what replaced them; the two were re-aimed and are caught. | `tests/breaks.json` | done |
| R9.1 | divergence | A camera behind the archmage, facing the station, follows them the way Ape Race does (T35). | The keys move the archmage along the room's axes, not the camera's. Once the camera turned to face a station, W no longer meant up the screen, and walking turned the camera further: around the map table it swung right round (T46). | The camera keeps the overview's heading, north, everywhere. It sits behind and a little right of the archmage and only tilts: how far ahead or back it looks is pulled toward each station by how near it is, so it slides from one to the next. The close camera is a little higher and further back, 6.4 m behind and 6 m up instead of 5.6 and 5.2, since a station south of the archmage is now in front of the camera. Reversion averted: camera-relative keys would have kept the turning. | `walking_anywhere_in_the_room_never_turns_or_jumps_the_camera`, `tests/breaks.json` `the camera turns to face the station` | done |
| R9.2 | divergence | The camera's height can follow where the archmage stands. | It came all the way down in 1.3 m, a third of a second of walking, and each 7.5 cm step moved it over 60 cm. | `Prep::dip` eases toward the closeness at a forty-fifth a tick, so the camera comes down over three quarters of a second however fast you walk. | `the_camera_comes_down_over_three_quarters_of_a_second`, `tests/breaks.json` `the camera drops at once` | done |
| R9.3 | worklist | The camera's new distances and pace are mine. | Behind 6.4 m, up 6 m, 1.7 m to the right, three quarters of a second to come down, and the look's pull from 7 m away; constants in `room.rs` and `prep.rs`, like R7.13's. | To judge by playing: walk up to each station, around the map table, and between the library and the book. | | person |
| R9.4 | divergence | The title would show either the tower or a sweep through a battle (T47 offers both). | The request puts the tower first and the battle sweep as the alternative "unless" it looks cool. | Built the tower: the camera 6.2 m up, 3 m behind the map table, looking 8.5 m out, one turn a minute (`demo_turn_s` 60, was 90). The battle sweep is not built. | `the_title_turns_in_the_middle_of_the_tower_while_the_archmage_walks_its_stations` | done |
| R9.5 | divergence | The title's archmage could be a computer rival playing its day. | Bots do not walk (`bot.rs` charges walking as time), so a bot's day shows nobody moving. | The title's own tour walks the archmage round six stations (the map table, under the camera, is left out), 3 s at each (`demo_stay_s`), in a match nobody plays. | `tests/breaks.json` `the title's archmage stands still` | done |
| R9.6 | worklist | The title's camera height, turn speed and the stations' order are mine. | | To judge on the title screen. | | person |
| R10.1 | divergence | A reason not to teach every student every spell (T50) could be a cost, a focus bonus or a limit. | Summed effectiveness (T42) rewards teaching everything; a soft cost would still be paid. | Spell slots: a student holds 2, 3 or 4 spells by quality (under 40, 40 to 79, 80 up; `spell_slots`). One-on-one, a full student forgets their weakest spell, and the podium names it first; a class passes full students over. The two left behind hold two. | `prep::slots`, `a_full_student_forgets_their_weakest_spell_alone_and_a_class_passes_them_over` | done |
| R10.2 | divergence | How much harder a class is (T52) was left to me. | The band already narrowed with the audience, by little at seven students (48 px to 34). | Each student after the first adds a stroke from `class_strokes`, two to a glyph, and takes `class_pct` 8 points off the effectiveness the drawing earns (two 92%, six 60%); `band_px` is now 48, 36, 16 px for 1, 2, 7. A class lesson never reaches 100%. | `each_student_in_a_class_adds_a_stroke_and_narrows_the_band_and_takes_effectiveness` | done |
| R10.3 | divergence | The builder golem keeps its place (T61). | It built for no mana; once a piece holds mana rather than spending it, a free builder would make holding meaningless, and a full-price one would have no point. | A golem-built piece holds `golem_hold_pct` 50% of its mana. | `a_golem_builds_after_its_delay_and_not_before` | done |
| R10.4 | divergence | Quiet Mind's discount applies to whatever you pay. | A held piece gives back what it holds when rubbed out; a discounted hold would hand back more than was paid. | Placing is not discounted; helpers, self spells and invasions still are. | `spells_on_yourself_speed_mana_hold_more_and_cost_less` | done |
| R10.5 | finding | A wall that could not rise kept nothing. | A rising piece the route check refused when it finished took its mana and gave none back. | It now gives back what it held. | `Prep::release` | done |
| R10.6 | divergence | A battle lasts until every enemy is gone (T54). | A rival's raider no one can reach would keep every battle running to the 10-minute limit. | Raiders still on the way keep a battle open; once the wave is gone, a raider still standing goes home. `battle_s` now spreads the waves' arrival; `battle_max_s` 600 is the limit. | `a_battle_lasts_until_its_enemies_are_gone_and_a_raider_alone_does_not_hold_it_open`, `a_battle_is_allowed_ten_minutes_whatever_the_waves_take` | done |
| R10.7 | divergence | A helper repeats with its fee (T55). | The recruitment elemental used to interview everyone at once, which T51 removes; the teaching elemental teaches the whole class. | A helper that finishes pays its mana and starts again; one that cannot pay waits; one with nothing left to do goes away; Send away stops any. The recruitment elemental hires one applicant a round; the teaching elemental teaches at the class's rate. | `a_helper_pays_again_each_time_it_finishes_waits_without_mana_and_can_be_sent_away` | done |
| R10.8 | divergence | The rehearsal (T48) shows how today's battle goes. | Rivals' raiders and their students' invasions are only known when the battle starts, and online the plans of other browsers arrive then too. | It plays your battle alone, from your plan, the day's enemies and the invasions already in your book, on the real battle's seed and field, in your browser only. With nothing changed it gives exactly what the day's battle takes. | `the_rehearsal_foretells_the_days_battle_and_says_how_a_change_moved_it`, `Match::rehearsal` | done |
| R10.9 | divergence | The room becomes two columns joined in the middle (T59). | The tall furniture (bookcases, jars) needs the full-height back half of the wall, so the columns run back to front: library, laboratory, map table on the right; office hours, podium, recruitment on the left; the book in the middle on the rug under the chandelier. | The library's cases moved to the right (298 to 392 thousandths), the jars to 235 where the east window was (removed), the posts mirrored. Flagstone paths join each column and cross through the book. The title circles the book. | `prep::STATIONS`, `room::PATHS` | done |
| R10.10 | finding | Stations about 5 m apart work. | My first layout put the library and laboratory 5.3 m apart: their 3 m reaches overlapped, and the close camera's look was pulled toward the neighbour. | Each column's stations are 6.9 m apart on a 10.2 m ring. A station you are close to now holds the camera's look: every other station's pull is scaled by how far you are from being at the near ones. | `walking_anywhere_in_the_room_never_turns_or_jumps_the_camera`, `room_camera` | done |
| R10.11 | finding | A wall keeps walkers on the field. | The stress probe's maze, put at row 24 on the smaller field, sat beside the entry: a walker pushed out of a wall cell on the edge left the grid. | `keep_out` clamps to the grid after pushing; the probe's maze rows scale with the field. | `a_body_pushed_out_of_a_wall_on_the_edge_stays_on_the_grid` | done |
| R10.12 | divergence | Fewer words (T49). | Ethan's T38 asked for each tile's sentence; it stays. | Instruction paragraphs moved behind a ? on their panel's title (`tuck_help`); five strings shortened (listed at the foot). Words on the first day's station panels, counted by `tests/words_probe.rs`: 745 before, 407 after. | `crates/core/tests/words_probe.rs` | done |
| R10.13 | finding | Every wild group has at least one enemy. | A heavy enemy's late group could round to none, and the book said "0 Hill Giant". | At least one. | `game.rs` wild count | done |
| R10.14 | divergence | A smaller field (T58). | | 26 × 26 cells (39 m), rocks in proportion (12), the map table's cells 22 px so its picture stays the same size, the battle views 50 m and 32 m from the field. | `arena::SIZE`, `the_starting_orbits_look_as_the_old_fixed_cameras_did_from_a_fifth_closer` | done |
| R10.15 | worklist | Round 10's numbers are mine. | | To judge by playing: `class_pct` 8 and the bands, the slot thresholds, 7 students, Lava Pool 10 a second for 20 mana, the golem's 50%, the 10-minute limit, the 26-cell field and the cameras, the stations' places; and the new words and What's new 1.8.0 (at the foot). | | person |
| R11.1 | divergence | How a spell reaches a rival's room (T62) was left to me. | Spells are chains made at the laboratory and cast by students in a battle; a rival's room is not in any battle. | Jinxes are a kind of piece of their own, cast by the archmage at the invasion book and never put in a spell (`lab.jinx_only`): Barrier, Bog and Black Ice, known from the first day. You pick one and a rival, click where it goes on their floor plan, then draw its glyph. Two of yours stand in each rival's tower at once (`jinx_each`); a jinx lasts one preparation at most. | `jinx.rs`, `PieceKind::Jinx`, `a_jinx_drawn_at_the_book_is_paid_for_and_sent_to_its_rival`, `two_of_your_jinxes_stand_in_a_rival_s_tower_at_once` | done |
| R11.2 | divergence | A computer rival can be jinxed and can jinx. | A computer rival does not walk: it spends its day from a budget of seconds at the start (`bot.rs`). | A jinx in its tower takes 25% of the jinx's seconds from its next day (`bot_jinx_pct`). From the second day the Attack Mage casts two jinxes and the Balanced archmage one, at the rival leading on score, before their lessons; they land 40% and 60% of the way through the preparation. So against the computer you are jinxed only while you lead. | `jinx::bot_loss_s`, `a_rival_whose_day_jinxes_took_does_nothing_with_it`, `a_computer_rival_s_jinxes_land_partway_through_the_preparation` | done |
| R11.3 | divergence | All mages run faster on the paths (T63). | Only the archmage walks in the room; the students sit. And *path* already means the enemies' way across the field. | The worn ways are walkways (a new glossary entry); walking on one is 150% (`walkway_pct`). In a bog the bog wins. | `jinx::pace_pct`, `walking_on_a_walkway_is_half_again_as_fast` | done |
| R11.4 | divergence | A jinx's strength (T64). | Nothing in a jinx had a strength; a lesson's strength is its drawing. | Strength is the drawing, at the invasion book with one student's band: 0% lasts 30 s, 100% 60 s, and a bar under the board says how long. The sizes are fixed: a barrier 4 m by 0.8 m, turned in steps of 45° with R; a bog 2.2 m across from its centre at a third of the pace; black ice the whole walkway, 3% grip. | `jinx::seconds`, `a_jinx_lasts_thirty_to_sixty_seconds_by_the_drawing` | done |
| R11.5 | divergence | A barrier must not shut anyone out of a station. | Barriers from three rivals could close a station's ways in. | A barrier stays 2.1 m from every station's centre (`STATION_CLEAR`) and inside the room, so one alone never covers a station, and one that lands on the archmage pushes them out the short way. Several together can still close a station for under a minute; I left that, since it is the point. | `jinx::fits`, `a_barrier_cannot_cover_a_station_or_leave_the_room` | done |
| R11.6 | divergence | Jinxes online. | Each browser prepares alone and the battles are checked by checksum (`online.rs`). | A jinx is a message (protocol 2, so a 1.8 browser and a 1.9 browser do not meet). The target's browser walks with it; every browser applies one aimed at a computer rival, so that rival's next day is the same everywhere. A jinx is checked on arrival (`sane_jinx`). The online check now has one browser jinx the other. | `Msg::Jinx`, `a_jinx_from_another_browser_must_be_one_that_could_be_cast`, `tests/browser/online.py` | done |
| R11.7 | finding | The drawing's words were readable. | On 1.8.0 the panel with a drawing's instructions and Cancel sat at the top of the screen, under the HUD (found taking the mockups' backgrounds). | It sits under the board now, clear of the corner's buttons. | `web/style.css` | done |
| R11.8 | divergence | The build matches its mockups. | | Differences on purpose: the plan draws each walkway as one band and the rug as a disc; the barrier is a wall of light rather than hatched glass; a bog is an eight-sided pool; the mockup's light running along the walkways is left out, since the quicker walking shows by itself. The book's road is lower, so the jinxes fit under it. | journal entry 11, part 3 | done |
| R11.9 | worklist | Round 11's numbers are mine. | | To judge by playing: walkways 150%, a bog 33%, ice 3% grip, the barrier's size, 30 to 60 s, two in a tower, 25, 20 and 20 mana, a computer rival's 25% and the Attack Mage's two a day; and the new words and What's new 1.9.0 (at the foot). | | person |

## Strings written by the agent

Each reader-visible string the agent wrote, with its key, for the person to read.

Every string in `data/strings.json` (263). The names in `data/content.json` are row R3.10's.

| Key | String |
| --- | --- |
| `game.name` | Mage Tower Battle |
| `title.blurb` | The last archmage was fired and wrecked the library on his way out. Read what he left on the floor, make spells, recruit students and teach them, then watch your tower hold against everything the day sends. |
| `title.versus` | Versus the computer |
| `title.practice` | Practice: one short day |
| `title.trials` | Trials |
| `title.achievements` | Achievements |
| `title.help` | How to play |
| `title.online` | Online, up to four |
| `back` | Back |
| `leave` | Leave the match |
| `setup.heading` | A match against three rivals |
| `setup.text` | Each day is a preparation and then a battle at every tower at once. Damage to your tower lowers your score; the highest score after the last day wins. |
| `setup.days` | Days |
| `setup.prep` | Preparation |
| `setup.battle` | Battle, at most |
| `setup.speed` | Battle speed |
| `setup.cap` | Ragdolls per field |
| `setup.skill` | Rivals' skill |
| `setup.less` | Less |
| `setup.more` | More |
| `setup.rivals` | Rivals |
| `setup.rival` | Rival {n} |
| `setup.change` | Change |
| `setup.start` | Start the match |
| `bot.spellcrafter` | The Spellcrafter |
| `bot.mass_teacher` | The Mass Teacher |
| `bot.attack_mage` | The Attack Mage |
| `bot.balanced` | The Balanced Archmage |
| `seat.you` | You |
| `hud.day` | Day {day} of {days} |
| `hud.battle` | Day {day} battle |
| `hud.score` | Score |
| `hud.students` | Students |
| `hud.pages` | Pieces known |
| `hud.mana` | Your mana |
| `station.library` | Library |
| `station.lab` | Spell laboratory |
| `station.book` | Invasion book |
| `station.map` | Map table |
| `station.podium` | Podium |
| `station.recruit` | Recruitment table |
| `prep.walk_hint` | Walk with WASD or the arrow keys. Stand at a station to use it. E does a station's main thing; Esc cancels. |
| `prep.ready` | Start the battle now |
| `prep.over` | The preparation is over. |
| `prep.not_here` | Walk to that station first. |
| `prep.busy` | Finish or cancel what you are doing first. |
| `prep.no_mana` | Not enough mana yet. It comes back over the day. |
| `prep.unknown` | That cannot be done here. |
| `prep.cancelled` | Cancelled. |
| `busy.reading` | Reading. |
| `busy.interview` | Interview: press J, K or L as each note reaches the line, and Space for the closing spell. |
| `busy.drawing` | Draw the glyphs in one stroke with the mouse: press, trace the line, let go. |
| `busy.cancel` | Cancel |
| `library.read` | Read a book |
| `library.left` | {n} books still teach something new. |
| `library.none_yet` | Pick up a book to learn what is in it. |
| `library.last` | Last read: {name}. {text} |
| `library.last_tome` | Last read: {name}, a whole spell. |
| `library.learned` | Learned {value}. |
| `library.tome` | Learned a whole spell: {value}. |
| `library.empty` | Every book here is read. |
| `lab.add` | Add |
| `lab.save` | Save the spell |
| `lab.clear` | Clear |
| `lab.choose` | Choose |
| `lab.chosen` | chosen |
| `lab.draft_empty` | Add pieces in order: a payload first, modifiers before what they change, a trigger before what it starts. |
| `lab.stats` | Power {power}, cooldown {cooldown} s, mana {mana}. |
| `lab.spell_mana` | {mana} mana |
| `lab.saved` | Saved {value}. |
| `lab.empty` | Add at least one piece. |
| `lab.too_long` | A spell holds six pieces at most. |
| `lab.no_payload_first` | Start with a payload: a trigger needs something to start from. |
| `lab.trigger_at_end` | A trigger needs a payload after it. |
| `lab.modifier_at_end` | A modifier needs a payload after it. |
| `lab.invade_without_summon` | Invade needs a summon after it. |
| `lab.too_deep` | Three triggers is the most a spell can hold. |
| `lab.helper` | A helper is summoned on its own, at the podium or the recruitment table. |
| `lab.place_only` | Raise Sub-tower is placed from the map table. |
| `lab.unknown` | You have not learned that piece yet. |
| `lab.bad` | That piece is not in the spell. |
| `lab.already` | You already have that spell. |
| `lab.full` | Your spellbook is full. |
| `kind.payload` | payload |
| `kind.modifier` | modifier |
| `kind.trigger` | trigger |
| `kind.helper` | helper |
| `recruit.text` | {n} applicants today. |
| `recruit.leans` | Good at {good}, poor at {bad}. |
| `recruit.interview` | Interview |
| `recruit.group` | Interview everyone at once |
| `recruit.summon` | Summon a recruitment elemental |
| `recruit.group_heading` | everyone waiting |
| `recruit.interviewing` | Interviewing {who} |
| `recruit.joined` | {value} joined the college. |
| `recruit.none` | Nobody is waiting today. |
| `podium.text` | Chosen spell: {spell}. Choose another below. |
| `podium.teach_all` | Teach the class ({n}) |
| `podium.teach_one` | Teach alone |
| `podium.not_taught` | not taught |
| `podium.summon` | Summon {name} |
| `podium.drawing` | Teach {spell} to {who} |
| `podium.to_class` | the class of {n} |
| `podium.summoning` | Summon {name} |
| `podium.taught` | Taught {value} students. |
| `podium.no_students` | There are no students to teach. |
| `podium.no_spell` | Save a spell at the laboratory first. |
| `helper.summoned` | Summoned at {value}%. |
| `helper.trained` | {value} has more mana now. |
| `helper.built` | The golem finished its work. |
| `helper.unknown` | You have not learned that helper. |
| `jobs.heading` | Helpers at work |
| `draw.bad` | That drawing could not be read. |
| `draw.none` | Nothing is being drawn. |
| `draw.score` | Drawn at {value}%. |
| `book.text` | What the next battle brings to your tower, and invasions you can send. |
| `book.wild` | Wild |
| `book.group` | {n} {name} ({type}) |
| `book.invasion` | {spell}, cast by their archmage |
| `book.students_heading` | Students' invasions |
| `book.students_hidden` | show here when the battle starts |
| `book.cast` | Send {spell} at {who} |
| `book.bad` | Choose a rival and an invasion. |
| `book.not_invasion` | Only a spell that starts with Invade can be sent. |
| `map.text` | {n} things placed. Open the map to change the layout. |
| `map.open` | Open the map |
| `map.how` | Pick something, then click a cell. R or Turn changes its direction. A wall may bend the route but never close it. |
| `map.none_known` | You have not learned anything you can place. |
| `map.pick` | Pick |
| `map.selected` | picked |
| `map.mana_cost` | {mana} mana, at once |
| `map.golem_cost` | no mana, {s} s for a golem |
| `map.turn` | Turn |
| `map.use_golem` | Use a golem instead |
| `map.use_mana` | Use mana instead |
| `map.close` | Close the map |
| `map.remove` | Remove a {name} |
| `layout.off_grid` | That runs off the field. |
| `layout.entry` | Keep the entry clear. |
| `layout.taken` | Something is already there. |
| `layout.closes_path` | That would close the route to your tower. |
| `layout.bad` | That cannot be placed. |
| `layout.unknown` | You have not learned how to raise that. |
| `layout.no_golem` | A golem cannot build that. |
| `layout.golem_busy` | Your golem is still building. |
| `view.all` | All towers |
| `view.near` | Closer |
| `view.far` | Further |
| `summary.heading` | Day {day}: how every tower held |
| `summary.text` | Damage taken comes off each score. |
| `summary.row` | took {taken}, killed {kills}, froze {frozen}, charmed {charmed}, ward took {warded}, sent {dealt} |
| `summary.next` | Next day |
| `summary.results` | See the results |
| `results.won` | Your tower stands highest. |
| `results.lost` | Another tower stands higher. |
| `results.text` | Final scores, highest first. |
| `results.place` | {n}. {name} |
| `results.taken` | took {taken} in all |
| `results.again` | Another match |
| `results.title` | Back to the title |
| `results.earned` | Earned this match |
| `waiting.heading` | Waiting for the other towers |
| `waiting.text` | The battle starts when every plan has arrived. |
| `trial.start` | Start |
| `trial.again` | Try again |
| `trial.best` | Your best: {value}. |
| `trial.no_best` | Not tried yet. |
| `trial.hidden` | ??? |
| `trial.got` | earned |
| `trial.seconds` | {n} s |
| `trial.result` | {value}: {medal}. |
| `trial.new_best` | A new personal best. |
| `trial.no_recruiting` | No recruiting in this trial. |
| `medal.bronze` | Bronze |
| `medal.silver` | Silver |
| `medal.gold` | Gold |
| `medal.archmage` | Archmage |
| `medal.none` | no medal |
| `achievement.earned` | Achievement: {name} |
| `achievement.got` | earned |
| `achievement.not_yet` | not yet |
| `achievement.count` | {n} of {of} earned. |
| `sound.mute` | Sound off |
| `sound.unmute` | Sound on |
| `save.broken` | Your saved progress could not be read, so it starts again. |
| `key.note0` | J |
| `key.note1` | K |
| `key.note2` | L |
| `key.note3` | ✦ |
| `error.bad` | That cannot be done. |
| `error.no_mode` | There is no mode "{name}". |
| `error.no_screen` | There is no screen "{name}". |
| `error.unknown` | That cannot be done ({key}). |
| `help.1.head` | The day |
| `help.1.text` | Each day you have a few minutes to prepare, then every tower is attacked at once and you watch. Time is what you spend: walking, reading, interviewing and teaching all take it. |
| `help.2.head` | Library |
| `help.2.text` | Read the books on the floor. Each teaches a spell piece, or a whole spell. |
| `help.3.head` | Spell laboratory |
| `help.3.text` | Join pieces into a spell, left to right: modifiers change the next payload, a trigger starts the rest when the payload hits, dies or ends. Strong spells with short cooldowns cost much more mana. |
| `help.4.head` | Recruitment table |
| `help.4.text` | Interview applicants with a short rhythm game. A better interview brings a better student. |
| `help.5.head` | Podium |
| `help.5.text` | Teach a spell by drawing its glyphs in one stroke. Teaching one student forgives a shaky line; a whole class needs a careful one. How well you draw is how well they learn. |
| `help.6.head` | Map table |
| `help.6.text` | Place walls, lava, bait and sub-towers from spells you know, with mana at once or a slow golem for free. A wall may bend the route but never close it. |
| `help.7.head` | Invasion book |
| `help.7.text` | See what the next battle brings. Spells that start with Invade send summons at a rival: yours show in their book at once, your students' only when the battle starts. |
| `help.8.head` | The battle |
| `help.8.text` | Students cast from the tower and sub-towers. Some types hurt others double or half; none is immune. An enemy that reaches your tower strikes it once and is gone. |
| `help.9.head` | Winning |
| `help.9.text` | Your score starts at {score} and only falls. The highest score after the last day wins. |
| `page.no_webgl` | This game needs WebGL 2, which this browser does not offer. |
| `online.heading` | Online play |
| `online.text` | Up to four archmages, each in their own browser. One hosts and shares a room code; the others type it in and join. Empty seats go to computer rivals. If a room code will not connect, try pasted codes: the host sends an invitation by any message app, and the joiner sends a reply back. |
| `online.name` | Your name |
| `online.code` | Room code |
| `online.default_name` | Archmage |
| `online.host` | Host a room |
| `online.join` | Join by room code |
| `online.invite` | Host by pasted codes |
| `online.reply` | Join by pasted codes |
| `online.lobby` | The room |
| `online.code_is` | Room code: {code}. Send it to the people you are playing with. |
| `online.by_paste` | Connected by pasted codes. |
| `online.copy_code` | Copy the room code |
| `online.settings` | The match: {days} days, {prep} of preparation and {battle} of battle each day, from the host's Versus settings. |
| `online.start` | Start the match |
| `online.ready` | Ready |
| `online.unready` | Not ready yet |
| `online.leave` | Leave the room |
| `online.you` |  (you) |
| `online.seat_host` | Host |
| `online.seat_person` | Player |
| `online.seat_open` | Computer rival |
| `online.ready_chip` | Ready |
| `online.not_ready_chip` | Not ready |
| `online.no_room` | There is no room open. |
| `online.not_everyone_ready` | Everyone in the room has to be ready first. |
| `online.waiting` | Waiting for players to join. |
| `online.joining` | Looking for the room. |
| `online.in_lobby` | In the room. Press Ready when you are. |
| `online.playing` | The match is on. |
| `online.garbled` | A message from another browser made no sense and was ignored. |
| `online.reload` | The host is running a different version. Reload the page, and ask them to reload too. |
| `online.full` | That room is full, or its match has started. |
| `online.left` | A player left. Their tower stands with what they had sent. |
| `online.host_left` | The host left. The match goes on here without the other players. |
| `online.desync` | This browser's scores no longer match another player's. The match goes on, but the results may differ between you. |
| `online.no_relay` | No relay answered. Check the connection, or try pasted codes. |
| `online.no_path` | The two browsers could not reach each other. Try another network, or pasted codes. |
| `page.invite_prompt` | Send this invitation to the player joining, then paste their reply below it and press Connect. |
| `page.reply_prompt` | Paste the host's invitation here. |
| `page.reply_ready` | Send this reply back to the host. |
| `page.copy` | Copy |
| `page.connect` | Connect |
| `page.make_reply` | Make a reply |
| `page.close` | Close |
| `page.copied` | Copied. |
| `page.copy_failed` | This browser would not copy. Select the text and copy it by hand. |
| `page.signal_failed` | That code did not work. Check it was pasted whole. |
| `page.bad_code` | A room code is four to eight letters and digits. |
| `page.loading` | Getting the connection ready. |
| `online.invite_more` | Invite another player |
| `office.lead` | A desk in the tower room. See each student, choose where they stand in the battle, and what they cast. |
| `office.class` | Your class · {n} students |
| `office.stats` | Stats |
| `office.range` | Range |
| `office.per_s_short` | {n}/s |
| `office.proficiency` | Proficiency by school |
| `office.spells_heading` | Spells and casting effectiveness |
| `office.col_spell` | Spell |
| `office.col_eff` | Effectiveness |
| `office.col_cast` | Cast |
| `office.no_spells` | Nothing taught yet. Teach a spell at the podium. |
| `office.where_short` | Where {name} stands |
| `office.board_text` | Click a seat or drag a student onto one: the ring is the tower's {n} seats, and a sub-tower has its own. Dark: chosen here. Pale: given the first free seat. The ground has less range. |
| `office.casts` | What {name} casts |
| `office.step_hint` | Click a step to take it out. |
| `office.add` | + {spell} |
| `map.tab_build` | Build |
| `map.tab_seats` | Seat students |
| `map.seats_how` | Pick a student, then click a seat on the map, or drag a student's disc from one seat to another. Two students trade seats. |
| `map.golem_chip` | golem, {s} s |
| `map.rise_chip` | {s} s a cell |
| `map.route_heading` | The route |
| `map.route_cells` | {n} cells |
| `map.route_was` | was {n} |
| `map.route_far` | Enemies walk {x} times as far as on the bare field. |
| `map.route_placed` | This field: {n} cells placed. |
| `map.route_note` | A wall that would close the route is refused, cell by cell, as you drag. |
| `map.seats_note` | Students stand where the map shows when the battle starts. Anyone you have not seated takes the first free seat. |
| `map.pick_student` | Pick a student first. |
| `seal.title` | Seal the spell |
| `seal.text` | Turn each ring until its piece's glyph sits under the gold mark. One ring for each piece, so a longer spell takes longer to seal. |
| `seal.ring` | Ring {n} · {piece} |
| `seal.sealed` | sealed |
| `seal.turning` | turning |
| `seal.next` | next |
| `key.left` | A |
| `key.right` | D |
| `key.space` | Space |
| `key.esc` | Esc |
| `seal.keys_turn` | turn the ring · |
| `seal.keys_lock` | lock it · |
| `seal.keys_stop` | stop |
| `seal.explain` | Each ring starts at a random turn and turns a step further if you lock it off the mark. |
| `seal.never_lead` | Never changes the spell. |
| `seal.never` | How well you seal only changes how long saving takes. Effectiveness comes from teaching at the podium. |
| `sort.title` | Sort the pile |
| `sort.text` | Shelve each book by its school before it lands: press its number or click its shelf. Somewhere in the pile is a book you have not read. |
| `sort.floor` | On the floor: {n} |
| `sort.searching` | Searching |
| `sort.count` | {n} of {of} shelved |
| `sort.explain` | {of} shelved finds the book. A wrong shelf drops the book back on the pile, and the next one falls faster. |
| `sort.streak` | Streak {n} |
| `sort.golem_heading` | Or send the golem |
| `sort.golem_text` | A librarian golem searches the pile alone: one unread book every {s} s, for this much mana. Slower than you, but you can teach meanwhile. |
| `sort.golem_wake` | Wake the golem |
| `builder.lead` | Each set reads left to right: optional modifiers, a payload, then a trigger. A trigger opens the next set; a branch starts a second row from the first trigger. |
| `builder.drag` | Drag a tile onto a slot, or click a tile to add it. A slot only takes its own kind: blue modifiers, red payloads, yellow triggers. |
| `builder.this_spell` | This spell |
| `builder.seal_note` | Sealing is a short puzzle. It never changes how strong the spell is. |
| `builder.your_spells` | Your spells |
| `builder.empty` | Start with a payload from the tray below. |
| `builder.set` | Set {n} |
| `builder.set_after` | Set {n}: {trigger} |
| `builder.unread` | {n} not read yet |
| `builder.wrong_slot` | That slot takes another kind of piece. |
| `lab.stats` | Power {power} · cooldown {cooldown} s |
| `lab.save` | Seal the spell |
| `news 1.6.0` | The tower is drawn like Ape Race's maps: faceted stone and timber, windows that let dusk light across the floor, full bookshelves, and each station dressed for what you do there. |
| `news 1.6.0` | Mages have arms and feet. They swing their arms as they walk, lift the staff to cast, hold a book up to read, and sit on the benches to listen. |
| `news 1.6.0` | Stand at a station and the camera comes down behind you, the way Ape Race follows its ape. Walk away and it rises again. |
| `news 1.6.0` | The battlefield has the same light: faceted grass, flowers and stones, and a haze far off. |
| `news 1.6.0` | The black outlines are gone, and the HUD is dark and see-through, so the scene shows behind it. |
| `sort.text` | Drag each book onto its school's shelf, or press its shelf's number. Take your time: nothing falls. Shelve the pile and you learn a book; shelve it with no mistake and you learn two. |
| `sort.explain` | A book on a wrong shelf goes back to the floor and costs the second book. Each spine shows its school's name. |
| `sort.shelving` | Shelving |
| `sort.perfect_yet` | No mistakes yet: two books |
| `sort.missed` | {n} on a wrong shelf: one book |
| `library.perfect` | A perfect sort: a second book. |
| `journey.title` | {name} sets out for {rival} |
| `journey.text` | Jump the gaps to reach their towers. The run moves by itself; a fall puts them back on the last ledge, and they arrive later. |
| `journey.jumps` | Gap {n} of {of} |
| `journey.falls` | Falls {n} · {s} s late |
| `journey.keys` |  jump |
| `raid.lead` | Send students to raid a rival. They walk in with that rival's enemies, cast their own spells, and attack the tower and its students. A student you send is not on your towers that battle. |
| `raid.students` | Your students |
| `raid.away` | Raiding {rival} |
| `raid.stats` | Health {hp} · {speed} m/s · spells known: {n} |
| `raid.recall` | Call back |
| `raid.how` | Drag a student onto a rival, or click a student and then a rival. Each journey is a short run of jumps. |
| `raid.you` | You |
| `raid.rival_none` | {rival} · {score}. No raiders of yours yet. |
| `raid.rival` | {rival} · {score}. Your raiders: {names}. |
| `raid.home_heading` | On your towers this battle |
| `raid.home` | {n} of {of} students. |
| `raid.spells_heading` | Invasion spells |
| `raid.label` | Raider · {name}, {rival}'s |
| `raid.fell` | Fell |
| `raid.coming` | On the way |
| `raid.raider_row` | {name}, {rival}'s |
| `raid.away_row` | {name}, at {rival} |
| `raid.on_field` | Raiders on your field |
| `raid.none_here` | None yet. |
| `raid.yours_away` | Yours away |
| `raid.arrived` | {value} reached the rival's field and will raid in the next battle. |
| `raid.pick_first` | Pick up a student first, then drop them on a rival. |
| `raid.bad` | That student is not here to send. |
| `raid.bad_rival` | Pick a rival's tower, not your own. |
| `raid.already` | That student is already raiding. |
| `bt.first` | First that works |
| `bt.seq` | In order |
| `bt.if.hurt` | Hurt? |
| `bt.if.foe_mage` | An enemy mage on this field? |
| `bt.if.foe_in_range` | An enemy in reach? |
| `bt.if.raiding` | Raiding? |
| `bt.if.foe_near_tower` | An enemy near the tower? |
| `bt.if.on_tower` | On a seat? |
| `bt.do.fall_back` | Fall back to the tower |
| `bt.do.leave_tower` | Leave the tower |
| `bt.do.approach_mage` | Close to casting reach |
| `bt.do.cast_mage` | Cast at that mage |
| `bt.do.cast_nearest` | Cast at the nearest |
| `bt.do.cast_near_tower` | Cast at the one nearest the tower |
| `bt.do.cast_tower` | Cast at the tower |
| `bt.do.advance` | Walk on toward the tower |
| `bt.do.hold` | Hold the seat |
| `bt.legend_ran` | ran last |
| `bt.legend_running` | still running |
| `bt.legend_failed` | checked, not true |
| `bt.read` | Read it from the top: a choice tries its options from left to right and stops at the first that works; a sequence does its steps in turn and stops at the first that does not. |
| `bt.mark_ran` | ✓ |
| `bt.mark_running` | ▶ |
| `bt.mark_failed` | ✗ |
| `bt.watching` | Watching {name} · {temper} |
| `bt.prev` | ‹ |
| `bt.next` | › |
| `bt.watch_stats` | Health {hp} of {of} |
| `office.summed` | Summed effectiveness {n}% |
| `office.health` | Health |
| `office.speed` | Speed |
| `office.speed_value` | {v} m/s |
| `office.summed_explain` | Each spell they learn adds its effectiveness to the sum. Health is {hp} plus {per} for every 100%; speed is {speed} m/s plus {per_speed} for every 100%. |
| `office.tab_seats` | Seats and casting |
| `office.tab_mind` | How they decide |
| `builder.in_order` | What it does, in order |
| `raid.no_rivals` | No rivals in practice: raids are for a match against others. |
| `temperament steadfast` | Steadfast: Stays on the seat and casts at whatever comes in reach. |
| `temperament hunter` | Hunter: Leaves the tower to go after enemy mages, and falls back when hurt. |
| `temperament guardian` | Guardian: Casts first at whatever is nearest the tower. |
| `temperament reckless` | Reckless: On a raid, makes straight for the tower and casts at it. |
| `temperament cautious` | Cautious: Casts at what comes in reach, and falls back when hurt. |
| `news 1.7.2` | The title screen is inside the tower: the camera turns slowly past every station while an archmage walks the room and the class waits on the benches. |
| `news 1.7.1` | Walking up to a station is easier to steer: the camera no longer turns to face it, so the keys always point the same way on the screen, and it comes down gently instead of dropping. |
| `news 1.7.0` | Every tile in the laboratory says what its piece does, with its numbers, and the spell's box reads the whole spell out in order. |
| `news 1.7.0` | Finding a book is a pile on the floor: drag each to its school's shelf, with no timer. Sort the pile with no mistake and you learn two books. |
| `news 1.7.0` | Send students to raid a rival from the invasion book. After a short run of jumps they join that rival's next battle, cast their own spells and attack the tower and its students. They are off your own towers while they are away. |
| `news 1.7.0` | Each student has a temperament, such as Hunter or Steadfast. Office hours draws how they decide as a tree, and in a battle you can watch it light up. |
| `news 1.7.0` | A student's health and speed grow with the spells they know: add up their effectiveness over every spell. |
| `setup.battle` | Enemies arrive over |
| `recruit.text` | {n} applicants today. Your college has {have} of {max} students. |
| `recruit.leans` | {good}, not {bad} |
| `podium.teach_all` | Teach the class ({n}), {pct}% each |
| `map.golem_cost` | half the mana held, {s} s for a golem |
| `help.4.text` | Interview applicants one at a time with a short rhythm game. A better interview brings a better student, who holds more spells. The college holds {students_max} students. |
| `help.5.text` | Teach a spell by drawing its glyphs in one stroke. One student alone gets the drawing's whole effectiveness; each student more in a class adds a stroke, narrows the line and gives everyone less. A student who is full forgets their weakest spell to learn a new one alone, and a class passes them over. |
| `help.6.text` | You start knowing Glass Wall and Raise Sub-tower; books teach more. Pick one at the map table and drag across cells. Each cell holds its mana out of your maximum for as long as it stands, and rises up to {raise_max} s later while you do other things; rub it out to get the mana back. A slow golem builds for half the mana instead. A wall may bend the route but never close it. |
| `help.8.text` | Students cast from the tower and sub-towers. Some types hurt others double or half; none is immune. An enemy that reaches your tower strikes it once and is gone. The battle lasts until every enemy has fallen or reached the tower, ten minutes at most. |
| `gloss.payload` | Power {power} · cooldown {cooldown} s |
| `office.about` | Quality {q} · holds {slots} spells |
| `builder.tray_mod` | Modifiers |
| `raid.rival_none` | {rival} · {score} |
| `hud.mana_held` | {n} · {held} held |
| `hud.held_hint` | Walls, pools and sub-towers hold their mana for as long as they stand. The striped end of the bar is what they hold; rub one out at the map table to get its mana back. |
| `recruit.full_text` | Your college is full: {n} students is as many as it holds. |
| `recruit.full` | Your college is full. |
| `podium.row` | {eff} · {n} of {slots} spells |
| `podium.forgets` | forgets {spell} |
| `podium.class_full` | Nobody in the class has room for this spell. |
| `podium.bad` | That student is not in the class. |
| `lesson.heading` | {spell}: drawn at {score}%, {pct}% of it to each |
| `lesson.up` | {from}% → {to}% |
| `lesson.same` | still {to}% |
| `lesson.new` | new, {to}% |
| `jobs.waiting` | waiting for {mana} mana |
| `jobs.again` | {t}, then {mana} mana to go again |
| `jobs.dismiss` | Send away |
| `helper.no_work` | {value} has nothing left to do and went away. |
| `helper.dismissed` | {value} went away. |
| `helper.bad` | That is not a helper at work. |
| `rehearsal.label` | If the battle began now |
| `rehearsal.loss` | −{n} |
| `rehearsal.first` | playing it… |
| `rehearsal.updating` | playing it again… |
| `rehearsal.better` | {n} better |
| `rehearsal.worse` | {n} worse |
| `rehearsal.same` | no change |
| `rehearsal.hint` | Today's battle played against your plan as it stands: your students, their spells and seats, what is on the field, the day's enemies and the invasions already in your book. Rivals' raiders and what their students send are not known until the battle. |
| `kind.jinx` | Jinx |
| `gloss.jinx` | {mana} mana; it lasts {lo} to {hi} s, by your drawing. |
| `lab.jinx_only` | A jinx is cast at a rival's tower from the invasion book, not put in a spell. |
| `jinx.bad` | That jinx cannot be cast there. |
| `jinx.unknown` | You do not know that jinx yet. |
| `jinx.full` | That tower already holds as many of your jinxes as it can. Wait for one to run out. |
| `jinx.outside` | A barrier has to stand inside the room. |
| `jinx.on_station` | A barrier cannot cover a station. |
| `jinx.no_walkway` | Black Ice goes on a walkway: click nearer one. |
| `jinx.cast` | {value} is on its way. |
| `jinx.landed` | {who} cast {value} in your tower. |
| `jinx.drawing` | {rival}: {name} |
| `jinx.lasts` | It lasts {s} s |
| `jinx.lasts_lo` | {s} s at 0% |
| `jinx.lasts_hi` | {s} s at 100% |
| `jinx.tag` | {name} {t} |
| `hud.jinxed` | Jinxes on your tower |
| `jinx.heading` | Jinxes |
| `jinx.how` | Pick a jinx, then a rival. Click where it goes on their floor plan, then draw its glyph: the better the drawing, the longer it lasts. |
| `jinx.no_rivals` | No rivals in practice: jinxes are for a match against others. |
| `jinx.cast_at` | Cast {name} at |
| `jinx.card` | {text} · your jinxes: {list} |
| `jinx.range` | {lo} to {hi} s |
| `jinx.short.barrier` | A wall of force no one can walk through. |
| `jinx.short.bog` | A patch of mud that slows walking. |
| `jinx.short.black_ice` | A walkway turns to ice, and walking on it slides. |
| `jinx.aim_title` | Jinx a tower |
| `jinx.aim_help` | Click where it goes on the plan, then draw its glyph. A barrier cannot cover a station. |
| `jinx.their_tower` | {rival}'s tower |
| `jinx.key_click` | Click |
| `jinx.key_click_does` |  where it goes   |
| `jinx.key_turn_does` |  turn it   |
| `jinx.key_esc_does` |  put it away |
| `jinx.close` | Close the plan |
| `jinx.their_heading` | Their tower |
| `jinx.none_here` | None of your jinxes stand here. You can have {each} at a time in each tower. |
| `jinx.here` | Your jinxes here: {list}. {n} of {each}. |
| `jinx.legend_walkway` | Walkway: quicker walking |
| `jinx.legend_barrier` | Barrier: no one walks through |
| `jinx.legend_bog` | Bog: slow walking |
| `jinx.legend_ice` | Black Ice: walking slides |
| `news 1.8.0` | While you prepare, the HUD shows what today's battle would take off your score if it began now, and how your last change moved it. |
| `news 1.8.0` | Teaching one student alone is far better than teaching a class: each student more adds a stroke, narrows the line and gives everyone less. A student holds two to four spells, and a new one replaces their weakest. |
| `news 1.8.0` | Interviews are one applicant at a time, and the college holds seven students. |
| `news 1.8.0` | Walls, pools and sub-towers hold their mana for as long as they stand, so building more leaves less for everything else. Lava Pool burns far less and holds far more. |
| `news 1.8.0` | Helpers take their mana again each time they finish, and go on until you send them away. |
| `news 1.8.0` | A battle lasts until every enemy has fallen or reached the tower, ten minutes at most. |
| `news 1.8.0` | The tower is laid out in two columns: the students' stations on the left, yours on the right, the invasion book between them, with paths worn across the floor. The field is smaller and the battle cameras closer. |
| `news 1.8.0` | Fewer words on each panel: how to use it is behind the ? by its title. A book you drag follows the pointer, and clicking an empty slot in the laboratory opens its tab. |
| `news 1.9.0` | Jinxes: at the invasion book, cast a barrier, a bog or black ice into a rival's tower. Click where it goes on their floor plan, then draw it: the better the drawing, the longer it lasts, from 30 seconds to a minute. |
| `news 1.9.0` | Walking on the walkways between stations is half again as fast. A bog slows you to a third, and on black ice you slide. |
| `news 1.9.0` | The Attack Mage and the Balanced archmage jinx whoever leads, and a jinx in a computer rival's tower costs it time the next day. |
| `news 1.9.0` | A drawing's words sit under the board now, clear of the HUD. |

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
| R4.11 | finding | The live browser check failed once after this round's push, waiting for two saved spells, and passed when run again. | The check clicked Add twice in a row; a panel redrawn between the clicks can take the second click. | The check now waits for each piece to show in the draft before the next click. | `tests/browser/interactions.py` | done |
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

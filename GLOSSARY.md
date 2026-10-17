# Glossary

The source of truth for what each word in this tool means. Strings, tests, the plan and the README use a term in the sense given here. The page shows a definition when a reader hovers the term.

Write each entry as a `### Term` heading, then one or two plain sentences, then an optional `Not:` line listing words that must not be used for it. A number a rule decides is a `{placeholder}` the core fills, so the glossary cannot disagree with the rules. Keep entries in alphabetical order. `crates/core/tests/copy.rs` checks the format, the placeholders and the `Not:` words; `TONE.md` governs the prose.

### Archmage

You: the new head of the mage college, hired after the last one was fired. You walk the tower, read books, make spells and teach students, and you do not fight in the battle.

Not: dean, headmaster, wizard

### Battle

The part of each day when every tower is attacked at once and the players watch. The enemies arrive over the first {battle.length_s} seconds, and the battle lasts until every one has fallen or reached the tower, or {battle.max_min} minutes at most.

Not: round, wave

### Book

One of the books the last archmage pulled off the library shelves. Reading it teaches you a spell piece or a whole spell.

Not: tome, scroll

### Branch

A piece that starts a spell's second row. When the trigger before it fires, the sets after the branch start too, beside the first row's.

### Builder golem

A stone helper that builds a sub-tower or a wall for you without a spell. It is slow, and what it builds holds only {golem.hold_pct}% of its mana.

Not: worker, construct

### College

The mage college you now run: its tower, its library and its students. A kind of magic is a school, not a college.

### Cooldown

The time a student waits between two casts of the same spell.

Not: recharge, delay

### Day

One preparation followed by one battle. A match has {match.days} days.

Not: round

### Difficulty

How strong the day's wild enemies are, chosen in setup for a match and on each trial's card: Easy, Normal, Hard or Brutal. A harder difficulty sends more enemies with more health.

### Effectiveness

How well a student learned a spell, from {effectiveness.min} to {effectiveness.max} percent. A spell cast at half effectiveness does half of what the spell can do.

Not: mastery

### Enemy

A creature that walks the path to a tower and attacks it. It comes from the day's draw or from a rival's invasion.

Not: monster

### Health

How much a mage on the field can take before they fall for the rest of the battle. The more effectiveness summed over the spells they know, the more health they have and the faster they move.

### Invasion

Enemies you add to a rival's battle by casting an invasion spell. Yours appear in their invasion book at once; your students' appear only when the battle starts.

Not: raid

### Invasion book

The book on its lectern that lists what the next battle will bring to your tower.

### Journey

The short run a raider makes to reach a rival's towers: a few jumps over gaps. A fall makes them arrive later.

### Librarian golem

A stone helper woken at the library for mana. It searches the pile on its own and brings you an unread book every so often, slower than sorting it yourself. It takes its mana again for each book, until you send it away.

### Library

The corner of the tower where the books lie scattered on the floor.

### Mana

What a spell costs to cast. Each student has a mana pool that refills at their mana regen. Your own mana also pays for helpers, and what stands on the field holds part of it for as long as it stands.

Not: energy

### Modifier

A spell piece that changes the next piece in the spell, such as turning one shot into a ring of them.

### Office hours

The desk in the tower room where you see each student's numbers and choose where they stand in the battle and what they cast.

### Payload

A spell piece that does something where it lands: an acid glob, a fire bolt, a lava pool, a skeleton.

Not: projectile

### Podium

Where you teach a spell to the students on the benches, by drawing its glyphs. Teaching one student alone gives them the drawing's whole effectiveness; a class makes the drawing longer and its band narrower, and gives each student less.

Not: lectern, stage

### Preparation

The part of each day when you act: {day.prep_s} seconds to read, make spells, recruit, teach and change the layout.

Not: build phase

### Raider

A student you send to a rival's battle. They walk in with that battle's enemies, cast their own spells, and attack the tower and the students on it. A raider is not on your own towers that battle.

### Recruitment table

Where you interview applicants who want to study at the college, one at a time. The college holds {students.max} students at most.

### Rehearsal

Today's battle played in the background against your plan as it stands, so you see what it would take off your score before the battle begins.

Not: forecast, prediction, simulation

### Rival

Another archmage in the match, played by a person online or by the computer.

Not: opponent

### Rotation

A list of spells a student casts in order, starting again at the end. Each step waits for that spell's cooldown and mana.

### School

One of the {schools.count} kinds of magic every spell piece belongs to. A student may be better or worse at a school.

Not: element

### Score

Your standing in the match. It starts at {score.start} and falls by every point of damage your tower takes; nothing raises it. The highest score after the last day wins.

Not: health, points, lives

### Seal

The short puzzle that keeps a spell at the laboratory: one ring for each piece, turned until its glyph sits under the star. It changes how long saving takes, never the spell.

### Spell

A chain of spell pieces made at the spell laboratory. A student casts the whole chain.

Not: ability

### Spell laboratory

The table where you combine spell pieces into a spell.

Not: lab, workshop

### Spell piece

One payload, modifier or trigger, learned from a book.

Not: rune, component

### Student

A mage who studies at your college. Students stand on your tower in the battle and cast the spells you taught them. A student holds two to four spells, more the better their interview went; a new one replaces their weakest.

Not: apprentice, pupil

### Sub-tower

A small tower placed nearer the path, with room for one student.

### Teaching elemental

A summoned penguin that teaches the class at the podium for you, as well as you drew its summoning glyphs. It takes its mana again for each lesson, until you send it away.

### Temperament

How a student decides what to do on the field, such as Hunter or Steadfast. Each temperament is a behaviour tree you can read at office hours.

### Terrain feature

Something placed on the battlefield: a wall, a lava pool, bait.

### Tower

Your mage tower. Enemies walk the path to it, and every hit that reaches it lowers your score. A tower never falls.

Not: castle

### Trigger

A spell piece that starts the rest of the spell when something happens: on hit, on death.

### Type

What an enemy or a spell is made of, such as Fire, Leaf or Bone. Some types take double or half damage from others, and none is immune.

Not: element

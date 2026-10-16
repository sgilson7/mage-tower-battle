# Triage

Stage 7. One row per request after the person has used the build: a bug, a change, or a new idea, in the person's own words. Rows are numbered T1, T2 and so on, and each commit that answers one names it ("Fix the timer reset (T4)"). `C4 Verify against hand-computed cases`.

Compare at least one result with a case worked out by hand. **Severity** is *blocks use*, *wrong*, or *wanted*. **Cost** is the agent's estimate. **Disposition** is *done*, *open*, *declined* (with the reason), or *try* (built on a branch and published as a TRY commit that says how to undo it).

| # | Request, in the person's words | Expected | What happened | Severity | Cost | Disposition |
| --- | --- | --- | --- | --- | --- | --- |
| T1 | "you should know a spell by default that creates walls and towers to station your mages on, they just take up to a minute to spawn and consume mana." | A new archmage can raise walls and sub-towers from the first day; each costs mana and takes time to appear. | Walls and sub-towers had to be learned from a library book first; with mana they appeared at once. | wrong | M | done (R4.1) |
| T2 | "how do you cast the elementals right now?" | A player can find out how to summon a helper. | A helper piece is learned from a book, then summoned at the podium by drawing its glyph; the screen did not say so. | wrong | S | done (R4.5) |
| T3 | "theres also no way to leave the map table right now without quitting the game or startingt he battle." | Esc or a button closes the map and the archmage walks again. | The map could not be left. | blocks use | S | done (R4.3) |
| T4 | "also write a testing suite similar to galactic survivors that tests permutations of combinations." | Every spell chain up to some length is played in a fixed scene and checked against rules, with a logbook like Galactic Survivors' rig log. | No such suite. | wanted | L | done (R4.20–R4.22) |
| T5 | "also make the battle window independently draggable, zoomable, move withinable, etc." | Each battle view can be orbited, zoomed and panned on its own with the mouse. | The cameras were fixed. | wanted | M | done (R4.7) |
| T6 | "also the chips for spells ar white background with white text so itslike impossible to read." | Spell chips are readable. | White text on a white chip. | blocks use | S | done (R4.4) |
| T7 | "also how do you cast invasion type magic?" | A player can find out how to send an invasion. | The screen did not explain it. | wrong | S | done (R4.5) |
| T8 | "also each spell should have a glossray index and a glossary the player can access at any time in game to read up on spells, modifiers, triggers, etc" | A glossary screen, open at any time, with an entry for every spell piece and every game term. | Only hover hints on some words. | wanted | M | done (R4.6) |
| T9 | "after you click the map window to place something ,all of js buttons like pick, turn, etc bucome unclickable permanently" | The map's buttons work after a placement. | Once something was rising, the map panel was rebuilt about 170 times a second, so a press and its release landed on different buttons and no click came. | blocks use | S | done (R4.12) |
| T10 | "the chip fpr the spell name should be way bigger, give an explanation and an icon, and linger for about 3 seconds or until the next book is read, and replace the glossary with these new chips" | A large card for the piece a book taught, with its glyph as an icon and what it does, for 3 s or until the next book; the glossary made of the same cards. | A one-line toast. | wanted | M | done (R4.13) |
| T11 | "i also want an animated landing page like ape race has when you start the game" | An animated title screen. | A plain menu over a dark background. | wanted | M | open |
| T12 | "and a whats new tracker" | A list of what changed in each update, with what is new since your last visit marked. | None. | wanted | S | open |

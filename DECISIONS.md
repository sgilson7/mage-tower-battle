# Decisions

Dated entries. Each quotes the person's request word for word, then says what the agent decided and where it is recorded. Grants of autonomy go here too ("you may publish"), with their scope: this run only, or until withdrawn.

## 2026-10-16

Answers to the ten questions in design journal entry 1, word for word:

> 1. yes 2. yes 3. yes 4. the more damage you take, the lower your score goes. whoever has the highest score at the end of 3 days wins. you cannot gain score, so it serves as essentially an infinite health pool used to differentiate between players at the end of 4 days  5. yes 6. yes 7. it is garbled but yeah thats the interpretation 8. yes. 9. yes i want to read the plan, then after my approval you do the entire mvp then deploy to my github pages 10. yes, publish to a public github repo and github pages as  the deployment ci. now create mage tower

The agent decided:

- Online play for four players is in the build, through `kits/online`, after solo play works (Q1; SECOND-ORDER R1.6).
- The five schools are Destruction, Alteration, Conjuration, Illusion and Restoration (Q2; R1.4).
- Computer-played rivals fill empty seats (Q3; R1.7).
- Towers never fall. Each tower has a score that only goes down, by the damage it takes; the highest score after the last day wins (Q4; R2.1). The answer names both 3 and 4 days; the plan uses 4, held in data, and asks (R2.2).
- 4-minute preparations, 4 days (Q5; R1.9). The mockup's teaching band and mana-cost numbers are starting values (Q6; R1.11). "Dukat Dock" means the spell at full effectiveness (Q7; R1.2). Students' invasions scale with each student's effectiveness (Q8; R1.14).
- **Autonomy, granted until withdrawn:** the person reads `PLAN.md` first. After the person approves it, the agent builds the whole MVP without stopping, creates a public GitHub repository, and deploys to GitHub Pages through a GitHub Actions workflow (Q9, Q10). Recorded in `BRIEF.md`'s Autonomy paragraph; the deploy mechanism is a change recorded in `BUILDER-SPEC.md` (R2.3).

## 2026-10-16 (plan approved)

The person's answers to the plan's questions and approval, word for word:

> 1. lets try 4 days 4 minutes first, with the understanding this may need to be tuned / adjustable in game as a setting 2. yes 3. yes 10000, it should be rather difficult if not borderline impossible to get someone to 0 4. max 90 seconds, with the understadning we may tune this / change simulation speed to improve gameplay flow 5. sure lets start with 24, and we can slowly start to push it to see how many it can take. try and use all the optimizations long time coming uses to get like a ton of orbs on the screen at once with no lag 6. make basic satisfying chip noises based on early nes games for all sounds 7. yes practice mode should be included, plus a vs ai mode if you dont have friends to play with online, and maybe some time trial / achievement seeking modes for extra single player fun like ape race has (ptonkle.github.io ape race). now build mage tower, dont stop till its deployed and an entry is made on my website, you have approval to deploy to sgilson7.github.io with a new chip for this game

The agent decided:

- The plan is approved (stage 3). The agent builds the MVP without stopping, deploys it, and adds a chip for it to sgilson7.github.io. **Grant, this run:** publish `sgilson7/mage-tower-battle` to GitHub Pages and change the website repository to add the game's entry.
- 4 days, 4-minute preparations, 90-second battles, each a setting in the game's setup, with a simulation-speed setting for the battle (R3.1).
- Score starts at 10,000 and is tuned so reaching 0 is close to impossible (R3.2).
- 24 ragdolls per arena to start; the probe pushes the cap up; orbs and effects are drawn with Long Time Comin's instancing methods (R3.3).
- Sound: square, triangle and noise chip sounds in the style of early NES games, for every sound (R3.4; changes D3's reach to sound bytes from the core).
- Modes: Practice, Versus AI, Online, and single-player trials with achievements after Ape Race (R3.5). Personal bests and achievements are kept in browser storage, a change to D17 (R3.6).

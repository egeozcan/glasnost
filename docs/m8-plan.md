# M8 plan: fidelity, AI depth, polish

Agreed with the project lead on 2026-10-21. PLAN.md §5 gives M8's scope: turning `unmeasured` and
`measurable` claims into measured ones, movement and crowding feel, accessibility and settings, low-end
iGPU performance, optional WebGPU extras, and music. This file breaks that into steps. Every step works
from approved specs and measurement records only (CLAUDE.md). Where a step needs behaviour that no spec
gives, it files a request and stops that part.

## Decisions of 2026-10-21

- **Order:** track B (AI depth) and step A1 (the arena-spread audit) start together. Neither needs a
  new spec. Track C (polish) follows.
- **Oracle and spec rounds:** the lead runs measurement scenarios and spec rounds regularly during M8.
  Track A is therefore a real track, fed by step A2's ranked scenario list and by the requests the steps
  file.
- **Balance patch:** after M8. M8 has no balance step. Whenever it comes, the patch is an optional
  rules layer, off by default, and never limits the AI (D14).
- **Specless items:** requests filed now: Q-1700 (music playback), Q-1701 (the electric-bolt shot,
  answered by S-0316 and built at C5 part 1), Q-1702 (voxel animations in play).
- **"Statistical suites green":** no nightly schedule. The oracle arena and the seed sweeps run at every
  M8 step's merge, as in M7b, and each step's ledger row gives the arena table. The oracle arena needs a
  real install, so it stays in the local lane, never in CI. CI keeps the synthetic arena.
- **Performance target:** 60 fps at 1080p in a late full game, in Firefox on WebGL2, on the
  integrated GPU of a typical Intel laptop (Iris Xe class). Step C4 needs such a laptop
  for its measurement.

## Decisions of 2026-10-25

- **Track B is done.** The lead accepts B7's exit misses: the dominator's tempo (+1 167 ticks) and
  the wins against saving off (28 where the exit asked for 30). There is no step B8.
- **Autonomy:** from here to the end of M9, the main session decides on its own, aiming for a result
  that is fun, bug free and maintainable, and accepts small divergences for that. The rules of
  CLAUDE.md and PROVENANCE.md still bind (approved specs and records only, citations, requests for
  what no spec gives). Where a spec leaves a case open, an interim chosen for play and stated as our
  design in the step's ledger goes ahead without waiting on the lead.
- **Pathing and other fixes (step C7), the main session's decision:** our own rules patch layer
  that sets chosen quirk switches to the off position their spec states, where the original's
  position reads as a bug in play and the change is not one of balance. It is an option of the
  skirmish setup and the network room, which the host sets, and it is **on by default** in the
  client's new-game setup. The switches' defaults in `quirks.rs` stay the original's (D6, D8),
  and the measurement runs (the oracle arena, the seed sweeps, the gates, the tick bench and the
  fidelity suite) stay on the reference ruleset unless `GLASNOST_PLAY_FIXES=1` is given (the
  sweeps and the skirmish lane read it). The layer is part of the rules hash, so replays, saves
  and network starts carry it.

## Lead rulings of 2026-10-24 (awaiting spec revisions)

The lead's rulings, relayed by the speccer. Each reaches the code only when the spec revision that
carries it is approved and taken in by an intake; until then the code keeps to the approved
revision. The intake that takes in each revision checks it against this list.

- **S-0108:** a captured tech building, or one that changes owner, restarts its production slots
  as in the original, with no quirk switch.
- **S-0109:** the Import mapping may name the stock invisibility animation literally.
- **S-0212:** the building-exemption leak is fixed in our engine, with no quirk switch.
- **S-0319** (new spec): a zero speed span or zero trailer separation takes the spec's safe
  fallback, with an importer warning.
- **S-0201:** the game-speed control stays shown after the player's defeat.
- **S-0105:** the IFV's per-stem defaults are stated in the spec (play notes 3 and 14, Q-1760).
- **S-0703:** the end screen opening at game over is accepted (built at intake 12).
- **S-0703 and S-0701:** while the options menu is open in a loaded skirmish, the music plays on,
  as in the original, with music time on the host's monotonic clock and lost focus still a full
  stand-still (S-0701 claim 30). This reverses intake 12's options-menu hold (the revisions
  landed as S-0701 r35 and S-0703 r58 and were built at intake 25). The glossary's *departure* and *audio clock* entries change with it.
- **S-0314:** a docked hover unit lost at a naval yard gets `quirks.docked_hover_water_loss`,
  default the original behaviour.

## Request blocks

| Block | Use |
|---|---|
| Q-1700 to Q-1709 | Plan level (Q-1700 to Q-1702 filed) |
| Q-1710 to Q-1719 | Step A1, arena spread |
| Q-1720 to Q-1729 | Track B, AI depth (only where the AI needs simulation behaviour no spec gives) |
| Q-1730 to Q-1739 | Track C, polish |
| Q-1740 to Q-1749 | Intakes of new spec revisions during M8 |
| Q-1750 to Q-1769 | Step A4, the lead's play notes ([m8-claims/a4-play-notes.md](m8-claims/a4-play-notes.md)) |
| Q-1770 onward | Further intakes |

## Decisions of 2026-10-29

- **Our own pathing improvements (the lead's decision):** "we can improve pathing where we see fit.
  Departure from the original game is approved." Pathing and movement changes of our own design go
  ahead without a spec or a request, marked `// design:` and recorded in the step's ledger, under
  the determinism rules of CLAUDE.md. The main session's choice for how: each improvement sits behind
  a switch whose default is the original's behaviour, and the play-fixes layer of C7 turns it on, so
  the client's new games get it by default while the oracle arena, the seed sweeps, the gates and
  the fidelity suite stay on the reference rules (as for C7, `GLASNOST_PLAY_FIXES=1` measures with
  it). The layer is in the rules hash, so replays, saves and network starts carry it.
- **Pathing part 5 kept whole although the layer's quick-lane median grew** (main session): with
  the part 5 layer, the median AI-against-AI game is 4.3 % longer than the reference (96 games,
  45 197 against 43 334 ticks). Every game still ends with a winner, nothing is stuck, harvest
  income per tick rises, and dropping either the rally spread or the harvesters' field cells alone
  brings the median back to the reference, so neither is the cause on its own. Units that no longer
  give up matter more in play than a slightly longer AI game, so both switches stay in the layer.

## Decisions of 2026-11-10

- **A6b merged although the arena count rose from 28 to 33 failing rows** (main session). The step
  built two original-only rules the approved specs give for the test-only computer-controlled
  houses: S-0309 claim 25 (e)'s occupation alternative and S-0215 claims 23 and 27's infantry rule
  for a building it may take. Leaving them out would keep a known departure from approved claims.
  The six rows that newly fail come from a chain of approved rules (a refused occupier falls back
  to rest and claim 28's hunt rule leaves occupier types out). Q-2926 asks whether the original
  behaves the same. The exit's "arena failures fall" waits on that answer; nothing is tuned toward
  the records meanwhile. Our AI houses are human-flagged and not reached; the quick lane, sweeps
  and goldens did not move.

## Track A: fidelity

M7b ended with the oracle arena failing 55 of 155 measured pairings (M-0001 to M-0003), mostly because
our games vary much less from seed to seed than the stock games. See *Arena mode (M7b)* in
docs/development.md.

| Step | Work | Exit check |
|---|---|---|
| A1 | **Arena-spread audit.** (a) Check that the arena's seed reaches every simulation stream of S-0202, not only one. (b) Tabulate each random draw that the combat, movement and targeting specs name and an arena game can reach, and mark it as built, behind a seam, or on an interim rule. (c) Rank the failing pairings by how far our spread falls short. (d) Build the draws that are specified but still behind seams. (e) File a request for each spread gap that no built or specified draw explains. Nothing is tuned against the records. | `docs/m8-claims/a1-arena-spread.md` holds the draw table and the ranking. The arena table is rerun and its counts given before and after. Requests are filed for what stays unexplained. |
| A2 | **Ranked scenario list for the oracle.** The `measurable` claims whose measurement would most change play (those the arena games reach, frequent rules, the combat and movement core), with each claim's spec scenario, for the lead's oracle rounds. | `docs/m8-claims/oracle-wishlist.md`, kept current as records land. |
| A3 | **Intakes (recurring).** Take in new measurement records and spec revisions as they land: claim ledgers, tests citing the M-ids, retired interim rules, and the arena rerun. | One ledger row per intake. |
| A4 | **Movement and crowding feel.** Starts once the lead's play notes or oracle records name concrete gaps. Each gap becomes a request, or an intake once a spec covers it. | Per gap. |
| A5 | **Animation playback in the simulation** (proposed by A1). S-0109's random parts (claims 11, 14, 15, 17, 28 and 29) are reached in nearly every arena game but not built; the arena's animation creation only leaves a note today. Also S-0312 claim 18 (survivors). | The S-0109 ledger rows reached by the arena are tested; the arena table is rerun before and after. |

## Track B: AI depth

This is our own AI (D3), so these steps need no new behaviour spec. They use only orders a person
could give (D3) and never hold the AI back (D14). Each step plays the quick lane (seeds 7 to 10, maps 0,
2 and 8, each side against each) and the six full games of M7b exit 4d (*Full skirmishes*,
docs/development.md), before and after.

| Step | Work | Exit check |
|---|---|---|
| B1 | **Expansion in full games.** No expansion yard vehicle was built in the six full games of the 4d rerun. Find out why (superweapon saving blocks it every time, the wealth threshold, no field qualifying), then fix the brain. | Each full game that lasts past the yard vehicle's start builds and deploys at least one expansion yard, or the ledger row names a reason a person would accept (no free field on the house's half, for example). |
| B2 | **Superweapon saving without losing tempo.** Saving is blamed for Yuri's drop against Soviet (8 of 12 wins with it kept, 11 of 12 before). Make the hold narrower: only items that compete for the money, only while it shortens the time to the superweapon, never while the house is losing a fight. | The quick lane wins for the saving side are no fewer than with saving off. The superweapon is ready no later than before. |
| B3 | **The dominator takes units on stock data.** In the 4d rerun every launch took nothing: the seen enemy units at the best cell were immune by data or in the air. Check the aim's candidates against the takeover rules of S-0612 claims 13 and 14, and aim where takeable units stand. Fire for damage only when no cell offers any. | In the full games where the dominator is launched, most launches take at least one object. |
| B4 | **Gate margins that do not hang on one seed or quirk.** Each duel gate's ability counters stay above zero over seeds 1 to 32, not only at the chosen seed, and with each quirk switch the duel reaches flipped. Fix by setup, as in M7b steps 2a and 2b, unless the AI itself is at fault. | A sweep table in the ledger: the minimum and median of each counter over 32 seeds. No zeros. |
| B5 | **Tempo after expansion** (follow-up of B1 and B2, which missed B2's exit narrowly). The Yuri superweapons against round 3's saving (mutator a median +618 ticks), the full game with no winner at 120 000 ticks (Allied, map 2, seed 7), a yard vehicle left on the line while the house is broke, and Yuri's full-game wins (4 of 6 before B1 and B2, 2 of 6 after). | B2's exit against round 3 and against saving off, over seeds 7 to 14; every full game of the six has a winner. |

B1 and B2 both change `Brain::economy_first`, so they share one branch. B3 runs beside them. B4 comes
after B1 to B3 and B5, since their AI changes move the gate counters it sets margins for.

## Track C: polish

These are our own engine and client, except where a step names a request.

| Step | Work | Exit check |
|---|---|---|
| C1 | **Music.** The decoder exists (the ADPCM path of the sound effects). Track choice, order, controls and volume follow Q-1700's answer. On the web, the imported files are kept between visits (the OPFS share of PLAN.md §5). | Waits for Q-1700. Then: a lane scene plays a skirmish's first tracks, and the music has no effect on the state hash (as S-0701 claim 29's test checks for sound). |
| C2 | **Settings.** A settings screen and its storage: volumes (sound, voices, music), scroll speed, the key bindings the client has, the display scale. Our own UI. No setting reaches the simulation. | Settings persist natively and on the web, and a replay's hash is the same whatever the settings. |
| C3 | **Accessibility.** UI scale, keyboard-only play of the sidebar and menus, captions for announcements, and a colour-blind option for house colours (presentation only, off by default). The lead may add or drop items before the step starts. | Each item with a lane picture, or a test where one is possible. |
| C4 | **Performance.** A benchmark that plays a late full-game replay with frame timing, natively and in the browser. Profile and fix the renderer and the client until the target is met. | 60 fps at 1080p on a typical Intel laptop iGPU in Firefox WebGL2 (see Decisions). |
| C5 | **Electric bolts and voxel animations.** The electric-bolt shot, its sparks (`emitters::electric_bolt_sparks`), voxel animations in play and their particle systems (`emitters::voxel_animation_system`). | Part 1 (electric bolts, S-0316, answering Q-1701): built, ledger [docs/m8-claims/c5-S-0316.md](m8-claims/c5-S-0316.md). Part 2 (voxel debris pieces, S-0317, answering Q-1702): built, ledger [docs/m8-claims/c5-S-0317.md](m8-claims/c5-S-0317.md). |
| C6 | **Optional WebGPU extras.** Only behind the runtime check (D9), after C4. | Lowest priority; may move past M8. |
| C7 | **Pathing and other fixes.** Our own rules patch layer (`data/layers/play-fixes.jsonc`) of quirk switches set off where the spec states the off position, the on position reads as a bug in play (a play note or a request names it, or the spec's own Edge cases or quirk text states its effect as a unit stuck, giving up although an open way exists, never moving again, or a visible glitch; widened 2026-10-28), and it is no balance change; first entries `corridor_retry_no_fallback` and `corridor_split_probe` (S-0212 claims 13 and 13a, answering pathing reports 1 and 2, Q-1771 and Q-1870; part 3 re-judged the list, see the ledger). A visible option in the skirmish setup and the network room (the host chooses), on by default; in the rules hash; saves, replays and every report keep it. Measurement stays on the reference ruleset unless a flag is given. | A synthetic test plays, saves, loads and replays a game with the layer; the lead's two pathing reports replayed with the layer show the harvester leaving the dock; the sweeps and the quick lane with the layer on have no stall or panic; every default run is byte-identical to main. Built, ledger [docs/m8-claims/c7-play-fixes.md](m8-claims/c7-play-fixes.md). |

## Housekeeping (any time)

- **H1.** Rename the synthetic type `BRUTE` in `crates/deterministic/sim/tests/claims_s0611.rs` to an invented name, since it matches a stock unit's ID.
- **H2.** Closed 2026-10-21, no change: a depot repairing a vehicle whose link stays untethered (seen during the M7b step 1j sprites intake) is what S-0314 claims 6 and 17 say ("paid repair does not require tether"); `depot::process_enter` sets the tether only through the pad exchange.
- **H3.** `docs/m6-claims/S-0501.md` row 20 still has M6a wording: bring it up to date.

## Exit

- Tracks A and B are done as above. Every step's ledger row gives the arena table and the seed-sweep minimums at its merge.
- The arena failures fall, with every change made against specs and records only.
- Track C's steps are done, except those still waiting on a request. Those stay listed with their request.
- The lead runs a playtester survey (PLAN.md §5).

## How the steps run

As in M7b: one implementer agent per step in its own worktree. A git-ignored brief gives the milestone,
the spec and request IDs and the lead's notes. Before every merge, `cargo fmt`, clippy, `cargo xtask
check`, `cargo test --workspace` and `cargo xtask gate` must pass. A golden or format change is named,
and its save or AI format is bumped. The step ledger is in docs/development.md, as for M7b.

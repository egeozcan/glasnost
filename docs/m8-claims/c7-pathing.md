# Our own pathing improvements: ledger (M8 step C7 part 4)

Brief: `.claude/worktrees/M8-C7-PATHING-BRIEF.md`. The lead's decision of 2026-10-29 (m8-plan.md, *Decisions of 2026-10-29*): "we can improve pathing where we see fit. Departure from the original game is approved." Each improvement here is our own design, marked `// design: C7-P4 <name>` in the code, and sits behind a quirk switch whose default (on) is the original's behaviour; the play-fixes layer (*Pathing and other fixes*, [c7-play-fixes.md](c7-play-fixes.md)) sets it off. The designs come from general pathfinding practice (wait-for graphs for deadlocks, searches that treat some cells as closed, nearest reachable goal, local spreading); none is read from the original.

No saved or hashed state was added (`path::Request::avoided` lives only for one search; the blockage timer `blocked_since` was already saved), so the save format (85) and the AI game format (47) are unchanged. No request was needed (block Q-2101 to Q-2105 unused).

## The scenario set

`crates/deterministic/sim/tests/pathing_common/mod.rs` (synthetic maps 50 × 50, types and layouts of our own, no game data) and `crates/deterministic/sim/tests/pathing_c7p4.rs`. `cargo test --release -p glasnost-sim --test pathing_c7p4 measure -- --ignored --nocapture` prints the table for the reference, the layer, and each of the three switches alone off (about 40 s); the `diag` test (ignored; `DIAG=<scenario> WATCH=<n> OFF=<switches> SEED=<n>`) traces one unit.

What is counted, per scenario, over seeds 7, 8 and 9 pooled:

| Column | Meaning |
|---|---|
| Median, Max | ticks from the order (from the release, for a factory's units) to the unit standing within the group's radius of its cell (half the side of the smallest square that holds the group, plus one) |
| Missing | units not arrived within the scenario's bound |
| Give-ups | failed computations that cleared the destination, and given-up moves; *open* those of units not arrived in a scenario whose goals are all reachable, *late* those of units already arrived (a later move inside the group's spread) |
| Reversals, Jostles | a unit entering the cell it left two cell changes before, before and after its arrival |
| Unsettled | units still holding a destination at the end |
| Shared | units whose first destination after the order is also another followed unit's |
| Searches, Failed | path computations, and those that failed |
| µs/tick | mean wall time of a tick (not compared; the same machine and run) |

The scenarios cover the brief's list: gaps of 1 and 2 cells (12, 16 and 20 vehicles), two groups head-on in corridors 1, 2 and 3 cells wide (1, 3 and 6 each way), 16 vehicles to one cell in the open, beside an L of buildings and on a shore, 4 harvesters on one refinery and field, a factory releasing 8 vehicles past 5 parked near its exit, infantry and vehicles mixed through a 2-cell gap, 16 infantry through a 1-cell gap, two speeds through a 2-cell gap, a cell walled in by buildings and a cell inside a closed ring of rock, a causeway 1 cell wide, a long wall to go round, and two closed rooms joined only by a 2-wide corridor (3 and 6 each way).

**The reference (every switch at its default), worst rows.** Two groups of 6 meeting head-on in a 2-wide corridor: none of the 36 arrives, 7 014 computations, every unit still holding its destination at the end. A group sent into a ring of buildings: all 24 give up at the first computation; a group sent into rock: all 24 are dropped by the gate and do not move. Beside buildings, 39 of 48 units share a destination and 33 give up after arriving; on a shore 18 share and 18 give up. The other rows have every unit arriving and no give-up, apart from the harvesters (12 give-ups, 29 jostles) and the factory stream (12 late give-ups, 24 shared; the rally point is one cell). These are the three fixes below.

## Fix 1: stalled blockers (`path_through_stalled_blockers` off)

**Problem.** In the corridor, each front unit's re-path after the blockage delay (S-0212 claims 8 and 22, mode 2) leads through the other group's front unit, which is a moving blocker at a cost, so both keep heading into each other. The original never resolves it.

**Design.** A *stalled* unit is a vehicle or infantry unit on the ground layer, holding a destination, whose blockage timer has run for the blockage delay. The stalled units form a wait-for graph: each waits on the stalled units in the cell it heads for (S-0212 claim 8's heading cell; the searching unit, whose stored path a refused step has just cleared, waits on the stalled units round it). Units that wait on nothing still in the graph are taken away until none is left (a queue whose head still moves drains this way); what remains are the units in a cycle and those waiting on one: the deadlock. When a stalled unit in the deadlock computes a path, a first search without a corridor treats the cells of the deadlock's other units as closed (the destination aside), bounded to 4 096 expansions (`DETOUR_EXPANSIONS`, a sixteenth of the search limit of S-0212 claim 12); when it finds no path, the computation's own searches follow unchanged. One more change makes the deadlock visible: a step refused by a unit holding a destination (the close case of S-0212 claim 27 among them) starts the blockage timer as a moving blocker does; without it the front units looped in fresh searches with no timer running.

**Effect.** Corridor 2, 6 each way: 36 missing → 0; median 585, max 772; 7 014 computations → 825; µs/tick 105 → 68. It is not elegant: both groups back out of the corridor and the side that ends up with room passes (27 reversals). A queue is left alone: the infantry through a 1-cell gap, 20 vehicles through a 1-cell gap and the 2-wide corridor with 3 each way play exactly as the reference (a first version without the graph sent queued units round, and the infantry's maximum rose from 562 to 1 490).

**Tests.** `pathing_c7p4.rs`: `stalled_blockers_off_clears_a_head_on_deadlock` (both positions: 12 missing on, 0 off and no give-up), `stalled_blockers_off_leaves_queues_alone` (the three queue shapes play the same in both positions).

## Fix 2: nearest reachable cell (`unreachable_move_dropped` off)

**Problem.** A move to a cell the unit cannot reach fails: outside the unit's reach region the gate fails and the destination is cleared (S-0212 claims 6 and 23), inside a ring of buildings the search fails and the unit gives up. The units stand where they were; a person clicking on a building or a rock sees nothing happen.

**Design.** When a move's computation fails, one more try: the nearest cell to the destination, ring by ring out to 12 (`NEAR_RADIUS`), that is on the map and usable, in the unit's own reach region (`regions::reach_read`), whose entry is clear or a moving blocker, and that no other unit of the same house holds as its destination (so a group gets spread cells); of a ring the cell nearest the destination, the first in row order on equal distances; and only when it is nearer than the unit's own cell. The destination becomes that cell and the computation runs once more. It applies to every failed computation of a move to a cell, the blockage re-paths too: a unit whose destination becomes unreachable on the way (a building placed on it) stops at the nearest cell instead of giving up. It rewrites the commanded destination, so a later look at the unit's destination shows the substitute.

**Effect.** Unreachable cell: 24 missing and 24 failed → 0 and 0, median 111, max 157. Inside buildings: 24 give-ups → 0 (this switch alone: median 127, max 190).

**Tests.** `unreachable_move_off_goes_to_the_nearest_reachable_cell` (rock ring: 8 missing and 8 failed on, 0 and 0 off; building ring: 8 missing and 8 give-ups on, 0 and 0 off).

## Fix 3: group spread (`group_spread_no_detour` off)

**Problem.** A group sent to one cell spreads its members round it (S-0213 claim 7). When a member's look runs into an obstacle (buildings, water) it gets the fallback cell, which other members already have; S-0213's Edge cases name the effect. The members crowd one cell, jostle and give up after arriving. S-0213 left the off position open; part 1 rejected the switch for that reason, and the lead's decision makes it ours to design.

**Design.** A member whose look gives no cell of its own gets the nearest open cell round the common cell, ring by ring out to 6 (`SPREAD_RADIUS`, the look's length of S-0213 claim 7): usable, in the common cell's reach region (`regions::reach_in`), not given to another member, within the height rule of S-0213 claim 7 (a level difference under 3) and not obstructed; nearest to the common cell. With none, the original's fallback stands.

**Effect.** By buildings: 39 shared and 33 late give-ups → 0 and 0, median 166 → 154, max 252 → 182, computations 423 → 306. Shore: 18 shared and 18 give-ups → 0 and 0, median 133 → 128, but max 176 → 217: the last members now walk round to cells of their own on the far side instead of stopping in the crowd. Inside buildings, this switch alone already gets 21 of 24 there (the building ring is round the common cell, and the members get cells outside it).

**Tests.** `group_spread_off_gives_each_member_its_own_cell` (by buildings: shared and give-ups on, none off; shore: shared on, none off).

The layer itself: `the_layer_turns_our_switches_off`; `layers::tests::play_fixes_sets_only_the_admitted_switches_off` now admits nine entries.

## The table with the layer (seeds 7 to 9 pooled)

Rows not listed play exactly as the reference: gap 1 (268/339), gap 2 (235/365), corridor 2 with 1 and 3 each way, corridor 3 (323/389), corridor 1 with 1 each way (532, 6 reversals), open (131/147), harvesters, factory stream, mixed through gap 2 (373/517), causeway (312/381), round a wall (387/415), infantry gap 1 (454/562), gap 1 with 20 (357/499), two speeds (337/427), two rooms with 3 each way (315/343).

| Scenario | Reference: median / max, missing, give-ups, shared, searches | Layer |
|---|---|---|
| corridor 2, 6 each way | –, 36, 0, 0, 7 014 | 585 / 772, 0, 0, 0, 825 |
| by buildings | 166 / 252, 0, 33, 39, 423 | 154 / 182, 0, 0, 0, 306 |
| inside buildings | –, 24, 24, 24, 24 | 111 / 179, 0, 0, 0, 96 |
| unreachable cell | –, 24, 0 (24 failed), 0, 24 | 111 / 157, 0, 0, 0, 63 |
| shore | 133 / 176, 0, 18, 18, 264 | 128 / 217, 0, 0, 0, 246 |
| two rooms, 6 each way | 287, 33, 0, 0, 6 831 | the same |

Tick cost in the set is the same or lower (corridor 105 → 68 µs, the others within a few µs).

## Left open, and proposals

- **Two closed rooms, 6 each way.** 33 of 36 stay stuck in both positions: there is no way round, so a search that keeps off the deadlock finds nothing and the original's path follows. It needs one side to give way. Proposal: when a deadlock's search finds no detour, the units of the deadlock with the lower priority (say the group whose order came later, or the higher object id) take a yield behaviour: they back out along their own path to the nearest cell off the other group's path and wait there until the other group's units have passed (a cooperative reservation of the passage, released as the units leave it). It is a new behaviour with saved state (the yield and its release), so a format bump; not done here.
- **The corridor resolves slowly.** Fix 1 lets both groups back out (median 585 against 280 for 3 each way). The yield above would also speed this up.
- **Harvesters** (12 give-ups, 29 jostles at the dock and the field) and the **factory stream** (12 late give-ups; all 8 released units go to one rally cell): not changed. A rally point spread like the group spread (each released unit its own cell round the rally point) would answer the second; the harvesters' docking queue is S-0402's and needs its own look.
- **Shore maximum** (176 → 217, above).
- **Random draws.** None of the switches draws on the simulation RNG. With the layer on, units end on other cells than in the reference, so later scatters and other draws differ; that is a different game, not an extra draw.
- **The lead's pathing reports 1 and 2.** The current `path-report` refuses both bundles (they were captured before later spec intakes changed the rules hash, and with save format 70 and 76), so they were not replayed; part 1 answered both with the corridor switches.

## Admission

`data/layers/play-fixes.jsonc` gains an exception to the layer's condition (1): our own pathing improvements (the lead's decision of 2026-10-29) are admitted without an approved spec's off position; their off position is our design, marked `// design: C7-P4` and described here, with a synthetic scenario that shows the bug in the on position. Conditions (2) and (3) still hold. This supersedes part 1's rejection of `group_spread_no_detour` ([c7-play-fixes.md](c7-play-fixes.md)).

## Results (local lane, outputs in the git-ignored `local-lane/m8/c7-pathing/`)

Two sets of binaries on the same base (main 574ef85): *before* from the commit that only adds the scenario set (no simulation change), *after* from the switches' commit with the ring gate (de2bb6d; the later commit changes switch text only). Then B10 (the AI's broke-house recovery, which changes the AI and the duel golden but no simulation code) was merged, and a third set, *rebased*, was built from this branch on main 50c6305 for the sweeps and the quick lane; intake 48 (S-0111 pips, client and importer warning only) came after.

- *Default runs, byte-identical.* The oracle arena (`arena --record all`): the three `ours-M-000*.csv` and the three oracle reports are byte-identical before and after. The reference M6c, M7a and M7b sweeps 1 to 16 print the same lines (only the test's wall time differs). The sim-gate goldens (`cargo test --release -p glasnost-sim-gate -- --include-ignored`, without update) pass and leave `crates/sim-gate` unchanged. The reference quick lane: all 48 games end on the same tick with the same result before and after.
- *Layer-on sweeps 1 to 16* (`GLASNOST_PLAY_FIXES=1`): all three pass. M6c smallest minimum 1 (the part-3 layer on the same base: 0, the radiation exits of seed 10; now seed 5's air-strike orders are 1); M7a smallest minimum 1 (before 0), and no seed ends early (before, seed 3 ended at tick 8 404 with a win, which the reference also does); M7b smallest drain minimum 3, as before.
- *Quick lane* (seeds 7 to 14, maps 8, 2 and 0, Yuri as house 1 against Allied and against Soviet, 48 games, `GLASNOST_AI_QUICK=1`). Reference: 47 games won (house 1 29, house 0 18) and one not over at the 120 000-tick bound (sides 1 and 2, map 0, seed 10), median 42 700 ticks counting that game at the bound. The part-3 layer: the same 47 and the same draw, median 41 933. With part 4's layer: every game over with a winner (house 1 27, house 0 21), no panic, median 42 240: no worse than the reference. On the rebased build (B10's AI): reference 47 won (house 1 27, house 0 20) and the same game not over at the bound, median 44 512; with the layer every game over with a winner (house 1 30, house 0 18), no panic, median 43 158.
- *Tick cost.* In the scenario set, the same or lower (above). In the quick lane, mean of the games' µs per tick: reference 1 810, layer 1 828 (+1 %; the part-3 layer before: 1 845); the games differ and another session's arena ran during part of it, so this is no like-for-like measurement, but nothing rose noticeably. On the rebased build: reference 1 907, layer 1 872.
- *Rebased sweeps.* The reference and layer-on sweeps print the same lines on the rebased build as on *after*, so B10 changes neither; the sim-gate goldens pass on the rebased tree against B10's regenerated duel golden.
- `cargo test --release -p glasnost-ai -- --include-ignored` on the rebased tree: 239 passed, none failed.

**Version effects.** Only the layered `sim_rules_hash` changes (the layer file's content is in it): a save, replay, report or room made with part 3's layer matches neither the reference nor the new layered hash and is refused on the rules hash. Reference files are unaffected. Save 85, AI game format 47, wire 9, pathing report 2 and desync bundle 3 are unchanged.

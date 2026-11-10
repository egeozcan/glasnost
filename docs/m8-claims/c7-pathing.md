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

Tick cost in the set (mean µs per tick, the same run): lower in the corridor (105 → 68, far fewer computations), within a few µs in most rows (unreachable cell 34 → 37, one more computation per unit), and higher in the case the fix cannot resolve: two rooms with 6 each way, 95 → 107 (+12 %; stalled blockers alone: 108). There every stalled unit's computation builds the wait-for graph over all objects and runs a detour search that fails at its 4 096 expansions before the original's searches, and the deadlock stays.

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

Two sets of binaries on one base (main as of the S-0111 intake 46 answers, before B10): *before* from the scenario-set commit (no simulation change), *after* from the ring-gate commit (the later commit changes switch text only). Then B10 (the AI's broke-house recovery, which changes the AI and the duel golden but no simulation code) was merged, and a third set, *rebased*, was built from this branch on main with B10 for the sweeps and the quick lane; intake 48 (S-0111 pips, client and importer warning only) came after, and the goldens, the workspace tests, fmt and clippy ran on the branch rebased onto it. The branch's simulation diff is the same on all three bases, and main's changes since the first touch only the AI, the duel golden, the importer's warning and the client, so the arena identity shown on the first base carries over.

- *Default runs, byte-identical.* The oracle arena (`arena --record all`): the three `ours-M-000*.csv` and the three oracle reports are byte-identical before and after. The reference M6c, M7a and M7b sweeps 1 to 16 print the same lines (only the test's wall time differs). The sim-gate goldens (`cargo test --release -p glasnost-sim-gate -- --include-ignored`, without update) pass and leave `crates/sim-gate` unchanged. The reference quick lane: all 48 games end on the same tick with the same result before and after.
- *Layer-on sweeps 1 to 16* (`GLASNOST_PLAY_FIXES=1`): all three pass. M6c smallest minimum 1 (the part-3 layer on the same base: 0, the radiation exits of seed 10; now seed 5's air-strike orders are 1); M7a smallest minimum 1 (before 0), and no seed ends early (before, seed 3 ended at tick 8 404 with a win, which the reference also does); M7b smallest drain minimum 3, as before.
- *Quick lane* (seeds 7 to 14, maps 8, 2 and 0, Yuri as house 1 against Allied and against Soviet, 48 games, `GLASNOST_AI_QUICK=1`). Reference: 47 games won (house 1 29, house 0 18) and one not over at the 120 000-tick bound (sides 1 and 2, map 0, seed 10), median 42 700 ticks counting that game at the bound (that game is the reference's own, on the before binaries too; so "every game ends with a winner" holds with the layer, not without it). The part-3 layer: the same 47 and the same draw, median 41 933. With part 4's layer: every game over with a winner (house 1 27, house 0 21), no panic, median 42 240: no worse than the reference. On the rebased build (B10's AI): reference 47 won (house 1 27, house 0 20) and the same game not over at the bound, median 44 512; with the layer every game over with a winner (house 1 30, house 0 18), no panic, median 43 158.
- *Tick cost.* In the scenario set, see above (higher only in the two-rooms deadlock it cannot resolve). In the quick lane, mean of the games' µs per tick: reference 1 810, layer 1 828 (+1 %; the part-3 layer before: 1 845); the games differ and another session's arena ran during part of it, so this is no like-for-like measurement, but nothing rose noticeably. On the rebased build: reference 1 907, layer 1 872.
- *Rebased sweeps.* The reference and layer-on sweeps print the same lines on the rebased build as on *after*, so B10 changes neither; the sim-gate goldens pass on the rebased tree against B10's regenerated duel golden.
- `cargo test --release -p glasnost-ai -- --include-ignored` on the rebased tree: 239 passed, none failed.

**Version effects.** Only the layered `sim_rules_hash` changes (the layer file's content is in it): a save, replay, report or room made with part 3's layer matches neither the reference nor the new layered hash and is refused on the rules hash. Reference files are unaffected. Save 85, AI game format 47, wire 9, pathing report 2 and desync bundle 3 are unchanged.

## Part 5

Brief: `.claude/worktrees/M8-C7-PATHING5-BRIEF.md`. The same rules as part 4: each change is our design, marked `// design: C7-P5 <name>`, behind a quirk switch whose default (on) is the original's, set off by the play-fixes layer, which now admits twelve switches. Part 4's statement above (no saved state, formats 85 and 47) holds for part 4; part 5 adds saved state (the yield, below). No request was needed (block Q-2116 to Q-2120 unused). The scenario set is part 4's, with one new ignored test, `cost` (below); `measure` now also prints a column for the layer less each switch.

### Rally spread (`rally_point_shared` off)

**Problem.** Every unit a factory releases with a cell rally point is sent to the rally cell itself (S-0417 claims 8 and 10, S-0404 claim 13). A stream crowds the one cell: in the factory stream all 8 units of a seed share their destination, and 12 give up after arriving.

**Design.** `orders::rally_cell`, at each of the three release paths (ground vehicle, naval, infantry: the saved guard spot or the move order): the rally cell while no other mobile unit of the house has it as its destination or guard spot and it is not obstructed; else part 4's `open_near` round the rally point, with those cells taken (the same open class as the group spread: usable, the rally cell's reach region, a level difference under 3, not obstructed, out to `SPREAD_RADIUS`); else the rally cell. An object rally point is kept. It takes no draw.

**Effect.** Factory stream: 12 give-ups and 24 shared → 0 and 0, computations 39 → 27, median 247 as before, max 249 → 283 (the last units now drive on to a cell of their own instead of stopping in the crowd).

**Tried and dropped.** A tie-break that preferred cells on the rally point's far side: on seed 7 one unit then accepted a scatter's request to move off (an original behaviour, S-0212's scatter) and lost its destination, so the plain nearest cell stays.

**Test.** `rally_spread_off_gives_each_released_unit_its_own_cell` (on: 8 shared and late give-ups; off: none shared, no give-up, none missing).

### Yield (`deadlock_no_yield` off)

**Problem.** Part 4's open case: two closed rooms joined by a 2-wide corridor, 6 units each way. There is no way round, so part 4's detour search finds nothing and 33 of 36 stay stuck in both positions.

**Design.** When a stalled unit in a deadlock (part 4's wait-for graph, `path_through_stalled_blockers` off) computes a path, part 4's bounded detour still runs first. Only when it finds none, and the unit is *shut in*, one side gives way:

- *Shut in*: the units of the deadlock heading against the unit are taken as walls, and a fill from the unit and one from its goal (each bounded to 2 048 cells, `YIELD_FLOOD`) do not meet. The fill gives up (no yield) on a deck, outside the usable area, at a bridge cell or at the bound. The test keeps the yield out of a deadlock that has a way round: an earlier version that gave way first made corridor 2 with 6 each way worse (max 772 → 791 to 838).
- *Who gives way*: the unit with the right of way is the deadlock's unit with the lowest object id that is not giving way. The units in play are the deadlock's and the house's moving units (a move to a cell, not giving way) within 12 cells (`YIELD_RADIUS`, part 4's `NEAR_RADIUS`; Chebyshev) of any of the deadlock's cells. Those heading against the right-of-way unit (a negative dot product of the headings) give way; the others keep going.
- *Where to*: the lane is the cells of the units that keep going and their fresh paths (S-0212's search). Each giving-way unit gets the nearest free cell off the lane by a breadth-first search from its own cell. If no unit gets a cell, nothing changes.
- *Resume*: `path::yield_turn`, each turn before movement. A giving-way unit resumes its order (a fresh computation) when every unit it gives way to has no destination, is off the map or has passed it (the vector from it to that unit points along that unit's heading), or after 900 ticks (`YIELD_WAIT`). Any order for the unit ends the giving way (`orders::apply_pending`).

**State.** `PathState::yielding: Option<Yield { resume, to, since }>`, saved and hashed: save format 85 → 86, AI game format 47 → 48. The trace gains `GaveWay` and `Resumed` (the pathing report prints them).

**Effect.** Two rooms, 6 each way: 33 missing → 0, median 715, max 838, 18 reversals, 6 831 computations → 960. Corridor 2, 6 each way keeps part 4's 585 / 772 (the detour finds a way there, so no unit is shut in). Every other row, reference and layer, is unchanged.

**Tests.** `yield_off_clears_a_deadlock_with_no_way_round` (the detour alone: units missing; with the yield none missing, no give-up, none unsettled), `yield_off_leaves_queues_and_the_reference_alone` (alone, without the graph, nothing changes; with the graph, the infantry and 20-vehicle queues and corridor 2 with 3 and with 6 each way play the same with and without it), `a_unit_giving_way_survives_save_and_load`, `the_giving_way_state_is_hashed`.

### Tick cost

Part 4's two-rooms deadlock cost 107 µs a tick against the reference's 95. A profile (temporary, removed) put about 20 % of the extra in building the wait-for graph and about 80 % in the detour search that fails at its 4 096 expansions. Two exact changes to `stalled_ring`: a unit with no stalled unit in its wait cells has an empty ring without the graph being built, and the graph is peeled in one pass by counts. Every scenario row prints the same before and after. Skipping the failing detour after a fill was tried and dropped: it was slower (the corridor's detour time rose from 4 ms to 31 ms).

`cost` (ignored; `ROUNDS=<n>`) prints the fastest ns per tick of each scenario for the reference, the layer, the layer without the yield and the layer with stalled blockers on. With the layer, most rows are within 0.5 % of the reference. Three rows are higher, all where the behaviour differs: by buildings +1.2 %, inside buildings +3 % and the unreachable cell +8 % (more computations). The column with stalled blockers on shows the same three rises (+1.4 %, +2.9 %, +7.4 %), so they come from part 4's nearest reachable cell and group spread, not from the stalled-blocker check; the brief's bar for that check (no higher than the reference in every scenario) holds. Corridor 2, 6 each way costs 70 000 against 105 600; two rooms, 6 each way 63 000 against 95 000. The layer without the yield in two rooms is 107 000 (+12 %, the failing detour every computation).

### Field cells (`harvest_cell_shared` off)

**Problem.** Harvesters sent from one place pick the same ore cells by the ring search (S-0402 claim 4) and block each other one step short of them. In the harvester scenario all 12 harvesters (4 a seed) share their first destination, with 12 give-ups and 29 jostles.

**Design.** In `econ::harvest::ring_search` a cell that another on-map unit of the harvester's house has as its cell destination is not acceptable. It takes no draw.

**Effect.** Harvesters: give-ups 12 → 6, jostles 29 → 22, failed computations 5 → 1, shared 12 → 0, median 91 → 77, max 203 → 80; credits 4 125 as before.

**The remaining 6 give-ups** are S-0402 claim 9's docking queue, not changed. The synthetic refinery has no queueing-cell shift, so its queueing cell is its top-left cell, the dock. A returning harvester drives to the dock another harvester holds and gives up within the close-enough distance (S-0207 F7). With a refinery whose queueing cell is off the dock it would wait there instead. The jostles were not analysed cell by cell; a harvester turning round on its trips counts as one.

**Test.** `field_cells_off_gives_each_harvester_its_own_cell` (on: shared and give-ups; off: none shared, fewer give-ups, no fewer credits).

### Left open

- **Shore maximum** (176 → 217, part 4's group spread): the tradeoff of the spread, the last members walk round to cells of their own. Not changed.
- **The harvesters' dock** (above).
- **The yield's choice of side** is by object id, not by group or order time; with more than two groups at one passage it has not been tried.

### The table with the layer (seeds 7 to 9 pooled), part 5

Only three rows change against part 4's layer; the reference rows are all unchanged.

| Scenario | Part 4's layer: median / max, missing, give-ups, shared, searches | Part 5's layer |
|---|---|---|
| factory stream | 247 / 249, 0, 12, 24, 39 | 247 / 283, 0, 0, 0, 27 |
| harvesters | 91 / 203, 0, 12, 12, 318 | 77 / 80, 0, 6, 0, 282 |
| two rooms, 6 each way | 287, 33, 0, 0, 6 831 | 715 / 838, 0, 0, 0, 960 |

### Results, part 5 (local lane, outputs in the git-ignored `local-lane/m8/c7-pathing5/`)

Two sets of binaries: *base*, origin/main fbae0ce (A6, which changes no simulation code), exported into the lane folder, and *branch*, this branch rebased onto it.

- *Default runs, byte-identical.* The oracle arena (`arena --record all`): the three `ours-M-000*.csv` and the three oracle reports are byte-identical between base and branch. The reference M6c, M7a and M7b sweeps 1 to 16 print the same lines. The sim-gate goldens (`cargo test --release -p glasnost-sim-gate -- --include-ignored`, without update) pass on the branch and leave `crates/sim-gate` unchanged. The reference quick lane: all 48 games end on the same tick with the same result on both (47 won, house 1 27 and house 0 20, and sides 1 and 2 on map 0 seed 10 not over at the 120 000-tick bound, as before; median 44 512).
- *Layer-on sweeps 1 to 16* (`GLASNOST_PLAY_FIXES=1`): all three pass on both. M6c smallest minimum 1 (base 1); M7a smallest minimum 1 (base 1), no seed ends early; M7b smallest drain minimum 2 (base 3: seed 2's mutator launches are 2, a different game with the layer).
- *Quick lane* (maps 8, 2 and 0, Yuri as house 1 against Allied and against Soviet, `GLASNOST_AI_QUICK=1`). Seeds 7 to 14 (48 games) as in part 4, and, because the layer's median came out longer, seeds 15 to 22 (48 more) as well; medians in ticks, house 1 and house 0 wins:

  | Run | Seeds 7 to 14 | Seeds 15 to 22 | All 96 (median; mean) |
  |---|---|---|---|
  | reference (the same on base and branch for 7 to 14; 15 to 22 on the branch; the game not over counted at the bound) | 44 512; 27, 20, one not over at the bound | 42 738; 24, 24 | 43 334; 47 112 |
  | part 4's layer (base) | 43 158; 30, 18 | 43 284; 26, 22 | 43 158; 46 062 |
  | part 5's layer (branch) | 45 197; 35, 13 | 45 370; 27, 21 | 45 197; 48 351 |
  | part 5's layer without field cells | 44 525; 39, 9 | 42 423; 29, 19 | 43 304; 45 659 |
  | part 5's layer without rally spread | 42 535; 32, 16 | 43 027; 26, 22 | 42 691; 44 908 |

  With the layer every game is over with a winner and none panics, in every row. Part 4's bar, "the median length is no worse" than the reference's, is **not met** on a plain reading: part 5's layer is 4.3 % longer over the 96 games (45 197 against 43 334) and 4.7 % longer than part 4's layer. The two variants (temporary builds with one switch's line removed from the layer, not committed) do not point at one switch: without either field cells or rally spread the median is at or below the reference's, and the five rows spread over 2 500 ticks while single games range from about 21 000 to 104 000 ticks. Nothing in the long games looks stuck: the two longest layer games (sides 0 and 2 map 8 seed 7, 103 156 ticks, and sides 1 and 2 map 0 seed 8, 101 912) end with a winner and their harvesters' ticks by behaviour have the same proportions as on base, and the layer's harvest income per 1 000 ticks (median over seeds 7 to 14) rises from 1 231 and 1 197 (houses 0 and 1, part 4's layer) to 1 246 and 1 298. Left for the lead: whether the longer median is a reason to hold back one of the two switches.
- *Tick cost.* In the scenario set, see *Tick cost* above. In the quick lane, mean of the games' µs per tick: reference 1 712 (base) and 1 716 (branch); layer 1 747 (base) and 1 781 (branch). The layer's games differ between the two, so this is no like-for-like measurement.
- `cargo test --release -p glasnost-ai -- --include-ignored` on the branch: 239 passed, none failed. `cargo test --workspace`: 6 817 passed, none failed (113 ignored). fmt and clippy (`-D warnings`) are clean. `cargo xtask check`: deps, determinism, cite and canary pass; stock-guard reports 16 lines in the measurement records M-0013 and M-0015 (local values shown in a Result table), which are main's as of fbae0ce and not touched here.

**Version effects.** Save format 86 and AI game format 48 (the yield's state): saves and AI games of earlier formats are refused. The layered `sim_rules_hash` changes again (the layer file's content is in it). The sim-gate goldens were regenerated for the format only: the nine files keep their line counts, and only the hashes of the saved state move (each object's path state gains one byte, `None` in every golden game since the reference never gives way). Wire 9, pathing report 2 and desync bundle 3 are unchanged.

## Part 6

Brief: `.claude/worktrees/M8-C7-PATHING6-BRIEF.md`. The same rules as parts 4 and 5: each change is our design, marked `// design: C7-P6 <name>`, behind a quirk switch whose default (on) is the original's, set off by the play-fixes layer, which now admits twenty switches. No saved state is added, so the formats are main's (save 126, AI game 92). No request was needed (block Q-2941 to Q-2945 unused).

### A tempo diagnostic for the quick lane

`GLASNOST_AI_TEMPO=1` (`skirmish_lane::Tempo`, documented in [own-ai.md](../dev/own-ai.md), *Lane diagnostics*) prints a `tempo` line per house under each game: credits earned and army units on the map at ticks 10 000, 20 000 and 30 000; unit-ticks moving and stuck (a unit with a destination that stays in one cell for at least the blockage delay) for harvesters, the army and the army's first trip; release to first rest; give-ups, those of harvesters within 4 cells of an own refinery another object is in contact with (*at a busy dock*) apart; failed searches, searches and yields. It reads the movement trace, which is never hashed or saved, so a game is the same with it. `GLASNOST_AI_HASH` and `GLASNOST_AI_OBJS` (already there) find the tick at which two builds' games part.

### Item 1: part 5's longer games

Quick lane, seeds 7 to 14, maps 0, 2 and 8, Yuri (house 1) against Allied and against Soviet: 48 games, the same seeds with the layer off and on (main at be378c8, before part 6's switch). Medians over the house-games:

| | Reference | Layer (part 5) |
|---|---|---|
| Game length, median; mean (ticks) | 45 600; 52 467 | 47 533; 53 130 |
| Wins house 0, house 1, not over | 22, 26, 0 | 21, 25, 2 |
| Credits earned at 10 000 / 20 000 ticks | 36 347 / 62 733 | 36 551 / 65 445 |
| Army on the map at 10 000 / 20 000 / 30 000 | 13 / 13 / 15 | 13.5 / 13 / 13 |
| Earned per 1 000 ticks | 3 076 | 3 230 |
| Stuck per mille: harvesters, army, first trip | 235, 131, 221 | 212, 127, 236 |
| Give-ups: harvesters (at a busy dock), army | 7 289 (3 669), 24 518 | 523 (380), 11 611 |

Paired game by game, the layer's game is longer in 24 and shorter in 24, with a median length ratio of 1.003, and has the same winner in 26. With the layer, houses earn as fast or faster and field the same army through 20 000 ticks; their units are stuck no more and give up less than half as often. The first trip's stuck share differs by house and map without a pattern (house 1 on maps 0, 2 and 8: 538, 385 and 304 per mille in the reference, 248, 479 and 451 with the layer). The two games the layer does not finish are an economy stalemate (seed 9, sides 1 and 2, map 2: house 1 has no refinery, 423 credits and sells out) and an army give-up loop of house 0 (seed 13, sides 0 and 2, map 2: 6 380 army give-ups, 373 per mille stuck), a kind the reference has too (seed 13, sides 0 and 2, map 8: 18 640 give-ups). **Decision: no switch is sluggish.** The longer median is the games' divergence: a different fight from the first few thousand ticks on, with no lost tempo in income, army or movement. No switch is changed for it (m8-plan.md, *Decisions of 2026-11-10*).

### Item 2: harvesters at a busy dock

With the layer, 380 harvester give-ups at a busy dock in the 48 games against 3 669 in the reference, which has them in 9 house-games (1 757, 1 096, 450, 244, 113 and fewer). With the layer, 372 of the 380 are in one house-game (seed 11, sides 1 and 2, map 8, house 0, which also runs captured buildings); the other four house-games have 1, 1, 2 and 4. A temporary print (not committed; `logs/diag-dock-print.txt`, `logs/diag-dock-watch2.txt`) shows the 372 are not a queue: they come from two harvesters, one after the other, that never move from one cell (54 give-ups of the first from tick 36 289 to 39 812, then 318 of a later one on the same cell from 40 412 to 41 408). Each search to the dock (and to the ore) fails, the move is given up, harvest falls to guard and starts again a tick or a few ticks later; a refinery within four cells with another harvester in contact makes the count read it as a busy dock. The cell is a pocket the search cannot leave (it looks like a factory exit closed by the house's own buildings; not confirmed). That give-up is the right answer for a unit with no path; the only cost is the searches it repeats. **No pathing fix** is made: the layer already cuts these give-ups by 90 %, and S-0402's queue rules are not ours to change.

The new scenario *harvesters, queueing cell off the dock* (`pathing_common::harvesters` with a refinery whose queueing cell is the cell left of its dock; commit "sim: pathing scenarios for a busy dock and 1-wide passages") has 4 harvesters a seed queue there. Reference: 21 give-ups (2 open, 19 late), 12 shared, maximum 203. Layer: 16 give-ups (none open, 16 late), none shared, maximum 80; credits 4 625 in both. The layer's give-ups are on the cells round the queueing cell (the `diag` trace), harvesters arriving at the queue while another waits on it, not on the way to the dock.

### Item 3: yield with more than two groups (`jammed_move_dropped` off, and the yield's crossing)

**Problem.** In a passage one cell wide, two groups meeting head-on never reach part 5's yield. The units read each other as blocked, the search fails, and S-0212 claim 23 (row 2) gives the move up: the units stand in the passage for good with no destination, so they are never *stalled* (part 4's wait-for graph needs a destination) and no deadlock forms. Three new scenarios show it: three ways (three groups of 3 meeting at a crossing of three 1-wide passages), narrow rooms (two rooms joined by a 1-wide passage, 3 each way) and the same with one unit parked in a room. In both positions of part 5's switches, none of the units arrives.

**Design (hold).** With `jammed_move_dropped` off, a failed search that would give the move up (`path::search_gives_up`, claim 23 row 2, shared with `path::failed`) is held instead when `path::jammed` says only moving units stand in the way: part 5's two-sided fill (`path::shut`) from the unit to its destination cell meets, with buildings, terrain objects, standing units (no destination) and wall overlays as walls and every unit with a destination passable, and the destination cell itself holds no standing or blocked unit (two units that would swap cells). A held unit keeps its destination, its steps are dropped, it counts as blocked from now if it was not, and it searches again after the retry delay. The C7-P4 nearest reachable cell is skipped for a held move. A unit is held while blocked for less than 300 ticks (`HOLD_FIRST`), and after that, up to 900 ticks (`HOLD_WAIT`, part 5's `YIELD_WAIT`), only while a unit within `YIELD_RADIUS` of it gives way. In the wait-for graph (`stalled_ring`) a blocked unit on a move with no stored path waits on every neighbour of its cell, as the computing unit does, so the held units form the deadlock and part 5's yield settles it.

**Design (three ways, under `deadlock_no_yield`).** Part 5 counted a unit at right angles to the right-of-way unit (a dot product of 0) as going its way, so at a crossing two of three groups never gave way to each other. `path::goes_with` now needs a positive dot product: a unit of the deadlock at right angles crosses the way and gives way; a unit near the deadlock but not in it gives way only when heading against (a negative dot product), so one crossing the way elsewhere is not stopped.

**Narrowing in the quick lane.** The first hold (held while only units stood in the way, every unit passable, up to 900 ticks) solved the scenarios but lengthened the quick lane against part 5's layer (14 games longer, 5 shorter; one game from 50 046 to 110 406 ticks with 365 yields). The game-parting tick of each longer game (`GLASNOST_AI_HASH`, `GLASNOST_AI_OBJS`) showed four causes, each narrowed without changing a scenario row:

1. *A held unit with a standing crowd in the way*: standing units are now walls in the fill.
2. *Two boats swapping cells at a gathering point* (each the other's destination, in open water where the yield never runs): a destination cell held by a standing or blocked unit is never held.
3. *A unit next to its destination*: the computation's hold skipped the nearest reachable cell although `failed` would not give the move up (the destination within one cell), and a driving unit then lost its destination. Now only a failure that gives up is held.
4. *A naval unit held in a crowded harbour* for the full 900 ticks: the hold now ends after 300 ticks unless a unit near it gives way.

A narrowing that also took blocked movers as walls (hold only when they shut the unit in) was tried and dropped: the hold must start before the group mates count as blocked, and three ways and narrow rooms went back to every unit missing.

**Effect.** Every other row, reference and layer, is unchanged; the three new rows with the layer:

| Scenario (layer) | Before: median / max, missing, give-ups, reversals, searches, failed | After |
|---|---|---|
| three ways, 3 each | -, 27 of 27, 11 381, 0, 26 091, 30 | 457 / 758, 0, 0, 33, 345, 36 |
| narrow rooms, 3 each way | -, 18 of 18, 45, 0, 147, 21 | 592 / 642, 0, 0, 18, 201, 30 |
| narrow rooms, 3 each way, one parked | -, 18 of 18, 36, 0, 156, 30 | 758 / 808, 0, 15 (none open or late), 15, 462, 24 |

The yield is deterministic (object-id order, ordered containers, no draw) and bounded: each hold ends after at most 900 ticks blocked, each yield after `YIELD_WAIT`, the fills after `YIELD_FLOOD` cells. Three groups pass in turn: the lowest-id unit's group keeps going and the two others give way, then the next deadlock settles the rest.

**Tests** (`pathing_c7p4.rs`, both positions): `hold_off_clears_passages_one_cell_wide` (on: none arrives, open give-ups; off: every unit arrives in the three scenarios, no give-up, none unsettled), `hold_off_alone_gives_up_after_the_first_hold` (the hold without the yield: held, then given up within `HOLD_FIRST`), `hold_off_leaves_unreachable_moves_alone` (a move into a ring of buildings or onto rock is given up or redirected as before).

**Tick cost** (`cost`, ns per tick, fastest of 3 rounds, full table in `logs/cost-final.txt`): the hold costs ticks only where it runs. Against the layer without the hold (`jammed_move_dropped` on): three ways 42 884 against 69 666 (the reference 73 093: the passages clear, so the searches stop); narrow rooms 28 915 against 26 934 (reference 27 160); narrow rooms with one parked 34 177 against 31 596 (reference 31 995), the held units' wait-for fills. Every other row is within 1.5 % of the layer without the hold (gap 1 63 947 against 63 719, by buildings 128 564 against 128 860, unreachable cell 38 423 against 38 392); the unreachable cell's distance from the reference (35 274) is part 4's `reachable_near`, not the hold.

### Item 4: shore maximum

Not taken: the shore's maximum (176 → 217 with the layer) is the group spread's tradeoff (the last members walk round to cells of their own), and nothing cheap changes it without giving back the spread.

### Results, part 6 (local lane, outputs in the git-ignored `local-lane/m8/c7-pathing6/`)

Base: origin/main 91e0a5b (with A6b), built in this worktree detached; branch: this branch rebased onto it. Fresh layer-off baselines were taken on the rebased base, so A6b's own arena moves are in both.

- *Default runs, byte-identical.* The oracle arena (`arena --record all --threads 2`): all nine files (`ours-M-0001` to `0003.csv`, the oracle reports M-0001 to M-0003 and M-0019 to M-0021) are byte-identical between base and branch (M-0019 0 of 6 rows failing, M-0020 1 of 25, M-0021 11 of 50 on both). The M6c, M7a and M7b sweeps 1 to 16 print the same lines on base and branch (only the run times differ) and pass (smallest minimum 0, 1 and drain minimum 3). The sim-gate goldens pass with `--include-ignored` and leave the tree unchanged.
- *Layer-on sweeps 1 to 16* (`GLASNOST_PLAY_FIXES=1`): all three pass and print the same lines on base and branch (smallest minimum 0 for M6c, 2 for M7a, drain minimum 3 for M7b; vehicles seized min 27 median 173, crates picked up min 10 median 20): the hold changes none of the sweeps' printed figures.
- *Quick lane, layer on* (seeds 7 to 14, maps 0, 2 and 8, sides 0 and 2 and sides 1 and 2, `GLASNOST_AI_QUICK=1 GLASNOST_AI_TEMPO=1`, 48 games), branch against part 5's layer (be378c8; A6b does not reach the AI houses): 47 games are identical tick for tick and one ends 90 ticks sooner (seed 7, sides 1 and 2, map 0: 50 046 → 49 956), all 48 with the same winner. Median 47 533 and mean 53 129 against 47 533 and 53 130; army give-ups 11 582 against 11 611; yields 2 against 5. The hold leaves real games almost alone and clears the passages the scenarios build.
- *Quick lane, layer off* (the same 48 games, branch against the rebased base): every game ends at the same tick with the same winner and prints the same tempo lines (income, unit-ticks, give-ups, searches); median 45 600 on both.

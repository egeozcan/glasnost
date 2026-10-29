# S-0212 r87: claim ledger, M8 step A3 intake 44

This intake takes in one spec revision:

- **S-0212 revision 87** (spec commit 3c4ef1e). Our engine built revision 83 at intake 37 (its
  claim 24 row in [intake34-S-0212.md](intake34-S-0212.md)); the diff runs from revision 83
  (5d1e9f5) to 87.

No request is answered by r87 (its `requests` are unchanged). It goes with S-0417 r68
(intake 43, [intake43-S-0417.md](intake43-S-0417.md)), whose claim 17 *No path* relies on it.
The specs it names were read as they stand: S-0209 claims 7 to 10 and C19, S-0214 claim 11,
S-0215 (the *move* behaviour without a destination), S-0503 claim 16 and H14.

## What r87 changes

- Claim 21: a new destination does not cut a running retry delay short for the walk locomotor;
  the first search for it waits for what remains of *R*. Backing: `unmeasured` (S-0417 U10 g
  run 2 would test it).
- State table, row *destination changes*: the same rule for the other locomotors, at low
  confidence (`unmeasured`, a new Open question).
- Claim 23, gate row: a unit with the driving, ship **or walk** locomotor has its destination
  cleared. For the walk locomotor this holds after a fresh search and after a re-path for a
  moving blocker, and its movement in progress ends in the same movement update.
- Claim 23, after the table: for the walk locomotor, the stop of rows 2 and 3 ends its movement
  in progress whether or not it keeps its destination.
- Backing and confidence: P7 run 7b for the walker's fresh search in the gate row; the re-path
  case `unmeasured`, medium; the end of movement in progress in rows 2 and 3 `unmeasured`, low.
- Scenario P7 run 7b (new): an infantry unit sent onto an island by a team script; no step, its
  destination cleared and its movement in progress ended in its first movement update, it stays
  in its team and nothing is queued.

## Code

- `path::failed`, gate branch: a walker's destination is cleared after a re-path too (`drive ||
  walker`; intake 43 had built the fresh search). A walker stands between steps when it
  re-paths, with no walking target, so clearing the destination and the stored path ends its
  movement in progress.
- Rows 2 and 3: intake 43's `PathState::failed_for` already ends a walker's movement in progress
  at a failed search that keeps the destination; row 2 clears it.
- The retry delay is no longer reset when a destination is given: `orders` (every order's
  destination), `transport::settle` (a transport's shore destination, S-0507 claim 8), `structures::absorb` (three sites),
  `econ::slaves` (three), `depot` (three), `econ::harvest` (`drive_to`, `drive_to_none`),
  `structures::bunker` (four) and `structures::grinder`. Each site now cites claim 21 and the
  state table. These resets carried no cite of their own. The hover's reset when its
  order-level destination is cleared stays: S-0503 claim 16 says so, and H14 expects a retarget
  during a running delay to wait for it, as here.
- No new saved or hashed state: save format 84 and AI game format 46 (intake 43) stay.

**Our engine has no team members** (as `path` and `scatter` note at S-0212 claim 14 and S-0208
claim 11), so "it stays in its team" holds by construction.

**Play-fixes layer (C7).** Not named. A unit given a new order while its retry delay runs now
waits up to *R* before it searches. That is a delay, not a unit stuck, and the original does it.
The layer's admission rule (`data/layers/play-fixes.jsonc`) is not met.

## Claims

| Claim or item | Status | Notes |
|---|---|---|
| 21 (a new destination waits for what remains of *R*, walk locomotor) | tested | `claim_21_walker_new_destination_waits_for_the_delay` (an order after a failed gate: no step, movement in progress, the first search at the failure's tick + 90) |
| State table, *destination changes* (the other locomotors, low) | tested | `state_table_drive_new_destination_waits_for_the_delay` (the driving locomotor); the hover by S-0503 H14 as built |
| 23 gate row, walk locomotor, fresh search | tested | `p7_run_7b_walker_to_an_island` (P7 run 7b: no step, the destination cleared and movement in progress ended in the first movement update, nothing queued, *R* from that tick); S-0417 r68's `claim_17_gate_failure_clears_then_accepts` |
| 23 gate row, walk locomotor, re-path | tested | `claim_23_walker_repath_gate_failure_clears` (a standing walker with a stored path) |
| 23, after the table (rows 2 and 3 end a walker's movement in progress) | tested | `claim_23_walker_search_failure_ends_movement` (row 2 cleared, row 3 kept; both not in progress); r68's `claim_17_failed_search_kept_destination_accepts` |
| 23, "stays in its team" | by construction | no teams in our engine |
| Backing, confidence, Open questions | text | |
| P7 run 7b | not ours | Oracle scenario. |

Tests changed by the rule:
- `claims_s0314_stranded` (all four): the rules set the path-retry delay to 0, since a move order
  given while the hover's delay runs now waits out the rest of it and these tests are about
  S-0314's settling (two of them failed: the hover came down on the depot's cells).

## Before and after

- Goldens (`glasnost-sim-gate`): every scenario moves (first moved line: scripted 152, combat
  122, skirmish 476, duel, duel-diagonal and duel-naval 469, duel-m6c 16, duel-m7a 249, duel-m7b
  590). The first moves are in the hash alone: a destination given after an earlier delay had
  run out used to set the stored retry tick to 0, and now leaves it, which the state hash sees
  with no change in play. The changes in play were traced with a probe at every removed reset:
  50 times in the nine scenarios a destination arrived while a delay still ran, 38 of them an
  order's destination (`orders`) and 12 a harvester's (`econ::harvest::drive_to`), the
  earliest at tick 300; each now waits out the rest of the delay (1 to 14 ticks) where the
  old reset let its search run at once. The walker re-path case of the gate row never arises in the gate's
  scenarios. Lengths change with the play: skirmish (near tick 24 800) and duel end earlier
  than before, and duel-diagonal runs longer. The same first lines move after the rebase onto
  M8 B9 (e40b1f9), where the goldens were regenerated again.
- Sweeps (seeds 1 to 16, ref and fix; after intake 43 to after intake 44): M7b moves one counter
  (mutator launches max 8 to 7). M7a moves 20 of 26 by a few (for example takeovers 5/23/37 to
  5/21/36, gas hits 463/1147/1918 to 528/1342/1918, minimum/median/maximum); its seeds with a
  counter at 0 go from 3 to 2 (seeds 7 and 13, hostile intent seen by a
  sensor, as before; seed 9's infantry absorbed is now above 0). M6c moves 23 of 29 (for
  example deploy orders 77/98/156 to 55/100/162, crate orders 55/126/223 to 95/117/280, erasures
  begun 4/8/13 to 5/11/22); seed 10 now has no radiation exit (radiation exits 13/47/70 to
  0/34/70; it was 0 in some seed before, see *Margins* in `docs/development.md`), every other
  seed keeps every counter at 4 or more. No run ends early. The gate's own seeds pass their exercise tests
  (`every_m6c_ability_is_exercised_and_the_replay_reproduces_the_game`, the M7a one and
  `the_drain_is_exercised_and_the_replay_reproduces_the_game`, release): M6c seed 1 keeps every
  counter at 4 or more, M7b seed 11 has none of its drain counters below 3, and M7a seed 17's
  psychic sensor orders fall from 2 to 1 (the known weak counter; every other counter is 3 or
  more). The play-fixes sweeps move as the reference ones. Tables in
  `local-lane/m8/a3-43/` (after43, after44); the sweeps and the arena ran on the base before
  B9 (5c1d621).
- Arena (`--record all`): the three CSVs (M-0001, M-0002, M-0003) are byte-identical to intake
  43's; failing pairings stay 30 of 96, 12 of 24 and 2 of 35. An arena unit gets one order, with
  no running delay to cut.

# S-0215 claim 28: capture and sabotage processing (M8 seam)

Brief: `.claude/worktrees/M8-SEAM-CAPTURE-BRIEF.md`. Spec: S-0215 r91, claim 28 (*Capture and
sabotage*), with claims 4, 5, 6, 7, 14, 15 and 19, table T1, S-0208 claims 11 and 13, S-0207
claim 16 and S-0501 claim 18.

Intake 66 found that *capture* and *sabotage* were not dispatched in `process::process`: they fell
through to the "not implemented yet" note with a 450-tick delay. *Eaten* already had claim 28's
table in `structures/grinder.rs`. No ledger row recorded claim 28 as built elsewhere. An engineer's
capture on arrival is separate (S-0509, `structures/entry.rs`) and is unchanged.

**Result.** Claim 28 is built once, in `process::process_capture`. *Capture* and *sabotage* use it
for infantry, vehicles and aircraft (claim 5, T1), and *eaten* calls the same function with the
narrower building-taker test. A building in any of the three has no processing rule (claim 4).
There is no quirk switch. No hashed state was added, so there is no format bump (save 92, AI game
54). Requests filed: **Q-2251** (an aircraft and the scatter off a building) and **Q-2252** (part
4).

## Claim 28, row by row

All tests are in `crates/deterministic/sim/tests/claims_s0215_capture.rs` unless named otherwise.

| # | Row or rule | Status | Code | Test |
|---|---|---|---|---|
| 1 | No destination; the target is a building and the actor is a building taker (charges, engineer, occupier, assaulter, infiltrator): the target becomes the destination | built | `process::process_capture`, `process::building_taker` | `claim_28_row_1_a_building_taker_takes_its_target_building_as_destination` |
| 2 | No destination, any other case: fall back to rest (claims 14, 15) | built | `process_capture` calls `fall_back` | `claim_28_row_2_a_non_taker_with_a_target_building_falls_back`, `claim_28_row_2_infantry_with_nothing_falls_back_and_stays`, `claim_28_row_2_an_unarmed_vehicle_queues_guard_and_eaten_does_not` |
| 2a | Row 2's scatter off a building: no threat point, forced, not move-anywhere; the cell after the fall-back is tested | built for infantry and vehicles | `scatter::request` from `process_capture` | `claim_28_scatter_off_a_building_infantry` (S-0208 claims 11, 13), `claim_28_scatter_off_a_building_vehicle_takes_no_draw` (S-0207 claim 16, no draw) |
| 2b | Row 2 for an aircraft: fall back to rest by S-0501 claim 18; the scatter request is sent | built; the scatter's effect is open | `air::process` calls `process_capture` | `claim_28_an_aircraft_in_capture_falls_back_with_the_standard_delay`. S-0215 and S-0501 do not say how an aircraft answers the request: **Q-2251**, interim no effect. |
| 3 | Destination and target: no change | built | `process_capture` | `claim_28_rows_3_and_4_a_destination_changes_nothing` |
| 4 | Destination, no target: the hunt rule, which never applies in our engine | built (no effect) | `process_capture` | `claim_28_rows_3_and_4_a_destination_changes_nothing`, `claim_28_a_computer_engineer_without_a_target_does_not_hunt` |
| 5 | The hunt rule (original only) | not built, by the claim's "In our engine" | — | `claim_28_a_computer_engineer_without_a_target_does_not_hunt`. This is a stated divergence of the claim, not a quirk switch. |
| 6 | The delay is always the standard delay (claim 6, the current behaviour's rate) | built; was 450 | `process_capture` returns `standard_delay` | `claim_28_the_delay_is_always_the_standard_delay` |
| 7 | *Sabotage* is processed as *capture* (claim 5, T1), with its own rate | built | dispatch in `process::process` and `air::process` | `claim_5_sabotage_is_processed_as_capture_with_its_own_rate` |
| 8 | *Eaten*: the same table; occupiers, assaulters and infiltrators are not building takers; no hunt rule; the same scatter | built (moved onto the shared function) | `grinder::process_eaten` keeps S-0603 claim 15's clear, then calls `process_capture(.., true)` | `claim_28_eaten_counts_fewer_building_takers`, `claim_28_row_2_an_unarmed_vehicle_queues_guard_and_eaten_does_not`; the existing grinder tests still pass |
| 9 | A building in *capture*, *sabotage* or *eaten*: no processing rule (claim 4, T1) | built | dispatch returns `NO_RULE_DELAY` | `claim_4_a_building_in_capture_has_no_processing_rule` |
| 10 | Building-arrival effects are separate (S-0509, S-0510, S-0508, S-0519) | unchanged | `structures/entry.rs` | existing tests |

## Part 4: the pad-bound route aircraft after P2

| Item | Status | Finding |
|---|---|---|
| S-0213 Scenario O8 x (6), the pad-bound game, after P2 | request **Q-2252**; no code change | After the route's last point, S-0213 claim 16d hands over to S-0501 claim 6. The aircraft has no contact with a building in its cell, so it falls back to rest. The test aircraft is unarmed, so T2's only row is "in flight, otherwise": the F8 landing spot next to the pad, with *move*. There it falls back again at every tick and hovers. The engine follows the rules as written; the scenario only says "Record what A does after P₂". The pad-search rows of T2 are for armed aircraft only. Pinned by `claim_16d_after_p2_an_unarmed_pad_bound_aircraft_falls_back_above_its_f8_spot` (`claims_s0213_r89.rs`). |

## Goldens

Only `duel-m6c` moved, in all 1200 lines. Its engineers and charge carriers take the standard
delay's draw from the first ten ticks on. The other eight goldens are unchanged.

## Lanes

The "before" numbers are intake 66's "after" (`local-lane/m8/intake66/after/`), which is the same
code as this branch's base (`git diff 797614e^2 797614e -- crates` is empty). "After" is
`local-lane/m8/seam-capture/after/`.

| Lane | Before | After |
|---|---|---|
| M6c sweep 1–16 | passes; smallest minimum 2 (air-strike orders) | passes; smallest minimum 4. Counters move (engineer orders min 6 → 5, garrison orders min 22 → 13, bombs planted max 24 → 72). |
| M7a sweep 1–16 | passes; smallest minimum 0 | identical |
| M7b sweep 1–16 | passes; smallest drain minimum 3 | identical |
| Arena (`--record all`) | 11 of 155 fail (M-0001 9/96, M-0002 1/24, M-0003 1/35) | identical; all three `ours-*.csv` byte for byte |
| `glasnost-ai --include-ignored` | all pass | all pass |
| Quick lane (seeds 7–10, maps 0, 2, 8; 36 games) | all games end; mean end tick 50,278; house 0 wins 19 | all games end; mean end tick 42,559; house 0 wins 15; 18 winners changed, 24 games end earlier |

### Capture and sabotage at the end of the quick-lane games

The army-end line counts the winner's units on the map. Each row below counts entries (one entry
covers ×n units).

| Entry | Before | After |
|---|---|---|
| *capture*, no destination, no target | 19 (25 units) | 1 |
| *capture*, object destination, no target | 8 (9 units) | 18 (25 units) |
| *capture*, object destination, object target | 0 | 1 |
| *sabotage*, no destination, no target | 15 (18 units) | 0 |
| *sabotage*, no destination, object target | 1 | 0 |
| *sabotage*, object destination, no target | 1 | 6 |

Row 2 releases every unit that had no destination: it falls back to rest (claim 14's guard or area
guard). What is left was checked with object dumps at N−301 and N−1 for seeds 9 and 10, and at
earlier ticks for seed 7 (`local-lane/m8/seam-capture/after/quick-dump/`):

- **Object destination, no target (row 4).** These are engineers sent to capture, spies sent to
  infiltrate and occupiers sent to garrison, all in *capture*, and charge carriers sent to sabotage
  in *sabotage*. All of them stand on a different cell at N−1 than at N−301, so they are on their
  way. Row 4 leaves them as they are, because the hunt rule does not apply in our engine. There are
  more of them than before: units that used to stay in *capture* with nothing to do now fall back
  and are idle, so the AI's garrison, capture and sabotage steps send them out again. Seed 8 was
  not dumped. Its entries are of the same kind (5 *capture*, 2 *sabotage*, each ×1).
- **No destination, no target (seed 9, the game that ends at 49,827).** An engineer whose
  destination was set at N−301 and cleared by N−1. Its next processing, which applies row 2, had
  not come yet when the game ended.
- **Object destination, object target (seed 10, the game that ends at 37,298).** An engineer in
  *capture* whose destination is a neutral tech building and whose target is another object. It
  has not moved from its cell since tick 31,994 or earlier, more than 5,300 ticks. Row 3 gives
  "no change", so claim 28 leaves it as it is. Something else keeps it still. The watch run that
  was meant to show its path and contact was stopped by the system for low memory and was not
  repeated, so the cause is open. It is the only stuck capture unit left in the 36 games.

Units inside a building or transport (on the map: no) can also be in *capture* with nothing; the
dumps show garrison occupants like that. The army-end line counts only units on the map, so they
are not in the table. This seam did not check whether claim 28 should process them.

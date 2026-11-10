# M8 seam sweep 7b: rows marked partly, untested or seam in intakes 117 to 122

Brief: `.claude/worktrees/M8-SEAMS-7-BRIEF.md`, part 7b. Method and row form follow
[seams-6.md](seams-6.md). Rows read: every row of `intake117.md` to `intake122.md`, and the
*Seams* text of the same steps' rows in `step-ledger-m8.md`, marked partly, tested in part, not
tested, untested, by construction, by inspection, built with no test, not built or seam. Each was
first checked for a later close (a later intake, a spec revision, its `requests:` field,
`git log --grep`). The step ledger's *Seams* text for these steps names only rows of the intake
ledgers, so it adds no row. Rows that only name the switch `aircraft_descent_touchdown_untested`
are not seam rows. No row here is the same claim as a 7a row (intakes 112 to 116, the
scatter-damage, air-retaliate and stalled-harvester rows).

**Counts.** 16 rows read: 1 closed earlier, 0 blocked by an open request, 9 tested (none with a
code change), 6 left. A row restated in two intakes counts twice (intake 121's not-an-assignment
row, restated as intake 122's third row of claim 21). Intake 117 has no marker row.

**Result.** 9 rows tested through the real path, in 10 new tests. Every test agrees with the
code: no simulation change, no hashed or replayed state, no format bump (save 125, AI game 91,
wire 12), goldens and lanes unaffected. No spec request filed (Q-2921 to Q-2925 unused). No
oracle wishlist row added: the `unmeasured` rows tested here are ones the specs say no scenario
sets up, and the `measurable` ones are scenarios P25 and C5, which the wishlist already reaches
(rows 23 and 24) or which the spec defines in full.

## Built

None. No test showed the code differing from an approved claim.

## Tested

| # | Intake | Spec | Claim or row | State before | Test and result |
|---|---|---|---|---|---|
| 1 | 121 (35) | S-0209 r76 | claim 10: a failed search after which a walker keeps its destination (S-0212 claim 23 row 3) is not an assignment, so the blockage delay runs on | built, no test | `seams7b_a_failed_search_that_keeps_the_destination_leaves_the_blockage_delay_running` (`claims_s0209.rs`): the corridor walker heads for the cell its moving blocker holds; half a delay in, that cell's land gets a zero foot multiplier, the next ask answers blocked, claim 7's search fails (S-0212 claim 15) and the destination is kept. The wait still counts from the first answer: the avoiding search comes a delay after it and before a delay after the failure. Control `seams7b_the_control_an_assignment_after_the_failed_search_restarts_it`: the same destination assigned in that tick restarts it. Code agrees. |
| 2 | 122 (30) | S-0212 r98, S-0209 r76 | claim 21, third row: the blockage-delay half of "not an assignment" | tested (retry half); blockage half intake 121's seam | the tests of row 1. Code agrees. |
| 3 | 122 (35) | S-0212 r98 | P25 part e, runs 1 and 2: the "leaves its cell" void, checked only up to the order | tested in part | `seams7b_p25_e_run_1_real_failed_search_an_order_to_the_held_cell`, `seams7b_p25_e_run_2_real_failed_search_an_order_to_the_second_tile` (`claims_s0212_r97.rs`): part (a)'s passed-gate walker through the order and tick path (road land with a zero foot multiplier east of the walker and south of that), no failure made by hand. The search at *c* fails and keeps the destination with *R* from *c*; the order in *c* + 10 ends the delay; one more computation, which fails again and starts *R* from it; the walker stays in its cell from *c* to *c* + 100. Code agrees. |
| 4 | 122 (33) | S-0212 r98 | claim 21: a destination given as nothing to a walker that has none | built, no test | `seams7b_nothing_given_to_a_walker_with_no_destination_keeps_the_delay` (`claims_s0212_r97.rs`): the delay runs on, in the same tick and a later one. Code agrees. |
| 5 | 118 (30) | S-0501 r172 | claims 17, 34 and 45: a visiting aircraft of a missile type takes the *Missile types* rows | built, not tested on its own | `seams7b_a_visiting_flagged_type_in_flight_takes_the_ground_rows` (`claims_s0501_r167.rs`): in flight with ammunition left, the fall-back forgets target and destination and gives *retreat*; a later *retreat* processing takes a cell again. The unflagged control gets *hunt* with its target and destination kept. Code agrees. |
| 6 | 118 (33) | S-0501 r172, S-0506 | claim 35 and T4: a returning flagged slot with ammunition left and a controller target becomes attacking | built, not tested on its own | `seams7b_a_returning_flagged_non_rocket_slot_with_ammunition_becomes_attacking` (`claims_s0506_r40.rs`, HAWKF made a flying type): attacking, the controller target received, *attack* queued, not released; with no ammunition it does not become attacking. The row's claim 29 half (a *guard* processing below cruise height can find it a pad) is the path `ac35_a_…` already checks for a map-placed flagged type; the carrier helpers run no slot aircraft flight, so it is not set up for a slot one. Code agrees. |
| 7 | 119 (26) | S-0501 r176 | T2: armed, not pad-bound, no pad grants contact: none, then the F8 spot and *move* | built, no test | `seams7b_not_pad_bound_with_no_grant_gets_the_f8_spot_and_move` (`claims_s0501_r176.rs`): F8's spot as destination, *move*, no contact, no crash, no pad recorded; the moment's draws only over a refused cell. Code agrees. |
| 8 | 119 (30) | S-0501 r178 | claim 20: the removal makes no S-0011 raise, the tag stays, a planted bomb is removed silently | tested in part | `seams7b_the_removal_makes_no_raise_keeps_the_tag_and_drops_the_bomb` (`claims_s0501_r178.rs`, `testkit::entry_triggers`): no detonation and T unhurt in the pass's tick, the tag attached with its carrier count, no destroyed-by-anything firing; the control (not enrolled) keeps the bomb, and its death sets the bomb off and fires the trigger. Code agrees. |
| 9 | 119 (32) | S-0501 r178, S-0506 | claim 20: a kamikaze not of a rocket type left its slot, so its removal changes no slot | built (already), no test | `seams7b_a_released_flagged_non_rocket_objects_removal_changes_no_slot` (`claims_s0506_r40.rs`): released by a reset, removed, the controller unchanged; the control's slot object killed in its slot empties the slot. Code agrees. |

## Closed earlier (no work)

| Intake | Row | Closed by |
|---|---|---|
| 119 (22) | S-0501 r176 claim 24: the record rule on the ground | `claims_s0501_r103::claim_24_a_pad_bound_aircraft_on_the_ground_away_from_its_pad_queues_move` (the building recorded) and `claims_s0501_r110::claim_24_the_pad_bound_rule_records_a_cell_destination_as_no_pad`; r176 does not change the rule |

## Blocked by an open request

None. Of the open requests (Q-2097, Q-2597), none names a row here.

## Left (no test written)

| Rows | Intake | Spec | Claim or row | Why left |
|---|---|---|---|---|
| 1 | 120 (31) | S-0404 | claim 13, ground row: an exit point off the map | S-0404 Open question 4 |
| 1 | 120 (35) | S-0309 | claim 31: whether the unit left standing holds up the new unit's exit track | S-0309 Open questions; nothing is built for it |
| 1 | 120 (36) | S-0309 | claim 31: which unit a factory with more than one place sends out first | S-0309 Open questions |
| 1 | 121 (32) | S-0209 | claim 10: whether the enter order's destination is the transport or its cell | S-0209 Open questions (*The destination at an accepted enter order*) |
| 2 | 121 (33), 122 (31) | S-0209, S-0212 | claim 10 and claim 21 (r98) at the enter orders the Open questions leave open | S-0209 Open questions (*An enter order's assignment to a walker with no contact*, low, `unmeasured`); nothing is built from those low results |

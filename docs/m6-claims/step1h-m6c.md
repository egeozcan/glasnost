# M6c step 1h claim ledger (spec revisions that landed on main after step 1g)

This is part 8 of step 1. Each spec is compared with the revision the code had taken in. The old state is the spec file at df9154e, the merge of step 1g (`git diff df9154e f43b0b5 -- spec/<path>`). Every changed claim gets one class:
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *not built*: with the reason;
- *(c)*: editorial (front matter, scenario text, rewording).

Per-spec ledgers: [teleport.md](teleport.md) for S-0207 claim 20, [../m5-claims/S-0402.md](../m5-claims/S-0402.md), [../m5-claims/S-0417.md](../m5-claims/S-0417.md), [S-0502.md](S-0502.md) and [S-0514.md](S-0514.md). S-0207 (outside claim 20), S-0215 and S-0214 have no ledger of their own.

| Spec | Revision | Old state | Answers |
|---|---|---|---|
| S-0207 ground vehicle movement | 36 (from 34; 35 included, 71487a9) | df9154e | Q-1130 questions 1 and 2 |
| S-0402 harvest cycle | 28 (from 27) | df9154e | Q-1130 question 3 |
| S-0215 behaviours | 54 (from 53, 7aa2646) | df9154e | (with S-0207) |
| S-0502 movement of jumpjets | 11 (from 10, 5c8b6f3) | df9154e | (with S-0207) |
| S-0214 order targeting rules | 38 (from 37, d9a8197) | df9154e | (with S-0207) |
| S-0514 chrono weapons | 7 (from 6, f43b0b5) | df9154e | (with S-0207) |

Requests:
- **Q-1130**, answered by S-0402 and S-0207. Its interim rules are retired:
  - Suppression is now tested before claim 20 (a)'s disabling refusal. It keeps the destination in any state and is consumed. Under the interim rule, a refused destination left the suppression pending.
  - A teleporting chrono miner now drops its seeking destination (the seeking drop). Under the interim rule, nothing was dropped.
- No new request.

The spec commits on main were checked again before finishing: `git log main -- spec/` shows nothing newer than f43b0b5.

## S-0207 revisions 35 and 36 (answer Q-1130 questions 1 and 2)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1130) | (c) | |
| Terms, *order-level destination*, *point suppression*, *pending* (new paragraph) | pointer | S-0212, S-0503 and claim 27. In our engine the order-level destination is `Object::dest`; pending suppression is `econ.point_suppression` (`refresh::suppression_pending`). |
| 20 (a): the locomotor takes the destination at once, unless a pending suppression stops it. The drop empties the order-level destination and happens only when the locomotor is offered the destination. A pending suppression is the only exception to the drop and to fixing a landing point | changed | `teleport::take_destination` now calls `refresh::skip_point` before the phasing-in, held and erased refusal. Before, it came after (the Q-1130 interim). Tests in `claims_s0207_r36.rs`: `q1130_row_2_phasing_in_or_held_keeps_the_destination` and `q1130_row_2_suppression_skips_the_landing_point`. |
| 20 (a), the disabling state *being erased* (restated in the Q-1130 table, third row) | changed | Before, *being erased* had no state in `take_destination` (a seam). A temporal-beam lock (`temporal::erased`) now drops the destination. A warp member in its leaving stage still follows claim 20 (b)'s warp-pending immobilization, which leaves *L* unchanged (step 1g). The electromagnetic pulse still has no state. Test: `q1130_row_3_dropped_without_suppression` covers phasing in, held and erased. |
| 20 (e): an object being erased drops its destination under part a, except that a pending suppression keeps it, and it cannot jump | changed | as 20 (a) and S-0215 claim 2. The jump is impossible because the erased turn is skipped (`temporal::turn`). Test: `being_erased_under_pending_suppression_keeps_the_destination`. |
| 20 (f): while phasing in every new destination is dropped, except one that a pending suppression keeps | changed | as 20 (a): `q1130_row_2_phasing_in_or_held_keeps_the_destination`. |
| 20 (h): a destination given while phasing in is dropped unless a pending suppression keeps it | changed | as 20 (f). |
| 20 (h), *An empty assignment while a jump is pending* (new). It cancels the pending jump and the moving state. The landing point stays remembered and its reservation does not move. No jump follows until a later nonempty destination is accepted, and that destination clears the reservation from the remembered point | changed | `take_destination` with no destination now clears `TeleportState::landing` (the pending jump, which is also what reports the moving state) and keeps `mark`. The next accepted destination clears the mark from that cell (`aircraft_mark`). Before, a cleared destination kept *L* and the jump followed on the next movement update. Test: `claim_20_h_an_empty_assignment_cancels_the_pending_jump`. This applies to every object with the teleporting locomotor, through any empty assignment (a stop, a behaviour clear). |
| 27, *The restoring assignment's destination*: "(see the following paragraphs)" | (c) | |
| 27, *Suppression meets a teleporting destination (Q-1130)* (new): the suppression test belongs to every nonempty vehicle assignment, whatever the active locomotor. An unchanged offer is rejected without consuming. A refusal before the order-level destination is kept leaves suppression pending | changed | the test sits after the unchanged-destination rejection (`o.dest == st.for_dest`, which consumes nothing). Every vehicle assignment stores the order-level destination before any locomotor runs (`econ::assign_destination`), so no locomotor-side refusal happens earlier. A deployed vehicle's assignment is replaced by an empty one (`inplace::blocks_destination`), and empty assignments never test suppression (`skip_point` runs only for a nonempty point). |
| 27, Q-1130 table row 1 (not disabled, no suppression: claim 20 a fixes the landing point) | satisfied | `claim_20_jump_timing_and_effects` (`claims_s0207_teleport.rs`) and `a_pending_earlier_teleport_is_replaced` (`claims_s0207_r34.rs`). |
| 27, Q-1130 table row 2 (any state, suppression pending): the locomotor is not offered the point. No landing point is fixed and no claim 20 (a) refusal runs. No earlier landing point, pending jump or reservation changes. Suppression is consumed and the order-level destination stays | changed | the new order in `take_destination`. `teleport::tick` no longer removes a destination kept this way: it is the only state in which the taken destination equals the order-level one with no landing point. Tests: `q1130_row_2_suppression_skips_the_landing_point` (no point, no mark, consumed, no jump in five ticks, an equal offer ignored, a different one fixes its point), `q1130_row_2_phasing_in_or_held_keeps_the_destination` and `q1130_row_2_an_earlier_pending_jump_is_unchanged` (the earlier *L* and its mark stay, and the earlier jump is made). |
| 27, Q-1130 table row 3 (disabled, held, being erased or phasing in, no suppression: the destination is dropped and nothing is consumed) | changed | as 20 (a); being erased is new. `q1130_row_3_dropped_without_suppression`. |
| 27, S-0402's seeking drop adopts a remembered cell by an ordinary assignment, so the table rows apply to it | changed | see S-0402 claim 17 below: `claim_17_suppression_fixes_no_point_for_the_adopted_cell` (`claims_s0402_r28.rs`). |
| 27: suppression is consumed by a teleporting vehicle even while phasing in or held. A later equal offer is ignored unless refresh is pending. Refusals before the order-level destination (a bunker, an open-topped transport) leave it pending, and empty assignments never test it | changed | as row 2. The bunker and open-topped refusals belong to their owning specs; for them it is satisfied as described in the row above. |
| 27, *Reachability* (new) | (c) | not a rule. The state is not reachable in ordinary play, so the gate games do not reach it. |
| 27: suppression applies equally to a driving point, a jumpjet point "and to a teleporting landing point" | changed | as row 2. |
| 27, Backing and Confidence | (c) | |
| T1 track tables: the values are "engine constants: fixed inside the original's program, not read from any data file" | (c) | wording only. The tables in `drive` are unchanged. |
| Edge cases, *Odd points in the track data* (wording) | (c) | |
| State and ordering: the *teleporting, idle* row adds "not held" and "no suppression pending" | changed | as 20 (a) and row 2. |
| State and ordering, new rows: *teleporting, any state, suppression pending*; *L fixed, jump pending, empty destination*; *L fixed, no pending jump, next movement update* | changed | `q1130_row_2_suppression_skips_the_landing_point` and `claim_20_h_an_empty_assignment_cancels_the_pending_jump`. |
| State and ordering, *phasing in, destination given*: dropped unless suppression is pending | changed | `q1130_row_2_phasing_in_or_held_keeps_the_destination`. |
| Edge cases, *Orders given while phasing in are lost*, except a destination that a pending suppression keeps | changed | as above. |
| Edge cases, *A teleporting vehicle keeps a destination without a landing point under pending suppression*: `quirks.teleport_suppression_keeps_destination` (proposed name, default the original) | satisfied (revision 37) | revision 36 proposed the switch and left it to the lead; request Q-1131 asked. Revision 37 answers: the original behaviour with no switch, and the S-0215 claim 2 and S-0514 claim 17 exceptions apply unconditionally. That is the code (the switch briefly built in the meantime is reverted). The state is not reachable in ordinary play. |
| D22 run 17, the D32 paragraph, *Cross-spec changes to confirm* (the Q-1130 notes to S-0514, S-0215 and S-0214), Open questions | (c) | measurement text and notes. The three cross-spec notes landed as the revisions below. |

## S-0402 revision 28 (answers Q-1130 question 3)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1130) | (c) | |
| Terms, *Landing point, pending jump, vehicle-present mark* (new) | pointer | S-0207 claims 20, 21 and 27. |
| 17, seeking bullet: "at a seeking processing it drops its destination and runs the ring search from where it stands", for a harvester whose current locomotor is the teleport locomotor | changed | `harvest::seeking`: when `teleport::is_teleporter` holds (active locomotor; false during temporary driving), the remembered cell is adopted and then cleared (`drive_to`, then `drive_to_none`). A present destination is cleared too, and the processing goes on to the ring-search rows. Before, this was not built (Q-1130 interim). Tests in `claims_s0402_r28.rs`: `claim_17_the_remembered_cell_is_dropped` and `claim_17_a_present_destination_is_dropped`. |
| 17, *The seeking drop (Q-1130)*: every seeking processing past the *full* test, not only the first after the miner enters *harvest*. The table of seeking events | changed | as above. The *full* row drops nothing: `claim_17_full_is_not_dropped`. A search that sent the miner away and whose jump was blocked is dropped and searched again: `claim_17_a_blocked_jump_is_dropped_at_the_next_processing`. |
| 17: the drop clears the order-level destination and cancels the pending jump, but not the fixed landing point or its reservation | changed | through S-0207 claim 20 (h) (above). `claim_17_the_remembered_cell_is_dropped`: the miner loads at its own cell and the reservation stays at the remembered cell. Under pending suppression or a disabling state no point is fixed for the adopted cell: `claim_17_suppression_fixes_no_point_for_the_adopted_cell`. The electromagnetic pulse has no state in our engine. |
| 17: within a turn, a due processing sees the destination before that turn's locomotion. A destination the search sets is jumped to in the same turn | satisfied | the behaviour runs before the locomotor in a vehicle's turn (S-0203). The last step of `claim_17_a_blocked_jump_is_dropped_at_the_next_processing` shows it. |
| 17: whether a harvest order's destination is dropped before or after the first jump depends on the order-to-first-processing timing (H8 part h, `unmeasured`) | satisfied | the spec permits either outcome. Our engine follows its own order and processing timing. No test pins the order, because the spec leaves it open. |
| 17: the "remains seeking" row of claim 3 still holds for every harvester with other active locomotion, a temporarily driving factory-built miner included | satisfied | `claim_17_a_driving_flagged_miner_keeps_its_destination` (a flagged miner that drives), and `claim_16_a_chrono_miner_drives_its_first_leg_to_the_ore` (`claims_s0417_r20.rs`) for the temporary drive. |
| 17, Backing and Confidence | (c) | |
| State and ordering, seeking rows "has a remembered cell" and "has a destination": for such a chrono miner the drop applies, and the next processing is that of the row the ring search reaches | changed | the delay returned is the ring-search row's: 1 for loading in `claim_17_the_remembered_cell_is_dropped`; standard for another cell. |
| H8 parts h and i, the falsifiers, Open questions | (c) | measurement text. |

## S-0215 revision 54

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| Terms, *Pending suppression* (new) | pointer | S-0207 claim 27. |
| 2, the erasing and leaving-stage bullet: the target is dropped on each such tick, and so is the destination, except that a vehicle under a pending point suppression keeps its destination. The exclusion from processing is unchanged | changed | `temporal::turn` (being erased) and the leaving stage of `superweapons::warp::turn` read `refresh::suppression_pending` and keep the destination. The target is still dropped. Tests in `claims_s0207_r36.rs`: `being_erased_under_pending_suppression_keeps_the_destination` and `leaving_stage_under_pending_suppression_keeps_the_destination` (with and without suppression). |
| 2, the backing of the pending-suppression exception; B-scenario parts (a) and (b) | (c) | |

## S-0502 revision 11

| Claim | Class | Notes |
|---|---|---|
| Header; Terms list gains *landing point* (S-0207 claim 20) | (c) | |
| 27: point suppression also covers a teleporting locomotor's landing point (S-0207 claim 27) | changed | the shared code of S-0207 claim 27 (`teleport::take_destination`). Tests: see S-0207 Q-1130 table row 2. Nothing changes for the jumpjets. |

## S-0214 revision 38

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| Terms, *Being erased, phasing in*: a destination given while phasing in is dropped, except under a pending point suppression, which keeps it as the order-level destination, fixes no landing point and consumes the suppression | changed | the shared code of S-0207 claim 27. The click refusal of row GC1 is separate and unchanged. Test: `q1130_row_2_phasing_in_or_held_keeps_the_destination`. |

## S-0514 revision 7

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 16: an erased target normally cannot make its own teleporting jump because its destination is cleared. Under a pending point suppression the destination is kept, and the target still cannot jump | changed | being erased now drops a destination in `teleport::take_destination` (S-0207 claim 20 a), unless suppression is pending. The erased turn is skipped, so no jump happens either way. Test: `being_erased_under_pending_suppression_keeps_the_destination`. |
| 17: being erased clears the target and destination, except that a pending point suppression keeps the destination | changed | `temporal::turn`. Same test. |

## Save format and goldens

No state field changed: the save format stays 45 and the AI game format stays 13. The behaviour changes are:
- teleporting chrono miners drop their seeking destinations;
- an empty assignment cancels a pending teleport jump;
- a temporal-beam lock refuses destinations;
- pending suppression meets a teleporting destination.

`cargo test --release -p glasnost-sim-gate -- --include-ignored` passes against the unchanged goldens of the base (f43b0b5): `full_run_matches_golden`, `debug_prefix_matches_golden` and both `m6c` tests. Their hashes did not change, but no type in the gate fixtures uses the teleporting locomotor, so this does not test the changes. The real-data AI lane and the seed sweeps were not run here.

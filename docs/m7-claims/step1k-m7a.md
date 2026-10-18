# M7a step 1k claim ledger (S-0215 revisions 63 and 64, S-0314 revisions 27 and 28, S-0606 revisions 4 and 5, and seven more)

This is part 11 of M7a step 1. It covers the approved revisions that reached main after the step 1j intake. First S-0215 revision 63, which was already on main at 81dbbf8 but not yet built. Then 81dbbf8 to b0bad49 (S-0206 revision 15, S-0603 revision 7, S-0402 revision 30, S-0314 revision 27, S-0606 revision 4), merged in during this step. Last b0bad49 to 652bafc (S-0605 revision 8, S-0415 revision 18, S-0212 revision 50, S-0215 revision 64, S-0314 revision 28, S-0606 revision 5, glossary 55), merged in at the coordinator's instruction. `git log 652bafc..main -- spec/` is empty at the end of the step. Every changed claim gets one class, as in [step1j-m7a.md](step1j-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered, so no citation needed re-mapping. S-0606 revision 4 adds claims 16 to 21 and formula F5, and revision 5 renames an Edge-case switch. Those are new numbers, not renumbered ones.

| Spec | Revision | Answers |
|---|---|---|
| S-0215 behaviours | 63 (from 62) | none (claim 11's turning clause, scenario B47) |
| S-0206 turn rates | 15 (from 14) | none (a Terms pointer to S-0215 claim 11) |
| S-0603 grinder | 7 (from 6) | none (the cross-spec change S-0215 revision 63 asks for) |
| S-0402 harvest cycle | 30 (from 29) | none (the same: the turning body and the unload starts) |
| S-0314 healing and repair | 27 (from 26) | none (claim 17's rotation gate; a type that is also a bunker) |
| S-0606 vehicle bunker | 4 (from 3) | Q-1300 to Q-1308 |
| S-0605 infantry absorber | 8 (from 7) | Q-1159 |
| S-0415 deploy and undeploy | 18 (from 17) | none (a bunkered vehicle's *unload*) |
| S-0212 pathfinding | 50 (from 49) | none (the bunker-exit no-cell outcome, a pointer to S-0606) |
| S-0215 behaviours | 64 (from 63) | none (the tank bunker's movement-arrival paragraph) |
| S-0314 healing and repair | 28 (from 27) | Q-1158 |
| S-0606 vehicle bunker | 5 (from 4) | none (switch renamed; claim 12's continuation) |
| glossary | 55 (from 54) | (c) |
| `spec/README.md` | | index rows moved and revised: (c) |
| `spec/requests/` Q-1158, Q-1159, Q-1300 to Q-1308 (answered); Q-1160 to Q-1164 (new, open) | | request status: (c) |

**Retired interims.**
- Q-1158 (a docked vehicle's enter order on its own depot; the destination of a place-taking request) is answered by S-0314 revision 28. Its first half was not as built and is changed (below). Its second half was as built.
- Q-1159 is answered by S-0605 revision 8 as the interim had it: a vehicle still driving from an earlier order is ready at its first start because a queued *enter* lifts claim 11's moving condition.
- Q-1300 to Q-1308 are answered by S-0606 revision 4. Every interim in `bunker.rs`, `orders.rs`, `process.rs`, `econ`, `objects.rs` and the AI test is gone or now cites the claim that answers it. Q-1300 (the nearest object's metric) became F5 with two switches. Q-1301 (the arrival repeat): the *enter* processing makes no arrival test. Q-1302 (an order mid-entry) is claim 16. Q-1303 (the contact under orders) is claim 17: orders end the contact, and the old keep is gone. Q-1304 is claim 18. Q-1305 (no cell found) is claim 12: the placeholder cell (0, 0). Q-1306 is claim 21. Q-1307 is claim 19. Q-1308 is claim 20, in part. Its processing order is Q-1163.

The step 1j and earlier ledgers are historical and stay as they were. The lead's "M7a step ledger" table in `development.md` is unchanged.

**New requests** (block Q-1160 to Q-1165; Q-1340 to Q-1349 unused):
- Q-1160: the start at which a turn's end makes a driving vehicle ready (B47's "the late start counts that update's turning"), and what B47's attack order does to the move in progress. Interim:
  1. The facing is read at the tick number (S-0206 F1), so the first start of the tick in which the facing reaches its target already sees the turn ended.
  2. The order removes the route. A turn in place with no destination left ends at the next drive update, and the body keeps turning to its target facing.
- Q-1161: claim 11's move-destination clause, under which every ordered move from *area guard*, *sleep* or *attack* would wait for arrival. Interim: apart from the turning clause, "moving" stays movement in progress as built (`occupancy::in_progress`).
- Q-1162: S-0314 claim 17's rotation toward 16384 for a tethered vehicle names no start at which it begins. Interim: the rotation starts at movement arrival, after *sleep* is queued, so *sleep* waits for it.
- Q-1163: S-0606 claim 20's order within one *repair* processing of a combined depot and bunker. Interim: one S1 row first, then the depot's processing, whose delay is returned.
- Q-1164: S-0606 revision 5 claim 12 puts S-0517's deploy-fire rule first in a bunkered vehicle's *unload* continuation. S-0415 revision 18 says the deploy-fire rule does not apply to a bunkered vehicle, and that such a vehicle chooses 450 ticks. Interim: S-0415 (as built): no deploy-fire rule in the continuation.

Q-1165 is unused.

**Callers of readiness.** `process::ready` is the only test of S-0215 claim 11's readiness. The other functions named `ready` are other specs' own conditions: `econ::inplace::ready` (S-0517 claim 8, the deploy transition, which `process::ready` also reads), `cloak::machine::ready` (S-0516 claim 4), `econ::deploy::deploy_ready` (S-0214 Terms, deploy readiness) and `air::ready` (aircraft, not driving). The direct `occupancy::in_progress` callers are "stops if moving" rows, S-0415 claim 2's own moving test, firing, scatter and path tests, none of them a start under claim 11. The first per-tick start (`process::turn`), the late start (`process::late_start`) and every "starts it if ready" in the code call it: S-0402's unload rows (`harvest`), S-0603's queued *guard* (the first start), S-0605's entrant (`absorb`), S-0314's retry (`depot::vehicle_turn`) and arrival *sleep* (the first start), mind control, the release rows and the bunker (`release`, `mind`, `bunker`). So the turning clause reaches all of them through `process::moving`, with no change at the callers. `occupancy::in_progress` (S-0206's term) is unchanged. Its callers keep the narrower term, as S-0215's cross-spec note asks.

**Formats.** See the end of this file.

## S-0215 revision 63

| Claim | Class | Notes |
|---|---|---|
| 11: for a vehicle with the driving locomotor, a body still turning towards its given facing, for any reason, is "moving" for readiness; it becomes ready at the first start or late start after the turn ends | changed | `process::moving` = `occupancy::in_progress` or `process::drive_body_turning`. The second reads `body.turning(t)` at the start's tick, for `Locomotor::Drive` only. The late start reads after the drive update, so it sees that update's turning. Tests: `claim_11_turning_to_face_a_target_is_not_ready`, `claim_11_turning_clause_is_the_driving_locomotors` (a ship and a hovering vehicle stay ready). |
| 11: the exceptions apply to a turning vehicle as to a travelling one | satisfied | `process::ready`'s exceptions run before the moving test. `claim_11_turning_exceptions`. |
| 11: "and also while the locomotor has a move destination, or a next point ..." (both clauses of S-0206's term) | satisfied in part, Q-1161 | Built as movement in progress (`occupancy::in_progress`: a track, a turn in place before a step, path steps left), not the presence of the locomotor's move destination. Q-1161 interim. |
| B47: a turning vehicle keeps a queued *attack*; *attack* current after the tick *t* on which the facing last changed | changed | `claim_11_b47_turning_vehicle_keeps_a_queued_attack`. The attack order removes the route, and `drive::tick` now ends a turn in place with no destination left, so the vehicle is no longer stuck "moving" for ever. Q-1160 interim for the start and for the order's effect. |
| 11 (late start backing, (ii)), Terms (body facing, target facing, next point), the turret sentence | (c) | |
| Cross-spec changes to confirm (S-0206, S-0603, S-0402, S-0314) | (c) | each taken in below as its owner revised it |

## S-0206 revision 15

| Claim | Class | Notes |
|---|---|---|
| Terms, *locomotor moving the vehicle*: S-0215 claim 11's readiness is a different, wider query | pointer | `occupancy::in_progress` is unchanged; `process::moving` adds the turning clause. |
| Header | (c) | |

## S-0603 revision 7

| Claim | Class | Notes |
|---|---|---|
| Terms (per-tick start, late start, moving): a driving vehicle whose body still turns, for any reason, is moving | changed | through `process::moving` (S-0215 above). |
| 3: a unit whose approach failed and whose driving body still turns, in place or to face a target, keeps *guard* queued until the turn ends | changed | `process::moving`; `claims_s0215_r63.rs`. The grinder's own tests (`claim_3_failed_approach_gives_up`) are unchanged, because a unit idle at the order is not turning. |
| Open questions, Confidence | (c) | |

## S-0402 revision 30

| Claim | Class | Notes |
|---|---|---|
| 10, row 4: *unload* is queued after the turn to 16384 has ended, so it does not wait | satisfied | `harvest` queues it at the processing after the turn; `process::ready` sees no turning. Existing harvest tests are unchanged. |
| 11, *Turning body*: a start "if ready to change" waits for a turn to end, unless an exception applies | changed | `process::moving`, read by every start in `harvest` through `process::ready`. The tests are those of S-0215 claim 11. No harvest scenario is proposed (S-0402 inherits the timing). |
| *Refinery sold while unloading*, *Order while unloading*: the start waits for a turning body | changed | as above. |
| Header | (c) | |

## S-0314 revision 27

| Claim | Class | Notes |
|---|---|---|
| 17: for a tethered vehicle, *sleep* is queued from arrival, before the completed-entry exchange rotates it toward 16384; it does not become current while the body still rotates; an untethered vehicle is not rotated | changed | `depot::movement_arrival` starts the rotation to `ARRIVAL_FACING` right after it queues *sleep* when the vehicle is tethered and not yet facing east. `process::moving` then holds *sleep* until the turn ends. Q-1162 interim for the start of the rotation. Test: `claim_17_rotation_gate_on_the_arrival_sleep` (tethered: *sleep* queued while the body turns, current at the first start after; untethered: no rotation, *sleep* at the first start). |
| Terms, *Vehicle bunker*; 6, 16, 17, 18: a type that is also a vehicle bunker | changed | `depot::accepts` takes the bunker's permission rows. `depot::repeated_request` also refuses on `bunker::eligible`. `depot::docking_point` uses `bunker::docking_point`. `bunker::arrival` skips a depot, so the depot's movement-arrival exchange runs. The repair processing runs one S1 row, then the depot's processing (Q-1163 interim). `claim_20_combined_roles` (S-0606 tests). The importer warns (`bunker_depot_warning`). |
| Scenario H16 (the rotation gate's falsifiers), backing and confidence | (c) | |

## S-0606 revisions 4 and 5

Claim by claim in [S-0606.md](S-0606.md). In summary:

| Claim | Class | Notes |
|---|---|---|
| 5: arrival from movement only, the destination kept | changed | `bunker::arrival`, `process_enter` has no arrival test; `claim_5_arrival`. |
| 6, F5, Edge cases: candidates (infantry, vehicles, aircraft and buildings of the ground list), the chosen object by F5, ties to the first listed; `bunker_clear_far_point` (new, default on) and `bunker_clear_chosen_only` (renamed in revision 5) | changed | `bunker::candidates`, `chosen_distance`, `examined`. Tests: `f5_chosen_distance_both_positions`, `claim_6_far_point_both_positions`, `claim_6_chosen_only_both_positions`, `claim_6_buildings_are_candidates`. `every_spec_quirk_has_a_switch` is green again. |
| 12: *move* queued, every contact ended, the placeholder cell (0, 0), 450 ticks or the continuation | changed | `bunker::process_unload`, `continue_unload`. Tests: `claim_12_release_delay_and_contacts`, `claim_12_no_cell_placeholder`, `claim_12_bunkered_deploy_fire_interim` (Q-1164). |
| 16: an order mid-entry | changed | `process_repair`'s no-vehicle branch stops special-1 and special-2 when the walls were going up. Test: `claim_16_order_after_raising_walls`. |
| 17: orders end a bunkered vehicle's contact, the link holds | changed | the keep in `orders.rs` is gone. Test: `claim_17_orders_end_contact_link_holds`. |
| 18: no destination while bunkered | satisfied | `econ::assign_destination`, `bunker::enforce`; citation moved. |
| 19: other removals | changed | `bunker::removing`: a surviving bunker unlinks and returns to clearing. Test: `claim_19_other_removal`. |
| 20: bunker and depot on one type | changed | as S-0314 revision 27 above; `claim_20_combined_roles`; Q-1163. |
| 21: every track point driven | satisfied | as built; citation moved from Q-1306. |
| 1 to 4, 7 to 11, 13 to 15, T1, T2, F1 to F4 (wording) | (c) | |
| Header, backing, confidence, open questions, scenarios K1 to K9 | (c) | |

## S-0605 revision 8

| Claim | Class | Notes |
|---|---|---|
| 3: a vehicle still driving from an earlier order is ready at its first start (a queued *enter* lifts the moving condition); the not-ready examples are now deploy transitions, the own unload processing, *sticky* and *rescue* | satisfied | as built under the Q-1159 interim (`process::ready`); `claim_3_vehicle_always_waits_in_line`. |
| Scenario A2c, header (requests list) | (c) | |

## S-0415 revision 18

| Claim | Class | Notes |
|---|---|---|
| 1: a bunkered vehicle's *unload* releases it, then continues with only the first of: harvester or weeder (S-0402), deploys into a building (claims 2 to 6, whatever its passengers or deploy-fire flag), simple deployer (S-0517); none of them: 450 ticks, no draw; the deploy-fire rule does not apply | changed | `bunker::continue_unload`. Tests: `claim_12_release_delay_and_contacts` (450), `claim_12_bunkered_deploy_fire_interim` (a deploy-fire type: no deploy-fire rule, 450; Q-1164 against S-0606 revision 5). |
| Backing, confidence | (c) | |

## S-0212 revision 50

| Claim | Class | Notes |
|---|---|---|
| 23 (the bunker-exit caller): no cell found gives the placeholder cell (0, 0) | pointer | S-0606 claim 12; `claim_12_no_cell_placeholder`. |

## S-0215 revision 64

| Claim | Class | Notes |
|---|---|---|
| 29, *Tank bunker*: the movement-arrival test applies at a bunker from movement only; the bunker queues *repair*, the vehicle stays in *enter* and queues nothing; arrival keeps the destination | satisfied | as S-0606 claim 5 revision 4: `bunker::arrival`; `claim_5_arrival`. |
| Cross-spec note on S-0206 | (c) | |

## S-0314 revision 28

| Claim | Class | Notes |
|---|---|---|
| 20: a docked vehicle standing on its depot given an enter order on that depot: the queued entry in full, *enter* queued without a start, the depot remembered, no destination; *enter* current at the next start and processed: only the repeated request, nothing queued at either side; the retry in the same turn refuses (claim 16), clearing contact, queued entry and destination; *enter* falls back at the next processing | changed | `depot::order_applied` returns `NoDestination` with the queued entry and memory. `depot::process_enter`'s tethered branch queues nothing for a queued entry. `vehicle_turn`'s refusal does not fall back for a vehicle standing on the depot. A new arm in `process::turn` falls back from an *enter* with no contact, memory or object destination (S-0215 claim 29). Test: `claim_20_same_depot_enter_while_docked` (rewritten). At full health contact still ends in the order's tick. |
| 23: a repeated request that takes a place gives no destination; only the retry's grant assigns the depot | satisfied | as built under the Q-1158 interim; `claim_23_free_place_taken_by_the_processing`. |
| 20, Table row "docked vehicle standing on the depot, enter order on this depot" | changed | as claim 20. |
| Scenario H18 (the tick-by-tick reads), backing, confidence, header | (c) | |

## Glossary 55, README

| Item | Class | Notes |
|---|---|---|
| *Bunkered*: claims 1 to 21 | (c) | |
| Index rows | (c) | |

## Formats and gate

- Save format 54 and AI game format 22, both unchanged: this step adds no saved or hashed state. The turning clause reads the existing body turning state. Bunker revisions 4 and 5 and S-0314 revisions 27 and 28 reuse the existing link, stage, contact, queued-entry and remembered-depot fields.
- Goldens regenerated. `duel.txt`, `duel-m6c.txt` and `duel-m7a.txt` change: the turning clause holds starts until a body's turn ends, and the bunker's unload, clearing and order rules changed, so play differs. `duel.txt` now ends 290 ticks later (3 031 hash lines, from 3 002).
- The 16-seed sweep (`GLASNOST_SWEEP=1-16`), after the last merge with main. Seed 4 ends at tick 11 250 with Alpha's win, and every other seed plays all 12 000 ticks. The gate seed 3 plays all 12 000 ticks. Every seed's smallest counter is the sensor orders (2, the known exception). Every other minimum is at least 3. Minimum / median / maximum:

| Counter | Min | Median | Max |
|---|---|---|---|
| berserk weapon orders (S-0610) | 14 | 25 | 36 |
| units made berserk (S-0610 claim 4) | 31 | 48 | 71 |
| psychic sensor orders (S-0610 claim 9) | 2 | 2 | 2 |
| hostile intent seen by a sensor (S-0610 claims 9 to 11) | 2 304 | 4 478 | 5 427 |
| psychic reveal orders (S-0610 claim 13) | 9 | 12 | 12 |
| psychic reveals fired (S-0610 claims 13 and 14) | 9 | 12 | 12 |
| slave-miner buildings (S-0402 claim 14) | 3 | 5 | 13 |
| slaves at work (S-0402 claim 14) | 2 713 | 6 783 | 9 273 |
| gattling stage rises (S-0609 claim 2) | 4 | 18 | 37 |
| gattling at its top stage (S-0609 claim 2) | 443 | 2 004 | 4 315 |
| takeover orders (S-0601) | 4 | 15 | 24 |
| takeovers (S-0601 claim 6) | 7 | 17 | 21 |
| gas clouds (S-0613 claims 2 to 4) | 44 | 126 | 260 |
| gas hits (S-0613 claim 13) | 70 | 386 | 1 278 |
| absorber entry orders (S-0605) | 5 | 17 | 109 |
| infantry absorbed (S-0605 claims 1 and 7) | 5 | 8 | 12 |
| grinder entry orders (S-0603) | 4 | 6 | 14 |
| units ground (S-0603 claims 5 and 8) | 4 | 6 | 13 |
| tank bunker build orders (S-0606) | 4 | 4 | 4 |
| bunker entry orders (S-0606 claim 3) | 3 | 9 | 14 |
| vehicles bunkered (S-0606 claim 7) | 3 | 7 | 12 |
| bunker exit orders (S-0606 claim 12) | 3 | 7 | 11 |

No minimum dropped below 3, so no setup change was needed.

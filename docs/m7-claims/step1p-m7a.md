# M7a step 1p claim ledger (S-0607 revision 7, S-0207 revision 41, S-0209 revision 43, S-0502 revision 15)

This is part 16 of M7a step 1 (intake 11). It covers the approved revisions that reached main after the step 1o intake, from e890daf to e0b0411 (commits fac238f, e870064, d1943f4 and e0b0411): S-0607 revision 7, S-0207 revision 41, S-0209 revision 43, S-0502 revision 15, the index and the five answered requests. Every changed claim gets one class, as in [step1o-m7a.md](step1o-m7a.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered. One quirk switch is new, `quirks.lift_standin_unmarked_cell` (default on). It is wired in `glasnost_rules::quirks` and the ruleset schema, and both positions are tested (`every_spec_quirk_has_a_switch` passes). The new tests are in `crates/deterministic/sim/tests/claims_s0607_r7.rs` (19 tests) unless another file is named.

| Spec | Revision | Answers |
|---|---|---|
| S-0607 magnetic lift | 7 (from 6) | Q-1337, Q-1338, Q-1339, Q-1340, Q-1341 (`answered-by`) |
| S-0207 ground vehicle movement | 41 (from 40) | none (claim 20 b points at S-0607 claim 23) |
| S-0209 occupancy and crowding | 43 (from 42) | none (claim 1 points at S-0607 claim 23) |
| S-0502 jumpjet movement | 15 (from 14) | none |
| `spec/README.md` | | index rows revised: (c) |
| `spec/requests/` Q-1337 to Q-1341 (answered) | | request status: (c) |

**Retired interims:**
- Q-1337 (the drop-pod grip end). The old interim kept the lifter's target. Claim 12 now has a drop-pod row: the grip ends with the unit set down and the lifter's target cleared, so claim 11's refusals stop at once and the wave ends at its next update. Built in `lift::drop_pod_ends`. Other attackers keep their target (design: our engine reproduces none of the original's touchdown target drop).
- Q-1338 (the stand-in's cell mark). Revision 5 had already answered it in substance, and step 1o built that. Revision 7 confirms it and puts it under the new quirk switch, whose fixed position keeps the own cell's mark at the seize for the driving, walking and ship stand-ins (`lift::install_stand_in`).
- Q-1339 (a landed aircraft with a ground stand-in). The old interim gave it no stand-in movement. Now a landed aircraft with the teleporting stand-in jumps under S-0207 claim 20 to the centre of the cell of the lifter's point. With driving, walking or ship it stays where it stands (design), and the drop pod works as for any unit (`lift::install_stand_in`, `instruct_stand_in`, `stand_in_tick`, `end_stand_in`).
- Q-1340 (the teleporting stand-in's arrival). The old interim ended the grip at the jump and kept the lifter's target. Now the jump ends the grip, and S-0207 claim 20 e's target drop clears the lifter's target in the same tick. With no jump (the landing point is the unit's own position), the grip ends at the first movement update and the lifter keeps its target. With no cell from the landing search at the seize, the grip is kept (`LiftState.grip_without_dest`, `lift::enforce`, `lift::given_no_destination`). The driving guard-row end at an exact point is built and tested.
- Q-1341 (no warning for an empty locomotor value). Claim 22 now says that an empty value makes no entry and does not warn. This was already built that way; the interim comments in `crates/convert/src/sim_combat.rs` and `sim_lift_tests.rs` are now citations.

**Step 1o's requests, checked against revision 7** (the request files are not edited):
- Q-1342 (a lifter-ended no-cell drop: whether the point and commitment are kept): not answered. Revision 7 changes only claim 23 and its neighbours, not claim 13's no-cell rule. Interim kept (`jump::stop_with`; `claims_s0607_release.rs`).
- Q-1343 (with `jumpjet_no_cell_self_destruct` off, whether claim 9's set-down descent follows the hover re-entry): not answered. Claim 23's new no-cell case is about the teleporting landing search, not the jumpjet stop. Interim kept (`lift::set_down_released`).

**New requests:** none. Block Q-1344 to Q-1352 is unused.

## S-0607 revision 7

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, request list) | (c) | |
| 12: new row, the gripped unit's drop-pod stand-in ends: the lifter's target is cleared | changed | `lift::drop_pod_ends` clears the lifter's target and burst. Test: `claim_12_drop_pod_end_clears_the_lifter_target` (no *rearming* answer after the end, the wave ends). Q-1337 retired. |
| 12: a lifter whose grip ended without clearing its target may fire again; the hit seizes nothing and does no damage | satisfied | claims 1 and 3 as built. Test: `claim_12_a_second_hit_on_a_seized_unit_seizes_nothing`. |
| 12: backing (L13 parts b and c) | (c) | |
| 22: an empty value makes no entry (S-0001 claim 8) and gives no warning; a malformed value is read as S-0207 claim 14 reads it and warns on a seizing warhead | satisfied | as built at step 1n. Tests: `lift_fields`, `lift_locomotor_malformed_keeps_the_earlier_layer` (convert). Q-1341's interim comments are now citations. |
| 23: Occupancy, driving and ship: no mark until the stand-in starts a step into a neighbouring cell, then S-0209 claim 1; a unit that has moved and stops keeps its last cell's mark | satisfied | as built at step 1o. Test: `claim_23_driven_stand_in_keeps_its_last_mark`. |
| 23: Occupancy: a unit that never moves stays unmarked for good, also after its grip ends | changed | `lift::end_stand_in` and `teleport::removal_marks` (through `lift::stand_in_marks`) no longer clear a mark that an unmoved unit does not hold; before, they cleared its cell's mark, which could be another vehicle's. Test: `claim_23_unmoved_stand_in_clears_no_foreign_mark`. |
| 23: Occupancy, teleporting: fixing the landing point moves the mark at once to the landing cell; the own cell stays unmarked until the jump; the landing cell stays marked after it | changed | `teleport::stand_in_instruction` calls `teleport::fix_landing_with(.., release_old = false)`: the seize has already given up the own mark, so the landing reservation releases nothing else. Test: `claim_23_teleport_marks_the_landing_cell` (another mark in the unit's cell survives). |
| 23: Occupancy, walking stays unmarked; drop pod marks again | satisfied | Tests: `quirk_lift_standin_unmarked_cell_both_positions`, `claim_23_drop_pod_ends_at_once` (`claims_s0607_standin.rs`). |
| 23: what the missing mark changes: listed occupants first; while the destination is the lifter the unit is moving and, before its first step, between steps: an allied mover meets *moving blocker* (VP22); after the destination is gone, *stationary friendly* (VP20); the enemy row; a crushable unit gives *clear* to a crusher (VR2, VR3) | changed | `drive::between_steps_or_chain` now also asks `lift::stand_in_between_steps` (a moving walking or teleporting stand-in, which has no driving state of its own). Before, an allied mover met *clear* at an unmoved walking stand-in. Tests: `claim_23_allied_entry_outcomes`, `claim_23_enemy_entry_outcomes`; `claim_23_unmoved_stand_in_holds_no_mark` (`claims_s0607_standin.rs`) now expects *moving blocker*. |
| 23: the occupancy-record-only rules see the cell as free: claim 7's search for another seize can choose an unmoved seized unit's cell | satisfied | as built. Test: `claim_7_search_can_choose_an_unmoved_seized_cell` (with the switch off the cell is refused). |
| 23: Fixed behaviour: driving, walking and ship keep their own cell's mark at the seize; teleporting unchanged | changed | `lift::install_stand_in` with the switch off claims the own cell's vehicle mark (and the driving claim for drive and ship). Test: `quirk_lift_standin_unmarked_cell_both_positions`. |
| 23: drop pod: the original's descent and touchdown; our engine ends it at once with the touchdown result: the lifter's target cleared, claim 11's refusals stop at once, the cell marked again; every other object keeps its target | changed | as claim 12's new row. Tests: `claim_12_drop_pod_end_clears_the_lifter_target`, `claim_23_drop_pod_other_attackers_keep_their_target`. The Q-1337 interim assert in `claim_23_drop_pod_ends_at_once` is removed. |
| 23: removals of the destination: the teleporting jump ends the grip, the jump's target drop clears the lifter's target in the same tick, no further jump | changed (tests) | as built (`lift::enforce` after `teleport::tick`; `teleport::targeters_drop`). Test: `claim_23_teleport_jump_ends_the_grip_and_clears_the_target`. Q-1340 retired. |
| 23: no jump (the landing point is the own position): the grip ends in that update and the lifter keeps its target | satisfied | Test: `claim_23_teleport_without_a_jump_keeps_the_target`. |
| 23: no cell from the landing search during the seize's assignment: the removal does not end the grip; the unit stays with no destination, never jumps, and the grip lasts until another event of claim 12 | changed | new `LiftState.grip_without_dest` (saved and hashed), set by `lift::seize` when the stand-in's destination is none. `lift::enforce` then keeps the grip, and a later empty assignment (`lift::given_no_destination`, from `econ::assign_destination`) is claim 12's no-destination event. Tests: `claim_23_teleport_no_cell_keeps_the_grip`, `claim_23_no_cell_grip_save_round_trip`. |
| 23: driving and ship at an exact point: S-0207 claim 26's guard row ends the grip; the lifter keeps its target | satisfied | Test: `claim_23_drive_exact_point_ends_the_grip`. The rare layout is made by placing the unit on its point. |
| 23: a landed aircraft, teleporting: S-0207 claim 20; entry outcome *clear* (S-0209 claim 14), so the landing point is normally the centre of the cell of the lifter's point; lands at terrain height; the grip ends at the jump | changed | `lift::install_stand_in` stashes the aircraft's own teleport state and installs a fresh one; `stand_in_tick` runs `teleport::tick`; `teleport::removal_marks` releases the object-present marks. Test: `claim_23_landed_aircraft_teleports`. Q-1339 retired. |
| 23: a landed aircraft, driving, walking and ship (design): stays where it stands, no move or turn, the grip ends only by a lifter-side event | changed (tests) | as the old interim, now the spec's rule. Test: `claim_23_landed_aircraft_other_stand_ins_stay`. |
| 23: a landed aircraft, drop pod: as for any unit | satisfied | Test: `claim_23_landed_aircraft_drop_pod`. |
| 23: taking off: with `lift_standin_stays_seized` on it never takes off; with the fixed behaviour it is restored and can take off | changed | `lift::end_stand_in` puts the aircraft's own marks back. Tests: `claim_23_landed_aircraft_teleports` (on), `claim_23_landed_aircraft_restored_takes_off` (off). |
| 23: backing and confidence | (c) | |
| State table: the three new rows (teleporting jump, another ground stand-in, drop pod) | changed | as claim 23. |
| Edge cases: `quirks.lift_standin_unmarked_cell` | changed | wired (`glasnost_rules::quirks`, `data/schema/ruleset.schema.json`). Test: `quirk_lift_standin_unmarked_cell_both_positions` and the both-position checks in `claim_23_allied_entry_outcomes` and `claim_7_search_can_choose_an_unmoved_seized_cell`. With the switch off, a landed aircraft is not marked: the fixed behaviour names only the vehicle-present mark. |
| Import mapping: warhead `Locomotor` (empty value: no entry, no warning) | satisfied | as claim 22. |
| Scenario L13 (parts b, c, e and f); open questions | (c) | |

## S-0207 revision 41

| Claim | Class | Notes |
|---|---|---|
| Terms: *drive vehicle* (stock-data wording removed) | (c) | |
| 20 b: a seize-time no-cell landing search removes the destination before the seized state; it does not end the grip (S-0607 claim 23) | changed | through S-0607 claim 23 (`LiftState.grip_without_dest`). Test: `claim_23_teleport_no_cell_keeps_the_grip`. |
| 20: backing and confidence for the seize-time no-cell case | (c) | |
| 25: backing (the 208-lepton completion boundary unmeasured) | (c) | |
| Formulas: the flag-carrier unit is progress units per tick | (c) | the code already uses progress per tick. |
| State table and Edge cases: a suppressed assignment leaves any earlier landing point, pending jump and reservation unchanged | satisfied | as built at M6c step 1h. Test: `q1130_row_2_an_earlier_pending_jump_is_unchanged` (`claims_s0207_r36.rs`). |
| Open questions (infantry or aircraft with the driving locomotor) | (c) | |

## S-0209 revision 43

| Claim | Class | Notes |
|---|---|---|
| Front matter (approval date) | (c) | |
| 1: seized vehicles with a ground stand-in: S-0607 claim 23 owns the exception, under `quirks.lift_standin_unmarked_cell` | changed | through S-0607 claim 23 (above). Tests: `quirk_lift_standin_unmarked_cell_both_positions`, `claim_23_allied_entry_outcomes`, `claim_23_enemy_entry_outcomes`, `claim_23_teleport_marks_the_landing_cell`. |

## S-0502 revision 15

| Claim | Class | Notes |
|---|---|---|
| Front matter; "Changes for other specs" removed (taken into S-0607 at revision 6) | (c) | |

## Format and gate

- `LiftState.grip_without_dest` is new hashed state, so the save format moves from 57 to 58. The AI game format stays 23: the brain did not change.
- All eight goldens were regenerated, because the state hash changed with the new field. The play counters did not change.
- Seed sweep over seeds 1 to 16: the same counters as at step 1o, to the last digit. The gate fixture's lifter warhead names the jumpjet locomotor, so claim 23 never runs in the duel. Seed 2 still plays all 12 000 ticks with every counter at least 3 except the sensor orders' known 2, so `M7A_SEED` stays 2. The counters are in [development.md](../development.md)'s S-0607 section (*Step 1p*).

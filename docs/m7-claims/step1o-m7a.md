# M7a step 1o claim ledger (S-0607 revisions 5 and 6, S-0502 revision 14)

This is part 15 of M7a step 1 (intake 10). It covers the approved revisions that reached main after the step 1n intake, from 94a8ae6 to e890daf (commits db4b2e0, e5af362 and e890daf): S-0607 revisions 5 and 6, S-0502 revision 14, the index and the five answered requests. Every changed claim gets one class, as in [step1n-m7a.md](step1n-m7a.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered and no quirk switch was added (`every_spec_quirk_has_a_switch` passes). Claim 13's no-cell rule is a design rule with no switch: the original has no behaviour there to keep. The new tests are in `crates/deterministic/sim/tests/claims_s0607_release.rs` unless another file is named.

| Spec | Revision | Answers |
|---|---|---|
| S-0607 magnetic lift | 6 (from 4) | Q-1332, Q-1333, Q-1334, Q-1335, Q-1336 (`answered-by`); Q-1338 in substance, though still `open` |
| S-0502 jumpjet movement | 14 (from 13) | none |
| `spec/README.md` | | index rows revised: (c) |
| `spec/requests/` Q-1332 to Q-1336 (answered) | | request status: (c) |

**Retired interims:**
- Q-1332 (the stop on dropping). The old interim released the reservation only, with no search and no self-destruction. Claim 13 now runs S-0502 claim 17's stop at a drop and at a set-down. It is built (below), and the comments and ledger text citing the interim are gone.
- Q-1333 (which buildings give a docking point). The old interim was the repair depot and the vehicle bunker. Claim 7 now names the pad, repair-depot and vehicle-bunker flags and gives a precedence. Built in `lift::lifter_point` and `lift::docking_point`. `depot::docking_point` is unchanged.
- Q-1334 and Q-1335 were already built at step 1n (claims 22 to 24). Nothing is left to retire.
- Q-1336 (claim 4 against another named locomotor). Claim 4 now says outright that it comes first, an unknown identifier included. This was already built that way; the interim comment is now a citation, and a test was added.
- Q-1338 (a ground stand-in's cell mark), answered in substance by claim 23's new Occupancy bullet. Installing a stand-in marks no cell, and the drop-pod stand-in marks the cell again when the unit's own locomotor returns. The interim mark is removed (`lift::install_stand_in`).

**Step 1n's requests, checked against revisions 5 and 6** (the request files are not edited):
- Q-1337 (the drop-pod grip end and the lifter's target): not answered. Claim 23 adds only the re-marking. Interim kept.
- Q-1339 (a landed aircraft with a ground stand-in): not answered. Interim kept.
- Q-1340 (the teleporting stand-in's jump, or a driving stand-in's arrival): not answered. Interim kept.
- Q-1341 (no warning for an empty warhead locomotor value): not answered. Interim kept.

**New request** (block Q-1342 to Q-1349), with its interim in the code (`jump::stop_with`) and in the S-0607 and S-0502 ledgers:
- Q-1342: on a lifter-ended drop with no cell for the stop, does the unit keep the seize's movement point and commitment? Claim 13 and S-0502 claim 17 say this only for a set-down unit. The answer decides whether the fall over a cell centre uses the look-ahead. Interim: both are cleared, as for a set-down unit.

Q-1343 to Q-1349 are unused.

## S-0607 revisions 5 and 6

| Claim | Class | Notes |
|---|---|---|
| Front matter, Scope and Terms wording ("with stock data" removed) | (c) | |
| 4: decided before the warhead's locomotor, whichever it names, an unknown identifier included | satisfied | as built (`lift::seize_hit` runs claim 4 before `SeizeLocomotor::of`); the Q-1336 comment is now a citation. Test: `claim_4_comes_first_whatever_the_warhead_names` (hover, drive and unknown). |
| 7: only the pad, repair-depot or vehicle-bunker flag gives a docking point; every other building, a refinery included, gives the foundation centre | changed | `lift::lifter_point` reads the three flags. Test: `claim_7_which_buildings_give_a_docking_point` (plain, refinery, weeder, war factory: the centre; depot and pad with one place: centre plus the displacement). |
| 7: the precedence (the obsolete-unloading fixed point, then the resource-unloading point at the centre +128 *x*, then the bunker's quadrant point on the unit's side, then the docking places) | changed | `lift::docking_point`. Test: `claim_7_docking_point_precedence`. |
| 7: the docking-place rule (count 0: the centre; 1: the sole displacement; more: the place held through a contact, else the centre) | changed | `lift::docking_point`. Test: `claim_7_docking_place_rule`. |
| 7: height (the building's height plus the displacement's); with the jumpjet stand-in only the cell matters | satisfied | `jump::lift_point` replaces the point by a cell centre at terrain height, as built at step 1m. Tests: `claim_5_the_seize`, `claim_7_point_fixed_at_the_seize` (`claims_s0607.rs`). |
| 9: claim 13's stop applies to a no-destination set-down, which flies to the found point before it descends | changed | `lift::steer` calls `set_down_released` (lifter-ended false). Test: `claim_9_no_destination_flies_to_the_found_point`. |
| 9: with no cell, S-0502 claim 17's self-destruction applies unchanged (quirk and no credit) | changed | `jump::stop_with` without `lifter_ended`. Test: `claim_9_no_cell_keeps_the_ordinary_outcome` (on: destroyed with no credit; off: alive in the air). |
| 10 and 11: refusal ranks unmeasured | (c) | backing only. |
| 13: the jumpjet stop in both cases (drop and set-down), in the same update | changed | `lift::drop` and `lift::end_grip` call `jump::lift_release_stop`. Tests: `claim_13_drop_keeps_the_found_point_and_commitment`, `claim_13_committed_set_down_takes_off_then_restores`. |
| 13: not moving: nothing happens | satisfied | `stop_with` returns early. Test: `claim_17_stopping_an_idle_jumpjet_does_nothing` (`claims_s0502_land.rs`). |
| 13: a cell found: the point becomes the stop's point; flight state, speeds and target height untouched; order-level destination none; commitment kept | changed | `jump::stop_with` (point form, claim 16 re-order). `jump::order_point` no longer flips a seized *descending* unit to *ascending*. Test: `claim_13_drop_keeps_the_found_point_and_commitment`. |
| 13: where a dropped unit falls (off-centre: claim 15's detour replaces the point; over the centre: zero speed, point kept) | changed | `jump::lift_dropped` (the detour) and `jump::lift_fall` (now keeps the point and commitment). Tests: `claim_13_drop_keeps_the_found_point_and_commitment`, `claim_17_release_over_clear_land` (`claims_s0607.rs`; the detour to the cell centre). |
| 13: how high it is held: a kept point in another cell gives the look-ahead at zero speed, from its own cell's clearance only | changed | `jump::height_step` (with S-0502 F3). Tests: `claim_13_kept_point_elsewhere_holds_the_fall_at_the_clearance`, `claim_13_point_in_own_cell_falls_by_the_direct_reference`. Both fail without the change (checked by mutation). |
| 13: no cell, grip ended by its lifter (design): no self-destruction in either quirk position; a dropped unit falls; a set-down unit keeps no point, is not moving, stays put and claim 20 restores it | changed | `jump::stop_with(.., lifter_ended)`. Tests: `claim_13_no_cell_lifter_drop_does_not_self_destruct`, `claim_13_no_cell_lifter_set_down_stops`. Whether a dropped unit keeps its point there: Q-1342 (interim: cleared). |
| 13: a set-down unit still moving keeps the point and commitment, takes off at its next eligible update, flies to the point and lands; restored only after | changed | `lift::set_down_released` leaves a committed unit to `jump`. Tests: `claim_13_committed_set_down_takes_off_then_restores`, `claim_10_set_down_seized_unit_deploys` (now waits for the landing, `claims_s0607.rs`). |
| 20: not restored while the movement commitment lasts | satisfied | `lift::restore_at_rest` already tests `jump::moving`. Test: `claim_13_committed_set_down_takes_off_then_restores`. |
| 22: claims 1 to 4 decide first; the mechanical-walking and walking glosses | (c) | as built at step 1n. Test: `claim_22_cannot_stand_in_does_not_seize` (`claims_s0607.rs`; hover, mech, fly, rocket, tunnel, unknown). |
| 23: Occupancy: the seize gives up the cell's mark and installing the stand-in marks none; the unit holds no mark until its stand-in's movement marks one | changed | `lift::install_stand_in` no longer claims the cell (Q-1338 interim retired). Tests: `claim_23_stand_in_marks_are_released` (rewritten), `claim_23_unmoved_stand_in_holds_no_mark` (`claims_s0607_standin.rs`; another vehicle's entry is clear). |
| 23: the drop pod marks its cell again as its own locomotor returns | satisfied | `lift::drop_pod_ends` puts the own marks back (`own_marks_back`). Test: `claim_23_drop_pod_ends_at_once` (`claims_s0607_standin.rs`). |
| 24: fixed alternative unmeasured | (c) | |
| State table: arrival drops only at height 1 lepton or more, otherwise set down | satisfied | `lift::arrived` goes through `end_grip`'s height test. Test: `claim_13_set_down_at_height_zero` (`claims_s0607.rs`). |
| State table: a committed set-down unit's row | changed | as claim 13. |
| Edge cases: no cell for the stop when a lifter lets go | changed | as claim 13. |
| Import mapping: `Helipad`, `UnitRepair`, `Bunker`, `Weeder`, `Refinery`, `NumberOfDocks`, `DockingOffset`*n* | satisfied | imported for S-0314, S-0404 and S-0606 (`BuildingType.helipad`, `unit_repair`, `bunker`, `econ.weeder`, `econ.refinery`, `econ.number_of_docks`, `econ.docking_offsets`). `WeaponsFactory` selects no docking point: `claim_7_which_buildings_give_a_docking_point`. |
| Measurement setup; scenarios L1, L3 (runs b, e, f), L9, L12 (docking variant) and L13 (parts a to d); backing; confidence; open questions | (c) | |

## S-0502 revision 14

| Claim | Class | Notes |
|---|---|---|
| 5: take-off at the first eligible update with a movement commitment, even with an empty order-level destination | satisfied | `jump::fly` keys the take-off on the commitment (`JumpState.moving`). Test: `claim_13_committed_set_down_takes_off_then_restores`. |
| 9 and F3: "destination cell" is the movement point's cell, not the order-level destination's; crashing included, at zero speed; no point: the direct reference | changed | `jump::height_step` decides by `JumpState.dest`'s cell; with no point it uses the direct reference. At zero speed only the own cell's clearance counts (as built). Tests: `claim_13_kept_point_elsewhere_holds_the_fall_at_the_clearance`, `claim_13_point_in_own_cell_falls_by_the_direct_reference`; the existing S-0502 claim 9 tests still pass. |
| 9: J27 b reachability note | (c) | |
| 17: the magnetic-release safety departure (design) | changed | `jump::stop_with(.., lifter_ended)` and `jump::lift_release_stop`. Tests: `claim_13_no_cell_lifter_drop_does_not_self_destruct`, `claim_13_no_cell_lifter_set_down_stops`, `claim_9_no_cell_keeps_the_ordinary_outcome`. |
| 24: the grounded-with-a-commitment row; "move order" now "movement commitment" | satisfied | the commitment and point are kept by the stop's point form; take-off as claim 5. Test: `claim_13_committed_set_down_takes_off_then_restores`. |
| Edge cases (the quirk switch and the safety departure) | changed | as claim 17. |
| State table (grounded row) | satisfied | as claim 5. |
| Scenario J27 (a, b, c); backing; "Changes for other specs" | (c) | J27 c is the design test above. |

## Format and gate

- No hashed state was added, so the save format stays at 57 and the AI game format at 23.
- Behaviour changed (the stop at release, the kept point, the look-ahead by the movement point, the unmarked stand-in), so the goldens were regenerated.
- Seed sweep over seeds 1 to 16: seed 1 still plays all 12 000 ticks but gives only 2 takeover orders. `M7A_SEED` moves from 1 to 2, the lowest seed that plays all 12 000 ticks with every counter at least 3 except the sensor orders' known 2. Only `duel-m7a.txt` changed; it was regenerated with seed 2. The counters are in [development.md](../development.md)'s S-0607 section (*Step 1o*).

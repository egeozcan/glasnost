# M7a step 1n claim ledger (S-0607 revision 4, S-0301 revision 23, S-0502 revision 13, S-0215 revision 68, S-0207 revision 40, glossary 57)

This is part 14 of M7a step 1 (intake 9). It covers the approved revisions that reached main after the step 1m intake, from f357d50 to 94a8ae6: S-0607 revision 4, S-0301 revision 23, S-0502 revision 13, S-0215 revision 68, S-0207 revision 40, the glossary at revision 57 and the index. Every changed claim gets one class, as in [step1m-m7a.md](step1m-m7a.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered. S-0607 gains claims 23 and 24, the Terms entry *stand-in locomotor* and the quirk `lift_standin_stays_seized`.

| Spec | Revision | Answers |
|---|---|---|
| S-0607 magnetic lift | 4 (from 3) | Q-1321 (`answered-by`); Q-1334 and Q-1335 in substance, though still `open` |
| S-0301 firing cycle | 23 (from 22) | none |
| S-0502 jumpjet movement | 13 (from 12) | none |
| S-0215 behaviours | 68 (from 67) | none |
| S-0207 ground vehicle movement | 40 (from 39) | none |
| glossary | 57 (from 56) | none |
| `spec/README.md` | | index rows moved and revised: (c) |
| `spec/requests/` Q-1321 (answered) | | request status: (c) |

**Retired interim.** Q-1321 (a seizing warhead naming another locomotor, or none). Its last interim (an empty or malformed value keeps the value in force, with a warning) is now claim 22's rule: the value is ignored as if the key were absent, so an earlier layer's value stays, else teleporting. The comment and ledger text citing the interim are gone. One difference remains: an empty value cannot warn, because under S-0001 claim 8 it makes no entry at all and the importer never sees it (Q-1341, below).

**Requests filed at step 1m, checked against revision 4** (the request files are not edited):
- Q-1332 (the stop on dropping and S-0502 claim 17): not answered. Interim kept.
- Q-1333 (which buildings give a docking point): not answered. Claim 23 mentions "a docking point" but does not say which buildings. Interim kept.
- Q-1334 (how a stand-in-seized unit moves and is restored): answered in substance by claims 23 and 24. Built (below); the interim is retired.
- Q-1335 (the mech locomotor): answered in substance by claim 22. Mechanical walking cannot stand in. Changed (`SeizeLocomotor::of`); the interim is retired.
- Q-1336 (claim 4 against another named locomotor): not answered. Interim kept.

**New requests** (block Q-1337 to Q-1345), each with an interim in the code and in the S-0607 ledger:
- Q-1337: claim 12 has no row for the drop-pod stand-in's grip end. Interim: the grip ends and the unit is set down, but the lifter's target is not cleared.
- Q-1338: whether a ground stand-in holds its cell's vehicle mark from the seize. Interim: claim 5's give-up runs; then the stand-in starts at rest holding its own cell's vehicle-present mark (S-0209 claim 1).
- Q-1339: a landed aircraft seized with a ground stand-in. Interim: it is seized, but it gets no stand-in movement and stays where it is.
- Q-1340: the teleporting stand-in's jump removes the destination (S-0207 claim 20). By claim 23 that ends the grip, but scenario L13 part b lists "a grip ended by arrival alone" as a falsifier. Interim: the specs are followed as written; the grip ends on the jump and the unit is set down.
- Q-1341: the warning for an empty warhead locomotor value. Under S-0001 claim 8 such a value makes no entry. Interim: an empty value reads as absent and does not warn; a malformed non-empty value warns.

Q-1342 to Q-1345 are unused.

## S-0607 revision 4

| Claim | Class | Notes |
|---|---|---|
| Scope, rules owned elsewhere, Terms (*stand-in locomotor*, *seized* until a restoration clears it) | (c) | |
| 5: for another named locomotor, claims 22 and 23 replace the jumpjet link and the destination bullet | changed | as claim 23. |
| 9: with the jumpjet stand-in its own locomotor waits for claim 20; other stand-ins follow claims 23 and 24 | changed | as claim 24. |
| 22: mechanical walking cannot stand in | changed | `SeizeLocomotor::of` maps `mech` to `NoStandIn` (Q-1335 interim retired). Tests: `claim_22_cannot_stand_in_does_not_seize` (now with `mech`), `claim_22_mechanical_walking_cannot_stand_in`. |
| 22: a stand-in locomotor is installed as the unit's stand-in | changed | `lift::install_stand_in`. Test: `claim_22_stand_in_seizes`. |
| 22: an absent key names the earlier layer's locomotor, else teleporting | satisfied | the importer's `fold` keeps the earlier layer's value; the sim reads `None` as teleporting (`SeizeLocomotor::default`). Tests: `lift_locomotor_malformed_keeps_the_earlier_layer`, `claim_22_stand_in_seizes` (absent key). |
| 22: an empty or malformed value is ignored as if absent (blanks trimmed); a seizing warhead warns, one without the flag does not | changed (importer) | `convert::sim_combat` warns for a seizing warhead whose value is not a class identifier. An empty value makes no entry (S-0001 claim 8), so it cannot warn: Q-1341. Tests: `lift_fields` (Bent, Bare warn; Blank, Padded, Quiet do not; Padded is jumpjet), `lift_locomotor_malformed_keeps_the_earlier_layer`. |
| 22: an unknown identifier (design) | satisfied | as built at step 1m. Test: `lift_fields` (`Odd`). |
| 23: at the seize the destination is the lifter object, set before *seized*; the stand-in gets claim 7's lifter point | changed | `lift::seize`, then `lift::instruct_stand_in` (drive and ship: the driving goal; teleport: `teleport::stand_in_instruction`). Tests: `claim_23_drive_stand_in_drives_to_the_lifter`, `claim_23_teleport_stand_in_jumps_to_the_lifter`. |
| 23: no movement point of its own (no claim-7 search) | changed | `jump::lift_point` runs only for the jumpjet. Test: `claim_23_drive_stand_in_drives_to_the_lifter` (the destination stays `Target::Object(lifter)`). |
| 23: driving under S-0207, teleporting under S-0207 claim 20, ship under S-0504 | changed | `tick` sends a unit with a moving stand-in to `lift::stand_in_tick` (`drive`, `teleport::tick`, `naval::tick`). Ship is coded but has no test on water. Tests: `claim_23_drive_stand_in_drives_to_the_lifter`, `claim_23_teleport_stand_in_jumps_to_the_lifter`. A landed aircraft: Q-1339. |
| 23: walking (design): no move, no turn | changed | `stand_in_tick` does nothing for walk. Test: `claim_23_walk_stand_in_stays`. |
| 23: drop pod (design): ends at once; own locomotor back; grip ended; set down | changed | `lift::drop_pod_ends`. Test: `claim_23_drop_pod_ends_at_once`. The lifter's target is kept: Q-1337. |
| 23: no arrival drop; the grip ends by claim 12's no-destination row when the stand-in's rules remove the destination; set down, not dropped | changed | `lift::enforce` runs after movement, at the start of the behaviour turn and in `econ::assign_destination`. Tests: `claim_23_drive_stand_in_drives_to_the_lifter` (S-0207 claim 25's give-up at an occupied lifter cell), `claim_23_no_destination_ends_the_grip`, `claim_23_teleport_stand_in_jumps_to_the_lifter` (the jump: Q-1340). |
| 23: claims 9 and 10 apply | changed | `lift::enforce` puts the destination back to the lifter while gripped, and to none after; firing is refused as before. Test: `claim_23_claims_9_and_10_apply`. |
| 23 (marks) | changed | claim 5's give-up applies to every stand-in; the stand-in's marks are released when it ends. Q-1338. Test: `claim_23_stand_in_marks_are_released`. |
| 24: drive, walk, ship, teleport never restored; drop pod gets its own locomotor back but stays seized (`lift_standin_stays_seized` on) | changed | `lift::restore_at_rest` returns early for a stand-in with the quirk on. Tests: `quirk_lift_standin_stays_seized_on`, `quirk_lift_standin_stays_seized_on_drop_pod`. |
| 24 fixed: claim 20 with the stand-in's own moving test; clears *seized*, *set down* and the dropper | changed | `lift::stand_in_moving`, `lift::end_stand_in` (own states and marks back). Test: `quirk_lift_standin_stays_seized_off`. |
| State table row, Edge cases (a non-jumpjet stand-in never lets go) | changed | as claim 24. |
| Import mapping (`Locomotor` row) | changed | as claim 22. |
| Save | changed | `LiftState.stand_in` is now the `StandIn` kind; new `LiftState.own` (`OwnLocomotor`). Test: `stand_in_save_round_trip`. |
| Scenario L13 parts a to c; backing, confidence; open questions | (c) | |

## S-0301 revision 23

| Claim | Class | Notes |
|---|---|---|
| 9, rank 4: grip *rearming* against the firer's own gripped unit outranks every other rank-4 answer except the materialising firer's *rearming* and the slave-master *illegal* | satisfied | as built at step 1m. `combat::check` tests `lift::grips` right after rank 2 *busy*. The materialising answer would give the same *rearming*, and our engine has no slave-master binding. The comment now cites S-0301 claim 9. Test: `claim_11_gripped_unit_rearming_with_any_weapon` (now cites S-0301 claim 9). |
| 9, rank 4: the beam's conditional *rearming* follows the target-layer and projectile tests | satisfied | as built (`lift::firing_answer` after the layer and projectile tests). Test: `claim_11_beam_answer_after_the_layer_tests` (now cites S-0301 claim 9). |
| Backing, confidence, scenario F7 part f, open questions | (c) | |

## S-0502 revision 13

| Claim | Class | Notes |
|---|---|---|
| 16: the point-assignment rule, with claim 23's search, runs once at a magnetic seize around the lifter's point; the point stays fixed | satisfied | built at step 1m (`jump::lift_point`). Tests: `claim_5_the_seize`, `claim_7_point_fixed_at_the_seize`. |
| 18: seized fall: target height 0, the rate afresh at each seize, the drop before the growth, single-precision narrowing | satisfied | built at step 1m (`jump::fly::physics`, `jump::lift_fall`, `lift::rate`). Test: `claim_15_crash_target_height_zero`. |
| 23: a magnetic seize runs the claim-16 search on the lifter's point (non-balloon vehicle row) | satisfied | as claim 16. |
| Constants table row, backing, magnetic-lift coverage | (c) | |

## S-0215 revision 68

| Claim | Class | Notes |
|---|---|---|
| 11: readiness asks the vehicle's current locomotor; a temporary replacement's report applies; the driving predicate is for a vehicle whose current locomotor is the driving one | changed | `process::driving` reads `lift::current_locomotor`. A jumpjet-seized vehicle uses the jumpjet's report; a driving stand-in counts as driving; S-0417 claim 5's temporary drive is kept. Tests: `s0215_claim_11_current_locomotor`, `claim_5_seized_turning_vehicle_is_ready`. |
| 11: backing, confidence | (c) | |

## S-0207 revision 40

| Claim | Class | Notes |
|---|---|---|
| 14: a seizing warhead's locomotor field uses the same class-identifier parse | satisfied | `convert::sim_combat` reads it with `class_identifier` (blanks trimmed). Test: `lift_fields` (Padded). |

## Glossary revision 57

| Entry | Class | Notes |
|---|---|---|
| *seized*: until a restoration clears it; a drop-pod stand-in can end while the unit stays seized | (c) | built as S-0607 claim 24. |
| *seizing warhead*: claims 22 to 24 | (c) | |
| *stand-in locomotor* (new) | (c) | |

## Formats and gate

The save format is 57 (from 56): `LiftState.stand_in` is now the `StandIn` kind and `LiftState.own` is new. The AI game format stays 23, because the brain did not change. All eight goldens were regenerated. The seed sweep and the gate seed are under *Gate* below.

## Gate

Seed sweep 1 to 16 after the step (`GLASNOST_SWEEP=1-16`). The counters are the same as at step 1m to the last digit. The fixture's lifter warhead names the jumpjet locomotor explicitly, so the new stand-in rules do not run in the duel.

| Seed | Ticks | Result | Counters below 3 |
|---|---|---|---|
| 1 | 12 000 | none | sensor orders 2 |
| 2 | 12 000 | none | sensor orders 2 |
| 3 | 11 222 | Alpha wins | sensor orders 2 |
| 4 | 12 000 | none | sensor orders 2, bunker exit orders 2 |
| 5 | 12 000 | none | sensor orders 2 |
| 6 | 11 253 | Alpha wins | sensor orders 2 |
| 7 | 11 360 | Alpha wins | sensor orders 2 |
| 8 | 12 000 | none | sensor orders 2, gattling stage rises 0, top-stage ticks 0, gas clouds 0, gas hits 0 |
| 9 | 12 000 | none | sensor orders 2 |
| 10 | 12 000 | none | sensor orders 2 |
| 11 | 12 000 | none | sensor orders 2 |
| 12 | 10 384 | Alpha wins | sensor orders 2 |
| 13 | 10 662 | Alpha wins | sensor orders 2, vehicles bunkered 1, bunker exit orders 1 |
| 14 | 12 000 | none | sensor orders 2 |
| 15 | 12 000 | none | sensor orders 2 |
| 16 | 11 635 | Alpha wins | sensor orders 2 |

**`M7A_SEED` moved from 3 to 1.** Seed 1 is the lowest seed that plays all 12 000 ticks and meets every minimum except the psychic sensor's known shortfall of 2. The goldens were regenerated with it, and the gate run passes without the update flag.

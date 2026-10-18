# M7a step 1m claim ledger (S-0607 revision 3, S-0108 revision 15, glossary 56)

This is part 13 of M7a step 1 (intake 8). It covers the approved revisions that reached main after the step 1l intake, from 615071c to f357d50: S-0607 revision 3, S-0108 revision 15, the glossary at revision 56 and the index. Every changed claim gets one class, as in [step1l-m7a.md](step1l-m7a.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered. S-0607 gains claim 22 (section *Other locomotors*) and the quirk `lift_impact_double_credit`.

| Spec | Revision | Answers |
|---|---|---|
| S-0607 magnetic lift | 3 (from 2) | Q-1320, Q-1322 to Q-1331; Q-1321 in substance (claim 22), though still `open` |
| S-0108 building sprites | 15 (from 14) | none |
| glossary | 56 (from 55) | none |
| `spec/README.md` | | index rows of S-0108 and S-0607 moved and revised: (c) |
| `spec/requests/` Q-1320, Q-1322 to Q-1331 (answered) | | request status: (c) |

**Retired interims.** Every one is gone from the code, the tests, `development.md` and the S-0607 ledger.
- Q-1320 (the pull point) is answered by claim 7, not as the interim had it. The interim pulled towards the lifter's exact position, refreshed each update. The answer is a movement point fixed at the seize: S-0502 claim 16's search around the lifter's point. Changed (below).
- Q-1322 (the starting crash rate) is answered by claim 15 as the interim had it (the type's crash value narrowed to single precision), plus two rules: drop before growth (as built) and a fresh start at every seize (as built, now tested).
- Q-1323 (blocked outcomes) is answered by claim 17 as the interim had it: every S-0209 entry outcome but *clear* blocks, and the crushable rows VR2 and VR3 give *clear*. As built (`vehicle_outcome`).
- Q-1324 (stunned) is answered by claim 17 as the interim had it: the stun repeats claim 13's stop on dropping and adds nothing beyond sinking.
- Q-1325 (no death weapon) is answered by claim 18's design rule as the interim had it (a plain area detonation of the warhead), plus an importer warning, which is new.
- Q-1326 (F3's negative sum) is answered by F3, not as the interim had it: *r*(*n*) = *r*(−*n*), so the root takes the magnitude. Changed.
- Q-1327 (the wave's update) is answered by claim 21, not as the interim had it: the wave joins the update order at creation and takes its own turn. Changed.
- Q-1328 (the rearming rank) is answered by claim 11: against its gripped unit the lifter answers *rearming* with any weapon, ahead of the target's layer, height, kind, armour and projectile tests. Changed.
- Q-1329 (restoration in guard; the deck landing) is answered by claim 20 (only at rest; the two-order sequence) and the Edge cases (the fixed survivor is put on the surface or the deck). As built; claim 20's two-order sequence is newly tested.
- Q-1330 (open cases) is answered as follows. The order of claims 3 and 4 is in State and ordering, as the interim had it. Deploying and unloading are not refused (claim 10, newly tested). The passengers of a dropped transport are covered by claim 18. "Leaves play" for the wave includes going inside a transport, bunker or building (claim 21, changed).
- Q-1331 (readiness of a seized driving vehicle) is answered by claim 20 as the interim had it (S-0502's *moving now*).

**Q-1321** (a seizing warhead naming another locomotor, or none) is still `open`. Claim 22 answers it in substance for a named locomotor and an unknown identifier, and claim 22 is built. The request file is not edited. Its interim is kept only for what revision 3 still leaves open: an empty or malformed value (Open questions). That case keeps the value already in force and warns.

**New requests** (block Q-1332 to Q-1339), each with an interim in the code and in the S-0607 ledger:
- Q-1332: what claim 13's stop on dropping takes from S-0502 claim 17's rule. Interim: the reservation is released; there is no search and no self-destruct.
- Q-1333: which building types "service docking units" (claim 7). Interim: the repair depot and the vehicle bunker, with the docking point each would give the seized unit.
- Q-1334: how a unit seized with a stand-in locomotor moves and is restored (claim 22). Interim: it keeps its own movement state and marks and stays where it stands; claims 9 and 10 apply; claim 20 restores it.
- Q-1335: claim 22 does not class the walking-vehicle (mech) locomotor. Interim: it stands in.
- Q-1336: claim 4 against a warhead naming another locomotor. Interim: claim 4's special case comes first, whatever the locomotor.

Q-1337 to Q-1339 are unused.

## S-0607 revision 3

| Claim | Class | Notes |
|---|---|---|
| Terms: movement point, heading point; claim 22 for other locomotors | (c) | |
| 5: the jumpjet locomotor is linked (claim 22 for others); the destination becomes the lifter, turned at once into a fixed movement point, with the order-level destination the cell of that point | changed | `lift::seize` calls `lift::lifter_point` and `jump::lift_point`. `o.dest` is the cell. Test: `claim_5_the_seize` (the cell is free and next to the lifter's). |
| 7: the movement point is fixed at the seize by S-0502 claim 16's search around the lifter's point (heading point for a mobile lifter, foundation centre for a building, the docking point for a docking-service building); no cell found: the point itself; not refreshed | changed | `jump::lift_point`; `jump::pull` and the per-update refresh are gone; `lift::steer` only keeps the destination on the fixed cell. A building's docking point: Q-1333. Tests: `claim_5_the_seize`, `claim_7_point_fixed_at_the_seize` (the lifter drives away; the point stays). |
| 8: arrival (strictly under 20 leptons of the movement point) places the unit on it and drops it over that cell, normally beside the lifter, so it lands unhurt | changed | follows from claim 7. Test: `claim_8_arrival_drops_beside_the_lifter` (replaces `claim_8_arrival_drop_on_the_lifter`). The controlled-vehicle pair now drops its victim onto a blocker after a lifter order (`a_lifted_controlled_vehicle_dies_and_frees_its_controllers_slot`). |
| 10: deploying, undeploying and unloading are not refused by being seized | satisfied (newly tested) | no seized-state test exists in those paths. Test: `claim_10_set_down_seized_unit_deploys` (set down, still seized, a deploy order starts the in-place cycle). |
| 11: against its gripped unit, *rearming* with any weapon, ahead of every layer, height, kind, armour and projectile test; the lifter keeps the unit as its target | changed | `combat::check` tests `lift::grips` right after the busy test; the old beam-only answer after the projectile tests is gone. Tests: `claim_11_rearming_while_gripping`, `claim_11_gripped_unit_rearming_with_any_weapon` (fails without the change). |
| 11: the magnetic beam's answer comes after the layer and projectile tests | satisfied | as built. |
| 13: the stop on dropping (destination, route, target cleared, contact broken off, deselected; the jumpjet stopped by S-0502 claim 17's rule) | changed | `lift::drop` clears the target and burst, breaks off contact and announces `Effect::Dropped` (the client deselects). `jump::lift_dropped` releases the reservation (Q-1332). Test: `claim_13_stop_on_dropping`. |
| 15: starting rate, drop before growth, a fresh start at every seize | satisfied | as built under Q-1322. Test: `claim_15_crash_target_height_zero` (second seize). |
| 15: target height 0 while seized and crashing | changed | `jump::fly::physics` and `jump::lift_fall`. Test: `claim_15_crash_target_height_zero` (fails without the change). |
| 17: blocked is any outcome but *clear*; VR2 and VR3 give *clear* | satisfied | as built under Q-1323 (`vehicle_outcome`). |
| 17, sinking row: the stun adds nothing | satisfied | as built under Q-1324. Test: `claim_17_blocked_water_sinks`. |
| 18: no death weapon (design) | satisfied; importer changed | the plain area detonation was built under Q-1325. The importer now warns when `[CombatDamage] DeathWeapon` is absent. Tests: `lift_fields_absent` (the warning), `lift_death_weapon_given` (none). |
| 18: credit when a detonation kills the falling unit (`quirks.lift_impact_double_credit`, default on: a second credit; off: skipped) | changed | `lift::impact` and `lift::impact_credit`. A dead unit is credited from its snapshot. Test: `quirk_lift_impact_double_credit_both_positions` (losses, kills and score doubled on, single off). |
| 18: a survivor removed under `lift_impact_removes_survivor` is credited once | changed | `lift::impact`. Test: `claim_18_removed_survivor_credited_once`. |
| 18: passengers of a killed transport die under the ordinary rules; a removed survivor's passengers leave without attacker credit | satisfied | as built (`death::passenger_death`; S-0507 claim 16). Not separately tested. |
| 20: only at rest; the first order after touchdown is refused and restores, the second moves | satisfied (newly tested) | as built under Q-1329. Test: `claim_20_first_order_restores_second_moves`. |
| 20: readiness while seized is the jumpjet's *moving now* | satisfied | as built under Q-1331. Test: `claim_5_seized_turning_vehicle_is_ready`. |
| 21: leaving play includes going inside a transport, bunker or building | changed | `lift::wave_turn`'s end test reads `in_play`. Test: `claim_21_firer_off_the_map_ends_the_wave`. The bunker pair now detects firing through the projectile (a wave at a bunkered target ends at once): `a_lift_attack_on_a_bunkered_vehicle_goes_to_its_bunker`. |
| 21: the wave joins the update order at creation, appended; one update at creation; later its own turn; the retarget is seen at once | changed | `World.waves` (a map keyed by a new object id), `w.update.enter` and `leave`, an `object_turn` arm in `tick`. Tests: `claim_21_wave_in_the_update_order`, `f4_wave_fade`. |
| 21: F3 is evaluated once, at creation, from the shot's points | satisfied | as built (`lift::shot`). |
| F3: a wrapped negative sum takes *r*(−*n*) | changed | `table_root(unsigned_abs)`. Test: `f3_wrapped_negative_sum`. |
| 22: a locomotor that cannot stand in (hover, fly, rocket, tunnel) or an unknown identifier: claim 5's other effects, the previous victim released, no seize | changed | `SeizeLocomotor::NoStandIn`, `lift::not_seized`. Tests: `claim_22_cannot_stand_in_does_not_seize`, `claim_22_cannot_stand_in_releases_the_previous_victim`. |
| 22: one that can stand in (drive, walk, ship, teleport, drop pod; an absent key is teleport): seized with that locomotor | changed | `SeizeLocomotor::StandIn`, `lift::seize(.., stand_in)`, `lift::enforce`. How it moves and is restored: Q-1334; the mech locomotor: Q-1335. Test: `claim_22_stand_in_seizes`. |
| 22: an unknown identifier warns at import (design) | changed (importer) | `convert::sim_combat`. Test: `lift_fields` (`Odd` → unknown, with a warning). |
| State and ordering: the previous victim released even when the new target is immune; an immune gatherer keeps its temporary locomotor; claim 4's case loses spawned objects and the parasite | satisfied | `claim_2_new_hit_at_an_immune_target_drops`, `claim_4_temporary_harvester`. |
| State table rows (movement point, drop beside the lifter) | (c) | follow claims 7 and 8. |
| Edge cases: the fixed survivor is put on the surface or deck, grounded, set down | satisfied | as built under Q-1329. Test: `quirk_lift_impact_removes_survivor_both_positions`. |
| Edge cases: double impact credit | changed | as claim 18. |
| Import mapping rows (`Locomotor`, the crash value, `DeathWeapon`) | changed / satisfied | as claims 15, 18 and 22. |
| Scenarios L2, L4, L9 f and g, L10, L11 c and d, L12, L13; backing, confidence; open questions | (c) | |

## S-0108 revision 15

| Claim | Class | Notes |
|---|---|---|
| 5: a sentence about later build-up loading dropped; confidence reworded | (c) | |
| 20: backing now cites S-0603 claim 12 and scenario G10 for natural completion; two restart cases `unmeasured` | (c) | no code or test cites S-0108 claim 20, so no citation points at a wrong claim. |
| Open questions (fence posts, wall-tower upgrades, F1's rounding fixture) reworded | (c) | |

## Glossary revision 56

| Entry | Class | Notes |
|---|---|---|
| *seized*: distinct from *held*, the parasite countdown | (c) | |
| *seizing warhead*: claim 22 owns the other named locomotors | (c) | |

## Formats and gate

The save format is 56 (from 55): `World.waves` replaces `LiftState.wave`, `Wave` gains its firer, and `LiftState` gains `stand_in`. The AI game format stays 23: the brain did not change. Goldens and the seed sweep: see the S-0607 section of `development.md`.

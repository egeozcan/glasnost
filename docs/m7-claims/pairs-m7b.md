# M7b interaction pairs

PLAN §M7 asks for a scenario test on the synthetic ruleset for every interaction pair of the Yuri
faction. This ledger lists the M7b pairs, the test for each, and the pairs still waiting for their
spec. Every test lives in `crates/deterministic/sim/tests/pairs_m7b.rs`, uses invented types, cites
the claims of both sides, and checks that the state hash survives a save/load round trip.

## Pairs with a scenario test

| Pair | Test | Specs |
|---|---|---|
| Special-hit effects × the lift fall: a fall ending on infantry crushes it with no death animation, whatever the crushing warhead's kind; a fall ending on a vehicle detonates, and the impact kills infantry in the next cell, which becomes a burning walker where it died | `a_lift_fall_crushes_silently_but_its_impact_makes_a_burning_walker` | S-0615, S-0607, S-0303 |
| Special-hit effects × gas deaths: an ordinary victim keeps its virus-death animation and no special animation; a parachutist killed by a cloud explodes at its position (kind 3), gets one cloud at once, and its extra virus animation is removed at once | `a_gas_cloud_kills_a_parachutist_with_one_cloud_and_an_explosion` | S-0615, S-0613, S-0107 |
| Drain × power: a drained power plant blacks its house out at the next working-out, so a needs-power defence with a drain stops; the disc's killing hit ends the link and the defence is back in operation | `drain_pairs::a_drained_plant_stops_the_defences_until_the_disc_dies` | S-0608, S-0405 |
| Drain × absorber: an empty absorber with no base output, drained, does not black its house out; with a passenger that walked in while drained (extra output) it does, and ending the link restores the output; the absorber refuses the balloon-hovering disc | `drain_pairs::a_drained_absorber_blacks_out_only_with_passengers` | S-0608, S-0405, S-0605 |
| Drain × grinder: a drained grinder of a resource-destination type pays its house's credits to the drainer's house in payouts of *A*, cannot be sold, and an own grinder refuses the balloon-hovering disc | `drain_pairs::a_drained_grinder_pays_and_cannot_be_sold` | S-0608, S-0603, S-0408 |
| Drain × tank bunker: a drained bunker's firing answer is *illegal* and it cannot be sold, but without output it blacks nothing out; a bunker refuses the balloon-hovering disc (row P6) | `drain_pairs::a_drained_bunker_refuses_fire_and_sale_but_no_blackout` | S-0608, S-0606, S-0405, S-0408 |
| Drain × tank bunker occupant: a bunkered vehicle under a drained bunker keeps its own firing answer (not the bunker's *illegal*) and its range bonus, and is not ejected | `drain_pairs::a_drained_bunker_keeps_its_vehicle_firing` | S-0608 claim 18, S-0606 |
| Drain × grinder recycling: a drained grinder accepts an infantry entrant and consumes it; the recycling credit goes to the grinder's owner and the drainer gets only payouts | `drain_pairs::a_drained_grinder_still_recycles_for_its_owner` | S-0608 claim 18, S-0603 |
| Genetic Mutator × mind control: a cell-mode launch converts a controlled unit, which leaves its controller's list; the made unit is the launching house's and inherits no control; a converted controller releases its other victims to their owners | `mutator_pairs::a_mutated_controlled_unit_and_a_mutated_controller` | S-0611, S-0601 |
| Genetic Mutator × tank bunker: a launch on a bunker hits neither the bunker nor its bunkered vehicle, which stays bunkered; infantry beside it in the block converts | `mutator_pairs::a_launch_on_a_tank_bunker_leaves_it_and_its_vehicle` | S-0611, S-0606 |
| Genetic Mutator × absorber: a passenger inside is not listed and is not hit; a walker still on its way in converts and its entry ends | `mutator_pairs::a_launch_on_an_absorber_spares_its_passenger_and_takes_an_entrant` | S-0611, S-0605 |
| Genetic Mutator × magnetic lift: a launch under a seized vehicle leaves it unhurt and gripped; infantry on the ground beneath converts | `mutator_pairs::a_launch_under_a_seized_vehicle_takes_only_the_infantry` | S-0611, S-0607 |
| Psychic Dominator × mind control: a controlled unit in the swept area is released by its controller outside it, then taken over permanently; a controller in the area is taken over and keeps its victim outside it | `dominator_pairs::a_domination_takes_a_controller_and_a_controlled_unit` | S-0612, S-0601 |
| Psychic Dominator × tank bunker: the vehicle in a two by two bunker is its cell's start object and is taken over; the bunker is not; the bunker link stays | `dominator_pairs::a_domination_takes_a_bunkered_vehicle_and_leaves_the_link` | S-0612, S-0606 |
| Psychic Dominator × absorber: the absorbed passenger is not listed and keeps its owner; infantry on the map beside the building is taken over; the building is not | `dominator_pairs::a_domination_spares_an_absorbed_passenger` | S-0612, S-0605 |
| Force Shield × mind control (and Cloning Vats): a shielded vat cannot be taken; the controller's firing check is *illegal* and a landing shot takes nothing, while the same landing takes an unshielded vat; the launching house is blacked out and its shielded vat still copies its released infantry | `shield_pairs::a_shielded_vat_refuses_mind_control_and_keeps_cloning` | S-0513, S-0601, S-0404, S-0405 |
| Force Shield × gattling: a player house's gattling fires at a shielded building and spins up as against an open one; its hits do nothing, while the open building is hurt | `shield_pairs::a_gattling_fires_at_a_shielded_building_and_spins_up_for_nothing` | S-0513, S-0609, S-0302 |
| Force Shield × gas clouds: a cloud's hits do nothing to a shielded building and hurt an unshielded one | `a_gas_cloud_does_nothing_to_a_shielded_building` | S-0513, S-0613, S-0302 |
| Force Shield × Psychic Dominator: the sweep's area damage spares the shielded building and hurts one just outside the radius; infantry within the radius is not shielded and is taken; no building is taken | `shield_pairs::a_domination_over_a_shielded_base_takes_its_units_and_spares_its_buildings` | S-0513, S-0612, S-0302 |
| Force Shield × Genetic Mutator: infantry within the radius is not shielded and converts; the made unit is the launching house's | `shield_pairs::a_mutation_inside_a_shield_converts_its_infantry` | S-0513, S-0611 |
| Force Shield × magnetic lift: a vehicle within the radius is not shielded and is seized | `shield_pairs::a_lift_seizes_a_vehicle_inside_a_shield` | S-0513, S-0607 |
| Force Shield × drain (S-0608 claim 19, answering Q-1507): the drainer links to a shielded power plant and the link holds while it is protected | `drain_pairs::a_shielded_plant_is_drained` | S-0513, S-0608 |
| Cloning Vats × mind control: a controlled vat copies for its controller's house and not for its owner; released back, it ranks after the owner's other vat, so the order of copies flips | `cloning_pairs::a_controlled_vat_clones_for_its_controller_and_ranks_last_when_released` | S-0404, S-0601 |
| Cloning Vats × Genetic Mutator: the unit a conversion makes is not copied; a factory release is | `cloning_pairs::a_mutation_made_unit_is_not_cloned` | S-0404, S-0611 |
| Cloning Vats × absorber: a passenger that the deploy order lets out is not copied | `cloning_pairs::a_passenger_let_out_of_an_absorber_is_not_cloned` | S-0404, S-0605 |
| Cloning Vats × Psychic Dominator: a vat in the swept area stays its owner's and keeps copying for it | `cloning_pairs::a_domination_leaves_a_vat_with_its_owner` | S-0404, S-0612 |

The parachutist's order of the virus animation and the explosion was request Q-1421; S-0615 revision 3
answers it (the virus effect first, then the kind 3 or 5 animation), and the test now checks the order
(M7b step 1c).

What a drained bunker, absorber or grinder does with its occupants and entrants was request Q-1444.
S-0608 revision 5 claim 18 answers it (no occupant gate), taken in at M7b step 1d: the absorber pair
now lets the walker in while drained, and two pairs check a bunkered vehicle's fire and the grinder's
recycling during a link.

The Force Shield and Cloning Vats rows are M7b step 4a. The shield is launched with
`superweapons::protect::force_shield`, never by writing protection on an object. It protects only
the buildings of houses allied with the launching house (S-0513 claim 31), so the shield rows
check its buildings, and the infantry and vehicles inside the radius as unprotected. A drain on a
protected building was request Q-1507; S-0608 revision 6 claim 19 and S-0513 revision 19 claim 27
answer it with the reading the interim had (the link forms and holds), so the test now cites
claim 19 (M7b step 1f). The scenario wording of S-0601 (a unit under a house's force
shield) reads as if a shield protected units; claim 31 says it does not, and the tests follow
claim 31.

## Pairs without a test

No approved spec states a relation for these pairs; each side runs by its own spec.

| Pair | Why no relation |
|---|---|
| Force Shield × berserk | S-0302 claim 10 gives a berserk hit no effect on a building, and S-0610 claim 3 refuses immune building types, so a shield changes nothing. |
| Force Shield × absorber, grinder and tank-bunker entry | No approved spec states the relation. Protection refuses damage and claim 27's list; entry into a shielded building is not in it. |
| Force Shield × psychic sensor and reveal | No approved spec states the relation. |
| Cloning Vats × drain and low power | No approved spec states a relation beyond power: S-0404 claim 14 has no power gate and S-0405 claim 9 does not stop factories for low power. The first shield row shows a blacked-out house's vat still copying. |
| Cloning Vats × gattling, berserk, gas, lift | No approved spec states the relation: copies are ordinary infantry once released, victims like any other. |

## Pairs left

None. Every pair of PLAN §M7's list has a scenario test above, a row in this table, or (the drain on
a protected building) a test of a request's interim.

## Local relation tests on the reference ruleset

PLAN §M7 also asks for a local relation test on the reference ruleset for each pair. Step 4b adds
`crates/tools/tests/relation_lane.rs`: one ignored test for each pair row of this ledger (26) and
of [pairs-m7a.md](pairs-m7a.md) (16). Each test loads the installed rules through
`arena::reference_rules`, finds the types it needs by their flags (a controller by its warhead, a
lifter by its locomotor warhead, a drainable plant, a tank bunker and so on, never by name),
builds the same scene as the synthetic test, and asserts only the direction of the relation. It
prints `relation <test>: PASS`, `PASS (<what was not formed>)` or `SKIP <reason>`, never a type
name or a value. Without `GLASNOST_GAME_DIR` every test prints `SKIP no install`. Run it with:

```
GLASNOST_GAME_DIR=<your install> cargo test --release -p glasnost-tools --test relation_lane -- --ignored --nocapture --test-threads=1
```

The first run on the user's install failed 12 tests. Every failure was in the test's scene, not
in the simulation: the type it chose, when it read the result, or a precondition that the
installed data does not give. Each was checked against the spec before the scene was changed. For
example, S-0612 claim 10 puts the sweep's damage before the takeover, and S-0607 claims 17 and 20
let a unit dropped on a clear cell survive, still seized. Where the sweep's damage destroys the
building of a pair before the takeover, the test runs again with the domination's damage set to 0
and says so.

The second run took 2.6 s once built and ran 42 tests: 37 pass and 5 skip, none fail. Six of the
passes are partial (`PASS (...)`). The five skips are not results: the installed data cannot form
those pairs. The run prints the reason for each partial pass and skip; the reasons describe the
installed data, so they stay in the git-ignored run output and out of this file.

| Pair | Relation test | Result on the user's install |
|---|---|---|
| Psychic (berserk) × bunker | `m7a_a_berserk_weapon_is_refused_at_a_bunkered_vehicle_and_its_bunker` | PASS |
| Mind control × absorber | `m7a_a_controlled_enemy_goes_into_an_absorber_and_comes_out_an_enemy` | PASS |
| Grinding a controlled unit (PLAN) | `m7a_a_controlled_enemy_is_ground_for_its_controllers_house` | PASS |
| Mind control × bunker | `m7a_a_controlled_vehicle_goes_into_its_controllers_bunker` | PASS |
| A controlled unit in a transport (PLAN) | `m7a_a_controller_riding_an_open_transport_takes_and_loses_a_victim` | PASS |
| Lift × parasite | `m7a_a_drone_stays_on_a_seized_host` | PASS |
| Gattling × bunker | `m7a_a_gattling_in_a_bunker_spins_up_as_in_the_open` | PASS |
| Mind control × gattling | `m7a_a_gattling_taken_over_while_firing_keeps_its_stage` | PASS |
| Lift × bunker | `m7a_a_lift_attack_on_a_bunkered_vehicle_goes_to_its_bunker` | PASS |
| Lift × mind control | `m7a_a_lifted_controlled_vehicle_dies_and_frees_its_controllers_slot` | PASS |
| Mind control fate × absorber | `m7a_a_released_neutral_unit_takes_the_bio_reactor_fate` | PASS |
| Mind control fate × grinder | `m7a_a_released_neutral_unit_takes_the_grinder_fate` | PASS |
| Lift × gattling | `m7a_a_seized_gattling_spins_down` | PASS |
| Lift × grinder | `m7a_a_seized_vehicle_ignores_its_owners_recycle_click` | PASS |
| Warp × bunker | `m7a_a_warp_takes_a_bunkered_vehicle_out_of_its_bunker` | PASS |
| Mind control × orders | `m7a_an_order_of_the_former_owner_still_applies_in_the_takeover_tick` | PASS |
| Force Shield × gas clouds | `m7b_a_gas_cloud_does_nothing_to_a_shielded_building` | PASS |
| Special-hit effects × gas deaths | `m7b_a_gas_cloud_kills_a_parachutist_with_one_cloud_and_an_explosion` | PASS |
| Special-hit effects × the lift fall | `m7b_a_lift_fall_crushes_silently_but_its_impact_makes_a_burning_walker` | PASS |
| Cloning Vats × mind control | `m7b_cloning_a_controlled_vat_clones_for_its_controller_and_ranks_last_when_released` | SKIP (the installed data cannot form the pair; the run prints why) |
| Cloning Vats × Psychic Dominator | `m7b_cloning_a_domination_leaves_a_vat_with_its_owner` | PASS, partial (part of the scene not formed on the installed data; the run prints which) |
| Cloning Vats × Genetic Mutator | `m7b_cloning_a_mutation_made_unit_is_not_cloned` | PASS |
| Cloning Vats × absorber | `m7b_cloning_a_passenger_let_out_of_an_absorber_is_not_cloned` | PASS |
| Psychic Dominator × absorber | `m7b_dominator_a_domination_spares_an_absorbed_passenger` | PASS, partial (part of the scene not formed on the installed data; the run prints which) |
| Psychic Dominator × tank bunker | `m7b_dominator_a_domination_takes_a_bunkered_vehicle_and_leaves_the_link` | PASS, partial (part of the scene not formed on the installed data; the run prints which) |
| Psychic Dominator × mind control | `m7b_dominator_a_domination_takes_a_controller_and_a_controlled_unit` | PASS, partial (part of the scene not formed on the installed data; the run prints which) |
| Drain × absorber | `m7b_drain_a_drained_absorber_blacks_out_only_with_passengers` | PASS, partial (part of the scene not formed on the installed data; the run prints which) |
| Drain × tank bunker occupant | `m7b_drain_a_drained_bunker_keeps_its_vehicle_firing` | SKIP (the installed data cannot form the pair; the run prints why) |
| Drain × tank bunker | `m7b_drain_a_drained_bunker_refuses_fire_and_sale_but_no_blackout` | SKIP (the installed data cannot form the pair; the run prints why) |
| Drain × grinder | `m7b_drain_a_drained_grinder_pays_and_cannot_be_sold` | SKIP (the installed data cannot form the pair; the run prints why) |
| Drain × grinder recycling | `m7b_drain_a_drained_grinder_still_recycles_for_its_owner` | SKIP (the installed data cannot form the pair; the run prints why) |
| Drain × power | `m7b_drain_a_drained_plant_stops_the_defences_until_the_disc_dies` | PASS |
| Force Shield × drain (S-0608 claim 19) | `m7b_drain_a_shielded_plant_is_drained` | PASS |
| Genetic Mutator × tank bunker | `m7b_mutator_a_launch_on_a_tank_bunker_leaves_it_and_its_vehicle` | PASS |
| Genetic Mutator × absorber | `m7b_mutator_a_launch_on_an_absorber_spares_its_passenger_and_takes_an_entrant` | PASS |
| Genetic Mutator × magnetic lift | `m7b_mutator_a_launch_under_a_seized_vehicle_takes_only_the_infantry` | PASS |
| Genetic Mutator × mind control | `m7b_mutator_a_mutated_controlled_unit_and_a_mutated_controller` | PASS |
| Force Shield × Psychic Dominator | `m7b_shield_a_domination_over_a_shielded_base_takes_its_units_and_spares_its_buildings` | PASS |
| Force Shield × gattling | `m7b_shield_a_gattling_fires_at_a_shielded_building_and_spins_up_for_nothing` | PASS |
| Force Shield × magnetic lift | `m7b_shield_a_lift_seizes_a_vehicle_inside_a_shield` | PASS |
| Force Shield × Genetic Mutator | `m7b_shield_a_mutation_inside_a_shield_converts_its_infantry` | PASS |
| Force Shield × mind control (and Cloning Vats) | `m7b_shield_a_shielded_vat_refuses_mind_control_and_keeps_cloning` | PASS, partial (part of the scene not formed on the installed data; the run prints which) |

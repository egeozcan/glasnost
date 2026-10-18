# M7a interaction pairs (step 5)

PLAN §M7 asks for a scenario test on the synthetic ruleset for every interaction pair of the Yuri
faction. This ledger lists the pairs that the M7a specs (S-0610, S-0601, S-0609, S-0613, S-0605,
S-0603, S-0606, S-0607) reach, the test for each, and the pairs left for M7b. Every test lives in
`crates/deterministic/sim/tests/pairs_m7a.rs`, uses invented types, cites the claims of both
sides, and those that create new state check that the state hash survives a save/load round trip.

## Pairs with a scenario test

| Pair | Test | Specs |
|---|---|---|
| A controlled unit in a transport (PLAN): a controller with no victims boards an open-topped transport, takes a victim by passenger fire, and the victim is freed when the transport dies with its passengers | `a_controller_riding_an_open_transport_takes_and_loses_a_victim` | S-0601, S-0507, S-0213, S-0312 |
| Mind control × gattling: taken over while firing, the stage is kept | `a_gattling_taken_over_while_firing_keeps_its_stage` | S-0601, S-0609 |
| Mind control × orders: the former owner's order applies in the takeover tick | `an_order_of_the_former_owner_still_applies_in_the_takeover_tick` | S-0601, S-0203, S-0213 |
| Mind control × absorber: a controlled enemy goes in and comes out an enemy | `a_controlled_enemy_goes_into_an_absorber_and_comes_out_an_enemy` | S-0601, S-0605, S-0214 |
| Mind control fate × absorber: a released neutral unit enters the nearest bio reactor | `a_released_neutral_unit_takes_the_bio_reactor_fate` | S-0601, S-0605, S-0215 |
| Grinding a controlled unit (PLAN): paid to the controller's house, the slot freed | `a_controlled_enemy_is_ground_for_its_controllers_house` | S-0601, S-0603, S-0214 |
| Mind control fate × grinder: a released neutral unit is ground | `a_released_neutral_unit_takes_the_grinder_fate` | S-0601, S-0603 |
| Gattling × bunker: spins up as in the open | `a_gattling_in_a_bunker_spins_up_as_in_the_open` | S-0609, S-0606 |
| Mind control × bunker: a controlled vehicle enters its controller's bunker | `a_controlled_vehicle_goes_into_its_controllers_bunker` | S-0601, S-0606 |
| Warp × bunker: the bunker link ends in place | `a_warp_takes_a_bunkered_vehicle_out_of_its_bunker` | S-0513, S-0606 |
| Psychic (berserk) × bunker: refused at the vehicle and its bunker | `a_berserk_weapon_is_refused_at_a_bunkered_vehicle_and_its_bunker` | S-0610, S-0606 |
| Lift × mind control: a lifted controlled vehicle dies, the slot is freed | `a_lifted_controlled_vehicle_dies_and_frees_its_controllers_slot` | S-0607, S-0601 |
| Lift × bunker: the attack is redirected to the bunker and seizes nothing | `a_lift_attack_on_a_bunkered_vehicle_goes_to_its_bunker` | S-0607, S-0606, S-0309 |
| Lift × gattling: a seized gattling spins down | `a_seized_gattling_spins_down` | S-0607, S-0609 |
| Lift × grinder: a seized vehicle ignores its owner's recycle click | `a_seized_vehicle_ignores_its_owners_recycle_click` | S-0607, S-0603, S-0214 |
| Lift × parasite: a drone stays on a seized host | `a_drone_stays_on_a_seized_host` | S-0607, S-0614 |

The two pairs PLAN names for M7a, *a controlled unit in a transport* and *grinding a controlled
unit*, are the first and sixth rows. Pairs within one spec (for example a gas cloud over a
garrisoned building, the overload of a controller, a slave miner's slaves under takeover) are
claim tests in that spec's `claims_s06xx*.rs` file and are listed in its ledger, not here.

## Pairs left for M7b

The other half of PLAN §M7's list is not built yet; each pair below waits for its spec (named
seams in the code).

- Floating Disc (S-0608 drain) with the bunker, the absorber, the grinder and power: `approach::draining`.
- Genetic Mutator (S-0611) with controlled, bunkered, absorbed and seized units.
- Psychic Dominator (S-0612) with controlled units, bunkers (`bunker::holds_through_owner_change`) and the absorber.
- Special-hit effects (S-0615) with the lift fall and gas deaths: built at M7b step 3.2, see `pairs-m7b.md`.
- Cloning Vats, and the Force Shield against the M7a weapons.

## Local relation tests on the reference ruleset

PLAN §M7 also asks for a local relation test on the reference ruleset for each pair. M7a has none
yet. The local lane (`crates/app/tests/m7a_lane.rs`, scenes for berserk, the psychic sensor
and reveal, mind control and overload, gattling spin, gas clouds, the absorber, the grinder, the
bunker and the magnetic lift) exercises each ability on the installed data, one at a time,
and the AI lane plays them together, but neither checks a pair. This is carried to M7b.

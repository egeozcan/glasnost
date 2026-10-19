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
| Drain × absorber: an empty absorber with no base output, drained, does not black its house out; with a passenger inside (extra output) a new link does; the absorber refuses the balloon-hovering disc | `drain_pairs::a_drained_absorber_blacks_out_only_with_passengers` | S-0608, S-0405, S-0605 |
| Drain × grinder: a drained grinder of a resource-destination type pays its house's credits to the drainer's house in payouts of *A*, cannot be sold, and an own grinder refuses the balloon-hovering disc | `drain_pairs::a_drained_grinder_pays_and_cannot_be_sold` | S-0608, S-0603, S-0408 |
| Drain × tank bunker: a drained bunker's firing answer is *illegal* and it cannot be sold, but without output it blacks nothing out; a bunker refuses the balloon-hovering disc (row P6) | `drain_pairs::a_drained_bunker_refuses_fire_and_sale_but_no_blackout` | S-0608, S-0606, S-0405, S-0408 |
| Genetic Mutator × mind control: a cell-mode launch converts a controlled unit, which leaves its controller's list; the made unit is the launching house's and inherits no control; a converted controller releases its other victims to their owners | `mutator_pairs::a_mutated_controlled_unit_and_a_mutated_controller` | S-0611, S-0601 |
| Genetic Mutator × tank bunker: a launch on a bunker hits neither the bunker nor its bunkered vehicle, which stays bunkered; infantry beside it in the block converts | `mutator_pairs::a_launch_on_a_tank_bunker_leaves_it_and_its_vehicle` | S-0611, S-0606 |
| Genetic Mutator × absorber: a passenger inside is not listed and is not hit; a walker still on its way in converts and its entry ends | `mutator_pairs::a_launch_on_an_absorber_spares_its_passenger_and_takes_an_entrant` | S-0611, S-0605 |
| Genetic Mutator × magnetic lift: a launch under a seized vehicle leaves it unhurt and gripped; infantry on the ground beneath converts | `mutator_pairs::a_launch_under_a_seized_vehicle_takes_only_the_infantry` | S-0611, S-0607 |

The parachutist's order of the virus animation and the explosion was request Q-1421; S-0615 revision 3
answers it (the virus effect first, then the kind 3 or 5 animation), and the test now checks the order
(M7b step 1c).

What a drained bunker, absorber or grinder does with its occupants and entrants (a bunkered
vehicle's fire, entries, releases and recycling during a link) is request Q-1444. Interim: only
the consequences S-0608 lists apply, so occupants are handled as at any other time; the drain
tests assert nothing about them (the absorber test ends its first link before the passenger walks
in).

## Pairs left

Each waits for its spec (named seams in the code).

- Psychic Dominator (S-0612) with controlled units, bunkers (`bunker::holds_through_owner_change`) and the absorber.
- Cloning Vats, and the Force Shield against the M7a weapons.

## Local relation tests on the reference ruleset

None yet (step 4b). The local lane (`crates/app/tests/m7b_lane.rs`, scenes `deaths`, `walker` and `disc`)
shows each special hit on the installed data, one at a time.

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

The parachutist's order of the virus animation and the explosion is request Q-1421 (interim: the virus
animation first); the test checks only what both orders share.

What a drained bunker, absorber or grinder does with its occupants and entrants (a bunkered
vehicle's fire, entries, releases and recycling during a link) is request Q-1444. Interim: only
the consequences S-0608 lists apply, so occupants are handled as at any other time; the drain
tests assert nothing about them (the absorber test ends its first link before the passenger walks
in).

## Pairs left

Each waits for its spec (named seams in the code).

- Genetic Mutator (S-0611) with controlled, bunkered, absorbed and seized units; death kind 9 in `combat::death`.
- Psychic Dominator (S-0612) with controlled units, bunkers (`bunker::holds_through_owner_change`) and the absorber.
- Cloning Vats, and the Force Shield against the M7a weapons.

## Local relation tests on the reference ruleset

None yet (step 4b). The local lane (`crates/app/tests/m7b_lane.rs`, scenes `deaths`, `walker` and `disc`)
shows each special hit on the installed data, one at a time.

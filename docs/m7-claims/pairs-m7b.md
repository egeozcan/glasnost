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

The parachutist's order of the virus animation and the explosion is request Q-1421 (interim: the virus
animation first); the test checks only what both orders share.

## Pairs left

Each waits for its spec (named seams in the code).

- Floating Disc (S-0608 drain) with the bunker, the absorber, the grinder and power: `approach::draining`.
- Genetic Mutator (S-0611) with controlled, bunkered, absorbed and seized units; death kind 9 in `combat::death`.
- Psychic Dominator (S-0612) with controlled units, bunkers (`bunker::holds_through_owner_change`) and the absorber.
- Cloning Vats, and the Force Shield against the M7a weapons.

## Local relation tests on the reference ruleset

None yet (step 4b). The local lane (`crates/app/tests/m7b_lane.rs`, scenes `deaths`, `walker` and `disc`)
shows each special hit on the installed data, one at a time.

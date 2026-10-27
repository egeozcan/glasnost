---
id: Q-1876
title: With quirks.docked_hover_water_loss off, is a hovering vehicle disabled away from any depot spared the water test?
status: answered
related: [S-0314, S-0503, S-0405]
answered-by: [S-0314]
---

## Need

S-0314 revision 77 (claim 21, *Over water, at a naval yard*, and the Edge case *Hovering vehicle
lost at a naval yard*) gives the rule for `quirks.docked_hover_water_loss` off. A spin end that
comes while the vehicle has contact with a depot (Terms) or is settled makes no water test. Once
its contact has ended and it is not settled, a later spin end is tested as usual by S-0503 claim 11.
The Edge case adds: "Everywhere else S-0503 claim 11's water test is unchanged."

S-0314 Terms (*Settled vehicle*, *What makes it settled*) lists "the switch-off of a disabling of
S-0405 claim 20 (claim 21)". It does not say that the vehicle must be in contact with a depot. Claim
21's *Disabling while docked* describes the case of a vehicle in contact, but the Terms entry
starts "A state that a vehicle takes on at a depot".

We built the Terms list as written (M8 step A3 intake 6): every disabling of S-0405 claim 20 makes
the vehicle settled, wherever it is. Until now, being settled away from a depot had no effect we
could find. A disabled hovering vehicle cannot travel anyway, and the enabling that ends the
disabling also ends being settled.

With the new switch off, this reading decides a case that is not at a depot. Take a hovering vehicle
of a type with the powered-unit flag, hovering over open water far from any depot. When its house
loses its last control centre, it is disabled and takes S-0503 claim 10's two draws. Its spin ends
when it comes down. Under the built reading it is then settled, so with the switch off it is not
tested and rests on the water. Under the other reading it has no contact and is not settled, so it
is tested and lost, as S-0503 claim 11 gives everywhere else. With the switch on, both readings give
the loss.

## Questions

1. Does the switch-off of a disabling of S-0405 claim 20 make a vehicle settled (S-0314 Terms) when
   the vehicle has no contact with any depot? Or does it do so only for a vehicle in contact, as in
   claim 21's *Disabling while docked*?
2. If it makes any vehicle settled: with `quirks.docked_hover_water_loss` off, should a hovering
   vehicle disabled over open water, away from any depot, be spared the water test at its spin's
   end? Or should the switch-off rule be read only for a vehicle at a depot, for example settled
   only through claim 17's arrival, a settling processing, or a disabling while in contact?
3. Is there any other effect, in the original, of a vehicle being settled while it has no depot
   contact and has never been at one? (The stranding of claim 22 needs a vehicle that was at a
   depot.)

## Possible observation

Question 1 matters to the original only through effects that S-0314 says are seen only at
depots, so it may not be observable. A design answer for the switch-off position is enough for the
engine.

## Current workaround

The code keeps to the built Terms reading. `hover::sink_test` skips the water test with the switch
off when the vehicle has depot contact (`depot::link`) or is settled (`depot::settled`). A disabling
anywhere makes the vehicle settled (`depot::disabled`). So with the switch off, a hovering vehicle
disabled through the loss of its control centre over open water is not tested. With the switch on,
which is the default, nothing changes. Test: `claims_s0314_naval_hover::q1876_interim_disabled_far_from_a_depot`.

This reading is broad. In our engine every switch-off of a hovering vehicle in play is either a
depot's (claim 21) or a disabling's (S-0405 claim 20), and both leave the vehicle settled. So with
the switch off, under the built reading, no hovering vehicle is lost to a spin that ends while it is
still settled from that switch-off, at a depot or not. Only a spin that ends after an enabling (or another end of being settled) has
cleared that state, with no depot contact, is still tested. If the answer narrows the Terms entry or
the switch-off rule, the gate in `hover::sink_test` or `depot::disabled` changes, and so does the
test above.

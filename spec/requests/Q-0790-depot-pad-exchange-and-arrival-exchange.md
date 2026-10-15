---
id: Q-0790
title: Pad depot tether with an empty place 0, and the depot's movement-arrival exchange
status: answered
related: [S-0314, S-0215, S-0503]
answered-by: [S-0314]
---

## Need

S-0314 revision 18 (claim 17) answers Q-0674's tether question. Three details it leaves open
are needed by `depot::pad_exchange`, `depot::moving_destination` and `depot::movement_arrival`.

## Questions

1. **Empty place 0.** For a depot with the pad flag, the repeated request tethers a vehicle in
   the foundation centre's cell that has a moving destination only when "the vehicle in contact
   place 0 is more than 128 leptons from that centre". At a depot with two or more places, place
   0 can be free while the requester holds place 1. Does an empty place 0 pass this test, fail
   it, or does the original read some other position?
2. **Ready mark of the movement-arrival exchange.** "Reaching the supplied docking-point cell
   during movement queues *repair* at the depot and *sleep* at the vehicle." The completed-entry
   exchange of a tethered vehicle also sets the depot's ready mark (S-0215 claim 29). Does the
   movement-arrival exchange set the ready mark too, or does the depot's *repair* start by
   S-0215 claim 11's per-tick start alone?
3. **"Its locomotor currently reports movement" (Terms, moving destination).** For the driving
   and ship locomotors, does this mean a track in progress, a turn in place, or stored path
   steps? For the hovering locomotor, does it mean S-0503's *is moving* (a destination *D* or a
   heading point), *is moving now* (also a nonzero throttle), or something else?
4. **A one-place pad depot.** With one place, place 0 is the requester itself. A vehicle that
   drives straight into the centre cell is at most 128 leptons from the centre when it first
   occupies the cell, and the movement-arrival exchange makes *sleep* current before a later
   *enter* processing can find it without a moving destination. So it is never tethered. Is that
   the intended outcome?

## Possible observation

S-0314 scenario H19 d's named test map: two pad depot types, one with two places. For 1, leave
place 0 empty and record the tether of a moving requester in place 1. For 2, record the depot's
ready mark and the tick its *repair* becomes current after a no-pad arrival. For 3, record the
tether of a driving requester and a hovering requester that stand in the centre cell while still
holding a destination, while turning, and while their throttle is 0. For 4, record the tether of
a straight-driving requester at a one-place pad depot.

## Current workaround

- An empty place 0 does not pass the distance test.
- The movement-arrival exchange sets no ready mark.
- A driving or ship vehicle reports movement while it has a track in progress, a turn in place
  or stored steps (`occupancy::in_progress`); a hovering vehicle while it *is moving* (S-0503
  Terms).
- The literal reading of question 4 is kept. The depot tests set the tether directly where they
  need a docked vehicle.

Each choice is marked in the code with this request's ID.

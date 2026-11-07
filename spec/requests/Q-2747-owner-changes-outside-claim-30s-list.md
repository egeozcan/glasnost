---
id: Q-2747
title: The destination and guard request at owner changes that S-0309 claim 30 does not list
status: answered
related: [S-0309, S-0509, S-0601, S-0215]
answered-by: [S-0309]
---

## Need

S-0309 r120 claim 30: a holder that changes owner drops its target, whatever the cause. Its third
bullet adds: "For every cause listed, the same change also clears the holder's destination and
requests *guard* in the way that S-0601 claim 9 gives for a release". The causes listed are the
occupation and return of a civilian garrison building (S-0508 claims 7 and 8), an engineer
capture (S-0509 claim 8), a trigger's change-owner action (S-0011 claim 24) and a mind-control
takeover or release (S-0601 claims 6 and 9).

Two owner changes in the specs are not in that list, and the claim does not say whether they
clear the destination and request *guard*:

- the takeover of a vehicle-like building by an infantry unit (S-0509 claim 17), which changes
  the building's owner;
- an object in contact with a building that changes owner, which stays near the docking point
  and changes owner with it, keeping its contact and its tether link (S-0509 claim 14), such as a
  harvester unloading at a refinery that is captured, or a unit still on a factory's exit.

For the second, a *guard* request and a cleared destination would end the harvester's unloading
and the exiting unit's walk-off (S-0417), which is why the engine cannot carry the bullet over by
itself. The code is `crate::structures::ownership::change_by` (the building) and
`crate::structures::ownership::contacts` (the objects that follow it).

## Questions

1. When an infantry unit takes over a vehicle-like building (S-0509 claim 17), does the building
   get its destination cleared and *guard* requested at the change, as at an engineer capture?
2. When a refinery is captured while a harvester of the old owner unloads at it, and the
   harvester changes owner with it (S-0509 claim 14): does the harvester's destination get cleared
   and *guard* requested, or does it go on unloading for its new owner as before the change?
3. The same for a unit that is still on a captured factory's exit (S-0509 claim 14, a factory of
   vehicles, where the distance is not tested): does it get *guard*, or does it go on with its
   walk-off?

## Possible observation

S-0309 scenario A19 part a, with (1) a vehicle-like building and the infantry unit that takes it
over in place of the defensive building and the engineer, and (2) a refinery with a harvester of
*E* unloading at it in place of *D*. Record the destination and the current and queued behaviour
of the building and of the harvester each tick.

## Current workaround

Both causes drop the holder's target at the change (claim 30 gives the drop for every cause).
Neither clears the destination or requests *guard*: the building taken over and the contacts that
follow a building keep their destination and behaviours, as before claim 30.

---
id: Q-2376
title: A building firer, or a firer beyond range, whose temporal beam holds a departed warp member
status: answered
related: [S-0513, S-0514, S-0309, S-0215]
answered-by: [S-0513]
---

## Need

S-0513 r89 claim 39a (*The firer*, Q-2317 answered) says what an infantry or vehicle firer does
while a warp member it holds by its temporal beam waits to land, from the departure update to
the appearance: when it is within its weapon's range of the member's old position and its
firing check answers *rearming*, it keeps the member, makes no shot and neither moves nor turns.
It says that a building firer is not covered there (an aircraft firer is the aircraft specs').
The firing check's *rearming* (S-0514 claim 19) is given for every firer while the beam is
attached, before and after the departure.

So two cases have no stated answer for what the firer's own processing does with the member
while it is off the map:

- a building firer (a building whose primary weapon's warhead is temporal);
- an infantry or vehicle firer that is beyond its weapon's range of the member's old position
  at the departure. Such a firer is one that takes no approach step (S-0215 claim 37 g, a
  deployed infantry unit of a human-controlled owner) or whose approach has not yet moved it,
  since claim 39a *Release* has a firer in *attack* that takes the approach step move towards
  a member beyond range and let go at its first new cell.

The engine's places are `combat::held::turn_start_drop` (the turn-start drop of a held object
that is off the map, S-0309 claim 27 as our rule), `combat::held::firing_target` (the firing
step's target) and the building's attack processing in `combat::behave` (S-0215 claim 32's
no-target row).

## Questions

1. While a warp member waits to land, does a building whose temporal beam is attached to it
   keep it as its target? If it does, does its attack processing treat it as a target (its
   firing check answering *rearming*, no shot), or as no target (S-0215 claim 32's no-target
   row: *guard* queued and started at once)?
2. While a warp member waits to land, does an infantry or vehicle firer whose temporal beam is
   attached to it, and that is beyond its weapon's range of the member's old position, keep it
   as its target? If it is in *attack* and takes the approach step, does it move towards the
   old position (and so let the beam go at its first arrival in a new cell, S-0514 claim 14)?

## Possible observation

SW12 part l's set-up with a temporal tower as *T* in range of the tank's old position: read
the tower's target and current behaviour in each tick from the tank's departure update to its
appearance, and whether its beam is still attached after the appearance. For question 2, part
l3's infantry *T* deployed (human-controlled owner) and placed beyond its weapon's range of the
tank's old position before the departure, with the same record.

## Current workaround

As intake 75 built it (the interim of Q-2317): a holder whose temporal beam is attached to a
departed member keeps it at the start of its turn, whatever its kind and distance. A building
firer's attack processing counts the off-map member as no target (S-0309 claim 27's firing-step
rule as our engine's), so it takes S-0215 claim 32's no-target row. An infantry or vehicle
firer beyond range takes claim 39a's rules for one within range: its firing step treats the
member as its target, its firing check answers *rearming* and it makes no shot; its approach
step, if it takes one, tests range alone against the old position.

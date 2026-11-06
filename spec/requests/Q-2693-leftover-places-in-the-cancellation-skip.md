---
id: Q-2693
title: Whether leftovers that take no hit hold places in the sweep's cancellation skip
status: answered
related: [S-0203, S-0414]
answered-by: [S-0203]
---

## Need

S-0414 claim 15 counts the places of the defeat sweep in creation order, and under
`quirks.defeat_cancellation_skip` a hit that deletes production objects at once makes the sweep
skip following places, "whoever owns the object". S-0203 r50 claim 13 (c) puts every leftover
infantry unit, vehicle, aircraft or building at its place in that order. Our engine offers the hit
only to leftover vehicles with no health, and with `quirks.defeat_sweep_leftovers` off to no
leftover at all.

The specs do not say whether a leftover that our engine offers no hit (one with health left, one of
another kind, one of another house, and every leftover with the switch off) still holds a place
that a skip can use up. In the original, as claim 13 (c) describes it, every leftover of the four
kinds holds its place. With the switch off, the departure could instead be read as a sweep that
does not see leftovers at all, as before revision 50.

## Questions

1. With `quirks.defeat_sweep_leftovers` on, does each leftover of the four kinds hold a place in
   S-0414 claim 15's order, so that a skip after a cancelling hit can fall on it, whether or not
   our engine offers it the hit?
2. With the switch off, do the leftovers still hold their places (only the hit is withheld), or
   does the sweep pass over them as if they did not exist, so that a skip falls on the next object
   in play?

## Possible observation

Question 1 is a question about the original: a house with a factory building an item, whose
destruction by the sweep deletes the item at once, followed in creation order by a vehicle of
another house destroyed earlier in the same tick and then by a third object of the house. Record
whether the third object is hit. Question 2 is a choice for our engine's departure.

## Current workaround

Every leftover of the four kinds holds its place in both positions of the switch, cited `spec:
S-0414 claim 15` and `S-0203 claim 13 (c)`: the switch withholds only the hit.

---
id: Q-1260
title: How the bio-reactor fate of a released unit finds and keeps its building
status: open
related: [S-0601, S-0605, S-0215]
answered-by: []
---

## Need
S-0601 claim 8 selects a *bio reactor* fate for a released or taken unit of a computer-controlled
house and points to S-0215 claim 24's enter-nearest search. S-0215 claim 24 describes the search as
part of an instruction that our engine's units never hold (S-0605 claim 13). The engine needs to
know what the fate does when it is chosen: `crates/deterministic/sim/src/structures/absorb.rs`,
`bio_reactor_fate`.

## Questions
1. Is the search made once when the fate is chosen, or repeated (for example while the chosen
   building fills up or is destroyed before the unit arrives)?
2. Which point of the building is the distance measured to (its centre, its top-left cell, the
   nearest foundation cell)? Which building wins a tie?
3. When no building of the unit's owner accepts it, does the fate fall back to *hunt* at once, or
   does the unit keep looking?
4. Does the unit go in under the ordinary rules of S-0605 claims 3 to 7 (contact, waiting in line,
   entry), or by another path?

## Possible observation
A neutral house with two absorbing buildings at different distances from a unit; take the unit
with a controller of another house and release it with the fate weights set so that the bio
reactor is always chosen. Record which building it walks to, and what it does when that
building is full or destroyed on the way.

## Current workaround
One search when the fate is chosen: the absorbing buildings of the unit's own owner that accept
it (S-0605 claim 1), the nearest by S-0215 F4's 3-D distance to the building's centre, ties to
the lowest object number. Found: that building as destination, *enter* queued, then S-0605
claim 3. None found: the fate's failed-assignment fall-back, *hunt*.

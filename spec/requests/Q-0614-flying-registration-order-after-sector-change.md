---
id: Q-0614
title: Where does a tracked flying object stand in its new sector's order after crossing?
status: answered
related: [S-0303, S-0506]
answered-by: [S-0303]
---

## Need

S-0303 F7 orders the candidates within one sector by registration order and says the game updates
an object's sector as it flies; claim 11 leaves the sector-crossing order unresolved. Our engine
needs a rule in `glasnost_sim::combat::flying` for the place a rocket missile (and later any
tracked aircraft) takes in its new sector when it crosses into it.

## Questions

1. When a tracked flying object crosses into another sector, does it come after every object
   already registered in that sector, or does it keep a place by its first registration?
2. With `quirks.flying_search_sectors` off, is the order of all tracked flying objects the order
   of first registration, or does a sector change also move an object to the end?

## Possible observation

Two missiles in the same sector at different spread distances, one of which crossed a sector
boundary later; a detonation that hits both and whose first hit's death weapon would kill the
second (S-0303 scenario A11 style): record which hit lands first.

## Current workaround

A crossing object is appended to the end of its new sector's list; with the quirk off, the order
of first registration is kept (marked Q-0614 in `glasnost_sim::combat::flying` and in
docs/development.md, M6a interim rules).

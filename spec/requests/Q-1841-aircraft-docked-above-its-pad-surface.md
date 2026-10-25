---
id: Q-1841
title: Where an aircraft docked at a positive docking height is listed
status: answered
related: [S-0501, S-0209, S-0404, S-0303]
answered-by: [S-0501]
---

## Need

S-0501 claim 47 (b) says a landed aircraft resting on a pad is never tracked and "is listed in its
cell layer instead (S-0209 claim 13)", and scenario AC30 part i (2) expects one area-damage hit
from the scanned cell for an aircraft landed on its house's pad. S-0209 claim 13 lists an aircraft
by its listing height: a positive listing height keeps it out of both cell-layer lists. S-0404
claim 15 gives a pad's docking position as the pad's centre plus the place's displacement, its
height component included, and S-0501 claim 43 gives no deck mark on a pad. So when a pad's
docking displacement has a positive height, a docked aircraft rests above the terrain surface
with a positive listing height: by S-0209 claim 13 it is in no cell list, and by claim 47 it is
not tracked either. It would then be in neither, and no area damage or search would find it,
which contradicts claim 47 (b)'s "listed instead" and AC30 i (2).

The code (`air::released_docked`, `air::touchdown`, `air::listing_layer`) follows S-0209 claim 13
as written: such an aircraft is listed in no cell layer and not tracked.

## Questions

1. An aircraft docked on a pad whose docking position is above the terrain surface: is it found
   by area damage from a detonation in or next to its cell, and by a ground search that looks for
   aircraft (S-0309 claim 7's ground-layer pass), as an aircraft landed on open ground is?
2. If yes, through which list: the ground cell layer of its cell (whatever its height above the
   surface), or another?
3. Is it found in the same way while it lands onto that docking height (after touchdown is
   accepted, while it rests at the docking position) and while it takes off from it before its
   height rises further?

## Possible observation

AC30 part i (2) with a pad whose docking displacement has a positive height component (a named
test copy of the pad with that displacement set, if the stock pad has none): count the hits that
the docked aircraft takes from one detonation within the spread. One hit means it is listed in a
cell layer; none means it is in neither list.

## Current workaround

As S-0209 claim 13 reads: listed in no cell layer while its listing height is positive, and not
tracked (S-0501 claim 47 (b)). No test of claim 47 depends on the case: its synthetic pads
dock at height 0.

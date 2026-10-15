---
id: Q-0684
title: Owner of a wall placed by a wall building of the map's building list (S-0112 claim 7, S-0210 claim 14)
status: open
related: [S-0112, S-0210, S-0406, S-0009]
answered-by: []
---

## Need
S-0112 claim 7, table "Initial state", third row: a wall from a wall building of the map's
building list is "placed as a wall built in play, so at stage 0, owned by the building's owner".
S-0210 claim 14 (a): "Once the map's buildings have been placed and before any starting unit of
a skirmish is created (S-0009), each cell that holds a wall overlay gets as its owner the owner
of the nearest building that, at that moment, is on the map and not destroyed, and whose owner's
country has the wall-owner flag". Read together, the wall building's owner is set when the wall is
placed and then replaced by claim 14 (a)'s nearest-building owner, which can be another house or
none (the converted wall building is itself no building, so it is not a candidate). If S-0112
means the owner the wall keeps at the start of the game, the two claims disagree. The owner
decides S-0209's wall rows (allied walls), wall sales (S-0408 claim 12), S-0406 claims 8, 9 and
11. The engine needs it in `objects::place_map_objects` and `crush::assign_wall_owners`.

## Questions
1. Which owner does a wall from a wall building of the map's building list have when the game
   starts: the wall building's owner, or claim 14 (a)'s nearest qualifying building's owner?
2. If the wall building's owner: does claim 14 (a) skip such walls, or does it apply only to
   walls of the overlay list?
3. Does a map-listed wall building whose owner is a human player's house in a skirmish (dropped by
   S-0009 claim 27 for structures) place its wall?

## Possible observation
OV6 or K12 variant: a map whose building list holds a wall building owned by house A, two cells
from a building of house B whose country has the wall-owner flag and far from any building of A;
record the wall's owner by a sell-wall order from each house and by S-0209's allied-wall row.

## Current workaround
Interim rule, recorded in docs/development.md: the wall is placed owned by the wall building's
owner, and S-0210 claim 14 (a), which runs after the map's buildings, then gives every wall cell
its nearest-building owner, these walls included. A wall building dropped by S-0009 claim 27 places
no wall.

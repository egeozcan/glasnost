---
id: Q-0620
title: What are the ordinary placement rules when a parasite comes back on the map?
status: open
related: [S-0614, S-0209, S-0208]
answered-by: []
---

## Need

S-0614 claim 4 (a failed latch) and claim 13 (leaving a living host) place the parasite "at the
centre of the cell … under the ordinary placement rules", and remove it silently "when it cannot
be placed there". Claim 15 places "a point within that cell … as for any placement" in the
fallback neighbour cell. No spec states those placement rules for a unit that appears on the
map in a given cell. Our engine needs one test in `glasnost_sim::parasite::place_ordinary`
(claims 4 and 13) and one position choice in `glasnost_sim::parasite::host_lost` (claim 15).

## Questions

1. When a vehicle parasite is put back in a cell, what blocks the placement: other vehicles,
   infantry, buildings, terrain objects (trees), overlays, cells its class cannot enter? Does the
   house of a blocking unit matter?
2. When an infantry parasite is put back in a cell, which standing position does it take (S-0208
   table P), in what order are the positions tried, and is the cell refused when all are taken?
3. Does a failed placement in the given cell ever move the parasite to another cell, or is it
   always removed silently as S-0614 claim 4 says for the failed latch?
4. For S-0614 claim 15's fallback neighbour: which point within the cell does an infantry
   parasite take, and does a vehicle parasite take the cell centre?

## Possible observation

S-0614 scenario P3 with the parasite's former cell taken, in turn, by an allied vehicle, an
enemy vehicle, an infantry unit (one, then five), a tree and a wall overlay; record whether the
parasite reappears there, elsewhere, or is removed. The same with an infantry-type parasite.

## Current workaround

An interim rule, noted in the game's notes and in docs/development.md: the cell must be
enterable by the parasite's own locomotion class and movement zone (S-0211 claim 10), judged
on the bridge deck when the cell is a bridge and the parasite came from the deck, and hold no
building. A vehicle parasite also needs a cell without any infantry or vehicle; an infantry
parasite takes the first free standing position of 0 to 4 and is refused when none is free.
The claim 15 fallback places either kind at the cell centre.

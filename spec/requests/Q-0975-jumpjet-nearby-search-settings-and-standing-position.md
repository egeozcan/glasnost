---
id: Q-0975
title: Settings of the nearby-cell searches of a jumpjet and the standing position it takes
status: open
related: [S-0502, S-0212, S-0208, S-0209, S-0517]
answered-by: []
---

## Need

S-0502 claim 16 (a move order) and claim 17 (a stopped or failed landing) run the nearby-cell
search of S-0212 claim 25, and S-0517 claim 6 runs it again at the end of an animated undeploy of a
landing-first vehicle. S-0212 table T5 has no row for these uses. Claim 16 gives only the locomotion
class (hover) and the movement zone (flying); claim 17 the unit's own class and zone. Not given:
the reach region, whether the search is in deck mode, the height condition, the further test, the
no-overlay condition, the reference cell, and whether bridge cells are excluded. Claim 16 also says
an infantry jumpjet "takes a free standing position in that cell (S-0209)" without saying which
free position, in which order, or whether a random draw is taken as the walking infantry choice of
S-0208 claim 3 does.

## Questions

1. For a jumpjet's move-order search (class hover, zone flying), what is T5's row: reach region
   (none, or the region of the unit's own cell for which zone), deck mode (never, or when the
   centre is a bridge cell, or when the unit stands on a deck), height condition (yes or no),
   further test (none), no-overlay condition (yes or no), reference cell (none, or the unit's
   cell), bridge cells (allowed or excluded)?
2. The same for the failed-landing search of claim 17 (the unit's own locomotion class and zone) and
   for S-0517 claim 6's search around the vehicle's position.
3. For an infantry jumpjet that takes a standing position in the found cell: which position does it
   take when the cell's centre position is free, and in which order are the others tried when it
   is not (the order of S-0208 table O for the class of the ordered point, or another)? Does the
   choice take a random draw?

## Possible observation

A jumpjet infantry unit ordered to land on a cell whose centre position is taken by another
infantry unit, in a cell with a high bridge, and in a cell with an overlay; record the cell found
and the standing position it lands on, with the random-state recorder around the order.

## Current workaround

`jump::search` uses: no reach region, deck mode when the centre is a bridge cell, the height
condition on, no reference cell, bridge cells allowed, no overlay condition, for all three uses.
`jump::free_standing_point` takes the position class of the ordered point if it is free, then the
side positions in S-0208 table O's order for that class with draw value 0, else the centre; it
takes no draw. The unit's own occupancy mark is lifted for the move-order search and laid again
after it, so that an order to its own cell finds that cell (scenario J2).

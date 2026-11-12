---
id: Q-3011
title: In a refused pack-up, do the building's holders drop it before or after the removal's cell recalculation?
status: answered
related: [S-0415, S-0309, S-0010, S-0202]
answered-by: [S-0415]
---

## Need

S-0415 claim 10 hands the targets of every object whose target was the packing building to the
vehicle that appears, and claim 12 makes the return-cell test with the building no longer on the
map. So the building is removed before it is known whether its holders hand over (claim 10) or
lose their target (claim 11, S-0309 claim 27's drop with its targeting-timer draw).

Our removal used to make every holder drop the building first, before the rest of the removal,
which includes the recalculation of the building's foundation cells (S-0010 claim 19 can start a
tile animation there, with S-0202 claim 10's creation draws). GitHub issue #221 found that this
drop took away the target claim 10 hands over. We now remove the building without the drop, make
the return-cell test, and on refusal make the holders drop it after the removal, before the
refund. The holders' timer draws therefore come after any tile-animation draws of the cell
recalculation, where before they came ahead of them. The specs do not say which order is right.

## Questions

1. In a pack-up refused at the return cell (S-0415 claim 11), do the holders of the building drop
   it (S-0309 claim 27, each with its timer draw, in creation order) before the removal's
   recalculation of the foundation cells and its tile-animation draws, or after them?
2. Is the same order also the one of an ordinary removal of a building, outside a pack-up?

## Possible observation

A packing building whose foundation includes a cell where a tile animation can start (S-0010
claim 19), held as target by two enemy units, packed up with its return cell blocked. The order
of the draws shows in what follows them on the scenario random stream; S-0202's draw recorder
would separate the two orders.

## Current workaround

On refusal, the holders drop the building after its removal, before the refund and before the
end of the old slave group (S-0312 claim 31); the code cites this request. A successful pack-up
and a deploy (S-0415 claims 6 and 10) hand the targets over and take no drop.

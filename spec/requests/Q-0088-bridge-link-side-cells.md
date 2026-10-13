---
id: Q-0088
title: Which side cells a bridge link connects (S-0212 claim 13b)
status: answered
related: [S-0212, S-0010]
answered-by: [S-0212]
---

## Need
`glasnost_sim::corridor` adds the connections of an enabled bridge link to
the sectors of every level (S-0212 claim 13b, *Links*): between the sectors
of its two end cells and, for a bridge, "between the sectors of the cell
beside one end cell and the cell beside the other end cell on the same side
of the bridge, for each of the two sides across the bridge's axis". The
claim adds: "Which side is which comes from the end cell's tile. For two of
the 16 tiles of each bridge role the original takes the two diagonal
neighbours (north-west and south-east) instead; whether those tiles occur at
a link's end is not established." The code needs the cells by rule.

## Questions
1. For an end cell whose tile is one of the ordinary tiles of the bridge or
   wooden-bridge role, which two neighbours are "the cells beside" it: the
   two neighbours across the link's line (for a link along cell x, the cells
   one step towards y − 1 and y + 1; for a link along cell y, towards x − 1
   and x + 1)? Which of the two counts as the same side as which neighbour of
   the other end cell?
2. Which two tiles of each role (by their place *i*, 0 … 15, in the role's
   range, as in S-0010 table B) take the north-west and south-east
   neighbours instead, and how are those two paired with the side cells of
   the other end cell (whose tile may be an ordinary one)?
3. Does the tile of the first end alone decide the side cells of both ends,
   or does each end cell's own tile decide its own?

## Possible observation
None practical: the side connections change only which corridor the coarse
search picks, which S-0212 already marks as unmeasured for bridge cells.
A reading of the original's rule is enough.

## Current workaround
For every tile, the cells beside an end cell are its two neighbours across
the link's line, and the neighbour towards y − 1 (or x − 1) of one end is
paired with the same neighbour of the other end; the diagonal exception is
not applied.

---
id: Q-3122
title: Which cell layers the congestion marks of a re-path read
status: open
related: [S-0212, S-0209]
---

## Need
S-0212 claim 19 builds the congestion marks of a mode-1 or mode-2 re-path from "a unit in the cell
ahead" (the qualifying units whose stored paths part (i) counts) and from "the occupied cells of the
5 × 5 block centred on the cell ahead" (part (ii)). It names no cell layer. The glossary (*cell
layer*) puts an object on a bridge deck in the deck layer of its cell and every other object of the
cell in its ground layer, so a bridge cell can hold units on both layers at once.

The claim's consequence ("the cell ahead, which is normally occupied (that is why the unit is
blocked), is counted once in (ii) and once in (iii)") holds for a unit on a deck only if the unit
that blocks it on the deck is seen. Our engine read the ground layer only, so for a unit blocked on
a deck no qualifying unit was found and the search ran as mode 0 with no mark (issue #305).

What the specs do not say:
- whether a unit on the other layer of the cell ahead (on the ground under the deck, for a unit
  blocked on the deck; on the deck above, for a unit blocked under it) can qualify for part (i);
- for part (ii), which layer of each bridge cell of the 5 × 5 block decides whether that cell is
  occupied: the layer of the blocked unit's step into the cell ahead, the layer a step from the
  unit's search pair would reach in that cell (claim 7), either layer, or both counted separately;
- whether an object of any kind (a building, a terrain object, an overlay object) under a deck makes
  a deck cell occupied for part (ii), and the same for an object on a deck over a ground cell.

## Questions
1. Which layer or layers of the cell ahead does claim 19 read for the qualifying units of part (i)?
2. For part (ii), which layer or layers of each cell of the 5 × 5 block decide whether it is
   occupied, and can a cell be counted twice in (ii) when both of its layers hold objects?
3. Does the answer depend on whether the blocked unit is on a deck, or on the height level of its
   search pair (S-0212 claim 7)?

## Possible observation
Scenario P14 (mode-1 stored-path marks) on a high bridge: a fast vehicle blocked on the deck behind a
slower vehicle of another type with two or more stored steps, once with and once without a parked
vehicle on the ground under the deck at a cell of the 5 × 5 block, and once with the slower vehicle
on the ground under the deck instead. The route the blocked vehicle takes after its re-path shows
which cells carried the mark.

## Current workaround
`congestion` in `crates/deterministic/sim/src/path.rs` reads the cell ahead, for part (i) and for
part (ii), on the layer of the blocked unit's step into it (claim 7's candidate layer from the unit's
search pair, as the blocker chain of claim 8 reads its first cell). Every other cell of the 5 × 5
block is read on the layer a step from the unit's search pair would reach in it by claim 7's rule.
A unit or object on the other layer of a cell is not counted, as before for the ground.

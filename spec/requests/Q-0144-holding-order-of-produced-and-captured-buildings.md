---
id: Q-0144
title: When a building joins its owner's holding order
status: open
related: [S-0402, S-0404, S-0406, S-0412]
answered-by: []
---

## Need

S-0402 claim 8 takes a house's buildings in holding order (S-0402 Terms) for the refinery choice
and for its tie and primary-building rules. Three points are open (S-0402 Open questions):

- whether a produced building joins the holding order when its production starts or when it is
  placed;
- where a building goes in the order after a capture, which is left to other specs;
- the order of the buildings a skirmish creates at its start, which is also left to other specs.

The engine needs this in `econ::harvest::choose_refinery`.

## Questions

1. Does a produced building join its owner's holding order when its production starts or when it
   is placed?
2. Where does a captured building go in its new owner's holding order?
3. In which order do the buildings a skirmish creates at its start join their owners' orders?

## Possible observation

Run scenario H4 part c with two refineries at equal distances from a returning harvester. Start
their production in one order and place them in the other.

## Current workaround

The holding order is the house's acquisition list (`HouseEcon::acquired`, kept by
`econ::events`). A building joins it when it is placed or captured and leaves it when it leaves
play. This is listed in the provisional table of `docs/development.md`.

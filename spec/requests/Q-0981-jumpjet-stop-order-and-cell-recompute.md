---
id: Q-0981
title: What a stop order does to a flying jumpjet, and when a jumpjet's cells are recomputed
status: answered
related: [S-0502, S-0213, S-0211]
answered-by: [S-0502]
---

## Need

Two gaps in S-0502.

First, claim 16 (no point) and claim 17 (stopped) and the Open question "Stopping outside a
landing": which orders stop a moving jumpjet, and what the stop does to one that is cruising,
ascending, hovering with an order or descending, is not traced. A stop order (S-0213 claim 8)
clears the destination.

Second, S-0211 claim 5 recomputes a cell when an object is put into it or leaves it. A walking or
driving unit does so at each cell crossing. S-0502 does not say whether a jumpjet in the air does
when its position crosses a cell edge (the recompute can create a tile animation with creation
draws, S-0010 claim 19), or only at take-off and touchdown.

## Questions

1. When a destination is cleared by a stop order: does a cruising or ascending jumpjet keep flying
   to its old point and land there, does it stop and hover where it is, or does it come down
   where it is? Does a hovering jumpjet with an order keep hovering?
2. Does a jumpjet's cell crossing in the air recompute the cells it leaves and enters?

## Possible observation

A stop order to a jumpjet in each flight state; a jumpjet flying over a cell with a tile
animation, with the random-state recorder.

## Current workaround

A cleared destination makes the jumpjet no longer *moving* and changes nothing else: a cruising,
ascending or descending jumpjet goes on to its stored point and lands there, a hovering one stays
hovering. Cells are recomputed at take-off (the cell left) and at touchdown (the cell entered), not
at crossings in the air.

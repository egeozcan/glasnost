---
id: Q-2507
title: Which position is the landing point of an aircraft that lands where it is, for claim 8's exception
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 r110 claim 8 makes one exception to the take-off: a landing aircraft given a destination
in the cell of the point it is landing at does not take off, and goes on landing. For an aircraft
that docks, that point is its docking position (claim 24). For one that lands where it is
(claim 6), the claim names "its own position".

An aircraft that lands where it is has no destination and, in our engine, no flight point. While
it lands it can still move: the landing starts while it still has some speed, and an aircraft whose
destination is cleared in flight (claims 6 and 8) slows down over several ticks. Its position when
the landing started and its position at the tick of the new destination can then lie in different
cells. The engine must decide which cell the exception compares with.

## Questions

1. For an aircraft that lands where it is (claim 6), is "its own position" in claim 8's exception
   its position at the tick the destination is given, or its position at the tick its landing
   started, or another point (for example a point the landing keeps)?
2. If the landing keeps a point of its own, does the aircraft's descent steer toward that point,
   or does it only slow down where it is?

## Possible observation

AC4's same-cell part, with the destination cleared while the aircraft flies at full speed so that
it starts to land as it crosses a cell boundary. Give it a cell destination at a later tick of the
landing, once in the cell where the landing started and once in the cell where it now is (when they
differ), and record whether it takes off.

## Current workaround

`air::ignores` takes the aircraft's position now, and the check is made again at every movement
update while the destination stands (and when the destination is given). A destination in the
cell it stands over is ignored for as long as it stays over that cell; if the landing aircraft
still drifts into another cell, the next movement update no longer ignores it and it takes off
for that destination. A destination in the cell where the landing started, if that differs,
makes it take off at once. Cited `S-0501 claim 8 … read as its position now, Q-2507`.

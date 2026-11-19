---
id: Q-3262
title: A walker's searches for an object destination that moves
status: answered
related: [S-0208, S-0212, S-0209, S-0507]
answered-by: [S-0208]
---

## Need

S-0208 Terms (r67) gives a walker's locomotor destination for a destination that names an object:
the object's movement point at the last assignment. It does not follow the object as it moves.
Claim 16 tests that point. Claim 6 (r67) says what a walker does after the object goes inside
without its locomotor being told (S-0507 claim 6a): its fresh searches (S-0212 claim 20) look for
that point's cell.

Neither spec says which cell a walker's searches look for while it still holds the object as its
destination and the object has moved away from that point. The searches are the fresh search of
S-0212 claim 20 when no stored steps are left, the immediate new path of S-0209 claim 7 and the
re-path of S-0209 claim 10. S-0212 claim 20 says, for a hovering unit only, that its searches use
its movement destination, not its order-level destination.

There is a related case for a walker standing with no stored path. Our engine finishes the move of
a walker standing in its destination's cell (S-0208 claims 16 and 18 speak of arrival ticks only).
For an object destination it compares with the object's current cell.

## Questions

1. While a walker holds an object as its destination, do its searches (S-0212 claim 20's fresh
   search, S-0209 claims 7 and 10) look for the cell of its locomotor destination, the point taken
   at the assignment? Or do they look for the cell the object is in at the time of the search?
2. Is the answer the same for each kind of object: infantry, vehicle, aircraft and building?
3. A walker standing with no stored path, its destination an object that has moved: which cell
   decides that it has nothing left to walk, the point's cell or the object's cell?

## Possible observation

Two synthetic walkers on open ground. Give walker A an order that makes walker B its destination
and no target, for example an area guard order on B (S-0213 claim 21). On the next tick order B to
a cell 20 cells away. Record A's position, its destination and its stored path each tick, until A
stops or reaches B. If A heads for B's first cell and stops there, the searches use the point. If
A follows B, they use the object's cell.

## Current workaround

The engine reads claim 16 with the point taken at the assignment, as S-0208 Terms gives it. Its
searches and the standing walker's check look for the cell the object is in at the time, as
before r67. After the object goes inside, both use the kept point's cell, as claim 6 gives it.

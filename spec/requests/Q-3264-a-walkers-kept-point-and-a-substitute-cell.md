---
id: Q-3264
title: A walker's kept locomotor destination and a substitute cell
status: open
related: [S-0208, S-0212, S-0507]
answered-by: []
---

## Need

S-0208 claim 6 (r67) says that a walker whose destination is lost without the locomotor being
told (S-0507 claim 6a) goes on along its stored path. Its fresh searches (S-0212 claim 20) look
for the kept point's cell, and claim 16 ends the walk by its second case. S-0212 claim 24 can
replace the cell a search looks for with a nearby substitute: for a cell destination the
substitute becomes the destination, so arrival is judged against it (departure D3). The walker
with a kept point has no destination for the substitute to replace. Two things are not settled:
what the walk then ends at, and the height that claim 16 uses for an object's movement point.

## Questions

1. A walker with a kept locomotor destination, and no destination, runs a fresh search, and
   S-0212 claim 24 adopts a substitute cell. Is the substitute only the cell the search looks
   for, with claim 16 still testing the kept point? Or does it replace the kept point, so the
   walk ends in the substitute cell? If it replaces it, what height does claim 16 compare with:
   the substitute's centre at claim 18's height, or the kept point's height?
2. For a destination that names an object, is claim 16's height the height of the object's
   movement point at the assignment, unchanged (for example an infantry unit's world height on
   a bridge deck)? Or is it adjusted as claim 18 adjusts a cell destination?

## Possible observation

Question 1: S-0507 scenario T16 with a walking holder, and a friendly unit standing still in the
cell of the kept point after the loss, so that the holder's next fresh search substitutes; record
the holder's cell and height when it stops. Question 2: a walker sent by an enter or guard order
to an infantry unit standing on a high-bridge deck; record when its walk ends.

## Current workaround

1. The substitute replaces the kept point: the walk goes on to the substitute's centre, at
   claim 18's height, as a cell destination's substitute does.
2. The movement point unchanged, with the object's own height.

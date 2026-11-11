---
id: Q-2988
title: A hunting vehicle's approach destination, left when its new target dies later in the same tick (M-0024)
status: open
related: [S-0215, S-0307, S-0309, M-0024]
answered-by: []
---

## Need

M-0024's Result row *Parts f and fc: un-hit free vehicles (step 4) classed neighbouring
destination: 0* is marked exact. After intake 124 (S-0217) the oracle lane's runs take different
draws, and the row fails on one vehicle-tick:

- Part f, F on side B, seed number 0, the tick in which the last F vehicle is destroyed.
- G's vehicle is in *hunt* and free at the end of the tick before: no target, no destination,
  nothing queued. It is not hit in the tick. At the end of the tick it has no target and a cell
  destination one cell from its cell (Chebyshev distance 1).
- No S-0217 check falls in that tick, and the vehicle's queue is empty. The draws moved because
  of the checks at earlier ticks.

As we read and build the claims, this comes from four rules acting in one tick:

1. The vehicle's *hunt* processing (S-0215 claim 27) searches and takes the last F vehicle as
   target.
2. Its approach step (S-0215 claim 37 part f, last row) searches for an approach position (S-0307
   claims 15 to 25). The position found is a cell next to its own.
3. Later in the same tick another G vehicle destroys that F vehicle.
4. S-0309 claim 27 drops it as every holder's target at the killing hit. No claim we know clears
   the holder's approach destination then, so the destination stays.

The record states 0 such rows over its 8 runs. Our run is not one of the record's games, so this
need not contradict the record. But the row is exact, so we ask whether the original can produce
such a row at all.

## Questions

1. In the original, when a vehicle in *hunt* takes a target and an approach destination in its
   processing, and that target is destroyed later in the same tick by another unit, does the
   vehicle keep the approach destination at the end of that tick?
2. If it does not: what clears it, and when (at the killing hit, as S-0309 claim 27 drops the
   target, or at the vehicle's next turn)?
3. Can S-0307's approach-position search give a cell next to the searcher's own cell for a target
   out of range, as it did here? Or is a neighbouring approach position a sign that a step of the
   search differs?

## Possible observation

M-0024's set-up, part f: for each tick in which a G vehicle's target is destroyed by another G
vehicle, record that vehicle's destination at the end of the tick before and of that tick.

## Current workaround

None: the claims as built (S-0215 claims 27 and 37, S-0307 claims 15 to 25, S-0309 claim 27). The
M-0024 oracle row fails on this one vehicle-tick, and is listed in `docs/m8-claims/intake124.md`.
The diagnostic that found it was a temporary local test, not committed.

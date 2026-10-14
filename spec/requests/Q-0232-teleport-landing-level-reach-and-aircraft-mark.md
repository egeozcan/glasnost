---
id: Q-0232
title: The teleporting landing's entry-outcome level, a destination in no reach region, an aircraft's mark and the warp animation's draws
status: answered
related: [S-0207, S-0202, S-0209, S-0212]
answered-by: [S-0207]
---

## Need

S-0207 claim 20 now gives the teleporting locomotor's own move, and our engine implements it
for vehicles in `crates/deterministic/sim/src/teleport.rs`. Four points it needs are left open
or appear to contradict another claim:

- Claim 20 (b) asks for the object's entry outcome (S-0209 claim 5) for the destination cell,
  but S-0207's Open questions say the height level at which it is asked (the ground or a
  bridge deck) was not settled. The entry outcome depends on that level.
- Claim 20 (b) uses the destination cell's reach region for the object's own zone. S-0207's Open
  questions say what happens when that cell lies in no reach region of the zone (water for a
  ground vehicle) was not checked.
- Claim 20 (a) moves the vehicle-present mark of the occupancy record from the old point's cell
  to *L*'s cell, and claim 20 holds for every kind. Our engine keeps no vehicle-present mark for
  aircraft (S-0209 gives the mark to ground objects), and an aircraft's layer at the old point
  and at *L* is not given.
- Claim 20 (e) plays a warp animation at the old point and at *L*. S-0202 claim 10 gives an
  animation's creation draws from the scenario random stream, while claim 20 (i) says the
  jump takes no draw and the only draw is the one at the end of phasing in.

## Questions

1. At which height level is the entry outcome of claim 20 (b) asked for the destination cell:
   the cell's own level always, the deck's level (4 levels up) in deck mode, or the level of
   the object's own height?
2. When the destination cell lies in no reach region for the object's zone, does the
   nearby-cell search of claim 20 (b) find any cell (and which), or does it give none, so that
   the destination is removed?
3. Does an aircraft with the teleporting locomotor move any occupancy mark at claim 20 (a), in
   which layer is it listed after the jump, and at what height does it land?
4. Do the two warp animations of claim 20 (e) take S-0202 claim 10's creation draws in the
   jump tick, or does claim 20 (i) mean that these animations are created without draws?

## Possible observation

S-0207 scenario D22 with the read-only recorder: record the random-stream position before and
after the jump tick of a vehicle whose type has the teleporting locomotor (question 4); order it
to a cell holding a vehicle on a bridge deck and to the ground cell under a deck (question 1); order
it to a water cell (question 2); and give a test aircraft type the teleporting locomotor and order
it to a cell (question 3).

## Current workaround

- Question 1: the destination cell's own level, plus 4 in deck mode.
- Question 2: the search runs with the "no region" value as its reach condition; whatever it
  gives decides, and no cell removes the destination.
- Question 3: an aircraft with the teleporting locomotor gets no locomotor in our engine and
  does not move by itself (as an infantry unit with it, Q-0181).
- Question 4: each warp animation takes S-0202 claim 10's creation draws.

Interim rows in `docs/development.md`.

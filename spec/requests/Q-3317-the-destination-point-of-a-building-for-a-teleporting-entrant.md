---
id: Q-3317
title: The destination point that a recycling building gives a teleporting entrant
status: open
related: [S-0603, S-0208, S-0207, S-0204, S-0212]
answered-by: []
---

## Need

S-0603 r15 claim 16 (b) says of a self-teleporting infantry unit sent into its own recycling
building: "The destination point is the building's centre, so the destination cell is the cell that
holds that point." For a 2 × 2 building the centre (S-0204 claim 4's reference point, the position
plus 128 × (W − 1, H − 1)) lies on the corner shared by the four foundation cells, so the cell that
holds it is the foundation's bottom-right cell. Claim 16 (e) says the same of a teleporting vehicle's
second order: it "lands in the cell of the building's centre".

The specs that own the landing-point choice read another point:

- S-0208 claim 19 takes *P* as "the destination point supplied to the locomotor". S-0208 Terms,
  *Locomotor destination*, make it, for a destination that names an object, "the object's movement
  point at that assignment". No spec we found defines a building's movement point as its centre.
- S-0207 claim 20 (b) speaks of "the destination cell" of an object destination without saying
  which point of a building gives it.
- Our engine gives every object destination the object's position (for a building, the centre of
  its top-left foundation cell, S-0204 claim 3), in the landing-point choice of both claims and
  elsewhere in movement.

With the engine's point the unit lands in the top-left foundation cell; with claim 16's it lands in
the bottom-right one. Both hold the building, so consumption (claim 16 c) follows either way, but
the landing cell, the standing position class that S-0208 claim 19 derives from *P*, and the
warp animation's place at *L* differ.

## Questions

1. For a teleporting entrant (infantry or vehicle) whose destination is a building, which point is
   *P* of S-0208 claim 19 and which cell is the destination cell of S-0207 claim 20 (b): the
   building's centre, its position, or another point?
2. Does the answer hold for every object destination that is a building (a building to capture,
   to enter or to attack-move to), or only for the recycle destination of S-0603 claim 16?
3. For a building with an even foundation side, whose centre lies on a cell boundary, which cell
   holds the centre?

## Possible observation

S-0603 scenario G12 part b with a 2 × 2 and a 3 × 3 recycling building, the unit started once from
each side; record the landing cell and the landing point of the jump tick.

## Current workaround

The engine keeps its point (the building's position) for both kinds. The S-0603 claim 16 tests
assert only that the entrant lands in a cell that holds the building, not which one.

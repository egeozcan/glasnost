---
id: Q-2841
title: The entry test at a ground vehicle factory's exit point
status: answered
related: [S-0309, S-0404, S-0209, S-0417]
answered-by: [S-0309]
---

## Need

S-0309 r126 claim 31 (*A unit that the factory has not yet sent out*) says that while a unit
that a vehicle factory released and did not send out stands at the factory's exit point, every
release that S-0404 claim 7 gives to the factory gives *failed*. It gives the reason: "The new
unit is put on the map only when the cell of its exit point answers *clear* to S-0209 claim 4's
entry test for it, and the stranded unit, a vehicle of the same house standing still there with
no destination, makes that answer *stationary friendly* (S-0209 claim 4)." The S-0417 claim 2
entry in S-0309's Changes for other specs repeats the result without the test.

S-0404 claim 13's ground vehicle factory row names no entry test. It puts the unit "inside *K*
at its *exit point*", and only the table's general sentence ("an appearing unit that cannot be
put on the map gives *failed*") could hold one. Our engine puts the unit there with no test
(`econ/release.rs`, `ground`).

Two things stop us building the test as the sentence reads:

- S-0209 claim 4 is the table for a walking infantry mover. The unit released by a ground vehicle
  factory is a vehicle, whose table is claim 5. The two differ for the stranded unit itself:
  claim 4's row WP16 (an allied vehicle with no movement in progress and no destination) gives
  *stationary friendly*, and claim 5's row VP20 gives the same for an allied vehicle that does
  not count as moving. They also differ for other occupants of that cell.
- The exit point lies inside the factory's foundation, so the factory itself is listed in that
  cell. Under claim 4, row WP19 gives an allied building *blocked*. Under claim 5, row VP19 gives
  an allied building that does not count as moving *blocked*, and row VP3 gives *none* only for
  a building the vehicle is in contact with. The new unit makes its contact with the factory only
  as part of the release, after it appears. Read either way, the factory would block every
  release, which contradicts S-0404 claim 13 and every factory release that the measurements
  show.

## Questions

1. Does a ground vehicle factory's release test the cell of its exit point before it puts the new
   unit there? If so, with which entry outcome table: S-0209 claim 4 (infantry), claim 5
   (vehicles), or another?
2. How does the factory itself, listed in the cell of its exit point, answer that test for the
   new unit: *none*, as a building it is in contact with, or by another row?
3. Which outcomes other than *clear* make the release give *failed*? For example: an enemy
   vehicle, an allied infantry unit, an allied vehicle that is moving, or a crate in that cell.
4. Does the same test apply when no unit is stranded there, for example when another of the
   house's vehicles drives over the factory's exit point at the moment of a release?

## Possible observation

S-0309 scenario A19 part k, with variants that put, instead of the stranded unit, an allied
vehicle moving across the exit point's cell, an allied infantry unit, or an enemy vehicle there at
the moment of a release (by a map trigger or a timed order), recording each release's result.

## Current workaround

Only the case that claim 31 states is built: a ground vehicle factory's release gives *failed*
when the cell of its exit point lists, in its ground layer, a vehicle of the factory's house,
other than the unit being released, with no movement in progress and no destination. No other
occupant changes the release, and the factory itself never blocks it. The rule is marked
`// design: Q-2841` in `econ/release.rs`.

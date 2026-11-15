---
id: Q-3134
title: The launch pass cadence of a spawner that takes no turn for ten ticks or more
status: open
related: [S-0506, S-0203]
---

## Need

S-0506 claim 2 says: "The first pass comes 20 ticks after the controller is
created; each later pass comes 10 ticks after the previous one." The glossary
entry *launch pass* says claim 2 owns the cadence and its exceptions, and claim
2 names none.

Our engine runs the controller only in the spawner's own turn. A spawner can
take no turn for ten ticks or more:

- a produced spawner is created when production starts and takes no turn until
  it is placed;
- a passenger of a transport that is not open-topped takes no turn;
- a mobile object that is being erased, or that is in a superweapon warp,
  leaves its turn early.

The engine now runs an overdue pass in the spawner's first turn after the gap,
at tick T, and schedules the next one for T + 1. That gives two passes one
tick apart (issue 343). It fits neither of the two readings below.

## Questions

1. When a spawner's pass is due while the spawner takes no turn, which of
   these does claim 2 mean?
   - (a) The passes keep their phase from creation + 20. The pass made in the
     first turn after the gap is followed by the next point of that phase
     after T.
   - (b) The next pass comes 10 ticks after the pass actually made: T + 10.
   - (c) The controller's passes run whether or not the spawner takes its
     turn, so nothing is overdue (and then: in which phase of the tick, and
     what does a pass do for a spawner that is off the map?).
2. For a produced spawner, does "created" mean the start of production (when
   the object is made) or the placement at the factory?
3. Are the answers the same for the four cases above: production, a closed
   transport, erasure and a warp?

## Possible observation

Produce a spawner vehicle of a named test type with a production time of 30
ticks or more, give it an attack order at the tick of its release, and
record the tick of each target adoption and of each launch. Repeat with a
spawner that boards a closed transport for 25 ticks with a target held, and
with one that a hostile erasing unit locks for 25 ticks.

## Current workaround

Unchanged: an overdue pass runs in the spawner's first turn after the gap and
the next one is due at that tick + 1, after which the 10-tick spacing
resumes. Issue 343 stays open until this is answered.

---
id: Q-2711
title: A rocket missile whose destination object is removed before its first move
status: open
related: [S-0506, S-0215, S-0309, S-0414]
answered-by: []
---

## Need

S-0506 claim 7 gives a launched rocket missile the controller target as its destination and the
move behaviour. Claim 14 fixes its aim point "when it first carries out its move behaviour, using
the destination given in claim 7", from the destination's heading point. Claim 24 puts that first
rocket update in the tick the aim is fixed. The specs do not say what happens when the destination
is an object that is removed after the launch and before that first move: the destination then
names an object that no longer exists, so it has no heading point.

S-0309 claim 27 drops only targets when an object is removed. No claim found clears another
object's *destination* naming it, for a rocket missile. S-0501 claim 32 does so for an aircraft,
but S-0501's Terms leave rocket missiles out. Claim 26's shared re-orders set the missile's target
to its release cell and queue *attack*, but claim 14 says later move orders are ignored, and the
flight only starts from the destination of claim 7.

Seen in the local lane (intake 104, an AI game of two houses on an install map; not reproduced on
main at bf6a19b, where that game takes another course):
- An infantry unit on a sabotage order was the controller target of a spawner with a rocket slot.
- In tick *t*, the launcher placed a rocket missile and released it. The release cell was the
  unit's cell, so the unit still existed at the release (claim 12).
- In the same tick the unit went inside its target building and was removed.
- The missile then stood on the map at its placement point, in *move*, with that removed object as
  its destination. It never paused, tilted or climbed, its speed stayed 0, and it never detonated
  (claim 19 tests only flying phases).
- Every 30 ticks it got the shared re-order of claim 26 (target its release cell, *attack*
  queued), with no effect.
- Two other missiles of that game were found in the same state, one of them of another rocket
  type, each with a removed object as its destination. When their destinations were removed was
  not traced.
- Later, at health 0 from a hit, it still stood there (claim 20 keeps a destroyed rocket above
  the terrain in play). It was still there at tick 110 000, about 75 000 ticks after the launch.

Our engine: `rocket::tick` (`crates/deterministic/sim/src/rocket.rs`) starts the flight cycle only
when the destination has a heading point. A missile whose destination is gone stays as described.

## Questions

1. When a rocket missile's destination object is removed after the launch and before the missile
   first carries out its move behaviour, does the missile still start its flight cycle (pause,
   tilt or raise, ascent, cruise)? If so, where does it aim:
   - the destination's last position or heading point;
   - its release cell (claim 12);
   - the neighbouring cell of claim 12's no-target case;
   - or somewhere else?
2. If it does not start, does it stay on the map at its placement point indefinitely? Or is it
   removed, or does it detonate at some point (and if so, when)?
3. Does the answer depend on how the destination left:
   - destroyed and still in play (a sinking or crashing object, an infantry death sequence);
   - removed;
   - gone inside a building or transport (off the map but still existing)?
4. Does a missile that never starts count as an aircraft on the map for its house's survival test
   (S-0414 claims 3 and 4)? A house whose last object it is would then never be defeated.

## Possible observation

Use a spawner with a rocket slot whose launch reaches the placement in the same tick as a removal:
- Give the spawner an attack order on an engineer that will enter a building it can capture in the
  tick the spawner launches.
- Or give it an order on a unit that a second weapon destroys in that tick, so that it leaves play
  at once.
- Record per tick, for each missile:
  - its position, height and pitch;
  - its phase as seen in its drawing;
  - when it leaves play;
  - any detonation.
- Repeat with the target destroyed (a death sequence) rather than removed.

## Current workaround

None in the engine: the missile stays on the map in *move*, as described. No interim is added.
Our engine reached this only through a rare tick coincidence on an install map, and main's games
do not show it.

---
id: Q-3239
title: A crashed infantry unit whose air-death-end request is turned down, and the crash end's height on a high bridge
status: answered
related: [S-0312, S-0107, S-0502]
answered-by: [S-0107, S-0312]
---

## Need
S-0312 claim 16 (*Infantry crash end*) says that an infantry unit whose type can crash, and whose
crash ends in jumpjet movement's *crashed* state, is put at ground height and makes the forced
request for *air-death-end* (S-0107 claims 6 and 21). When that sequence starts, its completion
removes the unit with no corpse. The claim leaves two points open, and its Open questions list
both:

- what removes a unit whose request is turned down;
- the height the unit is put at in a high-bridge cell. The *Vehicle crash end* has the same open
  point.

S-0107 claim 6 turns a start request down in several rows. The one that data can reach here is
row 1: the type's own *air-death-end* entry has C = 0. Art that gives a jumpjet infantry type no
air-death frames does this. Row 4 (the resulting kind is already current) does not arise at a
crash end, since the unit's sequence then is *air-death-start* or *air-death-fall*. The engine
must decide whether such a unit stays in play, and if so what ends it.

Sim batch S32 (#252) builds the infantry crash end in `jump::impact` and
`combat::death::infantry_death_turn`.

## Questions
1. A crashable jumpjet infantry type has an *air-death-end* entry with no frames. One of its units
   is killed in the air, falls and hits the ground. Does the unit stay on the map after the
   impact? If it stays, for how long, and does it leave a corpse when it goes? Can it be selected,
   targeted or run over meanwhile?
2. Does the answer change when the type also has no *air-death-start* or *air-death-fall* frames,
   or when the impact comes while one of those is still playing?
3. A crashable jumpjet unit (infantry, and a vehicle for the *Vehicle crash end*) is killed above
   a high-bridge cell and its crash ends there. Is it then at the terrain height under the bridge,
   or at the deck height? Does it matter whether it fell onto the deck or past it (S-0502 claim
   18's deck crossing)?

## Possible observation
Give a crashable jumpjet infantry test type an *air-death-end* entry with no frames. Shoot one
down over open ground and record, tick by tick after the impact, whether it is still drawn,
selectable or targetable, and when it disappears. Repeat with the type's ordinary art as control,
which should remove it when *air-death-end* completes. For question 3, shoot down the same type
and a crashable jumpjet vehicle above a high bridge, once while it is above the deck and once
while it is between the deck and the water or ground, and record the height at which each stands
when it is removed.

## Current workaround
A unit whose request is turned down leaves play at its crash end with no corpse, as every crashed
infantry unit did before S32 (`design: Q-3239 interim` in `jump::impact`). In a high-bridge cell
the unit is put at the surface height under it, as the vehicle crash end already does.

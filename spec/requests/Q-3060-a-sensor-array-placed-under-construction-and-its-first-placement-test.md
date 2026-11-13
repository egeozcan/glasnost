---
id: Q-3060
title: Is a sensor-array or disguise-detector building placed under construction "in operation" at its first placement?
status: open
related: [S-0516, S-0405, S-0406, S-0215, S-0415]
answered-by: []
---

## Need

S-0516 claim 23 gives a sensor-array building its sensor disc "only at its first placement, and
only if it is in operation at that moment", and an array "placed unpowered never senses, even
after power returns" (`quirks.sensor_building_activation_once`). Claim 40 applies the same
activation rule to a disguise-detector building.

Every building that enters play by a place order or a deploy enters under construction. S-0406
claim 17: "At entry it has no current behaviour and *construction* queued. A rule that asks for
its behaviour while none is current sees the queued *construction*." S-0215 claim 1 has the same
general rule. The in-operation test of S-0405 claim 8 has the row "not building or selling: its
current behaviour is *construction* or *selling*", and then says: "The last row reads the
building's current behaviour only, not its queued behaviour."

Those two sentences give opposite answers for a building at its entry under construction:

- by S-0405 claim 8 it passes the last row (no current behaviour), so an array placed with power
  to spare is in operation at its first placement and senses;
- by S-0406 claim 17 the row sees the queued *construction*, so no building placed by order or
  deploy is ever in operation at its first placement, and under claim 23 no such array or detector
  ever senses or detects. Its later removal still decrements its owner's counts with no floor
  (claim 23, **Removal**), leaving its cells at −1.

S-0405 claim 8's backing says its scenario W6 "tests current behaviour only" and leaves the
queued case `unmeasured`. S-0516's wording ("placed unpowered", CS12 (c)'s "placed operational")
treats power as the deciding condition and does not mention the construction state.

GitHub issue #168 reports the second reading as a bug: with the engine as it is (S-0406
claim 17's reading), every sensor array and disguise detector built in play is inert.

## Questions

1. A sensor-array building placed by a place order, with its owner's power sufficient and every
   other condition of S-0405 claim 8 met: does it add its sensor disc at its placement (so that a
   cloaked enemy unit next to it is sensed once the build-up has ended)?
2. The same for a disguise-detector building (S-0516 claim 40).
3. The same for each of them entering play by a deploy (S-0415 claim 6).
4. If the answer to 1 is no: does such an array ever add its disc (for example at the end of its
   build-up), or is it inert for the rest of the game? And is its removal's decrement then still
   applied?
5. Which reading of S-0405 claim 8's last row applies to the other rules that ask for the in-operation
   test at a building's entry (power output and drain at entry, S-0405 claims 1 and 2)?

## Possible observation

S-0516 scenario CS12 (c)'s "placed operational" run, with the array explicitly placed by a
place order: a powered house places the array, a cloaked enemy unit waits beside it, and the
recorder logs whether the unit is sensed once the build-up has ended. Repeat with a disguise
detector and a disguised enemy infantry unit (CS15 (b)), and with a deploy.

## Current workaround

None: the engine keeps S-0406 claim 17's reading (the queued *construction* fails the last
row), so an array or detector placed under construction does not activate. A building put on the
map without the construction state (map objects at game start) activates as before. No code
change for #168 until this is answered.

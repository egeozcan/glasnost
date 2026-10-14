---
id: Q-0181
title: How a unit with the teleporting locomotor moves (chrono miner, types without a locomotor field)
status: answered
related: [S-0207, S-0215, S-0402, S-0212]
answered-by: [S-0208]
---

## Need
S-0207 claim 14 gives every infantry, vehicle and aircraft type without a
locomotor field the teleporting locomotor, and S-0402 claim 17 has the chrono
miner reach its field and refinery "by teleporting", saying that how it
teleports is the movement specs'. S-0215 (Terms, *chrono move*) leaves the
move, its leaving stage and phase-in and their lengths to the superweapon and
locomotion topics. No approved spec gives that movement, so the simulation
does not move such units at all (`glasnost_sim::rules::Locomotor::None`): an
ordered chrono miner keeps its destination and stays where it is, and on an
install the scripted skirmish of a country whose only buildable harvester is
a chrono miner earns no credits.

## Questions
1. When a unit with the teleporting locomotor is given a destination, what
   happens tick by tick: does it turn, wait, or start the jump at once; how
   long is the leaving stage and the phase-in, and are these lengths fixed or
   do they depend on the distance or a rules value?
2. Where does it reappear: exactly the destination cell, or a nearby cell
   when the destination is occupied or not passable for it (which search,
   which tests)? Can it jump onto cells it could not drive to (cliffs, water,
   other islands)?
3. Is there a delay between successive jumps, and what does the unit do
   while it waits (can it be ordered, fire, be hit, be crushed)?
4. What lepton position, facing and height does it have at each stage, and
   which cells does it occupy during the jump (S-0209 occupancy)?
5. For a target that moves, or a destination given while a jump is under
   way, what happens?
6. Which random draws, if any, does a jump take?

## Possible observation
A tiny map with one chrono miner and one ordinary harvester, both ordered to
the same far cell; record per tick the positions and occupancy, and repeat
with the destination blocked and on another island.

## Current workaround
Units with the teleporting locomotor are not moved. `glasnost_ai`'s scripted
opponent uses only harvester and army types the simulation moves, and the
local lane's scripted skirmish picks countries that can build such a
harvester.

---
id: Q-0120
title: Where a factory building puts a released unit, and when it retries a blocked exit
status: answered
related: [S-0404, S-0411, S-0212, S-0213]
answered-by: [S-0404]
---

## Need
S-0404 claim 7 releases a done unit from the chosen factory building and
says that the structures specs own exits: "When the building's exit is
momentarily blocked, the unit stays done and the building retries by
itself" (also Open question 2). No approved spec says where the released
unit appears, which facing and behaviour it gets, what makes an exit
blocked, or when the building tries again. The release in
`sim::econ::production::release` needs all of these.

## Questions
1. For each kind of factory building (infantry, ground vehicles, naval
   vehicles, aircraft), at which cell (or position) does a released unit
   appear relative to the building's foundation, and does it then move on
   to another cell by itself (for example out of a door, or to the
   building's rally point, S-0213 claim 20)?
2. Which facing does the unit have when it appears, and which behaviour
   (guard, area guard, a move to the rally point)?
3. When is the exit blocked: which objects or terrain in which cells make
   the release wait?
4. When the exit is blocked, in which ticks does the building try again
   (every tick, in its own turn, after a delay), and does a try take any
   random draw?
5. Does the release itself take any draw from the scenario random stream
   (S-0404 claim 11 leaves the release's draws to the structures specs)?

## Possible observation
A vehicle factory building with its surrounding cells filled by test units in
turn, recording the tick and cell at which the released vehicle appears, its
facing, and the random-state recorder's stream position per tick.

## Current workaround
A placeholder, noted at run time: the unit is put on the map at once, facing
0, in the cell that the nearby-cell search of S-0212 (claim 25, its scatter
use, with the unit's locomotion class) finds around the cell just beyond the
foundation's far corner (top-left cell plus the foundation's width and
height). When that search finds no cell the exit counts as blocked, and the
release is tried again in the production part of every following tick.
Listed in docs/development.md, provisional rules.

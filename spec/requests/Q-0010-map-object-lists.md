---
id: Q-0010
title: Field layout and meaning of a map's object lists
status: open
related: [S-0009, S-0102, S-0107, S-0108]
answered-by: []
---

## Need
S-0009 claim 27 says the object lists are read, and its Import mapping
says their formats have no spec yet. The glossary's claim G1 gives only the
sixth value of a `[Units]` entry. The viewer must place every pre-placed
object and draw it in its initial state. Code gap:
`glasnost_convert::map` (object lists) and `glasnost_map::Objects`.

## Questions
1. For `[Units]`, `[Infantry]`, `[Aircraft]` and `[Structures]`: which
   comma-separated value is the owner, the type, the health, the cell x and
   y, the facing, and what are the remaining values? What happens to an
   entry with fewer values, or a value that does not parse?
2. The health value: what scale is it on (what value means full
   strength), and how does it become the health whose ratio S-0108 F3 and
   claim 2 compare with the condition ratios? Is it rounded, and can it
   exceed full strength?
3. Infantry: which value is the position within the cell, what are its
   possible values, and which world position (leptons) does each give?
   Which value is the facing, and on what scale?
4. Structures: is the cell given the top-left cell of the foundation? Is
   the facing value used (for example by a turret)?
5. Aircraft: position and height when placed by a map.
6. `[Terrain]`: is the key a cell number decoded like a waypoint
   (S-0009 claim 15) and the value the terrain object's type name? Where in
   the cell does the object stand?
7. `[Smudge]`: field layout, and which cell(s) a smudge covers.
8. Owner names other than the houses of claim 27 are dropped in skirmish;
   what does a type name that names no type of the kind do (dropped, or an
   error)?

## Possible observation
A test map with one object of each kind at known cells, health values
256, 128 and 1, every infantry position value and several facings;
screenshots and the in-game health bar show the result.

## Current workaround
Field positions follow OpenRA's `ImportGen2MapCommand` (GPL-3), a file
loader: owner, type, health, x, y; facing sixth for units, aircraft and
structures, eighth for infantry, with the infantry position sixth. Health
is kept as the raw integer; the viewer shows a building damaged only if a
spec gives the scale, and otherwise draws it undamaged and lists the
health in the object info. Infantry are drawn at the cell centre whatever
their position value, marked provisional. Terrain objects and smudges are
placed at their cell's centre.

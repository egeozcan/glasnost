---
id: Q-0145
title: When and where a building's free unit appears
status: open
related: [S-0402, S-0404, S-0406, S-0408, S-0215]
answered-by: []
---

## Need

S-0408 and S-0404 name a building type's free unit (the vehicle type its free-unit field names)
and use its cost. S-0215 claim 15 row 4, the placement fall-back, gives *harvest* to a harvester
that a building places. But no approved spec says:

- that a free unit is created at all;
- when it is created (on placement, at the end of construction, or at another time);
- who owns it, which cell it appears in and which facing it has;
- whether its creation takes draws.

The engine needs this to give a newly placed refinery its harvester. `econ::harvest` is ready for
it through `process::fall_back(w, id, true)`.

## Questions

1. When is a building's free unit created: when the building is placed, when its construction
   ends, or at another moment? Is it also created for a building the map pre-places, for a
   captured building, and for a building that a deploying vehicle creates?
2. In which cell does it appear, relative to the building's foundation, and with which facing?
   What happens if that cell is blocked?
3. Which behaviour does it start with? Does its creation take draws beyond the object creation
   draw (S-0202 claim 10)?

## Possible observation

Place a test refinery type that has a free unit, once on an open map and once with the cells
around it blocked. Record the objects created on each tick, with their cells, facings and
behaviours.

## Current workaround

No free unit is created. A harvester given the placement fall-back
(`process::fall_back(w, id, true)`) starts *harvest*, so a later free-unit rule only needs to
create the unit and make that call. This is listed in the provisional table of
`docs/development.md`.

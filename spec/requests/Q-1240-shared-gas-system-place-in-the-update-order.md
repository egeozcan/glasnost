---
id: Q-1240
title: Where the shared gas system enters play at game start
status: answered
related: [S-0613, S-0203, S-0109, S-0010]
answered-by: [S-0613]
---

## Need

S-0613 claim 1 says the shared gas system is created "when a game starts" and takes turns as a
particle system (S-0203 claim 2). Its turn is where every virus cloud hits and draws, so its
place in the update order decides the order of those hits and draws against every other
object's turn. S-0203 claim 3 gives the game-start order of map-placed objects (terrain
objects, vehicles, aircraft, infantry, buildings, then the skirmish starting units) and says
an object of one kind never stands between two map-placed objects of another kind, but it
does not name the shared gas system.

Map loading can also deliver particles into the shared system: S-0109 claim 14 sends every
middle-event particle there, and a tile or cell animation with μ = 0 has its middle event at
creation during loading (S-0010 claim 19, S-0009 claim 12). So the system may need to exist
before the map's cells are processed.

The code is `crates/deterministic/sim/src/gas.rs` (`create_shared`), called from
`crates/deterministic/sim/src/load.rs` (`new_game`).

## Questions

1. At game start, does the shared gas system enter the update order before every map-placed
   object, after the skirmish starting units, or somewhere else?
2. Does it already exist while the map is loaded, so that particles requested by a μ = 0 tile or
   cell animation during loading are created in it (and later receive gas updates)? If it does
   not exist yet, what becomes of those particles?

## Possible observation

A map with a tile animation whose type spawns a gas particle at μ = 0: watch whether that
particle drifts and damages after the game starts. For the order: a virus cloud killing a
unit in the same tick as the last map-placed building fires, recording which draw comes first.

## Current workaround

The shared system is created before the map is read, so it is first in the update order and
takes particles from loading-time middle events. Docs: docs/development.md, *Simulation (M7a:
gas clouds)*.

---
id: Q-0016
title: The initial state of objects a map places
status: answered
related: [S-0105, S-0108, S-0102, S-0009]
answered-by: [S-0009]
---

## Need
The map viewer draws the map's initial state, before any tick. S-0108
claim 6 says state changes during the game's set-up are immediate, but no
spec says which state a building placed by the map is in then; and S-0105
needs some starting values that no spec gives for a map-placed object.
Code gap: `glasnost_app::mapview::build`.

## Questions
1. A building placed by the map: is it complete at once (never in the
   *construction* state), and in which body animation state does it start
   (*idle*, or another)? Does its grand opening (S-0108 claim 11) happen at
   placement, so that its slot animations start then?
2. A vehicle placed by the map with a turret: what is its turret's starting
   facing (the body's facing, or a fixed value)?
3. A building's voxel turret: which facing does it start with (the map
   entry's facing value, on which scale)?
4. "The building's draw point" (S-0105 Terms, turret anchor): which screen
   point is it? Is it the image anchor of S-0108 claim 8 (the foundation's
   top corner), or another point?
5. In which order are the map's objects placed relative to each other
   (buildings, vehicles, infantry, aircraft, terrain objects), where that
   order shows (S-0102 claim 6: equal sort keys keep their insertion order;
   claim 16: buildings are painted one after another)?

## Possible observation
A test map with a building whose idle and active ranges differ, a turreted
vehicle at facing 64, and a defence with a voxel turret at facing 128;
screenshot the first visible tick.

## Current workaround
Buildings start complete in the idle state at stage *a* of the idle range,
with no slot animations drawn. A vehicle's turret faces as its body. A
building's voxel turret takes the map entry's facing value × 256, and the
turret anchor is the S-0108 claim 8 image anchor. Objects are placed
structures first, then vehicles, infantry and aircraft, each in entry
order. All marked provisional.

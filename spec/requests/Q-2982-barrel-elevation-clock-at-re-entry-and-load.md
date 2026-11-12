---
id: Q-2982
title: When does a non-building object's barrel-elevation turn start, after re-entering the map or a load?
status: answered
related: [S-0105]
answered-by: [S-0105]
---

## Need

S-0105 claim 6 says that when a vehicle or other non-building object is placed on the map, its barrel elevation starts at 16384 and is then turned toward (16384 − 256 × A) mod 65536 at turn rate 3. MS-14 measures the steps "from placement" for a vehicle the map places.

Render issue #81 (GitHub, render audit #154) found that the client counted the turn from tick 0 for every object, so a vehicle built later was drawn with its barrel already turned. The fix counts from the tick the view first sees the object on the map. The claim does not say when the clock starts in a few cases, and the client needs the answer for them.

## Questions

1. An object leaves the map and comes back without being created again. Examples are a passenger unloaded from a transport, a unit leaving a building it entered, an object returned by a chronosphere warp, a unit put down by a carrier, and a dropped object landing. Does each of these place it on the map in claim 6's sense, so that its elevation is set to 16384 again and turns from that tick? Or does it keep the elevation it had when it left?
2. A newly made object enters the map during a tick: a unit leaving a factory, or a unit dropped by a paradrop. Is the first tick of the 0, 0, 1, 1, … sequence the tick it was placed in, or the next tick?
3. A saved game is loaded, or a player joins a network game in progress. Is each object's barrel elevation part of what is saved and sent, so that the turn continues from where it was? Or does every object start level again, or show the final elevation at once?

## Possible observation

For question 1, give a vehicle type with a barrel part a fire angle of 20, as in MS-14, and load it into a transport. Let the transport wait until well past 6 ticks, then unload it. Capture each tick from the unload and record the barrel's 32-step pitch: 8, 8, 7, 7, 6, … means the turn starts over; 6 from the first capture means the elevation was kept. Repeat for a unit leaving a building and for a chronosphere warp.

For question 2, build such a vehicle at a factory and capture from the first tick it is drawn outside the factory.

For question 3, save the game within 6 ticks of placing such a vehicle, load the save, and capture the first ticks after the load.

## Current workaround

`crates/app/src/mapview/live.rs` (`Live::note_placements`, marked `design: render (#81)`) does the following until this request is answered:

- Objects on the map when the view starts count from tick 0. This covers a loaded or joined game, whose barrels show their final elevation at once.
- Any other object counts from the first tick the view sees it on the map, and is drawn level on that tick.
- An object that leaves the map and comes back starts level again.

The interim is recorded in `docs/m8-claims/render-issues.md` (batch R3, #81).

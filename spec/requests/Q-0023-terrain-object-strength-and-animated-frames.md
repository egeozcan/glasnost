---
id: Q-0023
title: Terrain object strength when the type omits it, and the frames of animated terrain objects
status: answered
related: [S-0102, S-0009]
answered-by: [S-0102, S-0009]
---

## Need

S-0102 claim 23 draws a terrain object with image frame 0 while its
health is 2 or more and frame 1 below 2, and a terrain object starts with
its type's strength (S-0009 claim 42). The Import mapping of S-0102 gives
the strength key's absent default as "as the rules specs say", but no
approved spec says it. Most terrain types of a typical install give no
strength key, so the viewer cannot decide frame 0 or 1 for them, and the
simulation cannot give terrain objects their starting health. Claim 23
also leaves the body frame of an animated terrain object (its current
animation stage) unspecified. Code: `app::mapview::build`,
`Builder::terrain_objects`; `glasnost_sim::objects::create`.

## Questions

1. What strength does a terrain object type have when no rules layer gives
   its strength key? Does a layer's empty or unreadable value keep the
   held value?
2. Which image frame does an animated terrain object show when it is
   placed by the map, and how does it advance (rate, frame count, loop)?

## Possible observation

1. A test map with a terrain type whose strength key is removed through
   the map's override section: whether the object shows frame 1 (strength
   below 2) and how many hits of a known damage destroy it.
2. A map-placed animated terrain object observed over consecutive ticks.

## Current workaround

1. The viewer draws such a terrain object with image frame 0 and notes
   Q-0023; the simulation gives terrain objects health 0 (its rules carry
   no strength field yet).
2. Animated terrain objects show image frame 0, with a note citing Q-0023.

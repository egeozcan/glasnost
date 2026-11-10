---
id: Q-1702
title: Voxel animations (debris) in play: flight, landing, removal and attached particle systems
status: answered
related: [S-0303, S-0312, S-0616, S-0007]
answered-by: [S-0317]
---

## Need

S-0303 F6 and claim 22, and S-0312 F1 and claim 28, say how many voxel-animation pieces a detonation or
a destruction makes, and of which types. S-0007 says how their art is found. No approved spec says
what a piece does once it exists. The engine makes each piece as a presentation effect at its start
position (`glasnost_sim::combat::effects::anim`) and keeps no object for it. The seam
`glasnost_sim::emitters::voxel_animation_system` (S-0616 claim 1, voxel-model animation row) is never
called, since there is no object to own the system and no import key names it.

## Questions

1. **Objects of play or presentation.** Is a piece an object of play, which takes part in later random
   draws and can hit things, or only a picture?
2. **Flight.** How does a piece move from its start: its first speed and direction (any random part, and
   from which stream), its fall, its spin? Which type keys set these, and what is each key's default when
   absent?
3. **Landing.** What happens when a piece reaches the ground or water: does it bounce, and how many
   times? Does it stop, sink, make an animation or a sound, or deal damage (with which warhead, to whom,
   and on behalf of which house)?
4. **Removal.** When is a piece removed: on landing, after a time, or on leaving the map? Is there a cap
   on the number of pieces in play, and what happens at the cap?
5. **Attached particle system.** Which type key names a piece's particle system? Where is the system
   made, does it follow the piece, and when is it marked as ending (S-0616 claim 4)?
6. **Bridges, cliffs and buildings.** Does a piece in flight stop at a cliff or a building, or land on
   a bridge deck?

## Possible observation

Destroy a vehicle type with debris types and maxima set, on flat ground, near water, near a cliff and on a
bridge. Record each piece's path, its landing and what it leaves. Repeat with a debris type that names a
particle system.

## Current workaround

Each piece is a presentation effect at its start position with no flight. The voxel-animation particle
system seam is never called.

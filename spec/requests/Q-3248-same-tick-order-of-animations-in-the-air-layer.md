---
id: Q-3248
title: Where animations made in a tick enter the air layer among that tick's particles, projectiles and debris
status: answered
related: [S-0102, S-0616, S-0109, S-0306, S-0317]
answered-by: [S-0102]
---

## Need

S-0616 claim 30 and S-0102 claims 3 and 7 say an object entering the air layer goes to the end
of its list, so the air-layer objects made in one tick are drawn in the order they were made,
"interleaved with the other air-layer objects (projectiles, animations) by the same rule".

For particles, projectiles and voxel debris pieces the engine can recover that order: one object
counter numbers them as they are made. Since issue #432 the drawing places them in that order
(`Live::sync_entrants` in `crates/app/src/mapview/live.rs`).

Animations are different, for two reasons:

- An animation made by a presentation event (a warhead's explosion, a shot's muzzle animation,
  a move marker) has no object number. The drawing makes it when it reads the tick's events, and
  nothing records when in the tick it was made compared with that tick's particles and
  projectiles.
- An animation the simulation keeps (S-0109 claims 28 and 29) gets its object number after its
  start event has run (S-0109's start event, which can make particles through S-0616). So
  particles made by an animation's own start get lower numbers than the animation. Object order
  would put those particles before the animation, and we do not know whether that is right.

Until this is answered, one tick's new animations go into the air layer first, by creation,
before that tick's new particles, projectiles and debris pieces. This only matters where an
animation and one of those objects made in the same tick overlap on screen.

## Questions

1. An animation whose first frame makes a particle system (for example an animation type that
   names a particle system on its first frame): in the air layer, is the animation drawn under
   or over the particles that system makes in the same tick?
2. A warhead detonation in the air that makes both an explosion animation and particles (or
   debris pieces) in the same tick: is the explosion drawn under or over them?
3. A shot that launches a projectile and makes a muzzle or firing animation in the same tick: is
   the projectile drawn under or over that animation while both are in play?
4. More generally: does an animation enter the air layer when it is made, in the same sequence
   as projectiles, particles and debris made in that tick, or at another fixed point of the tick
   (for example all animations before or after the other objects)?

## Possible observation

Use a synthetic air-layer animation with a large opaque frame and a particle system with a
contrasting opaque sprite on the same point, made by one detonation in one tick, and see which
one shows where they overlap. Repeat with a projectile fired in the same tick as a muzzle
animation at the same screen point.

## Current workaround

`Live::sync_entrants` places one tick's new animations first, by creation (the drawing's own
serial order), then that tick's particles, projectiles and debris pieces by object number. It is
marked `design: render (#432)` and cites this request.

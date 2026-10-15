---
id: Q-0264
title: Which sound plays when a weapon fires, when an object dies and when a building is damaged?
status: answered
related: [S-0301, S-0312, S-0302, S-0202]
answered-by: [S-0701]
---

## Need

The simulation emits a presentation event for every shot that leaves a firer (S-0301) and every
killed object (S-0312), and S-0302 claim 12 mentions a building's damage sound at *now yellow* and
*now red*. S-0312 says the sounds a death plays draw from the unsynchronised random stream. No
spec's Import mapping gives the keys these sounds come from, so `glasnost-audio` plays nothing for
them.

## Questions

1. When a weapon fires, which sound plays (a key of the weapon, of the firer's type, or both)?
   Where (at the firer, at the fire point)? Does a list of sounds pick one per shot, and how? Is
   there a separate sound for a rapid-fire weapon's spin-up (S-0202 claim 11 mentions one)?
2. When an object is killed, which sounds play: a die sound of its type, a voice of the dying
   unit, the explosion animations' start sounds (S-0109), or several of these? Which keys, and
   which draws from the unsynchronised stream?
3. Which key gives a building's damage sound at *now yellow* and *now red* (S-0302 claim 12)?
4. Does the owner of the object change any of these (only the owner hears)?

## Possible observation

A test weapon and test types given distinct sound names in a loose rules copy, fired and killed
in view, recording what is heard and when.

## Current workaround

`Effect::Shot` and `Effect::Killed` play nothing. The start sound of an animation that a death or
detonation creates is played, as S-0109 claim 13 states, from the animation's `Report`.
Documented in docs/development.md, "Audio (M5b)".

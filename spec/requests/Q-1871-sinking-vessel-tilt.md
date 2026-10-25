---
id: Q-1871
title: Does a sinking vessel's body tilt (list, pitch or roll) as it goes down, and how?
status: answered
related: [S-0504, S-0106, S-0105, S-0312]
answered-by: [S-0504]
---

## Need

The lead's play note of 2026-10-25 (M8 step A4): "the ships do not sink with an angle". Our build
lowers a sinking vessel by S-0504 claim 4's descent (5 leptons a tick until below −400) and draws its
body with the same transform as before the killing hit, so it goes straight down level.

No approved spec gives a sinking vessel a tilt:
- S-0504 claim 10 says the sinking state is "distinct from the visual list during descent
  (S-0106)", which reads as if S-0106 owns a list (a lean) of the descending vessel.
- S-0106 has no sinking rule. Its *Rocking* term is a pitch or roll that "its locomotor applies",
  and claim 9 leaves the causes of pitch and roll to the vehicle specs; S-0504 names none for
  sinking.
- S-0105 builds the body transform from facing, slope tilt and rocking only.

The renderer (`crates/app/src/mapview`, the voxel body transform) and possibly the simulation (if the
tilt is locomotor state that other rules read) need the rule.

## Questions

1. Does a vessel that starts sinking at a killing hit (S-0312 claim 10, S-0504 claim 4) tilt its
   body while it descends? About which axis (pitch along its heading, roll across it, or both), in
   which direction, and is the direction fixed, taken from its facing, or random (which stream)?
2. How does the angle change over the descent: a rate per tick, a fixed target angle, an
   easing? Is it the same for every vessel type, or does a type key set it?
3. Does the tilt apply to vessels set sinking by other causes (S-0614 claim 11, S-0607's drop into
   water, S-0502 claim 19), to underwater types, and to a vessel sinking from a height other than 0?
4. Is the tilt drawing only, or is it locomotor state that the simulation reads (S-0106 *Rocking*:
   for example the 0.005-radian rocking test, the image cache, shadows, or the turret and fire
   positions of S-0313)?
5. Is this what S-0504 claim 10's "visual list during descent (S-0106)" refers to?

## Possible observation

Sink a destroyer and a submarine-type vessel on open water, facing different directions, and
record the body's angle every few ticks from the killing hit until it leaves play.

## Current workaround

The body keeps its pre-hit transform while it descends (no tilt). No switch is added: there is no
approved rule to switch between.

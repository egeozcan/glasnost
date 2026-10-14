---
id: Q-0217
title: The start-event draws of an animation created during play
status: open
related: [S-0109, S-0202, S-0312, S-0401]
answered-by: []
---

## Need

S-0109 claim 13 gives an animation created with no loop delay a start event
at creation, and with μ = 0 that start event is the immediate middle event of
claim 14. S-0202 claim 15 says which draws that middle event takes (the
requested particles, the scorch-or-crater choice below 30 leptons above the
terrain, a crater's removal of bales) and in what order, but only for
animations created during map loading. Animations are also created during
play: warp-in and warp-out animations of the teleporting locomotor (S-0207
claim 20), weapon and warhead animations (S-0301, S-0303), and death and
debris animations (S-0312). `load::anim_creation_draws` has to decide which
draws such an animation's start event takes.

## Questions

1. When an animation with no loop delay and μ = 0 is created during play,
   does its immediate middle event take the same draws, in the same order, as
   S-0202 claim 15 gives for one created during loading (particles, then the
   scorch-or-crater choice, then the crater's bale removal)?
2. If not, which of those draws does it take, and in which random stream?
3. Does the 30-lepton height test use the same terrain height (S-0106) during
   play as during loading?

## Possible observation

Create one animation type with both the scorch and crater flags and a
particle request, play it as a weapon's detonation animation in a recorded
game, and compare the random draws (or the scorch, crater and particle
outcomes over many seeds) with the same type started at map load.

## Current workaround

An animation created during play takes its creation draws only (S-0202 claim
10). Its start event's particle, smudge and crater draws are not taken, and a
note is written.

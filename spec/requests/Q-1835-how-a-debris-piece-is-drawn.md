---
id: Q-1835
title: How a debris piece with a voxel model is drawn (pose, spin, light, palette, depth, shadow)
status: open
related: [S-0317, S-0007, S-0102, S-0105]
answered-by: []
---

## Need
S-0317 claim 15 says the spin rate turns the drawn model each tick about the spin axis, S-0007
claim 30 names the model (its own model with no motion file, or a shared part), and S-0102 puts a
voxel animation in the air layer. Nothing says how the model is drawn. `glasnost-app`
`mapview::build::live_piece` and `mapview::live::piece_pose` must choose a pose, a light, a
palette, a depth rule and a draw point, so the drawing is our guess until a spec states it.

## Questions
1. The pose: in what orientation is the model drawn at its creation (before any spin), and how
   does the spin turn it: about which axis in the model's frame (the spin axis of F5 normalised,
   or something else), by the spin rate times the ticks since creation or by another amount,
   and does the turn build up smoothly or in steps?
2. The light: is the model lit as a vehicle body is (S-0105), as a voxel projectile is (S-0306
   F16), or in another way, and does the light turn with the model?
3. The palette: which palette and brightness does the model use: the house colour of the piece's
   house (and what for a piece without a house), the unit palette, or another? Is it lit by the
   cell's lighting?
4. The draw point and depth: is the model drawn at the piece's position with its height lift, and
   does it read or write the depth buffer (against buildings and terrain), with what bias? Is it
   drawn while it is under the shroud or fog?
5. Does a piece cast a shadow on the ground?
6. A shared part (S-0007 claim 30): is it drawn in the frame 0 of its motion file, or does the
   motion file play over the piece's life?

## Possible observation
Captured frames of a few pieces of one type with known spin draws (scenario V1's logged draws),
compared over consecutive ticks; a piece next to a building and over a cliff for the depth; a
piece of a house with a distinctive colour, and one thrown by a detonation, for the palette.

## Current workaround
Our engine draws the own model (no motion file) at the screen point of the piece's position with
its height lift, in the air layer at the end of its list (S-0102 claims 3, 4 and 7). The pose is the
fixed view of S-0105 claim 1, then a turn about the model axis (x, y or z) of the spin axis's
largest component (the first of equal ones; no turn for a zero axis), by the spin rate times the
ticks since creation with that component's sign, in whole steps of π/16 modulo 32. The light is
S-0306 F16's diffuse term. The palette is the 53-shade house-colour palette of the piece's house
(the neutral colour without one) at brightness 1000, not cell-lit. No depth test, no shadow, drawn
whatever the shroud. A shared part is not built yet: such a type draws nothing.

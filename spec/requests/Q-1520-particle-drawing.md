---
id: Q-1520
title: How a particle is drawn (palette, brightness, frame, translucency stages, placement)
status: open
related: [S-0613, S-0616, S-0007, S-0102, S-0103, S-0104]
answered-by: []
---

## Need
The game client draws every particle of a gas, smoke, fire, spark or railgun system. The specs
give part of the picture: the sprite name (S-0007 claim 28), the layer (air, S-0102 claim 3),
the drawn frame of a gas particle (its stage, without the start frame, S-0613 claim 15) and
the translucency of S-0613 claim 16. They do not give the rest, so the client
(`crates/app/src/mapview/live.rs`, `draw_effects`, and the particle drawing in
`crates/app/src/mapview/build.rs`) cannot draw a particle the way the game does. The lead's
review of 2026-10-19 found our placeholders (green, white and orange discs) wrong for gas
clouds, chaos smoke, the smoke of a damaged vehicle and the fire of a fire weapon.

## Questions
1. Which palette draws a particle's sprite: the anim palette, the unit palette, the theater
   palette, a house-colour palette of the system's owner house, or something the particle type
   names? Is it lit by the light of its cell (which level: scaled or unscaled ground level,
   S-0104 claim 13) or always drawn at neutral brightness?
2. For smoke and fire particles (and sparks or railgun particles, if they use a sprite): is
   the drawn image frame the particle stage plus the type's start frame, the stage alone, or
   something else? S-0613 claim 15 says the start frame is "not added for gas", which suggests
   it is added for other kinds; please confirm.
3. What do the type's translucent-25 and translucent-50 stage fields do: does a particle switch
   to 25 % (or 50 %) translucency once its stage reaches that field's value, and how do they
   combine with the type's own translucency and with S-0613 claim 16's detail rule?
4. S-0616 claims 8 and 9 give a smoke successor a translucency 25 points above its parent's
   (drawn only). Is that value kept per particle for its whole life and drawn as in S-0613
   claim 16 (25, 50, and 75 for 75 … 127), so that a successor of a successor is more
   translucent again?
5. Spark and railgun particles have a colour list, two start colours and a colour speed
   (S-0616 Import mapping), and S-0616 claim 2 mentions the railgun's laser and helix. Are
   these particles drawn as sprites at all, or as coloured pixels or lines in the colour of the
   list? How does the colour move through the list over a particle's life?
6. Where is the sprite placed relative to the particle's position: centred on the projected
   point with the height lift (S-0102 claim 19), or with an offset? Is a shadow drawn?
7. Within the air layer, in what order are particles drawn: by system and then the particle
   list order (oldest first), or sorted?

## Possible observation
A gas weapon kills infantry on flat, evenly lit ground; capture the clouds frame by frame at
the highest detail level and compare the pixels with the particle sprite's frames under the
anim, unit and theater palettes. Repeat on a cell lit by a coloured building light. Repeat with
a smoke type that sets a nonzero start frame and the 25/50 stage fields (named test rules).

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): every particle is drawn with
its sprite (S-0007 claim 28) in the air layer, centred on its projected position, with the anim
palette at the unscaled ground level of its cell (the same row as an unowned animation, S-0103
claim 8 row 5). The frame is the stage for gas (S-0613 claim 15) and the stage plus the start
frame for the other kinds. Translucency follows S-0613 claim 16 from the type's translucency
only; the stage fields and the successor's raised value are not drawn. A particle without a
sprite draws nothing.

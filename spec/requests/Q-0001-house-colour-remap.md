---
id: Q-0001
title: Which sprite and voxel colours take the owner's house colour, and how
status: answered
related: []
answered-by: [S-0103]
---

## Need
The asset viewer (and later the renderer) draws palette-indexed sprites and
voxels with a per-house palette row (PLAN.md 4.4). To build those rows the
engine must know which palette indices of an object's image show the owner's
colour and what colours replace them. PLAN.md 4.4 marks the remap and unlit
index ranges "to be confirmed by spec"; the lead indicated a spec numbered
S-0103 is planned for this. Code gap: `glasnost_render::remap` and the
viewer's remap control (`crates/app`).

## Questions
1. For sprites (SHP), which palette indices are drawn in the owner's colour?
   Is the set the same for every sprite, or does it depend on the palette,
   the object type or the theater?
2. For voxel models, which colours take the owner's colour? Is it the same
   set as for sprites, or does it depend on something in the voxel file?
3. How is the owner's colour turned into the replacement colours: how many
   shades, from which source (the colour chosen in the game setup, a rules
   entry, a palette), and in which order (darkest first or brightest first)?
4. Are there palette indices that are never lit or shaded (always drawn at
   full brightness)? Which ones, for sprites and for voxels?
5. Does the remap apply to shadows, to damaged frames, and to building
   animations drawn on top of the main image?

## Possible observation
One sprite with every palette index painted in separate stripes, built into
a test map for two players with contrasting colours: screenshots show which
stripes change with the owner and to what.

## Current workaround
The viewer's remap range is a user setting with no default (remap off until
the user sets a range); the code carries no range number. Remap rows are
built only from what the user enters.

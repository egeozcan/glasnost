---
id: Q-0951
title: Which art section gives an animation's playback fields when its image name differs from its type name
status: answered
related: [S-0007, S-0108, S-0109]
answered-by: [S-0007]
---

## Need

S-0007 claim 2 reads "all art settings of a type" from its *art section*, the section named by
the image name, and gives the *own art section* (named by the animation's type name) only the
image key (claims 1 and 3), the translucent-sprite name (claim 23) and the two flags of the claim
2 table. S-0109's Import mapping reads Start, End, LoopStart, LoopEnd, LoopCount, Rate and the
other playback keys "in the animation's art section" and does not say which of the two it means
when the image name differs from the type name.

Our importer reads them from the image-name section (S-0007 claim 2 read literally;
`mapview::slots::read_anim` is called with `TypeArt::image`).

In the installed art database the damaged variants of building slot animations are separate
animation types whose own section sets an image key naming the normal variant's section and
sets its own start field, while the image-name section (the normal variant's) sets none. Read
from the image-name section, a damaged variant starts at the normal variant's first image frame
and draws the normal pictures; the body (frame *k* + 1 of S-0108 claim 2, the damaged body)
and the slot animation then show two different pictures of the building at the same place.

Lead review of 2026-10-15 (`all-owners/map-DESERT-24.png`, `skirmish-view/map-DESERT-24.png`):
a damaged tesla-type coil showed two orbs side by side, and a damaged reactor cluster showed
the same doubling. Experiment (local lane only, not committed): reading the playback fields
from the own art section instead removed both doublings, the damaged animation then showing the
damaged pictures aligned with the damaged body.

## Questions

1. For an animation whose image name differs from its type name, which art section gives each
   playback key of S-0109's Import mapping (start, end, loop start, loop end, loop count, rate,
   random rate, reverse, ping-pong, shadow, tiled, flat, y draw offset, detail, translucency
   keys, next, damage, normalised): the image-name section, the own section, or the own section
   with the image-name section as fallback for an absent key?
2. Is the answer the same for each key, or does it differ by key (for example the drawing
   fields of S-0102 and S-0103 Import mapping: layer, alternative palette, cell drawer, normal
   light)?
3. Does it hold in every type read of the game (S-0007 claim 2's table depends on the previous
   read finding the sprite file), or only in some?

## Possible observation

A test animation type with its own art section (Image key naming a second section, Start set
to a value A) and a second section with Start set to a value B: record which image frame the
animation shows when created (any S-0109 playback scenario that records the first image frame), for a building slot animation
and for an effect animation.

## Current workaround

Playback and drawing fields are read from the image-name section (S-0007 claim 2). Interim rule
in `docs/development.md`, section *Lead visual review (M6c, row 4)*. The doubling of damaged
coils and reactors stays in the viewer until this is answered.

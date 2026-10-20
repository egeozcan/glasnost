---
id: Q-1528
title: Protection appearance of a building's bib and door images and of a unit drawn translucent or cloaked
status: answered
related: [S-0102, S-0513, S-0108, S-0109]
answered-by: [S-0513]
---

## Need
S-0102 claim 34's table covers the ordinary opaque drawings of protected objects: the bodies, a
vehicle's voxel turret and barrel, a building's voxel turret and barrel and its slot animations.
Its Open questions leave the others explicitly open (translucent animations and cloaked or
translucent units, building bibs and door accessories, remembered images, unusual sprite-turret
accessory combinations) and say not to infer them from the table. The game client draws a
protected building's bib, under-door and door images (S-0108 claims 7, 22 and 23) and can draw a
protected unit while it cloaks (S-0103, the cloak drawing); it has to decide whether these take
the brightness envelope of S-0513 F9, a colour mask, both or neither
(`crates/app/src/mapview/build.rs`, `building_items`; `crates/render/src/compose.rs`, `Paint`).

## Questions
1. Does a protected building's bib take the protection brightness (S-0513 F9) and, under the
   force shield, the force-shield mask of S-0102 F9?
2. The same for its under-door image and its door image while it unloads.
3. A protected unit drawn translucent or in its cloak drawing: does it take the brightness
   envelope, the mask (and if so is the mask joined before or after the blend of S-0102 F8), or
   neither?
4. A building drawn from its remembered image under the shroud while protected: is the
   appearance shown?

## Possible observation
S-0102 D44's set-up with a building that has a bib and an unloading door (a factory releasing a
unit) under each protection, and a cloaking unit under the curtain; compare the bib, door and
cloaked pixels with the same objects unprotected at the same phase factor.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): a protected building's bib,
under-door and door images keep their ordinary appearance (no envelope brightness, no mask). A
protected unit drawn translucent while it cloaks keeps the body's envelope brightness in its
blended source pixels but takes no mask (`Paint::Translucent` ignores the mask on both the CPU
and the GPU path). Remembered images are not drawn by the client yet.

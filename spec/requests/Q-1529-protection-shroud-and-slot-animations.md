---
id: Q-1529
title: Whether the shrouded-centre-cell protection row reaches a building's slot animations
status: open
related: [S-0513, S-0102, S-0108]
answered-by: []
---

## Need
S-0513 claim 29b says that while the centre cell of a protected object is shrouded for the local
player, every part of the object that is still drawn keeps the protection brightness but takes no
colour mask. S-0102 claim 17 makes a building's slot animations animation objects of their own,
not attached to the building, and S-0102 claim 34's animation rule gives such an animation the
force-shield mask of the building occupying the animation's cell while that building is
force-shielded. Neither says whether the shroud row also removes that mask from the slot
animations of a force-shielded building whose centre cell is shrouded, so the body, bib and door
images lose their colour and the slot animations on the same building may or may not keep it
(`crates/app/src/mapview/build.rs`, `slot_animations`; `crates/app/src/mapview/live.rs`,
`protected_cells`).

## Questions
1. A force-shielded building whose centre cell is shrouded for the local player, with an opaque
   slot animation drawn over one of its visible foundation cells: does that slot animation show the
   force-shield colour, or only the brightness?
2. Does the answer depend on whether the slot animation's own cell is shrouded, or only on the
   building's centre cell?

## Possible observation
S-0513 SW10 part i's shroud-edge set-up with a factory whose idle or active slot animation sits on
a corner of the foundation that stays visible while the centre cell is shrouded; capture that
animation's pixels under the force shield and compare them with the body pixels beside it and with
an unprotected control at the same phase factor.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): the shroud row is read as reaching
the slot animations through the building that supplies their mask. While the occupying building's
centre cell is shrouded for the local player it supplies no mask; the slot animation keeps its
own building's brightness.

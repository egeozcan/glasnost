---
id: Q-1526
title: How the berserk tint is combined with a unit's pixels
status: open
related: [S-0610, S-0102, S-0104, S-0006]
answered-by: []
---

## Need
S-0610 claim 7 says a berserk vehicle or infantry "is drawn with an extra additive tint", the
entry of the colour-adjustment table (S-0006 claim 2) at the rules' berserk-colour position, and
that "it adds to any other tint the object has". No rendering spec says how such a tint reaches
the screen. The only tint the rendering specs define is S-0102 claim 34's protection colour mask,
which is a bitwise union of packed pixels and which that claim says is "neither RGB addition with
saturation nor multiplication by a tint", so it cannot be assumed that "additive" means the same
operation. The client (`crates/app/src/mapview/build.rs`, `crates/render`) draws berserk units
untinted; the lead's review of 2026-10-19 asked why the units turned berserk show no colour.

## Questions
1. How is the berserk tint applied to an opaque pixel of the unit: as a colour mask joined to the
   packed pixel as in S-0102 F9, as a per-channel addition (with or without saturation, before or
   after shading), or otherwise?
2. How does the table entry's triple become the value that is applied (are the components scaled,
   and for which surface layout)?
3. "It adds to any other tint the object has": which other tints exist for a vehicle or infantry
   (protection, S-0102 claim 34; others?) and how are two tints combined: union of masks, sum,
   or the later one replacing the earlier?
4. Which parts take the tint: sprite body, voxel body, turret and barrel, the unit's own shadow,
   animations attached to it?
5. Is the tint applied in every frame while the unit is berserk, or does it pulse?

## Possible observation
Turn a test vehicle (voxel) and a test infantry unit berserk on flat ground, with the berserk
colour set by named test rules to table positions holding known triples (one pure red, one with
all channels high); capture at the highest detail and compare each pixel with the same unit not
berserk. Repeat on a curtain-protected test vehicle.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): no berserk tint is drawn; the
berserk state and its animations are unchanged.

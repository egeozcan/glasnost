---
id: Q-1885
title: Depth bias and drawing order of the voxel barrel on a sprite-body vehicle
status: open
related: [S-0102, S-0105, S-0007]
answered-by: []
---

## Need
S-0105 r35 claim 12 (*Sprite body*) now says that a vehicle drawn with a sprite body and the
turret flag draws its single voxel barrel model (S-0007 claim 19) at its turret position, turned
with its turret facing and pitched by its barrel elevation, and that claim 11's overlap rule by
4-step turret facing decides whether the barrel or the sprite turret wins where they overlap.

To draw it the renderer has to place the barrel image in the scene as an item with a depth bias
and a profile (S-0102 claim 13). S-0102 claim 13's table has a row for "vehicle drawn as a
sprite" (*U* − 2, upright) and a row for "vehicle or aircraft drawn as a voxel model" (*U*,
upright), but none for a voxel part of a vehicle drawn as a sprite. Which row applies decides
whether the barrel can be hidden by, or hide, other objects' pixels (a building, another
vehicle, a terrain object) that the sprite body itself is not hidden by. The barrel is not drawn
today (`crates/app/src/mapview/build.rs`, `sprite_vehicle`).

## Questions
1. Where another object's pixels overlap the voxel barrel of a sprite-body vehicle, does the
   barrel win or lose exactly as that vehicle's sprite body and sprite turret would at the same
   place? If not, does it behave as the body of a voxel vehicle at the same position would?
2. While the vehicle's height above the surface is not 0 (on a bridge deck, for instance), does
   the barrel follow the sprite body's depth or a voxel body's?
3. Does the barrel stand in the same render layer and the same place in the drawing order as
   the sprite turret (claim 22 of S-0102 draws the body and turret frames as one image), so
   that only claim 11's rule decides between barrel and sprite turret, and both cover the
   body?
4. Is the barrel of a sprite-body vehicle drawn in frame 0 of its HVA, as a barrel drawn with a
   voxel turret is (`quirks.barrel_frame_zero`), even though no voxel turret is drawn with it?

## Possible observation
S-0105 MS-33 part k's first sprite run, with a building and a second vehicle placed so that
their pixels overlap the barrel from the north, the south, the east and the west, at height 0
and on a bridge deck; and a barrel model whose HVA has several clearly different frames.

## Current workaround
The barrel of a sprite-body vehicle is not drawn: the sprite body and sprite turret are drawn as
before (S-0102 claim 22), and no voxel part is drawn on a sprite body. The selected turret index
of such a vehicle is derived as for any vehicle (S-0105 claim 43, *Any body*) and reaches nothing
drawn. The M8 intake 19 ledger lists this as a seam.

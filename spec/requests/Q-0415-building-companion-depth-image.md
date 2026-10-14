---
id: Q-0415
title: The building companion depth image: which file, and where it is placed
status: answered
related: [S-0102, S-0108]
answered-by: [S-0102]
---

## Need

S-0102 F5 says a building narrower than 8 cells draws its body with one
shared companion depth image, placed by a displacement built from two engine
constants. It does not name the file, and its Open questions leave open which
point of the body sprite the constants count from, so the sign of the
displacement is not known either. Without that image the body's depth is the
plain upright profile anchored at the sprite's bottom row (F2). A tall
building part, such as the tank of a water tower standing in front of a
barn, then gets a depth that is too far back, and the barn behind it is
drawn over it. The code is `crates/app/src/mapview/build.rs`
(`building_items`, `ItemDepth::Profile { companion: None }`, note
`NO_COMPANION`).

## Questions

1. Which game data file holds the shared companion depth image, and which
   image of that file is used?
2. From which point of the body sprite (canvas corner, canvas centre, draw
   point, or another point) is the displacement of F5 measured, for a live
   building and for the remembered image of a fogged building? In which
   direction does a positive displacement move the depth image?
3. Where the depth image does not cover a body pixel, which depth does that
   pixel get?

## Possible observation

A purpose-made map with a tall 1×1 or 2×2 building placed one cell in front
of a wide, low building, and a unit walking behind and in front of the tall
building's upper part. Record per frame which of them is drawn on top at the
pixels where they overlap.

## Current workaround

No companion depth image: the body uses the upright profile alone (S-0102
F2). Listed in docs/development.md (interim rendering rules).

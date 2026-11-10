---
id: Q-1621
title: What a marked building shows when the laser-target selector names no table position
status: answered
related: [S-0519, S-0102, S-0513, S-0610]
answered-by: [S-0519]
---

## Need
S-0519 claim 26 has a marked building take the colour mask of the colour-adjustment entry that
the rules' laser-target colour (`[AudioVisual] LaserTargetColor`) selects. The table has 16
positions (S-0006). S-0513 claim 29a (protection selectors) and S-0610 Edge cases (the berserk
selector) both say what happens when their selector is below 0 or above 15: no mask, and the
importer warns, as a departure. S-0102 F9 says "with no selected adjustment, K = 0" and names
those two specs, but no spec says the same for the laser-target selector, and S-0519's Import
mapping gives no out-of-range row. The engine must decide what a marked building's body, voxel
parts and slot animations take when the selector is out of range, and whether the importer
warns. The code is `mapview::protection::Masks::read` (the mask) and the importer's
`[AudioVisual]` selector check in `glasnost_convert::sim`.

## Questions
1. With `LaserTargetColor` below 0 or above 15, what colour change does a marked building show
   in the original: none, or some other change?
2. Should our engine treat such a selector like the protection and berserk selectors (no mask,
   the importer warns), or reduce or clamp it?

## Possible observation
S-0519 T27 with `LaserTargetColor=16` and then `-1` in a rules layer: capture the marked
building and compare its pixels with an unmarked capture taken at the same mark factor.

## Current workaround
Interim rule: a laser-target selector outside 0 to 15 gives no mask (S-0102 F9's "no selected
adjustment"), and the importer warns once, naming the key and its value, as it does for the
protection selectors. The mark's brightness pulse (S-0519 F7) is unaffected.

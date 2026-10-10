---
id: Q-0013
title: Which cell light value terrain tiles are drawn with
status: answered
related: [S-0104]
answered-by: [S-0104]
---

## Need
S-0104 Formulas ("Incremental update") and Open questions leave open
whether a terrain tile is drawn at its cell's scaled or unscaled ground
level. Under a non-white map tint the two differ by the normalisation
factor *K*. Code gap: `glasnost_render::light` (tile brightness).

## Questions
1. Which of the three light values of S-0104 Formulas ("Cell light")
   gives a terrain tile its brightness (claim 15 row)?
2. Do terrain animations on tiles follow the same value (S-0104 claim 13
   gives them the scaled ground level)?

## Possible observation
S-0104's Scenario L9 secondary observable: a map tint of (1.20, 1.00,
1.00) and plain ground; compare the tile pixels with both predictions.

## Current workaround
Tiles use the scaled ground level, the value S-0104 claim 13 gives every
other thing drawn with a tile shade table (terrain animations, plain
terrain objects, terrain-palette buildings). Marked provisional.

---
id: Q-3146
title: How a building's light source changes when a hit reports now yellow
status: open
related: [S-0302, S-0104, S-0405]
answered-by: []
---

## Need

S-0302 claim 12 says that on *now yellow* the intensity of a building's
light source is scaled by a fixed factor (scenario D11, part c), and its
Open questions say the factor's value is not stated yet. S-0104 does not
mention damage at all. The view's building light sources follow the game's
lit buildings (S-0104 claim 12, S-0405 claim 9; `mapview::building_lights`,
`Painter::relight` in `crates/app/src/mapview/build.rs`), but nothing in
the specs says what the scaled light is, so the renderer cannot draw a
damaged building's light.

## Questions

1. By what factor is the light source's intensity scaled on *now yellow*?
   Is it the same for every building type and for a negative intensity?
2. Is anything else of the light source changed with it (its visibility
   radius, its colour terms), or only the intensity?
3. How long does the change last? Does repair back to half the strength or
   above restore the intensity, and does a second *now yellow* after such a
   repair scale it again (so that it compounds), or is it scaled once only?
4. Is the result rounded or truncated, and where (the intensity itself, or
   each cell's light term)?
5. When is the change seen on the cells around the building: in the frame
   of the hit's tick, or at a later light update?
6. Does a capture, a power change or a switch-off and on (S-0405 claim 9)
   keep the scaled intensity or bring back the type's value?

## Possible observation

Scenario D11 part c already reads the intensity before and after the hit.
Extending it: a building type with a light source of a known intensity
and radius, on flat ground at a fixed ambient level. Read the light of a
cell on the building's light radius and of one next to it before the
hit, after a *now yellow* hit, after repair to full health, and after a
second *now yellow* hit. Then a run with a negative intensity. Then a run
where the building is captured or switched off and on between the hit and
the reading.

## Current workaround

The renderer never scales a building's light for its damage: a damaged
building lights its cells as an undamaged one does. This seam is named in
the doc comment of `Painter::relight`. No value is invented.

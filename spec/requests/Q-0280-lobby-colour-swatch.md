---
id: Q-0280
title: Which shade of a colour definition does the setup screen's colour swatch show?
status: answered
related: [S-0103, S-0412]
answered-by: [S-0103]
---

## Need

S-0103 claim 10 maps lobby colour slots 0 ... 7 to colour definitions, and claim 6 gives the
16-entry house-colour ramp of a definition. The setup screen shows a swatch for each slot, but no
spec says which colour of the definition (the definition itself, a ramp entry, or something else)
the swatch shows.

## Questions

1. Which RGB colour does the setup screen show for a colour slot: the definition's HSV converted
   with S-0103's formula, one entry of the ramp (which), or another colour?
2. Is the swatch a flat fill, or drawn from an image tinted by the colour?

## Possible observation

Screenshot the setup screen's colour choices and compare the swatch pixels with the definition
and each ramp entry.

## Current workaround

Interim rule: a flat swatch of the definition's own HSV through S-0103's integer HSV-to-RGB
formula (`glasnost_render::palette::hsv_to_rgb`). Recorded in docs/development.md, "Game client
(M5b)".

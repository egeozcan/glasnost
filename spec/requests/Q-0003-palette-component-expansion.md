---
id: Q-0003
title: How 6-bit palette components become displayed colours
status: open
related: []
answered-by: []
---

## Need
Palette files store each colour component in 6 bits (0 to 63). To draw
colours that match the game, the renderer must expand them to 8-bit display
values the way the game does. Code gap: `glasnost_fmt::pal`
(`Palette6::to_palette`) and every place a palette is uploaded.

## Questions
1. Which 8-bit value does the game display for each 6-bit component value?
   In particular: does the brightest value (63) show as full intensity, and
   are the low bits of the result zero or filled?
2. Does the answer differ between palettes used for sprites, terrain tiles,
   voxels and the user interface, or between colour depths of the display mode?
3. Are any palette entries treated as transparent or replaced at load time
   (for example the first entry, or shadow entries)?

## Possible observation
A test palette with a ramp of all 64 component values, applied to a strip of
64 single-colour cells; capture the screen in a mode with 32-bit colour and
read the pixel values.

## Current workaround
`Palette6::to_palette` takes an explicit `Expansion` argument; the viewer
offers both shifting left by two bits and scaling to the full range, marked
provisional. Nothing in the simulation depends on it.

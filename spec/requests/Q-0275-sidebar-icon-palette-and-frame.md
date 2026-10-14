---
id: Q-0275
title: Which palette and which frame draw a sidebar icon?
status: answered
related: [S-0007, S-0411, S-0103]
answered-by: [S-0411]
---

## Need

S-0007 claims 22 and 25 name the icon file a buildable type shows in the sidebar, and S-0411
says when and in which state it shows. Neither says which palette the icon's indices go through,
which frame of the file is drawn, or where in the icon slot the frame sits. The game client's
sidebar (`app::game`) has to draw the icon.

## Questions

1. Which palette file (or palette built at game start) do sidebar icons use? Does it depend on
   the theater, the player's side or the house colour?
2. Which frame of a multi-frame icon file is drawn, and is there any other frame shown for a
   state (darkened, not ready)?
3. How is the darkened look of S-0411 claim 3 produced: a darker palette, a shading table, or a
   separate image?
4. Where does the frame sit in its slot when the file's canvas is larger or smaller than the slot?

## Possible observation

A loose icon file with a frame of every index 1 ... 255 in stripes, shown in the sidebar in each
theater and for each side, with screenshots compared against the candidate palettes.

## Current workaround

Interim rule: frame 0 of the icon file through the palette file `PALETTE.PAL` (the file S-0111's
Import mapping names for the overlay sprites), index 0 transparent, the file's whole canvas centred
in the slot and clipped to it. A darkened icon gets a translucent black veil of our own. Recorded
in docs/development.md, "Game client (M5b)".

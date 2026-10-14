---
id: Q-0276
title: Where are the sidebar's parts, and how do the clock, labels and power bar look?
status: open
related: [S-0411, S-0405, S-0410, S-0416]
answered-by: []
---

## Need

S-0411 says what the sidebar shows (tabs, icons, the clock stage, the ready and on-hold labels,
queue counts), S-0405 gives power output and drain, S-0410 the radar image size and S-0416 the
credit counter. None of them gives the screen positions and sizes of these parts, the look of the
clock overlay at a stage, the label text, or how the power output and drain are drawn. The game
client draws all of them.

## Questions

1. At a given screen size, where are the radar, the credit counter, the power display, the tab
   buttons, the icon grid (columns, rows, icon size, scroll buttons) and the sell, repair and
   power-mode buttons?
2. How is clock stage *s* (1 ... 55) drawn over an icon?
3. What text, font and position do the ready and on-hold labels and the queue count use?
4. How are power output and drain shown (a bar, its scale, colours for low power)?
5. Does the layout change with the player's side?

## Possible observation

Screenshots of the sidebar at each supported resolution, with a building at known progress
stages, a held item and a queue of several units.

## Current workaround

Interim rule: our own layout (`glasnost_ui::hud::Layout`): a 168-pixel bar on the right with the
radar at the top, the credit text, a power bar filled to drain/output (red when the drain exceeds
the output), three mode buttons, four tab buttons, a two-column grid of 68 × 52 icon slots with
scroll buttons, and an options button. The clock is a translucent veil covering the part of the
icon not yet built (stage/55 of its height, from the top); "READY" and "ON HOLD" labels and the
queue count are drawn in our own bitmap font. Recorded in docs/development.md, "Game client
(M5b)".

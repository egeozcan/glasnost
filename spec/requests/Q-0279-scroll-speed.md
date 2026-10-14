---
id: Q-0279
title: How fast does the view scroll at the screen edge and with the keyboard?
status: answered
related: [S-0101, S-0201]
answered-by: [S-0101]
---

## Need

S-0101 claim 8 limits the view centre wherever it is set, scrolling included, but no spec gives
how far the view moves per frame or per tick when the pointer is at a screen edge or a scroll key
is held, how close to the edge the pointer must be, or whether the speed setting or the game
speed changes it.

## Questions

1. How close to each screen edge (in pixels) must the pointer be to scroll, and do corners
   scroll diagonally?
2. How many pixels does the view move per drawn frame (or per tick, or per unit of time) at each
   scroll-speed setting, and does it accelerate while held?
3. Do the arrow keys scroll at the same speed? Does scrolling go on while the game is paused?

## Possible observation

Hold the pointer at an edge for a counted number of frames at each scroll-speed setting and
record the view position.

## Current workaround

Interim rule: the view scrolls by 16 pixels per drawn frame when the pointer is within 4 pixels of
a window edge (corners scroll diagonally) or an arrow key is held, also while paused; then
S-0101 claim 8's limits apply. Recorded in docs/development.md, "Game client (M5b)".

---
id: Q-2266
title: Whether palette index 0 of the sidebar button images is transparent, and whether their colours go through the 16-bit step
status: open
related: [S-0411, S-0103]
answered-by: []
---

## Need
S-0411 revision 28 claim 19 and its Import mapping give the sell, repair,
tab and scroll button images, their frames per state, and the sidebar
palette with its conversion (a file byte *c* gives the component
(4·*c*) mod 256). Two things about how a pixel of these images shows are
not stated:

- For the sidebar icons, S-0103 claim 13 says index 0 is transparent and the
  converted colour is packed by claim 9's 16-bit step. S-0411 claim 19 says
  neither for the button images.
- Our client draws each image into its own cell over our sidebar's panel
  (claim 19, *Layout*), so whether index 0 lets the panel show through
  changes what the player sees around each button.

The code is `game::sidebar_art` (`palette_rgba`) and
`Session::register_art` in `crates/app`, and `hud::view` in `crates/ui`.

## Questions
1. Is a pixel of palette index 0 in these images (the sell, repair, four tab
   and two scroll images of S-0411 claim 19) transparent, so that what lies
   beneath shows through, or is it drawn in palette entry 0's colour?
2. Is any other index of these images transparent, or drawn as a shadow?
3. Is a converted sidebar-palette colour shown at full 8-bit precision, or
   packed through S-0103 claim 9's 16-bit step as the icons are?

## Possible observation
S-0411 scenario S15 part b: compare a captured button's pixels with part a's
renders of the same frame. Index-0 pixels show whatever lies beneath the
button in the original's sidebar (or entry 0's colour). Compare the top bits
of other pixels with the converted entry.

## Current workaround
Index 0 is transparent and every other index is opaque, as for every sprite
our painter draws, the icons included (S-0103 claim 13). Colours are drawn at
the converted 8-bit value, with no 16-bit step. Both are noted as interim in
`docs/m8-claims/intake69-S-0411.md`.

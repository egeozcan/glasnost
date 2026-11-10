---
id: Q-2238
title: Which install images draw the sidebar's command buttons and tab buttons, and in which states
status: answered
related: [S-0411, S-0410]
answered-by: [S-0411]
---

## Need
S-0411 (Summary, *Interface looks*, lead decision of 2026-10-14) leaves the
sidebar's positions, sizes and the lit tab's artwork to the implementer
(Q-0276, Q-0283). Its Import mapping names only the icon files
(`Cameo`, `AltCameo`, the missing-icon file) and the unavailable-icon mask. No
spec names the images of the sell, repair and power-mode buttons or of the four
tab buttons, or which frames show their states.

M8 round 7 note 41 ("these buttons should use graphic symbols") asks for
pictures in place of our text labels. We now draw our own symbols (below). To
draw the original's pictures from the player's own install instead, the client
needs to know where they are.

## Questions
1. Which files of the install, and which frames of them, show the sell, repair
   and power-mode buttons, and which show the four sidebar tabs?
2. Which frame shows each state: ready, pressed or in its mode, lit (S-0411
   claim 11's flash), and unavailable (a tab with no icons)?
3. Which palette draws them, and does it depend on the player's side?
4. Are these images placed at fixed positions relative to each other, so that
   a client that draws them must also take the original's spacing, or can they
   be drawn into our own layout's button cells?

## Possible observation
None needed in play: the answer is a file-format and data mapping, which the
spec side can state as an Import mapping (as S-0411 does for the icon files).

## Current workaround
Our design, marked `// design:` in `crates/ui/src/hud.rs` (`symbol`): each
command and tab button shows a symbol of our own drawing (a coin with a dollar
sign for sell, a wrench for repair, a lightning bolt for power, a factory for
buildings, a turret for defences, a soldier for infantry, a tank for vehicles),
with the button's name in a tooltip on hover. The buttons keep their places,
hit areas, keyboard focus and states.

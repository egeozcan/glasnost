---
id: Q-0339
title: When the sidebar strip counts as scrolled past its end
status: answered
related: [S-0701, S-0411]
answered-by: [S-0411]
---

## Need

S-0701 claim 24 (revision 2) says that scrolling the sidebar strip past its end plays the
game-wide refusal sound. It does not say what "past its end" means. The client's
`Play::scroll_icons` in `crates/app/src/game/play.rs` clamps every scroll request to the strip.
It has to decide which requests raise the cue.

## Questions

1. Does the sound play only when a scroll request cannot move the strip at all, because it is
   already at that end?
2. Does it also play when a request moves the strip part of the way and is then clamped, for
   example a step of several rows with only one row left?
3. Does a strip with no more icons than fit, which cannot scroll either way, play the sound on
   every scroll click?
4. Does the sound depend on how the scroll is asked for: a scroll button click, holding the button,
   or a mouse wheel?

## Possible observation

Fill one sidebar tab with enough icons for two extra rows. Click the scroll-up button with the
strip at the top, then scroll down one row at a time past the bottom. Record which clicks give the
refusal sound. Repeat with a tab whose icons all fit.

## Current workaround

A scroll click plays the refusal sound only when it cannot move the strip at all. That includes
every click on a strip whose icons all fit. A request that moves the strip part of the way is
silent. Documented in docs/development.md, "Audio (M5b)".

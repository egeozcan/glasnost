---
id: Q-3131
title: A press, band box or scroll drag in progress when the pointer leaves the window
status: open
related: [S-0214, S-0111, S-0101]
answered-by: []
---

## Need

A windowed client (a browser tab, a desktop window) can lose the pointer
while a mouse button is held: the pointer leaves the window, and the
button's release may arrive outside it or not at all. The game client
must decide what becomes of the input in progress. S-0214 claim 18
defines the band box from its press to its release, and S-0111 claim 20
says the pointer leaving the window is not an event of its own for the
tips, but no approved spec says what a held press, a band box being
dragged, a right-button scroll drag or a held right press does when the
pointer leaves the battlefield window before its release. The gap is in
`crates/app/src/game/session.rs`, `Session::pointer_left` (issue #150).

## Questions

1. A left press in the battlefield, the pointer dragged more than the
   band-box threshold (S-0214 claim 18), then moved out of the game's
   window and the button released outside it: is a band box applied, and
   with which rectangle (the last position inside, the edge, or none)?
2. The same with the pointer brought back into the window before the
   release: does the band box continue from its original press pixel?
3. A left press without movement (an ordinary click pending), the
   pointer leaving the window before the release: is the click applied
   on the release outside, applied on return, or dropped?
4. A right-button press or right-drag scroll in progress when the
   pointer leaves the window: does scrolling continue, stop, or resume
   on return; is the right click's order (or deselection) applied on the
   release?

## Possible observation

In a windowed display mode, for each case: start the press in the
battlefield, move the pointer across the window edge, release outside
(and, separately, return before releasing), and record the selection,
the orders given and the view position afterwards.

## Current workaround

`pointer_left` drops the pending press, the band box, the right press
and the held view button: nothing is applied on a later release. This is
the code's state before the request, kept unchanged.

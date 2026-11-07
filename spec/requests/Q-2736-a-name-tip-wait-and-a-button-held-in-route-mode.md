---
id: Q-2736
title: A left or right button pressed in the route mode and held while the pointer rests
status: answered
related: [S-0111, S-0213]
answered-by: [S-0111]
---

## Need

S-0111 r40 claim 20 (Blocked) says a left or right mouse button that was pressed over the
battlefield view and is held blocks the name tip, and its Open questions leave one case undecided:
whether a left or right button pressed in the route mode (S-0213 claim 16) and still held blocks the
tip after the pointer moves and rests while it is held. `Session::pointer_down` in
`crates/app/src/game/session.rs` sets `view_button_held` for every such press and
`Session::update_name_tip` blocks on it. The line is marked `design: Q-2736`.

## Questions

1. In the route mode, with the left or the right button pressed over the battlefield view and held,
   does the pointer resting for 1000 ms after a movement (the press itself hides the tip and stops
   the wait) show the name tip, or is it blocked as a held button is outside the route mode?
2. Does the answer differ for a press that the route mode refuses (S-0213 claim 16d, a refused
   press ends the click) from one that it takes as a point or a band box?

## Possible observation

S-0111 scenario NT1 (g) with the route-mode key held: press the left button over open ground beside
the vehicle, move the pointer onto the vehicle by less than the drag threshold with the button held,
and rest for 3 s; repeat with the right button and with a refused press.

## Current workaround

A held left or right button pressed over the view blocks the tip in the route mode as it does
outside it, for every press, the refused ones included. Nothing else is assumed.

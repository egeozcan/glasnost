---
id: Q-2193
title: Which points count as inside a click area for the route-mode kind test, and whether right presses meet it
status: open
related: [S-0213]
answered-by: []
---

## Need
S-0213 r74 claim 16d, *When the tests are made*: "The kind test is made
when the button is pressed, for every route-mode press on the battlefield
with the cursor outside every click area (claim 16b)". Two readings are
open:

- Claim 16b, *Click areas*, uses two sets of points. The candidate is
  picked from the points drawn for the player. A press that takes nothing
  is tested against the points of any house, drawn or not. Claim 16d does
  not say which set "every click area" means.
- "The button" does not say which button. Claim 16h gives right clicks in
  route mode a rule of their own: one whose release is under 4 pixels from
  its press, made with nothing selected, clears the point selection.

## Questions
1. An aircraft or building is selected. A route-mode press lands only in
   the click area of a point that is not drawn for the player (a point of
   another house that is not allied). Is the press tested for kind?
2. Does `quirks.hidden_point_refuses_click` (off: a point the player
   cannot see never refuses a press) change the answer to question 1?
3. An aircraft or building is selected. Is a right-button press in route
   mode, on the battlefield and outside every click area, refused for
   kind? If so, does it give the message and sound, keep the selection,
   and stop the scroll drag?

## Possible observation
For question 1: with an enemy route point at a known place, select an own
building, hold the route-mode key and press inside that point's click
area. Note whether the building-refused message plays. For question 3: with
an own building selected, hold the route-mode key and right-click on bare
ground. Note whether the message plays and whether a right-drag still
scrolls.

## Current workaround
Our design, marked `// design (Q-2193)`:
- In `routes::press_kind_refusal`
  (`crates/deterministic/sim/src/routes.rs`), "inside a click area" uses
  the same points as the press's own test. Those are any house's points
  under `quirks.hidden_point_refuses_click` on, and only the drawn ones
  under off.
- In `Session::pointer_down` (`crates/app/src/game/session.rs`), only a
  left press is a route-mode press, for the kind test and for the point
  tests.

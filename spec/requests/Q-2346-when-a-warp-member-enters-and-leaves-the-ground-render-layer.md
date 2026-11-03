---
id: Q-2346
title: When a superweapon warp member enters the ground render layer at the launch, and when it leaves it after the arrival hold
status: answered
related: [S-0102, S-0306, S-0513]
answered-by: [S-0102]
---

## Need
S-0102 revision 34 claim 3 adds a first row to the layer table: a superweapon warp member, any
kind, is in the ground layer from its warp's launch until at least the end of its arrival hold,
whatever its height (S-0306 claim 17). Claim 4 says an object's layer is evaluated only at its
listed moments and that between them it stays in the list it was last put into. Neither the
launch nor the end of the arrival hold is one of claim 4's moments. Two things the drawing must
decide are therefore not stated:

- A member that is in a list other than ground when its warp launches, such as a unit with
  jumpjet movement at its cruise height (S-0513 claim 34 keeps aircraft in the air out of a
  warp), next reaches claim 4 at a jumpjet check. That check evaluates the layer before and after
  the step, and the new row gives ground on both sides, so the check never re-files it. Read as
  an evaluation row alone, the member would stay in the top list until its departure takes it
  off the map. Read as the state claim 3 states ("is in the ground layer from its warp's
  launch"), it is in the ground layer from the launch on.
- After the arrival hold, S-0102 claim 3 and S-0306 claim 17 say when the member leaves the
  ground layer is not established.

The code is `mapview::layers` in `crates/app` (`evaluate_at`, `Layers::evaluate_mobiles`). The
member state the simulation keeps from the launch until the hold ends is read through
`superweapons::warp::in_ground_layer`.

## Questions
1. When a warp launches while a member is drawn in a layer other than ground (for example a
   unit with jumpjet movement hovering at its cruise height), is the member drawn in the ground
   layer from the launch's tick on, and is it then placed in the ground layer as claim 6 places
   any ground object? Or does it keep its place in its upper list until something else moves it?
2. After a member's arrival hold ends, in which tick does it leave the ground layer, if it is
   at a height that the ordinary rows give another layer? Is that at the next of claim 4's
   moments for its kind (for a jumpjet unit, the first movement step that changes its
   evaluated layer), at the end of the hold itself, or later?

## Possible observation
A Yuri's Revenge skirmish on flat open ground with a unit with jumpjet movement hovering at its
cruise height near another unit with jumpjet movement at the same height that is not warped,
both overlapping on screen. Launch a chronosphere warp on the first. Capture each tick from the
launch to the departure: whether the warped unit is drawn under or over the other (top layer
order) or by ground depth. Repeat at the arrival, from the arrival hold's last tick until the
member next moves.

## Current workaround
Interim in `mapview::layers`: the new row comes first in `evaluate_at`, so any evaluation during
the warp gives ground. In addition, after each tick a member that is still in a list other than
ground once its warp has launched is put into the ground layer (question 1 read as the state
claim 3 states). After the hold the ordinary rows apply again, at the next of claim 4's moments:
with `quirks.layer_change_on_events_only` on the member stays in the ground layer until such a
moment re-files it; off, the sweep moves it in the tick the hold ends. Noted in
`docs/m8-claims/intake78.md`.

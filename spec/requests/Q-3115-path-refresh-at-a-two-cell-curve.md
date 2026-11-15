---
id: Q-3115
title: How many steps of the new path the refresh at the last stored step drops when that step is a two-cell curve
status: answered
related: [S-0212, S-0207]
answered-by: [S-0212]
---

## Need
S-0212 claim 20 says that a vehicle with the driving or ship locomotor recomputes its path when it
begins the last step of its stored path while its heading point is more than 512 leptons away. With
`quirks.path_refresh_keeps_old_step` on, "the unit still takes that old step, and the new path's
first step is dropped: the unit continues with the new path's second step from the cell the old
step reached. When the new path's first step is the old step, as is usual, this has no visible
effect."

S-0207 claim 7 lets a vehicle begin a two-cell curve that covers its next two stored steps. When those
are the last two steps of the stored path, the track in progress covers two steps, not one. Claim 20
does not say what the "old step" is then, or how many steps of the new path are dropped.

Our engine drops one step. On synthetic data (a one-cell-wide corridor opening onto flat ground; the
last two stored steps east then south-east, begun as one curve from (7, 0) to (9, 1)), the new path
from (7, 0) begins with the same two steps. With one step dropped, the vehicle finishes the curve and
then takes the curve's second step again: it drives one cell past its destination diagonally and comes
back. The claim's "no visible effect" sentence suggests two steps should go. (Issue #263, case 2.)

## Questions
1. When the recomputation of claim 20 is made as a two-cell curve begins that uses up the stored path,
   how many steps of the new path are dropped: one, two, or the steps up to the cell the curve ends in?
2. With `quirks.path_refresh_keeps_old_step` off ("the unit takes the new path's first step instead of
   the old one"), what replaces a two-cell curve: the new path's first step alone, or a new choice of
   track by claim 7 from the new path's first two steps?
3. May a curve begin at all when its two steps are the last two stored steps and the refresh is due,
   or does the vehicle then begin the straight or diagonal track of the first step only?

## Possible observation
S-0212 scenario P5 run 4 with the route arranged so that the 23rd and 24th stored steps form a curve
(for example a corridor that turns once near the 24th cell). Record each cell the vehicle enters after
the 22nd step: whether it overshoots the turn by one cell answers question 1.

## Current workaround
One step of the new path is dropped whatever the track in progress covers
(`refresh_from_target` in `crates/deterministic/sim/src/drive.rs`), as before issue #263. With the
switch off, the refresh is made before the track begins and the step check starts again from the new
path's first step, which then chooses its track by claim 7.

---
id: Q-2160
title: How do the V3 launchers beat the Magnetrons in TELE against V3 when the Magnetrons start on side A?
status: answered
related: [S-0607, S-0215, S-0309, S-0506]
answered-by: [S-0607]
---

## Need
M-0016 replays M-0001's TELE against V3 games on stock data. The V3 group wins all 12 games, 6 on
each side. With M-0016's rows the pairing is scored (M8 step A3 intake 52b). Ours fails its side
outcome on side A (the Magnetrons on side A, the V3 launchers on side B): over seeds 0 to 63 the
V3 group wins 8 games there and 54 end with no winner (`glasnost-headless arena --record M-0001
--seed-range TELE,V3,A,0-63`). With the Magnetrons on side B, the V3 group wins every game of
seeds 0 to 5, as in the stock games.

A trace of ours (`--trace-summary TELE,V3,A,1`) shows the Magnetrons holding the V3 launchers over
and over. In that game the Magnetrons start 131 holds. Most V3 launchers are seized and restored
again more than ten times each, about every 200 to 300 ticks, and the V3 group fires 27 shots in
6080 frames while the Magnetrons fire 618. At the end the V3 launchers are at full health, in
*hunt*, several of them on the lower ground west of side A.

This is the side and the terrain of Q-2113 (a Magnetron whose gripped unit comes inside its
minimum range backs away to the lower ground beside the plateau) and the restoring of Q-2035, but
here the holds keep starting again, while against BFRT the fortress stays seized. No approved claim
we found tells what lets the stock V3 launchers fire between holds, or what ends the stock holds,
for a launcher.

The code concerned is the lifter of `lift.rs` (S-0607) and the V3 launcher's firing
(S-0506's spawned missile).

## Questions
1. In the stock games with the Magnetrons on side A, are the V3 launchers seized at all, and how
   often is one held again after it is restored?
2. Does a V3 launcher fire while it is seized, held or being set down, or in the ticks right after
   it is restored, and does its rocket keep flying when its launcher is seized?
3. Does a Magnetron start a hold on a launcher that is inside its weapon's minimum range, and does
   it then take the approach step (Q-2113 question 1, for a launcher)?

## Possible observation
M-0016's replay of TELE against V3, side A, seed 1, recording for each V3 launcher every tick it is
seized, held, set down and restored, and every fire event with its tick and whether it made a
rocket; and for each Magnetron, its hold starts and its distance to the held unit.

## Current workaround
The approved rules, unchanged. The pairing stays failing on its side-A outcome.

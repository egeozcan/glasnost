---
id: Q-2216
title: How do the stock HTK destroy every disabled robot tank in ROBO against HTK when the robot tanks start on side B?
status: answered
related: [S-0306, S-0503, S-0405]
answered-by: [S-0306]
---

## Need
M-0003's ROBO against HTK games end with the HTK group winning all 12, 6 on each side. The last
sample falls between frames 3040 and 5776. Every robot tank is disabled, as M-0005 found for ROBO
against LTNK.

Since S-0306 r53 is built (M8 step A3 intake 61), our shots at a switched-off hovering vehicle are
led while S-0503 counts it as moving (claim 3 b, `quirks.switched_off_hover_lead` on). A straight
or arcing projectile that still has a firer is also no longer detonated below the ground or in a
building band (T1's first row, `quirks.owned_shell_ground_pass` on). This answered M-0005's stall
in LTNK against ROBO and ROBO against APOC, and both pairings now pass. ROBO against HTK now fails
on side B instead.

Over seeds 0 to 63 (`glasnost-headless arena --record M-0003 --seed-range ROBO,HTK,B,0-63`), with
the robot tanks on side B and the HTK on side A:

- Our HTK win 23 games. The other 41 end with no winner at the frame limit.
- Before intake 61 the HTK won all 64, every game by frame 1520.

With the robot tanks on side A, ours win 59 of 64, which is close to the stock.

A trace of ours (`--trace-summary ROBO,HTK,B,2`) shows what happens on side B:

- Each of the 18 HTK fires about 285 shots at the robot tanks.
- The 8 robot tanks of the column nearer the HTK are destroyed between ticks 2241 and 5844.
- The 7 robot tanks of the column behind it are still in play at frame 6080.

No approved claim we found tells which of the rules of claims 3 b and 6 a and T1 apply to the
HTK's shots in the stock games, or what lets them reach the robot tanks of the rear column. The
code concerned is `combat::projectile::led` and the first row of `combat::projectile::turn`
(S-0306).

## Questions
1. In the stock games of ROBO against HTK with the robot tanks on side B, are the HTK's shots at a
   disabled robot tank that keeps a heading point led, and where do they detonate: at the robot
   tank, short of it, or beyond it?
2. Does a shot from an HTK that passes below the ground fly on, as T1's first row states for a
   projectile with a firer, or is it detonated there?
3. Are the robot tanks of the rear column hit by shots aimed at them, or by the blasts of shots
   aimed at the front column?

## Possible observation
A replay of M-0003's ROBO against HTK, side B, seed 2. Record:

- For each HTK shot: its tick, its target, its aim point, and its detonation point and height.
- For each robot tank: its heading point, its speed fraction, its health, and the tick it leaves
  play.

## Current workaround
The approved rules, unchanged. The pairing fails its side-B outcome.

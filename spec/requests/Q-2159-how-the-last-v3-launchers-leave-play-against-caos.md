---
id: Q-2159
title: How do the last V3 launchers leave play in CAOS against V3?
status: answered
related: [S-0610, S-0302, S-0309, S-0506]
answered-by: [S-0506]
---

## Need
M-0016 replays M-0001's CAOS against V3 games on stock data. The CAOS group wins 7 of 12 (2 of 6
on side A, 5 of 6 on side B), and the other 5 end with no winner. With M-0016's rows the pairing
is scored (M8 step A3 intake 52b), and ours fails its win test: 1 row win in 12, the other 11
with no winner (`glasnost-headless arena --record M-0001`).

In a watched game of ours (`--watch CAOS,V3,B,1`) the V3 group shrinks to two launchers by frame
3648 (by the side's value), and those two stay in play to the frame limit at 6080. At the end both
are in *hunt*, one with a CAOS unit and one with the other launcher as its target, and one of them
is damaged. The CAOS units stand next to them and keep firing (about a thousand shots in the
game). A launcher's rocket is a spawned missile (S-0506).

No approved claim we found tells how the stock CAOS group finishes the last launchers in the games
it wins: whether the last launchers destroy each other, destroy themselves with their own rockets
at close range, are crushed, or are worn down by the CAOS weapon itself.

## Questions
1. In the stock games that CAOS wins, how does each of the last V3 launchers leave play: destroyed
   by another launcher's rocket, by its own rocket or its blast, crushed by a CAOS unit (S-0210),
   or in another way?
2. Does a berserk V3 launcher fire at a target closer than its weapon's minimum range, or at a
   target next to itself, and does its rocket then damage it?
3. Does a CAOS unit take a berserk launcher, or the cell under it, as a crush target?

## Possible observation
M-0016's replay of one CAOS against V3 game that CAOS wins (side B, seed 1), recording for every
V3 launcher the tick it leaves play with the attacker and warhead of the killing damage event, its
berserk state, its target, and the position of each of its rockets at launch and at detonation.

## Current workaround
The approved rules, unchanged. The pairing stays failing on its win test.

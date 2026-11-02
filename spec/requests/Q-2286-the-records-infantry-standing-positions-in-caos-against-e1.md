---
id: Q-2286
title: Where did M-0001's infantry stand in their cells at tick 0, given that CAOS against E1 turns on it in our arena?
status: open
related: [S-0216, S-0208, S-0610, M-0001, M-0006]
answered-by: []
---

## Need

This extends Q-2277 (M-0002's YURI against GGI and SHK) to a third pairing, in M-0001.
S-0216 revision 37 claim 8 places each arena infantry unit through a play-time placement with
claim 2's standing position and its one draw R(0, 3). At intake 71, M-0001's CAOS against E1
went from 0/11/1 (row / column / neither wins) to 0/4/8; stock is 0/12/0. Eight games now run
to the time limit.

Two runs of M-0001 on uncommitted copies of main (M8 seam trace) separate the causes:

- *hunt* made current at placement, all else as claim 8 gives: 0/4/8, unchanged.
- infantry at the cell centre with no standing-position draw, all else as claim 8 gives: 0/11/1,
  as before intake 71.

So, as in Q-2277, the standing positions decide the games in our engine. What follows from them
is traced and follows the approved specs:

- The drones' gas sends the E1 berserk (S-0610 claim 4). The berserk E1 kill each other
  (S-0309's friendly-row exceptions) faster than the E1 that are not berserk kill the drones.
  The drones die more slowly than before: in seed 0 with side A as the row, six of nine are dead
  by frame 490, where before intake 71 all nine were dead by frame 351.
- Each time-limit game ends with one E1 left beside two to seven drones. It hunts with no target
  and never fires at them again (S-0610 claim 6). The drones that attack it gas it every few
  ticks, and each re-hit resets its remaining berserk time (S-0610 claim 4). The gas does not
  change its health, so it stays berserk until the time limit.

Stock wins all twelve games for E1, so in the records the E1 kill the drones before that end
state arises. If the records' harness put its infantry somewhere else, the arena needs another
rule for its infantry, and Q-2277's answer would settle this pairing as well.

## Questions

1. In M-0001's CAOS against E1 games, at tick 0, did each E1 stand at its cell's centre point or
   at one of the cell's other standing positions? For a cell holding more than one E1 (a unit
   moved off its stated cell by claim 8's rule), which positions did they take?
2. Did placing each E1 take a scenario-stream draw, as claim 2's R(0, 3) does?
3. In those games, by which frame were all the drones destroyed, and how many E1 had gone
   berserk by then?

## Possible observation

Record M-0001's CAOS against E1 at seed 0, both sides. At tick 0, record each E1's position
within its cell and the scenario-stream draws taken from the first unit's creation to the first
update. Then record the frame of each drone's destruction and of each E1's first berserk hit.

## Current workaround

As built at intake 71: S-0216 claim 8, with infantry on claim 2's drawn standing positions.
CAOS against E1 counts as a failing pairing, beside YURI against GGI and SHK (Q-2277).

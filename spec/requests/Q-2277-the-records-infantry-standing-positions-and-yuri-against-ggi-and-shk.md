---
id: Q-2277
title: Where did the records' infantry stand in their cells at tick 0, given that YURI against GGI and SHK turns on it in our arena?
status: open
related: [S-0216, S-0208, M-0002, M-0006]
answered-by: [M-0019, S-0216, M-0020]
---

## Need

S-0216 revision 37 claim 8 gives our arena's set-up: each infantry unit is placed by a play-time
placement with claim 2's standing position and its one draw R(0, 3). The claim expects a
difference from the records' harness to show "only as a failed acceptance check". At intake 71,
two pairings of M-0002 that passed before fail the win test instead:

| Pairing | Stock (row wins) | Ours before intake 71 | Ours after |
|---|---|---|---|
| YURI against GGI | 12 of 12 | 12 of 12 | 3 of 12 (p = 0.0001) |
| YURI against SHK | 12 of 12 | 12 of 12 | 4 of 12 (p = 0.0005) |

Before intake 71 our arena put every infantry unit at its cell's centre point with no
standing-position draw (A1's interim, Q-2097). Two runs of M-0002 on an uncommitted copy of the
intake 71 code separate the causes:

- *hunt* forced current at placement, as before, with all else as claim 8 gives: the same 3 of 12
  and 4 of 12. The queued *hunt* is not the cause.
- infantry put at the cell centre with no standing-position draw, as before, with all else as claim 8
  gives: YURI against GGI 11 of 12 and YURI against SHK 12 of 12.

Neither GGI nor SHK has more than one unit moved off its stated cell, and YURI has none. So the
standing positions decide these games in our engine. Both are short games, over at 304 to 912
frames. We did not trace why the positions decide them.

Our arena follows claim 8 and stays as built. If the records' harness put its infantry
somewhere else, the arena would need another rule for its infantry. If it used the same
positions, the cause is a difference in play to look for elsewhere.

## Questions

1. In the records' games, at tick 0, did each infantry unit stand at its cell's centre point or
   at one of the cell's other standing positions? For several infantry units on one cell (M-0006
   records cells with more than one unit), which positions did they take?
2. Did placing each infantry unit take a scenario-stream draw, as claim 2's R(0, 3) does?
3. In a stock game set up as M-0002's YURI against GGI, with the GGI on side B, does moving the
   side-A YURI units between the centre point and the other standing positions of their cells
   change which side wins?

## Possible observation

Record M-0002's YURI against GGI and YURI against SHK games at seed 0, both sides. At tick 0,
record each infantry unit's position within its cell (centre or which side position) and the
scenario-stream draws taken from the first unit's creation to the first update. Then record each
side's units' first target, and the tick of the first mind-control capture.

## Current workaround

As built at intake 71: S-0216 claim 8, with infantry on claim 2's drawn standing positions.
YURI against GGI and YURI against SHK are counted as failing pairings
([intake 71 ledger](../../docs/m8-claims/intake71-S-0513-S-0216.md), Lanes).

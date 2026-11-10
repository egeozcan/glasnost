---
id: Q-2113
title: A lifter whose gripped unit comes inside its weapon's minimum range
status: answered
related: [S-0607, S-0215, S-0307, M-0015, M-0001]
answered-by: [S-0607, S-0215]
---

## Need
M-0001's TELE against BFRT pairing fails on side A: the stock games end at the tick limit with
units of both sides left on every seed (M-0015 for seeds 0 and 1), ours end that way on 16 of 64
seeds; the rest are mostly row wins (seed 0: TELE wins at tick 3344).

In our games every hold ends the same way, by these approved rules:

1. The gripped fortress is pulled towards its Magnetron (S-0607 claims 7 and 8). Before it
   arrives, its 3-D distance falls below the minimum range of the Magnetron's weapon.
2. At the Magnetron's next *hunt* processing, the approach step's range test (S-0215 claim 37 b:
   S-0307 claims 1 to 12, minimum range included, claim 9) says the target is out of range. The
   firing check would answer *rearming* (S-0607 claim 11), but part b does not use it.
3. The hunter keeps its target (part d) and searches an approach position (part f). The search
   (S-0307 claims 15 and 16) takes the greatest qualifying radius, so it finds a cell about the
   weapon's range away from the fortress, on the Magnetron's side of it and beyond the Magnetron.
4. The Magnetron accepts that destination, which ends the grip (S-0607 claim 12, row 2: "an
   approach step"). The fortress is dropped (claim 13).

So our Magnetrons back away from the fortress at every hold. The plateau of the arena's W is
narrow, and over the game they end up on the lower ground beside it. There, three fortresses of
seed 0 are released over cells with a tree or with rock land. The tree survives claim 16's hit
(S-0302 claim 20: the crushing warhead has no wood flag), claim 17 finds the cell blocked, and
claim 18 kills the fortress. A fourth comes down on a Magnetron. M-0015 records no damage event at
a landing other than one landing crush per seed, so the stock fortresses do not come down on such
cells. Ours also start fewer holds (35 and 25 against 56 and 52 in seeds 0 and 1).

Each step above follows an approved claim. We did not find a claim that keeps a lifter from
taking an approach step against its own gripped unit.

## Questions
1. Does a lifter that grips a unit take the approach step of S-0215 claim 37 against that unit,
   as against any target, when the unit comes inside its weapon's minimum range?
2. If it does: is the approach destination the one claims 15 and 16 choose (a cell about the
   weapon's range away from the gripped unit), and does accepting it end the grip as claim 12
   row 2 says?
3. If it does not: what ends a hold in M-0015's games? Possible events are claim 8's arrival, a
   destination from another rule, or claim 12's other rows.
4. Where are the fortresses of M-0015 released and where do they land? Cells may stay local; the
   land type and whether a terrain object stands there are enough.

## Possible observation
In M-0015's two replays, record for each Magnetron the tick and cell of every new destination
during a hold, its distance to its gripped fortress at that tick, and each landing cell's land
type and terrain object.

## Current workaround
None: the code follows S-0215 claim 37 and S-0607 claim 12 as approved, and the pairing keeps
failing. Nothing was changed for it.

---
id: Q-2546
title: At which point of a tick M-0021 reads its first-shot distances, and whether G's vehicles took their turns before F's
status: open
related: [S-0203, S-0310, S-0307, S-0215, S-0207]
answered-by: []
---

## Need

M-0021's first-shot row (Procedure step 6: "the centre distance from its firer to its target at
that tick") gives 1527 leptons for F and 1280 for G. Our arena harness reads it from the per-tick
positions taken after each tick has run, the convention that M-0004, M-0011 and M-0015 state
("a per-unit state stamped tick *f* is read after tick *f*'s logic has run"). M-0021 states no
convention. With that reading ours gives 1504 for F and 1280 for G in all 16 runs, so the F row
fails (M8 seam trace 5; Q-2536 asked for the record's distances and stays open).

A trace of our runs (the same in all 16) shows the cause: a reading point, not a missed shot.

- **F's first firer.** It reaches a cell centre and stops (S-0207 claim 19 (a): its target's
  cell centre is 1536 leptons away, exactly F's range). Its firing step comes later in the same
  turn (S-0310 claim 9). F's 16 vehicles come before G's in the update order (S-0203 claim 3,
  with Q-2516's creation order: F's group first), so G has not moved yet in that tick. The step
  measures 1551 to the target's reference point, which is beyond 1536 (S-0307 claim 9), and F
  does not fire. G then moves, and the tick ends at 1527. In the next tick the firing step
  measures 1527 and F fires. That tick ends at 1504 after G's move.
- **G's first firer.** It stopped a few ticks earlier at a cell centre with its target's cell
  centre at 1280 (claim 19 (a) again), while its target, an F vehicle, was still closing. In the
  tick of G's first shot that F vehicle takes its turn first and comes from 1291 to 1280. G's
  firing step then measures 1280 and fires. That tick ends at 1280.

So, in ours, the three possible readings give:

| Reading of the step 6 distance | F's first shot | G's first shot | Matches the record |
|---|---|---|---|
| per-tick positions after the shot's tick has run | 1504 | 1280 | G only |
| per-tick positions before the shot's tick runs (after the tick before) | 1527 | 1291 | F only |
| at the shot, as the firing step measures it | 1527 | 1280 | both |

In each run, each side's first shot fires in the first tick in which that firing step's range
test passes. We find no rule of S-0301, S-0307, S-0309, S-0310 or S-0215 that ours breaks there.
The rearm countdown has not started, the turret is not turning, the firer has no destination
after its stop, and the target is held from early in the approach.

The record's values fit the per-tick reading after the tick only if the original differs from
ours in one of two ways. One is that G's group took its turns before F's group. Then F's firing
step sees G's move of the same tick (1527) and G's sees the F vehicle before its move, so G fires
one tick later (1280, with the target stopped). The other is that F's firing step measured
something other than the target's reference point.

## Questions

1. At which point of a tick are M-0021's per-tick positions read: after the tick's logic has
   run (the convention of M-0004, M-0011 and M-0015), or before it? Is the first-shot distance of
   Procedure step 6 computed from those per-tick positions, or read at the shot itself, for
   example by the recorder at the fire event?
2. If step 6 uses positions read after the shot's tick has run: in the record's runs, did the
   vehicles of G's group take their turns before those of F's group in a tick (S-0203 claim 3),
   although F's group was created first (Q-2516)? If not, what distance did F's firing step
   measure in the tick of F's first shot?
3. Under the answer to question 1, which tick of the record is F's first shot: the tick at whose
   end (or start) the record reads 1527?

## Possible observation

The recorder's own definition answers question 1. Question 2 is read from any part A run: the
positions of F's first firer and of its target at the two ticks before F's first shot and at
the shot's tick, and those of G's first firer and its target at the same three ticks for G's
first shot. Q-2536 question 1 asks for F's part of these. This request asks how to read them, and
adds G's.

## Current workaround

None in the simulation: no rule changes. The harness keeps reading step 6 from the per-tick
positions after each tick, as the other records state, and shows the F row failing with ours at
1504. Should the answer be "at the shot", only the harness's reading changes. Should it be the
update order, the request goes to S-0203 and S-0216 claim 8.

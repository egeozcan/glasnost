---
id: Q-2536
title: When M-0021's F vehicle first fires after its target comes into range, and why part B's outcome varies with the seed
status: open
related: [S-0307, S-0215, S-0309, S-0413]
answered-by: []
---

## Need

With Q-2516 answered (M8 step A3 intake 95), the arena's M-0021 harness plays the record's setup
as the record states it: the creation order, G's numbered weapons, the observer with one building
and no vehicle, and the run's end at tick 608. The same 22 of 24 rows pass and the same 2 fail:

- F's first-shot centre distance is 1504 leptons in every run of ours, the record's is 1527.
  G's is 1280 in both.
- Part B's G-dead counts are 4, 4, 4 (F on side A, seeds 1000 to 1002) and 7, 7, 7 (F on side B)
  in ours, the record's are 4, 4, 7 and 7, 6, 7. Ours does not change with the seed at all.

Our F fires at tick 34 and G at tick 44. The two groups close at about 22 leptons a tick in ours,
so the tick before our first F shot the distance was already about 1526, inside F's range of 1536.
The record's 1527 is also inside the range. Whether ours fires one tick after its target is first
in range, or the closing speed differs (the countries' speed factors are kept `local`), the record
as written cannot say. The choice of tank type does not matter: a different base tank gives the
same numbers.

## Questions

1. In the record's runs, what was the centre distance from F's firer to its target at the tick
   before its first shot, and at the tick of its first shot? Was the target within F's weapon
   range (S-0307 claim 1) already at the tick before? (The record has these positions in its
   per-tick rows.)
2. How far did one F vehicle and one G vehicle move between ticks 20 and 30 of a part A run
   (centre positions at both ticks)? This gives the closing speed with the countries' factors
   applied, without stating the factors.
3. Part B varies with the seed in the record (G dead 4, 4, 7 with F on side A). What draws or
   choices that depend on the game seed change which vehicles fire, are hit or die in a part B
   run, and are they in a spec? Does the recording show a difference between the seeds 1000, 1001
   and 1002 before the first death (a shot's tick, target or a hit's health)?

## Possible observation

Questions 1 and 2 are read from the record's recording of any part A run. Question 3 is the first
tick at which two part B runs of one side differ.

## Current workaround

None needed: the harness shows both rows with the record's values and ours, and counts them as
failing. No simulation rule was changed. The two rows stay failing until the answers say whether
the cause is the setup (speed factors) or a rule (the tick of a first shot, a seed-dependent
choice).

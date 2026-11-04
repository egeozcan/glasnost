---
id: Q-2597
title: Whether every vehicle of M-0021 had fired its first shot before the seeds first differ
status: open
related: [S-0301, S-0307]
answered-by: []
---

## Need

M-0021 (revised for Q-2536 and Q-2546), Result, *Seeds*, lists what agrees in the three seeds
before the first difference as "every shot's tick, firer, target and weapon, every hit, and every
per-tick row, including every vehicle's first shot". Under the table it says the first difference
falls "after each side's first shots and before any vehicle's death". The two statements can be
read two ways:

- (a) all 32 vehicles had fired their first shot before the first difference (which is the tick of
  some G vehicles' second shots);
- (b) each vehicle's first shot is the same in the three seeds wherever it falls, and only each
  side's first shot precedes the first difference.

In our runs the seeds first differ at a G vehicle's second shot, after each side's first shots and
before any death, in the same tick in parts A and B on both sides, as the record's rows say. Every
F vehicle has fired by then, but some G vehicles fire their first shot after it. Reading (b) holds
in every run of ours; reading (a) does not. The record's own part B row has every G vehicle reach stage 1 54 to 182 ticks before
the first death, so its slowest G vehicle also fires its first shot late, which fits (b) but does
not show (a) false.

## Questions

1. In the record's runs, at the tick of the first difference between the seeds, how many F vehicles
   and how many G vehicles had fired at least one shot (a count, not a tick)?
2. Is every vehicle's first shot (tick, firer, target, weapon) the same in the three seeds, also
   for a vehicle that fires it after the first difference?

## Possible observation

Both are read from the shot rows of the three seeds of any part A run.

## Current workaround

None in the simulation. The harness reports reading (a) as the failing row "every vehicle's first
shot falls before the first difference", and reading (b) as its own passing row beside it. The
failing row stays until the answer says which reading the record means.

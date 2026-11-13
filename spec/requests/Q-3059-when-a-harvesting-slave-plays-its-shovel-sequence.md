---
id: Q-3059
title: When does a harvesting slave start its shovel sequence, and which slave states count as harvesting for its loop?
status: open
related: [S-0107, S-0402, S-0401]
answered-by: []
---

## Need

S-0107 claim 21 lists *shovel* among the kinds "started by mechanics that other specs own, which
also say when", with harvesting as its mechanic. Its completion table gives *shovel* followed by
*shovel* (forced), which loops, while "the unit is harvesting", and by *stand* (forced)
otherwise. Claim 9 keeps the facing on a completed *shovel*.

S-0402 claim 5a says that the slaves' own harvesting sequence belongs to S-0107, and S-0402's
slave rules (claim 14, **Slaves**) give the slave's search, walk, loading interval *h* and return,
but no sequence start. No spec other than S-0107 names *shovel*. So the engine has no rule for
the tick on which a slave starts *shovel*, and no definition of "the unit is harvesting" for the
completion row.

GitHub issue #422 reports that slaves gather with no digging motion. The engine has the
sequence's row and its facing exception, but nothing requests it.

## Questions

1. On which tick does a slave first start *shovel*? Some candidates: at the processing of its
   *harvest* at which it arrives on the cell it found; at its first bale; at the first processing
   at which it would load (successful or not); or something else.
2. Is the start forced, as the loop row is?
3. When a slave loads a bale while *shovel* is already playing, is *shovel* started again
   (restarted), or left to run?
4. For the completion row, which slave states count as "the unit is harvesting"? For example:
   any slave whose behaviour is *harvest*; only one standing on a tiberium cell with free capacity;
   only one between its first and its last bale on the current cell. Does a slave that has just
   loaded its last bale (now full), or whose cell has just run out, count while its sequence is
   still playing?
5. Does anything else stop *shovel* before it completes, for example a new order, a walk away
   from the cell, or the slave being picked up by the miner?
6. Does a harvester of another kind that is an infantry type (not a slave) play *shovel* under the
   same rules, or only slaves?

## Possible observation

S-0107 scenarios M7(c) and M17(a) already name the harvesting starts of *shovel* as measurable.
A slave miner deployed beside a small field of one tiberium type, with the recorder logging per
tick each slave's sequence kind and stage, its cell, its behaviour, its load and the cell's
remaining amount, answers questions 1 to 5. A field cell holding fewer bales than the slave's
capacity shows the end on an exhausted cell. A slave given a move order mid-sequence shows
question 5.

## Current workaround

None: no code starts *shovel*. Slaves keep their ordinary sequences while they gather, as before
this request. The facing exception of claim 9 and the sequence row are already in the engine and
are unaffected.

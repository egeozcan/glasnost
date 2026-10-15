---
id: Q-0729
title: Cloak field growth past buildings, and the generator restart distance
status: open
related: [S-0516]
answered-by: []
---

## Need

S-0516 claim 20 leaves three points open:

- which cells miss coverage when a growing ring meets an own building's centre;
- which distance the restart rule ("closer than 2 (r + 2) cells") measures;
- whether a removed generator, not just a shrinking one, also restarts its neighbours.

The code in `crates/deterministic/sim/src/cloak/field.rs` (`turn`, `restart_neighbours`, `vanish`)
needs the rules.

## Questions

1. When a growing ring covers an own building's centre cell, which cells of the ring are not
   covered? Are they covered later?
2. How is the restart distance measured? Is it:
   - the straight-line distance between centre cells;
   - the larger of the x and y differences; or
   - the distance in leptons between foundation centres?
3. When a generator is sold or destroyed, its field vanishes at once. Does it restart its
   neighbours as one that shrank to 0 does?

## Possible observation

- Two powered generators of one house, with centres at measured distances around 2 (r + 2) cells.
  Switch one off, then sell the other.
- A generator ringed by own buildings at radius 3.

## Current workaround

- Rings are applied in full, whatever stands on them.
- The restart distance is the straight-line distance between the generators' centre cells.
- A removed generator restarts its neighbours as one that shrank to 0 does. A captured one does
  not.

These choices are marked in the code with this request's ID.

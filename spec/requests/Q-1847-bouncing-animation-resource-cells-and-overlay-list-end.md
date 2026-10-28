---
id: Q-1847
title: A bouncing animation's new resource cells, the resource lists and the end of the overlay list
status: answered
related: [S-0319, S-0401, S-0202, Q-1762]
answered-by: [S-0319]
---

## Need
S-0319 (revision 9) claim 10 gives each germinable offered cell an overlay that stands R(0, 3)
places after the type's resource spawn overlay type in the overlay list, at the stage R(0, 2).
Two points are not covered, and `glasnost-sim` `bounce::offer` needs a rule for each:

- S-0319's Open questions leave open how such a cell enters S-0401's resource lists (growth and
  spread), and so whether it takes S-0401 claim 11's growth-list raw draw and where among its
  draws that draw falls (claim 13).
- No spec says what happens when the spawn overlay type's place plus the drawn number lies past
  the end of the overlay list, for example a spawn overlay type that is the list's last or
  second-last entry. A map cell's overlay byte also cannot hold every index (S-0202's "no
  overlay" value), so a place at or past it has the same question.

## Questions
1. Does a cell that receives resource from a bouncing animation enter the growth list, the
   spread list, both or neither? If it enters the growth list, does it take the growth-list raw
   draw at that moment, and is that draw before or after the stage draw R(0, 2)?
2. When the place R(0, 3) after the spawn overlay type lies past the end of the overlay list, does
   the cell receive any overlay (the last one, none, something else), does the stage draw still
   follow, and is the cell then still germinable for a later offer in the same landing?
3. Does the same hold for a place at or past the index a cell's overlay byte uses for "no
   overlay", when the list itself is that long?

## Possible observation
S-0319's scenario BA10 part b with a resource spawn overlay type set to the last and to the
second-last entry of a custom overlay list, logging each offered cell's overlay, stage and the
draws per landing; and BA11 part b′ with S-0401's growth-list draws logged in the ticks after the
landing.

## Current workaround
Each interim cites Q-1847 in `crates/deterministic/sim/src/bounce.rs`:
- the new resource cell is not offered to S-0401's growth or spread lists and takes no
  growth-list draw;
- a place past the overlay list, or at or past the "no overlay" index, places nothing; the
  overlay draw and the stage draw are still taken and the stage is not written. The cell stays
  germinable, but it takes no further offer in the same landing. (Read literally, F5 would offer
  it again for every further pair that names it, with two draws each time; under
  `quirks.anim_resource_spread_neighbours_only` up to 2R + 1 pairs of one i name the same cell,
  and every eighth i names it again, so the interim bars it after one offer. With the switch off each pair names a
  different cell, so this changes nothing there.)

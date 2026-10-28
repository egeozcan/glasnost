---
id: Q-1771
title: Does a unit at a factory's exit find its way out through a built-up base, or give up after 5 corridor searches?
status: answered
related: [S-0212, S-0209, Q-1280]
answered-by: [S-0212]
---

## Need

The lead's first pathing report (M8 step A4, play note 1, docs/m8-claims/a4-play-notes.md,
"Pathing report 1") shows two infantry units of an engineer type that would not walk to a building
they were ordered to capture. This request takes up the question held as Q-1752 for play note 1;
Q-1752 itself was never written.

`glasnost_sim::path` (`corridor_search`) and `glasnost_sim::corridor` (`coarse_why`, `retry`)
follow S-0212 claims 13, 13a and 13b as written, and the report's case follows from them. The case,
in cells only:

- Each unit leaves a factory whose foundation covers (141..143, 69..70) and stands at its exit cell
  (142, 71), just south of it. The ground round the factory is open. North of it, between the exit
  and the targets, other buildings of the same base stand in short rows across x 135 to 142 at
  y 60 to 64, with open ground east of them.
- From (142, 71), every order to a cell about 15 cells north, (136..139, 54..57), fails: 19 path
  computations in all, at ticks 8897 to 10465. Each makes 5 cell searches inside a corridor that
  expand 9 to 26 pairs each and find nothing; the 5th failure fails the computation (claim 13's
  table), the unit gives up, its destination is cleared and it rests in *guard* (claim 23 row 2).
  Orders to (147, 69), 5 cells east, succeed from the same cell. Once moved to (145, 67), the same
  unit's capture order succeeds with a 19-step path round the east ends of the rows.
- Why, by the claims: the coarse searches see through buildings (claim 1), so the first level-0
  corridor runs straight north through the factory's own cells. The free part of the exit's 2 × 2
  sector is the exit cell and its east neighbour, the probe cell lies outside the sector, so it is
  *whole* (claim 13a) and its connections north are closed. The level-1 and level-2 corridors
  still hold the factory's block, so the next corridors run round the factory's west side into the
  next row of buildings, and so on. The level-2 corridor takes in the blocks east of the factory
  (x 144 and beyond) only at the 5th search, whose level-0 corridor crosses the factory's corner
  diagonally (claim 13b counts that pair of building cells as a connection), and it fails too.
- In a diagnostic build that allowed more than 5 cell searches, the 7th search inside a corridor
  found a path of 17 to 19 steps. With `quirks.corridor_retry_no_fallback` off, or
  `quirks.corridor_split_probe` off, the computations still fail: the first switch acts only when a
  repeated coarse search fails, and here no coarse search fails; the cap of 5 ends the computation.
- A synthetic layout of our own reproduces the shape (`claims_s0212_cap.rs`): a 3 × 2 block of
  1 × 1 buildings, an infantry unit at its south exit, a row of ten 1 × 1 buildings 4 cells north of
  the block's top with open ground round both ends, the destination 16 cells north and 3 west of the exit. The
  move order fails after 5 cell searches inside corridors and the destination is cleared at once,
  with the switch on or off, although a cell search without a corridor finds the way round.

The Edge case "A short line of buildings across the route" (Q-1280) covers the failed repeated
coarse search and gives it a switch because a unit refuses an order although an open way round
exists. This case has the same impact through the cap, which no switch covers. P12's expectation
says that long walls make the unit give up; a factory and a base laid out in rows are not walls, and
the lead saw it in ordinary play.

## Questions

1. In the original, an infantry unit has just left an infantry factory and stands at the factory's
   exit. It is ordered to a cell about 15 cells away on the far side of the factory, across a base
   whose buildings stand in short rows with open ground round their ends. Does it walk there, and
   by which side? Does a vehicle at a vehicle factory's exit behave the same?
2. If it walks there, which part of claim 13 differs from the text? For example: more than 5 cell
   searches; one cell search without a corridor after the 5th failed one (as
   `quirks.corridor_retry_no_fallback` off gives after a failed repeated coarse search); building
   cells cutting connections between sectors or making a sector split; or other closures.
3. If it gives up as the claims say: since the failure has gameplay impact, should a switch let our
   engine lift it, for example one cell search without a corridor after the 5th failed one? Claim 13
   now counts the extra search of the switch's off case towards the 5, so it cannot follow the 5th
   search; the rule for that search (its limits, whether it is the 6th) would need stating.

## Possible observation

A flat test map: a 3 × 2 block of 1 × 1 buildings of the human house, an infantry unit at the
cell south of its middle, a row of ten 1 × 1 buildings 4 cells north of the block's top, its west
end 5 cells west of the block, its east end 2 cells east of it, and open ground round both ends.
Order the unit to the cell 16 cells north and 3 west of its own. Repeat with the block and the row shifted by
0 to 7 cells east and north against the sector grid (claim 13b), and with a vehicle. Observable:
whether the unit moves at once or its destination is cleared in the tick after the order, and the
cells it enters. A second run: build an infantry factory and three rows of 1 × 1 buildings north of
it, and order a fresh unit at the exit to a cell 15 cells north.

## Current workaround

S-0212 as written: the 5th failed cell search inside a corridor fails the path computation and the
unit gives up. The tests in `crates/deterministic/sim/tests/claims_s0212_cap.rs` pin this. No
switch is added until this is answered.

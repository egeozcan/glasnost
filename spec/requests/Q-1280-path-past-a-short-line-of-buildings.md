---
id: Q-1280
title: Does a unit find its way past a short line of buildings across its route?
status: open
related: [S-0212, S-0603, S-0605]
answered-by: []
---

## Need
`glasnost_sim::path` (the corridor search, S-0212 claims 13 and 13a) and
`glasnost_sim::corridor::retry` follow S-0212 as written. On open synthetic
ground this has a consequence that S-0212 does not discuss and that decides
whether a unit sent from afar reaches a recycling building (S-0603 claims 3
and 4), an absorbing building (S-0605) or any cell inside a base.

The case: a straight north–south line of seven 1 × 1 buildings, with open
ground all round it. A vehicle 20 cells west of the line is ordered to a cell
2 cells east of it (a move order, or the recycle order to a 2 × 2 recycling
building just behind the line). Under S-0212 as written, the
path computation fails on the first try:
- the first cell search inside the corridor fails at the line (the coarse
  search sees through buildings, claim 1);
- the retry (claim 13's table, claim 13a) finds the level-0 and level-1
  sectors of the progress cell *whole*, then *split*, and closes connections.
  At level 2 the free part borders the next sector, so nothing is closed
  there and the level-2 corridor stays as it was;
- the second cell search fails the same way. After the second retry the new
  coarse search fails at level 1, because the remaining way round lies
  outside the unchanged level-2 corridor (claim 13: below level 2 a sector can
  be on the corridor only when its parent is). The table's row "new coarse
  search, it fails at any level: the path computation fails" then applies.

So the unit stops at once and its destination is cleared (claim 23). Lines of
three or five buildings are passed. An infantry unit given the same move
fails alike. Edge cases in S-0212 covers a destination *enclosed* by buildings ("up to 5
cell searches, before it fails"), but not an open way round a short line.

## Questions
1. A vehicle 20 cells from a straight line of seven small buildings, on open
   ground, is ordered to a cell just behind the line. Does it drive round the
   line and arrive, or does it stop at once without moving?
2. The same with an infantry unit, and with the recycle order to a recycling
   building just behind the line.
3. If it arrives, which step of claim 13 differs from the text? For example:
   is a failed repeated coarse search followed by a cell search without a
   corridor, or is the level-2 corridor searched again?

## Possible observation
On a flat test map, build a north–south line of seven 1 × 1 buildings (or
walls of a type that counts as a building) and order a vehicle placed 20
cells west of the line to a cell 2 cells east of its middle. Watch whether
it moves. Repeat with lines of three, five and nine.

## Current workaround
The engine keeps S-0212 as written: the order fails, and the test
`claims_3_4_approach_past_a_line_of_buildings` (crates/deterministic/sim/tests/claims_s0603.rs)
pins this. Our own computer player first moves a unit that is far from a
recycling building to a cell beside it, and sends it in from there.

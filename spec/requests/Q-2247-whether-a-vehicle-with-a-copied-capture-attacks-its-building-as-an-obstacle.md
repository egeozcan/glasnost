---
id: Q-2247
title: Whether a vehicle with a copied capture attacks its destination building as an enemy obstacle
status: open
related: [S-0213, S-0207, S-0209, S-0215]
answered-by: []
---

## Need
S-0213 r89 claim 16b (*Joins*, Q-2227 question 2) and Scenario O8 part ac
say that a vehicle which issues an engineer's copied *capture* on an enemy
building X gets *capture* with X as its destination and no target, heads for
X, and does not attack X, since the order names no target. Part ac is
falsified if the vehicle fires at X, or X loses health, while the engineer is
still on its way to F.

Two other specs, applied as written, make the vehicle attack X once it
reaches it:
- S-0209 claim 4 gives a driving vehicle the outcome *enemy obstacle* for a
  cell holding an enemy building (row VP28). No vehicle row exempts the
  building that is the vehicle's destination, or its mission target, as row
  WP3 does for a walker; VP5, VP6 and VP17 exempt only *enter* and *eaten*
  destinations.
- S-0207 claim 15, table T4: at a repeated check with the outcome *enemy
  obstacle*, the vehicle takes the first listed object of the cell as its
  target and attacks it by an override (S-0215 claim 10), with its
  destination put aside.

In our engine (synthetic test, `claims_s0213_r89.rs`), the vehicle drives
to a cell next to X. Its next step is a cell of X's foundation. Seven ticks
later the repeated check of T4 gives it *attack* with X as its target and no
destination (X put aside), so it is under *attack* on X. The engineer has
not yet reached F.

The engine must decide which rule holds when a vehicle in *capture* (or
*sabotage*) has an enemy building as its destination and its next step is a
cell of that building.

## Questions
1. A vehicle has *capture* current, an enemy building X as its destination
   and no target, and drives up to X so that its next step enters a cell of
   X's foundation. Does it take X as its target and attack it, as S-0207
   table T4 gives for an enemy obstacle? Or does that cell give it another
   outcome (for example *blocked*, or none), so that it stays in *capture*
   and never fires?
2. If it does not attack: where does it stop, and what does it do while
   *capture* stays current (stand still next to X, give up the move, or
   keep trying the step)?
3. Is the answer the same for a vehicle in *sabotage* with an enemy building
   as its destination?

## Possible observation
Scenario O8 part ac as written, with the record continued every tick after
T reaches X, until the engineer reaches F: T's behaviour, target and
destination, its cell, every shot of T's, and X's health.

## Current workaround
None needed in code: the engine follows S-0207 claim 15 table T4 and S-0209
row VP28, so the vehicle attacks X by an override. The test of part ac's full
window, `claim_16b_a_vehicle_with_a_copied_capture_never_attacks_x`, is
`#[ignore = "Q-2247"]`. A second test checks part ac up to the tick in which T
stands next to X: `claim_16b_a_vehicle_issues_a_copied_capture_and_heads_for_x`.

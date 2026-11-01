---
id: Q-2228
title: Whether a turreted building under move takes a later order
status: open
related: [S-0213, S-0215]
answered-by: []
---

## Need
S-0213 r84 claim 16d: a building that holds a *move* route point does
nothing under *move* and issues no further point. Scenario O8 part x (7)
then expects that the player, without the route-mode key, gives the building
an *attack* order on an enemy, and that the building fires at it.

S-0215 claim 11 makes a building ready to change only through its ready
mark. For a turreted building the idle body range sets no mark (claim 11),
*move* has no processing rule for a building (claim 4 and table T1), so
nothing that our engine models sets the mark while a turreted building is
under *move*. The *attack* order is then queued and never becomes current.
A building without the turret flag gets the mark from its idle body range,
so its queued *attack* starts.

The two specs may agree (part x's building has no turret) or disagree (a
turreted building does fire in part x). The engine needs to know which.

## Questions
1. A turreted building with a weapon is made to issue a *move* route point
   (part x (7)). The player then gives it an *attack* order on an enemy in
   range. Does it fire? After how long?
2. If it fires, what sets it ready to change while *move* is current: some
   update under *move*, the order itself, or a rule other than the ready
   mark?

## Possible observation
S-0213 Scenario O8 part x (7) with a turreted defensive building, then the
same with a defensive building without a turret, recording the tick at which
each first fires after the *attack* order.

## Current workaround
The engine follows S-0215 claim 11 as written: a turreted building under
*move* keeps the queued *attack* and does not fire. The claim test
`claim_16d_a_building_under_move_does_nothing`
(`crates/deterministic/sim/tests/claims_s0213_r84.rs`) uses a building
without a turret, marked `design (Q-2228)`.

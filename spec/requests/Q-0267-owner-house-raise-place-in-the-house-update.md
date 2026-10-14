---
id: Q-0267
title: Where in a house's own update the every-eighth-tick raise of its tags comes
status: answered
related: [S-0011, S-0203, S-0405, S-0407, S-0410, S-0411, S-0414]
answered-by: [S-0414]
---

## Need
S-0011 claim 35 puts the raise of a house's assigned tags "as part of its own per-tick update near
the end of that tick's game logic", after the objects' updates. S-0203 row 7 and S-0414 claim 16 order
the other topics of one house's update (power, radar recheck, the defeat test and surrender
destruction, the tech-tree recheck, the automatic release order). Our engine must place the raise
among them. The order is observable: a raise whose trigger tests a type count, or changes a variable
that another house's tag or a later topic reads, sees or does not see the destruction of a
surrendering house's objects and the defeat test of the same tick.

## Questions
1. In a tick whose number is a multiple of 8, does a house raise its tags before or after its own
   defeat test and the destruction of a surrendering house's objects (S-0414 claim 16)?
2. Is the raise before or after the house's power and radar update, and before or after its
   tech-tree recheck (S-0407 claim 12)?
3. Does a house removed by the unused-house cleanup, or a defeated house, still take this place in
   house order (claim 35 says a defeated house keeps raising)?

## Possible observation
A map with a repeating any-event trigger owned by the first player's country whose condition is
type-does-not-exist of that house's last building type, with an action that sets a variable; a
second tag tests the variable. Destroy the building in a tick that is a multiple of 8 and record in
which tick each trigger fires.

## Current workaround
The raise comes last in the house's update, after the automatic release order (crates/deterministic/sim/src/econ/houses.rs,
`triggers::house_raise`). Marked in docs/development.md, section *Map triggers (M5b)*.

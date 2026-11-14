---
id: Q-3101
title: Which nearby cell a hover-attack type takes as its destination in attack and guard
status: open
related: [S-0215]
answered-by: []
---

## Need
S-0215 claim 23's general rules of *attack* include the hover-attack cell: "the actor's type is a
hover-attack type and its height above the surface is 0, with or without a target | it takes a
nearby cell as its destination". Claim 24's *guard* table has the same rule for a target present
when the processing began. Scenario B24 (d) leaves the cell itself to the movement specs, and no
movement spec gives it. The engine imports the hover-attack flag but does not build the rule
(`process_attack` and `process_guard` in `crates/deterministic/sim/src/process.rs`, issue #234).

## Questions
1. Which cell does the rule take: how is "nearby" chosen (a search around the actor's cell or
   around its target, its radius or ring order, any random draws and their range), and what
   happens when no candidate cell is acceptable?
2. Which tests must a candidate pass (usable cell, passable for the actor's locomotor, not
   occupied, within the weapon's range of the target, not the actor's own cell)?
3. Does the rule run at every processing, also while the actor is already moving to a cell this
   rule gave it, or only when it has no destination? Does it replace a destination the actor
   already holds?
4. Is the destination assigned by the ordinary destination assignment (so that the actor's
   locomotor starts a route), and does the processing then queue anything?
5. Claim 23 says "with or without a target" for *attack*. With no target the fall-back of the
   same table also applies in that processing: which effect comes first, and does the fall-back
   clear the destination the rule just gave?

## Possible observation
S-0215 scenario B24 (d): an armed ground vehicle type given the hover-attack flag, ordered to
attack a stationary enemy building in range, and in another run in *guard* with an enemy
building in range. Recording the destination cell at each processing over several runs, with
the vehicle's cell and the target's cell, would show the choice and any draws.

## Current workaround
The rule is not built: a hover-attack type is processed like any other type in *attack* and
*guard*. The gap is recorded in the module documentation of `process.rs`.

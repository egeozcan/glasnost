---
id: Q-1530
title: What the object that closes a cycle of spawned and slave types keeps
status: answered
related: [S-0611, S-0506, S-0402]
answered-by: [S-0611]
---

## Need
S-0611 F5 (revision 3) says that when a chain of spawned and slave types leads back to a type
already in it, our engine's importer warns and "gives the type that closes the cycle no initial
fill when it is created inside that chain". That settles that the object is created (with its
creation draw and number) and that its slots and slave places are not filled at creation. It
does not say whether that object still gets a spawn controller with its *N* slots (S-0506
claim 1) and a slave group with its *M* places (S-0402 claim 19), only empty.

The two readings differ later in play. Empty slots regenerate under S-0506 claim 9, and a lost
slave place gets a replacement under S-0402 claims 21 and 26. Each regenerated or replacement
object is a new creation outside the chain, so it receives a full initial fill of its own, whose
members include the cycle-closing type again. With a controller and group kept, the number of
objects can keep growing over the game; without them, the cycle-closing object never has slot
objects or slaves.

The code is `glasnost_sim::objects::fill` (`crates/deterministic/sim/src/objects.rs`).

## Questions
1. Does the object that closes the cycle get its spawn controller (with *N* empty slots that
   then regenerate as usual) and its slave group (with *M* places that then get replacements as
   usual), or neither?
2. If it keeps them: should the importer clamp anything to prevent unbounded growth, or is the
   growth the intended consequence of bad data?

## Possible observation
None in the original: F5 calls its result undefined, so this is a design choice for our engine.

## Current workaround
Interim: the cycle-closing object gets neither a spawn controller nor a slave group, so nothing
fills it later. It still takes its creation draw and number (S-0611 claim 14), and the importer
warns (`MissileRules::warnings`). Test: `claims_creation_set::s0611_f5_cycles_end`.

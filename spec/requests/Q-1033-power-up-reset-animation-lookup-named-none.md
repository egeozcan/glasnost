---
id: Q-1033
title: How an animation type named NONE can exist for the power-up reset lookup
status: open
related: [S-0520, S-0003, S-0004, S-0006]
answered-by: []
---

## Need

S-0520 revision 3 claim 28 and its Import mapping say that a power-up entry omitted from a layer's
power-up section gets weight 0 and "animation lookup of `NONE` (no animation unless an animation type
of that name exists)", and scenario CR19 asks for "a separately defined" animation with that name that
then appears in the reset result. S-0003 claim 14 says `none` and `<none>` name no type in a type
reference or type list, so a type list entry `NONE` creates no animation type, and S-0004 claim 2
treats the no-type value specially. The importer (`convert::sim` `type_key`) therefore never finds an
animation type called NONE, and the reset always gives no animation.

## Questions

1. Is an animation type named `NONE` reachable at all (for example by a section that is read as a type
   because another reference creates it), and if so how, given S-0003 claim 14?
2. Or is the "unless an animation type of that name exists" qualifier unreachable in our engine, so the
   reset always means no animation?

## Possible observation

CR19's separately defined animation named NONE in a named rules file: does the reset animation appear?

## Current workaround

Interim rule: the reset animation is the importer's lookup of the name NONE among the registered
animation types (`convert::sim::econ::crate_import`), which finds none because `none` names no type.
Noted in `docs/development.md` (section *Simulation (M6c: crates)*).

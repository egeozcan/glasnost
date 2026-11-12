---
id: Q-3000
title: How deep does a chain of death weapons nest inside one killing hit, and what happens past that depth?
status: open
related: [S-0312, S-0303]
answered-by: [S-0312]
---

## Need

S-0312 claims 1 and 6 set off an exploding object's death weapon inside its killing hit, and
S-0303 claim 1 makes the effects of a detonation that a hit sets off come before the outer
detonation's later hits. So a row of exploding objects, each within the death weapon's reach of the
next, dies depth first: the first one's detonation kills the second before it hits anything else,
the second's kills the third, and so on. The spec sets no limit on that nesting.

Our engine resolved it by recursion on the machine's own stack, so a chain of about 40 such
objects aborted the program on a 1 MiB stack and about 310 on an 8 MiB stack, and two machines
with different stacks failed at different lengths (GitHub issue #218). The fix has to give every
machine the same outcome at every length, so it needs a defined rule beyond some depth. A chain of
that length needs a mod or a custom map, but a map with many exploding objects packed together
could reach a few dozen.

## Questions

1. Does the game limit how many death-weapon detonations nest inside one killing hit? If so, at
   what depth, and what does a death weapon set off past it do: go off later in the same hit (at
   which point), go off in a later tick, or not go off?
2. If there is no limit, does a long enough chain stop the game, and at about what length?
3. Is the order of S-0303 claim 1 kept for every depth in a chain of, say, 50 objects?

## Possible observation

A test map lays 60 invented exploding vehicles of one house in a single-file row, each one cell
from the next, with a death weapon whose warhead reaches exactly one cell and kills one. Kill the
first with a direct hit and record, per tick, which vehicles are still alive, and the order of
their death animations within the tick if it can be seen. A second run with 300 vehicles shows
whether the game survives.

## Current workaround

`// nocite: our own input validation (#218)`, in `combat::death::kill`. Death-weapon detonations
nest on the stack to a depth of 16 (`death::DEATH_WEAPON_DEPTH`), in the spec's order. A death
weapon whose object dies deeper than that is put on a list in the world instead, with its weapon,
damage, position, source and house. The outermost death weapon of the chain, once its own
detonation is complete, sets off the listed ones in the order they were listed, each again nesting
to depth 16 and listing deeper ones. So a chain of any length resolves inside the one hit, the
same on every machine; up to depth 16 the order is the spec's, and past it a listed detonation
comes after the outer detonations still in progress. Tests in
`crates/deterministic/sim/tests/death_chain_stack.rs`.
